//! Reconhece os relatórios pelo cabeçalho e lê as linhas em registros tipados.
//!
//! Os nomes de coluna são comparados sem maiúsculas, acentos e pontuação, com variações
//! conhecidas ("Código de Negociação"/"Codigo Negociacao", "Preço unitário"/"Preço",
//! "Valor da Operação"/"Valor"). O cabeçalho pode não estar na primeira linha (títulos
//! antes da tabela) e pode se repetir numa aba com vários blocos: cada linha que parece
//! cabeçalho troca o mapeamento das linhas seguintes. Linhas de total são puladas.

use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::sheet::{Cell, SheetRow, Table, Workbook};
use super::{B3ReportKind, B3SheetInfo, B3Source, rules};
use crate::assets::AssetClass;
use crate::i18n::{args, t, t_args};
use crate::import::text::fold;
use crate::portfolio::TradeSide;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Field {
    // Negociação
    TradeDate,
    Side,
    Market,
    Code,
    Price,
    Value,
    // Movimentação
    Direction,
    Movement,
    Date,
    UnitPrice,
    // Comuns
    Product,
    Institution,
    Quantity,
    // Posição
    Account,
    Close,
    Updated,
    Indexer,
    Maturity,
    Issuer,
}

/// Apelidos de cada coluna, já dobrados, em ordem de preferência. `*` no fim = prefixo.
fn aliases(kind: B3ReportKind, f: Field) -> &'static [&'static str] {
    use B3ReportKind::*;
    use Field::*;
    match (kind, f) {
        (Trades, TradeDate) => &["data do negocio", "data negocio", "data do pregao", "data"],
        (Trades, Side) => &[
            "tipo de movimentacao",
            "tipo movimentacao",
            "compra venda",
            "c v",
            "tipo de operacao",
            "operacao",
            "tipo",
        ],
        (Trades, Market) => &["mercado"],
        (Trades, Institution) => &["instituicao", "corretora", "instituicao financeira"],
        (Trades, Code) => &[
            "codigo de negociacao",
            "codigo negociacao",
            "codigo do ativo",
            "codigo",
            "ativo",
            "ticker",
        ],
        (Trades, Quantity) => &["quantidade", "qtd", "qtde", "quantidade negociada"],
        (Trades, Price) => &[
            "preco",
            "preco unitario",
            "preco medio",
            "preco de negociacao",
        ],
        (Trades, Value) => &[
            "valor",
            "valor total",
            "valor da operacao",
            "valor operacao",
        ],

        (Movements, Direction) => &["entrada saida", "entrada ou saida", "credito debito", "e s"],
        (Movements, Date) => &["data", "data da movimentacao", "data movimentacao"],
        (Movements, Movement) => &["movimentacao", "tipo de movimentacao", "evento"],
        (Movements, Product) => &["produto", "ativo"],
        (Movements, Institution) => &["instituicao", "corretora", "instituicao financeira"],
        (Movements, Quantity) => &["quantidade", "qtd", "qtde"],
        (Movements, UnitPrice) => &["preco unitario", "preco", "valor unitario"],
        (Movements, Value) => &[
            "valor da operacao",
            "valor operacao",
            "valor",
            "valor total",
        ],

        (Position, Product) => &["produto", "ativo", "titulo"],
        (Position, Institution) => &["instituicao", "corretora", "instituicao financeira"],
        (Position, Account) => &["conta"],
        (Position, Code) => &["codigo de negociacao", "codigo negociacao", "codigo"],
        (Position, Quantity) => &["quantidade", "qtd", "qtde"],
        (Position, Close) => &[
            "preco de fechamento",
            "preco fechamento",
            "preco atualizado*",
        ],
        (Position, Updated) => &[
            "valor atualizado",
            "valor atualizado*",
            "valor liquido",
            "valor bruto",
        ],
        (Position, Indexer) => &["indexador"],
        (Position, Maturity) => &["vencimento", "data de vencimento"],
        (Position, Issuer) => &["emissor"],
        _ => &[],
    }
}

