#!/usr/bin/env bash
# Assina e completa a release de uma versão do Treasury, no Mac do mantenedor, com a YubiKey na
# mão (docs/trust.md, docs/distribution.md). É o único lugar onde uma versão ganha as
# assinaturas que o app instalado exige; o CI só compila.
#
# Uso: scripts/sign-release.sh vX.Y.Z [--notes arquivo.md]
#
# O que faz, nesta ordem, e para no primeiro problema:
#   1. confere que a tag está assinada por uma chave sua (git verify-tag);
#   2. baixa os pacotes do rascunho que o release.yml criou;
#   3. confere a atestação de proveniência de cada arquivo (gh attestation verify): saiu deste
#      repositório, do release.yml, da tag certa, num runner do GitHub;
#   4. assina cada pacote de atualização duas vezes: chave de hardware (ssh-keygen -Y sign,
#      namespace treasury-update@X.Y.Z; a do Mac pede a digital, a YubiKey pede toque e PIN) e
#      minisign (chave decifrada só na memória);
#   5. monta o latest.json com todos os pacotes (macOS, AppImage, .deb, .rpm, NSIS, .msi);
#   6. faz o SHA256SUMS.txt de todos os arquivos e o assina com a chave de hardware (namespace
#      treasury-release), junto com o allowed_signers para quem baixa conferir;
#   7. envia tudo ao rascunho. Publicar continua sendo um passo seu, à mão.
#
# Variáveis: TREASURY_UPDATE_REPO (padrão knuppe/treasury), TREASURY_PRIVATE_DIR (padrão
# ~/Treasury-privado), TREASURY_UPDATE_SK_KEY (padrão: a chave do Mac,
# <privado>/updater/id_ecdsa_sk_rk_treasury-update, ou a <privado>/updater/yubikey-1 se ela não
# existir), TREASURY_AGE_IDENTITY (padrão <privado>/updater/age-identities.txt).
set -euo pipefail

die() { echo "erro: $*" >&2; exit 1; }
step() { printf '\n== %s\n' "$*"; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

tag="${1:-}"
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "uso: scripts/sign-release.sh vX.Y.Z [--notes arquivo.md]"
shift
notes_file=""
while [ $# -gt 0 ]; do
  case "$1" in
    --notes) notes_file="${2:-}"; shift 2 ;;
    *) die "opção desconhecida: $1" ;;
  esac
done
version="${tag#v}"
repo="${TREASURY_UPDATE_REPO:-knuppe/treasury}"
private="${TREASURY_PRIVATE_DIR:-$HOME/Treasury-privado}"
keys_dir="$private/updater"
# shellcheck source=scripts/update-signing.sh
source "$ROOT/scripts/update-signing.sh"
sk="${TREASURY_UPDATE_SK_KEY:-$(hw_default_key "$keys_dir")}"
identity="${TREASURY_AGE_IDENTITY:-$keys_dir/age-identities.txt}"
minisign_age="$keys_dir/treasury-updater.key.age"
hw_keys="apps/desktop/src-tauri/update-keys.pub"
tauri="$ROOT/apps/desktop/node_modules/.bin/tauri"

[ "$(uname -s)" = Darwin ] || die "rode no Mac do mantenedor"
for c in gh git ssh-keygen age python3 shasum; do
  command -v "$c" >/dev/null || die "falta o comando $c"
done
[ -x "$tauri" ] || die "falta a CLI do Tauri (cd apps/desktop && npm ci)"
hw_has_keys "$hw_keys" ||
  die "$hw_keys sem chaves de hardware: o app recusaria a atualização (docs/auto-update.md, \"Criar as chaves\")"
[ -f "$sk" ] || die "falta o arquivo da chave que assina: $sk"
[ -f "$minisign_age" ] || die "falta a chave minisign cifrada: $minisign_age (a chave sem cifra não é aceita aqui)"
[ -f "$identity" ] || die "faltam as identidades do age (Mac e YubiKey): $identity"
if [ -n "$notes_file" ]; then [ -f "$notes_file" ] || die "notas não encontradas: $notes_file"; fi

step "Tag $tag"
git fetch --quiet origin "refs/tags/$tag:refs/tags/$tag" 2>/dev/null || true
git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null || die "a tag $tag não existe aqui"
git verify-tag "$tag" || die "a tag $tag não tem uma assinatura sua válida (git verify-tag)"
app_version="$(sed -n 's/^version = "\(.*\)"/\1/p' <(git show "$tag:apps/desktop/src-tauri/Cargo.toml") | head -1)"
[ "$app_version" = "$version" ] || die "a tag $tag não é a versão do app ($app_version)"
# As chaves que o app desta versão aceita são as do commit da tag, não as da árvore de trabalho.
signers_src="$(mktemp)"
git show "$tag:$hw_keys" >"$signers_src"
hw_has_keys "$signers_src" || die "a versão $version foi compilada sem chaves de hardware"
# A chave que vai assinar tem de ser uma das que o app desta versão aceita.
hw_prepare "$sk" "$signers_src"
echo "assina: $HW_KEY_NAME"

