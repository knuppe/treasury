//! Insights without AI over a year of hand-built data (today = 2026-10-18; the last full
//! month is September).

use rusqlite::params;

use super::*;
use crate::reports::fixture::{self, cat, expense, new_db, txn, write};
use crate::transactions::TxnKind;

fn d(s: &str) -> NaiveDate {
    crate::time::parse_date(s).unwrap()
}

fn index(conn: &Connection, series: &str, monthly: &str) {
    for m in crate::reports::engine::months_between("2025-10", "2026-09").unwrap() {
        conn.execute(
            "INSERT INTO index_value (series_id, date, value, estimated, source, fetched_at) VALUES (?1, ?2, ?3, 0, 'teste', 'x')",
            params![series, format!("{m}-01"), monthly],
        )
        .unwrap();
    }
}

fn scenario() -> fixture::TestDb {
    let mut t = new_db();
    write(&mut t.db, |tx| {
        let acc = fixture::account(tx, "Corrente", "checking", "BRL", 0, "2025-01-01");
        let other = fixture::account(tx, "Conta 2", "checking", "BRL", 0, "2025-01-01");
        let salario = cat(tx, "Salário");
        let aluguel = cat(tx, "Moradia › Aluguel");
        let mercado = cat(tx, "Alimentação › Mercado");
        let padaria = cat(tx, "Alimentação › Padaria");
        let comb = cat(tx, "Transporte › Combustível");
        for m in crate::reports::engine::months_between("2025-10", "2026-09").unwrap() {
            txn(
                tx,
                &acc,
                &format!("{m}-05"),
                500_000,
                TxnKind::Income,
                Some(&salario),
                None,
                "SALARIO",
            );
            expense(
                tx,
                &acc,
                &format!("{m}-10"),
                -150_000,
                Some(&aluguel),
                None,
                "ALUGUEL",
            );
            let food = if m == "2026-09" { -200_000 } else { -80_000 };
            expense(
                tx,
                &acc,
                &format!("{m}-15"),
                food,
                Some(&mercado),
                None,
                "MERCADO",
            );
            expense(
                tx,
                &acc,
                &format!("{m}-20"),
                -30_000,
                Some(&comb),
                None,
                "POSTO",
            );
            expense(
                tx,
                &acc,
                &format!("{m}-01"),
                -4_000,
                None,
                None,
                "TARIFA PACOTE SERVICOS",
            );
            expense(
                tx,
                &acc,
                &format!("{m}-02"),
                -6_000,
                None,
                None,
                "JUROS CHEQUE ESPECIAL",
            );
        }
        for i in 0..20 {
            expense(
                tx,
                &acc,
                &format!("2026-09-{:02}", i + 3),
                -1_500,
                Some(&padaria),
                None,
                "PADARIA",
            );
        }
        // Account 2 overdrawn from Sep 1 to Sep 20.
        expense(tx, &other, "2026-09-01", -50_000, None, None, "SAQUE");
        txn(
            tx,
            &other,
            "2026-09-20",
            60_000,
            TxnKind::Income,
            None,
            None,
            "DEPOSITO",
        );
        Ok(())
    });
    let c = t.db.conn();
    index(c, "ipca_g1", "1.0");
    index(c, "ipca_g2", "0.5");
    index(c, "ipca_g5", "0.2");
    index(c, "ipca", "0.4");
    t
}

#[test]
fn categories_map_to_ipca_groups() {
    assert_eq!(group_of("Alimentação › Mercado"), Some("ipca_g1"));
    assert_eq!(
        group_of("Moradia › Internet"),
        Some("ipca_g9"),
        "a subcategoria vence"
    );
    assert_eq!(group_of("Moradia › Aluguel"), Some("ipca_g2"));
    assert_eq!(group_of("Transporte › Combustível"), Some("ipca_g5"));
    assert_eq!(group_of("Impostos e taxas › Tarifas bancárias"), None);
    assert_eq!(group_of("Investimentos"), None);
}

