#!/usr/bin/env python3
"""Auditoria dos indicadores fundamentalistas no universo inteiro (docs/indicators.md).

Recalcula por fora, direto dos arquivos de dados abertos da CVM e só com a biblioteca padrão
do Python, o que o núcleo calcula, e compara com o `treasury-dev serve` de um cofre de teste
já coletado (`research.sync` com fca, b3, dfp, itr e b3_listados). Quatro partes:

1. Extração: os valores-base de cada companhia (receita, lucro bruto, EBIT, lucro dos
   controladores pela regra da conferência, lucro total, caixa operacional, D&A, ativo,
   circulantes, caixa, dívida, patrimônio, não controladores) lidos dos CSV da CVM x os fatos
   que o núcleo guardou. Critério: nenhuma diferença.
2. Fórmulas: cada indicador refeito a partir dos fatos e da cotação que o próprio núcleo
   devolve. Critério: nenhuma diferença.
3. Conferência do núcleo: quantos indicadores saíram conferidos, corrigidos, com aviso ou
   bloqueados, e por quê.
4. Validade dos dados de entrada: identidades contábeis e campos que a CVM não confere.

Uso:
    python3 scripts/indicators-audit.py --download --cvm-dir /tmp/cvm --year 2026
    python3 scripts/indicators-audit.py --cvm-dir /tmp/cvm --year 2026 \\
        --api http://127.0.0.1:7777/api --token TOKEN [--report saida.md]

`--download` baixa (uma vez) DFP e ITR do ano e do anterior e o FCA do ano para `--cvm-dir`.
Os arquivos baixados são dados: o script só lê CSV, nunca executa nada deles. Sai com código 1
se as partes 1 ou 2 acharem diferença.
"""

from __future__ import annotations

import argparse
import collections
import csv
import datetime as dt
import io
import json
import os
import re
import sys
import unicodedata
import urllib.error
import urllib.request
import zipfile

csv.field_size_limit(10**9)
BASE = "https://dados.cvm.gov.br/dados/CIA_ABERTA/DOC"
STMTS = ["BPA", "BPP", "DRE", "DFC_MI", "DFC_MD", "DVA"]
UA = "Treasury-auditoria (app local de financas pessoais; +https://github.com/knuppe/treasury)"


# ---------------------------------------------------------------------------------------
# Arquivos da CVM


def download(cvm_dir: str, year: int) -> None:
    os.makedirs(cvm_dir, exist_ok=True)
    for kind, y in (("dfp", year - 1), ("dfp", year), ("itr", year - 1), ("itr", year), ("fca", year)):
        name = f"{kind}_cia_aberta_{y}"
        dest = os.path.join(cvm_dir, name)
        if os.path.isdir(dest) and os.listdir(dest):
            continue
        url = f"{BASE}/{kind.upper()}/DADOS/{name}.zip"
        print(f"baixando {url}", file=sys.stderr)
        req = urllib.request.Request(url, headers={"User-Agent": UA})
        try:
            with urllib.request.urlopen(req, timeout=600) as r:
                data = r.read()
        except urllib.error.HTTPError as e:
            if e.code == 404:
                continue
            raise
        os.makedirs(dest, exist_ok=True)
        with zipfile.ZipFile(io.BytesIO(data)) as z:
            for m in z.namelist():
                # Só CSV, sem caminhos: nada de fora da pasta.
                if m.endswith(".csv") and "/" not in m and "\\" not in m:
                    with open(os.path.join(dest, m), "wb") as f:
                        f.write(z.read(m))


def norm(s: str) -> str:
    """Minúsculas, sem acentos nem pontuação (como o núcleo compara descrições de contas)."""
    s = unicodedata.normalize("NFD", s.lower())
    s = "".join(ch for ch in s if unicodedata.category(ch) != "Mn")
    return " ".join("".join(ch if ch.isalnum() or ch == " " else " " for ch in s).split())


def rows(path: str):
    with open(path, encoding="latin1", newline="") as f:
        yield from csv.DictReader(f, delimiter=";", quoting=csv.QUOTE_NONE)


def depth(code: str) -> int:
    return code.count(".")


