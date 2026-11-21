#!/usr/bin/env python3
"""Gera os dados embutidos da "Linha da vida" (docs/life-expectancy.md §10):
`crates/treasury-core/data/longevity/wpp2024.zip`.

Nunca baixa nada: só lê arquivos locais. Os endereços para baixar à mão estão em `--urls`.

Uso:
    python3 -I scripts/longevity-data.py --urls              # o que baixar e de onde
    python3 -I scripts/longevity-data.py PASTA                # dados reais (ONU + IBGE)
    python3 -I scripts/longevity-data.py PASTA --sem-ibge     # só a ONU (sem a referência do IBGE)
    python3 -I scripts/longevity-data.py WPP --ibge IBGE      # os do IBGE noutra pasta
    python3 -I scripts/longevity-data.py --synthetic          # tábua sintética (desenvolvimento)
    ... [--out ARQUIVO]                                       # outro destino (padrão: o do núcleo)

PASTA tem os seis arquivos da ONU (World Population Prospects 2024, variante média, tábuas
completas idade a idade) e os três do IBGE (Tábuas Completas de Mortalidade 2024):

    WPP2024_Life_Table_Complete_Medium_{Both,Female,Male}_{1950-2023,2024-2100}.csv.gz
    ambos_os_sexos.xlsx  homens.xlsx  mulheres.xlsx

Formato do zip (o mesmo do contrato com a interface e do leitor em `longevity/data.rs`):

- `index.json`: `{"source": "wpp2024" | "synthetic", "countries": [ISO2, na ordem dos dados],
  "sexes": ["both", "female", "male"], "years": [2024, 2100], "ages": [0, 100],
  "cohort_years": [1950, 2026]}`.
- `qx.bin`: u16 little-endian, qx × 65535 arredondado, na ordem país, sexo, ano (2024..=2100),
  idade (0..=100). A idade 100 é o grupo aberto (100+): guarda a probabilidade anual de uma
  força de mortalidade constante que dá a expectativa da tábua nessa idade,
  q = 1 − exp(−1 / e100); o núcleo repete esse q depois dos 100.
- `e0_cohort.bin`: u16 LE, expectativa ao nascer por geração × 100, na ordem país, sexo, ano de
  nascimento (1950..=2026), pela diagonal das taxas de 1950 a 2100 (depois de 2100 repete
  2100); 0 = sem dado.
- `ibge2024.json` (só Brasil, referência de período): `{"both"|"female"|"male": {"qx":
  [0..=90], "ex": [0..=90]}}`, qx como probabilidade (0 a 1).

Expectativa restante em todo o app: e(a) = 0,5 + Σ_{k≥1} S(a+k), com S(a) = 1 e
S(x+1) = S(x) × (1 − q(x)) (mortes no meio do ano), igual ao `longevity/math.rs`.

Só biblioteca padrão (csv, gzip, zipfile, xml): rode com `python3 -I`.
"""

from __future__ import annotations

import argparse
import csv
import gzip
import io
import json
import math
import re
import struct
import sys
import xml.etree.ElementTree as ET
import zipfile
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crates/treasury-core/src/country/catalog.rs"
OUT = ROOT / "crates/treasury-core/data/longevity/wpp2024.zip"

SEXES = ["both", "female", "male"]
WPP_SEX = {"both": "Both", "female": "Female", "male": "Male"}
YEARS = (2024, 2100)  # anos guardados em qx.bin
AGES = (0, 100)  # 100 = grupo aberto
COHORTS = (1950, 2026)  # nascimentos com a expectativa ao nascer por geração
FIRST_YEAR = 1950  # primeiro ano das tábuas da ONU
WPP_PERIODS = ["1950-2023", "2024-2100"]
IBGE_FILES = {"both": "ambos_os_sexos.xlsx", "male": "homens.xlsx", "female": "mulheres.xlsx"}
IBGE_MAX_AGE = 90  # 90 ou mais