#[test]
fn insights_from_a_year_of_data() {
    let t = scenario();
    let r = compute(t.db.conn(), d("2026-10-18")).unwrap();
    let kinds: Vec<InsightKind> = r.items.iter().map(|i| i.kind).collect();
    // Personal inflation: heavy food spending (1% per month) pulls above IPCA (0.4% per month).
    let p = r.personal_inflation.as_ref().expect("personal inflation");
    let ipca = p.ipca.unwrap();
    assert!((ipca - (1.004f64.powi(12) - 1.0)).abs() < 1e-9);
    assert!(p.personal > ipca, "{} × {ipca}", p.personal);
    assert_eq!(p.groups[0].name, "Alimentação e bebidas");
    let w: f64 = p.groups.iter().map(|g| g.weight).sum();
    assert!((w - 1.0).abs() < 1e-9);
    assert!(kinds.contains(&InsightKind::PersonalInflation));
    let infl = r
        .items
        .iter()
        .find(|i| i.kind == InsightKind::PersonalInflation)
        .unwrap();
    assert_eq!(infl.tone, InsightTone::Warning);
    assert!(
        infl.title.starts_with("Sua inflação em 12 meses"),
        "{}",
        infl.title
    );

    // Fees and interest: 12 × R$ 40 + 12 × R$ 60.
    let fees = r
        .items
        .iter()
        .find(|i| i.kind == InsightKind::FeesAndInterest)
        .unwrap();
    assert_eq!(fees.amount, Some(120_000));
    assert_eq!(fees.data["fees"], 48_000);
    assert_eq!(fees.data["interest"], 72_000);
    assert!(fees.action.as_deref().unwrap().contains("3.919"));

    // September groceries well above the median: Alimentação (R$ 2,000 + R$ 300 of bakery).
    let spike = r
        .items
        .iter()
        .find(|i| i.kind == InsightKind::CategorySpike)
        .unwrap();
    assert_eq!(spike.data["category"], "Alimentação");
    assert_eq!(spike.data["normal"], 80_000);
    assert_eq!(spike.data["amount"], 230_000);

    // Small purchases: 20 × R$ 15.
    let small = r
        .items
        .iter()
        .find(|i| i.kind == InsightKind::SmallPurchases)
        .unwrap();
    assert_eq!(small.amount, Some(30_000));

    // Account 2 negative from Sep 1 to Sep 20.
    let od = r
        .items
        .iter()
        .find(|i| i.kind == InsightKind::Overdraft)
        .unwrap();
    assert_eq!(od.data["negative_days"], 19);
    assert_eq!(od.data["min_balance"], -50_000);
    assert!(od.title.starts_with("Conta 2"));

    // Income of R$ 5,000 per month (plus the R$ 600 deposit in Account 2): essential × rest ×
    // surplus.
    let split = r
        .items
        .iter()
        .find(|i| i.kind == InsightKind::BudgetSplit)
        .unwrap();
    assert_eq!(split.data["income"], 1_560_000);
    let total = split.data["essential"].as_i64().unwrap()
        + split.data["other"].as_i64().unwrap()
        + split.data["saved"].as_i64().unwrap();
    assert_eq!(total, 1_560_000);

    // No spending above income; warnings come before the good news.
    assert!(!kinds.contains(&InsightKind::SpendingAboveIncome));
    let tones: Vec<InsightTone> = r.items.iter().map(|i| i.tone).collect();
    let mut sorted = tones.clone();
    sorted.sort();
    assert_eq!(tones, sorted);
}

#[test]
fn no_data_no_insights() {
    let t = new_db();
    let r = compute(t.db.conn(), d("2026-10-18")).unwrap();
    // Only the recovery kit reminder (vault without a kit).
    let kinds: Vec<InsightKind> = r.items.iter().map(|i| i.kind).collect();
    assert_eq!(kinds, [InsightKind::RecoveryKit]);
    assert!(r.personal_inflation.is_none());
}

#[test]
fn recovery_kit_reminder_until_made_and_again_after_a_password_change() {
    let t = new_db();
    let kit = |t: &crate::reports::fixture::TestDb| {
        compute(t.db.conn(), d("2026-10-18"))
            .unwrap()
            .items
            .into_iter()
            .find(|i| i.kind == InsightKind::RecoveryKit)
    };
    let i = kit(&t).unwrap();
    assert_eq!(i.id, "recovery-kit:2026-10", "volta todo mês");
    assert_eq!(i.screen.as_deref(), Some("ajustes?secao=seguranca"));
    crate::unlock::kit_shown(t.db.conn()).unwrap();
    assert!(kit(&t).is_none());
    // Changing the password invalidates the kit: the notice returns with a different text.
    crate::unlock::password_changed(t.db.conn()).unwrap();
    let i = kit(&t).unwrap();
    assert_eq!(i.data["outdated"], true);
    crate::unlock::kit_shown(t.db.conn()).unwrap();
    assert!(kit(&t).is_none());
    // A demo has no real password.
    crate::settings::delete(t.db.conn(), crate::unlock::KIT_AT).unwrap();
    crate::settings::set(t.db.conn(), crate::demo::SETTING, &true).unwrap();
    assert!(kit(&t).is_none());
}

/// The texts come out in the installation's language, with amounts and months in its format;
/// in pt-BR, as they always were.
#[test]
fn insights_follow_the_locale() {
    use crate::i18n::{Locale, set_current};
    let t = scenario();
    let find =
        |r: &Insights, k: InsightKind| r.items.iter().find(|i| i.kind == k).cloned().unwrap();
    let r = compute(t.db.conn(), d("2026-10-18")).unwrap();
    assert_eq!(
        find(&r, InsightKind::FeesAndInterest).title,
        "R$ 1.200,00 em tarifas e juros em 12 meses"
    );
    assert_eq!(
        find(&r, InsightKind::CategorySpike).title,
        "Alimentação acima do normal em set/2026"
    );
    set_current(Locale::En);
    let r = compute(t.db.conn(), d("2026-10-18")).unwrap();
    let infl = find(&r, InsightKind::PersonalInflation);
    assert!(
        infl.title.starts_with("Your inflation over 12 months: "),
        "{}",
        infl.title
    );
    assert!(
        infl.body
            .contains("What weighed the most: Food and beverages"),
        "{}",
        infl.body
    );
    assert_eq!(
        r.personal_inflation.as_ref().unwrap().groups[0].name,
        "Food and beverages"
    );
    assert_eq!(
        find(&r, InsightKind::FeesAndInterest).title,
        "R$1,200.00 in fees and interest over 12 months"
    );
    assert_eq!(
        find(&r, InsightKind::CategorySpike).title,
        "Alimentação above normal in Sep 2026"
    );
    set_current(Locale::PtBr);
}
