/**
 * Total effective cost (CET) in the mock, mirror of `debt/cet.rs` (Res. CMN 4.881/2020): the
 * rate that makes the installments of the contract as signed worth the amount released (the
 * principal minus the upfront costs), days / 365; the US APR of Regulation Z (monthly unit
 * period, odd days over 30, × 12); and the realized CET of what was paid.
 */
import type { CetReport } from '@/api/generated/CetReport';
import type { Installment } from '@/api/generated/Installment';
import type { UnrecordedDrop } from '@/api/generated/UnrecordedDrop';
import type { UpfrontCosts } from '@/api/generated/UpfrontCosts';

import { addMonths, diffDays } from '../ledger/dates';
import { xirr, type Row } from './schedule';

const DROP_TOLERANCE = 5_000;

export const upfrontTotal = (u: UpfrontCosts): number => u.iof + u.fees + u.insurance;

/** Percentage with two decimals, a trailing 5 to the even digit (NBR 5891). */
export function roundPct(rate: number): string {
  const v = rate * 100;
  const scaled = v * 100;
  const floor = Math.floor(scaled);
  const diff = scaled - floor;
  const r =
    Math.abs(diff - 0.5) < 1e-9 ? (floor % 2 === 0 ? floor : floor + 1) : Math.round(scaled);
  return (r / 100).toFixed(2);
}

/** Art. 4º: annual rate that makes `flows` (paid by the borrower) worth `released`. */
export function cetRate(
  released: number,
  release: string,
  flows: readonly { date: string; amount: number }[],
): number | null {
  return xirr([{ date: release, amount: -released }, ...flows]);
}

function monthsAndDays(from: string, to: string): [number, number] {
  let t = 0;
  while (addMonths(to, -(t + 1)) >= from) t += 1;
  return [t, diffDays(addMonths(to, -t), from)];
}

/** US APR (Regulation Z, Appendix J) for monthly payments: 12 × the unit-period rate. */
export function usApr(
  released: number,
  release: string,
  flows: readonly { date: string; amount: number }[],
): number | null {
  if (released <= 0 || !flows.length) return null;
  const periods = flows.map((f) => {
    const [t, days] = monthsAndDays(release, f.date);
    return { t, f: days / 30, a: f.amount };
  });
  const pv = (i: number) => periods.reduce((s, p) => s + p.a / ((1 + p.f * i) * (1 + i) ** p.t), 0);
  let lo = -0.99;
  let hi = 10;
  if (Math.sign(pv(lo) - released) === Math.sign(pv(hi) - released)) return null;
  for (let n = 0; n < 200; n++) {
    const mid = (lo + hi) / 2;
    if (pv(mid) > released) lo = mid;
    else hi = mid;
  }
  return (12 * (lo + hi)) / 2;
}

/** CET of the contract as signed, from its schedule with the floating index at zero. */
export function contractCet(
  rows: readonly Row[],
  release: string,
  principal: number,
  upfront: UpfrontCosts,
  indexLabel: string | null,
  contracted: string | null,
): CetReport | null {
  const last = rows.at(-1);
  if (!last) return null;
  const released = principal - upfrontTotal(upfront);
  const flows = rows.map((r) => ({ date: r.due_date, amount: r.total }));
  const annual = cetRate(released, release, flows);
  if (annual == null) return null;
  const sum = (f: (r: Row) => number) => rows.reduce((s, r) => s + f(r), 0);
  const annualPct = roundPct(annual);
  const typed = contracted ? Number(contracted.trim().replace(',', '.')) : NaN;
  return {
    annual,
    annual_pct: annualPct,
    monthly_pct: roundPct((1 + annual) ** (1 / 12) - 1),
    us_apr: usApr(released, release, flows),
    principal,
    upfront,
    released,
    amortization: sum((r) => r.amortization),
    interest: sum((r) => r.interest),
    insurance: sum((r) => r.insurance),
    fees: sum((r) => r.fees),
    total_paid: sum((r) => r.total) + upfrontTotal(upfront),
    installments: rows.length,
    release_date: release,
    last_date: last.due_date,
    days: diffDays(last.due_date, release),
    index_excluded: indexLabel,
    contracted_gap_pp: Number.isFinite(typed) ? typed - Number(annualPct) : null,
  };
}

/** Realized CET: installments paid, extra payments, unexplained balance drops and the balance
 * after the last installment paid. */
export function realizedCet(
  released: number,
  principal: number,
  contractDate: string,
  inst: readonly Installment[],
): { rate: number | null; drops: UnrecordedDrop[] } {
  const flows: { date: string; amount: number }[] = [];
  const drops: UnrecordedDrop[] = [];
  let prev: [number, number] = [0, principal];
  let last: [string, number] | null = null;
  for (const i of inst.filter((x) => x.paid)) {
    const date = i.paid_date ?? i.due_date;
    let drop =
      prev[0] + 1 === i.number
        ? prev[1] + i.adjustment - i.amortization - i.extra_payment - i.balance_after
        : 0;
    if (drop > DROP_TOLERANCE) drops.push({ number: i.number, date, amount: drop });
    else drop = 0;
    flows.push({ date, amount: (i.paid_amount ?? i.total) + i.extra_payment + drop });
    prev = [i.number, i.balance_after];
    last = [i.due_date, i.balance_after];
  }
  if (!last) return { rate: null, drops };
  if (last[1] > 0)
    flows.push({ date: last[0] > contractDate ? last[0] : contractDate, amount: last[1] });
  return { rate: cetRate(released, contractDate, flows), drops };
}