class Filings:
    """Documentos (DFP/ITR) por companhia: última versão de cada data de referência."""

    def __init__(self, cvm_dir: str, year: int):
        # filings[cnpj][(kind, ref)] = {"version", "name", "data": {(stmt, cons): {(ordem, ini, fim): {conta: (desc, valor)}}}}
        self.f: dict = collections.defaultdict(dict)
        self.shares: dict = {}
        self.year = year
        for kind, y in (("dfp", year - 1), ("dfp", year), ("itr", year - 1), ("itr", year)):
            folder = os.path.join(cvm_dir, f"{kind}_cia_aberta_{y}")
            if not os.path.isdir(folder):
                continue
            for st in STMTS:
                for basis in ("con", "ind"):
                    p = os.path.join(folder, f"{kind}_cia_aberta_{st}_{basis}_{y}.csv")
                    if os.path.exists(p):
                        self._read(kind, st, basis == "con", p)
            p = os.path.join(folder, f"{kind}_cia_aberta_composicao_capital_{y}.csv")
            if os.path.exists(p):
                for r in rows(p):
                    k = (r["CNPJ_CIA"], kind, r["DT_REFER"])
                    if k not in self.shares or int(r["VERSAO"]) > int(self.shares[k]["VERSAO"]):
                        self.shares[k] = r

    def _read(self, kind: str, st: str, cons: bool, path: str) -> None:
        for r in rows(path):
            code = r["CD_CONTA"]
            d = depth(code)
            if d > 3 and not (d == 4 and code.startswith(("2.01.05.", "2.02.02."))):
                continue
            key = (kind, r["DT_REFER"])
            v = int(r["VERSAO"])
            fl = self.f[r["CNPJ_CIA"]].get(key)
            if fl is None or v > fl["version"]:
                fl = self.f[r["CNPJ_CIA"]][key] = {
                    "version": v,
                    "name": r["DENOM_CIA"],
                    "data": collections.defaultdict(lambda: collections.defaultdict(dict)),
                }
            elif v < fl["version"]:
                continue
            val = float(r["VL_CONTA"])
            if not code.startswith("3.99") and r["ESCALA_MOEDA"].upper().startswith("MIL"):
                val *= 1000
            ordem = "P" if r["ORDEM_EXERC"].upper().startswith("PEN") else "U"
            fl["data"][(st, cons)][(ordem, r.get("DT_INI_EXERC") or "", r["DT_FIM_EXERC"])][code] = (
                norm(r["DS_CONTA"]),
                val,
            )


# ---------------------------------------------------------------------------------------
# Regras de leitura (as mesmas do núcleo, escritas de novo)


def children(l: dict, parent: str):
    want = depth(parent) + 1
    return [(c, l[c][0], l[c][1]) for c in sorted(l) if c.startswith(parent + ".") and depth(c) == want]


def level2(l: dict, root: str, prefix: str):
    for c in sorted(l):
        if c.startswith(root + ".") and depth(c) == 1 and l[c][0].startswith(prefix):
            return c, l[c][1]
    return None


def ni_line(l: dict):
    for c in sorted(l, reverse=True):
        d = l[c][0]
        if (c.startswith("3.") and depth(c) == 1 and c != "3.99" and d.startswith("lucro")
                and not d.startswith("lucro por") and "antes" not in d and "continuad" not in d):
            return c, l[c][1]
    return None


def ni_parts(l: dict):
    k = ni_line(l)
    if not k:
        return None
    ctrl, minority = None, 0.0
    for _, d, v in children(l, k[0]):
        if "controladora" in d:
            ctrl = v
        elif "nao controladores" in d:
            minority += v
    return k[1], ctrl, minority


def controllers_income(parts, individual, bank):
    total, ctrl, minority = parts
    ind = individual if individual else None
    if bank and ind is not None:
        return ind
    if ctrl is None:
        return total
    if ctrl != 0:
        return ctrl
    rest = total - minority
    if abs(rest) <= 1:
        return ctrl
    return ind if ind is not None else rest


def equity_parts(l: dict):
    eq = level2(l, "2", "patrimonio liquido")
    if not eq:
        eq = next(((c, l[c][1]) for c in sorted(l, reverse=True)
                   if c.startswith("2.") and depth(c) == 1 and "patrimonio" in l[c][0]), None)
    if not eq:
        return None
    m = next(((v, True) for _, d, v in children(l, eq[0]) if "nao controladores" in d), (0.0, False))
    return eq[1], m[0], m[1]


