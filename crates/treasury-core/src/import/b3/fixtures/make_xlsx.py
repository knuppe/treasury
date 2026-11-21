"""Gera as planilhas .xlsx de teste da importação B3 (dados inventados).

Só biblioteca padrão (zipfile): um .xlsx mínimo com strings inline, números e datas com
formato de data (numFmt 14), para exercitar a leitura pelo calamine. As entradas do zip
têm data fixa, então rodar de novo gera os mesmos bytes.

    python3 make_xlsx.py   # na pasta fixtures/
"""

import csv
import datetime
import os
import zipfile
from xml.sax.saxutils import escape

HERE = os.path.dirname(os.path.abspath(__file__))
EPOCH = datetime.date(1899, 12, 30)


class Date:
    """Célula de data gravada como número de série com formato de data."""

    def __init__(self, text):
        d, m, y = (int(x) for x in text.split("/"))
        self.serial = (datetime.date(y, m, d) - EPOCH).days


def col(i):
    s = ""
    i += 1
    while i:
        i, r = divmod(i - 1, 26)
        s = chr(65 + r) + s
    return s


def cell(ref, v):
    if v is None or v == "":
        return ""
    if isinstance(v, Date):
        return f'<c r="{ref}" s="1"><v>{v.serial}</v></c>'
    if isinstance(v, (int, float)):
        return f'<c r="{ref}"><v>{v!r}</v></c>'
    return f'<c r="{ref}" t="inlineStr"><is><t xml:space="preserve">{escape(str(v))}</t></is></c>'


def sheet_xml(rows):
    out = []
    for r, row in enumerate(rows, start=1):
        cells = "".join(cell(f"{col(c)}{r}", v) for c, v in enumerate(row))
        out.append(f'<row r="{r}">{cells}</row>')
    return (
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
        f'<sheetData>{"".join(out)}</sheetData></worksheet>'
    )


def write_xlsx(path, sheets):
    ct = [
        '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>',
        '<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>',
    ]
    wb_sheets, rels = [], []
    for i, (name, _) in enumerate(sheets, start=1):
        ct.append(
            f'<Override PartName="/xl/worksheets/sheet{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>'
        )
        wb_sheets.append(f'<sheet name="{escape(name)}" sheetId="{i}" r:id="rId{i}"/>')
        rels.append(
            f'<Relationship Id="rId{i}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{i}.xml"/>'
        )
    n = len(sheets) + 1
    rels.append(
        f'<Relationship Id="rId{n}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
    )
    files = {
        "[Content_Types].xml": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
            '<Default Extension="xml" ContentType="application/xml"/>'
            + "".join(ct)
            + "</Types>"
        ),
        "_rels/.rels": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>'
            "</Relationships>"
        ),
        "xl/workbook.xml": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" '
            'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">'
            f'<sheets>{"".join(wb_sheets)}</sheets></workbook>'
        ),
        "xl/_rels/workbook.xml.rels": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            + "".join(rels)
            + "</Relationships>"
        ),
        "xl/styles.xml": (
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
            '<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
            '<fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>'
            '<fills count="1"><fill><patternFill patternType="none"/></fill></fills>'
            '<borders count="1"><border/></borders>'
            '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
            '<cellXfs count="2"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
            '<xf numFmtId="14" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/></cellXfs>'
            "</styleSheet>"
        ),
    }
    for i, (_, rows) in enumerate(sheets, start=1):
        files[f"xl/worksheets/sheet{i}.xml"] = sheet_xml(rows)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, content in files.items():
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, content.encode("utf-8"))


def num(s):
    """'1.925,00' → 1925.0; '50' → 50."""
    v = float(s.replace(".", "").replace(",", "."))
    return int(v) if v.is_integer() else v


def negociacao():
    """Mesmas linhas do negociacao.csv, como o Excel da B3 grava: números e datas tipados
    (metade das datas como texto), com uma linha de título antes do cabeçalho."""
    with open(os.path.join(HERE, "negociacao.csv"), encoding="utf-8") as f:
        rows = list(csv.reader(f, delimiter=";"))
    out = [["Negociação de Ativos (relatório fictício para testes)"], [], rows[0]]
    for i, r in enumerate(rows[1:]):
        date = Date(r[0]) if i % 2 == 0 else r[0]
        out.append([date, r[1], r[2], r[3], r[4], r[5], num(r[6]), num(r[7]), num(r[8])])
    write_xlsx(os.path.join(HERE, "negociacao.xlsx"), [("Negociação", out)])


