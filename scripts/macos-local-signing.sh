#!/bin/bash
# Cria (uma vez) uma identidade de assinatura de código local para o Treasury no Mac.
#
# Por quê: os builds locais saem com assinatura ad hoc, que muda a cada versão. O Keychain
# do macOS amarra o acesso ao item do cofre a essa assinatura, então cada versão nova pede
# a senha do Mac depois do Touch ID ("treasury quer usar informações do seu Keychain").
# Assinando sempre com o mesmo certificado, o "Sempre permitir" vale para as próximas
# versões e o Touch ID volta a bastar.
#
# O certificado é autoassinado, só para assinar código, e a chave privada fica no seu
# Keychain de sessão (login). Nada sai do computador. O macOS pede a sua senha uma vez para
# confiar no certificado para assinatura de código.
#
# Uso: scripts/macos-local-signing.sh
set -euo pipefail

NAME="Treasury Local Signing"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"

if security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "A identidade \"$NAME\" já existe. Liberando o uso pelo codesign sem diálogo…"
  security set-key-partition-list -S apple-tool:,apple:,codesign: -s -l "$NAME" "$KEYCHAIN" >/dev/null
  security find-identity -v -p codesigning | grep "\"$NAME\""
  exit 0
fi

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cat > "$WORK/cert.cnf" <<CNF
[ req ]
distinguished_name = dn
x509_extensions = ext
prompt = no
[ dn ]
CN = $NAME
O = Treasury (local)
[ ext ]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
subjectKeyIdentifier = hash
CNF

openssl req -x509 -newkey rsa:3072 -nodes -days 3650 \
  -keyout "$WORK/key.pem" -out "$WORK/cert.pem" -config "$WORK/cert.cnf" >/dev/null 2>&1
PASS=$(openssl rand -hex 16)
openssl pkcs12 -export -legacy -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
  -name "$NAME" -out "$WORK/id.p12" -passout "pass:$PASS" 2>/dev/null ||
  openssl pkcs12 -export -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
    -name "$NAME" -out "$WORK/id.p12" -passout "pass:$PASS"

# Só o codesign usa a chave sem perguntar.
security import "$WORK/id.p12" -k "$KEYCHAIN" -P "$PASS" -T /usr/bin/codesign >/dev/null
# Confiança só para assinatura de código, no domínio do usuário (pede a senha do Mac).
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$WORK/cert.pem"
# Deixa o codesign usar a chave sem diálogo (builds rodando fora da sessão gráfica, como
# os do agente, falham com errSecInternalComponent sem isso). Pede a senha do Keychain.
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -l "$NAME" "$KEYCHAIN" >/dev/null

echo "Pronto. Identidade criada:"
security find-identity -v -p codesigning | grep "\"$NAME\""
echo
echo "Os próximos builds saem assinados com ela. Na primeira abertura, o macOS pergunta uma"
echo "última vez pelo Keychain: escolha \"Sempre permitir\"."
