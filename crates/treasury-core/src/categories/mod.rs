//! Categorias (árvore) de receitas e despesas, com um conjunto inicial por país. Os nomes do
//! conjunto inicial vêm do catálogo `locales/*/categorias.ftl` no idioma da instalação, e cada
//! categoria criada por ele guarda a chave de origem (`seed_key`, V127) para ser reconhecida e
//! traduzida depois (docs/i18n-design.md › Dados semeados).

pub mod api;

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, Locale};
use crate::ids::{Id, new_id};
use crate::time::now_utc;
use crate::transactions::util::{clean_color, clean_name, clean_opt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CategoryKind {
    Income,
    Expense,
}

impl CategoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CategoryKind::Income => "income",
            CategoryKind::Expense => "expense",
        }
    }
    pub fn parse(s: &str) -> Self {
        if s == "income" {
            CategoryKind::Income
        } else {
            CategoryKind::Expense
        }
    }
}

/// Grupo de gasto de uma categoria de despesa: para onde vai a renda (V129). Receitas não têm
/// grupo. Nomes e dicas no catálogo `grupos.ftl` (`grupos-<grupo>`, `grupos-<grupo>-dica`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SpendGroup {
    /// O que não dá para cortar: moradia, mercado, contas da casa, transporte, saúde, escola.
    Essential,
    /// Parcelas de empréstimos e financiamentos, juros, tarifas e encargos.
    Debt,
    /// O que fica guardado: reserva, investimentos, previdência e metas.
    Future,
    /// O que dá para ajustar: lazer, restaurantes, compras, assinaturas, viagens.
    NonEssential,
}

