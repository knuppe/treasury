#!/bin/bash
# Gera o AppImage e o .deb do Linux (x86_64) num contêiner Ubuntu 22.04 (as mesmas
# dependências do CI), a partir do commit atual. Funciona num Mac com Docker Desktop
# (Apple Silicon roda a imagem amd64 emulada; a primeira vez leva ~20 min, depois ~5 min).
#
# Uso: scripts/linux/build.sh [pasta de saída]   (padrão: dist/<versão>)
#      ARCH=arm64 scripts/linux/build.sh            (ARM64, nativo no Apple Silicon: só o .deb,
#                                                   para testar numa VM ARM do UTM)
#
# Caches em volumes do Docker (treasury-linux-*): target, registro do cargo e as ferramentas
# do AppImage. Para limpar: docker volume rm treasury-linux-target treasury-linux-cargo
# treasury-linux-tauricache
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
VERSION=$(sed -n 's/.*"version": "\(.*\)".*/\1/p' apps/desktop/src-tauri/tauri.conf.json | head -1)
OUT="${1:-$ROOT/dist/$VERSION}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
ARCH="${ARCH:-amd64}"
case "$ARCH" in
  amd64) SUFFIX=""; BUNDLES=deb,appimage ;;
  arm64) SUFFIX="-arm64"; BUNDLES=deb ;;
  *) echo "ARCH deve ser amd64 ou arm64"; exit 1 ;;
esac
IMG=treasury-linux-build$SUFFIX:22.04
PLATFORM=linux/$ARCH

docker info >/dev/null 2>&1 || { echo "Docker não está rodando (abra o Docker Desktop)"; exit 1; }
docker build --platform $PLATFORM -t $IMG scripts/linux
git archive HEAD | tar -x -C "$WORK"
for v in target cargo tauricache; do docker volume create "treasury-linux$SUFFIX-$v" >/dev/null; done
run() {
  docker run --rm --platform $PLATFORM -e APPIMAGE_EXTRACT_AND_RUN=1 \
    -v "$WORK":/src -v "treasury-linux$SUFFIX-target":/target \
    -v "treasury-linux$SUFFIX-cargo":/opt/cargo/registry \
    -v "treasury-linux$SUFFIX-tauricache":/root/.cache/tauri -v "$OUT":/out $IMG bash -c "$1"
}
mkdir -p "$OUT"
if [ "$ARCH" = arm64 ]; then
  run "cd /src/apps/desktop && npm ci --no-audit --no-fund && CARGO_TARGET_DIR=/target npx tauri build --bundles $BUNDLES \
    && cp /target/release/bundle/deb/Treasury_${VERSION}_arm64.deb /out/"
else
  # As ferramentas do AppImage (linuxdeploy) são AppImages: sob emulação, os bytes mágicos no
  # cabeçalho ELF fazem o binfmt recusar o binário ("Exec format error"); zerados, rodam.
  PLUGIN=https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage
  run "cd /root/.cache/tauri && { [ -f linuxdeploy-plugin-appimage.AppImage ] || curl -fsSL -o linuxdeploy-plugin-appimage.AppImage $PLUGIN; } && chmod +x linuxdeploy-plugin-appimage.AppImage && dd if=/dev/zero of=linuxdeploy-plugin-appimage.AppImage bs=1 count=3 seek=8 conv=notrunc 2>/dev/null"
  run "cd /src/apps/desktop && npm ci --no-audit --no-fund && CARGO_TARGET_DIR=/target npx tauri build --bundles $BUNDLES \
    && cp /target/release/bundle/appimage/Treasury_${VERSION}_amd64.AppImage /target/release/bundle/deb/Treasury_${VERSION}_amd64.deb /out/"
fi
echo "Pronto: $OUT"
ls -la "$OUT"
