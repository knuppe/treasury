import { describe, expect, it } from 'vitest';

import type { Installment } from '@/api/generated/Installment';
import type { PrepaySimulation } from '@/api/generated/PrepaySimulation';
import { build, monthlyRate, type Params } from '@/api/mock/debt/schedule';
import { readChartTheme } from '@/charts/theme';

import { breakdownOption, visibleParts } from './chartOptions';
import {
  balanceSeries,
  breakdown,
  defaultBreakdownMode,
  firstPaymentPreview,
  installmentStatus,
  linkableInstallments,
  progress,
  rateText,
  simulationSummary,
  suggestedInstallment,
  termText,
  todayIndex,
  totals,
} from './transform';

function inst(n: number, over: Partial<Installment> = {}): Installment {
  return {
    id: `i${n}`,
    version: 1,
    version_number: n,
    number: n,
    due_date: `2026-${String(n).padStart(2, '0')}-10`,
    amortization: 1_000,
    interest: 500,
    adjustment: 0,
    insurance: 0,
    insurance_mip: 0,
    insurance_dfi: 0,
    fees: 0,
    total: 1_500,
    balance_after: 10_000 - n * 1_000,
    extra_payment: 0,
    estimated: false,
    paid: false,
    paid_date: null,
    paid_amount: null,
    txn_id: null,
    txn_date: null,
    txn_description: null,
    txn_account_id: null,
    txn_account_name: null,
    overdue: false,
    index_factor: null,
    business_days: null,
    ...over,
  };
}

const base: Params = {
  principal: 12_000_000,
  annualRatePct: 12.682503, // ≈ 1% per month
  system: 'price',
  term: 120,
  grace: 0,
  firstDue: '2026-01-10',
  insurance: 0,
  fees: 0,
  indexFactors: [],
  extras: [],
};

describe('debt transforms', () => {
  it('formats rates, terms and progress', () => {
    expect(rateText('9.5')).toBe('9,5%');
    expect(rateText('0.7592', 4)).toBe('0,7592%');
    expect(rateText(null)).toBe('—');
    expect(termText(1)).toBe('1 parcela');
    expect(termText(302)).toBe('302 parcelas (25 anos e 2 meses)');
    expect(progress({ paid_count: 12, remaining_count: 36 })).toBe(0.25);
    expect(progress({ paid_count: 0, remaining_count: 0 })).toBe(0);
  });

  it('installment status: paid, late, next and planned', () => {
    expect(installmentStatus(inst(1, { paid: true }), 'i2')).toBe('paid');
    expect(installmentStatus(inst(2, { overdue: true }), 'i2')).toBe('overdue');
    expect(installmentStatus(inst(3), 'i3')).toBe('next');
    expect(installmentStatus(inst(4), 'i3')).toBe('planned');
  });

  it('outstanding balance starts at the financed amount and marks today', () => {
    const pts = balanceSeries([inst(1), inst(2, { estimated: true })], 10_000, '2025-12-15');
    expect(pts.map((p) => p.balance)).toEqual([10_000, 9_000, 8_000]);
    expect(pts[2]!.estimated).toBe(true);
    expect(todayIndex(pts, '2026-01-20')).toBe(1);
    expect(todayIndex(pts, '2025-01-01')).toBe(-1);
  });

  it('breakdown by year sums the year installments; long contracts group by year', () => {
    const rows = [
      inst(1, { due_date: '2026-11-10', adjustment: 30, insurance: 20, fees: 5 }),
      inst(2, { due_date: '2026-12-10', extra_payment: 5_000 }),
      inst(3, { due_date: '2027-01-10', estimated: true }),
    ];
    const byYear = breakdown(rows, 'ano');
    expect(byYear.map((r) => r.key)).toEqual(['2026', '2027']);
    expect(byYear[0]).toMatchObject({
      amortization: 2_000,
      interest: 1_000,
      adjustment: 30,
      charges: 25,
      total: 3_000,
      extra: 5_000,
      count: 2,
      date: '2026-12-10',
    });
    expect(byYear[1]!.estimated).toBe(true);
    expect(breakdown(rows, 'parcela')).toHaveLength(3);
    expect(defaultBreakdownMode(360)).toBe('ano');
    expect(defaultBreakdownMode(48)).toBe('parcela');
    expect(totals(rows)).toMatchObject({ amortization: 3_000, charges: 25, extra: 5_000 });
  });

  it('breakdown chart: a single axis, parts without value are left out, fixed color per part', () => {
    const rows = breakdown([inst(1), inst(2)], 'parcela');
    expect(visibleParts(rows)).toEqual(['amortization', 'interest']);
    const t = readChartTheme('dark', false);
    const opt = breakdownOption(t, { rows, mode: 'parcela' }) as Record<string, unknown>;
    expect(Array.isArray(opt.yAxis)).toBe(false);
    const series = opt.series as { name: string; stack: string; color: string }[];
    expect(series.map((s) => s.name)).toEqual(['Amortização', 'Juros']);
    expect(new Set(series.map((s) => s.stack)).size).toBe(1);
    expect(series[0]!.color).toBe(t.series[0]);
    expect(series[1]!.color).toBe(t.series[1]);
    // With indexation, the indexation takes the 3rd slot (the "Juros" color does not change).
    const withAdj = breakdown([inst(1, { adjustment: 10 })], 'parcela');
    const s2 = (
      breakdownOption(t, { rows: withAdj, mode: 'parcela' }) as { series: { color: string }[] }
    ).series;
    expect(s2.map((s) => s.color)).toEqual([t.series[0], t.series[1], t.series[2]]);
  });

  it('simulation summary counts the remaining installments from the date', () => {
    const sim: PrepaySimulation = {
      mode: 'reduce_term',
      amount: 1_000_000,
      interest_before: 9_000,
      interest_after: 6_000,
      interest_saved: 3_000,
      months_before: 12,
      months_after: 8,
      next_payment_before: 1_500,
      next_payment_after: 1_500,
      schedule: [1, 2, 3, 4, 5, 6, 7, 8].map((n) => inst(n)),
    };
    expect(simulationSummary(sim, '2026-03-15')).toEqual({
      interestSaved: 3_000,
      remainingBefore: 9,
      remainingAfter: 5,
      monthsSaved: 4,
      nextBefore: 1_500,
      nextAfter: 1_500,
      payoffDate: '2026-08-10',
    });
  });

  it('installments for a payment: without a transaction (or with it), the closest to the date', () => {
    const list = Array.from({ length: 12 }, (_, k) => inst(k + 1));
    list[3] = { ...list[3]!, paid: true, txn_id: 'outro' }; // April: paid by another transaction
    list[4] = { ...list[4]!, paid: true, txn_id: 't1' }; // May: paid by this one
    list[5] = { ...list[5]!, paid: true }; // June: paid without a transaction
    const near = linkableInstallments(list, '2026-05-12', 't1', 4);
    expect(near.map((i) => i.number)).toEqual([3, 5, 6, 7]);
    // Suggested: the open one with the closest due date (the paid ones only when none is open).
    expect(suggestedInstallment(near, '2026-05-12')?.number).toBe(7);
    expect(suggestedInstallment(near, '2026-03-01')?.number).toBe(3);
    const paid = near.map((i) => ({ ...i, paid: true }));
    expect(suggestedInstallment(paid, '2026-05-12')?.number).toBe(5);
    expect(suggestedInstallment([], '2026-05-12')).toBeNull();
  });

  it('first installment preview: Price and SAC, with an interest-only grace period', () => {
    const price = firstPaymentPreview({
      principal: 12_000_000,
      annualRatePct: 12.682503,
      system: 'price',
      term: 120,
      grace: 0,
      charges: 0,
    })!;
    expect(Math.abs(price.first - 172_165)).toBeLessThanOrEqual(1);
    const sac = firstPaymentPreview({
      principal: 12_000_000,
      annualRatePct: 12.682503,
      system: 'sac',
      term: 120,
      grace: 6,
      charges: 1_000,
    })!;
    expect(sac.grace).toBe(121_000);
    expect(sac.first).toBeGreaterThan(sac.last);
    expect(firstPaymentPreview({ ...base, principal: 0, annualRatePct: 1, charges: 0 })).toBeNull();
  });
});