impl SpendGroup {
    /// Os quatro, na ordem de exibição.
    pub const ALL: [SpendGroup; 4] = [
        SpendGroup::Essential,
        SpendGroup::Debt,
        SpendGroup::Future,
        SpendGroup::NonEssential,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            SpendGroup::Essential => "essential",
            SpendGroup::Debt => "debt",
            SpendGroup::Future => "future",
            SpendGroup::NonEssential => "non_essential",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.as_str() == s)
    }

    /// Nome do grupo no idioma (`grupos.ftl`).
    pub fn label_in(self, l: Locale) -> String {
        let key = match self {
            SpendGroup::Essential => "grupos-essential",
            SpendGroup::Debt => "grupos-debt",
            SpendGroup::Future => "grupos-future",
            SpendGroup::NonEssential => "grupos-non_essential",
        };
        i18n::tr(l, key, None)
    }

    /// Nome do grupo no idioma da instalação.
    pub fn label(self) -> String {
        self.label_in(i18n::current())
    }

    /// Conta como gasto essencial (reserva de emergência, saúde financeira): essenciais e
    /// dívidas, que são obrigações. `is_essential` é sempre isto.
    pub fn is_essential(self) -> bool {
        matches!(self, SpendGroup::Essential | SpendGroup::Debt)
    }

    /// Grupo de quem só tem o "essencial" (cofre de antes da V129).
    fn from_essential(essential: bool) -> Self {
        if essential {
            SpendGroup::Essential
        } else {
            SpendGroup::NonEssential
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Category {
    pub id: Id,
    pub parent_id: Option<Id>,
    pub name: String,
    pub kind: CategoryKind,
    pub icon: Option<String>,
    pub color: Option<String>,
    /// Gasto essencial: entra no cálculo da reserva de emergência. Nas despesas, o grupo é
    /// `essential` ou `debt` (dívidas são obrigações).
    pub is_essential: bool,
    /// Grupo de gasto (despesas); `null` nas receitas.
    pub spend_group: Option<SpendGroup>,
    pub archived: bool,
    #[ts(type = "number")]
    pub sort_order: i64,
    /// Caminho completo para exibição: `Alimentação › Mercado`.
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryNode {
    #[serde(flatten)]
    pub category: Category,
    pub children: Vec<CategoryNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryInput {
    pub name: String,
    pub kind: CategoryKind,
    #[ts(optional)]
    pub parent_id: Option<Id>,
    #[ts(optional)]
    pub icon: Option<String>,
    #[ts(optional)]
    pub color: Option<String>,
    /// Sem `spend_group`, vale o grupo do pai (sem pai, não essencial), ajustado por este:
    /// `true` num grupo não essencial ou futuro leva a "essenciais"; `false` em essenciais ou
    /// dívidas leva a "não essencial".
    #[ts(optional)]
    pub is_essential: Option<bool>,
    /// Grupo de gasto (só despesas). Vence `is_essential`.
    #[serde(default)]
    #[ts(optional)]
    pub spend_group: Option<SpendGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryPatch {
    pub id: Id,
    #[ts(optional)]
    pub name: Option<String>,
    /// Mover para outro pai (`null` = raiz).
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub parent_id: Option<Option<Id>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    #[ts(optional)]
    pub color: Option<Option<String>>,
    /// Sem `spend_group`: `true` num grupo não essencial ou futuro passa para "essenciais"
    /// (dívidas continuam dívidas); `false` em essenciais ou dívidas passa para "não essencial".
    #[ts(optional)]
    pub is_essential: Option<bool>,
    /// Grupo de gasto (só despesas). Vence `is_essential`, que fica verdadeiro em `essential` e
    /// `debt`.
    #[serde(default)]
    #[ts(optional)]
    pub spend_group: Option<SpendGroup>,
    #[ts(optional)]
    pub archived: Option<bool>,
    #[ts(optional, type = "number")]
    pub sort_order: Option<i64>,
}

/// Filtro de `categories.list` e `categories.tree`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryListArgs {
    #[ts(optional)]
    pub kind: Option<CategoryKind>,
    /// Incluir arquivadas (padrão: não). Filhas de uma arquivada também ficam de fora.
    #[ts(optional)]
    pub include_archived: Option<bool>,
}

/// Onde uma categoria é usada (antes de apagar) ou quanto foi reatribuído.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryUsage {
    #[ts(type = "number")]
    pub transactions: i64,
    #[ts(type = "number")]
    pub splits: i64,
    #[ts(type = "number")]
    pub budget_lines: i64,
    #[ts(type = "number")]
    pub rules: i64,
    #[ts(type = "number")]
    pub recurrences: i64,
    #[ts(type = "number")]
    pub installment_plans: i64,
    #[ts(type = "number")]
    pub children: i64,
}

impl CategoryUsage {
    fn in_use(&self) -> bool {
        self.transactions
            + self.splits
            + self.budget_lines
            + self.rules
            + self.recurrences
            + self.installment_plans
            > 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryDeleteArgs {
    pub id: Id,
    /// Categoria que recebe os lançamentos, divisões, linhas de orçamento, regras,
    /// recorrências e parcelamentos da apagada. Sem ela, só apaga se não estiver em uso.
    #[ts(optional)]
    pub reassign_to: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryMergeArgs {
    /// Categoria que deixa de existir.
    pub from: Id,
    /// Categoria que recebe tudo (inclusive as subcategorias de `from`).
    pub into: Id,
}

/// `categories.set_group`: o mesmo grupo de gasto para várias categorias de despesa (as de
/// receita da lista ficam como estão).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetGroupArgs {
    pub ids: Vec<Id>,
    pub group: SpendGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetGroupResult {
    /// Categorias que mudaram de grupo.
    pub updated: u32,
}

/// Separador usado no caminho das categorias.
pub const PATH_SEP: &str = " › ";

/// Semente: nome-base em pt-BR (a identidade dela, ver [`seed_key`]), essencial, subcategorias.
pub(crate) type Seed = (&'static str, bool, &'static [&'static str]);

const EXPENSES: &[Seed] = &[
    (
        "Moradia",
        true,
        &[
            "Aluguel",
            "Condomínio",
            "Energia",
            "Água",
            "Gás",
            "Internet",
            "IPTU",
            "Manutenção",
            "Diarista e serviços domésticos",
        ],
    ),
    (
        "Alimentação",
        true,
        &["Mercado", "Restaurantes", "Delivery", "Padaria"],
    ),
    (
        "Transporte",
        true,
        &[
            "Combustível",
            "Transporte público",
            "Aplicativos",
            "Estacionamento",
            "Manutenção do veículo",
            "IPVA e licenciamento",
            "Seguro do veículo",
        ],
    ),
    (
        "Saúde",
        true,
        &[
            "Plano de saúde",
            "Farmácia",
            "Consultas",
            "Pediatra",
            "Exames",
            "Dentista",
            "Terapias e psicólogo",
            "Vacinas",
            "Óculos e lentes",
            "Itens de saúde e primeiros socorros",
        ],
    ),
    (
        "Educação",
        true,
        &[
            "Escola",
            "Creche",
            "Material escolar",
            "Uniforme",
            "Transporte escolar",
            "Cantina e lanche escolar",
            "Faculdade",
            "Idiomas",
            "Cursos",
            "Livros e material",
        ],
    ),
    (
        "Lazer",
        false,
        &["Viagens", "Passeios", "Cinema e shows", "Hobbies"],
    ),
    (
        "Compras",
        false,
        &["Vestuário", "Eletrônicos", "Casa e decoração"],
    ),
    (
        "Cuidados pessoais",
        false,
        &["Cabelo e estética", "Academia", "Cosméticos"],
    ),
    (
        "Assinaturas",
        false,
        &["Streaming", "Software e apps", "Telefone"],
    ),
    (
        "Impostos e taxas",
        true,
        &[
            "Tarifas bancárias",
            "Juros e multas",
            "IOF",
            "Imposto de renda",
        ],
    ),
    (
        "Seguros",
        true,
        &[
            "Seguro de vida",
            "Seguro residencial",
            "Seguro viagem",
            "Seguro educacional",
        ],
    ),
    // Contas, dívidas e cartão pagos pela conta corrente. Fatura de cartão que o app acompanha
    // vira pagamento de fatura (sem categoria) ao ligar ao cartão; a categoria serve aos
    // cartões de fora do app, para não somar a compra duas vezes.
    (
        "Pagamentos",
        true,
        &[
            "Contas e boletos",
            "Empréstimos",
            "Financiamentos",
            "Parcelas",
            "Fatura do cartão",
        ],
    ),
    ("Pets", false, &["Ração", "Veterinário"]),
    ("Presentes e doações", false, &["Presentes", "Doações"]),
    (
        "Filhos",
        true,
        &[
            "Mesada",
            "Atividades",
            "Esportes e aulas",
            "Fraldas e higiene",
            "Roupas e calçados infantis",
            "Brinquedos",
            "Babá e cuidadores",
            "Festas e aniversários",
            "Pensão alimentícia",
        ],
    ),
    // Dinheiro guardado de propósito todo mês (orçado como despesa para caber no plano).
    (
        "Poupança e investimentos",
        false,
        &[
            "Poupança dos filhos",
            "Reserva de emergência",
            "Previdência privada",
            "Aportes em investimentos",
        ],
    ),
    ("Outras despesas", false, &[]),
];

// Receitas: trabalho (com carteira, por conta própria, por aplicativo e vendas informais),
// rendimentos, benefícios, aposentadoria e programas do governo; depois o dinheiro que volta do
// patrimônio (resgates, venda de bens, dívidas recebidas, empréstimos tomados), que entra no
// caixa mas não é renda do trabalho nem do capital.
const INCOMES: &[Seed] = &[
    (
        "Salário",
        false,
        &[
            "13º salário",
            "Férias",
            "PLR e bônus",
            "Horas extras",
            "Comissões",
            "Rescisão",
        ],
    ),
    ("Pró-labore", false, &[]),
    (
        "Serviços e freelance",
        false,
        &[
            "Projetos e consultoria",
            "Bicos e diárias",
            "Aulas particulares",
            "Gorjetas e caixinhas",
        ],
    ),
    (
        "Trabalho por aplicativo",
        false,
        &["Corridas", "Entregas", "Serviços por aplicativo"],
    ),
    (
        "Vendas e negócio próprio",
        false,
        &[
            "Faturamento do MEI",
            "Revenda por catálogo",
            "Comida e produtos caseiros",
            "Artesanato",
            "Vendas online",
        ],
    ),
    (
        "Rendimentos",
        false,
        &[
            "Dividendos",
            "Juros",
            "JCP",
            "Rendimentos de FII",
            "Aluguéis recebidos",
            "Aluguel por temporada",
        ],
    ),
    (
        "Benefícios",
        false,
        &["Vale-alimentação e refeição", "Auxílio-creche"],
    ),
    (
        "Aposentadoria e pensão",
        false,
        &[
            "INSS",
            "Servidor público",
            "Previdência privada",
            "Pensão por morte",
        ],
    ),
    (
        "Auxílios do governo",
        false,
        &[
            "Seguro-desemprego",
            "Auxílio por incapacidade (INSS)",
            "Bolsa Família",
            "BPC/LOAS",
            "Abono salarial (PIS/Pasep)",
            "Outros programas sociais",
        ],
    ),
    ("Pensão alimentícia recebida", false, &[]),
    (
        "Reembolsos",
        false,
        &[
            "Divisão de contas",
            "Despesas do trabalho",
            "Plano de saúde",
            "Estornos e devoluções",
            "Cashback e recompensas",
        ],
    ),
    ("Restituição de IR", false, &[]),
    (
        "Prêmios e cashback",
        false,
        &[
            "Cashback",
            "Sorteios e loterias",
            "Programas de nota fiscal",
        ],
    ),
    ("Indenizações", false, &["Seguros", "Ações judiciais"]),
    (
        "Presentes recebidos",
        false,
        &["Ajuda de familiares", "Mesada"],
    ),
    ("Herança", false, &[]),
    (
        "Resgate de investimentos",
        false,
        &["Poupança", "Previdência privada", "Saque do FGTS"],
    ),
    (
        "Venda de bens",
        false,
        &[
            "Veículos",
            "Imóveis",
            "Usados e desapegos",
            "Milhas e pontos",
        ],
    ),
    (
        "Recebimento de dívidas",
        false,
        &["Empréstimo devolvido", "Fiado e vendas a prazo"],
    ),
    ("Empréstimos recebidos", false, &[]),
    ("Outras receitas", false, &[]),
];

/// As duas tabelas do conjunto inicial, com o tipo.
const TABLES: [(CategoryKind, &[Seed]); 2] = [
    (CategoryKind::Expense, EXPENSES),
    (CategoryKind::Income, INCOMES),
];

/// Categorias de despesa do conjunto inicial (nome-base, essencial, subcategorias).
#[cfg(test)]
pub(crate) fn seed_expenses() -> &'static [Seed] {
    EXPENSES
}

/// Subcategorias herdam "essencial", exceto as de lazer óbvias dentro de grupos essenciais.
/// O "essencial" das tabelas é o de antes dos grupos de gasto (V129): o grupo padrão sai dele
/// e de [`seed_group`], e a recuperação dos cofres antigos compara com ele para saber se a
/// pessoa mudou alguma coisa ([`backfill_spend_groups`]). Não mude esses valores.
fn child_essential(parent: bool, child: &str) -> bool {
    parent
        && !matches!(
            child,
            "Restaurantes"
                | "Delivery"
                | "Aplicativos"
                | "Brinquedos"
                | "Festas e aniversários"
                | "Seguro viagem"
        )
}

/// Grupos de gasto que não saem do "essencial" nem do pai: (pai, nome-base, grupo); sem pai,
/// categoria de primeiro nível. As subcategorias de um grupo de primeiro nível daqui herdam o
/// grupo dele.
const GROUP_EXCEPTIONS: &[(Option<&str>, &str, SpendGroup)] = &[
    (None, "Poupança e investimentos", SpendGroup::Future),
    (
        Some("Impostos e taxas"),
        "Tarifas bancárias",
        SpendGroup::Debt,
    ),
    (Some("Impostos e taxas"), "Juros e multas", SpendGroup::Debt),
    (Some("Pagamentos"), "Empréstimos", SpendGroup::Debt),
    (Some("Pagamentos"), "Financiamentos", SpendGroup::Debt),
    (Some("Pagamentos"), "Parcelas", SpendGroup::Debt),
];

/// Grupo de gasto padrão de uma semente de despesa (pai com o grupo dele, nome-base e o
/// "essencial" de antes): a exceção da tabela; senão o do pai, quando ele não é "essenciais"
/// (Lazer, Poupança e investimentos…); senão "essenciais" ou "não essencial" pelo "essencial".
fn seed_group(parent: Option<(&str, SpendGroup)>, name: &str, essential: bool) -> SpendGroup {
    if let Some((_, _, g)) = GROUP_EXCEPTIONS
        .iter()
        .find(|(p, n, _)| *p == parent.map(|x| x.0) && *n == name)
    {
        return *g;
    }
    match parent {
        Some((_, g)) if g != SpendGroup::Essential => g,
        _ => SpendGroup::from_essential(essential),
    }
}

/// Grupo padrão e "essencial" de antes de cada semente de despesa, pela chave.
fn seed_groups_index() -> &'static HashMap<String, (SpendGroup, bool)> {
    static INDEX: OnceLock<HashMap<String, (SpendGroup, bool)>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut out = HashMap::new();
        for (name, essential, children) in EXPENSES {
            let g = seed_group(None, name, *essential);
            out.insert(seed_key(None, name), (g, *essential));
            for c in children.iter() {
                let e = child_essential(*essential, c);
                out.insert(
                    seed_key(Some(name), c),
                    (seed_group(Some((name, g)), c, e), e),
                );
            }
        }
        out
    })
}

// ---------------------------------------------------------------------------
// Nomes do conjunto inicial (docs/i18n-design.md › Dados semeados)
//
// O nome-base em pt-BR das tabelas acima é a identidade da semente: decide se ela existe no país
// e se é essencial, e dá a chave do catálogo `locales/*/categorias.ftl`. O nome que aparece sai
// do catálogo, no idioma da instalação e com a variante do país. Para mudar um nome, mude o texto
// no catálogo; o nome-base daqui não muda (a chave fica gravada nos cofres).

/// Prefixo das chaves do catálogo das categorias.
const KEY_PREFIX: &str = "categorias-";

/// Subcategorias de nome repetido em outro grupo: a chave leva o grupo
/// (`categorias-filhos-mesada`). Um nome repetido novo entra aqui só com o par novo, para as
/// chaves que já existem não mudarem.
const QUALIFIED: &[(&str, &str)] = &[
    ("Saúde", "Plano de saúde"),
    ("Reembolsos", "Plano de saúde"),
    ("Filhos", "Mesada"),
    ("Presentes recebidos", "Mesada"),
    ("Poupança e investimentos", "Previdência privada"),
    ("Aposentadoria e pensão", "Previdência privada"),
    ("Resgate de investimentos", "Previdência privada"),
    ("Indenizações", "Seguros"),
];

/// Variantes por país no catálogo, além do nome-base.
const ALL_VARIANTS: [&str; 11] = [
    "", "--us", "--pt", "--es", "--gb", "--ca", "--fr", "--de", "--ar", "--cl", "--outro",
];

/// Nome-base sem acentos, em minúsculas e com hífens (`13º salário` → `13o-salario`).
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars().map(crate::transactions::fold_accent) {
        let c = if c == 'º' {
            'o'
        } else {
            c.to_ascii_lowercase()
        };
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// Chave da semente no catálogo, pelo nome-base (e o do grupo, para as subcategorias).
pub(crate) fn seed_key(parent: Option<&str>, name: &str) -> String {
    match parent {
        Some(p) if QUALIFIED.contains(&(p, name)) => {
            format!("{KEY_PREFIX}{}-{}", slug(p), slug(name))
        }
        _ => format!("{KEY_PREFIX}{}", slug(name)),
    }
}

/// Variantes do país, na ordem de procura (o nome-base vem depois): Estados Unidos, Portugal e
/// Espanha têm as suas e caem nas dos outros países quando não têm; o Brasil usa o nome-base.
fn variants(country: &str) -> &'static [&'static str] {
    match country {
        "BR" => &[],
        "US" => &["--us", "--outro"],
        "PT" => &["--pt", "--outro"],
        "ES" => &["--es", "--outro"],
        "GB" => &["--gb", "--outro"],
        "CA" => &["--ca", "--outro"],
        "FR" => &["--fr", "--outro"],
        "DE" => &["--de", "--outro"],
        "AR" => &["--ar", "--outro"],
        "CL" => &["--cl", "--outro"],
        _ => &["--outro"],
    }
}

/// A semente existe no país? Sai o que só existe no Brasil (IOF, JCP, FII, FGTS…) ou não tem
/// equivalente lá (docs/countries.md, docs/international.md).
fn exists_in(country: &str, base: &str) -> bool {
    if country == "BR" {
        return true;
    }
    if base == "Saque do FGTS" {
        return false;
    }
    match country {
        "US" => !matches!(
            base,
            "13º salário" | "Férias" | "Vale-alimentação e refeição" | "IOF" | "JCP"
        ),
        "PT" => !matches!(
            base,
            "JCP" | "Rendimentos de FII" | "Abono salarial (PIS/Pasep)"
        ),
        // Espanha: as pagas extra ficam (no lugar do 13º); não há férias pagas à parte.
        "ES" => !matches!(
            base,
            "IOF"
                | "JCP"
                | "Rendimentos de FII"
                | "Abono salarial (PIS/Pasep)"
                | "BPC/LOAS"
                | "Férias"
        ),
        // Argentina: o aguinaldo fica (no lugar do 13º).
        "AR" => !matches!(
            base,
            "IOF"
                | "JCP"
                | "Rendimentos de FII"
                | "Abono salarial (PIS/Pasep)"
                | "Bolsa Família"
                | "BPC/LOAS"
                | "Férias"
        ),
        // 13º e férias só onde o app conhece os pagamentos extras do ano.
        _ => !matches!(
            base,
            "IOF"
                | "JCP"
                | "Rendimentos de FII"
                | "Abono salarial (PIS/Pasep)"
                | "Bolsa Família"
                | "BPC/LOAS"
                | "13º salário"
                | "Férias"
        ),
    }
}

/// Nome da semente no idioma e no país: em cada idioma da cadeia de reserva, a variante do país
/// e depois o nome-base.
pub(crate) fn seed_name(l: Locale, country: &str, key: &str) -> String {
    for loc in l.chain() {
        for v in variants(country).iter().copied().chain([""]) {
            let k = format!("{key}{v}");
            if i18n::has_key(*loc, &k) {
                return i18n::tr(*loc, &k, None);
            }
        }
    }
    key.to_string()
}

/// Todas as chaves do conjunto inicial (de todos os países).
fn all_seed_keys() -> impl Iterator<Item = String> {
    TABLES.into_iter().flat_map(|(_, seeds)| {
        seeds.iter().flat_map(|(name, _, children)| {
            std::iter::once(seed_key(None, name))
                .chain(children.iter().map(|c| seed_key(Some(name), c)))
        })
    })
}

/// Nomes padrão de cada semente: o texto de cada variante em cada idioma (os catálogos vêm
/// dentro do app, então a tabela é montada uma vez).
fn seed_names_index() -> &'static HashMap<String, Vec<(Locale, String)>> {
    static INDEX: OnceLock<HashMap<String, Vec<(Locale, String)>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        all_seed_keys()
            .map(|key| {
                let mut names: Vec<(Locale, String)> = vec![];
                for l in i18n::AVAILABLE {
                    for v in ALL_VARIANTS {
                        let k = format!("{key}{v}");
                        if i18n::has_key(l, &k) {
                            names.push((l, i18n::tr(l, &k, None)));
                        }
                    }
                }
                (key, names)
            })
            .collect()
    })
}

/// O nome é um nome padrão da semente (em algum idioma, de algum país)? Só pt-BR com `br`.
fn is_default_name(key: &str, name: &str, br: bool) -> bool {
    seed_names_index().get(key).is_some_and(|v| {
        v.iter()
            .any(|(l, s)| s == name && (!br || *l == Locale::PtBr))
    })
}

/// Categoria do conjunto inicial no país, com o nome no idioma.
#[derive(Debug, Clone)]
pub(crate) struct SeedCat {
    pub key: String,
    pub name: String,
    /// `group == essential`.
    pub essential: bool,
    /// Grupo de gasto padrão (só despesas).
    pub group: Option<SpendGroup>,
}

impl SeedCat {
    fn new(kind: CategoryKind, key: String, name: String, group: SpendGroup) -> Self {
        let group = (kind == CategoryKind::Expense).then_some(group);
        SeedCat {
            key,
            name,
            essential: group.is_some_and(SpendGroup::is_essential),
            group,
        }
    }

    fn group_str(&self) -> Option<&'static str> {
        self.group.map(SpendGroup::as_str)
    }
}

