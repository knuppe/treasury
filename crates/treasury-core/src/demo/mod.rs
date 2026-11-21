//! Cofre de demonstração (docs/demo.md): a partir do que a pessoa conta no assistente
//! (família, renda, despesas, contas, investimentos e dívidas), gera um histórico de exemplo
//! para testar o app sem dados reais. Sempre num cofre novo, separado (`vaults.new_demo`),
//! marcado como demonstração (`demo_vault` nos ajustes do cofre).
//!
//! O histórico é determinístico (mesma entrada, mesmos dados) e entra num único conjunto de
//! alterações, pelos mesmos caminhos da interface: contas, lançamentos com titular, cartão com
//! faturas pagas no vencimento, aportes, operações e proventos na carteira, previdência com o
//! plano, contratos de dívida com as parcelas ligadas aos pagamentos, recorrências para os
//! próximos meses, orçamento do mês e a meta de reserva de emergência.
//!
//! As cotações da carteira são fictícias e ficam como cotação manual, para o exemplo não
//! misturar preços reais com operações inventadas.

pub mod api;
mod daily;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_invariants;
mod texts;

use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Transaction, params};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::household::{MemberPatch, MemberProfile, MemberRelationship, MemberSex};
use crate::i18n::{args, t, t_args};
use crate::ids::Id;
use crate::profile::Level;
use crate::time::{add_months, fmt_date, month_bounds};

use texts::Texts;

/// Marca do cofre de demonstração nos ajustes do cofre.
pub const SETTING: &str = "demo_vault";

/// Chave fixa dos cofres de demonstração sem senha (`vaults.new_demo` sem `password`). Não é
/// segredo: o cofre continua cifrado como os outros, mas qualquer um com o app abre, por isso
/// só serve para dados fictícios.
pub const DEMO_PASSWORD: &str = "treasury-demonstracao-sem-senha-dados-ficticios";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DemoAdult {
    pub name: String,
    /// Renda líquida mensal, em centavos.
    #[ts(type = "number")]
    pub income: i64,
    /// Renda variável (autônomo, comissões): muda de um mês para o outro, sem 13º.
    #[serde(default)]
    pub variable: bool,
    /// Aposentado: a renda é a aposentadoria (no Brasil, o INSS, com 13º).
    #[serde(default)]
    #[ts(optional)]
    pub retired: Option<bool>,
    /// Idade hoje (a data de nascimento sai dela); sem ela, uma idade que combina com o perfil
    /// (aposentado de 64 a 71 anos, pais compatíveis com a idade dos filhos…).
    #[serde(default)]
    #[ts(optional)]
    pub age: Option<u32>,
    /// Sexo do cadastro (linha da vida e INSS); sem ele, fica não informado.
    #[serde(default)]
    #[ts(optional)]
    pub sex: Option<MemberSex>,
}

/// Idade e sexo de um filho, na ordem de `children`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DemoChild {
    #[serde(default)]
    #[ts(optional)]
    pub age: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub sex: Option<MemberSex>,
}

/// Perfil de exemplo escolhido no assistente (só muda o rótulo do cofre; os valores vêm no
/// próprio `DemoProfile`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DemoKind {
    /// Família de classe média com folga.
    Comfortable,
    /// Família de baixa renda.
    LowIncome,
    /// Família em dificuldade: dívidas, saldo negativo e contas atrasadas.
    Struggling,
    /// Uma pessoa só, jovem.
    Single,
    /// Aposentado ou aposentada.
    Retiree,
    /// Montado à mão no assistente.
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DemoHousing {
    /// Aluguel.
    Rent,
    /// Financiamento imobiliário (vira um contrato de dívida).
    Mortgage,
    /// Casa própria quitada.
    Own,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DemoTransport {
    Car,
    App,
    Public,
    None,
}

/// O que o assistente pergunta. Valores mensais em centavos; zero = não tem.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DemoProfile {
    /// Meses de histórico (3 a 36).
    pub months: u32,
    /// Um ou dois adultos com renda.
    pub adults: Vec<DemoAdult>,
    #[serde(default)]
    pub children: Vec<String>,
    /// Idade e sexo de cada filho, na ordem de `children` (o que faltar sai do perfil).
    #[serde(default)]
    #[ts(optional)]
    pub child_details: Option<Vec<DemoChild>>,
    /// Banco da conta principal (nome livre).
    pub bank: String,
    /// Limite do cartão de crédito; sem ele, sem cartão.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub card_limit: Option<i64>,
    /// Outras rendas por mês (aluguel recebido, por exemplo).
    #[serde(default)]
    #[ts(type = "number")]
    pub other_income: i64,
    pub housing: DemoHousing,
    /// Aluguel ou parcela do financiamento.
    #[serde(default)]
    #[ts(type = "number")]
    pub housing_amount: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub condo: i64,
    /// Luz, água, internet e celular.
    #[serde(default)]
    #[ts(type = "number")]
    pub utilities: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub groceries: i64,
    pub transport: DemoTransport,
    #[serde(default)]
    #[ts(type = "number")]
    pub transport_amount: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub school_per_child: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub health: i64,
    /// Restaurantes, delivery, passeios e cinema.
    #[serde(default)]
    #[ts(type = "number")]
    pub leisure: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub subscriptions: i64,
    /// Roupas, casa e eletrônicos.
    #[serde(default)]
    #[ts(type = "number")]
    pub shopping: i64,
    /// Reserva de emergência hoje e quanto vai para ela por mês.
    #[serde(default)]
    #[ts(type = "number")]
    pub reserve: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub reserve_monthly: i64,
    /// Carteira no início do histórico, por classe.
    #[serde(default)]
    #[ts(type = "number")]
    pub stocks: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub fiis: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub fixed_income: i64,
    /// Aporte mensal na corretora.
    #[serde(default)]
    #[ts(type = "number")]
    pub invest_monthly: i64,
    /// Previdência (VGBL): saldo no início e aporte mensal.
    #[serde(default)]
    #[ts(type = "number")]
    pub pension: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub pension_monthly: i64,
    /// Financiamento do carro: parcela e quantas faltam hoje.
    #[serde(default)]
    #[ts(type = "number")]
    pub car_installment: i64,
    #[serde(default)]
    pub car_installments_left: u32,
    /// Empréstimo pessoal: parcela e quantas faltam hoje.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub personal_loan: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub personal_loan_left: Option<u32>,
    /// Benefício social por mês (no Brasil, o Bolsa Família).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub benefit: Option<i64>,
    /// Limite do cheque especial da conta principal; com saldo negativo no fim do mês, cobra
    /// juros no começo do seguinte.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub overdraft_limit: Option<i64>,
    /// Algumas contas da casa pagas com atraso (multa e juros).
    #[serde(default)]
    #[ts(optional)]
    pub late_bills: Option<bool>,
    /// Paga só parte da fatura do cartão (o resto fica no rotativo, com juros).
    #[serde(default)]
    #[ts(optional)]
    pub card_revolving: Option<bool>,
    /// Holerites dos últimos meses para as rendas fixas (padrão: sim).
    #[serde(default)]
    #[ts(optional)]
    pub payslips: Option<bool>,
    /// País da família (ISO, ex.: "BR", "US", "PT"); padrão: o do app.
    #[serde(default)]
    #[ts(optional)]
    pub country: Option<String>,
    /// Perfil de exemplo escolhido no assistente.
    #[serde(default)]
    #[ts(optional)]
    pub kind: Option<DemoKind>,
    /// Nível da interface no cofre de demonstração (padrão: avançado).
    #[serde(default)]
    #[ts(optional)]
    pub level: Option<Level>,
    /// Semente do sorteio (padrão: tirada dos nomes).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub seed: Option<u64>,
}

/// O que foi gerado.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DemoSummary {
    pub from: String,
    pub to: String,
    pub accounts: u32,
    pub transactions: u32,
    pub assets: u32,
    pub loans: u32,
}

/// 100 milhões (na moeda da família), em centavos.
const MAX_AMOUNT: i64 = 10_000_000_000;

/// Para que serve o cofre (na lista de cofres), com o perfil escolhido no assistente.
pub fn purpose(kind: Option<DemoKind>) -> String {
    let l = crate::i18n::current();
    let key = match kind {
        Some(DemoKind::Comfortable) => "demodados-perfil-confortavel",
        Some(DemoKind::LowIncome) => "demodados-perfil-baixa-renda",
        Some(DemoKind::Struggling) => "demodados-perfil-dificuldade",
        Some(DemoKind::Single) => "demodados-perfil-sozinho",
        Some(DemoKind::Retiree) => "demodados-perfil-aposentado",
        Some(DemoKind::Custom) | None => {
            return crate::i18n::tr(l, "demodados-proposito", None);
        }
    };
    let perfil = crate::i18n::tr(l, key, None);
    texts::Texts::new("BR").ta("demodados-proposito-perfil", &[("perfil", perfil)])
}

