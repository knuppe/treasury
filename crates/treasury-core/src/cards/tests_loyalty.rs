//! Cartões, fase 5: programas de pontos, saldo com vencimentos (lotes na ordem de vencimento),
//! transferência com bônus, custo do milheiro, pontos previstos × creditados e o aviso de
//! vencimento.

use super::loyalty::{
    EntryKind, LoyaltyEntryInput, LoyaltyKind, LoyaltyProgramInput, LoyaltyTransferInput,
};
use super::rewards::{RewardKind, RewardProgramInput};
use super::*;
use crate::transactions::testkit::{Fx, d};

fn program(fx: &mut Fx, name: &str, kind: LoyaltyKind, value: Option<i64>) -> Id {
    let input = LoyaltyProgramInput {
        name: name.into(),
        kind: Some(kind),
        member_id: None,
        unit_label: None,
        value_per_1000: value,
        value_currency: None,
        notes: None,
    };
    fx.write(|tx| loyalty::create_program(tx, &input)).unwrap()
}

fn entry(
    fx: &mut Fx,
    program: &str,
    date: &str,
    points: i64,
    kind: EntryKind,
    expires: Option<&str>,
) {
    let input = LoyaltyEntryInput {
        program_id: program.into(),
        date: date.into(),
        points,
        kind,
        expires_on: expires.map(str::to_string),
        statement_id: None,
        cost: if kind == EntryKind::Purchase {
            Some(points * 7) // R$ 70 o milheiro
        } else {
            None
        },
        cost_currency: None,
        notes: None,
    };
    fx.write(|tx| loyalty::add_entry(tx, &input)).unwrap();
}

#[test]
fn balance_expiry_transfer_and_value() {
    let mut fx = Fx::new();
    let bank = program(&mut fx, "Banco Pontos", LoyaltyKind::Bank, Some(2_500));
    let air = program(&mut fx, "Aérea Milhas", LoyaltyKind::Airline, None);
    entry(
        &mut fx,
        &bank,
        "2026-01-10",
        10_000,
        EntryKind::Earn,
        Some("2026-12-31"),
    );
    entry(
        &mut fx,
        &bank,
        "2026-02-10",
        5_000,
        EntryKind::Earn,
        Some("2026-11-30"),
    );
    entry(
        &mut fx,
        &bank,
        "2026-03-10",
        3_000,
        EntryKind::Earn,
        Some("2026-06-30"),
    );
    // Resgate de 4 mil consome primeiro os que vencem antes (os 3 mil de junho e mil de novembro).
    entry(&mut fx, &bank, "2026-04-01", 4_000, EntryKind::Redeem, None);
    let p = loyalty::get(fx.conn(), &bank, d("2026-10-01")).unwrap();
    assert_eq!(p.balance, 14_000);
    assert_eq!(p.next_expiry.as_deref(), Some("2026-11-30"));
    assert_eq!(p.next_expiry_points, 4_000);
    assert_eq!(p.expiring_90d, 4_000, "só novembro cabe nos 90 dias");
    // R$ 25 o milheiro.
    assert_eq!(p.estimated_value, Some(35_000));
    // Os de novembro venceram sem uso: saem do saldo.
    let p = loyalty::get(fx.conn(), &bank, d("2026-12-05")).unwrap();
    assert_eq!(p.balance, 10_000);

    // Transferência com 100% de bônus: sai 5 mil, entram 10 mil.
    let t = LoyaltyTransferInput {
        from_program_id: bank.clone(),
        to_program_id: air.clone(),
        date: "2026-10-02".into(),
        points_out: 5_000,
        points_in: 10_000,
        expires_on: Some("2028-10-02".into()),
        notes: None,
    };
    fx.write(|tx| loyalty::transfer(tx, &t)).unwrap();
    let all = loyalty::list(fx.conn(), d("2026-10-03")).unwrap();
    let bal = |id: &str| all.iter().find(|p| p.id == id).unwrap().balance;
    assert_eq!((bal(&bank), bal(&air)), (9_000, 10_000));
    // Apagar um lado apaga o par.
    let out_id = loyalty::entries(fx.conn(), &bank)
        .unwrap()
        .into_iter()
        .find(|e| e.kind == EntryKind::TransferOut)
        .unwrap()
        .id;
    let n = fx.write(|tx| loyalty::delete_entry(tx, &out_id)).unwrap();
    assert_eq!(n, 2);

    // Compra de pontos: custo médio do milheiro.
    entry(
        &mut fx,
        &air,
        "2026-10-05",
        20_000,
        EntryKind::Purchase,
        None,
    );
    let p = loyalty::get(fx.conn(), &air, d("2026-10-06")).unwrap();
    assert_eq!(p.purchase_cost_per_1000, Some(7_000));

    // Transferência para o mesmo programa e tipo de par avulso são recusados.
    let mut bad = t.clone();
    bad.to_program_id = bank.clone();
    assert_eq!(
        fx.write(|tx| loyalty::transfer(tx, &bad))
            .unwrap_err()
            .code(),
        "invalid"
    );
    let e = LoyaltyEntryInput {
        program_id: bank.clone(),
        date: "2026-10-05".into(),
        points: 10,
        kind: EntryKind::TransferIn,
        expires_on: None,
        statement_id: None,
        cost: None,
        cost_currency: None,
        notes: None,
    };
    assert_eq!(
        fx.write(|tx| loyalty::add_entry(tx, &e))
            .unwrap_err()
            .code(),
        "invalid"
    );
}

