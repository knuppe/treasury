/** "Este cartão vale a pena?" no mock, como `cards/value.rs` (sem câmbio: só a moeda base). */
import type { CardScenario } from '@/api/generated/CardScenario';
import type { CardValue } from '@/api/generated/CardValue';
import type { CardValueReport } from '@/api/generated/CardValueReport';

import { addMonths, today } from './dates';
import { baseCurrency } from './domain';
import { find, rows } from './store';
import { normalizeDescription } from './text';

const CHARGE = new Set(['juros', 'encargo', 'encargos', 'rotativo', 'multa', 'mora']);

export function cardValueReport(T = today()): CardValueReport {
  const base = baseCurrency();
  const from = addMonths(T, -12);
  const cards = rows('account')
    .filter((a) => a.kind === 'credit_card' && !a.archived)
    .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  const cbCat = rows('category').find((c) => c.name === 'Cashback e recompensas')?.id;
  const byCategory = new Map<string | null, number>();
  const out: CardValue[] = [];
  for (const a of cards) {
    const v: CardValue = {
      account_id: a.id,
      account_name: a.name,
      currency: a.currency,
      spend: 0,
      annual_fee: 0,
      charges: 0,
      iof: 0,
      cashback: 0,
      points: 0,
      points_value: null,
      net: 0,
    };
    for (const t of rows('txn')) {
      if (t.account_id !== a.id || t.date <= from || t.date > T) continue;
      if (t.kind === 'card_payment' || t.status === 'planned') continue;
      const d = normalizeDescription(t.description);
      const words = d.split(/\s+/);
      if (d.includes('anuidade')) v.annual_fee -= t.amount;
      else if (words.includes('iof')) v.iof -= t.amount;
      else if (words.some((w) => CHARGE.has(w))) v.charges -= t.amount;
      else if (t.amount > 0 && ((cbCat && t.category_id === cbCat) || d.includes('cashback')))
        v.cashback += t.amount;
      else if (t.kind === 'expense') {
        v.spend -= t.amount;
        if (a.currency === base)
          byCategory.set(t.category_id, (byCategory.get(t.category_id) ?? 0) - t.amount);
      }
    }
    v.net = v.cashback - v.annual_fee - v.charges - v.iof;
    out.push(v);
  }
  const total = [...byCategory.values()].reduce((s, n) => s + n, 0);
  const scenarios: CardScenario[] = [];
  for (const a of cards) {
    const p = rows('card_reward_program')
      .filter((r) => r.account_id === a.id && r.starts_on <= T && (!r.ends_on || r.ends_on >= T))
      .sort((x, y) => y.starts_on.localeCompare(x.starts_on))[0];
    if (!p) continue;
    const rates = new Map(p.multipliers.map((m) => [m.category_id, Number(m.rate)]));
    let raw = 0;
    for (const [cat, amount] of byCategory) {
      const parent = find('category', cat)?.parent_id ?? null;
      raw +=
        amount * ((cat && rates.get(cat)) || (parent && rates.get(parent)) || Number(p.base_rate));
    }
    const fee = rows('card_annual_fee').find(
      (f) => f.account_id === a.id && f.starts_on <= T && (!f.ends_on || f.ends_on >= T),
    );
    const feeAmount = !fee
      ? 0
      : fee.waiver_kind === 'full'
        ? 0
        : fee.waiver_kind === 'spend' && total / 12 >= (fee.waiver_threshold ?? Infinity)
          ? 0
          : fee.waiver_kind === 'discount'
            ? Math.round(fee.amount * (1 - Number(fee.waiver_discount ?? 0)))
            : fee.amount;
    const reward = p.kind === 'cashback' ? Math.round(raw) : null;
    scenarios.push({
      account_id: a.id,
      account_name: a.name,
      reward,
      points: p.kind === 'points' ? Math.floor(raw / 100) : null,
      annual_fee: feeAmount,
      net: reward == null ? null : reward - feeAmount,
    });
  }
  scenarios.sort((x, y) => (y.net ?? -Infinity) - (x.net ?? -Infinity));
  return { from, to: T, currency: base, cards: out, total_spend: total, scenarios };
}
