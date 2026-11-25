//! Regras de interpretação dos relatórios da B3: códigos de negociação, classes de ativo,
//! tipos de movimentação e mercados, e as linhas que **não** viram lançamento (com o
//! motivo de cada uma).
//!
//! Tudo que é comparado passa por [`fold`] (minúsculas, sem acentos, pontuação vira
//! espaço), então "Juros Sobre Capital Próprio", "JUROS SOBRE CAPITAL PROPRIO" e
//! "juros sobre capital próprio" são a mesma coisa.

use crate::assets::{AssetClass, FixedIncomeKind, Indexer};
use crate::i18n::t;
use crate::import::text::fold;
use crate::portfolio::{CorporateActionKind, IncomeKind};

use super::B3IgnoreRule;

// ---------------------------------------------------------------------------
// Códigos de negociação

/// Código de negociação da B3: 4 caracteres de raiz (letras, às vezes dígitos, como
/// "B3SA") e 1–2 dígitos de tipo (3 ON, 4 PN, 11 unit/FII/ETF, 34 BDR…).
fn is_ticker(s: &str) -> bool {
    let b = s.as_bytes();
    if !(5..=6).contains(&b.len()) || !b[0].is_ascii_uppercase() {
        return false;
    }
    let (root, kind) = b.split_at(4);
    root.iter()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && kind.iter().all(u8::is_ascii_digit)
}

/// Normaliza o código: maiúsculas, sem espaços e sem o sufixo `F` do mercado fracionário
/// (PETR4F → PETR4, o mesmo ativo). `None` se não parece um código de ação/FII/ETF/BDR
/// (opções, termo e produtos de renda fixa têm outros formatos).
pub fn normalize_ticker(code: &str) -> Option<String> {
    let c: String = code
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();
    if is_ticker(&c) {
        return Some(c);
    }
    if let Some(base) = c.strip_suffix('F')
        && is_ticker(base)
    {
        return Some(base.to_string());
    }
    None
}

/// Separa "PETR4 - PETROLEO BRASILEIRO S.A. PETROBRAS" em código e nome. Produtos sem
/// código (Tesouro, CDB, debêntures) ficam com o nome completo e sem ticker.
pub fn split_product(product: &str) -> (Option<String>, String) {
    let p = product.trim();
    if let Some((left, right)) = p.split_once(" - ")
        && let Some(t) = normalize_ticker(left)
    {
        let name = right.trim();
        return (
            Some(t.clone()),
            if name.is_empty() { t } else { name.to_string() },
        );
    }
    if let Some(t) = normalize_ticker(p) {
        return (Some(t.clone()), t);
    }
    (None, p.to_string())
}

/// Chave do ativo na prévia: o ticker, ou o nome do produto sem diferenciar maiúsculas e
/// acentos (renda fixa e Tesouro).
pub fn asset_key(ticker: Option<&str>, name: &str) -> String {
    match ticker {
        Some(t) => t.to_string(),
        None => match treasury_key(name) {
            Some(k) => k,
            None => format!("produto:{}", fold(name)),
        },
    }
}

/// What a report row says about its asset.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductId {
    pub ticker: Option<String>,
    /// Name from the file (for a contract, the "<TYPE> <ISSUER> <CODE>" built from it).
    pub name: String,
    /// The file has a name besides the code.
    pub has_name: bool,
    pub key: String,
    /// Fixed income contract code.
    pub contract: Option<String>,
    pub issuer: Option<String>,
}

/// Ticker, name and key of a row. `code` is the position report's code column
/// (`Código de Negociação` for listed assets, `Código` of the contract for fixed income).
pub fn identify(product: &str, code: Option<&str>) -> ProductId {
    let (pt, name) = split_product(product);
    let ticker = code.and_then(normalize_ticker).or(pt);
    if ticker.is_none()
        && let Some(c) = contract_code(product, code)
    {
        let issuer = contract_issuer(product, &c);
        return ProductId {
            ticker: None,
            name: contract_name(product, &c, issuer.as_deref()),
            has_name: true,
            key: contract_key(&c),
            contract: Some(c),
            issuer,
        };
    }
    let has_name = ticker.as_deref() != Some(name.as_str());
    ProductId {
        key: asset_key(ticker.as_deref(), &name),
        ticker,
        name,
        has_name,
        contract: None,
        issuer: None,
    }
}