URLS = """\
ONU, World Population Prospects 2024 (licença CC BY 3.0 IGO):
  página de downloads (CSV, "Life Tables" > "Complete", variante média):
    https://population.un.org/wpp/downloads?folder=Standard%20Projections&group=CSV%20format
  arquivos (confira o endereço exato na página; o padrão costuma ser este):
{wpp}
  metodologia: https://population.un.org/wpp/assets/Files/WPP2024_Methodology-Report_Final.pdf

IBGE, Tábuas Completas de Mortalidade 2024 (publicadas em 28/11/2025):
  página: https://www.ibge.gov.br/estatisticas/sociais/populacao/9126-tabuas-completas-de-mortalidade.html
  arquivos ambos_os_sexos.xlsx, homens.xlsx e mulheres.xlsx (pasta xlsx da edição 2024, no
  FTP: https://ftp.ibge.gov.br/Tabuas_Completas_de_Mortalidade/Tabuas_Completas_de_Mortalidade_2024/)
  publicação: https://biblioteca.ibge.gov.br/visualizacao/periodicos/3097/tcmb_2024.pdf

Ponha os nove arquivos numa pasta só e rode:
  python3 -I scripts/longevity-data.py PASTA
"""

WPP_URL = (
    "https://population.un.org/wpp/assets/Excel%20Files/1_Indicator%20(Standard)/CSV_FILES/"
)


def wpp_names() -> list[str]:
    return [
        f"WPP2024_Life_Table_Complete_Medium_{WPP_SEX[s]}_{p}.csv.gz"
        for s in SEXES
        for p in WPP_PERIODS
    ]


def catalog_countries() -> list[str]:
    """Países do catálogo do app (`country/catalog.rs`), na ordem do arquivo."""
    src = CATALOG.read_text(encoding="utf-8")
    codes = re.findall(r'\(\s*"([A-Z]{2})"\s*,\s*"', src)
    if not codes:
        sys.exit(f"nenhum país em {CATALOG}")
    return codes


# ---------------------------------------------------------------------------
# Matemática (a mesma do núcleo)


def remaining(qfun, age: int, year: int) -> float:
    """Expectativa restante na idade `age` no ano `year`, seguindo a diagonal da geração."""
    s, total = 1.0, 0.5
    x, y = age, year
    while x < 220:
        s *= 1.0 - qfun(y, x)
        if s < 1e-12:
            break
        total += s
        x += 1
        y += 1
    return total


def open_group_q(e_open: float) -> float:
    """q anual de uma força de mortalidade constante com expectativa `e_open`."""
    return 1.0 - math.exp(-1.0 / max(e_open, 0.05))


# ---------------------------------------------------------------------------
# Dados reais: ONU


def _col(header: list[str], *names: str) -> int | None:
    low = [h.strip().lower() for h in header]
    for n in names:
        if n in low:
            return low.index(n)
    return None