/// Confere a entrada antes de criar o cofre.
pub fn validate(p: &DemoProfile) -> Result<()> {
    if !(3..=36).contains(&p.months) {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-meses")));
    }
    if p.adults.is_empty() || p.adults.len() > 2 {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-adultos")));
    }
    if p.children.len() > 4 {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-filhos")));
    }
    let mut names: Vec<String> = vec![];
    for n in p
        .adults
        .iter()
        .map(|a| a.name.as_str())
        .chain(p.children.iter().map(String::as_str))
    {
        let n = n.trim();
        if n.is_empty() {
            return Err(Error::invalid(t("nucleo-geral-erro-demo-nome")));
        }
        if names.iter().any(|x| x.eq_ignore_ascii_case(n)) {
            return Err(Error::invalid(t_args(
                "nucleo-geral-erro-demo-nome-repetido",
                &args(&[("nome", n.to_string().into())]),
            )));
        }
        names.push(n.to_string());
    }
    if p.adults.iter().all(|a| a.income <= 0) {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-renda")));
    }
    if p.bank.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-banco")));
    }
    let amounts = [
        p.card_limit.unwrap_or(0),
        p.other_income,
        p.housing_amount,
        p.condo,
        p.utilities,
        p.groceries,
        p.transport_amount,
        p.school_per_child,
        p.health,
        p.leisure,
        p.subscriptions,
        p.shopping,
        p.reserve,
        p.reserve_monthly,
        p.stocks,
        p.fiis,
        p.fixed_income,
        p.invest_monthly,
        p.pension,
        p.pension_monthly,
        p.car_installment,
        p.personal_loan.unwrap_or(0),
        p.benefit.unwrap_or(0),
        p.overdraft_limit.unwrap_or(0),
    ];
    if amounts
        .iter()
        .chain(p.adults.iter().map(|a| &a.income))
        .any(|v| *v < 0 || *v > MAX_AMOUNT)
    {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-valores")));
    }
    if p.housing == DemoHousing::Mortgage && p.housing_amount <= 0 {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-financiamento")));
    }
    if p.car_installment > 0 && !(1..=96).contains(&p.car_installments_left) {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-parcelas-carro")));
    }
    if p.personal_loan.unwrap_or(0) > 0 && !(1..=96).contains(&p.personal_loan_left.unwrap_or(0)) {
        return Err(Error::invalid(
            "informe quantas parcelas do empréstimo faltam (1 a 96)",
        ));
    }
    if let Some(c) = &p.country
        && (c.len() != 2 || !c.chars().all(|ch| ch.is_ascii_alphabetic()))
    {
        return Err(Error::invalid("país inválido"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Sorteio determinístico (SplitMix64)

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniforme em [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Uniforme em [a, b].
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.unit()
    }
    fn int(&mut self, a: u32, b: u32) -> u32 {
        a + (self.next() % u64::from(b - a + 1)) as u32
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[(self.next() % v.len() as u64) as usize]
    }
    /// `v` com variação de ±`pct`, arredondado ao centavo.
    fn vary(&mut self, v: i64, pct: f64) -> i64 {
        (v as f64 * self.range(1.0 - pct, 1.0 + pct)).round() as i64
    }
}

fn seed_from(p: &DemoProfile) -> u64 {
    p.seed.unwrap_or_else(|| {
        p.adults
            .iter()
            .flat_map(|a| a.name.bytes())
            .chain(p.bank.bytes())
            .fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
            })
    })
}

/// Divide `total` em `n` partes com pesos sorteados (a soma é exata).
fn split(rng: &mut Rng, total: i64, n: usize) -> Vec<i64> {
    if n == 0 || total <= 0 {
        return vec![];
    }
    let w: Vec<f64> = (0..n).map(|_| rng.range(0.6, 1.4)).collect();
    let sum: f64 = w.iter().sum();
    let mut out: Vec<i64> = w
        .iter()
        .map(|x| (total as f64 * x / sum).round() as i64)
        .collect();
    let diff = total - out.iter().sum::<i64>();
    out[0] += diff;
    out
}

// ---------------------------------------------------------------------------
// Geração

fn input<T: DeserializeOwned>(v: Value) -> Result<T> {
    serde_json::from_value(v).map_err(|e| {
        Error::Other(t_args(
            "nucleo-geral-erro-demo-interno",
            &args(&[("erro", e.to_string().into())]),
        ))
    })
}

/// Dia `day` do mês de `month` (primeiro dia), limitado ao fim do mês.
/// Maior gasto que o gerador deixa no dia de hoje (centavos): R$ 30, um pão ou um lanche.
const TODAY_MAX_SPEND: i64 = 3_000;

fn on(month: NaiveDate, day: u32) -> NaiveDate {
    let last = month_bounds(month.year(), month.month()).1;
    month.with_day(day.min(last.day())).unwrap_or(last)
}

/// Prestação de um financiamento pela tabela Price.
fn present_value(installment: i64, monthly_rate: f64, n: u32) -> i64 {
    let i = monthly_rate;
    (installment as f64 * (1.0 - (1.0 + i).powi(-(n as i32))) / i).round() as i64
}

struct Gen<'a, 'b> {
    tx: &'a Transaction<'b>,
    p: &'a DemoProfile,
    rng: Rng,
    today: NaiveDate,
    start: NaiveDate,
    cats: HashMap<String, Id>,
    members: HashMap<String, Id>,
    txns: Vec<Id>,
    accounts: u32,
    /// Textos no idioma do app e o país da família.
    s: Texts,
    /// Quantas menores unidades fazem uma unidade da moeda da família (100 no real, 1 no iene e
    /// no peso chileno): os valores do perfil chegam na menor unidade.
    unit: f64,
}

impl Gen<'_, '_> {
    fn t(&self, key: &str) -> String {
        self.s.t(key)
    }