def layout_of(l: dict) -> str:
    d = l.get("3.01", ("", 0))[0]
    if "intermediacao financeira" in d:
        return "bank"
    if "segur" in d or "previdencia" in d or "resseguro" in d:
        return "insurance"
    return "industrial"


def has_content(l: dict) -> bool:
    return any(v[1] for v in l.values())


def facts_of(fl_by_key: dict, shares: dict, cnpj: str):
    """Fatos por (item, tipo, início, fim) -> (prioridade, valor). O original vence a reapresentação."""
    out: dict = {}
    lay = "industrial"

    def put(item, pt, s, e, v, pri):
        k = (item, pt, s, e)
        if k not in out or pri >= out[k][0]:
            out[k] = (pri, v)

    for (kind, ref), fl in sorted(fl_by_key.items(), key=lambda x: x[0][1]):
        data = fl["data"]
        nz = lambda st, code: any(o == "U" and code in l and l[code][1] != 0 for (o, _, _), l in data.get((st, True), {}).items())
        cons = nz("BPA", "1") or nz("DRE", "3.01")
        pick = lambda st: data.get((st, cons)) or data.get((st, not cons)) or {}
        ind = lambda st: data.get((st, False)) if cons else None
        dre = pick("DRE")
        lay = next((layout_of(l) for (o, _, _), l in dre.items() if o == "U"), "industrial")
        industrial = lay == "industrial"

        def classify(groups):
            res = []
            for ordem in ("U", "P"):
                keys = [k for k in groups if k[0] == ordem and k[1]]
                ystart = min((k[1] for k in keys), default=None)
                for k in keys:
                    days = (dt.date.fromisoformat(k[2]) - dt.date.fromisoformat(k[1])).days + 1
                    if kind == "dfp" and 300 <= days <= 400:
                        res.append((k, "annual"))
                    elif kind == "itr" and k[1] == ystart and days <= 300:
                        res.append((k, "ytd"))
            return res

        dre_ind = ind("DRE") or {}
        for k, pt in classify(dre):
            l = dre[k]
            pri = 2 if k[0] == "U" else 1
            if "3.01" in l:
                put("revenue", pt, k[1], k[2], l["3.01"][1], pri)
            if "3.03" in l and "resultado bruto" in l["3.03"][0]:
                put("gross_profit", pt, k[1], k[2], l["3.03"][1], pri)
            if industrial and "3.05" in l and "antes do resultado financeiro" in l["3.05"][0]:
                put("ebit", pt, k[1], k[2], l["3.05"][1], pri)
            p = ni_parts(l)
            if p:
                li = dre_ind.get(k)
                i = ni_line(li)[1] if li and has_content(li) and ni_line(li) else None
                put("net_income", pt, k[1], k[2], controllers_income(p, i, lay == "bank"), pri)
                put("net_income_total", pt, k[1], k[2], p[0], pri)
        cf = pick("DFC_MI") or pick("DFC_MD")
        dva = pick("DVA")
        for k, pt in classify(cf):
            l = cf[k]
            pri = 2 if k[0] == "U" else 1
            if "6.01" in l:
                put("cfo", pt, k[1], k[2], l["6.01"][1], pri)
            da_dva = next((abs(v) for c, (d, v) in sorted(dva.get(k, {}).items()) if d.startswith("depreciacao amortizacao")), None)
            da_dfc = sum(abs(v) for _, d, v in children(l, "6.01.01") if "deprecia" in d or "exaust" in d or "deplec" in d)
            if da_dva is not None:
                put("da", pt, k[1], k[2], da_dva, pri)
            elif industrial and da_dfc:
                put("da", pt, k[1], k[2], da_dfc, pri)
        for k, pt in classify(dva):
            l = dva[k]
            da_dva = next((abs(v) for c, (d, v) in sorted(l.items()) if d.startswith("depreciacao amortizacao")), None)
            if da_dva is not None:
                put("da", pt, k[1], k[2], da_dva, 2 if k[0] == "U" else 1)
        bpp_ind = ind("BPP") or {}
        for (o, s, e), l in pick("BPA").items():
            pri = 2 if o == "U" else 1
            if "1" in l:
                put("total_assets", "instant", "", e, l["1"][1], pri)
            if "1.01" in l and "circulante" in l["1.01"][0]:
                put("current_assets", "instant", "", e, l["1.01"][1], pri)
                c = l["1.01.01"][1] if "1.01.01" in l and "caixa" in l["1.01.01"][0] else None
                a = l["1.01.02"][1] if "1.01.02" in l and "aplicac" in l["1.01.02"][0] else None
                if c is not None or a is not None:
                    put("cash", "instant", "", e, (c or 0) + (a or 0), pri)
            elif "1.01" in l and "caixa" in l["1.01"][0]:
                put("cash", "instant", "", e, l["1.01"][1], pri)
        for key, l in pick("BPP").items():
            o, s, e = key
            pri = 2 if o == "U" else 1
            if "2.01" in l and "circulante" in l["2.01"][0] and "nao circulante" not in l["2.01"][0]:
                put("current_liabilities", "instant", "", e, l["2.01"][1], pri)
            if industrial:
                st_ = l["2.01.04"][1] if "2.01.04" in l and "emprestimo" in l["2.01.04"][0] else None
                lt = l["2.02.01"][1] if "2.02.01" in l and "emprestimo" in l["2.02.01"][0] else None
                if st_ is not None or lt is not None:
                    put("gross_debt", "instant", "", e, (st_ or 0) + (lt or 0), pri)
            ep = equity_parts(l)
            if ep:
                total, minority, has_m = ep
                eq = total - minority
                if cons and lay == "bank":
                    li = bpp_ind.get(key)
                    ei = equity_parts(li) if li and has_content(li) else None
                    if ei:
                        eq = ei[0] - ei[1]
                put("equity", "instant", "", e, eq, pri)
                if has_m:
                    put("minority", "instant", "", e, minority, pri)
                if "2" in l:
                    put("total_liabilities", "instant", "", e, l["2"][1] - total, pri)
    return out, lay