def read_wpp(folder: Path, countries: list[str]):
    """q[sexo][país][ano][idade] (1950..=2100, 0..=100), com o grupo aberto convertido."""
    wanted = set(countries)
    q: dict = {s: {c: {} for c in countries} for s in SEXES}
    e_open: dict = {s: {c: {} for c in countries} for s in SEXES}
    for sex in SEXES:
        for period in WPP_PERIODS:
            path = folder / f"WPP2024_Life_Table_Complete_Medium_{WPP_SEX[sex]}_{period}.csv.gz"
            print(f"lendo {path.name}…", flush=True)
            with gzip.open(path, "rt", encoding="utf-8-sig", newline="") as f:
                reader = csv.reader(f)
                header = next(reader)
                ci = _col(header, "iso2_code", "iso2")
                ti = _col(header, "time", "year")
                ai = _col(header, "agegrpstart", "agestart", "age")
                qi = _col(header, "qx")
                ei = _col(header, "ex")
                vi = _col(header, "variant")
                si = _col(header, "sex")
                li = _col(header, "loctypeid")
                if None in (ci, ti, ai, qi, ei):
                    sys.exit(
                        f"{path.name}: colunas esperadas ISO2_code, Time, AgeGrpStart, qx, ex; "
                        f"cabeçalho: {header}"
                    )
                for row in reader:
                    iso = row[ci].strip()
                    if iso not in wanted:
                        continue
                    if vi is not None and row[vi].strip() not in ("", "Medium"):
                        continue
                    # Só países e áreas (LocTypeID 4), nunca agregados com o mesmo código.
                    if li is not None and row[li].strip() not in ("", "4"):
                        continue
                    if si is not None:
                        val = row[si].strip().lower()
                        ok = {"both": ("total", "both", "both sexes")}.get(sex, (sex,))
                        if val and val not in ok:
                            sys.exit(f"{path.name}: sexo inesperado {row[si]!r}")
                    year = int(float(row[ti]))
                    age = int(float(row[ai]))
                    if not (FIRST_YEAR <= year <= YEARS[1]) or not (AGES[0] <= age <= AGES[1]):
                        continue
                    qx = float(row[qi])
                    ex = float(row[ei])
                    if age == AGES[1]:
                        e_open[sex][iso][year] = ex
                        qx = open_group_q(ex)
                    if not (0.0 <= qx <= 1.0):
                        sys.exit(f"{path.name}: qx fora de 0..1 em {iso} {year} {age}: {qx}")
                    ages = q[sex][iso].setdefault(year, {})
                    if age in ages and abs(ages[age] - qx) > 1e-12:
                        sys.exit(f"{path.name}: linha repetida para {iso} {year} {age}")
                    ages[age] = qx
    missing = []
    for sex in SEXES:
        for c in countries:
            for y in range(FIRST_YEAR, YEARS[1] + 1):
                ages = q[sex][c].get(y, {})
                if len(ages) != AGES[1] - AGES[0] + 1:
                    missing.append(f"{c}/{sex}/{y} ({len(ages)} idades)")
    if missing:
        sys.exit(
            f"faltam {len(missing)} tábuas, ex.: {', '.join(missing[:8])}. "
            "Confira os arquivos e os países do catálogo."
        )
    return q, e_open


# ---------------------------------------------------------------------------
# Dados reais: IBGE (xlsx lido só com zipfile + xml)

NS = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
REL = "{http://schemas.openxmlformats.org/officeDocument/2006/relationships}"
PKG_REL = "{http://schemas.openxmlformats.org/package/2006/relationships}"


def _col_index(ref: str) -> int:
    n = 0
    for ch in ref:
        if not ch.isalpha():
            break
        n = n * 26 + (ord(ch.upper()) - 64)
    return n - 1


def xlsx_rows(path: Path):
    """Linhas da primeira planilha (listas de células como texto; vazias = None)."""
    with zipfile.ZipFile(path) as z:
        names = set(z.namelist())
        shared: list[str] = []
        if "xl/sharedStrings.xml" in names:
            root = ET.fromstring(z.read("xl/sharedStrings.xml"))
            for si in root.iter(f"{NS}si"):
                shared.append("".join(t.text or "" for t in si.iter(f"{NS}t")))
        wb = ET.fromstring(z.read("xl/workbook.xml"))
        first = wb.find(f"{NS}sheets")[0]
        rid = first.get(f"{REL}id")
        rels = ET.fromstring(z.read("xl/_rels/workbook.xml.rels"))
        target = next(r.get("Target") for r in rels.iter(f"{PKG_REL}Relationship")
                      if r.get("Id") == rid)
        sheet_path = target.lstrip("/") if target.startswith("/") else f"xl/{target}"
        sheet = ET.fromstring(z.read(sheet_path))
        for row in sheet.iter(f"{NS}row"):
            cells: dict[int, str] = {}
            for i, c in enumerate(row.findall(f"{NS}c")):
                ref = c.get("r")
                col = _col_index(ref) if ref else i
                kind = c.get("t")
                v = c.find(f"{NS}v")
                if kind == "s" and v is not None:
                    val = shared[int(v.text)]
                elif kind == "inlineStr":
                    val = "".join(t.text or "" for t in c.iter(f"{NS}t"))
                else:
                    val = v.text if v is not None else None
                if val is not None and val.strip() != "":
                    cells[col] = val.strip()
            if cells:
                yield [cells.get(i) for i in range(max(cells) + 1)]