/// Grupo do conjunto inicial e as subcategorias dele.
#[derive(Debug, Clone)]
pub(crate) struct SeedGroup {
    pub cat: SeedCat,
    pub children: Vec<SeedCat>,
}

/// O conjunto inicial do país (só o que existe lá), com os nomes no idioma.
pub(crate) fn seed_plan(country: &str, l: Locale) -> Vec<(CategoryKind, Vec<SeedGroup>)> {
    TABLES
        .into_iter()
        .map(|(kind, seeds)| {
            let groups = seeds
                .iter()
                .filter(|(name, _, _)| exists_in(country, name))
                .map(|(name, essential, children)| {
                    let key = seed_key(None, name);
                    let group = seed_group(None, name, *essential);
                    SeedGroup {
                        cat: SeedCat::new(kind, key.clone(), seed_name(l, country, &key), group),
                        children: children
                            .iter()
                            .filter(|c| exists_in(country, c))
                            .map(|c| {
                                let key = seed_key(Some(name), c);
                                let g = seed_group(
                                    Some((name, group)),
                                    c,
                                    child_essential(*essential, c),
                                );
                                SeedCat::new(kind, key.clone(), seed_name(l, country, &key), g)
                            })
                            .collect(),
                    }
                })
                .collect();
            (kind, groups)
        })
        .collect()
}

/// Conjunto inicial (grupos e subcategorias) com os nomes do país no idioma atual.
#[cfg(test)]
pub(crate) type LocalSeed = (CategoryKind, Vec<(&'static str, bool, Vec<&'static str>)>);

#[cfg(test)]
pub(crate) fn seeds_for(country: &str) -> Vec<LocalSeed> {
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    seed_plan(country, i18n::current())
        .into_iter()
        .map(|(kind, groups)| {
            let groups = groups
                .into_iter()
                .map(|g| {
                    let kids = g.children.into_iter().map(|c| leak(c.name)).collect();
                    (leak(g.cat.name), g.cat.essential, kids)
                })
                .collect();
            (kind, groups)
        })
        .collect()
}

pub fn seed_defaults(tx: &Transaction<'_>) -> Result<()> {
    seed_defaults_for(tx, "BR")
}

/// Cria o conjunto inicial (editável depois) com as categorias do país da família e os nomes
/// no idioma da instalação, cada uma com a chave de origem.
pub fn seed_defaults_for(tx: &Transaction<'_>, country: &str) -> Result<()> {
    let now = now_utc();
    let mut stmt = tx.prepare(
        "INSERT INTO category (id, parent_id, name, kind, is_essential, sort_order, seed_key, created_at, updated_at, spend_group)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9)",
    )?;
    for (kind, groups) in seed_plan(country, i18n::current()) {
        for (i, g) in groups.iter().enumerate() {
            let pid = new_id();
            stmt.execute(params![
                pid,
                None::<String>,
                g.cat.name,
                kind.as_str(),
                g.cat.essential,
                i as i64,
                g.cat.key,
                now,
                g.cat.group_str()
            ])?;
            for (j, c) in g.children.iter().enumerate() {
                stmt.execute(params![
                    new_id(),
                    pid,
                    c.name,
                    kind.as_str(),
                    c.essential,
                    j as i64,
                    c.key,
                    now,
                    c.group_str()
                ])?;
            }
        }
    }
    Ok(())
}