def add_months(d: dt.date, n: int) -> dt.date:
    y, m = divmod(d.year * 12 + d.month - 1 + n, 12)
    m += 1
    last = (dt.date(y + (m == 12), m % 12 + 1, 1) - dt.timedelta(days=1)).day
    return dt.date(y, m, last if d.day >= 28 else min(d.day, last))


def near(a: dt.date, b: dt.date) -> bool:
    return abs((a - b).days) <= 10


def ltm(facts: dict, item: str, at: dt.date):
    fl = [(k, v[1]) for k, v in facts.items() if k[0] == item and k[1] in ("annual", "ytd")]
    a = next((v for k, v in fl if k[1] == "annual" and near(dt.date.fromisoformat(k[3]), at)), None)
    if a is not None:
        return a
    cs = [(k, v) for k, v in fl if k[1] == "ytd" and near(dt.date.fromisoformat(k[3]), at)]
    if not cs:
        return None
    ck, cv = max(cs, key=lambda x: (dt.date.fromisoformat(x[0][3]) - dt.date.fromisoformat(x[0][2])).days)
    s = dt.date.fromisoformat(ck[2])
    an = next((v for k, v in fl if k[1] == "annual" and near(dt.date.fromisoformat(k[3]), s - dt.timedelta(days=1))), None)
    pe, ps = add_months(at, -12), add_months(s, -12)
    p = next((v for k, v in fl if k[1] == "ytd" and near(dt.date.fromisoformat(k[3]), pe) and near(dt.date.fromisoformat(k[2]), ps)), None)
    if an is None or p is None:
        return None
    return an + cv - p


def reference(facts: dict):
    for item in ("net_income", "revenue"):
        e = [dt.date.fromisoformat(k[3]) for k in facts if k[0] == item and k[1] in ("annual", "ytd")]
        if e:
            return max(e)
    return None


def latest_instant(facts: dict, item: str, today: dt.date):
    c = [(dt.date.fromisoformat(k[3]), v[1]) for k, v in facts.items() if k[0] == item and k[1] == "instant" and dt.date.fromisoformat(k[3]) <= today]
    return max(c) if c else None


# ---------------------------------------------------------------------------------------
# API do treasury-dev


class Api:
    def __init__(self, url: str, token: str):
        self.url, self.token = url, token

    def call(self, cmd: str, args: dict | None = None):
        req = urllib.request.Request(
            self.url,
            data=json.dumps({"cmd": cmd, "args": args or {}}).encode(),
            headers={"Authorization": f"Bearer {self.token}", "Content-Type": "application/json"},
        )
        try:
            with urllib.request.urlopen(req, timeout=600) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            return json.loads(e.read())


