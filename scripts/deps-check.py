#!/usr/bin/env python3
"""Confere que nenhuma dependência nova entrou sem aprovação (docs/development.md, seção
"Dependências").

A lista aprovada (`scripts/deps-approved.txt`) tem o nome de cada crate do `Cargo.lock` e de
cada pacote npm do `apps/desktop/package-lock.json`, diretos e indiretos, sem a versão: subir a
versão de algo que já está na lista não pede nada; um nome novo pede. O arquivo está no
`.github/CODEOWNERS`, então acrescentar um nome é uma decisão do mantenedor.

Uso (da raiz do repositório):

    python3 -I scripts/deps-check.py            # falha com os nomes novos
    python3 -I scripts/deps-check.py --update   # reescreve a lista com o que está nos locks
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
APPROVED = ROOT / "scripts" / "deps-approved.txt"
HEADER = """\
# Dependências aprovadas (scripts/deps-check.py; política em docs/development.md).
# Um nome por linha: `cargo <crate>` ou `npm <pacote>`, diretos e indiretos, sem versão.
# Um nome novo aqui é uma decisão do mantenedor (.github/CODEOWNERS).
"""


def cargo_names() -> set[str]:
    # Cada `[[package]]` do Cargo.lock; os crates do próprio workspace não têm `source`. Sem
    # `tomllib`, que só existe a partir do Python 3.11.
    names = set()
    for block in (ROOT / "Cargo.lock").read_text().split("[[package]]")[1:]:
        name = re.search(r'^name = "([^"]+)"$', block, re.M)
        if name and re.search(r"^source = ", block, re.M):
            names.add(name.group(1))
    return names


def npm_names() -> set[str]:
    lock = json.loads((ROOT / "apps" / "desktop" / "package-lock.json").read_text())
    names = set()
    for path, meta in lock.get("packages", {}).items():
        if not path or meta.get("link"):
            continue
        names.add(meta.get("name") or path.rsplit("node_modules/", 1)[-1])
    return names


def current() -> set[str]:
    return {f"cargo {n}" for n in cargo_names()} | {f"npm {n}" for n in npm_names()}


def approved() -> set[str]:
    if not APPROVED.exists():
        return set()
    return {
        line.strip()
        for line in APPROVED.read_text().splitlines()
        if line.strip() and not line.startswith("#")
    }


def main() -> int:
    now = current()
    if "--update" in sys.argv[1:]:
        APPROVED.write_text(HEADER + "".join(f"{n}\n" for n in sorted(now)))
        print(f"{len(now)} dependências em {APPROVED.relative_to(ROOT)}")
        return 0
    ok = approved()
    new = sorted(now - ok)
    gone = sorted(ok - now)
    if gone:
        print("Saíram da árvore (pode tirar da lista com --update):")
        for n in gone:
            print(f"  {n}")
    if new:
        print("Dependências novas, fora da lista aprovada:")
        for n in new:
            print(f"  {n}")
        print(
            "\nSiga a política de docs/development.md (seção Dependências) e, com a aprovação "
            "do mantenedor, rode `python3 -I scripts/deps-check.py --update`."
        )
        return 1
    print(f"{len(now)} dependências, todas aprovadas")
    return 0


if __name__ == "__main__":
    sys.exit(main())