    fn ta(&self, key: &str, a: &[(&'static str, String)]) -> String {
        self.s.ta(key, a)
    }

    /// Um nome sorteado da lista do catálogo.
    fn pick_text(&mut self, key: &str) -> String {
        let list = self.s.list(key);
        if list.is_empty() {
            return self.t(key);
        }
        self.rng.pick(&list).clone()
    }

    /// Categoria padrão pelo caminho em pt-BR. Subcategoria que não existe no país da família
    /// (IPVA, INSS…) cai na categoria-mãe.
    fn cat(&self, path: &str) -> Result<Id> {
        self.cats
            .get(path)
            .or_else(|| {
                path.rsplit_once(" › ")
                    .and_then(|(parent, _)| self.cats.get(parent))
            })
            .cloned()
            .ok_or_else(|| {
                Error::Other(t_args(
                    "nucleo-geral-erro-demo-categoria",
                    &args(&[("caminho", path.to_string().into())]),
                ))
            })
    }

    fn member(&self, name: &str) -> Option<Id> {
        self.members.get(name.trim()).cloned()
    }

    fn account(&mut self, v: Value) -> Result<Id> {
        self.accounts += 1;
        crate::accounts::create(self.tx, &input(v)?, self.today)
    }

    /// Lançamento efetivado (`amount` com sinal), só até hoje.
    fn txn(
        &mut self,
        account: &str,
        date: NaiveDate,
        amount: i64,
        cat: &str,
        description: &str,
        member: Option<&Id>,
    ) -> Result<Option<Id>> {
        if date > self.today || amount == 0 {
            return Ok(None);
        }
        // O cofre abre hoje: compras maiores que um lanche no próprio dia passam para ontem,
        // para o "quanto posso gastar hoje" começar com o dia livre (as pequenas ficam).
        let date = if date == self.today && amount < -TODAY_MAX_SPEND && self.today.day() > 1 {
            date - chrono::Duration::days(1)
        } else {
            date
        };
        let kind = if amount > 0 { "income" } else { "expense" };
        let mut v = json!({
            "account_id": account,
            "date": fmt_date(date),
            "amount": amount,
            "kind": kind,
            "status": "cleared",
            "description": description,
            "category_id": self.cat(cat)?,
        });
        if let Some(m) = member {
            v["member_id"] = json!(m);
        }
        let id = crate::transactions::create(self.tx, &input(v)?, self.today)?;
        self.txns.push(id.clone());
        Ok(Some(id))
    }

    fn transfer(
        &mut self,
        from: &str,
        to: &str,
        date: NaiveDate,
        amount: i64,
        description: &str,
    ) -> Result<()> {
        if date > self.today || amount <= 0 {
            return Ok(());
        }
        crate::transactions::create_transfer(
            self.tx,
            &input(json!({
                "from_account_id": from,
                "to_account_id": to,
                "date": fmt_date(date),
                "amount": amount,
                "description": description,
                "kind": "investment",
                "status": "cleared",
            }))?,
            self.today,
        )?;
        Ok(())
    }

    /// Meses do histórico (primeiro dia de cada um), do mais antigo ao atual.
    fn months(&self) -> Vec<NaiveDate> {
        (0..self.p.months)
            .map(|i| add_months(self.start, i as i32))
            .collect()
    }
}

struct Accounts {
    checking: Id,
    card: Option<Id>,
    savings: Option<Id>,
    broker: Option<Id>,
    pension: Option<Id>,
}

/// Gera o histórico no cofre aberto (que deve estar sem lançamentos).
pub fn generate(tx: &Transaction<'_>, p: &DemoProfile, today: NaiveDate) -> Result<DemoSummary> {
    validate(p)?;
    let has_txns: bool = tx.query_row("SELECT EXISTS (SELECT 1 FROM txn)", [], |r| r.get(0))?;
    if has_txns {
        return Err(Error::invalid(t("nucleo-geral-erro-demo-cofre-com-dados")));
    }
    let this_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .ok_or_else(|| Error::invalid(t("nucleo-geral-erro-demo-data")))?;
    let start = add_months(this_month, -(p.months as i32 - 1));
    // Pelos caminhos em pt-BR: o cofre pode ter nascido com as categorias em outro idioma.
    let cats = crate::categories::br_categories(tx)?
        .into_iter()
        .map(|c| (c.path, c.id))
        .collect();
    let members = {
        let mut stmt = tx.prepare("SELECT name, id FROM member")?;
        stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Id>(1)?)))?
            .collect::<rusqlite::Result<HashMap<_, _>>>()?
    };
    let mut g = Gen {
        tx,
        p,
        rng: Rng(seed_from(p)),
        today,
        start,
        cats,
        members,
        txns: vec![],
        accounts: 0,
        s: Texts::new(&crate::country::family_country(tx)),
        unit: 10f64.powi(crate::money::minor_units(&crate::household::base_currency(tx)?) as i32),
    };
    people(&g)?;
    let acc = accounts(&mut g)?;
    let mut loans = 0;
    for (index, month) in g.months().into_iter().enumerate() {
        income(&mut g, &acc, month)?;
        home(&mut g, &acc, month)?;
        living(&mut g, &acc, month)?;
        daily::extras(&mut g, &acc, month, index as u32)?;
    }
    daily::installment_purchase(&mut g, &acc)?;
    if p.housing == DemoHousing::Mortgage {
        loan_history(&mut g, &acc, LoanKind::Mortgage)?;
        loans += 1;
    }
    if p.car_installment > 0 {
        loan_history(&mut g, &acc, LoanKind::Car)?;
        loans += 1;
    }
    if p.personal_loan.unwrap_or(0) > 0 {
        loan_history(&mut g, &acc, LoanKind::Personal)?;
        loans += 1;
    }
    pay_card(&mut g, &acc)?;
    receipts(&mut g)?;
    let assets = invest(&mut g, &acc)?;
    overdraft_interest(&mut g, &acc)?;
    payslips(&mut g, &acc)?;
    future(&mut g, &acc)?;
    budget_and_goal(&mut g, &acc)?;
    let level = p.level.unwrap_or(Level::Avancado);
    let mut prof = crate::profile::get(tx)?;
    prof.level = level;
    prof.tone = crate::profile::tone_for(level);
    prof.source = crate::profile::Source::Escolha;
    prof.onboarding_pending = false;
    crate::profile::save(tx, &prof)?;
    crate::settings::set(tx, SETTING, &true)?;
    let transactions: u32 = tx.query_row("SELECT count(*) FROM txn", [], |r| r.get(0))?;
    Ok(DemoSummary {
        from: fmt_date(start),
        to: fmt_date(today),
        accounts: g.accounts,
        transactions,
        assets,
        loans,
    })
}