work="dist/$version/release"
step "Pacotes do rascunho ($repo)"
rm -rf "$work"
mkdir -p "$work"
gh release download "$tag" --repo "$repo" --dir "$work" --clobber
# Assinaturas e índices de uma rodada anterior saem: tudo é refeito aqui.
rm -f "$work"/*.sig "$work"/latest.json "$work"/SHA256SUMS* "$work"/allowed_signers
shopt -s nullglob
files=("$work"/*)
[ ${#files[@]} -gt 0 ] || die "o rascunho não tem pacotes"
for f in "${files[@]}"; do
  case "$(basename "$f")" in
    *"$version"*) ;;
    *) die "arquivo de outra versão no rascunho: $(basename "$f")" ;;
  esac
done

step "Atestações de proveniência"
for f in "${files[@]}"; do
  gh attestation verify "$f" --repo "$repo" \
    --signer-workflow "$repo/.github/workflows/release.yml" \
    --source-ref "refs/tags/$tag" --deny-self-hosted-runners >/dev/null ||
    die "a atestação de $(basename "$f") não confere: não assino"
  echo "ok  $(basename "$f")"
done

# Pacote de atualização → entradas do latest.json (o app procura <sistema>-<arch>-<pacote>
# e, sem ela, <sistema>-<arch>).
payloads=()
for f in "${files[@]}"; do
  case "$(basename "$f")" in
    *_aarch64.app.tar.gz | *.AppImage | *.deb | *.rpm | *-setup.exe | *.msi) payloads+=("$f") ;;
  esac
done
[ ${#payloads[@]} -gt 0 ] || die "nenhum pacote de atualização no rascunho"

allowed="$(mktemp)"
hw_allowed_signers "$signers_src" "treasury-update@$version,treasury-release" >"$allowed"

step "Chave minisign (digital no Mac, ou PIN e toque na YubiKey)"
minisign_key="$(age -d -i "$identity" "$minisign_age")" || die "não consegui decifrar a chave minisign"

sign_payload() {
  local f="$1"
  echo "assinando $(basename "$f") com a $HW_KEY_NAME"
  "$HW_SSH_KEYGEN" -Y sign -q -f "$sk" -n "treasury-update@$version" "$f" || die "a $HW_KEY_NAME não assinou $(basename "$f")"
  mv "$f.sig" "$f.yubikey.sig"
  ssh-keygen -Y verify -f "$allowed" -I treasury -n "treasury-update@$version" -s "$f.yubikey.sig" <"$f" >/dev/null ||
    die "a assinatura de hardware de $(basename "$f") não confere com update-keys.pub"
  TAURI_SIGNING_PRIVATE_KEY="$minisign_key" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" \
    "$tauri" signer sign --app-version "$version" "$f" >/dev/null || die "a minisign não assinou $(basename "$f")"
}

step "Assinaturas da atualização (${#payloads[@]} pacotes)"
for f in "${payloads[@]}"; do sign_payload "$f"; done
unset minisign_key

step "latest.json"
notes="Treasury $version"
[ -n "$notes_file" ] && notes="$(cat "$notes_file")"
python3 - "$work" "$version" "$repo" "$tag" "$notes" <<'PY'
import datetime, json, os, sys
work, version, repo, tag, notes = sys.argv[1:6]
targets = {
    ".app.tar.gz": ["darwin-aarch64-app", "darwin-aarch64"],
    ".AppImage": ["linux-x86_64-appimage", "linux-x86_64"],
    ".deb": ["linux-x86_64-deb"],
    ".rpm": ["linux-x86_64-rpm"],
    "-setup.exe": ["windows-x86_64-nsis", "windows-x86_64"],
    ".msi": ["windows-x86_64-msi"],
}
platforms = {}
for name in sorted(os.listdir(work)):
    for suffix, keys in targets.items():
        if name.endswith(suffix):
            path = os.path.join(work, name)
            entry = {
                "signature": open(path + ".sig").read().strip(),
                "url": f"https://github.com/{repo}/releases/download/{tag}/{name}",
                "yubikey_signature": open(path + ".yubikey.sig").read(),
            }
            for k in keys:
                platforms[k] = entry
data = {
    "version": version,
    "notes": notes,
    "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "platforms": platforms,
}
with open(os.path.join(work, "latest.json"), "w") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
print("\n".join(sorted(platforms)))
PY

step "SHA256SUMS.txt ($HW_KEY_NAME)"
cp "$allowed" "$work/allowed_signers"
(cd "$work" && shasum -a 256 $(ls | grep -v '^SHA256SUMS') >SHA256SUMS.txt)
"$HW_SSH_KEYGEN" -Y sign -q -f "$sk" -n treasury-release "$work/SHA256SUMS.txt" || die "a $HW_KEY_NAME não assinou o SHA256SUMS.txt"
ssh-keygen -Y verify -f "$allowed" -I treasury -n treasury-release -s "$work/SHA256SUMS.txt.sig" <"$work/SHA256SUMS.txt" >/dev/null ||
  die "a assinatura do SHA256SUMS.txt não confere"
rm -f "$allowed" "$signers_src"

step "Envio ao rascunho"
# O *.sig já inclui o SHA256SUMS.txt.sig; repetir o arquivo faz o GitHub recusar o envio (422).
gh release upload "$tag" --repo "$repo" --clobber \
  "$work"/*.sig "$work/latest.json" "$work/SHA256SUMS.txt" "$work/allowed_signers"

cat <<EOF

Pronto: $tag assinada e enviada ao rascunho. Falta você:
  1. conferir o rascunho (arquivos, notas, latest.json) no GitHub;
  2. publicar:   gh release edit $tag --repo $repo --draft=false --latest
     (versão de teste: --prerelease no lugar de --latest; o app não se atualiza para ela);
  3. abrir um Treasury da versão anterior e conferir que ele oferece e instala a $version.
EOF