/// Categoria do conjunto inicial que ainda não existe no cofre.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MissingDefault {
    pub kind: CategoryKind,
    /// Caminho que será criado ("Filhos › Fraldas e higiene").
    pub path: String,
    /// Só a subcategoria é nova (o grupo já existe).
    pub parent_exists: bool,
}

/// A categoria do cofre que é esta semente: pela chave de origem e, nas que não têm chave, pelo
/// nome (sem acentos nem maiúsculas, em qualquer idioma) no mesmo lugar. Grupos ou
/// subcategorias arquivados contam como existentes.
fn find_seed<'a>(
    rows: &'a [Row],
    kind: CategoryKind,
    parent: Option<&str>,
    seed: &SeedCat,
) -> Option<&'a Row> {
    if let Some(r) = rows
        .iter()
        .find(|r| r.kind == kind && r.seed_key.as_deref() == Some(seed.key.as_str()))
    {
        return Some(r);
    }
    let mut names: HashSet<String> = seed_names_index()
        .get(&seed.key)
        .into_iter()
        .flatten()
        .map(|(_, s)| fold(s))
        .collect();
    names.insert(fold(&seed.name));
    rows.iter().find(|r| {
        r.kind == kind
            && r.seed_key.is_none()
            && r.parent_id.as_deref() == parent
            && names.contains(&fold(&r.name))
    })
}

