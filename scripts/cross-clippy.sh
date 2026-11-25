#!/bin/bash
# Clippy for Linux and Windows on the Mac, in the Docker images of scripts/linux and
# scripts/windows, so a warning that only exists on another system (code under
# `cfg(target_os = ...)`, an item used only by the macOS side) fails here and not in CI.
#
#   scripts/cross-clippy.sh            # Linux and Windows
#   scripts/cross-clippy.sh linux      # one of them
#
# Linux runs as a native arm64 container (the `cfg` is the same as on the x86_64 runners);
# Windows cross-compiles to x86_64-pc-windows-gnu with MinGW, only the app and the core (what
# the Windows installer ships; treasury-dev is Unix-only). Both check the working tree as it
# is, uncommitted changes included. Targets and the cargo registry live in Docker volumes
# (treasury-clippy-*), so only the first run compiles everything (~10 min each). The library and
# the tests are checked in two steps, as in CI on macOS: the core's compiler takes 7 to 8 GB, and
# two at once do not fit in Docker Desktop's default 8 GB.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [ $# -gt 0 ]; then systems=("$@"); else systems=(linux windows); fi

docker info >/dev/null 2>&1 || {
  echo "Docker is not running (open Docker Desktop), or skip with SKIP_CROSS_CLIPPY=1." >&2
  exit 1
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
# Tracked and new files, without what git ignores (target/, node_modules/, dist/).
git ls-files -z --cached --others --exclude-standard | tar --null -T - -cf - | tar -xf - -C "$WORK"

run() { # image, system, command
  for v in target cargo; do docker volume create "treasury-clippy-$2-$v" >/dev/null; done
  docker run --rm --platform linux/arm64 -v "$WORK":/src \
    -v "treasury-clippy-$2-target":/target -v "treasury-clippy-$2-cargo":/opt/cargo/registry \
    -e CARGO_TARGET_DIR=/target -w /src "$1" bash -c "$3"
}

for system in "${systems[@]}"; do
  case "$system" in
    linux)
      printf '\n== clippy (Linux)\n' >&2
      img=treasury-linux-build-arm64:22.04
      docker build -q --platform linux/arm64 -t $img scripts/linux >/dev/null
      run $img linux "cargo clippy --locked --workspace -- -D warnings \
        && cargo clippy --locked --workspace --all-targets -- -D warnings"
      ;;
    windows)
      printf '\n== clippy (Windows)\n' >&2
      img=treasury-win-build:24.04
      docker build -q --platform linux/arm64 -t $img scripts/windows >/dev/null
      run $img windows "cargo clippy --locked -p treasury-desktop -p treasury-core --target x86_64-pc-windows-gnu -- -D warnings \
        && cargo clippy --locked -p treasury-desktop -p treasury-core --all-targets --target x86_64-pc-windows-gnu -- -D warnings"
      ;;
    *)
      echo "unknown system: $system (linux or windows)" >&2
      exit 1
      ;;
  esac
done