/// Cupons fiscais das compras de mercado dos últimos três meses (só no Brasil): itens de
/// mercado inventados (sem marca), com o preço subindo um pouco por mês e somando o valor de
/// cada lançamento (docs/consumption.md); os itens saem de [`daily::receipt`].
fn receipts(g: &mut Gen<'_, '_>) -> Result<()> {
    if g.s.country != "BR" {
        return Ok(());
    }
    let Some(cat) = g.cats.get("Alimentação › Mercado").cloned() else {
        return Ok(());
    };
    let from = add_months(
        NaiveDate::from_ymd_opt(g.today.year(), g.today.month(), 1).expect("data"),
        -2,
    );
    let mut stmt = g.tx.prepare(
        "SELECT id, date, -amount, description FROM txn
          WHERE category_id = ?1 AND date >= ?2 AND amount < -3000 ORDER BY date",
    )?;
    let rows: Vec<(String, String, i64, String)> = stmt
        .query_map(params![cat, fmt_date(from)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);
    for (id, date, paid, store) in rows {
        let d = crate::time::parse_date(&date)?;
        // Inflação inventada: 0,8% ao mês desde o primeiro cupom.
        let months = (d.year() - from.year()) * 12 + d.month() as i32 - from.month() as i32;
        let factor = 1.0 + 0.008 * f64::from(months);
        daily::receipt(g, &id, d, &store, paid, factor)?;
    }
    Ok(())
}

/// Idades de hoje (adultos, filhos): as informadas e, no lugar das que faltam, idades que
/// combinam com o perfil. Sorteio próprio, para não mexer no resto do histórico.
fn ages(p: &DemoProfile) -> (Vec<u32>, Vec<u32>) {
    let mut rng = Rng(seed_from(p) ^ 0xA6E5_1DAD);
    let details = p.child_details.as_deref().unwrap_or(&[]);
    let n = p.children.len() as u32;
    let mut kids = Vec::with_capacity(p.children.len());
    // Filhos sem idade: o mais velho de 3 a 11 anos (mais, com mais irmãos), 2 a 4 anos entre um
    // e outro.
    let mut prev: Option<u32> = None;
    for i in 0..p.children.len() {
        let age = match details.get(i).and_then(|d| d.age) {
            Some(a) => a.min(17),
            None => match prev {
                None => rng.int(
                    3 + 3 * n.saturating_sub(1),
                    (11 + 2 * n.saturating_sub(1)).min(17),
                ),
                Some(pr) => pr.saturating_sub(rng.int(2, 4)),
            },
        };
        prev = Some(age);
        kids.push(age);
    }
    let oldest = kids.iter().copied().max();
    let retiree = |i: usize, a: &DemoAdult| {
        a.retired == Some(true) || (i == 0 && p.kind == Some(DemoKind::Retiree))
    };
    let mut adults: Vec<u32> = Vec::with_capacity(p.adults.len());
    for (i, a) in p.adults.iter().enumerate() {
        let age = match a.age {
            Some(x) => x.clamp(18, 100),
            None if retiree(i, a) => rng.int(64, 71),
            None if i == 1 => {
                let first = adults[0];
                if retiree(0, &p.adults[0]) {
                    // Cônjuge que ainda trabalha de quem já se aposentou.
                    rng.int(57, 63)
                } else {
                    let floor = oldest.map_or(18, |o| o + 20);
                    (first + rng.int(0, 6)).saturating_sub(3).max(floor)
                }
            }
            None if p.kind == Some(DemoKind::Single) => rng.int(25, 31),
            None => match oldest {
                Some(o) => (o + rng.int(25, 32)).max(28),
                None => rng.int(29, 39),
            },
        };
        adults.push(age);
    }
    (adults, kids)
}

/// Data de nascimento de quem tem `age` anos hoje (o aniversário já passou neste ano de vida).
fn birth_date(today: NaiveDate, age: u32, rng: &mut Rng) -> NaiveDate {
    let year = today.year() - age as i32;
    let anchor = NaiveDate::from_ymd_opt(year, today.month(), today.day())
        .or_else(|| NaiveDate::from_ymd_opt(year, today.month(), 28))
        .unwrap_or(today);
    anchor - chrono::Duration::days(i64::from(rng.int(1, 360)))
}

/// Cadastro de cada pessoa: nascimento coerente com o perfil, sexo informado e parentesco.
fn people(g: &Gen<'_, '_>) -> Result<()> {
    let p = g.p;
    let (adult_ages, kid_ages) = ages(p);
    let mut rng = Rng(seed_from(p) ^ 0xB1_27D4);
    let details = p.child_details.as_deref().unwrap_or(&[]);
    let mut save = |name: &str, age: u32, sex: Option<MemberSex>, rel: MemberRelationship| {
        let Some(id) = g.member(name) else {
            return Ok(());
        };
        let born = birth_date(g.today, age, &mut rng);
        crate::household::update_member(
            g.tx,
            &MemberPatch {
                id,
                name: None,
                color: None,
                is_primary: None,
                archived: None,
                sort_order: None,
                profile: Some(MemberProfile {
                    birth_date: Some(fmt_date(born)),
                    relationship: Some(rel),
                    irpf_dependent: false,
                    occupation: None,
                    notes: None,
                    sex,
                }),
            },
        )
        .map(|_| ())
    };
    for (i, (a, age)) in p.adults.iter().zip(adult_ages).enumerate() {
        let rel = if i == 0 {
            MemberRelationship::Titular
        } else {
            MemberRelationship::Conjuge
        };
        save(&a.name, age, a.sex, rel)?;
    }
    for (i, (c, age)) in p.children.iter().zip(kid_ages).enumerate() {
        let sex = details.get(i).and_then(|d| d.sex);
        save(c, age, sex, MemberRelationship::Filho)?;
    }
    Ok(())
}

fn accounts(g: &mut Gen<'_, '_>) -> Result<Accounts> {
    let p = g.p;
    let bank = p.bank.trim().to_string();
    let opening = fmt_date(g.start.pred_opt().unwrap_or(g.start));
    let adults: Vec<Id> = p.adults.iter().filter_map(|a| g.member(&a.name)).collect();
    let income: i64 = p.adults.iter().map(|a| a.income).sum::<i64>() + p.other_income;
    let mut checking = json!({
        "name": g.ta("demodados-conta", &[("banco", bank.clone())]),
        "kind": "checking",
        "institution_name": bank,
        "member_ids": adults,
        "opening_balance": income,
        "opening_date": opening,
    });
    if let Some(limit) = p.overdraft_limit.filter(|l| *l > 0) {
        // Quem vive no cheque especial começa o histórico sem sobra na conta.
        checking["opening_balance"] = json!(0);
        checking["overdraft"] = json!({ "limit": limit, "monthly_rate": "8" });
    }
    let checking = g.account(checking)?;
    let last4 = format!("{:04}", g.rng.int(1000, 9999));
    let card = match p.card_limit.filter(|l| *l > 0) {
        Some(limit) => Some(g.account(json!({
            "name": g.ta("demodados-cartao", &[("banco", bank.clone())]),
            "kind": "credit_card",
            "institution_name": bank,
            "member_ids": adults.first().map(|m| vec![m.clone()]).unwrap_or_default(),
            "opening_date": opening,
            "card": {
                "credit_limit": limit,
                "closing_day": 3,
                "due_day": 10,
                "payment_account_id": checking,
                "network": "Mastercard",
                "last4": last4,
            },
        }))?),
        None => None,
    };
    let savings = if p.reserve > 0 || p.reserve_monthly > 0 {
        let before = (p.reserve - p.reserve_monthly * i64::from(p.months)).max(p.reserve / 3);
        Some(g.account(json!({
            "name": g.ta("demodados-reserva", &[("banco", bank.clone())]),
            "kind": "savings",
            "institution_name": bank,
            "member_ids": adults,
            "opening_balance": before.max(0),
            "opening_date": opening,
        }))?)
    } else {
        None
    };
    let broker = if p.stocks + p.fiis + p.fixed_income + p.invest_monthly > 0 {
        Some(g.account(json!({
            "name": g.t("demodados-corretora"),
            "kind": "brokerage",
            "institution_name": g.t("demodados-corretora-instituicao"),
            "member_ids": adults,
            "opening_date": opening,
        }))?)
    } else {
        None
    };
    let pension = if p.pension + p.pension_monthly > 0 {
        Some(g.account(json!({
            "name": g.ta("demodados-previdencia", &[("banco", bank.clone())]),
            "kind": "brokerage",
            "institution_name": bank,
            "member_ids": adults.first().map(|m| vec![m.clone()]).unwrap_or_default(),
            "opening_date": opening,
        }))?)
    } else {
        None
    };
    Ok(Accounts {
        checking,
        card,
        savings,
        broker,
        pension,
    })
}

/// Empresa do adulto `i` (da lista do catálogo, no idioma do app).
fn employer(g: &Gen<'_, '_>, i: usize) -> String {
    let list = g.s.list("demodados-empresas");
    if list.is_empty() {
        return String::new();
    }
    list[(i * 3 + g.p.adults.len()) % list.len()].clone()
}

fn retired(adult: &DemoAdult) -> bool {
    adult.retired.unwrap_or(false)
}

/// Descrição e categoria da renda fixa do adulto `i`: salário ou aposentadoria.
fn fixed_income(g: &Gen<'_, '_>, i: usize, adult: &DemoAdult) -> (String, &'static str) {
    if retired(adult) {
        // A subcategoria do INSS só existe no Brasil; nos outros países, a categoria-mãe.
        let cat = if g.s.brazil() {
            "Aposentadoria e pensão › INSS"
        } else {
            "Aposentadoria e pensão"
        };
        (g.t("demodados-aposentadoria"), cat)
    } else {
        (
            g.ta("demodados-salario", &[("empresa", employer(g, i))]),
            "Salário",
        )
    }
}

fn income(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let p = g.p;
    for (i, adult) in p.adults.iter().enumerate() {
        if adult.income <= 0 {
            continue;
        }
        let who = g.member(&adult.name);
        if adult.variable {
            let parts = g.rng.int(2, 3) as usize;
            let total = g.rng.vary(adult.income, 0.25);
            for part in split(&mut g.rng, total, parts) {
                let day = g.rng.int(3, 26);
                let client = g.pick_text("demodados-clientes");
                let text = g.ta("demodados-recebimento", &[("cliente", client)]);
                g.txn(
                    &a.checking,
                    on(month, day),
                    part,
                    "Serviços e freelance › Projetos e consultoria",
                    &text,
                    who.as_ref(),
                )?;
            }
        } else {
            let (text, cat) = fixed_income(g, i, adult);
            g.txn(
                &a.checking,
                on(month, 5),
                adult.income,
                cat,
                &text,
                who.as_ref(),
            )?;
            if g.s.brazil() {
                // 13º em duas parcelas (novembro e dezembro), também para quem é aposentado.
                let half = adult.income / 2;
                if month.month() == 11 {
                    let text = g.t("demodados-13-primeira");
                    g.txn(
                        &a.checking,
                        on(month, 28),
                        half,
                        "Salário › 13º salário",
                        &text,
                        who.as_ref(),
                    )?;
                }
                if month.month() == 12 {
                    let text = g.t("demodados-13-segunda");
                    g.txn(
                        &a.checking,
                        on(month, 18),
                        adult.income - half,
                        "Salário › 13º salário",
                        &text,
                        who.as_ref(),
                    )?;
                }
            } else if g.s.portugal() {
                // Subsídios de Natal (dezembro) e de férias (julho): um salário cada.
                if month.month() == 12 {
                    let text = g.t("demodados-subsidio-natal");
                    g.txn(
                        &a.checking,
                        on(month, 15),
                        adult.income,
                        "Salário › 13º salário",
                        &text,
                        who.as_ref(),
                    )?;
                }
                if month.month() == 7 {
                    let text = g.t("demodados-subsidio-ferias");
                    g.txn(
                        &a.checking,
                        on(month, 15),
                        adult.income,
                        "Salário › Férias",
                        &text,
                        who.as_ref(),
                    )?;
                }
            }
        }
    }
    if p.other_income > 0 {
        let text = g.t("demodados-aluguel-recebido");
        g.txn(
            &a.checking,
            on(month, 12),
            p.other_income,
            "Rendimentos › Aluguéis recebidos",
            &text,
            None,
        )?;
    }
    if let Some(benefit) = p.benefit.filter(|b| *b > 0) {
        let cat = if g.s.brazil() {
            "Auxílios do governo › Bolsa Família"
        } else {
            "Auxílios do governo › Outros programas sociais"
        };
        let text = g.t("demodados-beneficio");
        let who = p.adults.first().and_then(|x| g.member(&x.name));
        g.txn(
            &a.checking,
            on(month, 20),
            benefit,
            cat,
            &text,
            who.as_ref(),
        )?;
    }
    Ok(())
}

/// Conta da casa paga pela conta principal. Com `late_bills`, uma em cada três vai com atraso
/// de dez dias, mais multa e juros.
fn bill(
    g: &mut Gen<'_, '_>,
    a: &Accounts,
    date: NaiveDate,
    amount: i64,
    cat: &str,
    key: &str,
) -> Result<()> {
    let text = g.t(key);
    let late = g.p.late_bills.unwrap_or(false) && g.rng.unit() < 0.34;
    if !late {
        g.txn(&a.checking, date, -amount, cat, &text, None)?;
        return Ok(());
    }
    let paid = date + chrono::Duration::days(10);
    g.txn(&a.checking, paid, -amount, cat, &text, None)?;
    // Multa de 2% e juros de 1% ao mês, com um mínimo.
    let fee = (amount * 3 / 100).max(500);
    let fee_text = g.ta("demodados-multa-atraso", &[("conta", text)]);
    g.txn(
        &a.checking,
        paid,
        -fee,
        "Impostos e taxas › Juros e multas",
        &fee_text,
        None,
    )?;
    Ok(())
}

fn home(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let p = g.p;
    if p.housing == DemoHousing::Rent && p.housing_amount > 0 {
        bill(
            g,
            a,
            on(month, 8),
            p.housing_amount,
            "Moradia › Aluguel",
            "demodados-aluguel",
        )?;
    }
    if p.condo > 0 {
        let condo = g.rng.vary(p.condo, 0.03);
        bill(
            g,
            a,
            on(month, 10),
            condo,
            "Moradia › Condomínio",
            "demodados-condominio",
        )?;
    }
    if p.utilities > 0 {
        let u = p.utilities;
        // Luz varia com a estação; internet e celular são fixos.
        let summer = matches!(month.month(), 12 | 1 | 2 | 3);
        let light = g.rng.vary(u * 40 / 100, 0.15) * if summer { 115 } else { 100 } / 100;
        let water = g.rng.vary(u * 15 / 100, 0.15);
        bill(
            g,
            a,
            on(month, 12),
            light,
            "Moradia › Energia",
            "demodados-luz",
        )?;
        bill(
            g,
            a,
            on(month, 14),
            water,
            "Moradia › Água",
            "demodados-agua",
        )?;
        let internet = u * 25 / 100;
        bill(
            g,
            a,
            on(month, 18),
            internet,
            "Moradia › Internet",
            "demodados-internet",
        )?;
        let phone = u - u * 40 / 100 - u * 15 / 100 - internet;
        let card = a.card.clone().unwrap_or_else(|| a.checking.clone());
        let text = g.t("demodados-celular");
        g.txn(
            &card,
            on(month, 20),
            -phone,
            "Assinaturas › Telefone",
            &text,
            None,
        )?;
    }
    Ok(())
}

fn living(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let p = g.p;
    let card = a.card.clone().unwrap_or_else(|| a.checking.clone());
    let adults: Vec<Option<Id>> = p.adults.iter().map(|x| g.member(&x.name)).collect();
    let someone = |g: &mut Gen<'_, '_>| -> Option<Id> { g.rng.pick(&adults).clone() };
    // Mercado da semana, feira e padaria (com o cupom no Brasil).
    daily::groceries(g, a, month)?;
    match p.transport {
        DemoTransport::Car if p.transport_amount > 0 => {
            let total = g.rng.vary(p.transport_amount, 0.12);
            let fuel = total * 85 / 100;
            for v in split(&mut g.rng, fuel, 4) {
                let day = g.rng.int(1, 28);
                let st = g.pick_text("demodados-postos");
                let who = someone(g);
                g.txn(
                    &card,
                    on(month, day),
                    -v,
                    "Transporte › Combustível",
                    &st,
                    who.as_ref(),
                )?;
            }
            let parking = g.t("demodados-estacionamento");
            for v in split(&mut g.rng, total - fuel, 2) {
                let day = g.rng.int(1, 28);
                g.txn(
                    &card,
                    on(month, day),
                    -v,
                    "Transporte › Estacionamento",
                    &parking,
                    None,
                )?;
            }
            if month.month() == 1 {
                let text = g.t("demodados-ipva");
                g.txn(
                    &a.checking,
                    on(month, 20),
                    -(p.transport_amount * 3),
                    "Transporte › IPVA e licenciamento",
                    &text,
                    None,
                )?;
            }
            if month.month() == 3 {
                let text = g.t("demodados-seguro-carro");
                g.txn(
                    &card,
                    on(month, 15),
                    -(p.transport_amount * 5 / 2),
                    "Transporte › Seguro do veículo",
                    &text,
                    None,
                )?;
            }
            if month.month() % 6 == 2 {
                let service = g.rng.vary(p.transport_amount * 2 / 3, 0.3);
                let text = g.t("demodados-revisao");
                g.txn(
                    &card,
                    on(month, 22),
                    -service,
                    "Transporte › Manutenção do veículo",
                    &text,
                    None,
                )?;
            }
        }
        DemoTransport::App if p.transport_amount > 0 => {
            let total = g.rng.vary(p.transport_amount, 0.15);
            let n = g.rng.int(8, 12) as usize;
            for v in split(&mut g.rng, total, n) {
                let day = g.rng.int(1, 28);
                let app = g.pick_text("demodados-apps");
                let who = someone(g);
                g.txn(
                    &card,
                    on(month, day),
                    -v,
                    "Transporte › Aplicativos",
                    &app,
                    who.as_ref(),
                )?;
            }
        }
        DemoTransport::Public if p.transport_amount > 0 => {
            let text = g.t("demodados-transporte-publico");
            g.txn(
                &a.checking,
                on(month, 2),
                -p.transport_amount,
                "Transporte › Transporte público",
                &text,
                None,
            )?;
        }
        _ => {}
    }
    for child in &p.children {
        let who = g.member(child);
        if p.school_per_child > 0 {
            let text = g.ta("demodados-escola", &[("nome", child.trim().to_string())]);
            g.txn(
                &a.checking,
                on(month, 5),
                -p.school_per_child,
                "Educação › Escola",
                &text,
                who.as_ref(),
            )?;
            if month.month() == 2 {
                let text = g.ta("demodados-material", &[("nome", child.trim().to_string())]);
                g.txn(
                    &card,
                    on(month, 3),
                    -(p.school_per_child * 3 / 5),
                    "Educação › Material escolar",
                    &text,
                    who.as_ref(),
                )?;
            }
        }
    }
    if p.health > 0 {
        let plan = p.health * 85 / 100;
        let text = g.t("demodados-plano-saude");
        g.txn(
            &a.checking,
            on(month, 15),
            -plan,
            "Saúde › Plano de saúde",
            &text,
            None,
        )?;
        let extra = g.rng.vary(p.health - plan, 0.4).max(1);
        for v in split(&mut g.rng, extra, 2) {
            let day = g.rng.int(1, 28);
            let who = someone(g);
            let store = g.pick_text("demodados-farmacias");
            g.txn(
                &card,
                on(month, day),
                -v,
                "Saúde › Farmácia",
                &store,
                who.as_ref(),
            )?;
        }
    }
    daily::leisure(g, a, month)?;
    daily::subscriptions(g, a, month)?;
    if p.shopping > 0 {
        // Novembro tem Black Friday.
        let total = g.rng.vary(p.shopping, 0.5) * if month.month() == 11 { 2 } else { 1 };
        let kinds = [
            ("Compras › Vestuário", "demodados-loja-roupa"),
            ("Compras › Casa e decoração", "demodados-loja-casa"),
            ("Compras › Eletrônicos", "demodados-loja-eletronicos"),
        ];
        let n = g.rng.int(1, 3) as usize;
        for v in split(&mut g.rng, total, n) {
            let (cat, key) = *g.rng.pick(&kinds);
            let store = g.t(key);
            let day = g.rng.int(1, 28);
            let who = someone(g);
            g.txn(&card, on(month, day), -v, cat, &store, who.as_ref())?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum LoanKind {
    Mortgage,
    Car,
    Personal,
}

/// Contrato de dívida com as parcelas do histórico pagas por lançamentos na categoria dele
/// (ligados às parcelas) e as anteriores marcadas como pagas.
fn loan_history(g: &mut Gen<'_, '_>, a: &Accounts, kind: LoanKind) -> Result<()> {
    let p = g.p;
    let in_history = g
        .months()
        .into_iter()
        .filter(|m| on(*m, 15) <= g.today)
        .count() as u32;
    let (name, loan_kind, cat, installment, rate_month, annual, term, paid_before, system) =
        match kind {
            LoanKind::Mortgage => (
                g.t("demodados-financiamento-casa"),
                "mortgage",
                "Pagamentos › Financiamentos",
                p.housing_amount,
                0.0080,
                "10.03",
                360u32,
                48u32,
                "price",
            ),
            LoanKind::Car => {
                // Parcelas que vencem no histórico (dia 15 de cada mês, até hoje).
                let paid = 12;
                let term = p.car_installments_left + in_history + paid;
                (
                    g.t("demodados-financiamento-carro"),
                    "vehicle",
                    "Pagamentos › Parcelas",
                    p.car_installment,
                    0.0160,
                    "20.98",
                    term,
                    paid,
                    "price",
                )
            }
            LoanKind::Personal => {
                // Empréstimo pessoal caro (4,5% ao mês), feito pouco antes do histórico.
                let paid = 3;
                let term = p.personal_loan_left.unwrap_or(0) + in_history + paid;
                (
                    g.t("demodados-emprestimo-pessoal"),
                    "personal",
                    "Pagamentos › Empréstimos",
                    p.personal_loan.unwrap_or(0),
                    0.045,
                    "69.59",
                    term,
                    paid,
                    "price",
                )
            }
        };
    // O contrato começou antes do histórico: `paid_before` parcelas já pagas.
    let first_due = add_months(on(g.start, 15), -(paid_before as i32));
    let contract = first_due.pred_opt().unwrap_or(first_due) - chrono::Duration::days(30);
    let principal = present_value(installment, rate_month, term);
    let id = crate::debt::create(
        g.tx,
        &input(json!({
            "name": name,
            "kind": loan_kind,
            "principal": principal,
            "annual_rate": annual,
            "system": system,
            "contract_date": fmt_date(contract),
            "first_due_date": fmt_date(first_due),
            "term_months": term,
            "debit_account_id": a.checking,
            "category_id": g.cat(cat)?,
        }))?,
    )?;
    crate::debt::mark_paid_until(g.tx, &id, &fmt_date(g.start))?;
    // O bem comprado com o financiamento (entrada de 20% no imóvel e de 40% no carro), ligado
    // ao contrato: o patrimônio fica com o bem e a dívida, como na vida real.
    let good = match kind {
        LoanKind::Mortgage => Some(("real_estate", "demodados-bem-imovel", 80)),
        LoanKind::Car => Some(("vehicle", "demodados-bem-carro", 60)),
        LoanKind::Personal => None,
    };
    if let Some((good_kind, key, financed_pct)) = good {
        let members: Vec<Id> = p.adults.iter().filter_map(|x| g.member(&x.name)).collect();
        crate::physical::create(
            g.tx,
            &input(json!({
                "name": g.t(key),
                "kind": good_kind,
                "purchase_date": fmt_date(contract),
                "purchase_price": principal * 100 / financed_pct,
                "member_ids": members,
                "loan_ids": [id],
            }))?,
        )?;
    }
    let due: Vec<(String, i64)> = {
        let mut stmt = g.tx.prepare(
            "SELECT due_date, total FROM loan_installment
              WHERE loan_id = ?1 AND due_date >= ?2 AND due_date <= ?3 ORDER BY number",
        )?;
        stmt.query_map(params![id, fmt_date(g.start), fmt_date(g.today)], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let from = g.txns.len();
    let text = g.ta("demodados-parcela", &[("contrato", name.to_lowercase())]);
    for (date, total) in due {
        let d = crate::time::parse_date(&date)?;
        g.txn(&a.checking, d, -total, cat, &text, None)?;
    }
    crate::debt::link_category_payments(g.tx, &g.txns[from..])?;
    Ok(())
}

/// Faturas vencidas pagas no vencimento, pela conta principal. No rotativo, paga só 70% de
/// cada fatura; o resto fica em aberto e o cartão cobra juros (12% sobre o que ficou) logo
/// depois do vencimento, na fatura seguinte.
fn pay_card(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<()> {
    let Some(card) = &a.card else {
        return Ok(());
    };
    let due: Vec<(Id, String)> = {
        let mut stmt = g.tx.prepare(
            "SELECT id, due_date FROM card_statement WHERE account_id = ?1 AND due_date <= ?2
              ORDER BY due_date",
        )?;
        stmt.query_map(params![card, fmt_date(g.today)], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    for (id, date) in due {
        let total: i64 = g.tx.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM txn WHERE statement_id = ?1",
            [&id],
            |r| r.get(0),
        )?;
        if total >= 0 {
            continue;
        }
        let mut pay = json!({
            "statement_id": id,
            "from_account_id": a.checking,
            "date": date,
        });
        if g.p.card_revolving.unwrap_or(false) {
            let paid = -total * 70 / 100;
            pay["amount"] = json!(paid);
            let left = -total - paid;
            let interest = left * 12 / 100;
            let due = crate::time::parse_date(&date)?;
            let text = g.t("demodados-juros-rotativo");
            g.txn(
                card,
                due + chrono::Duration::days(1),
                -interest,
                "Impostos e taxas › Juros e multas",
                &text,
                None,
            )?;
        }
        crate::cards::pay_statement(g.tx, &input(pay)?, g.today)?;
    }
    Ok(())
}

struct DemoAsset {
    ticker: Option<&'static str>,
    name: &'static str,
    class: &'static str,
    price: f64,
    drift: f64,
    vol: f64,
    /// Provento mensal (FII) ou trimestral (ação), em fração do valor.
    yield_month: f64,
    quarterly: bool,
    /// Quantidade fracionária (Tesouro, previdência).
    fractional: bool,
}

const STOCKS: [DemoAsset; 4] = [
    DemoAsset {
        ticker: Some("PETR4"),
        name: "Petrobras PN",
        class: "stock_br",
        price: 36.0,
        drift: 0.004,
        vol: 0.07,
        yield_month: 0.03,
        quarterly: true,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("ITSA4"),
        name: "Itaúsa PN",
        class: "stock_br",
        price: 10.5,
        drift: 0.006,
        vol: 0.05,
        yield_month: 0.015,
        quarterly: true,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("WEGE3"),
        name: "WEG ON",
        class: "stock_br",
        price: 42.0,
        drift: 0.008,
        vol: 0.06,
        yield_month: 0.004,
        quarterly: true,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("BBAS3"),
        name: "Banco do Brasil ON",
        class: "stock_br",
        price: 27.0,
        drift: 0.005,
        vol: 0.06,
        yield_month: 0.02,
        quarterly: true,
        fractional: false,
    },
];

const FIIS: [DemoAsset; 3] = [
    DemoAsset {
        ticker: Some("HGLG11"),
        name: "CSHG Logística",
        class: "fii",
        price: 160.0,
        drift: 0.002,
        vol: 0.03,
        yield_month: 0.0075,
        quarterly: false,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("MXRF11"),
        name: "Maxi Renda",
        class: "fii",
        price: 10.2,
        drift: 0.0,
        vol: 0.025,
        yield_month: 0.009,
        quarterly: false,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("XPML11"),
        name: "XP Malls",
        class: "fii",
        price: 105.0,
        drift: 0.002,
        vol: 0.03,
        yield_month: 0.0078,
        quarterly: false,
        fractional: false,
    },
];

const TREASURY: DemoAsset = DemoAsset {
    ticker: None,
    name: "Tesouro Selic 2029",
    class: "treasury_br",
    price: 15_200.0,
    drift: 0.0085,
    vol: 0.0,
    yield_month: 0.0,
    quarterly: false,
    fractional: true,
};

/// Fora do Brasil: ETFs amplos no lugar das ações, imóveis listados no lugar dos FIIs e um
/// ETF de títulos no lugar do Tesouro (nos EUA, os listados lá; nos outros países, os
/// europeus de distribuição global).
const STOCKS_US: [DemoAsset; 2] = [
    DemoAsset {
        ticker: Some("VTI"),
        name: "Vanguard Total Stock Market ETF",
        class: "etf_us",
        price: 260.0,
        drift: 0.007,
        vol: 0.04,
        yield_month: 0.0035,
        quarterly: true,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("VXUS"),
        name: "Vanguard Total International Stock ETF",
        class: "etf_us",
        price: 62.0,
        drift: 0.005,
        vol: 0.04,
        yield_month: 0.0075,
        quarterly: true,
        fractional: false,
    },
];

const REITS_US: [DemoAsset; 1] = [DemoAsset {
    ticker: Some("VNQ"),
    name: "Vanguard Real Estate ETF",
    class: "reit",
    price: 88.0,
    drift: 0.003,
    vol: 0.045,
    yield_month: 0.0105,
    quarterly: true,
    fractional: false,
}];

const BONDS_US: DemoAsset = DemoAsset {
    ticker: Some("BND"),
    name: "Vanguard Total Bond Market ETF",
    class: "etf_us",
    price: 72.0,
    drift: 0.003,
    vol: 0.01,
    yield_month: 0.003,
    quarterly: false,
    fractional: false,
};

const STOCKS_INTL: [DemoAsset; 2] = [
    DemoAsset {
        ticker: Some("VWCE"),
        name: "Vanguard FTSE All-World UCITS ETF",
        class: "etf_intl",
        price: 120.0,
        drift: 0.007,
        vol: 0.04,
        yield_month: 0.0,
        quarterly: false,
        fractional: false,
    },
    DemoAsset {
        ticker: Some("IWDA"),
        name: "iShares Core MSCI World UCITS ETF",
        class: "etf_intl",
        price: 95.0,
        drift: 0.007,
        vol: 0.04,
        yield_month: 0.0,
        quarterly: false,
        fractional: false,
    },
];

const REITS_INTL: [DemoAsset; 1] = [DemoAsset {
    ticker: Some("IPRP"),
    name: "iShares European Property Yield UCITS ETF",
    class: "etf_intl",
    price: 30.0,
    drift: 0.002,
    vol: 0.045,
    yield_month: 0.009,
    quarterly: true,
    fractional: false,
}];

const BONDS_INTL: DemoAsset = DemoAsset {
    ticker: Some("AGGH"),
    name: "iShares Core Global Aggregate Bond UCITS ETF",
    class: "etf_intl",
    price: 5.5,
    drift: 0.002,
    vol: 0.01,
    yield_month: 0.0,
    quarterly: false,
    fractional: false,
};

const PENSION: DemoAsset = DemoAsset {
    ticker: None,
    name: "VGBL Moderado",
    class: "pension",
    price: 1.0,
    drift: 0.007,
    vol: 0.01,
    yield_month: 0.0,
    quarterly: false,
    fractional: true,
};

/// Um ativo cadastrado, com a série de preços (fim de cada semana do histórico).
struct Holding {
    id: Id,
    def: &'static DemoAsset,
    prices: Vec<(NaiveDate, f64)>,
    qty: f64,
}

impl Holding {
    fn price_at(&self, d: NaiveDate) -> f64 {
        self.prices
            .iter()
            .take_while(|(pd, _)| *pd <= d)
            .last()
            .or(self.prices.first())
            .map(|(_, v)| *v)
            .unwrap_or(self.def.price)
    }
}

fn dec(v: f64, places: usize) -> String {
    format!("{v:.places$}")
}

fn register(g: &mut Gen<'_, '_>, def: &'static DemoAsset) -> Result<Holding> {
    let bank = g.p.bank.trim().to_string();
    let brazil = g.s.brazil();
    let name = if def.class == "pension" && brazil {
        format!("{} {bank}", def.name)
    } else if def.class == "pension" {
        g.ta("demodados-fundo-aposentadoria", &[("banco", bank.clone())])
    } else {
        def.name.to_string()
    };
    // Fora do Brasil, a previdência é um fundo de aposentadoria comum (sem plano VGBL).
    let class = if def.class == "pension" && !brazil {
        "fund"
    } else {
        def.class
    };
    let mut v = json!({
        "name": name,
        "class": class,
        "price_provider": "manual",
        "notes": g.t("demodados-ativo-nota"),
    });
    if !brazil {
        v["currency"] = json!(crate::household::base_currency(g.tx)?);
    }
    if let Some(t) = def.ticker {
        v["ticker"] = json!(t);
    }
    // ETFs europeus: a bolsa alemã, onde todos os do exemplo são negociados.
    if def.class == "etf_intl" {
        v["market"] = json!("XETRA");
    }
    if def.class == "treasury_br" {
        v["treasury_code"] = json!(def.name);
    }
    if class == "pension" {
        v["pension"] = json!({
            "plan_type": "vgbl",
            "tax_regime": "regressive",
            "insurer": bank,
            "admin_fee_pct": "1.0",
            "start_date": fmt_date(g.start),
        });
    }
    let id = crate::assets::create(g.tx, &input(v)?)?;
    // Preço semanal: passeio aleatório com tendência.
    let mut prices = vec![];
    let mut price = def.price * g.rng.range(0.92, 1.08);
    let mut d = g.start;
    while d <= g.today {
        prices.push((d, price));
        let week = 1.0 + def.drift / 4.33 + def.vol / 2.08 * g.rng.range(-1.0, 1.0);
        price = (price * week).max(def.price * 0.2);
        d += chrono::Duration::days(7);
    }
    if prices.last().map(|(pd, _)| *pd) != Some(g.today) {
        prices.push((g.today, price));
    }
    let rows: Vec<crate::market::PriceRow> = prices
        .iter()
        .map(|(pd, v)| crate::market::PriceRow {
            date: *pd,
            open: None,
            high: None,
            low: None,
            close: dec(
                *v,
                if def.fractional && def.price < 10.0 {
                    6
                } else {
                    2
                },
            )
            .parse()
            .unwrap_or_default(),
            adj_close: None,
            volume: None,
        })
        .collect();
    crate::market::upsert_prices(g.tx, &id, &rows, "manual")?;
    Ok(Holding {
        id,
        def,
        prices,
        qty: 0.0,
    })
}

/// Compra `amount` (na menor unidade da moeda) do ativo na data; devolve quanto custou.
fn buy(
    g: &mut Gen<'_, '_>,
    h: &mut Holding,
    account: &str,
    date: NaiveDate,
    amount: i64,
    cash: bool,
) -> Result<i64> {
    if date > g.today || amount <= 0 {
        return Ok(0);
    }
    let price = h.price_at(date);
    let places = if h.def.price < 10.0 { 6 } else { 2 };
    let price_s = dec(price, places);
    let price_r: f64 = price_s.parse().unwrap_or(price);
    let qty = if h.def.fractional {
        ((amount as f64 / g.unit / price_r) * 100.0).floor() / 100.0
    } else {
        (amount as f64 / g.unit / price_r).floor()
    };
    if qty <= 0.0 {
        return Ok(0);
    }
    crate::portfolio::records::create_trade(
        g.tx,
        &input(json!({
            "asset_id": h.id,
            "account_id": account,
            "date": fmt_date(date),
            "side": "buy",
            "quantity": dec(qty, if h.def.fractional { 2 } else { 0 }),
            "price": price_s,
            "cash_movement": cash,
        }))?,
    )?;
    h.qty += qty;
    Ok((qty * price_r * g.unit).round() as i64)
}

fn invest(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<u32> {
    let p = g.p;
    let months = g.months();
    let mut assets = 0;
    // Reserva: aporte mensal e rendimento da poupança.
    if let Some(savings) = a.savings.clone() {
        let mut balance = (p.reserve - p.reserve_monthly * i64::from(p.months)).max(p.reserve / 3);
        for m in &months {
            if balance > 0 {
                let interest = (balance as f64 * 0.0065).round() as i64;
                let text = g.t("demodados-rendimento-reserva");
                g.txn(
                    &savings,
                    on(*m, 1),
                    interest,
                    "Rendimentos › Juros",
                    &text,
                    None,
                )?;
                balance += interest;
            }
            let text = g.t("demodados-aporte-reserva");
            g.transfer(&a.checking, &savings, on(*m, 6), p.reserve_monthly, &text)?;
            balance += p.reserve_monthly;
        }
    }
    if let Some(broker) = a.broker.clone() {
        let mut held: Vec<Holding> = vec![];
        let mut groups: Vec<(i64, &'static [DemoAsset])> = vec![];
        let (stocks, reits, bonds): (
            &'static [DemoAsset],
            &'static [DemoAsset],
            &'static DemoAsset,
        ) = if g.s.brazil() {
            (&STOCKS, &FIIS, &TREASURY)
        } else if g.s.usa() {
            (&STOCKS_US, &REITS_US, &BONDS_US)
        } else {
            (&STOCKS_INTL, &REITS_INTL, &BONDS_INTL)
        };
        if p.stocks > 0 || p.invest_monthly > 0 {
            groups.push((p.stocks, stocks));
        }
        if p.fiis > 0 || p.invest_monthly > 0 {
            groups.push((p.fiis, reits));
        }
        if p.fixed_income > 0 {
            groups.push((p.fixed_income, std::slice::from_ref(bonds)));
        }
        for (initial, defs) in groups {
            let share = initial / defs.len() as i64;
            for def in defs {
                let mut h = register(g, def)?;
                assets += 1;
                buy(g, &mut h, &broker, g.start, share, false)?;
                held.push(h);
            }
        }
        // Aporte mensal: transferência e compra de um ativo por vez (rodízio).
        let buyable: Vec<usize> = (0..held.len())
            .filter(|i| held[*i].def.class != "treasury_br")
            .collect();
        for (i, m) in months.iter().enumerate() {
            let day = on(*m, 6);
            if p.invest_monthly > 0 && !buyable.is_empty() {
                let text = g.t("demodados-aporte-corretora");
                g.transfer(&a.checking, &broker, day, p.invest_monthly, &text)?;
                let k = buyable[i % buyable.len()];
                buy(g, &mut held[k], &broker, day, p.invest_monthly, true)?;
            }
            // Proventos: FIIs todo mês, ações no fim de cada trimestre.
            for h in &held {
                if h.qty <= 0.0 || h.def.yield_month <= 0.0 {
                    continue;
                }
                if h.def.quarterly && m.month() % 3 != 0 {
                    continue;
                }
                let pay = on(*m, 15);
                if pay > g.today {
                    continue;
                }
                let gross = (h.qty * h.price_at(pay) * h.def.yield_month * g.unit).round() as i64;
                if gross <= 0 {
                    continue;
                }
                let kind = if h.def.class == "fii" {
                    "fii_income"
                } else {
                    "dividend"
                };
                crate::portfolio::records::create_income(
                    g.tx,
                    &input(json!({
                        "asset_id": h.id,
                        "account_id": broker,
                        "kind": kind,
                        "ex_date": fmt_date(on(*m, 5)),
                        "pay_date": fmt_date(pay),
                        "quantity": dec(h.qty, 0),
                        "gross": gross,
                        "status": "received",
                        "cash_movement": true,
                    }))?,
                )?;
            }
        }
    }
    if let Some(account) = a.pension.clone() {
        let mut h = register(g, &PENSION)?;
        assets += 1;
        buy(g, &mut h, &account, g.start, p.pension, false)?;
        for m in &months {
            let day = on(*m, 20);
            if p.pension_monthly > 0 && day <= g.today {
                let text = g.t("demodados-aporte-previdencia");
                g.transfer(&a.checking, &account, day, p.pension_monthly, &text)?;
                buy(g, &mut h, &account, day, p.pension_monthly, true)?;
            }
        }
    }
    Ok(assets)
}

/// As rendas fixas cadastradas em 02 · Família (salário ou aposentadoria), com os holerites
/// dos últimos meses (até seis): bruto, INSS e IRRF pelas regras do ano a partir do líquido.
fn payslips(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<()> {
    let p = g.p;
    for (i, adult) in p.adults.iter().enumerate() {
        if adult.income <= 0 || adult.variable {
            continue;
        }
        let Some(member) = g.member(&adult.name) else {
            continue;
        };
        let is_retired = retired(adult);
        let (name, cat) = fixed_income(g, i, adult);
        let company = (!is_retired).then(|| employer(g, i));
        let source = crate::income::create_source(
            g.tx,
            &input(json!({
                "member_id": member,
                "name": name,
                "kind": if is_retired { "retirement" } else { "clt" },
                "employer": company,
                "net_monthly": adult.income,
                "pay_day": 5,
                "has_thirteenth": g.s.brazil() || g.s.portugal(),
                "account_id": a.checking,
                "category_id": g.cat(cat)?,
            }))?,
        )?;
        let src = crate::income::get_source(g.tx, &source)?;
        let (gross, inss, irrf, other, net) = crate::income::declared_payslip(g.tx, &src, g.today)?;
        if let Some(company) = &company {
            daily::fgts(g, &member, i, &source, company, gross)?;
        }
        if !p.payslips.unwrap_or(true) {
            continue;
        }
        // Competência: o mês anterior a cada pagamento do dia 5 já feito; só os seis últimos.
        let mut refs: Vec<NaiveDate> = g
            .months()
            .into_iter()
            .filter(|m| on(*m, 5) <= g.today)
            .map(|m| add_months(m, -1))
            .collect();
        let skip = refs.len().saturating_sub(6);
        refs.drain(..skip);
        let now = crate::time::now_utc();
        let fgts = (g.s.brazil() && !is_retired).then_some(gross * 8 / 100);
        for r in refs {
            g.tx.execute(
                "INSERT INTO payslip (id, source_id, reference_month, kind, pay_date, gross,
                        social_security, income_tax, other_deductions, net, fgts, items,
                        created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'monthly', ?4, ?5, ?6, ?7, ?8, ?9, ?10, '[]', ?11, ?11)",
                params![
                    crate::ids::new_id(),
                    source,
                    format!("{:04}-{:02}", r.year(), r.month()),
                    fmt_date(on(add_months(r, 1), 5)),
                    gross,
                    inss,
                    irrf,
                    other,
                    net,
                    fgts,
                    now,
                ],
            )?;
        }
    }
    Ok(())
}

/// Cheque especial: com saldo negativo no fim do mês, juros de 8% sobre o que ficou negativo
/// no dia 1 do mês seguinte (que também entram no saldo dos meses seguintes).
fn overdraft_interest(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<()> {
    if g.p.overdraft_limit.unwrap_or(0) <= 0 {
        return Ok(());
    }
    let text = g.t("demodados-juros-cheque-especial");
    for m in g.months() {
        let next = add_months(m, 1);
        if next > g.today {
            break;
        }
        let end = next.pred_opt().unwrap_or(next);
        let balance = crate::accounts::balance(g.tx, &a.checking, &fmt_date(end), false)?;
        if balance < 0 {
            let interest = (-balance * 8 / 100).max(100);
            g.txn(
                &a.checking,
                next,
                -interest,
                "Impostos e taxas › Juros e multas",
                &text,
                None,
            )?;
        }
    }
    Ok(())
}

/// Recorrências dos próximos meses (salários e contas fixas), para os vencimentos.
fn future(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<()> {
    let p = g.p;
    let next = add_months(
        NaiveDate::from_ymd_opt(g.today.year(), g.today.month(), 1).unwrap_or(g.today),
        1,
    );
    let rec = |g: &mut Gen<'_, '_>,
               description: String,
               amount: i64,
               kind: &str,
               cat: &str,
               day: u32,
               member: Option<Id>|
     -> Result<()> {
        let mut v = json!({
            "description": description,
            "account_id": a.checking,
            "amount": amount,
            "kind": kind,
            "category_id": g.cat(cat)?,
            "spec": { "freq": "MONTHLY", "by_month_day": day },
            "start_date": fmt_date(on(next, day)),
        });
        if let Some(m) = member {
            v["member_id"] = json!(m);
        }
        crate::recurrence::create(g.tx, &input(v)?, g.today)?;
        Ok(())
    };
    for (i, adult) in p.adults.iter().enumerate() {
        if adult.income > 0 && !adult.variable {
            let (text, cat) = fixed_income(g, i, adult);
            let who = g.member(&adult.name);
            rec(g, text, adult.income, "income", cat, 5, who)?;
        }
    }
    if p.housing == DemoHousing::Rent && p.housing_amount > 0 {
        rec(
            g,
            g.t("demodados-aluguel"),
            -p.housing_amount,
            "expense",
            "Moradia › Aluguel",
            8,
            None,
        )?;
    }
    if p.condo > 0 {
        rec(
            g,
            g.t("demodados-condominio"),
            -p.condo,
            "expense",
            "Moradia › Condomínio",
            10,
            None,
        )?;
    }
    for child in &p.children {
        if p.school_per_child > 0 {
            let who = g.member(child);
            rec(
                g,
                g.ta("demodados-escola", &[("nome", child.trim().to_string())]),
                -p.school_per_child,
                "expense",
                "Educação › Escola",
                5,
                who,
            )?;
        }
    }
    if p.health > 0 {
        rec(
            g,
            g.t("demodados-plano-saude"),
            -(p.health * 85 / 100),
            "expense",
            "Saúde › Plano de saúde",
            15,
            None,
        )?;
    }
    Ok(())
}

/// Orçamento do mês atual pelo que a pessoa informou e a meta de reserva de emergência.
fn budget_and_goal(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<()> {
    let p = g.p;
    let kids = p.children.len() as i64;
    let round = |v: i64| (v + 5_000) / 10_000 * 10_000;
    let transport = match p.transport {
        DemoTransport::None => 0,
        _ => p.transport_amount,
    };
    let rent = if p.housing == DemoHousing::Rent {
        p.housing_amount
    } else {
        0
    };
    let mut lines = vec![
        ("Moradia", rent + p.condo + p.utilities * 80 / 100),
        ("Alimentação", p.groceries + p.leisure * 65 / 100),
        ("Transporte", transport),
        ("Educação", p.school_per_child * kids),
        ("Saúde", p.health),
        ("Lazer", p.leisure * 35 / 100),
        ("Assinaturas", p.subscriptions + p.utilities * 20 / 100),
        ("Compras", p.shopping),
    ];
    // O que é próprio do perfil (diarista, academia, pet, gás…) e a parcela do mês.
    for (cat, v) in daily::budget_extras(g) {
        match lines.iter_mut().find(|(c, _)| *c == cat) {
            Some(line) => line.1 += v,
            None => lines.push((cat, v)),
        }
    }
    let mut out = vec![];
    for (cat, v) in lines {
        if v > 0 {
            out.push(json!({ "category_id": g.cat(cat)?, "amount": round(v * 105 / 100) }));
        }
    }
    let month = format!("{:04}-{:02}", g.today.year(), g.today.month());
    crate::budget::set_lines(g.tx, &input(json!({ "month": month, "lines": out }))?)?;
    if let Some(savings) = &a.savings {
        let essential = rent
            + p.condo
            + p.utilities
            + p.groceries
            + transport
            + p.school_per_child * kids
            + p.health
            + if p.housing == DemoHousing::Mortgage {
                p.housing_amount
            } else {
                0
            }
            + p.car_installment;
        crate::goals::create(
            g.tx,
            &input(json!({
                "name": g.t("demodados-meta-reserva"),
                "kind": "emergency",
                "target_amount": round(essential * 6),
                "monthly_contribution": p.reserve_monthly,
                "account_ids": [savings],
            }))?,
        )?;
    }
    Ok(())
}