#[test]
fn card_points_expected_vs_credited_and_expiry_warning() {
    let mut fx = Fx::new();
    let card = fx.card("Cartão", 3, 10, None);
    let bank = program(&mut fx, "Banco Pontos", LoyaltyKind::Bank, None);
    let input = RewardProgramInput {
        account_id: card.clone(),
        kind: RewardKind::Points,
        base_rate: "1".into(),
        rate_currency: None,
        cycle_cap: None,
        destination: None,
        destination_account_id: None,
        payout_days: Some(15),
        starts_on: Some("2026-01-01".into()),
        notes: None,
        multipliers: None,
        loyalty_program_id: Some(bank.clone()),
    };
    fx.write(|tx| rewards::set(tx, &input, d("2026-01-01")))
        .unwrap();
    fx.txn(&card, "2026-09-10", -100_000, "Loja");
    let st = statement_for(fx.conn(), &card, "2026-09-10")
        .unwrap()
        .unwrap();
    // 1 ponto por real: 1.000 pontos previstos; o banco creditou 400.
    let credit = LoyaltyEntryInput {
        program_id: bank.clone(),
        date: "2026-10-12".into(),
        points: 400,
        kind: EntryKind::Earn,
        expires_on: Some("2026-11-20".into()),
        statement_id: Some(st.clone()),
        cost: None,
        cost_currency: None,
        notes: None,
    };
    fx.write(|tx| loyalty::add_entry(tx, &credit)).unwrap();
    let v = rewards::view(fx.conn(), &card, d("2026-10-25")).unwrap();
    let s = v.statements.iter().find(|s| s.statement_id == st).unwrap();
    assert_eq!(
        (s.expected, s.received, s.missing),
        (Some(1_000), Some(400), true)
    );
    let p = loyalty::get(fx.conn(), &bank, d("2026-10-25")).unwrap();
    assert_eq!(p.cards.len(), 1, "o cartão que acumula no programa");

    // Vencimento a 26 dias: um aviso na faixa de 30 dias; de novo a 6 dias, na de 7.
    crate::notify::run(fx.conn(), d("2026-10-25")).unwrap();
    crate::notify::run(fx.conn(), d("2026-10-26")).unwrap();
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'loyalty_expiring'"),
        1
    );
    crate::notify::run(fx.conn(), d("2026-11-14")).unwrap();
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'loyalty_expiring'"),
        2
    );
    assert_eq!(
        fx.count("SELECT count(*) FROM alert WHERE kind = 'card_cashback_missing'"),
        1,
        "pontos que não vieram"
    );
}

#[test]
fn redemption_after_a_lot_expired_does_not_use_it() {
    let mut fx = Fx::new();
    let p = program(&mut fx, "Banco Pontos", LoyaltyKind::Bank, None);
    // Lote A venceu em junho sem uso; o resgate de setembro usa o lote B.
    entry(
        &mut fx,
        &p,
        "2026-01-10",
        1_000,
        EntryKind::Earn,
        Some("2026-06-30"),
    );
    entry(&mut fx, &p, "2026-02-10", 1_000, EntryKind::Earn, None);
    entry(&mut fx, &p, "2026-09-01", 1_000, EntryKind::Redeem, None);
    let v = loyalty::get(fx.conn(), &p, d("2026-10-01")).unwrap();
    assert_eq!(v.balance, 0, "A venceu e B foi resgatado");

    // Com a expiração de A registrada depois do vencimento, o saldo é o mesmo (sem contar duas
    // vezes).
    entry(&mut fx, &p, "2026-07-01", 1_000, EntryKind::Expire, None);
    entry(
        &mut fx,
        &p,
        "2026-09-05",
        500,
        EntryKind::Earn,
        Some("2027-09-05"),
    );
    let v = loyalty::get(fx.conn(), &p, d("2026-10-01")).unwrap();
    assert_eq!(v.balance, 500);
    assert_eq!(v.expiring_90d, 0);
    assert_eq!(v.next_expiry.as_deref(), Some("2027-09-05"));
}