const FIELDS: &[(B3ReportKind, &[Field])] = &[
    (
        B3ReportKind::Trades,
        &[
            Field::TradeDate,
            Field::Side,
            Field::Market,
            Field::Institution,
            Field::Code,
            Field::Quantity,
            Field::Price,
            Field::Value,
        ],
    ),
    (
        B3ReportKind::Movements,
        &[
            Field::Direction,
            Field::Date,
            Field::Movement,
            Field::Product,
            Field::Institution,
            Field::Quantity,
            Field::UnitPrice,
            Field::Value,
        ],
    ),
    (
        B3ReportKind::Position,
        &[
            Field::Product,
            Field::Institution,
            Field::Account,
            Field::Code,
            Field::Quantity,
            Field::Close,
            Field::Updated,
            Field::Indexer,
            Field::Maturity,
            Field::Issuer,
        ],
    ),
];

#[derive(Debug, Clone)]
struct Header {
    kind: B3ReportKind,
    cols: HashMap<Field, usize>,
    names: Vec<String>,
    line: u32,
}

fn map_columns(kind: B3ReportKind, folded: &[String]) -> HashMap<Field, usize> {
    let fields = FIELDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, f)| *f)
        .unwrap_or(&[]);
    let mut used = vec![false; folded.len()];
    let mut out = HashMap::new();
    // Duas passadas: nomes exatos primeiro, prefixos depois (para "valor atualizado mtm"
    // não roubar a coluna de um apelido exato de outro campo).
    for exact in [true, false] {
        for &f in fields {
            if out.contains_key(&f) {
                continue;
            }
            for a in aliases(kind, f) {
                let (alias, prefix) = match a.strip_suffix('*') {
                    Some(p) => (p, true),
                    None => (*a, false),
                };
                if exact == prefix {
                    continue;
                }
                let hit = folded.iter().enumerate().position(|(i, h)| {
                    !used[i]
                        && if prefix {
                            h.starts_with(alias)
                        } else {
                            h == alias
                        }
                });
                if let Some(i) = hit {
                    used[i] = true;
                    out.insert(f, i);
                    break;
                }
            }
        }
    }
    out
}

/// A linha é cabeçalho de algum relatório? Exige as colunas que identificam cada um.
fn detect_header(row: &SheetRow) -> Option<Header> {
    let names: Vec<String> = row.cells.iter().map(Cell::text).collect();
    let folded: Vec<String> = names.iter().map(|n| fold(n)).collect();
    if folded.iter().filter(|f| !f.is_empty()).count() < 3 {
        return None;
    }
    let has = |s: &str| folded.iter().any(|f| f == s);
    let kind = if (has("entrada saida") || has("entrada ou saida") || has("credito debito"))
        && has("movimentacao")
        && has("produto")
    {
        B3ReportKind::Movements
    } else if (has("data do negocio") || has("data negocio") || has("data do pregao"))
        || ((has("tipo de movimentacao") || has("compra venda") || has("c v"))
            && !has("produto")
            && (has("codigo de negociacao") || has("codigo negociacao") || has("codigo")))
    {
        B3ReportKind::Trades
    } else if (has("produto") || has("titulo"))
        && has("quantidade")
        && !has("movimentacao")
        && (has("codigo de negociacao")
            || has("preco de fechamento")
            || folded.iter().any(|f| f.starts_with("valor atualizado"))
            || has("indexador")
            || has("vencimento")
            || has("valor aplicado")
            || has("conta"))
    {
        B3ReportKind::Position
    } else {
        return None;
    };
    let cols = map_columns(kind, &folded);
    let required: &[Field] = match kind {
        B3ReportKind::Trades => &[Field::TradeDate, Field::Side, Field::Code, Field::Quantity],
        B3ReportKind::Movements => &[
            Field::Direction,
            Field::Date,
            Field::Movement,
            Field::Product,
        ],
        B3ReportKind::Position => &[Field::Product, Field::Quantity],
    };
    if required.iter().any(|f| !cols.contains_key(f)) {
        return None;
    }
    Some(Header {
        kind,
        cols,
        names,
        line: row.line,
    })
}

