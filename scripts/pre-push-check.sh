#!/bin/bash
# The CI checks, on this machine, before a push (docs/development.md, "Before a push").
#
#   scripts/pre-push-check.sh          # quick (the pre-push hook): what fails the CI in seconds
#   scripts/pre-push-check.sh --full   # everything the CI runs: before pushing a release
#
# Quick: the whole `audit` job (approved dependencies, cargo deny, npm audit, third-party
# licenses), cargo fmt, prettier, lint and types. Full adds clippy, the workspace tests in UTC
# (the CI machines run in UTC), the generated TS types, the interface tests, the build and the
# end-to-end scenarios.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
export PATH="$HOME/.cargo/bin:$PATH"
full=0
[ "${1:-}" = "--full" ] && full=1

step() { printf '\n== %s\n' "$1" >&2; }

if ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo-deny is not installed (cargo install cargo-deny --locked)." >&2
  exit 1
fi

step "approved dependencies"
python3 -I scripts/deps-check.py
step "cargo deny"
cargo deny --log-level error check advisories bans sources licenses
step "npm audit (production dependencies)"
(cd apps/desktop && npm audit --omit=dev --audit-level=moderate)
step "third-party licenses"
python3 -I scripts/third-party-licenses.py --check
step "cargo fmt"
cargo fmt --all --check
step "prettier, lint and types"
(cd apps/desktop && npm run --silent format:check && npm run --silent lint && npm run --silent typecheck)

if [ "$full" = 1 ]; then
  step "clippy"
  cargo clippy --workspace --all-targets -- -D warnings
  step "workspace tests (UTC)"
  TZ=UTC cargo test --workspace -q
  step "generated TS types"
  git diff --exit-code -- apps/desktop/src/api/generated
  step "interface tests (UTC) and build"
  (cd apps/desktop && TZ=UTC npx vitest run && npx vite build)
  step "end-to-end (Chromium)"
  (cd apps/desktop && npm run e2e -- --trace off)
fi
printf '\nAll checks passed.\n' >&2
