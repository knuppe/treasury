# Chave de hardware que assina as atualizações (docs/auto-update.md). Para ser lido com
# `source` pelo release.sh e pelo sign-release.sh; precisa da função `die` de quem o lê.
#
# Duas chaves valem, as que estão em apps/desktop/src-tauri/update-keys.pub:
# - a do Secure Enclave do Mac (linha que começa com `secure-enclave`), que pede a digital;
# - a da YubiKey (FIDO2), que pede toque e PIN.

# Linha de chave aceita, com ou sem a palavra `secure-enclave` na frente.
HW_KEY_RE='^(secure-enclave )?(sk-(ssh-ed25519|ecdsa-sha2-nistp256)@openssh\.com .*)$'

# hw_has_keys <arquivo de chaves>: tem pelo menos uma chave?
hw_has_keys() { grep -Eq "$HW_KEY_RE" "$1"; }

# hw_allowed_signers <arquivo de chaves> <namespaces>: as linhas para o `ssh-keygen -Y verify`.
hw_allowed_signers() {
  sed -n -E "s/$HW_KEY_RE/treasury namespaces=\"$2\" \\2/p" "$1"
}

# hw_default_key <pasta das chaves>: a do Mac, se existir; senão a YubiKey principal.
hw_default_key() {
  if [ -f "$1/id_ecdsa_sk_rk_treasury-update" ]; then
    echo "$1/id_ecdsa_sk_rk_treasury-update"
  else
    echo "$1/yubikey-1"
  fi
}

# hw_expiry_warning <linha do sc_auth>: avisa quando faltam menos de 60 dias para a identidade
# da chave do Mac vencer. A data sai no formato do sistema (dia/mês/ano, ou mês/dia/ano com o
# macOS em inglês dos EUA); se não der para ler, não avisa.
hw_expiry_warning() {
  python3 -I - "$1" "$(defaults read -g AppleLocale 2>/dev/null || true)" <<'PY' || true
import datetime, re, sys

match = re.search(r"\b(\d{1,2})/(\d{1,2})/(\d{4})\b", sys.argv[1])
if match:
    a, b, year = (int(x) for x in match.groups())
    day, month = (b, a) if sys.argv[2].startswith("en_US") else (a, b)
    try:
        left = (datetime.date(year, month, day) - datetime.date.today()).days
    except ValueError:
        left = None
    if left is not None and left < 60:
        print(
            f"aviso: a identidade da chave do Mac vence em {left} dias; "
            "renove (docs/auto-update.md, \"Renovar a chave do Mac\")",
            file=sys.stderr,
        )
PY
}

# hw_prepare <arquivo da chave> <arquivo de chaves>: confere que a chave é uma das aceitas e
# define HW_SSH_KEYGEN (o ssh-keygen que assina) e HW_KEY_NAME (para as mensagens). Na chave do
# Mac, confere também que o macOS a guarda como não exportável e protegida pela digital: o app
# não consegue ver isso na assinatura, então a conferência é aqui, antes de assinar.
hw_prepare() {
  local sk="$1" keys="$2" kind blob fp row line
  [ -f "$sk" ] && [ -f "$sk.pub" ] || die "falta o arquivo da chave que assina: $sk (e $sk.pub)"
  read -r kind blob _ <"$sk.pub"
  if awk -v k="$kind" -v b="$blob" '$1 == "secure-enclave" && $2 == k && $3 == b {f = 1} END {exit !f}' "$keys"; then
    [ -x /usr/bin/ssh-keygen ] && [ -f /usr/lib/ssh-keychain.dylib ] ||
      die "a chave do Secure Enclave precisa do ssh-keygen e do ssh-keychain.dylib do macOS"
    fp="$(/usr/bin/ssh-keygen -lf "$sk.pub" | awk '{print $2}')"
    line="$(sc_auth list-ctk-identities -t ssh 2>/dev/null | awk -v fp="$fp" '$2 == fp')"
    row="$(echo "$line" | awk '{print $1, $3}')"
    [ "$row" = "p-256-ne bio" ] ||
      die "a chave do Mac não está no Secure Enclave com a digital exigida (sc_auth mostra: ${row:-chave não encontrada}); recrie com 'sc_auth create-ctk-identity -k p-256-ne -t bio' e troque a linha em update-keys.pub"
    # A identidade que o macOS cria vale um ano. Vencida, assine com a YubiKey e cadastre uma
    # chave nova do Mac por uma atualização (docs/auto-update.md, "Renovar a chave do Mac").
    [ "$(echo "$line" | awk '{print $NF}')" = YES ] ||
      die "a identidade da chave do Mac venceu (sc_auth list-ctk-identities); assine com a YubiKey (TREASURY_UPDATE_SK_KEY) e renove a chave do Mac (docs/auto-update.md)"
    hw_expiry_warning "$line"
    export SSH_SK_PROVIDER=/usr/lib/ssh-keychain.dylib
    HW_SSH_KEYGEN=/usr/bin/ssh-keygen
    HW_KEY_NAME="chave do Mac (digital)"
  elif awk -v k="$kind" -v b="$blob" '$1 == k && $2 == b {f = 1} END {exit !f}' "$keys"; then
    HW_SSH_KEYGEN=ssh-keygen
    HW_KEY_NAME="YubiKey (toque e PIN)"
  else
    die "a chave $sk não é uma das aceitas em $keys"
  fi
}