def _number(s: str | None) -> float | None:
    if s is None:
        return None
    t = s.replace(" ", "").replace(" ", "")
    if re.fullmatch(r"-?\d+(\.\d+)?([eE]-?\d+)?", t):
        return float(t)
    # Texto no formato brasileiro (1.234,56).
    if re.fullmatch(r"-?\d{1,3}(\.\d{3})*(,\d+)?|-?\d+,\d+", t):
        return float(t.replace(".", "").replace(",", "."))
    return None


def read_ibge(path: Path) -> dict:
    """qx (0 a 1) e ex de 0 a 90+, de uma tábua completa do IBGE."""
    qx: dict[int, float] = {}
    ex: dict[int, float] = {}
    for row in xlsx_rows(path):
        cells = [c for c in row if c is not None]
        if len(cells) < 4:
            continue
        head = cells[0].lower().replace(" ", "")
        if head.startswith("90") and ("mais" in head or "+" in head):
            age = IBGE_MAX_AGE
        else:
            n = _number(cells[0])
            if n is None or n != int(n) or not (0 <= n <= IBGE_MAX_AGE):
                continue
            age = int(n)
        nums = [x for x in (_number(c) for c in cells[1:]) if x is not None]
        if len(nums) < 3 or age in qx:
            continue
        qx[age] = nums[0]
        ex[age] = nums[-1]
    ages = list(range(0, IBGE_MAX_AGE + 1))
    lost = [a for a in ages if a not in qx]
    if lost:
        sys.exit(f"{path.name}: faltam as idades {lost[:10]}… (formato inesperado da planilha)")
    q = [qx[a] for a in ages]
    if max(q) > 1.0:  # por mil
        q = [v / 1000.0 for v in q]
    e = [ex[a] for a in ages]
    if not (50 < e[0] < 95) or any(e[i] < e[i + 1] for i in range(5, IBGE_MAX_AGE)):
        sys.exit(f"{path.name}: expectativas estranhas (e0 = {e[0]}); confira a planilha")
    return {"qx": [round(v, 6) for v in q], "ex": [round(v, 2) for v in e]}


# ---------------------------------------------------------------------------
# Tábua sintética (desenvolvimento): Gompertz–Makeham que melhora com o ano