static CONTRACT: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\b(CDB|RDB|LCI|LCA|LC|CRI|CRA|DEB)[A-Z0-9]{6,}\b").expect("regex")
});

/// Contract code of a fixed income product, unique per investment ("RDB31497YWB" in
/// "RDB - RDB31497YWB - COOPERATIVA …"). The position report keeps it in the `Código`
/// column (`code`) and puts only the type and issuer in `Produto`.
pub fn contract_code(product: &str, code: Option<&str>) -> Option<String> {
    let up = product.to_uppercase();
    if let Some(m) = CONTRACT.find(&up) {
        return Some(m.as_str().to_string());
    }
    let c: String = code?
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase();
    (c.len() >= 6
        && c.chars().all(|ch| ch.is_ascii_alphanumeric())
        && normalize_ticker(&c).is_none()
        && fixed_income_kind(product).is_some())
    .then_some(c)
}

/// Key of a fixed income contract.
pub fn contract_key(code: &str) -> String {
    format!("rf:{code}")
}

/// Issuer written after the contract code in the movements report
/// ("CDB - CDBU24FGBLH - NU FINANCEIRA S.A." → "NU FINANCEIRA S.A.") or after the type
/// in the position report ("RDB - COOPERATIVA …" → "COOPERATIVA …").
pub fn contract_issuer(product: &str, code: &str) -> Option<String> {
    let parts: Vec<&str> = product.split(" - ").map(str::trim).collect();
    let kind_or_code = |p: &str| {
        p.eq_ignore_ascii_case(code) || FIXED_INCOME_PREFIXES.iter().any(|(k, _)| fold(p) == *k)
    };
    let rest: Vec<&str> = parts.into_iter().filter(|p| !kind_or_code(p)).collect();
    (!rest.is_empty()).then(|| rest.join(" - "))
}

/// Display name of a contract: "<TYPE> <ISSUER> <CODE>".
pub fn contract_name(product: &str, code: &str, issuer: Option<&str>) -> String {
    let kind = fold(product).split(' ').next().unwrap_or("").to_uppercase();
    let kind = if kind.is_empty() || kind.eq_ignore_ascii_case(code) {
        code.chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect::<String>()
    } else {
        kind
    };
    match issuer.map(str::trim).filter(|s| !s.is_empty()) {
        Some(i) => format!("{kind} {i} {code}"),
        None => format!("{kind} {code}"),
    }
}

/// Stable key of a Tesouro Direto bond, from its type and maturity year. The name changes
/// between the movements and position reports and between brokers ("Tesouro IPCA+ com Juros
/// Semestrais 2035", "TESOURO IPCA+ C/ JUROS SEMESTRAIS 2035", "NTN-B 2035"…).
pub fn treasury_key(name: &str) -> Option<String> {
    if !is_treasury_name(name) {
        return None;
    }
    let raw = crate::import::text::fold_keep_punct(name);
    let f = format!(" {} ", fold(name));
    let year = f
        .split(' ')
        .rev()
        .find(|w| w.len() == 4 && w.starts_with("20") && w.chars().all(|c| c.is_ascii_digit()))?;
    let semi = f.contains("juros semestrais") || f.contains(" c juros") || f.contains("cupom");
    let kind = if raw.contains("renda+") || f.contains(" renda mais ") {
        "renda+"
    } else if f.contains("educa") {
        "educa+"
    } else if f.contains("ipca") {
        if semi { "ipca-js" } else { "ipca" }
    } else if f.contains("selic") {
        "selic"
    } else if f.contains("igp") {
        if semi { "igpm-js" } else { "igpm" }
    } else if f.contains("prefixado") || f.contains(" pre ") {
        if semi { "pre-js" } else { "pre" }
    } else {
        return None;
    };
    Some(format!("tesouro:{kind}:{year}"))
}

// ---------------------------------------------------------------------------
// Classes

pub fn is_treasury_name(name: &str) -> bool {
    fold(name).starts_with("tesouro ")
}

