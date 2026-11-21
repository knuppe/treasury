use base64::Engine as _;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::rules::{self, MarketRule, MoveRule};
use super::sheet::{self, Cell};
use super::*;
use crate::assets::{AssetClass, FixedIncomeKind, Indexer};
use crate::import::testutil::TestEnv;
use crate::portfolio::{CorporateActionKind, IncomeKind};

const POSICAO: &[u8] = include_bytes!("fixtures/posicao-2024-05-02.xlsx");
const NEGOCIACAO_XLSX: &[u8] = include_bytes!("fixtures/negociacao.xlsx");

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn f(name: &str, content: &str) -> B3File {
    B3File {
        filename: name.into(),
        content_base64: b64(content.as_bytes()),
    }
}

fn brokerage(env: &mut TestEnv, name: &str) -> String {
    let id = crate::ids::new_id();
    env.db
        .conn()
        .execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
             VALUES (?1, ?2, 'brokerage', 'BRL', '2023-01-01', 'x', 'x')",
            rusqlite::params![id, name],
        )
        .unwrap();
    id
}

fn preview_of(env: &mut TestEnv, files: Vec<B3File>) -> B3Preview {
    preview(
        &mut env.ctx(),
        &B3PreviewArgs {
            files,
            accounts: None,
            position_date: None,
        },
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Regras

#[test]
fn tickers_and_products() {
    assert_eq!(rules::normalize_ticker("PETR4F").as_deref(), Some("PETR4"));
    assert_eq!(rules::normalize_ticker(" petr4 ").as_deref(), Some("PETR4"));
    assert_eq!(rules::normalize_ticker("HGLG11").as_deref(), Some("HGLG11"));
    assert_eq!(rules::normalize_ticker("B3SA3F").as_deref(), Some("B3SA3"));
    assert_eq!(rules::normalize_ticker("AAPL34").as_deref(), Some("AAPL34"));
    assert_eq!(rules::normalize_ticker("PETRB400"), None);
    assert_eq!(rules::normalize_ticker("VALE3T"), None);
    assert_eq!(rules::normalize_ticker("Tesouro Selic 2029"), None);

    assert_eq!(
        rules::split_product("PETR4 - PETROLEO BRASILEIRO S.A. PETROBRAS"),
        (
            Some("PETR4".into()),
            "PETROLEO BRASILEIRO S.A. PETROBRAS".into()
        )
    );
    assert_eq!(
        rules::split_product("HGLG11 - CSHG LOGISTICA FDO INV IMOB - FII"),
        (
            Some("HGLG11".into()),
            "CSHG LOGISTICA FDO INV IMOB - FII".into()
        )
    );
    assert_eq!(
        rules::split_product("CDB - BANCO FICTICIO S.A."),
        (None, "CDB - BANCO FICTICIO S.A.".into())
    );
    assert_eq!(
        rules::split_product("Tesouro IPCA+ 2035"),
        (None, "Tesouro IPCA+ 2035".into())
    );
    assert_eq!(
        rules::split_product("ITSA4"),
        (Some("ITSA4".into()), "ITSA4".into())
    );
}

#[test]
fn class_inference() {
    let c = |t: Option<&str>, name: &str| rules::infer_class(t, name, None, true).class;
    assert_eq!(c(Some("AAPL34"), "APPLE INC"), AssetClass::Bdr);
    assert_eq!(c(Some("ROXO34"), "NU HOLDINGS"), AssetClass::Bdr);
    assert_eq!(c(Some("XPTO39"), "BDR ETF FICTICIO"), AssetClass::Bdr);
    assert_eq!(
        c(Some("HGLG11"), "CSHG LOGISTICA FDO INV IMOB - FII"),
        AssetClass::Fii
    );
    assert_eq!(
        c(Some("ABCD11"), "FUNDO DE INVESTIMENTO IMOBILIARIO FICTICIO"),
        AssetClass::Fii
    );
    assert_eq!(
        c(Some("BOVA11"), "ISHARES BOVA FUNDO DE INDICE"),
        AssetClass::EtfBr
    );
    assert_eq!(c(Some("XYZW11"), "XYZ ETF FICTICIO"), AssetClass::EtfBr);
    assert_eq!(c(Some("PETR4"), "PETROBRAS"), AssetClass::StockBr);
    assert_eq!(c(None, "Tesouro IPCA+ 2035"), AssetClass::TreasuryBr);
    assert_eq!(c(None, "CDB - BANCO FICTICIO"), AssetClass::FixedIncome);
    assert_eq!(c(None, "LCA BANCO FICTICIO"), AssetClass::FixedIncome);
    // 11 sem nome: presunção fraca.
    let g = rules::infer_class(Some("KLBN11"), "KLBN11", None, false);
    assert_eq!(g.class, AssetClass::StockBr);
    assert!(g.weak);
    // A aba da posição manda.
    let g = rules::infer_class(Some("KLBN11"), "KLBN11", Some(AssetClass::StockBr), false);
    assert!(!g.weak);
    assert_eq!(
        rules::infer_class(Some("IVVB11"), "x", Some(AssetClass::EtfBr), true).class,
        AssetClass::EtfBr
    );

    assert_eq!(
        rules::fixed_income_kind("LCI - BANCO X"),
        Some(FixedIncomeKind::Lci)
    );
    assert_eq!(
        rules::fixed_income_kind("DEB - EMPRESA"),
        Some(FixedIncomeKind::Debenture)
    );
    assert_eq!(rules::fixed_income_kind("PETR4"), None);
    assert_eq!(rules::indexer("DI"), Some(Indexer::Cdi));
    assert_eq!(rules::indexer("CDI"), Some(Indexer::Cdi));
    assert_eq!(rules::indexer("IPCA"), Some(Indexer::Ipca));
    assert_eq!(rules::indexer("PRÉ"), Some(Indexer::Prefixado));
    assert_eq!(rules::indexer("SELIC"), Some(Indexer::Selic));

    assert_eq!(
        rules::position_sheet_class("Acoes"),
        Some(AssetClass::StockBr)
    );
    assert_eq!(
        rules::position_sheet_class("Ações"),
        Some(AssetClass::StockBr)
    );
    assert_eq!(
        rules::position_sheet_class("Fundo de Investimento"),
        Some(AssetClass::Fii)
    );
    assert_eq!(
        rules::position_sheet_class("Renda Fixa"),
        Some(AssetClass::FixedIncome)
    );
    assert_eq!(
        rules::position_sheet_class("Tesouro Direto"),
        Some(AssetClass::TreasuryBr)
    );
    assert_eq!(rules::position_sheet_class("Empréstimo de Ativos"), None);
}

#[test]
fn movement_and_market_rules() {
    use B3IgnoreRule as R;
    let m = rules::move_rule;
    assert_eq!(m("Transferência - Liquidação"), MoveRule::Settlement);
    assert_eq!(m("TRANSFERENCIA - LIQUIDACAO"), MoveRule::Settlement);
    assert_eq!(m("Compra/Venda"), MoveRule::Settlement);
    assert_eq!(m("Transferência"), MoveRule::Ignore(R::CustodyTransfer));
    assert_eq!(m("Dividendo"), MoveRule::Income(IncomeKind::Dividend));
    assert_eq!(
        m("Juros Sobre Capital Próprio"),
        MoveRule::Income(IncomeKind::Jcp)
    );
    assert_eq!(
        m("JUROS SOBRE CAPITAL PROPRIO"),
        MoveRule::Income(IncomeKind::Jcp)
    );
    assert_eq!(m("Rendimento"), MoveRule::Income(IncomeKind::FiiIncome));
    assert_eq!(m("Amortização"), MoveRule::Income(IncomeKind::Amortization));
    assert_eq!(
        m("Dividendo - Transferido"),
        MoveRule::ForwardedIncome(IncomeKind::Dividend)
    );
    assert_eq!(m("Desdobro"), MoveRule::Action(CorporateActionKind::Split));
    assert_eq!(
        m("Grupamento"),
        MoveRule::Action(CorporateActionKind::ReverseSplit)
    );
    assert_eq!(
        m("Bonificação em Ativos"),
        MoveRule::Action(CorporateActionKind::Bonus)
    );
    assert_eq!(m("Fração em Ativos"), MoveRule::Fraction);
    assert_eq!(m("Leilão de Fração"), MoveRule::FractionAuction);
    assert_eq!(
        m("Direito de Subscrição"),
        MoveRule::Ignore(R::SubscriptionRight)
    );
    assert_eq!(
        m("Direitos de Subscrição - Não Exercido"),
        MoveRule::Ignore(R::SubscriptionRight)
    );
    assert_eq!(
        m("Cessão de Direitos"),
        MoveRule::Ignore(R::SubscriptionRight)
    );
    assert_eq!(m("Atualização"), MoveRule::Ignore(R::BalanceUpdate));
    assert_eq!(m("Resgate"), MoveRule::Redemption);
    assert_eq!(m("Vencimento"), MoveRule::Redemption);
    assert_eq!(m("Incorporação"), MoveRule::Ignore(R::ManualEvent));
    assert_eq!(m("Empréstimo"), MoveRule::Ignore(R::Lending));
    assert_eq!(m("Empréstimo - Registro"), MoveRule::Ignore(R::Lending));
    assert_eq!(m("Algo Novo da B3"), MoveRule::Ignore(R::UnknownMovement));

    assert_eq!(rules::market_rule("Mercado à Vista"), MarketRule::Spot);
    assert_eq!(rules::market_rule("MERCADO FRACIONARIO"), MarketRule::Spot);
    assert_eq!(rules::market_rule(""), MarketRule::Spot);
    assert_eq!(
        rules::market_rule("Opção de Compra"),
        MarketRule::Ignore(R::Option)
    );
    assert_eq!(
        rules::market_rule("Exercício de Opção de Venda"),
        MarketRule::Ignore(R::Option)
    );
    assert_eq!(rules::market_rule("Termo"), MarketRule::Ignore(R::Forward));
    assert_eq!(rules::market_rule("Futuro"), MarketRule::Ignore(R::Future));
    assert_eq!(
        rules::market_rule("Balcão"),
        MarketRule::Ignore(R::UnsupportedMarket)
    );
}

#[test]
fn institution_matching() {
    assert_eq!(
        rules::institution_tokens("XP INVESTIMENTOS CCTVM S/A"),
        vec!["xp"]
    );
    assert_eq!(
        rules::institution_tokens("NU INVEST CORRETORA DE VALORES S.A."),
        vec!["nu"]
    );
    assert_eq!(
        rules::institution_tokens("RICO INVESTIMENTOS - GRUPO XP"),
        vec!["rico"]
    );
    assert_eq!(
        rules::institution_tokens("BTG PACTUAL CTVM S.A."),
        vec!["btg", "pactual"]
    );
    let t = rules::institution_tokens("XP INVESTIMENTOS CCTVM S/A");
    assert!(rules::institution_score(&t, "XP Investimentos ") > 0);
    assert_eq!(rules::institution_score(&t, "Rico "), 0);
    // "xp" não casa dentro de outra palavra.
    assert_eq!(rules::institution_score(&t, "Expresso "), 0);
}

// ---------------------------------------------------------------------------
// Leitura

#[test]
fn numbers_and_dates() {
    let n = |s: &str, dc| sheet::parse_number(s, dc);
    assert_eq!(n("1.234,56", None), Some(dec!(1234.56)));
    assert_eq!(n("1234,56", None), Some(dec!(1234.56)));
    assert_eq!(n("R$ 1.925,00", None), Some(dec!(1925)));
    assert_eq!(n("30.45", None), Some(dec!(30.45)));
    assert_eq!(n("1,234.56", None), Some(dec!(1234.56)));
    assert_eq!(n("1.234", None), Some(dec!(1234)));
    assert_eq!(n("1.234", Some(false)), Some(dec!(1.234)));
    assert_eq!(n("1,234", Some(false)), Some(dec!(1234)));
    assert_eq!(n("0,7", Some(true)), Some(dec!(0.7)));
    assert_eq!(n("-5", None), Some(dec!(-5)));
    assert_eq!(n("(5,00)", None), Some(dec!(-5)));
    assert_eq!(n("abc", None), None);
    assert!(Cell::Text("-".into()).is_blank());
    assert_eq!(Cell::Text("-".into()).number(None), Ok(None));
    assert!(Cell::Text("abc".into()).number(None).is_err());

    let d = |s: &str| sheet::parse_date_text(s).map(|d| d.to_string());
    assert_eq!(d("10/01/2024").as_deref(), Some("2024-01-10"));
    assert_eq!(d("2024-01-10").as_deref(), Some("2024-01-10"));
    assert_eq!(d("10/01/2024 00:00:00").as_deref(), Some("2024-01-10"));
    assert_eq!(d("10-01-2024").as_deref(), Some("2024-01-10"));
    assert_eq!(d("32/01/2024"), None);
    // Número de série do Excel sem formato de data.
    assert_eq!(
        Cell::Num(Decimal::from(45301))
            .date()
            .unwrap()
            .map(|d| d.to_string())
            .as_deref(),
        Some("2024-01-10")
    );
}

#[test]
fn header_variants_title_rows_and_comma_delimiter() {
    // Sem acentos, em maiúsculas, colunas em outra ordem, título antes, vírgula como
    // delimitador e ponto decimal.
    let csv = "RELATORIO DE NEGOCIACAO\r\n\r\n\
               CODIGO DE NEGOCIACAO,INSTITUICAO,DATA DO NEGOCIO,TIPO DE MOVIMENTACAO,QUANTIDADE,PRECO,VALOR,MERCADO\r\n\
               PETR4,XP INVESTIMENTOS CCTVM S/A,10/01/2024,COMPRA,100,35.50,3550.00,MERCADO A VISTA\r\n\
               PETR4F,XP INVESTIMENTOS CCTVM S/A,11/01/2024,C,7,36.00,252.00,MERCADO FRACIONARIO\r\n";
    let r = detect(&f("neg.csv", csv)).unwrap();
    assert_eq!(r.kinds, vec![B3ReportKind::Trades]);
    assert_eq!(r.sheets[0].row_count, 2);

    let mut env = TestEnv::new();
    let xp = brokerage(&mut env, "XP");
    let p = preview_of(&mut env, vec![f("neg.csv", csv)]);
    assert_eq!(p.institutions[0].account_id.as_deref(), Some(xp.as_str()));
    assert_eq!(p.trades.len(), 2);
    assert_eq!(p.trades[0].price, "35.5");
    assert!(
        p.trades
            .iter()
            .all(|t| t.ticker.as_deref() == Some("PETR4"))
    );
    assert!(p.trades.iter().all(|t| t.status == B3RowStatus::New));

    // Movimentação com ";" e cabeçalho com outra grafia ("Preço" no lugar de "Preço unitário").
    let mov = "Entrada/Saida;Data;Movimentacao;Produto;Instituicao;Quantidade;Preco;Valor\n\
               Credito;15/02/2024;JUROS SOBRE CAPITAL PROPRIO;PETR4 - PETROBRAS;XP INVESTIMENTOS CCTVM S/A;100;0,1;8,50\n\
               Credito;16/02/2024;Dividendo;PETR4 - PETROBRAS;XP INVESTIMENTOS CCTVM S/A;100;-;-\n\
               Debito;17/02/2024;Dividendo;PETR4 - PETROBRAS;XP INVESTIMENTOS CCTVM S/A;100;0,1;10,00\n";
    let p = preview_of(&mut env, vec![f("mov.csv", mov)]);
    assert_eq!(p.incomes.len(), 1);
    // 8,50 ÷ 0,85 = 10,00 bruto, 1,50 retido.
    assert_eq!(
        (
            p.incomes[0].gross,
            p.incomes[0].tax_withheld,
            p.incomes[0].net
        ),
        (1000, 150, 850)
    );
    let rules: Vec<B3IgnoreRule> = p.ignored.iter().map(|i| i.rule).collect();
    assert_eq!(
        rules,
        vec![B3IgnoreRule::NoValue, B3IgnoreRule::IncomeReversal]
    );
}

#[test]
fn bad_rows_and_unknown_files() {
    let csv = "Data do Negócio;Tipo de Movimentação;Mercado;Instituição;Código de Negociação;Quantidade;Preço\n\
               31/02/2024;Compra;Mercado à Vista;XP;PETR4;100;35,00\n\
               10/01/2024;Doação;Mercado à Vista;XP;PETR4;100;35,00\n\
               10/01/2024;Compra;Mercado à Vista;XP;PETR4;abc;35,00\n";
    let mut env = TestEnv::new();
    brokerage(&mut env, "XP");
    let p = preview_of(&mut env, vec![f("neg.csv", csv)]);
    assert!(p.trades.is_empty());
    assert_eq!(p.ignored.len(), 3);
    assert!(p.ignored.iter().all(|i| i.rule == B3IgnoreRule::Invalid));
    assert!(p.ignored[0].reason.contains("data inválida"));
    assert!(p.ignored[1].reason.contains("Doação"));
    assert!(p.ignored[2].reason.contains("quantidade"));

    let err = preview(
        &mut env.ctx(),
        &B3PreviewArgs {
            files: vec![f("x.csv", "a;b;c\n1;2;3\n")],
            accounts: None,
            position_date: None,
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("nenhuma linha"));
    assert!(detect(&f("x.pdf", "%PDF-1.4")).is_err());
}

#[test]
fn xlsx_cells_are_typed() {
    let wb = sheet::read(NEGOCIACAO_XLSX, "negociacao.xlsx").unwrap();
    assert_eq!(wb.format, B3FileFormat::Xlsx);
    let t = &wb.tables[0];
    assert_eq!(t.name, "Negociação");
    // Linha 4: primeira linha de dados, com a data tipada (formato de data).
    let row = t.rows.iter().find(|r| r.line == 4).unwrap();
    assert_eq!(
        row.cells[0],
        Cell::Date(chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap())
    );
    assert_eq!(row.cells[7], Cell::Num(dec!(38.5)));
    // Linha 5: data como texto.
    let row = t.rows.iter().find(|r| r.line == 5).unwrap();
    assert_eq!(row.cells[0], Cell::Text("15/03/2024".into()));

    let wb = sheet::read(POSICAO, "posicao.xlsx").unwrap();
    let names: Vec<&str> = wb.tables.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Acoes",
            "BDR",
            "ETF",
            "Fundo de Investimento",
            "Tesouro Direto",
            "Empréstimo de Ativos"
        ]
    );
}

#[test]
fn position_only_compares_with_existing_portfolio() {
    let mut env = TestEnv::new();
    let xp = brokerage(&mut env, "XP Investimentos");
    // Nada importado ainda: tudo diverge, e nada é proposto para gravar.
    let p = preview(
        &mut env.ctx(),
        &B3PreviewArgs {
            files: vec![B3File {
                filename: "posicao.xlsx".into(),
                content_base64: b64(POSICAO),
            }],
            accounts: None,
            position_date: Some("2024-05-02".into()),
        },
    )
    .unwrap();
    assert!(p.trades.is_empty() && p.incomes.is_empty() && p.actions.is_empty());
    assert!(p.assets.is_empty());
    let check = p.position_check.unwrap();
    assert_eq!(check.date, "2024-05-02");
    assert_eq!(check.matched, 0);
    let petr = check.lines.iter().find(|l| l.asset_key == "PETR4").unwrap();
    assert_eq!(petr.status, B3PositionStatus::Divergent);
    assert_eq!(petr.account_id.as_deref(), Some(xp.as_str()));
    assert_eq!(petr.b3_quantity.as_deref(), Some("62"));
    assert_eq!(petr.close_price.as_deref(), Some("40"));
    assert_eq!(petr.b3_value, Some(248000));
    // Linhas de total não viram posição; a aba de empréstimo não soma.
    assert!(
        check
            .lines
            .iter()
            .all(|l| !l.name.to_lowercase().contains("total"))
    );
    assert_eq!(
        check
            .lines
            .iter()
            .filter(|l| l.asset_key == "PETR4")
            .count(),
        1
    );
}

#[test]
fn sale_after_ignored_custody_transfer_explains_the_error() {
    let mut env = TestEnv::new();
    brokerage(&mut env, "XP Investimentos");
    brokerage(&mut env, "Nu Invest");
    let neg = "Data do Negócio;Tipo de Movimentação;Mercado;Instituição;Código de Negociação;Quantidade;Preço\n\
               10/01/2024;Compra;Mercado à Vista;XP INVESTIMENTOS CCTVM S/A;AAPL34;10;45,00\n\
               20/03/2024;Venda;Mercado à Vista;NU INVEST CORRETORA DE VALORES S.A.;AAPL34;10;50,00\n";
    let mov = "Entrada/Saída;Data;Movimentação;Produto;Instituição;Quantidade;Preço unitário;Valor da Operação\n\
               Debito;05/03/2024;Transferência;AAPL34 - APPLE INC;XP INVESTIMENTOS CCTVM S/A;10;-;-\n\
               Credito;05/03/2024;Transferência;AAPL34 - APPLE INC;NU INVEST CORRETORA DE VALORES S.A.;10;-;-\n";
    let p = preview_of(&mut env, vec![f("neg.csv", neg), f("mov.csv", mov)]);
    let sale = p
        .trades
        .iter()
        .find(|t| t.side == crate::portfolio::TradeSide::Sell)
        .unwrap();
    assert_eq!(sale.status, B3RowStatus::Error);
    let e = sale.error.as_deref().unwrap();
    assert!(e.contains("maior que a posição"), "{e}");
    assert!(e.contains("transferência de custódia"), "{e}");
    assert!(p.warnings.iter().any(|w| w.contains("venda(s) maiores")));
    assert_eq!(p.counts.errors, 1);

    // Apontando as duas instituições para a mesma conta, a venda passa.
    let xp_id = p.institutions[0].account_id.clone();
    let p = preview(
        &mut env.ctx(),
        &B3PreviewArgs {
            files: vec![f("neg.csv", neg), f("mov.csv", mov)],
            accounts: Some(vec![B3AccountChoice {
                institution: "NU INVEST CORRETORA DE VALORES S.A.".into(),
                account_id: xp_id,
            }]),
            position_date: None,
        },
    )
    .unwrap();
    assert!(p.trades.iter().all(|t| t.status == B3RowStatus::New));
}

#[test]
fn reverse_split_and_fixed_income() {
    let mut env = TestEnv::new();
    brokerage(&mut env, "XP Investimentos");
    let neg = "Data do Negócio;Tipo de Movimentação;Mercado;Instituição;Código de Negociação;Quantidade;Preço\n\
               10/01/2024;Compra;Mercado à Vista;XP INVESTIMENTOS CCTVM S/A;ABCD3;1000;1,00\n";
    // Grupamento de 10 para 1: a B3 baixa 900 ações.
    let mov = "Entrada/Saída;Data;Movimentação;Produto;Instituição;Quantidade;Preço unitário;Valor da Operação\n\
               Debito;01/03/2024;Grupamento;ABCD3 - EMPRESA FICTICIA S.A.;XP INVESTIMENTOS CCTVM S/A;900;-;-\n\
               Credito;10/01/2024;Compra/Venda;CDB - BANCO FICTICIO S.A.;XP INVESTIMENTOS CCTVM S/A;1;1.000,00;1.000,00\n\
               Credito;10/02/2024;Juros;CDB - BANCO FICTICIO S.A.;XP INVESTIMENTOS CCTVM S/A;1;-;12,34\n\
               Debito;10/04/2024;Vencimento;CDB - BANCO FICTICIO S.A.;XP INVESTIMENTOS CCTVM S/A;1;1.050,00;1.050,00\n";
    let p = preview_of(&mut env, vec![f("neg.csv", neg), f("mov.csv", mov)]);
    let a = &p.actions[0];
    assert_eq!(a.kind, CorporateActionKind::ReverseSplit);
    assert_eq!(a.factor.as_deref(), Some("0.1"));
    assert_eq!(a.status, B3RowStatus::New);
    let cdb = p.assets.iter().find(|a| a.ticker.is_none()).unwrap();
    assert_eq!(cdb.class, AssetClass::FixedIncome);
    assert!(cdb.needs_review);
    // Compra e vencimento do CDB viram operações; juros, provento.
    let cdb_trades: Vec<_> = p.trades.iter().filter(|t| t.ticker.is_none()).collect();
    assert_eq!(cdb_trades.len(), 2);
    assert!(cdb_trades.iter().all(|t| t.status == B3RowStatus::New));
    assert_eq!(p.incomes[0].kind, IncomeKind::Interest);

    let r = commit(
        &mut env.ctx(),
        &B3CommitArgs {
            token: p.token.clone(),
            skip: None,
            include_possible_duplicates: None,
            approve_actions: None,
            assets: None,
            cash_movement: None,
        },
    )
    .unwrap();
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    // Sem a taxa, a renda fixa entra como "Outros".
    let class: String = env
        .db
        .conn()
        .query_row("SELECT class FROM asset WHERE ticker IS NULL", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(class, "other");
    let held = crate::portfolio::engine::quantity_held(
        env.db.conn(),
        &env.db
            .conn()
            .query_row("SELECT id FROM asset WHERE ticker = 'ABCD3'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        None,
        "2024-03-02",
    )
    .unwrap();
    assert_eq!(held, dec!(100));
}

#[test]
fn fixed_income_override_creates_terms() {
    let mut env = TestEnv::new();
    brokerage(&mut env, "XP Investimentos");
    let mov = "Entrada/Saída;Data;Movimentação;Produto;Instituição;Quantidade;Preço unitário;Valor da Operação\n\
               Credito;10/01/2024;Compra/Venda;LCA - BANCO FICTICIO S.A.;XP INVESTIMENTOS CCTVM S/A;1;1.000,00;1.000,00\n";
    let p = preview_of(&mut env, vec![f("mov.csv", mov)]);
    let key = p.assets[0].key.clone();
    let r = commit(
        &mut env.ctx(),
        &B3CommitArgs {
            token: p.token.clone(),
            skip: None,
            include_possible_duplicates: None,
            approve_actions: None,
            assets: Some(vec![B3AssetOverride {
                key,
                asset_id: None,
                class: None,
                name: Some("LCA Banco Fictício 2026".into()),
                fixed_income: Some(crate::assets::FixedIncomeTerms {
                    kind: FixedIncomeKind::Lca,
                    indexer: Indexer::Cdi,
                    rate: "95".into(),
                    issue_date: Some("2024-01-10".into()),
                    maturity_date: Some("2026-01-10".into()),
                    grace_until: None,
                    liquidity: crate::assets::Liquidity::AtMaturity,
                    ir_exempt: true,
                    issuer: None,
                }),
            }]),
            cash_movement: None,
        },
    )
    .unwrap();
    assert_eq!(r.assets_created, 1);
    let (class, name): (String, String) = env
        .db
        .conn()
        .query_row("SELECT class, name FROM asset", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(
        (class.as_str(), name.as_str()),
        ("fixed_income", "LCA Banco Fictício 2026")
    );
}

/// JCP líquido da B3: bruto pela alíquota da data do crédito (15% até 2025, 17,5% desde
/// 2026; LC 224/2025 art. 8º e art. 14, III; Receita, Nota Cetad/Coest nº 009/2026).
#[test]
fn b3_jcp_gross_uses_the_rate_of_the_credit_date() {
    let d = |s: &str| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
    assert_eq!(
        super::plan::gross_and_tax(IncomeKind::Jcp, 825, d("2026-03-02")),
        (1_000, 175)
    );
    assert_eq!(
        super::plan::gross_and_tax(IncomeKind::Jcp, 850, d("2025-12-30")),
        (1_000, 150)
    );
    assert_eq!(
        super::plan::gross_and_tax(IncomeKind::Dividend, 850, d("2026-03-02")),
        (850, 0)
    );
}
