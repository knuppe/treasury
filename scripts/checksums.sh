#!/bin/bash
# Refaz o SHA256SUMS.txt de uma pasta de release (docs/distribution.md): uma linha por arquivo,
# no formato do `shasum -a 256 -c` / `sha256sum -c`. Os SHA256SUMS-<sistema>.txt do CI ficam de
# fora (o arquivo final cobre tudo).
#
# Uso: scripts/checksums.sh dist/1.0.0
set -euo pipefail

dir="${1:?uso: scripts/checksums.sh <pasta da release>}"
[ -d "$dir" ] || { echo "pasta não existe: $dir" >&2; exit 1; }
sha256() {
  if command -v shasum >/dev/null; then shasum -a 256 "$1" | cut -d' ' -f1; else sha256sum "$1" | cut -d' ' -f1; fi
}
sums="$(mktemp)"
(
  cd "$dir"
  for f in *; do
    case "$f" in SHA256SUMS.txt | SHA256SUMS-*.txt) continue ;; esac
    if [ -f "$f" ]; then printf '%s  %s\n' "$(sha256 "$f")" "$f"; fi
  done
) >"$sums"
mv "$sums" "$dir/SHA256SUMS.txt"
echo "$dir/SHA256SUMS.txt: $(wc -l <"$dir/SHA256SUMS.txt" | tr -d ' ') arquivo(s)"