def synthetic(countries: list[str]):
    """Mesma forma de `read_wpp`, com taxas plausíveis (calibradas no Brasil de 2024:
    e0 ≈ 73 homens e 80 mulheres) e queda da mortalidade ano a ano."""
    c = 0.095

    def factor(code: str) -> float:
        if code == "BR":
            return 1.0
        return 0.55 + 1.05 * ((zlib.crc32(code.encode()) % 1000) / 999.0)

    def mu(x: float, sex: str, y: int, f: float) -> float:
        a = {"male": 0.0007, "female": 0.0004}[sex]
        b = {"male": 3.4e-5, "female": 2.15e-5}[sex] * f
        d = 0.036 * f
        h = {"male": 0.0016, "female": 0.0002}[sex]
        if y >= 2024:
            imp, dimp = math.exp(-0.008 * (y - 2024)), math.exp(-0.03 * (y - 2024))
        else:
            imp, dimp = math.exp(0.016 * (2024 - y)), math.exp(0.045 * (2024 - y))
        return (
            a * imp**0.5
            + b * imp * math.exp(c * x)
            + d * dimp * math.exp(-2.2 * x)
            + h * imp**0.5 * math.exp(-(((x - 23) / 7) ** 2))
        )

    def hazard(x: float, sex: str, y: int, f: float) -> float:
        # Platô da mortalidade nas idades muito altas.
        if sex == "both":
            return min(0.7, (mu(x, "male", y, f) + mu(x, "female", y, f)) / 2)
        return min(0.7, mu(x, sex, y, f))

    q: dict = {s: {} for s in SEXES}
    for code in countries:
        f = factor(code)
        for sex in SEXES:
            per_year = {}
            for y in range(FIRST_YEAR, YEARS[1] + 1):
                ages = {
                    x: min(0.999, 1.0 - math.exp(-hazard(x + 0.5, sex, y, f)))
                    for x in range(AGES[0], AGES[1])
                }
                # Grupo aberto: a expectativa aos 100 com as taxas dali em diante.
                e100 = remaining(
                    lambda _y, x, sex=sex, y=y: min(
                        0.999, 1.0 - math.exp(-hazard(x + 0.5, sex, y, f))
                    ),
                    AGES[1],
                    y,
                )
                ages[AGES[1]] = open_group_q(e100)
                per_year[y] = ages
            q[sex][code] = per_year
    return q


# ---------------------------------------------------------------------------
# Saída


def q_of(table: dict):
    """q(ano, idade) com os limites do app (depois de 2100 repete 2100; depois dos 100, o
    grupo aberto)."""

    def f(y: int, x: int) -> float:
        y = min(max(y, FIRST_YEAR), YEARS[1])
        return table[y][min(x, AGES[1])]

    return f


def build(q: dict, countries: list[str], source: str, ibge: dict | None) -> dict[str, bytes]:
    qx_vals: list[int] = []
    e0_vals: list[int] = []
    for code in countries:
        for sex in SEXES:
            table = q[sex][code]
            for y in range(YEARS[0], YEARS[1] + 1):
                for x in range(AGES[0], AGES[1] + 1):
                    qx_vals.append(max(0, min(65535, round(table[y][x] * 65535))))
            qf = q_of(table)
            for b in range(COHORTS[0], COHORTS[1] + 1):
                e0_vals.append(max(1, min(65535, round(remaining(qf, 0, b) * 100))))
    index = {
        "source": source,
        "countries": countries,
        "sexes": SEXES,
        "years": list(YEARS),
        "ages": list(AGES),
        "cohort_years": list(COHORTS),
    }
    files = {
        "index.json": json.dumps(index, ensure_ascii=False, indent=1).encode(),
        "qx.bin": struct.pack(f"<{len(qx_vals)}H", *qx_vals),
        "e0_cohort.bin": struct.pack(f"<{len(e0_vals)}H", *e0_vals),
    }
    if ibge is not None:
        files["ibge2024.json"] = json.dumps(ibge, separators=(",", ":")).encode()
    return files


def write_zip(path: Path, files: dict[str, bytes]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as z:
        for name, data in files.items():
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, data, compresslevel=9)
    path.write_bytes(buf.getvalue())