const FIXED_INCOME_PREFIXES: &[(&str, FixedIncomeKind)] = &[
    ("cdb", FixedIncomeKind::Cdb),
    ("lci", FixedIncomeKind::Lci),
    ("lca", FixedIncomeKind::Lca),
    ("lc", FixedIncomeKind::Lc),
    ("cri", FixedIncomeKind::Cri),
    ("cra", FixedIncomeKind::Cra),
    ("deb", FixedIncomeKind::Debenture),
    ("debenture", FixedIncomeKind::Debenture),
    ("debentures", FixedIncomeKind::Debenture),
    ("lf", FixedIncomeKind::Other),
    ("lfs", FixedIncomeKind::Other),
    ("lig", FixedIncomeKind::Other),
    ("rdb", FixedIncomeKind::Rdb),
    ("cce", FixedIncomeKind::Other),
    ("ccb", FixedIncomeKind::Other),
    ("nc", FixedIncomeKind::Other),
];

/// Tipo de renda fixa pelo começo do nome ("CDB - BANCO X", "LCA BANCO Y", "DEB …").
pub fn fixed_income_kind(name: &str) -> Option<FixedIncomeKind> {
    let f = fold(name);
    let first = f.split(' ').next().unwrap_or("");
    FIXED_INCOME_PREFIXES
        .iter()
        .find(|(p, _)| *p == first)
        .map(|(_, k)| *k)
}

/// Indexador pela coluna "Indexador" da posição de renda fixa/Tesouro.
pub fn indexer(s: &str) -> Option<Indexer> {
    let f = fold(s);
    let has = |w: &str| f.split(' ').any(|x| x == w) || f.contains(w);
    if has("ipca") {
        Some(Indexer::Ipca)
    } else if has("selic") {
        Some(Indexer::Selic)
    } else if has("igp") || has("igpm") {
        Some(Indexer::Igpm)
    } else if f.split(' ').any(|x| x == "di" || x == "cdi") || has("cdi") {
        Some(Indexer::Cdi)
    } else if has("pre") || has("prefixado") {
        Some(Indexer::Prefixado)
    } else {
        None
    }
}

/// Abas do relatório de posição que não são posição em ativo: empréstimos (as ações
/// emprestadas já estão na aba da classe), opções, termo e futuro.
pub fn non_position_sheet(sheet: &str) -> bool {
    let f = fold(sheet);
    f.contains("emprestimo") || f.contains("opc") || f.contains("termo") || f.contains("futuro")
}

/// Classe da aba do relatório de posição ("Acoes", "BDR", "ETF", "Fundo de Investimento",
/// "Renda Fixa", "Tesouro Direto"). `None` para abas sem posição em ativo (empréstimos,
/// opções, termo) ou desconhecidas.
pub fn position_sheet_class(sheet: &str) -> Option<AssetClass> {
    let f = fold(sheet);
    if non_position_sheet(sheet) {
        None
    } else if f.contains("tesouro") {
        Some(AssetClass::TreasuryBr)
    } else if f.contains("renda fixa") {
        Some(AssetClass::FixedIncome)
    } else if f.contains("bdr") {
        Some(AssetClass::Bdr)
    } else if f.contains("etf") {
        Some(AssetClass::EtfBr)
    } else if f.contains("fundo") || f.contains("fii") || f.contains("imobiliario") {
        Some(AssetClass::Fii)
    } else if f.contains("acao") || f.contains("acoes") {
        Some(AssetClass::StockBr)
    } else {
        None
    }
}

/// Palavras no nome que indicam FII ou ETF.
fn name_says_fii(name: &str) -> bool {
    let f = format!(" {} ", fold(name));
    f.contains(" fii ")
        || f.contains("fdo inv imob")
        || f.contains("fundo de investimento imobiliario")
        || f.contains("fundo invest imob")
        || f.contains("fundo de investimento imob")
        || f.contains(" fiagro ")
}