// ---------------------------------------------------------------------------
// Registros

#[derive(Debug, Clone)]
pub struct TradeRec {
    pub src: B3Source,
    pub file: usize,
    pub date: NaiveDate,
    pub side: TradeSide,
    pub market: String,
    pub institution: String,
    pub code: String,
    pub qty: Decimal,
    pub price: Decimal,
}

#[derive(Debug, Clone)]
pub struct MoveRec {
    pub src: B3Source,
    pub file: usize,
    pub credit: bool,
    pub date: NaiveDate,
    pub movement: String,
    pub product: String,
    pub institution: String,
    pub qty: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub value: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct PosRec {
    pub src: B3Source,
    pub sheet_class: Option<AssetClass>,
    pub product: String,
    pub institution: String,
    pub code: Option<String>,
    pub qty: Decimal,
    pub close: Option<Decimal>,
    pub value: Option<Decimal>,
    pub indexer: Option<String>,
    pub maturity: Option<NaiveDate>,
    pub issuer: Option<String>,
}

/// Linha que parecia dado mas não pôde ser lida.
#[derive(Debug, Clone)]
pub struct BadRow {
    pub src: B3Source,
    pub report: B3ReportKind,
    pub description: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct Parsed {
    pub trades: Vec<TradeRec>,
    pub moves: Vec<MoveRec>,
    pub positions: Vec<PosRec>,
    pub bad: Vec<BadRow>,
    pub sheets: Vec<B3SheetInfo>,
    pub warnings: Vec<String>,
}

impl Parsed {
    pub fn kinds(&self) -> Vec<B3ReportKind> {
        let mut k: Vec<B3ReportKind> = self.sheets.iter().filter_map(|s| s.kind).collect();
        k.sort_by_key(|k| *k as u8);
        k.dedup();
        k
    }

    pub fn extend(&mut self, other: Parsed) {
        self.trades.extend(other.trades);
        self.moves.extend(other.moves);
        self.positions.extend(other.positions);
        self.bad.extend(other.bad);
        self.sheets.extend(other.sheets);
        self.warnings.extend(other.warnings);
    }
}

fn get<'a>(row: &'a SheetRow, h: &Header, f: Field) -> &'a Cell {
    static EMPTY: Cell = Cell::Empty;
    h.cols
        .get(&f)
        .and_then(|i| row.cells.get(*i))
        .unwrap_or(&EMPTY)
}

fn text_of(row: &SheetRow, h: &Header, f: Field) -> String {
    let c = get(row, h, f);
    if c.is_blank() {
        String::new()
    } else {
        c.text().trim().to_string()
    }
}

fn is_total_row(row: &SheetRow) -> bool {
    row.cells
        .iter()
        .find(|c| !c.is_blank())
        .is_some_and(|c| fold(&c.text()).starts_with("total"))
}

