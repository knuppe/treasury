#!/bin/bash
# Gera o app de release do Treasury: poda o cache velho, confere, sobe a versão (opcional),
# compila e instala (opcional).
#
# Uso:
#   scripts/release.sh                       # verificações + build da versão atual
#   scripts/release.sh 0.6.1 --install       # sobe a versão, commita, compila e instala
#   scripts/release.sh --skip-checks         # só o build (iteração rápida)
#
# Opções:
#   X.Y.Z          sobe a versão do app, registra as migrações novas no migrations.lock e
#                  commita "release: X.Y.Z (…)". Exige a árvore sem mudanças.
#   --install      (macOS) fecha o Treasury, manda o app antigo para a Lixeira, instala o novo
#                  em /Applications e abre.
#   --skip-checks  pula fmt, clippy, testes e as verificações da interface.
#   --dist         perfil `dist` (LTO completa, uma unidade de código): binário menor e um
#                  pouco mais rápido, build bem mais lento. Para instaladores públicos.
#   --dmg          gera também o .dmg (macOS).
#   --update       gera um pacote de atualização assinado do macOS e o latest.json em
#                  target/<perfil>/bundle/update, para testar a atualização numa máquina sua
#                  (docs/auto-update.md). Duas assinaturas: minisign (chave em
#                  ~/Treasury-privado/updater, cifrada para a YubiKey em
#                  treasury-updater.key.age) e a da YubiKey (ssh-keygen -Y sign; pede toque e
#                  PIN); sem chaves em update-keys.pub, recusa. A release pública não sai daqui:
#                  o release.yml compila no CI e o scripts/sign-release.sh assina
#                  (docs/distribution.md).
#   --no-clean     não poda target/ antes do build.
#
# Distribuição (docs/distribution.md): com --dmg ou --update, os arquivos públicos (.dmg, pacote
# de atualização, .sig e latest.json) são copiados para dist/<versão>/, onde também ficam os de
# Linux e Windows baixados do CI, e o SHA256SUMS.txt é refeito com todos.
#
# Atualização automática: $TREASURY_UPDATE_REPO (dono/repositório no GitHub, padrão
# knuppe/treasury) define onde o app compilado procura versões novas. Com
# TREASURY_UPDATE_REPO=none, o app não procura nada.
#
# A versão do app fica em apps/desktop/src-tauri/Cargo.toml (e tauri.conf.json, package.json,
# Brand.tsx, mock/settings.ts). A versão do núcleo (Cargo.toml do workspace) fica fixa: mudá-la
# a cada release recompilaria o núcleo inteiro e deixaria cópias velhas de GBs em target/.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
DESKTOP="apps/desktop"
LOCK="crates/treasury-core/migrations.lock"
GENERATED="$DESKTOP/src/api/generated"
LOGS="target/release-logs"

version=""
install=0
checks=1
dist=0
dmg=0
update=0
clean=1
for arg in "$@"; do
  case "$arg" in
    --install) install=1 ;;
    --skip-checks) checks=0 ;;
    --dist) dist=1 ;;
    --dmg) dmg=1 ;;
    --update) update=1 ;;
    --no-clean) clean=0 ;;
    -h | --help)
      awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "$0"
      exit 0
      ;;
    [0-9]*.[0-9]*.[0-9]*) version="$arg" ;;
    *)
      echo "opção desconhecida: $arg (veja --help)" >&2
      exit 2
      ;;
  esac
done

