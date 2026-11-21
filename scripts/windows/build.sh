#!/bin/bash
# Gera o instalador do Windows (x64, NSIS: Treasury_<versão>_x64-setup.exe) por compilação
# cruzada num contêiner Linux (MinGW-w64, alvo x86_64-pc-windows-gnu), a partir do commit atual.
# Roda num Mac com Docker Desktop (contêiner nativo arm64; ~10 min na primeira vez).
#
# Uso: scripts/windows/build.sh [pasta de saída]   (padrão: dist/<versão>)
#
# Limites da compilação cruzada: o instalador não é assinado (o SmartScreen avisa na primeira
# execução) e o MCP local do Windows não tem o socket do app (o `treasury mcp` abre o cofre só
# em modo leitura). O WebView2 vem do Windows 10/11; o instalador oferece instalar se faltar.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
VERSION=$(sed -n 's/.*"version": "\(.*\)".*/\1/p' apps/desktop/src-tauri/tauri.conf.json | head -1)
OUT="${1:-$ROOT/dist/$VERSION}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
IMG=treasury-win-build:24.04

docker info >/dev/null 2>&1 || { echo "Docker não está rodando (abra o Docker Desktop)"; exit 1; }
docker build -t $IMG scripts/windows
git archive HEAD | tar -x -C "$WORK"
for v in target cargo; do docker volume create "treasury-win-$v" >/dev/null; done
mkdir -p "$OUT"
docker run --rm -v "$WORK":/src -v treasury-win-target:/target -v treasury-win-cargo:/opt/cargo/registry \
  -v "$OUT":/out -e CARGO_TARGET_DIR=/target $IMG bash -c "cd /src/apps/desktop && npm ci --no-audit --no-fund \
  && npx tauri build --target x86_64-pc-windows-gnu --bundles nsis \
  && cp /target/x86_64-pc-windows-gnu/release/bundle/nsis/Treasury_${VERSION}_x64-setup.exe /out/"
echo "Pronto: $OUT"
ls -la "$OUT"
