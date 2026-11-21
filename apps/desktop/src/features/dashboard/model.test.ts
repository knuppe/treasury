import { describe, expect, it } from 'vitest';

import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { Dashboard } from '@/api/generated/Dashboard';
import type { MonthFlow } from '@/api/generated/MonthFlow';

import { budgetPace, composeDashboard, savingsRate } from './model';

const flow = (month: string, income: number, expense: number): MonthFlow => ({
  month,
  income,
  expense,
  net: income - expense,
  savings_rate: income ? (income - expense) / income : null,
  missing_fx: [],
  estimated: false,
});

const dashboard = (): Dashboard => ({
  month: '2026-10',
  basis: 'accrual',
  net_worth: {
    as_of: '2026-10-14',
    base_currency: 'BRL',
    assets: 0,
    liabilities: 0,
    net: 0,
    investments: 0,
    physical: 0,
    debts: 0,
    accounts: [],
    missing_fx: [],
    estimated: false,
  },
  flow: flow('2026-10', 100, 80),
  flow_history: [flow('2026-09', 100, 50), flow('2026-10', 100, 80)],
  savings_rate_12m: 0.35,
  top_categories: [],
  upcoming: [],
  budget_planned: null,
  budget_actual: 80,
  uncategorized_count: 2,
  currency: 'BRL',
});

describe('dashboard', () => {
  it('savings rate of several months from the totals', () => {
    expect(savingsRate([flow('a', 100, 50), flow('b', 300, 150)])).toBe(0.5);
    expect(savingsRate([flow('a', 0, 10)])).toBeNull();
  });

  it('on the default basis it uses the dashboard; on the other, the other basis queries', () => {
    const d = dashboard();
    const same = composeDashboard(d, 'accrual', { flowHistory: [flow('2026-10', 1, 1)] });
    expect(same.flow.expense).toBe(80);
    expect(same.savingsRate12m).toBe(0.35);
    const cash = composeDashboard(d, 'cash', {
      flowHistory: [flow('2026-09', 200, 100), flow('2026-10', 200, 150)],
    });
    expect(cash.basis).toBe('cash');
    expect(cash.defaultBasis).toBe('accrual');
    expect(cash.flow.expense).toBe(150);
    expect(cash.savingsRate12m).toBeCloseTo(0.375);
    // While the other basis loads, shows the default one (and says which it is).
    expect(composeDashboard(d, 'cash', {}).basis).toBe('accrual');
  });

  it('budget pace: expected up to today by the fraction of the month', () => {
    const b = {
      month: '2026-10',
      basis: 'accrual',
      lines: [],
      month_elapsed: 0.5,
      currency: 'BRL',
      missing_fx: [],
      estimated: false,
      totals: {
        planned_expense: 1000,
        actual_expense: 600,
        planned_income: 0,
        actual_income: 0,
        committed_expense: 200,
        committed_income: 0,
        uncategorized_expense: 0,
      },
    } satisfies BudgetStatus;
    expect(budgetPace(b)).toMatchObject({ expectedByNow: 500, ratio: 0.6, committed: 200 });
  });
});
