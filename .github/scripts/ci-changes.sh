#!/usr/bin/env bash
# Decide quais jobs do CI rodam, pelo que mudou (job `changes` do ci.yml). O padrão é rodar tudo:
# um job só é pulado quando todos os caminhos mudados estão numa lista conhecida, abaixo.
#
# Com o que cada evento compara:
#   - pull request: a base do PR (o primeiro pai do commit de merge que o GitHub monta);
#   - push na main: o último commit da main em que o CI passou, não o commit anterior. Assim um
#     commit só de documentação por cima de um commit quebrado não deixa a main verde;
#   - execução da noite: roda tudo, se a main mudou desde a última noite que passou;
#   - à mão (workflow_dispatch): roda tudo.
#
# Saídas (true/false) em $GITHUB_OUTPUT: rust, ui, e2e, audit, bundle e full (suíte completa do
# macOS). Para conferir uma lista de caminhos sem git:
#   printf 'docs/trust.md\n' | .github/scripts/ci-changes.sh --classify
set -euo pipefail

rust=0 ui=0 audit=0 bundle=0 all=0 full=0
base="" head="HEAD"

# Marca o que um caminho mudado exige.
classify_path() {
  local f="$1"

  # Empacotamento: o que muda o instalador sem necessariamente mudar um teste.
  case "$f" in
    apps/desktop/src-tauri/tauri.conf.json | apps/desktop/src-tauri/Cargo.toml | \
      apps/desktop/src-tauri/build.rs | apps/desktop/src-tauri/Info.plist | \
      apps/desktop/src-tauri/release.entitlements | apps/desktop/src-tauri/icons/* | \
      apps/desktop/src-tauri/capabilities/* | apps/desktop/src-tauri/permissions/* | \
      apps/desktop/package.json | apps/desktop/package-lock.json | apps/desktop/vite.config.ts | \
      crates/*/Cargo.toml | crates/*/build.rs | Cargo.toml | Cargo.lock | rust-toolchain.toml | \
      third_party/* | LICENSE | NOTICE | THIRD_PARTY_LICENSES.md | DATA_LICENSES.md | \
      .github/workflows/ci.yml)
      bundle=1
      ;;
  esac

  case "$f" in
    # Lidos pelo núcleo e pela interface: catálogos de idioma, Termos, dados da carga
    # tributária e os tipos TS que os testes do núcleo geram.
    locales/* | docs/terms.md | docs/terms.pt-BR.md | \
      crates/treasury-core/src/tax_burden/data/* | apps/desktop/src/api/generated/*)
      rust=1 ui=1
      ;;
    # Rust: os crates, o app, as dependências e os dados que o núcleo embute.
    crates/* | apps/desktop/src-tauri/* | Cargo.toml | Cargo.lock | rust-toolchain.toml | \
      .cargo/* | third_party/* | deny.toml | methodology/*)
      rust=1
      ;;
    # Interface. Um teste do núcleo lê as chamadas `useCore` da interface (toda chamada precisa
    # estar no MCP ou na lista de exceções): arquivo com essas chamadas também roda o núcleo.
    apps/desktop/*)
      ui=1
      case "$f" in
        apps/desktop/src/*.test.* | apps/desktop/src/*/mock/*) ;;
        apps/desktop/src/*.ts | apps/desktop/src/*.tsx)
          if uses_core "$f"; then rust=1; fi
          ;;
      esac
      ;;
    # Só o job audit confere estes.
    THIRD_PARTY_LICENSES.md | scripts/deps-check.py | scripts/deps-approved.txt | \
      scripts/third-party-licenses.py)
      audit=1
      ;;
    # Nenhum job lê estes: documentação, scripts da release e de uso local, modelos do GitHub.
    docs/* | *.md | LICENSE | NOTICE | .gitignore | .githooks/* | scripts/* | \
      .github/ISSUE_TEMPLATE/* | .github/pull_request_template.md | .github/FUNDING.yml | \
      .github/CODEOWNERS | .github/dependabot.yml)
      ;;
    # Workflows e qualquer caminho que esta lista não conhece: roda tudo.
    *)
      all=1
      ;;
  esac
}

# O arquivo tem (ou tinha, na base) uma chamada `useCore<`? Sem git (modo --classify), supõe que sim.
uses_core() {
  local f="$1" rev
  [ "${NO_GIT:-0}" = 1 ] && return 0
  for rev in "$head" "$base"; do
    [ -n "$rev" ] || continue
    if git cat-file -e "$rev:$f" 2>/dev/null && git show "$rev:$f" | grep -q 'useCore<'; then
      return 0
    fi
  done
  return 1
}

classify_stdin() {
  local f
  while IFS= read -r f; do
    if [ -n "$f" ]; then classify_path "$f"; fi
  done
}

# SHA do último CI que passou, pelo evento (`push` só existe na main; `schedule` também). Vazio se
# não houver ou se a API falhar. Pelo evento e não pelo nome do branch: o PR de um fork feito a
# partir do `main` dele também aparece como branch `main`.
last_green() {
  local sha
  sha="$(gh api "repos/$GITHUB_REPOSITORY/actions/workflows/ci.yml/runs?status=success&per_page=1&$1" \
    --jq '.workflow_runs[0].head_sha // empty' 2>/dev/null)" || sha=""
  if is_sha "$sha"; then echo "$sha"; fi
}

# 40 dígitos hexadecimais, e não o SHA zerado de um branch novo.
is_sha() {
  [[ "$1" =~ ^[0-9a-f]{40}$ && "$1" != 0000000000000000000000000000000000000000 ]]
}

emit() {
  if [ "$all" = 1 ]; then rust=1 ui=1 audit=1; fi
  local e2e=0
  if [ "$rust" = 1 ] || [ "$ui" = 1 ]; then e2e=1 audit=1; fi
  local name value line
  for name in rust ui e2e audit bundle full; do
    value=false
    [ "${!name}" = 1 ] && value=true
    line="$name=$value"
    echo "$line"
    if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "$line" >>"$GITHUB_OUTPUT"; fi
  done
}

if [ "${1:-}" = "--classify" ]; then
  NO_GIT=1
  classify_stdin
  emit
  exit 0
fi

case "${GITHUB_EVENT_NAME:?}" in
  workflow_dispatch)
    echo "à mão: roda tudo"
    all=1 full=1 bundle=1
    ;;
  schedule)
    if [ "$(last_green event=schedule)" = "$GITHUB_SHA" ]; then
      echo "a main não mudou desde a última noite que passou: só o audit (avisos novos de segurança)"
      audit=1
    else
      echo "execução da noite: roda tudo"
      all=1 full=1 bundle=1
    fi
    ;;
  pull_request)
    # O checkout de um PR é o commit de merge: o primeiro pai é a base.
    if git rev-parse -q --verify 'HEAD^2' >/dev/null; then
      base="$(git rev-parse 'HEAD^1')"
    fi
    ;;
  push)
    base="$(last_green event=push)"
    [ -n "$base" ] || base="${BEFORE_SHA:-}"
    is_sha "$base" || base=""
    if [ -n "$base" ] && ! git cat-file -e "$base^{commit}" 2>/dev/null; then
      git fetch --quiet --no-tags --depth=1 origin "$base" || base=""
    fi
    ;;
  *)
    all=1
    ;;
esac

case "$GITHUB_EVENT_NAME" in
  pull_request | push)
    if [ -z "$base" ]; then
      echo "sem base para comparar: roda tudo"
      all=1
    else
      echo "comparando com $base"
      files="$(git diff --name-only --no-renames "$base" "$head")"
      printf '%s\n' "$files" | sed 's/^/  /'
      classify_stdin <<<"$files"
    fi
    ;;
esac

emit