/// Categorias do conjunto inicial que faltam neste cofre (pela chave de origem; nas sem chave,
/// pelo nome), com os nomes no idioma atual.
pub fn missing_defaults(conn: &Connection) -> Result<Vec<MissingDefault>> {
    let rows = load_rows(conn)?;
    let mut out = vec![];
    let country = crate::country::family_country(conn);
    for (kind, groups) in seed_plan(&country, i18n::current()) {
        for g in &groups {
            match find_seed(&rows, kind, None, &g.cat) {
                None => {
                    out.push(MissingDefault {
                        kind,
                        path: g.cat.name.clone(),
                        parent_exists: false,
                    });
                    for c in &g.children {
                        out.push(MissingDefault {
                            kind,
                            path: format!("{}{PATH_SEP}{}", g.cat.name, c.name),
                            parent_exists: false,
                        });
                    }
                }
                Some(p) => {
                    for c in &g.children {
                        if find_seed(&rows, kind, Some(&p.id), c).is_none() {
                            out.push(MissingDefault {
                                kind,
                                path: format!("{}{PATH_SEP}{}", p.name, c.name),
                                parent_exists: true,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Quais sugestões adicionar: os caminhos de [`missing_defaults`] (sem a lista, todas).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct AddDefaultsArgs {
    #[ts(optional)]
    pub paths: Option<Vec<String>>,
}

/// Cria as categorias do conjunto inicial que faltam (todas, ou só as de `only`, pelos
/// caminhos de [`missing_defaults`]), sem mudar nem apagar as existentes, com os nomes no
/// idioma atual e a chave de origem. Uma subcategoria escolhida cria o grupo dela, se faltar.
/// Devolve quantas criou.
pub fn add_missing_defaults(tx: &Transaction<'_>, only: Option<&[String]>) -> Result<u32> {
    let wanted: Option<HashSet<String>> = only.map(|p| p.iter().map(|s| fold(s)).collect());
    let picked = |path: &str| wanted.as_ref().is_none_or(|w| w.contains(&fold(path)));
    let now = now_utc();
    let mut n = 0;
    let insert = |pid: Option<&str>, c: &SeedCat, kind: CategoryKind, sort: usize| -> Result<Id> {
        let id = new_id();
        tx.execute(
            "INSERT INTO category (id, parent_id, name, kind, is_essential, sort_order, seed_key, created_at, updated_at, spend_group)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9)",
            params![
                id,
                pid,
                c.name,
                kind.as_str(),
                c.essential,
                100 + sort as i64,
                c.key,
                now,
                c.group_str()
            ],
        )?;
        Ok(id)
    };
    for (kind, groups) in seed_plan(&crate::country::family_country(tx), i18n::current()) {
        for (i, g) in groups.iter().enumerate() {
            let rows = load_rows(tx)?;
            let (pid, parent_name) = match find_seed(&rows, kind, None, &g.cat) {
                Some(p) => (p.id.clone(), p.name.clone()),
                None => {
                    let any_child = g
                        .children
                        .iter()
                        .any(|c| picked(&format!("{}{PATH_SEP}{}", g.cat.name, c.name)));
                    if !picked(&g.cat.name) && !any_child {
                        continue;
                    }
                    n += 1;
                    (insert(None, &g.cat, kind, i)?, g.cat.name.clone())
                }
            };
            for (j, c) in g.children.iter().enumerate() {
                if find_seed(&rows, kind, Some(&pid), c).is_some()
                    || !picked(&format!("{parent_name}{PATH_SEP}{}", c.name))
                {
                    continue;
                }
                insert(Some(&pid), c, kind, j)?;
                n += 1;
            }
        }
    }
    Ok(n)
}

// ---------------------------------------------------------------------------
// Chave de origem e tradução das categorias padrão

/// Ajuste que marca os cofres de antes da V127 já reconhecidos (ver [`backfill_seed_keys`]).
const BACKFILL_SETTING: &str = "categories.seed_keys_backfilled";

/// Cofres de antes da V127: dá a chave de origem às categorias padrão, reconhecidas pelo tipo,
/// pelo lugar (raiz, ou dentro do grupo padrão) e pelo nome em pt-BR de qualquer país, sem
/// acentos nem maiúsculas, que é como o app as criou até aqui. Roda uma vez por cofre, fora do
/// histórico: a chave não muda nada do que a pessoa vê (o histórico tolera a coluna ao desfazer,
/// ver `changes`). Devolve quantas ganharam a chave.
pub fn backfill_seed_keys(conn: &Connection) -> Result<u32> {
    if crate::settings::get_or(conn, BACKFILL_SETTING, false)? {
        return Ok(0);
    }
    let tx = conn.unchecked_transaction()?;
    let rows = load_rows(&tx)?;
    let taken: HashSet<String> = rows.iter().filter_map(|r| r.seed_key.clone()).collect();
    let br_match = |key: &str, r: &Row| {
        r.seed_key.is_none()
            && seed_names_index().get(key).is_some_and(|v| {
                v.iter()
                    .any(|(l, s)| *l == Locale::PtBr && fold(s) == fold(&r.name))
            })
    };
    let mut assigned: HashMap<Id, String> = HashMap::new();
    for (kind, seeds) in TABLES {
        for (name, _, children) in seeds {
            let key = seed_key(None, name);
            let root = rows
                .iter()
                .find(|r| r.kind == kind && r.seed_key.as_deref() == Some(key.as_str()))
                .or_else(|| {
                    (!taken.contains(&key))
                        .then(|| {
                            rows.iter().find(|r| {
                                r.kind == kind
                                    && r.parent_id.is_none()
                                    && !assigned.contains_key(&r.id)
                                    && br_match(&key, r)
                            })
                        })
                        .flatten()
                });
            let Some(root) = root else { continue };
            if root.seed_key.is_none() {
                assigned.insert(root.id.clone(), key);
            }
            for c in children.iter() {
                let ck = seed_key(Some(name), c);
                if taken.contains(&ck) {
                    continue;
                }
                if let Some(r) = rows.iter().find(|r| {
                    r.parent_id.as_deref() == Some(root.id.as_str())
                        && !assigned.contains_key(&r.id)
                        && br_match(&ck, r)
                }) {
                    assigned.insert(r.id.clone(), ck);
                }
            }
        }
    }
    for (id, key) in &assigned {
        tx.execute(
            "UPDATE category SET seed_key = ?2 WHERE id = ?1 AND seed_key IS NULL",
            params![id, key],
        )?;
    }
    crate::settings::set(&tx, BACKFILL_SETTING, &true)?;
    tx.commit()?;
    Ok(assigned.len() as u32)
}

/// Ajuste que marca os cofres de antes da V129 já com o grupo das sementes (ver
/// [`backfill_spend_groups`]).
const GROUPS_BACKFILL_SETTING: &str = "categories.spend_groups_backfilled";

/// Cofres de antes da V129: a migração deu a cada despesa o grupo do "essencial" (essenciais ou
/// não essencial). Aqui cada categoria padrão (pela chave de origem; rode depois de
/// [`backfill_seed_keys`]) recebe o grupo da semente quando o "essencial" dela ainda é o padrão
/// (a pessoa não mexeu), e a subcategoria criada pela pessoa acompanha o pai que mudou quando
/// estava no mesmo grupo dele. Roda uma vez por cofre, fora do histórico, como a chave de origem.
/// O "essencial" não muda: as sementes que vão para dívidas eram essenciais e continuam (dívida
/// conta como essencial), e as que vão para futuro não eram. Num cofre novo não muda nada.
/// Devolve quantas mudaram.
pub fn backfill_spend_groups(conn: &Connection) -> Result<u32> {
    if crate::settings::get_or(conn, GROUPS_BACKFILL_SETTING, false)? {
        return Ok(0);
    }
    let tx = conn.unchecked_transaction()?;
    let rows = load_rows(&tx)?;
    let by_id: HashMap<&str, &Row> = rows.iter().map(|r| (r.id.as_str(), r)).collect();
    // Profundidade (à prova de ciclos), para os pais virem antes dos filhos.
    let depth = |r: &Row| {
        let mut seen = HashSet::from([r.id.as_str()]);
        let mut cur = r.parent_id.as_deref();
        let mut d = 0;
        while let Some(p) = cur.and_then(|p| by_id.get(p)) {
            if !seen.insert(p.id.as_str()) {
                break;
            }
            d += 1;
            cur = p.parent_id.as_deref();
        }
        d
    };
    let mut order: Vec<&Row> = rows
        .iter()
        .filter(|r| r.kind == CategoryKind::Expense)
        .collect();
    order.sort_by_key(|r| depth(r));
    let index = seed_groups_index();
    // Categoria que mudou → (grupo de antes, grupo novo).
    let mut changed: HashMap<&str, (SpendGroup, SpendGroup)> = HashMap::new();
    for r in order {
        let cur = r
            .spend_group
            .unwrap_or_else(|| SpendGroup::from_essential(r.is_essential));
        let target = match r.seed_key.as_deref().and_then(|k| index.get(k)) {
            Some((g, essential)) if r.is_essential == *essential => *g,
            Some(_) => cur,
            None => match r.parent_id.as_deref().and_then(|p| changed.get(p)) {
                Some((old, new)) if cur == *old => *new,
                _ => cur,
            },
        };
        if r.spend_group != Some(target) || r.is_essential != target.is_essential() {
            changed.insert(r.id.as_str(), (cur, target));
        }
    }
    for (id, (_, g)) in &changed {
        tx.execute(
            "UPDATE category SET spend_group = ?2, is_essential = ?3 WHERE id = ?1",
            params![id, g.as_str(), g.is_essential()],
        )?;
    }
    crate::settings::set(&tx, GROUPS_BACKFILL_SETTING, &true)?;
    tx.commit()?;
    Ok(changed.len() as u32)
}

/// Nomes que o idioma dá à semente em cada país (com a reserva do idioma).
fn language_names(l: Locale, key: &str) -> Vec<String> {
    // "ZZ": um país qualquer sem variante própria (os "outros").
    ["BR", "US", "PT", "ZZ"]
        .iter()
        .map(|c| seed_name(l, c, key))
        .collect()
}

/// Categorias padrão que ainda têm um nome padrão de outro idioma: (id, nome no idioma atual e
/// no país da família). A renomeada pela pessoa fica de fora, a que já tem um nome do idioma
/// atual também (mesmo que de outro país: trocar de país não renomeia nada, docs/countries.md),
/// e a que ficaria com o mesmo nome de outra no mesmo nível.
fn translation_plan(conn: &Connection) -> Result<Vec<(Id, String)>> {
    let rows = load_rows(conn)?;
    let country = crate::country::family_country(conn);
    let l = i18n::current();
    let mut siblings: HashMap<(Option<Id>, CategoryKind), HashSet<String>> = HashMap::new();
    for r in &rows {
        siblings
            .entry((r.parent_id.clone(), r.kind))
            .or_default()
            .insert(fold(&r.name));
    }
    let mut out = vec![];
    for r in &rows {
        let Some(key) = r.seed_key.as_deref() else {
            continue;
        };
        if !seed_names_index().contains_key(key)
            || !is_default_name(key, &r.name, false)
            || language_names(l, key).contains(&r.name)
        {
            continue;
        }
        let target = seed_name(l, &country, key);
        let (old, new) = (fold(&r.name), fold(&target));
        let level = siblings.entry((r.parent_id.clone(), r.kind)).or_default();
        if old != new {
            if level.contains(&new) {
                continue;
            }
            level.remove(&old);
            level.insert(new);
        }
        out.push((r.id.clone(), target));
    }
    Ok(out)
}

/// Quantas categorias padrão ainda estão com o nome padrão de outro idioma (as que
/// [`translate_defaults`] traduziria).
pub fn untranslated_defaults(conn: &Connection) -> Result<u32> {
    Ok(translation_plan(conn)?.len() as u32)
}

/// Passa para o idioma atual as categorias padrão que ainda têm um nome padrão de outro idioma
/// (as renomeadas pela pessoa ficam como estão). Devolve quantas mudou.
pub fn translate_defaults(tx: &Transaction<'_>) -> Result<u32> {
    let plan = translation_plan(tx)?;
    let now = now_utc();
    for (id, name) in &plan {
        tx.execute(
            "UPDATE category SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, name, now],
        )?;
    }
    Ok(plan.len() as u32)
}

// ---------------------------------------------------------------------------
// Categorias padrão pelo nome em pt-BR (buscas do núcleo)

/// Nome em pt-BR de uma categoria, para o núcleo achar as categorias padrão em qualquer idioma:
/// a categoria padrão com um nome padrão de outro idioma volta ao nome em pt-BR no país da
/// família; a que já está em pt-BR, a renomeada e a criada pela pessoa ficam com o nome que têm.
fn br_name(r: &Row, country: &str) -> String {
    match r.seed_key.as_deref() {
        Some(k) if !is_default_name(k, &r.name, true) && is_default_name(k, &r.name, false) => {
            seed_name(Locale::PtBr, country, k)
        }
        _ => r.name.clone(),
    }
}

/// Categoria com o nome e o caminho em pt-BR (ver [`br_categories`]).
#[derive(Debug, Clone)]
pub struct BrCategory {
    pub id: Id,
    pub kind: CategoryKind,
    pub archived: bool,
    pub name: String,
    /// Nome em pt-BR da categoria-pai.
    pub parent_name: Option<String>,
    /// Caminho com os nomes em pt-BR (`Alimentação › Mercado`).
    pub path: String,
    /// Chave da semente de onde saiu (categoria padrão).
    pub seed_key: Option<String>,
}

/// As categorias com os nomes em pt-BR: o que o núcleo usa para achar uma categoria padrão
/// pelo nome (regras prontas, DARF, proventos, câmbio, previdência, estimativa de impostos,
/// demonstração), qualquer que seja o idioma dos nomes no cofre.
pub fn br_categories(conn: &Connection) -> Result<Vec<BrCategory>> {
    let country = crate::country::family_country(conn);
    let rows: Vec<Row> = load_rows(conn)?
        .into_iter()
        .map(|mut r| {
            r.name = br_name(&r, &country);
            r
        })
        .collect();
    let paths = paths(&rows);
    let names: HashMap<&str, &str> = rows
        .iter()
        .map(|r| (r.id.as_str(), r.name.as_str()))
        .collect();
    Ok(rows
        .iter()
        .map(|r| BrCategory {
            id: r.id.clone(),
            kind: r.kind,
            archived: r.archived,
            name: r.name.clone(),
            parent_name: r
                .parent_id
                .as_deref()
                .and_then(|p| names.get(p))
                .map(|s| s.to_string()),
            path: paths.get(&r.id).cloned().unwrap_or_else(|| r.name.clone()),
            seed_key: r.seed_key.clone(),
        })
        .collect())
}

/// Chave da semente pelo caminho com os nomes-base (`Rendimentos › JCP`), se for uma.
fn base_key(kind: Option<CategoryKind>, path: &str) -> Option<String> {
    use crate::import::text::fold;
    let parts: Vec<String> = path.split(PATH_SEP).map(fold).collect();
    let (root, child) = match parts.as_slice() {
        [r] => (r, None),
        [r, c] => (r, Some(c)),
        _ => return None,
    };
    let (_, (name, _, children)) = TABLES
        .iter()
        .filter(|(k, _)| kind.is_none_or(|x| x == *k))
        .flat_map(|(k, seeds)| seeds.iter().map(move |s| (k, s)))
        .find(|(_, (name, _, _))| fold(name) == *root)?;
    match child {
        None => Some(seed_key(None, name)),
        Some(c) => children
            .iter()
            .find(|x| fold(x) == *c)
            .map(|x| seed_key(Some(name), x)),
    }
}

/// Nome em pt-BR de cada categoria (ver [`br_categories`]).
pub fn br_names(conn: &Connection) -> Result<HashMap<Id, String>> {
    Ok(br_categories(conn)?
        .into_iter()
        .map(|c| (c.id, c.name))
        .collect())
}

/// Subcategoria `pai › filha` pelos nomes em pt-BR (sem acentos, maiúsculas nem pontuação), do
/// tipo e fora das arquivadas, em qualquer idioma: a categoria padrão pela chave de origem (vale
/// também renomeada ou movida), as outras pelos nomes.
pub fn find_br_child(
    conn: &Connection,
    kind: CategoryKind,
    parent: &str,
    child: &str,
) -> Result<Option<Id>> {
    use crate::import::text::fold;
    let all = br_categories(conn)?;
    // A semente pela chave (mesmo renomeada ou movida); senão, pelos nomes.
    let key = base_key(Some(kind), &format!("{parent}{PATH_SEP}{child}"));
    if let Some(x) = all
        .iter()
        .find(|x| x.kind == kind && !x.archived && x.seed_key.is_some() && x.seed_key == key)
    {
        return Ok(Some(x.id.clone()));
    }
    let (p, c) = (fold(parent), fold(child));
    Ok(all
        .into_iter()
        .find(|x| {
            x.kind == kind
                && !x.archived
                && fold(&x.name) == c
                && x.parent_name.as_deref().is_some_and(|n| fold(n) == p)
        })
        .map(|x| x.id))
}

/// Categoria pelo caminho em pt-BR (`Impostos e taxas › Imposto de renda`, sem acentos,
/// maiúsculas nem pontuação), arquivada ou não, em qualquer idioma; só do tipo, se informado. A
/// categoria padrão vem pela chave de origem (vale também renomeada ou movida).
pub fn find_br_path(
    conn: &Connection,
    kind: Option<CategoryKind>,
    path: &str,
) -> Result<Option<Id>> {
    use crate::import::text::fold;
    let all = br_categories(conn)?;
    let key = base_key(kind, path);
    if let Some(x) = all
        .iter()
        .find(|x| kind.is_none_or(|k| x.kind == k) && x.seed_key.is_some() && x.seed_key == key)
    {
        return Ok(Some(x.id.clone()));
    }
    let want = fold(path);
    Ok(all
        .into_iter()
        .find(|x| kind.is_none_or(|k| x.kind == k) && fold(&x.path) == want)
        .map(|x| x.id))
}

/// Ids de uma categoria e de todos os descendentes.
pub fn with_descendants(conn: &rusqlite::Connection, ids: &[Id]) -> Result<Vec<Id>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "WITH RECURSIVE t(id) AS (SELECT id FROM category WHERE id IN ({placeholders})
           UNION SELECT c.id FROM category c JOIN t ON c.parent_id = t.id)
         SELECT id FROM t"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(ids.iter()), |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Leitura

#[derive(Debug, Clone)]
struct Row {
    id: Id,
    parent_id: Option<Id>,
    name: String,
    kind: CategoryKind,
    icon: Option<String>,
    color: Option<String>,
    /// Como está gravado (ver [`Row::essential`]).
    is_essential: bool,
    archived: bool,
    sort_order: i64,
    /// Chave da semente de onde a categoria saiu (`categorias-mercado`); `None`: criada pela
    /// pessoa (ou de cofre antigo ainda não reconhecido).
    seed_key: Option<String>,
    /// Grupo de gasto (V129); `None` nas receitas.
    spend_group: Option<SpendGroup>,
}

impl Row {
    /// Essencial: o grupo decide nas despesas (mesmo se um desfazer antigo mexeu só no
    /// `is_essential`); sem grupo, o que está gravado.
    fn essential(&self) -> bool {
        self.spend_group
            .map_or(self.is_essential, SpendGroup::is_essential)
    }
}

fn load_rows(conn: &Connection) -> Result<Vec<Row>> {
    let mut stmt = conn.prepare(
        "SELECT id, parent_id, name, kind, icon, color, is_essential, archived, sort_order, seed_key,
                spend_group
         FROM category ORDER BY sort_order, name COLLATE NOCASE, id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Row {
                id: r.get(0)?,
                parent_id: r.get(1)?,
                name: r.get(2)?,
                kind: CategoryKind::parse(&r.get::<_, String>(3)?),
                icon: r.get(4)?,
                color: r.get(5)?,
                is_essential: r.get(6)?,
                archived: r.get(7)?,
                sort_order: r.get(8)?,
                seed_key: r.get(9)?,
                spend_group: r
                    .get::<_, Option<String>>(10)?
                    .as_deref()
                    .and_then(SpendGroup::parse),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Caminho de cada categoria (`Pai › Filho`), à prova de ciclos.
fn paths(rows: &[Row]) -> HashMap<Id, String> {
    let by_id: HashMap<&str, &Row> = rows.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut out = HashMap::with_capacity(rows.len());
    for r in rows {
        let mut parts = vec![r.name.as_str()];
        let mut seen = HashSet::from([r.id.as_str()]);
        let mut cur = r.parent_id.as_deref();
        while let Some(pid) = cur {
            let Some(p) = by_id.get(pid) else { break };
            if !seen.insert(pid) {
                break;
            }
            parts.push(p.name.as_str());
            cur = p.parent_id.as_deref();
        }
        parts.reverse();
        out.insert(r.id.clone(), parts.join(PATH_SEP));
    }
    out
}

/// Caminho de todas as categorias (inclusive arquivadas), para montar lançamentos.
pub fn path_map(conn: &Connection) -> Result<HashMap<Id, String>> {
    Ok(paths(&load_rows(conn)?))
}

fn to_category(r: &Row, paths: &HashMap<Id, String>) -> Category {
    Category {
        id: r.id.clone(),
        parent_id: r.parent_id.clone(),
        name: r.name.clone(),
        kind: r.kind,
        icon: r.icon.clone(),
        color: r.color.clone(),
        is_essential: r.essential(),
        spend_group: r.spend_group,
        archived: r.archived,
        sort_order: r.sort_order,
        path: paths.get(&r.id).cloned().unwrap_or_else(|| r.name.clone()),
    }
}

/// Árvore ordenada (por `sort_order` e nome), filtrada.
pub fn tree(conn: &Connection, args: &CategoryListArgs) -> Result<Vec<CategoryNode>> {
    let rows = load_rows(conn)?;
    let paths = paths(&rows);
    let include_archived = args.include_archived.unwrap_or(false);
    let ids: HashSet<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let mut children: HashMap<Option<&str>, Vec<&Row>> = HashMap::new();
    for r in &rows {
        // Pai inexistente (não deveria acontecer): trata como raiz.
        let parent = r.parent_id.as_deref().filter(|p| ids.contains(p));
        children.entry(parent).or_default().push(r);
    }
    fn build(
        parent: Option<&str>,
        children: &HashMap<Option<&str>, Vec<&Row>>,
        paths: &HashMap<Id, String>,
        args: &CategoryListArgs,
        include_archived: bool,
        seen: &mut HashSet<String>,
    ) -> Vec<CategoryNode> {
        let mut out = Vec::new();
        for r in children.get(&parent).map(Vec::as_slice).unwrap_or_default() {
            if (!include_archived && r.archived)
                || args.kind.is_some_and(|k| k != r.kind)
                || !seen.insert(r.id.clone())
            {
                continue;
            }
            out.push(CategoryNode {
                category: to_category(r, paths),
                children: build(
                    Some(r.id.as_str()),
                    children,
                    paths,
                    args,
                    include_archived,
                    seen,
                ),
            });
        }
        out
    }
    let mut seen = HashSet::new();
    let mut roots = build(None, &children, &paths, args, include_archived, &mut seen);
    // Receitas depois das despesas, como no conjunto inicial.
    roots.sort_by_key(|n| n.category.kind == CategoryKind::Income);
    Ok(roots)
}

/// Lista plana na ordem da árvore (pais antes dos filhos), com `path`.
pub fn list(conn: &Connection, args: &CategoryListArgs) -> Result<Vec<Category>> {
    fn flatten(nodes: Vec<CategoryNode>, out: &mut Vec<Category>) {
        for n in nodes {
            out.push(n.category);
            flatten(n.children, out);
        }
    }
    let mut out = Vec::new();
    flatten(tree(conn, args)?, &mut out);
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> Result<Category> {
    let rows = load_rows(conn)?;
    let paths = paths(&rows);
    rows.iter()
        .find(|r| r.id == id)
        .map(|r| to_category(r, &paths))
        .ok_or_else(|| Error::not_found(category_ref(id)))
}

/// "categoria {id}" (detalhe do erro "não encontrado").
pub(crate) fn category_ref(id: &str) -> String {
    i18n::t_args(
        "nucleo-lancamentos-ref-categoria",
        &i18n::args(&[("id", id.to_string().into())]),
    )
}

/// Tipo e estado de arquivamento da categoria.
pub fn kind_of(conn: &Connection, id: &str) -> Result<(CategoryKind, bool, String)> {
    let (kind, archived, name): (String, bool, String) = conn
        .query_row(
            "SELECT kind, archived, name FROM category WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(category_ref(id)))?;
    Ok((CategoryKind::parse(&kind), archived, name))
}

pub fn usage(conn: &Connection, id: &str) -> Result<CategoryUsage> {
    let count = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [id], |r| r.get(0))?) };
    Ok(CategoryUsage {
        transactions: count("SELECT count(*) FROM txn WHERE category_id = ?1")?,
        splits: count("SELECT count(*) FROM txn_split WHERE category_id = ?1")?,
        budget_lines: count("SELECT count(*) FROM budget_line WHERE category_id = ?1")?,
        rules: count(
            "SELECT count(*) FROM rule WHERE json_valid(actions) AND json_extract(actions, '$.category_id') = ?1",
        )?,
        recurrences: count("SELECT count(*) FROM recurrence WHERE category_id = ?1")?,
        installment_plans: count("SELECT count(*) FROM installment_plan WHERE category_id = ?1")?,
        children: count("SELECT count(*) FROM category WHERE parent_id = ?1")?,
    })
}

// ---------------------------------------------------------------------------
// Escrita

/// Nome comparável: minúsculas e sem acentos.
fn fold(s: &str) -> String {
    s.chars()
        .map(crate::transactions::fold_accent)
        .collect::<String>()
        .to_lowercase()
}

fn ensure_unique_sibling(
    conn: &Connection,
    parent_id: Option<&str>,
    kind: CategoryKind,
    name: &str,
    except: Option<&str>,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, name FROM category WHERE parent_id IS ?1 AND kind = ?2 AND (?3 IS NULL OR id <> ?3)",
    )?;
    let target = fold(name);
    let clash = stmt
        .query_map(params![parent_id, kind.as_str(), except], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .any(|(_, n)| fold(&n) == target);
    if clash {
        return Err(Error::conflict(i18n::t_args(
            "nucleo-lancamentos-categoria-erro-repetida",
            &i18n::args(&[("nome", name.to_string().into())]),
        )));
    }
    Ok(())
}

/// Tipo para os seletores do catálogo (`[receita]`, `[despesa]`).
fn kind_arg(k: CategoryKind) -> &'static str {
    match k {
        CategoryKind::Income => "receita",
        CategoryKind::Expense => "despesa",
    }
}

fn check_parent(conn: &Connection, parent_id: &str, kind: CategoryKind) -> Result<()> {
    let (pkind, _, pname) = kind_of(conn, parent_id).map_err(|_| {
        Error::not_found(i18n::t_args(
            "nucleo-lancamentos-ref-categoria-pai",
            &i18n::args(&[("id", parent_id.to_string().into())]),
        ))
    })?;
    if pkind != kind {
        return Err(Error::invalid(i18n::t_args(
            "nucleo-lancamentos-categoria-erro-tipo-pai",
            &i18n::args(&[("pai", pname.into()), ("tipo", kind_arg(pkind).into())]),
        )));
    }
    Ok(())
}

pub fn create(tx: &Transaction<'_>, input: &CategoryInput) -> Result<Category> {
    let name = clean_name(&input.name, "da categoria")?;
    let parent = clean_opt(input.parent_id.clone());
    if let Some(p) = &parent {
        check_parent(tx, p, input.kind)?;
    }
    ensure_unique_sibling(tx, parent.as_deref(), input.kind, &name, None)?;
    let sort: i64 = tx.query_row(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM category WHERE parent_id IS ?1 AND kind = ?2",
        params![parent, input.kind.as_str()],
        |r| r.get(0),
    )?;
    no_group_on_income(input.kind, input.spend_group)?;
    let group = match input.kind {
        CategoryKind::Income => None,
        CategoryKind::Expense => Some(match input.spend_group {
            Some(g) => g,
            // Subcategoria herda o grupo do pai (sem pai: não essencial), ajustado pelo
            // "essencial" informado.
            None => {
                let from_parent = match &parent {
                    Some(p) => load_rows(tx)?.iter().find(|r| r.id == *p).map(|r| {
                        r.spend_group
                            .unwrap_or_else(|| SpendGroup::from_essential(r.essential()))
                    }),
                    None => None,
                };
                patched_group(
                    from_parent.unwrap_or(SpendGroup::NonEssential),
                    input.is_essential,
                )
            }
        }),
    };
    let id = new_id();
    tx.execute(
        "INSERT INTO category (id, parent_id, name, kind, icon, color, is_essential, sort_order, created_at, updated_at, spend_group)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, ?10)",
        params![
            id,
            parent,
            name,
            input.kind.as_str(),
            clean_opt(input.icon.clone()),
            clean_color(input.color.clone())?,
            group.is_some_and(SpendGroup::is_essential),
            sort,
            now_utc(),
            group.map(SpendGroup::as_str)
        ],
    )?;
    get(tx, &id)
}

/// Categoria de receita não tem grupo de gasto.
fn no_group_on_income(kind: CategoryKind, group: Option<SpendGroup>) -> Result<()> {
    if kind == CategoryKind::Income && group.is_some() {
        return Err(Error::invalid(i18n::t(
            "nucleo-lancamentos-categoria-erro-grupo-receita",
        )));
    }
    Ok(())
}

/// Grupo de gasto de quem não informou o grupo, só (talvez) o "essencial": marcar essencial
/// num grupo não essencial ou futuro leva a "essenciais" (dívidas continuam dívidas);
/// desmarcar em essenciais ou dívidas leva a "não essencial"; sem o "essencial", fica `cur`.
fn patched_group(cur: SpendGroup, essential: Option<bool>) -> SpendGroup {
    match essential {
        Some(true) if !cur.is_essential() => SpendGroup::Essential,
        Some(false) if cur.is_essential() => SpendGroup::NonEssential,
        _ => cur,
    }
}

pub fn update(tx: &Transaction<'_>, patch: &CategoryPatch) -> Result<Category> {
    let cur = get(tx, &patch.id)?;
    let name = match &patch.name {
        Some(n) => clean_name(n, "da categoria")?,
        None => cur.name.clone(),
    };
    let parent = match &patch.parent_id {
        Some(p) => clean_opt(p.clone()),
        None => cur.parent_id.clone(),
    };
    if parent != cur.parent_id
        && let Some(p) = &parent
    {
        if *p == cur.id || with_descendants(tx, std::slice::from_ref(&cur.id))?.contains(p) {
            return Err(Error::invalid(i18n::t(
                "nucleo-lancamentos-categoria-erro-ciclo",
            )));
        }
        check_parent(tx, p, cur.kind)?;
    }
    if parent != cur.parent_id || fold(&name) != fold(&cur.name) {
        ensure_unique_sibling(tx, parent.as_deref(), cur.kind, &name, Some(&cur.id))?;
    }
    let icon = match &patch.icon {
        Some(i) => clean_opt(i.clone()),
        None => cur.icon.clone(),
    };
    let color = match &patch.color {
        Some(c) => clean_color(c.clone())?,
        None => cur.color.clone(),
    };
    no_group_on_income(cur.kind, patch.spend_group)?;
    // O grupo vence; `is_essential` fica verdadeiro em essenciais e dívidas (receitas: sem
    // grupo, não essencial).
    let group = match cur.kind {
        CategoryKind::Income => None,
        CategoryKind::Expense => Some(patch.spend_group.unwrap_or_else(|| {
            patched_group(
                cur.spend_group
                    .unwrap_or_else(|| SpendGroup::from_essential(cur.is_essential)),
                patch.is_essential,
            )
        })),
    };
    tx.execute(
        "UPDATE category SET parent_id = ?2, name = ?3, icon = ?4, color = ?5, is_essential = ?6, archived = ?7,
                sort_order = ?8, updated_at = ?9, spend_group = ?10
         WHERE id = ?1 AND (parent_id IS NOT ?2 OR name IS NOT ?3 OR icon IS NOT ?4 OR color IS NOT ?5
                            OR is_essential IS NOT ?6 OR archived IS NOT ?7 OR sort_order IS NOT ?8
                            OR spend_group IS NOT ?10)",
        params![
            cur.id,
            parent,
            name,
            icon,
            color,
            group.is_some_and(SpendGroup::is_essential),
            patch.archived.unwrap_or(cur.archived),
            patch.sort_order.unwrap_or(cur.sort_order),
            now_utc(),
            group.map(SpendGroup::as_str)
        ],
    )?;
    get(tx, &cur.id)
}

/// O mesmo grupo de gasto para várias categorias de despesa, com `is_essential` junto. As de
/// receita da lista ficam como estão; id desconhecido é erro. Devolve quantas mudaram.
pub fn set_group(tx: &Transaction<'_>, ids: &[Id], group: SpendGroup) -> Result<u32> {
    let now = now_utc();
    let mut seen = HashSet::new();
    let mut n = 0;
    for id in ids {
        if !seen.insert(id.as_str()) {
            continue;
        }
        if kind_of(tx, id)?.0 == CategoryKind::Income {
            continue;
        }
        n += tx.execute(
            "UPDATE category SET spend_group = ?2, is_essential = ?3, updated_at = ?4
             WHERE id = ?1 AND (spend_group IS NOT ?2 OR is_essential IS NOT ?3)",
            params![id, group.as_str(), group.is_essential(), now],
        )?;
    }
    Ok(n as u32)
}

/// Move todo o uso de `from` para `into` (lançamentos, divisões, orçamento, regras,
/// recorrências e parcelamentos). Linhas de orçamento do mesmo mês são somadas.
fn reassign(tx: &Transaction<'_>, from: &str, into: &str) -> Result<CategoryUsage> {
    let now = now_utc();
    let mut moved = CategoryUsage {
        transactions: tx.execute(
            "UPDATE txn SET category_id = ?2, updated_at = ?3 WHERE category_id = ?1",
            params![from, into, now],
        )? as i64,
        splits: tx.execute(
            "UPDATE txn_split SET category_id = ?2, updated_at = ?3 WHERE category_id = ?1",
            params![from, into, now],
        )? as i64,
        ..Default::default()
    };
    // Orçamento: soma no destino quando o mesmo mês já tem linha para ele.
    let merged = tx.execute(
        "UPDATE budget_line AS t
            SET amount = t.amount + (SELECT f.amount FROM budget_line f WHERE f.budget_id = t.budget_id AND f.category_id = ?1),
                updated_at = ?3
          WHERE t.category_id = ?2
            AND EXISTS (SELECT 1 FROM budget_line f WHERE f.budget_id = t.budget_id AND f.category_id = ?1)",
        params![from, into, now],
    )?;
    tx.execute(
        "DELETE FROM budget_line WHERE category_id = ?1
            AND budget_id IN (SELECT budget_id FROM budget_line WHERE category_id = ?2)",
        params![from, into],
    )?;
    moved.budget_lines = merged as i64
        + tx.execute(
            "UPDATE budget_line SET category_id = ?2, updated_at = ?3 WHERE category_id = ?1",
            params![from, into, now],
        )? as i64;
    moved.rules = tx.execute(
        "UPDATE rule SET actions = json_set(actions, '$.category_id', ?2), updated_at = ?3
          WHERE json_valid(actions) AND json_extract(actions, '$.category_id') = ?1",
        params![from, into, now],
    )? as i64;
    moved.recurrences = tx.execute(
        "UPDATE recurrence SET category_id = ?2, updated_at = ?3 WHERE category_id = ?1",
        params![from, into, now],
    )? as i64;
    moved.installment_plans = tx.execute(
        "UPDATE installment_plan SET category_id = ?2, updated_at = ?3 WHERE category_id = ?1",
        params![from, into, now],
    )? as i64;
    Ok(moved)
}

fn check_target(conn: &Connection, from: &Category, into: &str) -> Result<Category> {
    if into == from.id {
        return Err(Error::invalid(i18n::t(
            "nucleo-lancamentos-categoria-erro-mesmo-destino",
        )));
    }
    let target = get(conn, into)?;
    if target.kind != from.kind {
        return Err(Error::invalid(i18n::t_args(
            "nucleo-lancamentos-categoria-erro-juntar-tipos",
            &i18n::args(&[
                ("origem", from.name.clone().into()),
                ("tipo_origem", kind_arg(from.kind).into()),
                ("destino", target.name.clone().into()),
                ("tipo_destino", kind_arg(target.kind).into()),
            ]),
        )));
    }
    if with_descendants(conn, std::slice::from_ref(&from.id))?.contains(&target.id) {
        return Err(Error::invalid(i18n::t_args(
            "nucleo-lancamentos-categoria-erro-juntar-sub",
            &i18n::args(&[
                ("destino", target.path.clone().into()),
                ("origem", from.name.clone().into()),
            ]),
        )));
    }
    Ok(target)
}

/// Apaga uma categoria. Com `reassign_to`, o uso passa para a outra categoria; sem ela,
/// só apaga se não estiver em uso. Categorias com subcategorias não podem ser apagadas
/// (use `merge` ou mova as subcategorias antes).
pub fn delete(tx: &Transaction<'_>, id: &str, reassign_to: Option<&str>) -> Result<CategoryUsage> {
    let cat = get(tx, id)?;
    let use_ = usage(tx, id)?;
    if use_.children > 0 {
        return Err(Error::conflict(i18n::t_args(
            "nucleo-lancamentos-categoria-erro-tem-subcategorias",
            &i18n::args(&[
                ("nome", cat.name.clone().into()),
                ("n", use_.children.into()),
            ]),
        )));
    }
    let moved = match reassign_to {
        Some(into) => {
            check_target(tx, &cat, into)?;
            reassign(tx, id, into)?
        }
        None => {
            if use_.in_use() {
                return Err(Error::conflict(i18n::t_args(
                    "nucleo-lancamentos-categoria-erro-em-uso",
                    &i18n::args(&[
                        ("nome", cat.name.clone().into()),
                        ("lancamentos", use_.transactions.into()),
                        ("divisoes", use_.splits.into()),
                        ("orcamento", use_.budget_lines.into()),
                        ("regras", use_.rules.into()),
                        ("recorrencias", use_.recurrences.into()),
                        ("parcelamentos", use_.installment_plans.into()),
                    ]),
                )));
            }
            CategoryUsage::default()
        }
    };
    tx.execute("DELETE FROM category WHERE id = ?1", [id])?;
    Ok(moved)
}

/// Junta `from` em `into`: reatribui todo o uso, move as subcategorias (juntando as de
/// mesmo nome) e apaga `from`.
pub fn merge(tx: &Transaction<'_>, from: &str, into: &str) -> Result<Category> {
    let src = get(tx, from)?;
    check_target(tx, &src, into)?;
    merge_inner(tx, &src.id, into)?;
    get(tx, into)
}

fn merge_inner(tx: &Transaction<'_>, from: &str, into: &str) -> Result<()> {
    let children = |parent: &str| -> Result<Vec<(Id, String)>> {
        let mut stmt = tx.prepare("SELECT id, name FROM category WHERE parent_id = ?1")?;
        let rows = stmt
            .query_map([parent], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    };
    let target_children = children(into)?;
    let now = now_utc();
    for (cid, cname) in children(from)? {
        match target_children
            .iter()
            .find(|(_, n)| fold(n) == fold(&cname))
        {
            Some((tid, _)) => merge_inner(tx, &cid, tid)?,
            None => {
                tx.execute(
                    "UPDATE category SET parent_id = ?2, updated_at = ?3 WHERE id = ?1",
                    params![cid, into, now],
                )?;
            }
        }
    }
    reassign(tx, from, into)?;
    tx.execute("DELETE FROM category WHERE id = ?1", [from])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::{Author, record};
    use crate::storage::Db;
    use crate::vault::KDF_TEST;

    #[test]
    fn seeds_tree_and_descendants() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", seed_defaults).unwrap();
        let n: i64 = db
            .conn()
            .query_row("SELECT count(*) FROM category", [], |r| r.get(0))
            .unwrap();
        assert!(n > 60);
        let moradia: String = db
            .conn()
            .query_row(
                "SELECT id FROM category WHERE name = 'Moradia' AND parent_id IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let all = with_descendants(db.conn(), &[moradia]).unwrap();
        assert_eq!(all.len(), 10);
    }

    #[test]
    fn missing_defaults_are_offered_and_added_without_touching_existing() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", seed_defaults).unwrap();
        assert!(missing_defaults(db.conn()).unwrap().is_empty());
        // Cofre antigo: sem "Filhos › Fraldas e higiene" e com "Filhos" renomeado à mão.
        record(db.conn_mut(), &Author::system("t"), "x", |tx| {
            tx.execute("DELETE FROM category WHERE name = 'Fraldas e higiene'", [])?;
            tx.execute("DELETE FROM category WHERE name = 'Poupança dos filhos'", [])?;
            tx.execute(
                "DELETE FROM category WHERE parent_id = (SELECT id FROM category WHERE name = 'Poupança e investimentos')",
                [],
            )?;
            tx.execute("DELETE FROM category WHERE name = 'Poupança e investimentos'", [])?;
            tx.execute(
                "UPDATE category SET name = 'Crianças' WHERE name = 'Mesada' AND kind = 'expense'",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        let miss = missing_defaults(db.conn()).unwrap();
        let paths: Vec<&str> = miss.iter().map(|m| m.path.as_str()).collect();
        assert!(paths.contains(&"Filhos › Fraldas e higiene"), "{paths:?}");
        assert!(paths.contains(&"Poupança e investimentos"));
        assert!(paths.contains(&"Poupança e investimentos › Poupança dos filhos"));
        let n = record(db.conn_mut(), &Author::system("t"), "add", |tx| {
            add_missing_defaults(tx, None)
        })
        .unwrap()
        .value;
        assert_eq!(n as usize, miss.len());
        assert!(missing_defaults(db.conn()).unwrap().is_empty());
        // O renomeado continua lá (e "Mesada" volta como sugestão adicionada).
        let renamed: i64 = db
            .conn()
            .query_row(
                "SELECT count(*) FROM category WHERE name = 'Crianças'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(renamed, 1);
    }

    #[test]
    fn adds_only_the_picked_suggestions() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
        record(db.conn_mut(), &Author::system("t"), "seed", seed_defaults).unwrap();
        // Cofre de antes das receitas novas: sem "Venda de bens", "Trabalho por aplicativo"
        // nem "Salário › Horas extras".
        record(db.conn_mut(), &Author::system("t"), "x", |tx| {
            for root in ["Venda de bens", "Trabalho por aplicativo"] {
                tx.execute(
                    "DELETE FROM category WHERE parent_id = (SELECT id FROM category WHERE name = ?1)",
                    [root],
                )?;
                tx.execute("DELETE FROM category WHERE name = ?1", [root])?;
            }
            tx.execute("DELETE FROM category WHERE name = 'Horas extras'", [])?;
            Ok(())
        })
        .unwrap();
        let miss = missing_defaults(db.conn()).unwrap();
        assert!(miss.iter().all(|m| m.kind == CategoryKind::Income));
        assert_eq!(miss.len(), 2 + 4 + 3 + 1, "{miss:?}");
        // Escolhe uma subcategoria de grupo novo (o grupo vem junto) e a de grupo existente,
        // com outra grafia.
        let only = vec![
            "Venda de bens › Veículos".to_string(),
            "salario › horas extras".to_string(),
        ];
        let n = record(db.conn_mut(), &Author::system("t"), "add", |tx| {
            add_missing_defaults(tx, Some(&only))
        })
        .unwrap()
        .value;
        assert_eq!(n, 3);
        let left: Vec<String> = missing_defaults(db.conn())
            .unwrap()
            .into_iter()
            .map(|m| m.path)
            .collect();
        assert_eq!(left.len(), 10 - 3, "{left:?}");
        assert!(left.contains(&"Venda de bens › Imóveis".to_string()));
        assert!(left.contains(&"Trabalho por aplicativo".to_string()));
        assert!(!left.iter().any(|p| p.contains("Horas extras")));
    }
}

#[cfg(test)]
mod tests_domain;

#[cfg(test)]
mod tests_locale;