def summary(q: dict, countries: list[str], ibge: dict | None, path: Path, source: str) -> None:
    size = path.stat().st_size
    print(f"\n{path} ({size / 1024:.0f} KiB, fonte {source}, {len(countries)} países)")

    def period(code: str, sex: str, year: int, age: int) -> float:
        t = q[sex][code][year]
        return remaining(lambda _y, x: t[min(x, AGES[1])], age, year)

    def cohort(code: str, sex: str, birth: int) -> float:
        return remaining(q_of(q[sex][code]), 0, birth)

    rows = []
    if "BR" in countries:
        for sex in SEXES:
            line = (
                f"  BR {sex:<6} período 2024: e0 {period('BR', sex, 2024, 0):5.1f}, "
                f"e60 {period('BR', sex, 2024, 60):4.1f}"
            )
            if ibge:
                line += f" | IBGE 2024: e0 {ibge[sex]['ex'][0]:5.1f}, e60 {ibge[sex]['ex'][60]:4.1f}"
            line += f" | coorte 1985 {cohort('BR', sex, 1985):5.1f}, 2023 {cohort('BR', sex, 2023):5.1f}"
            rows.append(line)
        rows.append("  esperado (IBGE 2024): e0 76,6 / 73,3 h / 79,9 m; e60 22,6 / 20,8 h / 24,2 m")
    if "GB" in countries:
        rows.append(
            f"  GB coorte 2023: homens {cohort('GB', 'male', 2023):5.1f}, mulheres "
            f"{cohort('GB', 'female', 2023):5.1f} (ONS, base 2022: 86,7 e 90,0); período "
            f"2024: {period('GB', 'male', 2024, 0):5.1f} e {period('GB', 'female', 2024, 0):5.1f}"
        )
    if "US" in countries:
        rows.append(
            f"  US coorte 1990: homens {cohort('US', 'male', 1990):5.1f}, mulheres "
            f"{cohort('US', 'female', 1990):5.1f} (compare com as tábuas de coorte da SSA)"
        )
    print("\n".join(rows))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("folder", nargs="?", type=Path, help="pasta com os CSVs da ONU e os xlsx do IBGE")
    ap.add_argument("--synthetic", action="store_true", help="gera a tábua sintética (desenvolvimento)")
    ap.add_argument("--sem-ibge", action="store_true", help="não inclui a referência do IBGE")
    ap.add_argument("--ibge", type=Path, help="pasta dos xlsx do IBGE (padrão: a mesma da ONU)")
    ap.add_argument("--urls", action="store_true", help="mostra o que baixar e de onde (não baixa)")
    ap.add_argument("--out", type=Path, default=OUT, help=f"arquivo de saída (padrão: {OUT})")
    a = ap.parse_args()

    wpp_list = "\n".join(f"    {WPP_URL}{n}" for n in wpp_names())
    if a.urls:
        print(URLS.format(wpp=wpp_list))
        return
    countries = catalog_countries()
    if a.synthetic:
        q = synthetic(countries)
        ibge = None
        if "BR" in countries and not a.sem_ibge:
            # Referência de período no formato do IBGE, tirada da própria tábua sintética.
            ibge = {}
            for sex in SEXES:
                t = q[sex]["BR"][2024]
                ibge[sex] = {
                    "qx": [round(t[x], 6) for x in range(IBGE_MAX_AGE)] + [1.0],
                    "ex": [
                        round(remaining(lambda _y, x, t=t: t[min(x, AGES[1])], age, 2024), 2)
                        for age in range(IBGE_MAX_AGE + 1)
                    ],
                }
        files = build(q, countries, "synthetic", ibge)
        write_zip(a.out, files)
        summary(q, countries, ibge, a.out, "synthetic")
        print("\nAtenção: dados SINTÉTICOS, só para desenvolvimento (index.json source = synthetic).")
        return
    if a.folder is None:
        ap.error("informe a PASTA com os arquivos (ou --synthetic, ou --urls)")
    ibge_dir = a.ibge or a.folder
    needed = [a.folder / n for n in wpp_names()]
    if not a.sem_ibge:
        needed += [ibge_dir / n for n in IBGE_FILES.values()]
    absent = [str(n) for n in needed if not n.is_file()]
    if absent:
        print("Faltam arquivos:\n  " + "\n  ".join(absent))
        print("\n" + URLS.format(wpp=wpp_list))
        sys.exit(1)
    q, _ = read_wpp(a.folder, countries)
    ibge = None
    if not a.sem_ibge and "BR" in countries:
        ibge = {sex: read_ibge(ibge_dir / IBGE_FILES[sex]) for sex in SEXES}
    files = build(q, countries, "wpp2024", ibge)
    write_zip(a.out, files)
    summary(q, countries, ibge, a.out, "wpp2024")


if __name__ == "__main__":
    main()