def posicao():
    """Posição em 02/05/2024, uma aba por classe, com linhas de total como no relatório."""
    xp = "XP INVESTIMENTOS CCTVM S/A"
    nu = "NU INVEST CORRETORA DE VALORES S.A."
    fict = "CORRETORA FICTICIA DTVM LTDA"
    acoes_h = [
        "Produto", "Instituição", "Conta", "Código de Negociação", "CNPJ da Empresa",
        "Código ISIN / Distribuição", "Tipo", "Escriturador", "Quantidade",
        "Quantidade Disponível", "Quantidade Indisponível", "Motivo",
        "Preço de Fechamento", "Valor Atualizado",
    ]

    def acao(prod, inst, code, qty, close):
        return [prod, inst, "123456", code, "00.000.000/0001-00", "BR0000000000", "PN",
                "BANCO ESCRITURADOR FICTICIO", qty, qty, 0, "-", close, round(qty * close, 2)]

    acoes = [
        acoes_h,
        acao("PETR4 - PETROLEO BRASILEIRO S.A. PETROBRAS", xp, "PETR4", 62, 40.0),
        acao("ITSA4 - ITAUSA S.A.", xp, "ITSA4", 40, 5.1),
        acao("KLBN11 - KLABIN S.A.", xp, "KLBN11", 10, 21.5),
        acao("VALE3 - VALE S.A.", xp, "VALE3", 100, 65.0),
        acao("BBAS3 - BANCO DO BRASIL S.A.", fict, "BBAS3", 100, 28.0),
        ["", "", "", "", "", "", "", "", "", "", "", "", "Total", 11829.0],
    ]
    bdr = [
        acoes_h,
        acao("AAPL34 - APPLE INC", nu, "AAPL34", 10, 47.0),
        ["Total", "", "", "", "", "", "", "", "", "", "", "", "", 470.0],
    ]
    etf_h = [
        "Produto", "Instituição", "Conta", "Código de Negociação", "CNPJ do Fundo",
        "Código ISIN / Distribuição", "Tipo", "Quantidade", "Quantidade Disponível",
        "Quantidade Indisponível", "Motivo", "Preço de Fechamento", "Valor Atualizado",
    ]
    etf = [
        etf_h,
        ["IVVB11 - ISHARES S&P 500 FDO INV COTAS FDO INDICE", xp, "123456", "IVVB11",
         "00.000.000/0001-00", "BR0000000001", "Cotas", 5, 5, 0, "-", 290.0, 1450.0],
    ]
    fundos_h = [
        "Produto", "Instituição", "Conta", "Código de Negociação", "CNPJ do Fundo",
        "Código ISIN / Distribuição", "Tipo", "Administrador", "Quantidade",
        "Quantidade Disponível", "Quantidade Indisponível", "Motivo",
        "Preço de Fechamento", "Valor Atualizado",
    ]
    fundos = [
        fundos_h,
        ["HGLG11 - CSHG LOGISTICA FDO INV IMOB - FII", nu, "654321", "HGLG11",
         "00.000.000/0001-00", "BR0000000002", "Cotas", "ADMINISTRADORA FICTICIA", 10, 10, 0,
         "-", 165.0, 1650.0],
        ["MXRF11 - MAXI RENDA FDO INV IMOB - FII", nu, "654321", "MXRF11",
         "00.000.000/0001-00", "BR0000000003", "Cotas", "ADMINISTRADORA FICTICIA", 100, 100,
         0, "-", 10.5, 1050.0],
    ]
    tesouro = [
        ["Produto", "Instituição", "Código ISIN", "Indexador", "Vencimento", "Quantidade",
         "Quantidade Disponível", "Quantidade Indisponível", "Motivo", "Valor Aplicado",
         "Valor bruto", "Valor líquido", "Valor Atualizado"],
        ["Tesouro Selic 2029", xp, "BRSTNCLF0000", "SELIC", "01/03/2029", 0.5, 0.5, 0, "-",
         7000.0, 7120.0, 7100.0, 7120.0],
    ]
    emprestimo = [
        ["Produto", "Instituição", "Natureza", "Número de Contrato", "Modalidade", "OPA",
         "Liquidação antecipada", "Taxa", "Comissão", "Data de registro", "Data de vencimento",
         "Quantidade", "Preço de Fechamento", "Valor Atualizado"],
        ["PETR4 - PETROLEO BRASILEIRO S.A. PETROBRAS", xp, "Doador", "1", "Voluntário",
         "Não", "Sim", "0,5", "0", "10/03/2024", "10/06/2024", 20, 40.0, 800.0],
    ]
    write_xlsx(
        os.path.join(HERE, "posicao-2024-05-02.xlsx"),
        [
            ("Acoes", acoes),
            ("BDR", bdr),
            ("ETF", etf),
            ("Fundo de Investimento", fundos),
            ("Tesouro Direto", tesouro),
            ("Empréstimo de Ativos", emprestimo),
        ],
    )


if __name__ == "__main__":
    negociacao()
    posicao()