def tickers_from_fca(cvm_dir: str, year: int) -> list[str]:
    p = os.path.join(cvm_dir, f"fca_cia_aberta_{year}", f"fca_cia_aberta_valor_mobiliario_{year}.csv")
    out = set()
    for r in rows(p):
        t = r["Codigo_Negociacao"].strip().upper()
        if re.fullmatch(r"(?=[A-Z0-9]*[A-Z])[A-Z0-9]{4}\d{1,2}", t):
            out.add(t)
    return sorted(out)


# ---------------------------------------------------------------------------------------
# Partes 1 e 2


FLOW = ["revenue", "gross_profit", "ebit", "net_income", "net_income_total", "cfo", "da"]
INST = ["total_assets", "current_assets", "current_liabilities", "cash", "gross_debt", "equity", "minority", "total_liabilities"]


def close(a, b, rel=1e-6, absol=1.0):
    return a is not None and b is not None and abs(a - b) <= max(absol, rel * max(abs(a), abs(b)))


def check_extraction(fl: Filings, universe: dict, today: dt.date):
    diffs = collections.defaultdict(list)
    n = 0
    seen = {}
    for t, v in sorted(universe.items()):
        if "fund" in v:
            seen.setdefault(v["fund"]["subject"]["issuer_code"], t)
    for cnpj, t in seen.items():
        if cnpj not in fl.f:
            continue
        f = universe[t]["fund"]
        eng = {x["item"]: x for x in f["latest"]}
        facts, _ = facts_of(fl.f[cnpj], fl.shares, cnpj)
        at = reference(facts)
        if at is None or str(at) != str(f.get("reference_date")):
            continue
        n += 1
        bal_dates = [d for d in (latest_instant(facts, "total_assets", today), latest_instant(facts, "equity", today)) if d]
        bal = max(d for d, _ in bal_dates) if bal_dates else None
        for item in FLOW + INST:
            e = eng.get(item)
            if item in FLOW:
                ev = float(e["value"]) if e and e["period_type"] == "ltm" else None
                mine = ltm(facts, item, at)
            else:
                ev = float(e["value"]) if e and e["period_end"] == str(bal) else None
                li = latest_instant(facts, item, today)
                mine = li[1] if li and bal and li[0] == bal else None
            if ev is None and mine is None:
                continue
            if mine is None and item in FLOW and at.year < fl.year:
                # Balanço de um ano anterior: os 12 meses pedem a DFP de dois anos atrás, que o
                # script não baixa.
                continue
            if not close(ev, mine):
                diffs[item].append((t, ev, mine))
    return n, diffs