fn name_says_etf(name: &str) -> bool {
    let f = format!(" {} ", fold(name));
    f.contains(" etf ")
        || f.contains("fundo de indice")
        || f.contains("fdo indice")
        || f.contains("fundo indice")
        || f.contains("ishares")
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassGuess {
    pub class: AssetClass,
    pub reason: String,
    /// Presunção fraca (ex.: código terminado em 11 sem nome): confira.
    pub weak: bool,
}

/// Classe de um ativo novo. Ordem: aba da posição (mais confiável) → Tesouro e renda fixa
/// pelo nome → BDR pelo sufixo 34/35/39 → FII/ETF pelo nome → ação.
pub fn infer_class(
    ticker: Option<&str>,
    name: &str,
    sheet_class: Option<AssetClass>,
    has_name: bool,
) -> ClassGuess {
    // `reason`: chave do motivo no catálogo.
    let g = |class, reason: &str, weak| ClassGuess {
        class,
        reason: t(reason),
        weak,
    };
    if let Some(c) = sheet_class {
        return g(c, "nucleo-importacao-b3-classe-aba", false);
    }
    if is_treasury_name(name) {
        return g(
            AssetClass::TreasuryBr,
            "nucleo-importacao-b3-classe-tesouro",
            false,
        );
    }
    let Some(t) = ticker else {
        return if fixed_income_kind(name).is_some() {
            g(
                AssetClass::FixedIncome,
                "nucleo-importacao-b3-classe-renda-fixa",
                false,
            )
        } else {
            g(
                AssetClass::Other,
                "nucleo-importacao-b3-classe-sem-codigo",
                true,
            )
        };
    };
    if ["34", "35", "39", "31", "32", "33"]
        .iter()
        .any(|s| t.ends_with(s) && t.len() == 6)
    {
        return g(AssetClass::Bdr, "nucleo-importacao-b3-classe-bdr", false);
    }
    if name_says_fii(name) {
        return g(AssetClass::Fii, "nucleo-importacao-b3-classe-fii", false);
    }
    if name_says_etf(name) {
        return g(AssetClass::EtfBr, "nucleo-importacao-b3-classe-etf", false);
    }
    if t.ends_with("11") && !has_name {
        return g(
            AssetClass::StockBr,
            "nucleo-importacao-b3-classe-final-11",
            true,
        );
    }
    g(
        AssetClass::StockBr,
        "nucleo-importacao-b3-classe-acao",
        false,
    )
}

// ---------------------------------------------------------------------------
// Negociação: mercado

#[derive(Debug, Clone, PartialEq)]
pub enum MarketRule {
    /// Mercado à vista ou fracionário (e leilão, que é negócio à vista).
    Spot,
    Ignore(B3IgnoreRule),
}

/// O que fazer com a linha de negociação conforme o mercado.
///
/// - **Opções** (opção de compra/venda, exercício): o app não acompanha opções; o prêmio e
///   o ativo-objeto do exercício precisam ser lançados à parte.
/// - **Termo**: o ativo só é seu na liquidação do termo, que aparece depois como negócio à
///   vista/liquidação; lançar o termo contaria duas vezes.
/// - **Futuro**: não é posição em ativo (só ajustes diários).
/// - Mercado vazio (CSV sem a coluna) vale como à vista.
pub fn market_rule(market: &str) -> MarketRule {
    let f = fold(market);
    if f.is_empty() || f.contains("vista") || f.contains("fracionario") || f.contains("leilao") {
        MarketRule::Spot
    } else if f.contains("opc") || f.contains("exercicio") {
        MarketRule::Ignore(B3IgnoreRule::Option)
    } else if f.contains("termo") {
        MarketRule::Ignore(B3IgnoreRule::Forward)
    } else if f.contains("futuro") {
        MarketRule::Ignore(B3IgnoreRule::Future)
    } else {
        MarketRule::Ignore(B3IgnoreRule::UnsupportedMarket)
    }
}

// ---------------------------------------------------------------------------
// Movimentação

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveRule {
    /// "Transferência - Liquidação" (e "Compra/Venda"): liquidação de um negócio. Crédito =
    /// compra, débito = venda. Duplica o relatório de negociação, por isso só é usada sem
    /// ele (e nunca as duas para o mesmo período). Tesouro e renda fixa não estão na
    /// negociação, então para eles a liquidação é sempre a operação.
    Settlement,
    /// Resgate e vencimento (Tesouro, renda fixa, fundos): venda da quantidade.
    Redemption,
    /// Provento em dinheiro. `Rendimento` vira rendimento de FII para FIIs.
    Income(IncomeKind),
    /// "Dividendo - Transferido" e afins: repasse do provento entre agentes. Se a linha
    /// principal (mesma data, ativo, instituição e valor) existe, é ignorada; senão, é o
    /// próprio provento.
    ForwardedIncome(IncomeKind),
    /// Desdobro, grupamento e bonificação: evento corporativo com fator calculado pelas
    /// quantidades (posição antes do evento × quantidade creditada/baixada).
    Action(CorporateActionKind),
    /// "Fração em Ativos": a fração (ex.: 0,5 ação de uma bonificação de 10% sobre 105) já
    /// entra no fator do evento e sai da posição no leilão; a linha em si é ignorada.
    Fraction,
    /// "Leilão de Fração": venda da fração que sobrou na posição, pelo valor recebido. Se não
    /// há fração na posição, vira provento "outros".
    FractionAuction,
    /// "Atualização": either a periodic snapshot of the custody (credits the quantity already
    /// held, without a price: ignored) or the real arrival of new shares after an exercised
    /// subscription or a conversion. Decided by the context in `plan`.
    Update,
    Ignore(B3IgnoreRule),
}

/// Regra da movimentação pelo nome ("Movimentação"), já sem diferenciar maiúsculas/acentos.
///
/// Ignoradas, com o motivo:
/// - **Transferência** (sem "Liquidação"): troca de custódia entre instituições; não muda a
///   posição total. O app não move o custo entre contas; quem trocou de corretora pode
///   apontar as duas instituições para a mesma conta.
/// - **Empréstimo**: aluguel de ações; as ações emprestadas continuam suas e a ida e a volta
///   não mudam a posição.
/// - **Direito de subscrição / cessão de direitos / recibo / solicitação**: o direito não é
///   posição até ser exercido; a subscrição exercida deve ser lançada como compra.
/// - **Atualização** ([`MoveRule::Update`]): a snapshot of the custody is ignored; the
///   arrival of subscribed shares becomes a buy (see `plan`).
/// - **Incorporação, cisão, subscrição, conversão, restituição de capital**: exigem o ativo
///   resultante, a proporção ou o custo, que o relatório não traz; registre em Eventos.
pub fn move_rule(movement: &str) -> MoveRule {
    let f = fold(movement);
    let forwarded = f.ends_with(" transferido") || f.ends_with(" transferida");
    let base = f
        .trim_end_matches(" transferido")
        .trim_end_matches(" transferida")
        .trim()
        .to_string();
    let income = |k: IncomeKind| {
        if forwarded {
            MoveRule::ForwardedIncome(k)
        } else {
            MoveRule::Income(k)
        }
    };
    let b = base.as_str();
    match b {
        "transferencia liquidacao"
        | "compra venda"
        | "compra"
        | "venda"
        | "compra e venda"
        | "aplicacao"
        | "aplicacao inicial" => MoveRule::Settlement,
        "transferencia" | "transferencia custodia" | "transferencia de custodia" => {
            MoveRule::Ignore(B3IgnoreRule::CustodyTransfer)
        }
        "resgate" | "vencimento" | "resgate antecipado" | "resgate total" => MoveRule::Redemption,
        "dividendo" | "dividendos" => income(IncomeKind::Dividend),
        "juros sobre capital proprio" | "jcp" | "juros s capital proprio" => {
            income(IncomeKind::Jcp)
        }
        "rendimento" | "rendimentos" => income(IncomeKind::FiiIncome),
        "amortizacao" => income(IncomeKind::Amortization),
        "juros" | "pagamento de juros" | "cupom" | "pagamento de cupom" => {
            income(IncomeKind::Interest)
        }
        "reembolso" | "premio" | "premio de desempenho" => income(IncomeKind::Other),
        "desdobro" | "desdobramento" => MoveRule::Action(CorporateActionKind::Split),
        "grupamento" | "agrupamento" => MoveRule::Action(CorporateActionKind::ReverseSplit),
        "bonificacao em ativos" | "bonificacao" | "bonificacao em acoes" => {
            MoveRule::Action(CorporateActionKind::Bonus)
        }
        "fracao em ativos" | "fracao" => MoveRule::Fraction,
        "leilao de fracao" | "leilao de fracoes" => MoveRule::FractionAuction,
        "atualizacao" | "atualizacao de saldo" => MoveRule::Update,
        _ if b.starts_with("emprestimo") => MoveRule::Ignore(B3IgnoreRule::Lending),
        _ if b.starts_with("direito")
            || b.starts_with("direitos")
            || b.starts_with("cessao de direito")
            || b.starts_with("recibo de subscricao")
            || b.starts_with("solicitacao de subscricao") =>
        {
            MoveRule::Ignore(B3IgnoreRule::SubscriptionRight)
        }
        _ if b.starts_with("incorporacao")
            || b.starts_with("cisao")
            || b.starts_with("subscricao")
            || b.starts_with("conversao")
            || b.starts_with("restituicao de capital")
            || b.starts_with("reducao de capital")
            || b.starts_with("grupamento e desdobro") =>
        {
            MoveRule::Ignore(B3IgnoreRule::ManualEvent)
        }
        _ if b.starts_with("transferencia") => MoveRule::Ignore(B3IgnoreRule::CustodyTransfer),
        _ => MoveRule::Ignore(B3IgnoreRule::UnknownMovement),
    }
}

impl B3IgnoreRule {
    /// Motivo para o usuário, no idioma da instalação.
    pub fn reason(self) -> String {
        t(match self {
            B3IgnoreRule::Option => "nucleo-importacao-b3-ignorada-opcao",
            B3IgnoreRule::Forward => "nucleo-importacao-b3-ignorada-termo",
            B3IgnoreRule::Future => "nucleo-importacao-b3-ignorada-futuro",
            B3IgnoreRule::UnsupportedMarket => "nucleo-importacao-b3-ignorada-mercado",
            B3IgnoreRule::Lending => "nucleo-importacao-b3-ignorada-emprestimo",
            B3IgnoreRule::CustodyTransfer => "nucleo-importacao-b3-ignorada-custodia",
            B3IgnoreRule::SettlementDuplicate => "nucleo-importacao-b3-ignorada-liquidacao",
            B3IgnoreRule::SubscriptionRight => "nucleo-importacao-b3-ignorada-subscricao",
            B3IgnoreRule::BalanceUpdate => "nucleo-importacao-b3-ignorada-atualizacao",
            B3IgnoreRule::CustodySnapshot => "nucleo-importacao-b3-ignorada-retrato",
            B3IgnoreRule::ManualEvent => "nucleo-importacao-b3-ignorada-evento-manual",
            B3IgnoreRule::ForwardedIncome => "nucleo-importacao-b3-ignorada-repasse",
            B3IgnoreRule::IncomeReversal => "nucleo-importacao-b3-ignorada-estorno",
            B3IgnoreRule::NoValue => "nucleo-importacao-b3-ignorada-sem-valor",
            B3IgnoreRule::Fraction => "nucleo-importacao-b3-ignorada-fracao",
            B3IgnoreRule::UnknownMovement => "nucleo-importacao-b3-ignorada-desconhecida",
            B3IgnoreRule::Invalid => "nucleo-importacao-b3-ignorada-invalida",
        })
    }
}

/// Root of a B3 code ("ABCD" in ABCD11, ABCD12, ABCD13): the subscription right (12),
/// the receipt (13/15) and the shares (11/3/4) share it.
pub fn ticker_root(ticker: &str) -> Option<&str> {
    normalize_ticker(ticker)?;
    ticker.get(..4)
}

/// The row is evidence that a subscription was exercised: a receipt or a request, or a
/// right that was not marked as "not exercised".
pub fn exercised_subscription(movement: &str) -> bool {
    let f = fold(movement);
    f.starts_with("recibo de subscricao")
        || f.starts_with("solicitacao de subscricao")
        || (f.starts_with("direito") && !f.contains("nao exercido"))
}

// ---------------------------------------------------------------------------
// Instituições → contas

/// Palavras genéricas em nomes de corretoras, que não identificam a instituição.
const INSTITUTION_STOPWORDS: &[&str] = &[
    "s",
    "a",
    "sa",
    "ltda",
    "cctvm",
    "ctvm",
    "dtvm",
    "cvm",
    "cvmc",
    "ccvm",
    "cv",
    "corretora",
    "corretor",
    "distribuidora",
    "de",
    "do",
    "da",
    "dos",
    "das",
    "e",
    "valores",
    "mobiliarios",
    "titulos",
    "cambio",
    "investimentos",
    "investimento",
    "invest",
    "banco",
    "bco",
    "securities",
    "financeira",
    "sociedade",
    "agente",
    "fiduciario",
    "bm",
    "bmf",
    "mercadorias",
    "futuros",
    "cooperativa",
    "cooperativo",
    "credito",
    "unibanco",
    "pagamentos",
    "ip",
    "scfi",
    "multiplo",
];

/// Palavras que identificam a instituição, na ordem (sem o que vem depois de "grupo":
/// "RICO INVESTIMENTOS - GRUPO XP" → ["rico"]).
pub fn institution_tokens(name: &str) -> Vec<String> {
    let f = fold(name);
    let mut out = vec![];
    for w in f.split(' ') {
        if w == "grupo" {
            break;
        }
        if w.is_empty() || INSTITUTION_STOPWORDS.contains(&w) {
            continue;
        }
        out.push(w.to_string());
    }
    out
}

/// Score of an account (account name + registered institution name) for a custodian of
/// the B3: each identifying word of the custodian found as a whole word counts 10; found as
/// the start of a word ("nu" in "NuInvest", "nu invest") counts 6. Zero when none matches.
pub fn institution_score(tokens: &[String], account_text: &str) -> u32 {
    let words: Vec<String> = fold(account_text)
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect();
    let joined: String = words.concat();
    let mut seen: Vec<&String> = vec![];
    let mut score = 0;
    for t in tokens {
        if seen.contains(&t) {
            continue;
        }
        seen.push(t);
        if words.iter().any(|w| w == t) {
            score += 10;
        } else if t.len() >= 2
            && (words.iter().any(|w| w.starts_with(t.as_str())) || joined.starts_with(t.as_str()))
        {
            score += 6;
        }
    }
    score
}

/// A custodian known under more than one name (renamed brokers).
pub struct Custodian {
    pub key: &'static str,
    /// Name for a new account.
    pub display: &'static str,
    /// CNPJ, digits only.
    pub cnpj: Option<&'static str>,
    pub broker: bool,
    /// Starts of the folded names it appears under.
    pub names: &'static [&'static str],
}