describe('mock schedule (mirror of debt/schedule.rs)', () => {
  it('Price of 120k at 1% per month over 120x ≈ 1,721.65 and pays off', () => {
    expect(monthlyRate(12.682503)).toBeCloseTo(0.01, 7);
    const r = build(base);
    expect(r).toHaveLength(120);
    expect(Math.abs(r[0]!.total - 172_165)).toBeLessThanOrEqual(1);
    expect(r.at(-1)!.balance_after).toBe(0);
    expect(r.reduce((s, x) => s + x.amortization, 0)).toBe(12_000_000);
  });

  it('SAC: constant amortization and decreasing installment', () => {
    const r = build({ ...base, system: 'sac' });
    expect(r[0]).toMatchObject({ amortization: 100_000, interest: 120_000, total: 220_000 });
    expect(r[1]!.total).toBeLessThan(r[0]!.total);
  });

  it('shortening the term before the 1st installment keeps the installment and shortens the contract', () => {
    const before = build(base);
    const r = build({
      ...base,
      extras: [{ afterNumber: 0, amount: 2_000_000, mode: 'reduce_term' }],
    });
    expect(r.length).toBeLessThan(120);
    expect(Math.abs(r[0]!.total - before[0]!.total)).toBeLessThanOrEqual(1);
    const inst = build({
      ...base,
      extras: [{ afterNumber: 0, amount: 2_000_000, mode: 'reduce_installment' }],
    });
    expect(inst).toHaveLength(120);
  });

  it('shortening the term with an index shortens the contract; reducing the installment afterwards keeps the term', () => {
    for (const system of ['sac', 'price'] as const) {
      const p: Params = {
        ...base,
        system,
        indexFactors: Array.from({ length: 720 }, () => [1.003, true] as [number, boolean]),
      };
      const term = build({
        ...p,
        extras: [{ afterNumber: 12, amount: 1_000_000, mode: 'reduce_term' }],
      });
      expect(term.length).toBeLessThan(120);
      const both = build({
        ...p,
        extras: [
          { afterNumber: 12, amount: 1_000_000, mode: 'reduce_term' },
          { afterNumber: 30, amount: 500_000, mode: 'reduce_installment' },
        ],
      });
      expect(Math.abs(both.length - term.length)).toBeLessThanOrEqual(1);
      expect(both[31]!.total).toBeLessThan(term[31]!.total);
      expect(both.at(-1)!.balance_after).toBe(0);
    }
  });
});
