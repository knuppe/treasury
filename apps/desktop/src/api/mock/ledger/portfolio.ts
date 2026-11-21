/**
 * Carteira fictícia coerente com o livro: um lote inicial na abertura e uma compra por mês
 * (os lançamentos `investment` da corretora). O valor de mercado segue um índice mensal
 * determinístico; entra no patrimônio como `investments` (como o núcleo).
 */
import type { ClassSummary } from '@/api/generated/ClassSummary';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import { daysBetween, monthIndex, monthsBetween } from '@/lib/months';

import { BASE } from './fixture';
import { getLedger } from './ledger';

/** Custo do lote inicial (posição que já existia na abertura do livro). */
export const INITIAL_COST = 14_500_000;
const IPCA_12M = 0.0452;

function monthlyReturn(month: string): number {
  const i = monthIndex(month);
  return 0.008 + 0.024 * Math.sin(i * 1.7) * Math.cos(i * 0.6);
}

/** Fator de valorização entre o fim do mês `from` e o fim do mês `to`. */
function growth(from: string, to: string): number {
  if (to <= from) return 1;
  return monthsBetween(from, to)
    .slice(1)
    .reduce((g, m) => g * (1 + monthlyReturn(m)), 1);
}

interface Lot {
  date: string;
  cost: number;
}

function lots(asOf: string): Lot[] {
  const { start, txns } = getLedger();
  if (asOf < start) return [];
  const out: Lot[] = [{ date: start, cost: INITIAL_COST }];
  for (const t of txns) {
    if (
      t.account_id === 'acc-xp' &&
      t.kind === 'investment' &&
      t.amount < 0 &&
      t.status !== 'planned' &&
      t.date <= asOf
    ) {
      out.push({ date: t.date, cost: -t.amount });
    }
  }
  return out;
}

export function investmentsAt(asOf: string): number {
  const month = asOf.slice(0, 7);
  return Math.round(lots(asOf).reduce((s, l) => s + l.cost * growth(l.date.slice(0, 7), month), 0));
}

/** XIRR por Newton sobre fluxos (data, valor); `null` se não convergir. */
export function xirr(flows: readonly { date: string; amount: number }[]): number | null {
  if (flows.length < 2) return null;
  const t0 = flows[0]!.date;
  const years = flows.map((f) => daysBetween(t0, f.date) / 365);
  let r = 0.1;
  for (let k = 0; k < 60; k++) {
    let f = 0;
    let df = 0;
    flows.forEach((fl, i) => {
      const y = years[i]!;
      const d = (1 + r) ** y;
      f += fl.amount / d;
      df -= (y * fl.amount) / (d * (1 + r));
    });
    if (Math.abs(df) < 1e-9) return null;
    const next = r - f / df;
    if (!Number.isFinite(next)) return null;
    if (Math.abs(next - r) < 1e-9) return next;
    r = Math.max(next, -0.99);
  }
  return r;
}

const CLASSES: { cls: ClassSummary['class']; label: string; weight: number; target: number }[] = [
  { cls: 'stock_br', label: 'Ações', weight: 0.38, target: 0.35 },
  { cls: 'fii', label: 'FIIs', weight: 0.19, target: 0.2 },
  { cls: 'fixed_income', label: 'Renda fixa', weight: 0.31, target: 0.3 },
  { cls: 'stock_us', label: 'Ações EUA', weight: 0.12, target: 0.15 },
];

export function portfolioSummary(asOf: string): PortfolioSummary {
  const { txns } = getLedger();
  const ls = lots(asOf);
  const cost = ls.reduce((s, l) => s + l.cost, 0);
  const market = investmentsAt(asOf);
  const income = txns
    .filter(
      (t) =>
        t.account_id === 'acc-xp' &&
        t.kind === 'income' &&
        t.status !== 'planned' &&
        t.date <= asOf,
    )
    .reduce((s, t) => s + t.amount, 0);
  const realized = 0;
  const flows = [
    ...ls.map((l) => ({ date: l.date, amount: -l.cost })),
    ...txns
      .filter(
        (t) =>
          t.account_id === 'acc-xp' &&
          t.kind === 'income' &&
          t.status !== 'planned' &&
          t.date <= asOf,
      )
      .map((t) => ({ date: t.date, amount: t.amount })),
    { date: asOf, amount: market },
  ].sort((a, b) => a.date.localeCompare(b.date));
  const rate = market > 0 ? xirr(flows) : null;
  return {
    as_of: asOf,
    base_currency: BASE,
    deflator: 'ipca',
    market_value: market,
    cost,
    unrealized: market - cost,
    realized,
    income,
    total_result: market - cost + realized + income,
    // Aproximação do mock: o custo perde o IPCA de 12 meses.
    real_result: Math.round(market - cost * (1 + IPCA_12M) + realized + income),
    inflation_effect: Math.round(cost * IPCA_12M),
    xirr: rate,
    xirr_real: rate == null ? null : (1 + rate) / (1 + IPCA_12M) - 1,
    by_class: CLASSES.map((c) => ({
      class: c.cls,
      label: c.label,
      market_value_base: Math.round(market * c.weight),
      cost_base: Math.round(cost * c.weight),
      weight: c.weight,
      target_pct: c.target * 100,
      deviation_pp: (c.weight - c.target) * 100,
      out_of_band: Math.abs(c.weight - c.target) > 0.05,
      xirr: null,
    })),
    positions: [],
    missing_prices: [],
    balance_due: [],
    estimated: false,
    concentration_alerts: [],
  };
}
