//! Cards, phase 6: cost × return of each card over 12 months and the simulation concentrating
//! the family's purchases on a single card.

use super::annual_fee::{AnnualFeeInput, WaiverKind};
use super::rewards::{RewardKind, RewardProgramInput};
use super::*;
use crate::transactions::TxnKind;
use crate::transactions::testkit::{Fx, d};

fn cashback(fx: &mut Fx, card: &str, rate: &str) {
    let input = RewardProgramInput {
        account_id: card.into(),
        kind: RewardKind::Cashback,
        base_rate: rate.into(),
        rate_currency: None,
        cycle_cap: None,
        destination: None,
        destination_account_id: None,
        payout_days: None,
        starts_on: Some("2026-01-01".into()),
        notes: None,
        multipliers: None,
        loyalty_program_id: None,
    };
    fx.write(|tx| rewards::set(tx, &input, d("2026-01-01")))
        .unwrap();
}

#[test]
fn cost_and_return_per_card_and_concentration() {
    let mut fx = Fx::new();
    let premium = fx.card("Premium", 3, 10, None);
    let basic = fx.card("Básico", 3, 10, None);
    cashback(&mut fx, &premium, "0.02");
    let fee = AnnualFeeInput {
        account_id: premium.clone(),
        amount: 120_000,
        installments: Some(12),
        anniversary_month: None,
        waiver_kind: Some(WaiverKind::Spend),
        waiver_threshold: Some(500_000),
        waiver_discount: None,
        starts_on: Some("2026-01-01".into()),
        notes: None,
    };
    fx.write(|tx| annual_fee::set(tx, &fee, d("2026-01-01")))
        .unwrap();
    // Premium: R$ 3,000 in purchases, 2 annual-fee installments, interest, IOF and R$ 60 cashback.
    fx.txn(&premium, "2026-05-10", -300_000, "Loja");
    fx.txn(&premium, "2026-05-11", -10_000, "ANUIDADE 5/12");
    fx.txn(&premium, "2026-06-11", -10_000, "ANUIDADE 6/12");
    fx.txn(&premium, "2026-06-12", -4_000, "JUROS ROTATIVO");
    fx.txn(&premium, "2026-06-13", -1_000, "IOF COMPRA INTERNACIONAL");
    let mut cb = Fx::input(&premium, "2026-06-20", 6_000, "CASHBACK");
    cb.kind = Some(TxnKind::Income);
    fx.txn_with(cb).unwrap();
    // Basic: R$ 7,000 in purchases, no program and no annual fee.
    fx.txn(&basic, "2026-07-10", -700_000, "Mercado");

    let r = value::report(fx.conn(), d("2026-10-01")).unwrap();
    let p = r.cards.iter().find(|c| c.account_id == premium).unwrap();
    assert_eq!(
        (p.spend, p.annual_fee, p.charges, p.iof, p.cashback),
        (300_000, 20_000, 4_000, 1_000, 6_000)
    );
    assert_eq!(p.net, 6_000 - 20_000 - 4_000 - 1_000);
    let b = r.cards.iter().find(|c| c.account_id == basic).unwrap();
    assert_eq!((b.spend, b.net), (700_000, 0));
    assert_eq!(r.total_spend, 1_000_000);
    // All on Premium: 2% of R$ 10,000 = R$ 200; average of R$ 833/month does not waive the fee.
    let s = &r.scenarios[0];
    assert_eq!(s.account_id, premium);
    assert_eq!(
        (s.reward, s.annual_fee, s.net),
        (Some(20_000), 120_000, Some(-100_000))
    );
}