def check_formulas(universe: dict):
    bad = collections.defaultdict(list)
    count = collections.Counter()
    for t, v in sorted(universe.items()):
        if "fund" not in v or "indicators" not in (v.get("ind") or {}):
            continue
        ind = {i["key"]: i for i in v["ind"]["indicators"]}
        notes = " ".join(v["ind"].get("notes", []))
        lay = v["fund"]["subject"].get("layout")
        fin = lay in ("bank", "insurance")
        holding = "Holding ou seguradora" in notes or "Holding company or insurer" in notes
        latest = v["fund"]["latest"]
        inst_dates = [x["period_end"] for x in latest if x["item"] in ("total_assets", "equity") and x["period_type"] == "instant"]
        bal = max(inst_dates) if inst_dates else None
        F = {}
        for x in latest:
            if x["period_type"] == "ltm" or (x["period_type"] == "instant" and (x["period_end"] == bal or x["item"].startswith("shares"))):
                F[x["item"]] = float(x["value"])
        g = F.get
        val = lambda k: ind.get(k, {}).get("value")
        mc = val("market_cap")
        div = lambda a, b: None if a is None or b in (None, 0) else a / b
        ni, nit = g("net_income"), g("net_income_total", g("net_income"))
        rev, ebit, da, gp, eq, assets = g("revenue"), g("ebit"), g("da"), g("gross_profit"), g("equity"), g("total_assets")
        debt, cash, ca, cl, liab, minority = g("gross_debt"), g("cash"), g("current_assets"), g("current_liabilities"), g("total_liabilities"), g("minority") or 0.0
        nd = debt - (cash or 0) if debt is not None else None
        ev = mc + nd + minority if mc is not None and nd is not None else None
        ebitda = ebit + da if ebit is not None and da is not None else None
        exp = {
            "pl": div(mc, ni) if ni else None,
            "pvp": div(mc, eq) if eq and eq > 0 else None,
            "p_ativo": div(mc, assets),
            "roe": div(ni, eq) if eq and eq > 0 else None,
            "roa": div(nit, assets) if assets and assets > 0 else None,
        }
        if not fin:
            exp["net_debt_equity"] = div(nd, eq) if eq and eq > 0 else None
            exp["current_ratio"] = div(ca, cl)
            exp["gross_debt_equity"] = div(debt, eq) if eq and eq > 0 else None
        if not fin and not holding:
            cap = eq + minority + nd if eq is not None and nd is not None else None
            exp.update({
                "ev": ev,
                "psr": div(mc, rev) if rev and rev > 0 else None,
                "ev_ebitda": div(ev, ebitda) if ebitda and ebitda > 0 else None,
                "ev_ebit": div(ev, ebit) if ebit and ebit > 0 else None,
                "roic": div(ebit * 0.66, cap) if ebit is not None and cap and cap > 0 and not (assets and cap < assets / 10) else None,
                "gross_margin": div(gp, rev) if rev and rev > 0 else None,
                "ebit_margin": div(ebit, rev) if rev and rev > 0 else None,
                "net_margin": div(nit, rev) if rev and rev > 0 else None,
                "ebitda_margin": div(ebitda, rev),
                "net_debt_ebitda": div(nd, ebitda) if ebitda and ebitda > 0 else None,
                "earnings_yield": div(ebit, ev),
                "p_ebit": div(mc, ebit) if ebit and ebit > 0 else None,
                "p_working_capital": div(mc, ca - cl) if ca is not None and cl is not None and ca > cl else None,
                "p_ncav": div(mc, ca - liab) if ca is not None and liab is not None and ca > liab else None,
                "ebit_assets": div(ebit, assets) if assets and assets > 0 else None,
                "asset_turnover": div(rev, assets) if assets and assets > 0 else None,
            })
        for key, e in exp.items():
            i = ind.get(key, {})
            if i.get("check") == "blocked":
                continue
            o = i.get("value")
            count[key] += 1
            if e is None and o is None:
                continue
            if not close(e, o, 1e-6, 1e-9):
                bad[key].append((t, o, e, i.get("unavailable_reason")))
    return count, bad


# ---------------------------------------------------------------------------------------
# Partes 3 e 4


def check_states(universe: dict):
    states = collections.defaultdict(collections.Counter)
    notes = collections.defaultdict(list)
    for t, v in sorted(universe.items()):
        for i in (v.get("ind") or {}).get("indicators", []):
            states[i["key"]][i.get("check", "?")] += 1
            if i.get("check") in ("corrected", "warning", "blocked") and i["key"] in ("pl", "pvp", "market_cap"):
                for n in i.get("check_notes", []):
                    if (t, n) not in notes[i["check"]]:
                        notes[i["check"]].append((t, n))
    return states, notes


def check_inputs(fl: Filings, listed: set, year: int):
    tol = lambda ref: max(2000.0, 1e-5 * abs(ref))
    res = collections.OrderedDict()

    def chk(name, ok):
        r = res.setdefault(name, [0, 0])
        r[0] += 1
        r[1] += 0 if ok else 1

    ref = f"{year}-09-30"
    for kind, refer, per in (("itr", ref, (f"{year}-01-01", ref)), ("dfp", f"{year-1}-12-31", (f"{year-1}-01-01", f"{year-1}-12-31"))):
        tag = f"{kind} {refer}"
        for cnpj in listed:
            x = fl.f.get(cnpj, {}).get((kind, refer))
            if not x:
                continue
            data = x["data"]
            nz = lambda st, code: any(o == "U" and code in l and l[code][1] != 0 for (o, _, _), l in data.get((st, True), {}).items())
            cons = nz("BPA", "1") or nz("DRE", "3.01")
            get = lambda st, key, c=None: (data.get((st, cons if c is None else c)) or {}).get(key, {})
            a, p = get("BPA", ("U", "", refer)), get("BPP", ("U", "", refer))
            d = get("DRE", ("U",) + per)
            if not (a and p and d):
                chk(f"{tag}: tem balanço e resultado", False)
                continue
            chk(f"{tag}: tem balanço e resultado", True)
            chk(f"{tag}: ativo total = passivo total", abs(a["1"][1] - p["2"][1]) <= tol(a["1"][1]))
            if layout_of(d) == "industrial":
                for tot, parts, nm in (("3.03", ("3.01", "3.02"), "resultado bruto = receita + custo"), ("3.05", ("3.03", "3.04"), "EBIT = bruto + despesas operacionais")):
                    if all(c in d for c in (tot,) + parts):
                        chk(f"{tag}: {nm}", abs(d[tot][1] - sum(d[c][1] for c in parts)) <= tol(d[tot][1]))
            parts = ni_parts(d)
            if parts and cons and parts[1] is not None and parts[0] != 0:
                zero = parts[1] == 0 and abs(parts[0] - parts[2]) > 1
                chk(f"{tag}: lucro dos controladores preenchido", not zero)
                di = get("DRE", ("U",) + per, False)
                if not zero and di and ni_line(di) and layout_of(d) != "bank":
                    i = ni_line(di)[1]
                    chk(f"{tag}: lucro dos controladores = individual (até 1%)", abs(parts[1] - i) <= max(tol(i), 0.01 * abs(i)))
    return res


