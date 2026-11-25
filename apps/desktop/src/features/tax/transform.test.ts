import { describe, expect, it } from 'vitest';

import type { Darf } from '@/api/generated/Darf';
import type { TaxMonth } from '@/api/generated/TaxMonth';
import type { TaxYear } from '@/api/generated/TaxYear';

import {
  activeMonths,
  copyText,
  darfOverdue,
  declarationLines,
  defaultMonth,
  lossesAtYearEnd,
  poolResult,
  rateLabel,
  yearOptions,
} from './transform';

function month(m: string, over: Partial<TaxMonth> = {}): TaxMonth {
  return {
    month: m,
    member_id: null,
    categories: [],
    pools: [],
    stock_sales: 0,
    stock_exempt: true,
    tax: 0,
    irrf: 0,
    irrf_estimated: false,
    irrf_carried_in: 0,
    irrf_carried_out: 0,
    irrf_day_trade_carried_out: 0,
    due: 0,
    carried_in: 0,
    darf_amount: 0,
    carried_out: 0,
    due_date: `${m}-28`,
    revenue_code: '6015',
    status: 'no_activity',
    darf_id: null,
    realized: [],
    notes: [],
    crypto: null,
    incomplete: false,
    ...over,
  };
}

function year(months: TaxMonth[], over: Partial<TaxYear> = {}): TaxYear {
  return {
    year: 2026,
    member_id: null,
    months,
    loss_common: 0,
    loss_day_trade: 0,
    loss_fii: 0,
    irrf_lei_11033_left: 0,
    irrf_day_trade_left: 0,
    total_tax: 0,
    total_darf: 0,
    exempt_stock_gains: 0,
    dividends: [],
    dividends_over_limit: [],
    fii_income: [],
    jcp: [],
    holdings: [],
    foreign: {
      realized: 0,
      income: 0,
      tax_paid_abroad: 0,
      tax_credit: 0,
      tax_without_credit: 0,
      loss_carried_in: 0,
      loss_carried_out: 0,
      estimated_tax: 0,
    },
    disclaimer: '',
    ...over,
  };
}

describe('IR transformations', () => {
  it('values to paste into the return and tax rates', () => {
    expect(copyText(123_456)).toBe('1.234,56');
    expect(copyText(-5)).toBe('−0,05');
    expect(copyText(0)).toBe('0,00');
    expect(rateLabel('0.15')).toBe('15%');
    expect(rateLabel('0.2')).toBe('20%');
  });

  it('active months and the month open by default (the last one to pay without a DARF)', () => {
    const y = year([
      month('2026-01', { status: 'no_tax' }),
      month('2026-02', { status: 'to_pay', darf_id: 'd1' }),
      month('2026-03'),
      month('2026-04', { status: 'to_pay' }),
      month('2026-05', { carried_in: 500 }),
      month('2026-06', { status: 'no_tax' }),
    ]);
    expect(activeMonths(y).map((m) => m.month)).toEqual([
      '2026-01',
      '2026-02',
      '2026-04',
      '2026-05',
      '2026-06',
    ]);
    expect(defaultMonth(y)).toBe('2026-04');
    expect(defaultMonth(year([month('2026-01', { status: 'no_tax' })]))).toBe('2026-01');
    expect(defaultMonth(year([]))).toBeNull();
  });

  it('result by group and losses on 12/31', () => {
    const m = month('2026-02', {
      pools: [
        {
          pool: 'fii',
          result: -300,
          loss_before: 0,
          loss_used: 0,
          base: 0,
          rate: '0.2',
          tax: 0,
          loss_after: 300,
        },
      ],
    });
    expect(poolResult(m, 'fii')).toBe(-300);
    expect(poolResult(m, 'common')).toBe(0);
    expect(lossesAtYearEnd(year([], { loss_fii: 300 }))).toEqual([
      { pool: 'common', amount: 0 },
      { pool: 'day_trade', amount: 0 },
      { pool: 'fii', amount: 300 },
    ]);
  });

  it('overdue DARF only when open after the due date', () => {
    const d: Darf = {
      id: 'd',
      member_id: null,
      period: '2026-05',
      revenue_code: '6015',
      principal: 100,
      fine: 0,
      interest: 0,
      total: 100,
      due_date: '2026-06-30',
      status: 'pending',
      paid_at: null,
      txn_id: null,
      notes: null,
    };
    expect(darfOverdue(d, '2026-07-01')).toBe(true);
    expect(darfOverdue(d, '2026-06-30')).toBe(false);
    expect(darfOverdue({ ...d, status: 'paid' }, '2026-07-01')).toBe(false);
  });

  it('return lines add up the income per schedule', () => {
    const lines = declarationLines(
      year([], {
        dividends: [
          { asset_id: 'a', ticker: 'ITUB4', name: 'Itaú', amount: 1_000 },
          { asset_id: 'b', ticker: 'PETR4', name: 'Petrobras', amount: 2_500 },
        ],
        jcp: [{ asset_id: 'a', ticker: 'ITUB4', name: 'Itaú', amount: 700 }],
        exempt_stock_gains: 21_457,
      }),
    );
    expect(lines.map((l) => [l.code, l.amount])).toEqual([
      ['09', 3_500],
      ['20', 21_457],
      [null, 0],
      ['10', 700],
    ]);
    expect(yearOptions('2026-10-14', 3)).toEqual([2026, 2025, 2024]);
  });
});
