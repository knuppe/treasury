#!/usr/bin/env bash
# Verificação de dados pessoais antes do commit (docs/development.md, "Dados pessoais").
# Olha só o que está sendo acrescentado (staged) e recusa:
#   1. documentos e imagens (PDF, OFX, planilhas, fotos, e-mails) fora das pastas de fixtures e
#      dos ícones;
#   2. CPF formatado com dígito verificador válido (fora os de exemplo);
#   3. qualquer termo da lista privada, guardada fora do repositório em
#      $TREASURY_PRIVATE_DIR/termos-privados.txt (padrão: ~/Treasury-privado), um por linha.
# Ligado por `git config core.hooksPath .githooks` (só neste repositório).
set -euo pipefail

private_dir="${TREASURY_PRIVATE_DIR:-$HOME/Treasury-privado}"
terms="$private_dir/termos-privados.txt"
problems=()

# 1. Documentos e imagens.
while IFS= read -r -d '' f; do
    case "$f" in
        */fixtures/* | apps/desktop/src-tauri/icons/* | crates/treasury-core/data/*) continue ;;
    esac
    shopt -s nocasematch
    if [[ "$f" =~ \.(pdf|ofx|qfx|qif|xls|xlsx|ods|png|jpe?g|heic|webp|gif|tiff?|eml|msg|db)$ ]]; then
        problems+=("$f: documento ou imagem fora de uma pasta fixtures/ (guarde em $private_dir)")
    fi
    shopt -u nocasematch
done < <(git diff --cached --name-only --diff-filter=AM -z)

# Linhas acrescentadas, como "arquivo<TAB>texto". O awk roda byte a byte (LC_ALL=C): fixtures
# em Latin-1 (extratos de bancos) não são UTF-8 válido e quebrariam o awk num terminal UTF-8.
added=$(git diff --cached -U0 --no-color --diff-filter=AM | LC_ALL=C awk '
    /^\+\+\+ / { file = substr($0, 7); next }
    /^\+/      { print file "\t" substr($0, 2) }
')

# 2. CPF válido.
cpf_ok() {
    local d=${1//[.-]/} s i r
    # Todos os dígitos iguais (000.000.000-00) passam na conta, mas não são CPF.
    [[ -z "${d//${d:0:1}/}" ]] && return 1
    for n in 9 10; do
        s=0
        for ((i = 0; i < n; i++)); do s=$((s + ${d:i:1} * (n + 1 - i))); done
        r=$(((s * 10) % 11 % 10))
        [[ $r -eq ${d:n:1} ]] || return 1
    done
}
while IFS=$'\t' read -r file cpf; do
    [[ -z "$cpf" || "$cpf" == "123.456.789-09" ]] && continue
    cpf_ok "$cpf" && problems+=("$file: CPF válido ($cpf); use 123.456.789-09 ou 000.000.000-00")
done < <(printf '%s\n' "$added" | LC_ALL=C awk -F'\t' '{
    line = $2
    while (match(line, /[0-9][0-9][0-9]\.[0-9][0-9][0-9]\.[0-9][0-9][0-9]-[0-9][0-9]/)) {
        print $1 "\t" substr(line, RSTART, RLENGTH)
        line = substr(line, RSTART + RLENGTH)
    }
}')

# 3. Termos da lista privada.
if [[ -f "$terms" ]]; then
    while IFS= read -r term; do
        term="${term%%#*}"
        term="$(printf '%s' "$term" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
        [[ -z "$term" ]] && continue
        while IFS= read -r file; do
            problems+=("$file: contém um termo da lista privada (\"$term\")")
        done < <(printf '%s\n' "$added" | grep -aiF -- "$term" | cut -f1 | sort -u || true)
    done < "$terms"
fi

if ((${#problems[@]})); then
    echo "Commit recusado: parece ter dado pessoal no que está sendo acrescentado." >&2
    printf '  - %s\n' "${problems[@]}" >&2
    echo "Troque por dados inventados (docs/development.md, \"Dados pessoais\")." >&2
    echo "Se for dado público ou fictício de propósito: git commit --no-verify." >&2
    exit 1
fi
