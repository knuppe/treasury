#!/usr/bin/env python3
"""Gera THIRD_PARTY_LICENSES.md: as licenças e os avisos de copyright de tudo o que vai
dentro do app (crates Rust do Cargo.lock, pacotes npm de produção, fontes, modelos de OCR
e as bibliotecas C compiladas junto, como SQLCipher e OpenSSL).

MIT, BSD e Apache pedem o texto da licença e os avisos junto do binário; o arquivo vai na
raiz do repositório e dentro dos instaladores (`bundle.resources` do tauri.conf.json).

Uso (da raiz do repositório, sem compilar nada):

    python3 -I scripts/third-party-licenses.py            # reescreve o arquivo
    python3 -I scripts/third-party-licenses.py --check    # falha se estiver desatualizado

Precisa do `cargo` (só `cargo metadata`) e do `apps/desktop/node_modules` instalado.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "THIRD_PARTY_LICENSES.md"
DESKTOP = ROOT / "apps" / "desktop"

LICENSE_FILE = re.compile(r"^(LICEN[CS]E|COPYING|NOTICE|COPYRIGHT|UNLICENSE)([-._].*)?$", re.I)
# Bibliotecas C compiladas dentro do binário: os avisos delas ficam em subpastas.
NESTED = {"sqlcipher-src": ["sqlcipher"], "openssl-src": ["openssl"]}

# O que não vem de crate nem de pacote npm, com o arquivo da licença no repositório.
EXTRA = [
    (
        "Modelos do PaddleOCR (det.onnx, rec_latin.onnx, rec_latin_dict.txt)",
        "Apache-2.0",
        "crates/treasury-core/assets/ocr/LICENSE",
        "PaddlePaddle e colaboradores (https://github.com/PaddlePaddle/PaddleOCR); conversão da "
        "detecção para ONNX pelo RapidOCR (https://github.com/RapidAI/RapidOCR).",
    ),
    (
        "Bandeiras do seletor de idioma (flag-icons 7.2.3)",
        "MIT",
        "apps/desktop/src/i18n/flags/LICENSE",
        "Panayiotis Lipiridis e colaboradores (https://github.com/lipis/flag-icons).",
    ),
]


# Textos padrão para quem declara a licença sem trazer o arquivo. Apache e MPL vêm de
# arquivos com o texto oficial; MIT, BSD-3-Clause e Zlib são os modelos do SPDX.
STANDARD_FILES = {
    "Apache-2.0": "crates/treasury-core/assets/ocr/LICENSE",
}
STANDARD_TEXT = {
    "MIT": """Permission is hereby granted, free of charge, to any person obtaining a copy of this
software and associated documentation files (the "Software"), to deal in the Software
without restriction, including without limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.""",
    "BSD-3-Clause": """Redistribution and use in source and binary forms, with or without modification, are
permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of
   conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice, this list of
   conditions and the following disclaimer in the documentation and/or other materials
   provided with the distribution.
3. Neither the name of the copyright holder nor the names of its contributors may be used
   to endorse or promote products derived from this software without specific prior
   written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS
OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE
COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR
TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.""",
    "Zlib": """This software is provided 'as-is', without any express or implied warranty. In no event
will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial
applications, and to alter it and redistribute it freely, subject to the following
restrictions:

1. The origin of this software must not be misrepresented; you must not claim that you
   wrote the original software. If you use this software in a product, an acknowledgment
   in the product documentation would be appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be misrepresented
   as being the original software.