/// Custodians whose name changed over time. The B3 reports do not bring the custodian's
/// CNPJ (the CNPJ columns are the company's or the fund's), so the names are matched.
pub const KNOWN_CUSTODIANS: &[Custodian] = &[Custodian {
    key: "nu-invest",
    display: "Nu Invest",
    // Former Easynvest - Título Corretora de Valores.
    cnpj: Some("62169875000179"),
    broker: true,
    names: &["nu investimentos", "nu invest", "easynvest"],
}];

pub fn known_custodian(name: &str) -> Option<&'static Custodian> {
    let f = fold(name);
    KNOWN_CUSTODIANS
        .iter()
        .find(|c| c.names.iter().any(|n| f.starts_with(n)))
}

/// Key that groups the names of the same custodian: the known custodian's key, or its
/// identifying words.
pub fn custodian_group(name: &str) -> String {
    match known_custodian(name) {
        Some(c) => format!("known:{}", c.key),
        None => {
            let tokens = institution_tokens(name);
            if tokens.is_empty() {
                format!("name:{}", fold(name))
            } else {
                format!("name:{}", tokens.join(" "))
            }
        }
    }
}

/// The name looks like a broker's ("corretora", "CTVM", "DTVM", "invest").
pub fn looks_like_broker(name: &str) -> bool {
    let f = format!(" {} ", fold(name));
    [
        " corretora ",
        " ctvm ",
        " cctvm ",
        " dtvm ",
        " invest ",
        " investimentos ",
        " securities ",
    ]
    .iter()
    .any(|w| f.contains(w))
}

/// Short name of a custodian for a new account: the known name, the acronym after " - "
/// ("COOPERATIVA … - VIACREDI" → "Viacredi") or the first identifying word, capitalized.
pub fn custodian_short_name(name: &str) -> String {
    if let Some(c) = known_custodian(name) {
        return c.display.to_string();
    }
    let cap = |w: &str| {
        let lower = w.to_lowercase();
        let mut c = lower.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    };
    if let Some((_, tail)) = name.rsplit_once(" - ") {
        let tail = tail.trim();
        if !tail.is_empty()
            && !tail.contains(' ')
            && !INSTITUTION_STOPWORDS.contains(&fold(tail).as_str())
        {
            return cap(tail);
        }
    }
    let first = name
        .split(|c: char| !c.is_alphanumeric())
        .find(|w| !w.is_empty() && !INSTITUTION_STOPWORDS.contains(&fold(w).as_str()));
    first.map(cap).unwrap_or_else(|| name.trim().to_string())
}