fn describe(row: &SheetRow) -> String {
    row.cells
        .iter()
        .filter(|c| !c.is_blank())
        .map(Cell::text)
        .take(6)
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Lê todas as abas do arquivo.
pub fn parse_workbook(file: usize, filename: &str, wb: &Workbook) -> Parsed {
    let mut out = Parsed::default();
    for t in &wb.tables {
        parse_table(file, filename, t, wb.decimal_comma, &mut out);
    }
    if out.sheets.iter().all(|s| s.kind.is_none()) {
        out.warnings.push(t_args(
            "nucleo-importacao-b3-aviso-sem-relatorio",
            &args(&[("arquivo", filename.to_string().into())]),
        ));
    }
    out
}

fn parse_table(file: usize, filename: &str, t: &Table, dc: Option<bool>, out: &mut Parsed) {
    let sheet_class = rules::position_sheet_class(&t.name);
    let skip_positions = rules::non_position_sheet(&t.name);
    let mut header: Option<Header> = None;
    let mut info = B3SheetInfo {
        name: t.name.clone(),
        kind: None,
        position_class: None,
        headers: vec![],
        header_line: None,
        row_count: 0,
        sample_rows: vec![],
    };
    for row in &t.rows {
        if let Some(h) = detect_header(row) {
            if info.kind.is_none() {
                info.kind = Some(h.kind);
                info.headers = h.names.iter().filter(|n| !n.is_empty()).cloned().collect();
                info.header_line = Some(h.line);
                if h.kind == B3ReportKind::Position {
                    info.position_class = sheet_class;
                }
            }
            header = Some(h);
            continue;
        }
        let Some(h) = &header else { continue };
        if is_total_row(row) || (h.kind == B3ReportKind::Position && skip_positions) {
            continue;
        }
        let src = B3Source {
            filename: filename.to_string(),
            sheet: t.name.clone(),
            line: row.line,
        };
        let res =
            match h.kind {
                B3ReportKind::Trades => read_trade(file, &src, row, h, dc)
                    .map(|r| r.map(|r| out.trades.push(r)).is_some()),
                B3ReportKind::Movements => read_move(file, &src, row, h, dc)
                    .map(|r| r.map(|r| out.moves.push(r)).is_some()),
                B3ReportKind::Position => read_position(&src, row, h, dc, sheet_class)
                    .map(|r| r.map(|r| out.positions.push(r)).is_some()),
            };
        match res {
            Ok(true) => {
                info.row_count += 1;
                if info.sample_rows.len() < 5 {
                    info.sample_rows
                        .push(row.cells.iter().map(Cell::text).collect());
                }
            }
            Ok(false) => {}
            Err(message) => {
                info.row_count += 1;
                out.bad.push(BadRow {
                    src,
                    report: h.kind,
                    description: describe(row),
                    message,
                });
            }
        }
    }
    if info.kind == Some(B3ReportKind::Position) && skip_positions {
        info.kind = None;
        out.warnings.push(t_args(
            "nucleo-importacao-b3-aviso-aba-ignorada",
            &args(&[("aba", t.name.clone().into())]),
        ));
    }
    out.sheets.push(info);
}

/// `what`: chave do rótulo do campo (`nucleo-importacao-b3-campo-*`), para a mensagem.
fn number(c: &Cell, dc: Option<bool>, what: &str) -> Result<Option<Decimal>, String> {
    c.number(dc).map_err(|e| {
        t_args(
            "nucleo-importacao-b3-erro-campo",
            &args(&[("campo", t(what).into()), ("erro", e.into())]),
        )
    })
}

fn read_trade(
    file: usize,
    src: &B3Source,
    row: &SheetRow,
    h: &Header,
    dc: Option<bool>,
) -> Result<Option<TradeRec>, String> {
    let code = text_of(row, h, Field::Code);
    let date_cell = get(row, h, Field::TradeDate);
    if code.is_empty() && date_cell.is_blank() {
        return Ok(None);
    }
    let date = date_cell
        .date()?
        .ok_or_else(|| t("nucleo-importacao-b3-linha-sem-data-negocio"))?;
    if code.is_empty() {
        return Err(t("nucleo-importacao-b3-linha-sem-codigo"));
    }
    let side_text = fold(&text_of(row, h, Field::Side));
    let side = if side_text.starts_with("compra") || side_text == "c" {
        TradeSide::Buy
    } else if side_text.starts_with("venda") || side_text == "v" {
        TradeSide::Sell
    } else {
        return Err(t_args(
            "nucleo-importacao-b3-linha-tipo-desconhecido",
            &args(&[("valor", text_of(row, h, Field::Side).into())]),
        ));
    };
    let qty = number(
        get(row, h, Field::Quantity),
        dc,
        "nucleo-importacao-b3-campo-quantidade",
    )?
    .ok_or_else(|| t("nucleo-importacao-b3-sem-quantidade"))?
    .abs();
    let price = number(
        get(row, h, Field::Price),
        dc,
        "nucleo-importacao-b3-campo-preco",
    )?;
    let value = number(
        get(row, h, Field::Value),
        dc,
        "nucleo-importacao-b3-campo-valor",
    )?;
    let price = match (price, value) {
        (Some(p), _) => p.abs(),
        (None, Some(v)) if !qty.is_zero() => (v.abs() / qty).round_dp(8),
        _ => return Err(t("nucleo-importacao-b3-linha-sem-preco")),
    };
    if qty.is_zero() {
        return Err(t("nucleo-importacao-b3-linha-quantidade-zero"));
    }
    Ok(Some(TradeRec {
        src: src.clone(),
        file,
        date,
        side,
        market: text_of(row, h, Field::Market),
        institution: text_of(row, h, Field::Institution),
        code,
        qty,
        price,
    }))
}

fn read_move(
    file: usize,
    src: &B3Source,
    row: &SheetRow,
    h: &Header,
    dc: Option<bool>,
) -> Result<Option<MoveRec>, String> {
    let movement = text_of(row, h, Field::Movement);
    let product = text_of(row, h, Field::Product);
    if movement.is_empty() && product.is_empty() {
        return Ok(None);
    }
    let dir = fold(&text_of(row, h, Field::Direction));
    let credit = if dir.starts_with("credito") || dir.starts_with("entrada") || dir == "c" {
        true
    } else if dir.starts_with("debito") || dir.starts_with("saida") || dir == "d" {
        false
    } else {
        return Err(t_args(
            "nucleo-importacao-b3-linha-direcao-desconhecida",
            &args(&[("valor", text_of(row, h, Field::Direction).into())]),
        ));
    };
    let date = get(row, h, Field::Date)
        .date()?
        .ok_or_else(|| t("nucleo-importacao-b3-linha-sem-data"))?;
    if product.is_empty() {
        return Err(t("nucleo-importacao-b3-linha-sem-produto"));
    }
    Ok(Some(MoveRec {
        src: src.clone(),
        file,
        credit,
        date,
        movement,
        product,
        institution: text_of(row, h, Field::Institution),
        qty: number(
            get(row, h, Field::Quantity),
            dc,
            "nucleo-importacao-b3-campo-quantidade",
        )?
        .map(|q| q.abs()),
        unit_price: number(
            get(row, h, Field::UnitPrice),
            dc,
            "nucleo-importacao-b3-campo-preco-unitario",
        )?
        .map(|q| q.abs()),
        value: number(
            get(row, h, Field::Value),
            dc,
            "nucleo-importacao-b3-campo-valor-operacao",
        )?
        .map(|q| q.abs()),
    }))
}

fn read_position(
    src: &B3Source,
    row: &SheetRow,
    h: &Header,
    dc: Option<bool>,
    sheet_class: Option<AssetClass>,
) -> Result<Option<PosRec>, String> {
    let mut product = text_of(row, h, Field::Product);
    let code = Some(text_of(row, h, Field::Code)).filter(|c| !c.is_empty());
    if product.is_empty() {
        match &code {
            Some(c) => product = c.clone(),
            None => return Ok(None),
        }
    }
    let Some(qty) = number(
        get(row, h, Field::Quantity),
        dc,
        "nucleo-importacao-b3-campo-quantidade",
    )?
    else {
        return Err(t("nucleo-importacao-b3-sem-quantidade"));
    };
    Ok(Some(PosRec {
        src: src.clone(),
        sheet_class,
        product,
        institution: text_of(row, h, Field::Institution),
        code,
        qty: qty.abs(),
        close: number(
            get(row, h, Field::Close),
            dc,
            "nucleo-importacao-b3-campo-preco-fechamento",
        )?,
        value: number(
            get(row, h, Field::Updated),
            dc,
            "nucleo-importacao-b3-campo-valor-atualizado",
        )?,
        indexer: Some(text_of(row, h, Field::Indexer)).filter(|s| !s.is_empty()),
        maturity: get(row, h, Field::Maturity).date().ok().flatten(),
        issuer: Some(text_of(row, h, Field::Issuer)).filter(|s| !s.is_empty()),
    }))
}