3. This notice may not be removed or altered from any source distribution.""",
}


def read(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace").replace("\r\n", "\n").strip()


def license_files(directory: pathlib.Path, nested: list[str]) -> list[pathlib.Path]:
    found = [p for p in sorted(directory.iterdir()) if p.is_file() and LICENSE_FILE.match(p.name)]
    for sub in nested:
        d = directory / sub
        if d.is_dir():
            found += [p for p in sorted(d.iterdir()) if p.is_file() and LICENSE_FILE.match(p.name)]
    return found


def rust_packages() -> list[dict]:
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    workspace = set(meta["workspace_members"])
    out = []
    for p in meta["packages"]:
        if p["id"] in workspace:
            continue
        directory = pathlib.Path(p["manifest_path"]).parent
        out.append(
            {
                "name": p["name"],
                "version": p["version"],
                "license": p.get("license") or "ver o arquivo",
                "authors": ", ".join(p.get("authors") or []),
                "files": license_files(directory, NESTED.get(p["name"], [])),
            }
        )
    return out


def npm_packages() -> list[dict]:
    lock = json.loads((DESKTOP / "package-lock.json").read_text(encoding="utf-8"))
    out = []
    for key, info in lock["packages"].items():
        if not key or info.get("dev") or info.get("devOptional"):
            continue
        directory = DESKTOP / key
        if not directory.is_dir():
            sys.exit(f"falta {key}: rode `npm ci` em apps/desktop")
        manifest = json.loads((directory / "package.json").read_text(encoding="utf-8"))
        author = manifest.get("author")
        if isinstance(author, dict):
            author = author.get("name", "")
        out.append(
            {
                "name": key.split("node_modules/")[-1],
                "version": info.get("version", ""),
                "license": info.get("license") or manifest.get("license") or "ver o arquivo",
                "authors": author or "",
                "files": license_files(directory, []),
            }
        )
    return out


def render() -> str:
    groups: dict[str, dict] = {}
    missing: list[str] = []

    def add(label: str, license_id: str, authors: str, files: list[pathlib.Path]) -> None:
        if not files:
            missing.append(f"{label} ({license_id})")
            return
        for f in files:
            text = read(f)
            h = hashlib.sha256(text.encode()).hexdigest()
            g = groups.setdefault(h, {"text": text, "items": []})
            entry = f"{label} ({license_id})" + (f", {authors}" if authors else "")
            if entry not in g["items"]:
                g["items"].append(entry)

    rust = rust_packages()
    npm = npm_packages()
    for p in rust:
        add(f"{p['name']} {p['version']}", p["license"], p["authors"], p["files"])
    for p in npm:
        add(f"{p['name']} {p['version']} (npm)", p["license"], p["authors"], p["files"])
    for label, license_id, path, authors in EXTRA:
        add(label, license_id, authors, [ROOT / path])

    lines = [
        "# Licenças de terceiros",
        "",
        "O Treasury é AGPL-3.0-only (`LICENSE`). Este arquivo reúne as licenças e os avisos de",
        "copyright dos componentes de terceiros que vão dentro do app: "
        f"{len(rust)} crates Rust, {len(npm)} pacotes npm de produção, os modelos de OCR e as",
        "bandeiras. Os dados embutidos (ONU, IBGE, Open Food Facts) estão em `DATA_LICENSES.md`,",
        "e o aviso da TradingView Lightweight Charts, em `NOTICE`.",
        "",
        "Gerado por `scripts/third-party-licenses.py`; não edite à mão. Cada texto aparece uma",
        "vez, com a lista dos componentes que o usam.",
        "",
    ]
    if missing:
        lines += [
            "## Sem arquivo de licença no pacote",
            "",
            "Estes componentes declaram a licença no manifesto, mas não trazem o arquivo; vale o",
            "texto padrão da licença indicada.",
            "",
        ]
        lines += [f"- {m}" for m in sorted(missing)]
        lines.append("")
    lines += ["## Textos padrão", ""]
    lines += [
        "Para os componentes da lista acima sem arquivo próprio. Quando a licença tem",
        "alternativas (\"MIT OR Apache-2.0\"), o Treasury usa o componente pela MIT.",
        "",
    ]
    for name, path in STANDARD_FILES.items():
        lines += [f"### {name}", "", "```text", read(ROOT / path), "```", ""]
    for name, text in STANDARD_TEXT.items():
        lines += [f"### {name}", "", "```text", text, "```", ""]
    mpl = [p for p in rust if p["license"] == "MPL-2.0" and p["files"]]
    if mpl:
        lines += ["### MPL-2.0", "", "```text", read(mpl[0]["files"][0]), "```", ""]
    ordered = sorted(groups.values(), key=lambda g: (-len(g["items"]), g["items"][0]))
    for i, g in enumerate(ordered, 1):
        lines += [f"## Texto {i}", "", "Usado por:", ""]
        lines += [f"- {item}" for item in sorted(g["items"])]
        lines += ["", "```text", g["text"].replace("```", "'''"), "```", ""]
    return "\n".join(lines)


def main() -> None:
    text = render()
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text(encoding="utf-8") != text:
            sys.exit("THIRD_PARTY_LICENSES.md desatualizado: rode scripts/third-party-licenses.py")
        return
    OUT.write_text(text, encoding="utf-8")
    print(f"{OUT.relative_to(ROOT)}: {len(text) // 1024} KiB")


if __name__ == "__main__":
    main()