started=$(date +%s)
timings=""
step_start=0
step() {
  step_start=$(date +%s)
  printf '\n\033[1m== %s\033[0m\n' "$1"
}
step_done() {
  local secs=$(($(date +%s) - step_start))
  timings="${timings}$(printf '  %-22s %4ss' "$1" "$secs")"$'\n'
}
die() {
  printf '\033[31merro:\033[0m %s\n' "$1" >&2
  exit 1
}
current_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' "$DESKTOP/src-tauri/Cargo.toml" | head -1
}
# 0 se $1 > $2 (versões X.Y.Z)
version_gt() {
  local IFS=.
  local -a a=($1) b=($2)
  local i
  for i in 0 1 2; do
    ((10#${a[i]} > 10#${b[i]})) && return 0
    ((10#${a[i]} < 10#${b[i]})) && return 1
  done
  return 1
}
sha256() {
  if command -v shasum >/dev/null; then shasum -a 256 "$1" | cut -d' ' -f1; else sha256sum "$1" | cut -d' ' -f1; fi
}
gb_free() {
  df -Pk "$ROOT" | awk 'NR==2 { printf "%d", $4 / 1024 / 1024 }'
}
tree_hash() {
  find "$GENERATED" -type f -print0 | sort -z | xargs -0 cat | cksum
}

# ---------------------------------------------------------------------------- preparo
step "Preparo"
os="$(uname -s)"
[ "$install" = 1 ] && [ "$os" != Darwin ] && die "--install só no macOS (no Linux, instale o pacote de target/*/bundle)"
if pgrep -f "tauri build" >/dev/null; then
  die "já há um 'tauri build' rodando (pgrep -fl 'tauri build'); espere ele terminar"
fi
old_version="$(current_version)"
if [ -n "$version" ]; then
  [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "versão inválida: $version (use X.Y.Z)"
  version_gt "$version" "$old_version" || die "a versão $version não é maior que a atual ($old_version)"
  git diff --quiet && git diff --cached --quiet || die "há mudanças não commitadas; commite antes de subir a versão"
fi
echo "versão atual: $old_version${version:+ → $version}"
echo "espaço livre: $(gb_free) GB"
step_done "preparo"

# ---------------------------------------------------------------------------- limpeza
if [ "$clean" = 1 ]; then
  step "Limpeza de target/"
  python3 scripts/clean-build.py
  step_done "limpeza"
fi
[ "$(gb_free)" -lt 15 ] && die "menos de 15 GB livres; libere espaço (python3 scripts/clean-build.py --keep 1 --days 1)"

# ---------------------------------------------------------------------------- verificações
if [ "$checks" = 1 ]; then
  mkdir -p "$LOGS"
  step "Verificações"
  # Duas filas em paralelo: a interface (prettier, tsc, eslint, vitest) e o Rust (fmt,
  # clippy, testes). Os testes `export_bindings` regravam os tipos TS gerados que o tsc lê,
  # por isso ficam de fora da fila do Rust e rodam no fim, sozinhos, com a conferência de
  # que nada mudou. Antes, os testes do Rust esperavam a interface terminar (~1,5 min a mais).
  web_log="$LOGS/interface.log"
  (
    cd "$DESKTOP"
    set -e
    npm run --silent format:check
    npm run --silent typecheck
    npm run --silent lint
    npm test --silent
  ) >"$web_log" 2>&1 &
  web_pid=$!
  stop_web() { kill "$web_pid" 2>/dev/null; }

  echo "cargo fmt --check"
  cargo fmt --all --check || {
    stop_web
    die "formatação: rode cargo fmt --all"
  }
  echo "cargo clippy (workspace, todos os alvos)"
  cargo clippy --workspace --all-targets --quiet -- -D warnings || {
    stop_web
    die "clippy"
  }
  echo "cargo test (workspace, sem export_bindings), junto com a interface"
  # Com prioridade menor e metade dos núcleos: o vitest, ao lado, tem esperas com prazo e
  # falhava por tempo quando o Rust ocupava a máquina inteira.
  threads=$(($(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 8) / 2))
  if ! nice -n 10 cargo test --workspace --quiet -- --skip export_bindings --test-threads="$threads" >"$LOGS/testes.log" 2>&1; then
    stop_web
    grep -E "FAILED|panicked|^error" "$LOGS/testes.log" | head -30
    die "testes (log completo: $LOGS/testes.log)"
  fi
  awk '/^test result/ { ok += $4; ign += $8 } END { printf "%d testes ok, %d ignorados\n", ok, ign }' "$LOGS/testes.log"

  echo "interface (prettier, tsc, eslint, vitest): aguardando…"
  if ! wait "$web_pid"; then
    tail -40 "$web_log"
    die "verificações da interface (log completo: $web_log)"
  fi
  echo "interface ok"

  echo "tipos TS gerados (export_bindings)"
  before="$(tree_hash)"
  if ! cargo test --workspace --quiet export_bindings >"$LOGS/bindings.log" 2>&1; then
    grep -E "FAILED|panicked|^error" "$LOGS/bindings.log" | head -30
    die "export_bindings (log completo: $LOGS/bindings.log)"
  fi
  if [ "$(tree_hash)" != "$before" ]; then
    git status --short -- "$GENERATED"
    die "os testes mudaram os tipos TS gerados ($GENERATED); confira, commite e rode de novo"
  fi
  step_done "verificações"
fi

# ---------------------------------------------------------------------------- versão
if [ -n "$version" ]; then
  step "Versão $version"
  perl -0pi -e 's/^version = "[^"]+"/version = "'"$version"'"/m' "$DESKTOP/src-tauri/Cargo.toml"
  perl -0pi -e 's/("version": ")[^"]+"/${1}'"$version"'"/' "$DESKTOP/src-tauri/tauri.conf.json"
  perl -pi -e "s/^export const APP_VERSION = '[^']+';/export const APP_VERSION = '$version';/" "$DESKTOP/src/app/Brand.tsx"
  perl -pi -e "s/^(\s+version: )'[^']+',/\${1}'$version',/" "$DESKTOP/src/api/mock/settings.ts"
  (cd "$DESKTOP" && npm version "$version" --no-git-tag-version --allow-same-version >/dev/null)
  cargo update --workspace --offline --quiet

  # Migrações novas (número maior que o último do lock) entram sob o cabeçalho da versão.
  last=$(awk '/^[0-9]+ /{ n = $1 } END { print n + 0 }' "$LOCK")
  added=""
  notes=""
  for f in $(ls crates/treasury-core/migrations/ | sed -n 's/^V\([0-9]*\)__.*\.sql$/\1 &/p' | sort -n | awk -v l="$last" '$1 > l { print $2 }'); do
    n="${f#V}"
    n="${n%%__*}"
    added="${added}${n} ${f} $(sha256 "crates/treasury-core/migrations/$f")"$'\n'
    name="${f#V*__}"
    name="${name%.sql}"
    notes="${notes:+$notes; }V$n: ${name//_/ }"
  done
  if [ -n "$added" ]; then
    printf '# %s\n%s' "$version" "$added" >>"$LOCK"
    count=$(printf '%s' "$added" | grep -c .)
    [ "$count" = 1 ] && subject="release: $version (migration $notes)" || subject="release: $version (migrations $notes)"
  else
    subject="release: $version (no new migrations)"
  fi
  if [ -n "$added" ]; then
    printf '%s' "$added"
    # Confere que nenhuma migração publicada mudou e que as novas entraram na ordem (mesmos
    # binários de teste das verificações: não recompila o núcleo de novo).
    cargo test --workspace --lib --quiet released_migrations_are_frozen >/dev/null ||
      die "migrations.lock inconsistente (cargo test -p treasury-core released_migrations_are_frozen)"
  fi
  git add -A "$DESKTOP/src-tauri/Cargo.toml" "$DESKTOP/src-tauri/tauri.conf.json" "$DESKTOP/package.json" \
    "$DESKTOP/package-lock.json" "$DESKTOP/src/app/Brand.tsx" "$DESKTOP/src/api/mock/settings.ts" Cargo.lock "$LOCK"
  git -c commit.gpgsign=false commit --quiet -m "$subject"
  echo "commit: $(git log -1 --format='%h %s')"
  step_done "versão"
fi

# ---------------------------------------------------------------------------- build
if [ "$os" = Darwin ]; then
  bundles="app"
  [ "$dmg" = 1 ] && bundles="app,dmg"
else
  bundles="appimage,deb"
fi
profile_dir="release"
cargo_args=()
if [ "$dist" = 1 ]; then
  profile_dir="dist"
  cargo_args=(-- --profile dist)
fi
# Onde o app procura atualizações (src/updater/ lê na compilação). Sem repositório, não procura.
update_repo="${TREASURY_UPDATE_REPO:-knuppe/treasury}"
[ "$update_repo" = none ] && update_repo=""
if [ -n "$update_repo" ]; then
  export TREASURY_UPDATE_ENDPOINT="https://github.com/$update_repo/releases/latest/download/latest.json"
  echo "atualizações: $TREASURY_UPDATE_ENDPOINT"
else
  unset TREASURY_UPDATE_ENDPOINT
  echo "atualizações: TREASURY_UPDATE_REPO=none, o app não vai procurar versões novas"
fi
step "Build ($profile_dir, $bundles)"
(cd "$DESKTOP" && npx tauri build --bundles "$bundles" ${cargo_args[@]+"${cargo_args[@]}"})
step_done "build"

bundle_dir="target/$profile_dir/bundle"
app="$bundle_dir/macos/Treasury.app"
built_version="$(current_version)"

# ---------------------------------------------------------------------------- assinatura
# macOS: hardened runtime (docs/local-security.md, item 4). Sem ele, DYLD_INSERT_LIBRARIES
# injeta código no app e outro processo do usuário pode ler a memória dele. Só aqui, no
# release: o build de desenvolvimento e os testes não mudam. Usa a identidade local
# ("Treasury Local Signing", scripts/macos-local-signing.sh) quando existe, para o Keychain
# continuar reconhecendo o app entre versões; senão, assinatura ad hoc.
if [ "$os" = Darwin ] && [ -d "$app" ]; then
  step "Assinatura (hardened runtime)"
  identity="${APPLE_SIGNING_IDENTITY:-}"
  if [ -z "$identity" ] && security find-identity -v -p codesigning 2>/dev/null | grep -q "Treasury Local Signing"; then
    identity="Treasury Local Signing"
  fi
  [ -n "$identity" ] || identity="-"
  codesign --force --options runtime --timestamp=none \
    --entitlements "$DESKTOP/src-tauri/release.entitlements" \
    --sign "$identity" "$app" || die "codesign falhou"
  codesign --verify --strict --verbose=2 "$app" || die "a assinatura do app não confere"
  flags="$(codesign -dv "$app" 2>&1 | sed -n 's/.*flags=\(0x[0-9a-f]*\)(\(.*\)).*/\2/p')"
  case "$flags" in
    *runtime*) echo "assinado com hardened runtime ($identity; $flags)" ;;
    *) die "o app saiu sem hardened runtime ($flags)" ;;
  esac
  # O .dmg do Tauri foi gerado antes desta assinatura: refaz com o app assinado.
  if [ "$dmg" = 1 ]; then
    for d in "$bundle_dir"/dmg/*.dmg; do
      [ -f "$d" ] || continue
      rm -f "$d"
      hdiutil create -quiet -volname Treasury -srcfolder "$app" -ov -format UDZO "$d" ||
        die "não consegui refazer $d"
      echo "dmg refeito com o app assinado: $d"
    done
  fi
  step_done "assinatura"
fi

# ---------------------------------------------------------------------------- atualização
# Pacote que o app instalado baixa sozinho (docs/auto-update.md): o .app já assinado acima,
# em .tar.gz, com duas assinaturas e o latest.json que vai na release do GitHub junto com ele.
# 1. minisign, presa à versão (--app-version), que o app confere (updater/minisign.rs). A chave fica
#    cifrada com o age para o Secure Enclave do Mac e para a YubiKey, e só é decifrada na memória.
# 2. YubiKey (FIDO2, ssh-keygen -Y sign, toque e PIN), que o app sempre exige; sem chaves em
#    src-tauri/update-keys.pub, o script recusa.
if [ "$update" = 1 ]; then
  step "Pacote de atualização"
  [ "$os" = Darwin ] || die "--update: por enquanto só no macOS"
  [ -d "$app" ] || die "não achei $app"
  keys_dir="$HOME/Treasury-privado/updater"
  key="${TREASURY_UPDATER_KEY:-$keys_dir/treasury-updater.key}"
  if [ -f "$key.age" ]; then
    command -v age >/dev/null || die "falta o age (brew install age age-plugin-se age-plugin-yubikey)"
    identity="${TREASURY_AGE_IDENTITY:-$keys_dir/age-identities.txt}"
    [ -f "$identity" ] || die "faltam as identidades do age (Mac e YubiKey): $identity"
    echo "abrindo a chave minisign: digital no Mac, ou PIN e toque na YubiKey"
    minisign_key="$(age -d -i "$identity" "$key.age")" || die "não consegui decifrar $key.age"
  elif [ -f "$key" ]; then
    echo "aviso: chave minisign sem a proteção da YubiKey ($key); veja docs/auto-update.md"
    minisign_key="$(cat "$key")"
  else
    die "chave das atualizações não encontrada: $key(.age)"
  fi
  arch="$(uname -m)"
  [ "$arch" = arm64 ] && arch=aarch64
  out="$bundle_dir/update"
  rm -rf "$out"
  mkdir -p "$out"
  tarball="$out/Treasury_${built_version}_${arch}.app.tar.gz"
  COPYFILE_DISABLE=1 tar -czf "$tarball" -C "$(dirname "$app")" "$(basename "$app")"

  # 2. YubiKey (antes da minisign: as duas gravariam em .sig).
  hw_keys="$DESKTOP/src-tauri/update-keys.pub"
  yubikey_sig=""
  # shellcheck source=scripts/update-signing.sh
  source "$ROOT/scripts/update-signing.sh"
  if hw_has_keys "$hw_keys"; then
    sk="${TREASURY_UPDATE_SK_KEY:-$(hw_default_key "$keys_dir")}"
    hw_prepare "$sk" "$hw_keys"
    echo "assinando o pacote com a $HW_KEY_NAME"
    # O namespace leva a versão: a assinatura não serve para republicar o pacote como outra versão.
    "$HW_SSH_KEYGEN" -Y sign -f "$sk" -n "treasury-update@$built_version" "$tarball" ||
      die "a $HW_KEY_NAME não assinou o pacote"
    yubikey_sig="$tarball.yubikey.sig"
    mv "$tarball.sig" "$yubikey_sig"
    # Confere com as chaves que o app aceita, antes de publicar.
    signers="$(mktemp)"
    hw_allowed_signers "$hw_keys" "treasury-update@$built_version" >"$signers"
    ssh-keygen -Y verify -f "$signers" -I treasury -n "treasury-update@$built_version" -s "$yubikey_sig" <"$tarball" ||
      { rm -f "$signers"; die "a assinatura de hardware não confere com update-keys.pub"; }
    rm -f "$signers"
  else
    die "update-keys.pub sem chaves de hardware: o app recusaria o pacote (docs/trust.md)"
  fi

  # 1. minisign. A chave vai pelo ambiente do processo, nunca na linha de comando: o `npx`
  #    grava os argumentos em ~/.npm/_logs, e a linha de comando aparece no `ps`.
  TAURI_SIGNING_PRIVATE_KEY="$minisign_key" \
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" \
    "$ROOT/$DESKTOP/node_modules/.bin/tauri" signer sign --app-version "$built_version" \
    "$ROOT/$tarball" >/dev/null || die "não consegui assinar $tarball"
  unset minisign_key

  [ -n "$update_repo" ] || echo "aviso: sem repositório; o latest.json sai com endereço provisório"
  python3 - "$out/latest.json" "$built_version" "darwin-$arch" "$tarball.sig" \
    "https://github.com/${update_repo:-DONO/REPOSITORIO}/releases/download/v$built_version/$(basename "$tarball")" \
    "${TREASURY_RELEASE_NOTES:-Treasury $built_version}" "$yubikey_sig" <<'PY'
import datetime, json, sys
out, version, platform, sig, url, notes, hw = sys.argv[1:8]
entry = {"signature": open(sig).read().strip(), "url": url}
if hw:
    entry["yubikey_signature"] = open(hw).read()
data = {
    "version": version,
    "notes": notes,
    "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "platforms": {platform: entry},
}
with open(out, "w") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
PY
  echo "pacote: $tarball"
  echo "publique na release v$built_version: $(basename "$tarball") e latest.json"
  step_done "atualização"
fi

# ---------------------------------------------------------------------------- distribuição
# Os arquivos que vão na release do GitHub, numa pasta só, com o SHA256SUMS.txt de tudo o que
# estiver nela (inclusive os de Linux e Windows que o CI gerou e foram baixados para lá).
if [ "$dmg" = 1 ] || [ "$update" = 1 ]; then
  step "Distribuição"
  pub="dist/$built_version"
  mkdir -p "$pub"
  shopt -s nullglob
  for f in "$bundle_dir"/dmg/*.dmg "$bundle_dir"/update/*; do
    cp -f "$f" "$pub/"
  done
  shopt -u nullglob
  scripts/checksums.sh "$pub"
  echo "pasta da release: $pub"
  step_done "distribuição"
fi

# ---------------------------------------------------------------------------- instalação
if [ "$install" = 1 ]; then
  step "Instalação"
  [ -d "$app" ] || die "não achei $app"
  # O identificador mudou para io.github.knuppe.treasury (docs/decisions.md, D30); o app instalado
  # ainda pode ser o de antes, com dev.treasury.desktop.
  for bundle_id in io.github.knuppe.treasury dev.treasury.desktop; do
    running() { [ "$(osascript -e "application id \"$bundle_id\" is running" 2>/dev/null)" = true ]; }
    running || continue
    osascript -e "tell application id \"$bundle_id\" to quit" || true
    for _ in $(seq 1 30); do
      running || break
      sleep 1
    done
    running && die "o Treasury não fechou; feche-o e rode de novo com --skip-checks --install"
  done
  if [ -d /Applications/Treasury.app ]; then
    prev="$(defaults read /Applications/Treasury.app/Contents/Info CFBundleShortVersionString 2>/dev/null || echo antiga)"
    dest="/Applications/Treasury-$prev.app"
    [ -e "$dest" ] && dest="/Applications/Treasury-$prev-$(date +%s).app"
    mv /Applications/Treasury.app "$dest"
    osascript -e "tell application \"Finder\" to delete POSIX file \"$dest\"" >/dev/null ||
      echo "aviso: não consegui mandar $dest para a Lixeira; apague à mão"
    echo "versão anterior ($prev) foi para a Lixeira"
  fi
  ditto "$app" /Applications/Treasury.app
  open /Applications/Treasury.app
  echo "instalado: /Applications/Treasury.app ($built_version)"
  step_done "instalação"
fi

# ---------------------------------------------------------------------------- resumo
total=$(($(date +%s) - started))
printf '\n\033[1m== Pronto: Treasury %s em %dm%02ds\033[0m\n' "$built_version" $((total / 60)) $((total % 60))
printf '%s' "$timings"
for b in "$bundle_dir"/*/*; do
  case "$b" in
    *.app | *.dmg | *.AppImage | *.deb | *.tar.gz | */latest.json) echo "  $(du -sh "$b" | cut -f1)	$b" ;;
  esac
done
echo "  target/: $(du -sh target 2>/dev/null | cut -f1), livre: $(gb_free) GB"