# ---------------------------------------------------------------------------------------


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--cvm-dir", required=True)
    ap.add_argument("--year", type=int, default=dt.date.today().year)
    ap.add_argument("--download", action="store_true")
    ap.add_argument("--api", default="http://127.0.0.1:7777/api")
    ap.add_argument("--token")
    ap.add_argument("--report", help="grava o relatório em Markdown neste arquivo")
    a = ap.parse_args()
    if a.download:
        download(a.cvm_dir, a.year)
    if not a.token:
        print("baixado; para comparar, passe --token do treasury-dev serve", file=sys.stderr)
        return 0
    api = Api(a.api, a.token)
    today = dt.date.today()
    universe = {}
    for t in tickers_from_fca(a.cvm_dir, a.year):
        f = api.call("research.fundamentals", {"ticker": t})
        if not f.get("ok"):
            continue
        i = api.call("research.indicators", {"ticker": t})
        universe[t] = {"fund": f["data"], "ind": i.get("data") if i.get("ok") else {}}
    fl = Filings(a.cvm_dir, a.year)
    out = []
    n, ext = check_extraction(fl, universe, today)
    out.append(f"## 1. Extração\n\n{n} companhias comparadas, {sum(len(v) for v in ext.values())} diferenças.\n")
    for k, v in ext.items():
        out.append(f"- {k}: {len(v)} — " + "; ".join(f"{t} {e} x {m}" for t, e, m in v[:8]))
    cnt, bad = check_formulas(universe)
    out.append(f"\n## 2. Fórmulas\n\n{sum(cnt.values())} valores conferidos em {len(cnt)} indicadores, {sum(len(v) for v in bad.values())} diferenças.\n")
    for k, v in bad.items():
        out.append(f"- {k}: {len(v)} — " + "; ".join(f"{t} {o} x {e}" for t, o, e, _ in v[:8]))
    states, notes = check_states(universe)
    out.append("\n## 3. Conferência do núcleo\n")
    for key in ("market_cap", "pl", "pvp", "roe", "ev_ebitda", "net_margin", "dy"):
        c = states.get(key, {})
        out.append(f"- {key}: " + ", ".join(f"{s} {c[s]}" for s in ("verified", "corrected", "warning", "blocked", "unchecked") if c.get(s)))
    for st in ("blocked", "corrected", "warning"):
        if notes.get(st):
            out.append(f"\n{st} ({len(notes[st])}):")
            out.extend(f"- {t}: {n}" for t, n in notes[st])
    listed = {v["fund"]["subject"]["issuer_code"] for v in universe.values() if "fund" in v}
    out.append("\n## 4. Validade dos dados de entrada\n")
    for k, (tot, fails) in check_inputs(fl, listed, a.year).items():
        out.append(f"- {k}: {fails} falhas em {tot} ({fails / tot * 100:.1f}%)")
    text = "\n".join(out) + "\n"
    print(text)
    if a.report:
        with open(a.report, "w") as f:
            f.write(f"# Auditoria dos indicadores ({today})\n\n" + text)
    return 1 if ext or bad else 0


if __name__ == "__main__":
    sys.exit(main())
