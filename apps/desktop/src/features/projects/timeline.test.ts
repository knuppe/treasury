import { describe, expect, it } from 'vitest';

import type { ProjectEntry } from '@/api/generated/ProjectEntry';
import type { ProjectView } from '@/api/generated/ProjectView';
import { projectTimelineOption } from '@/charts/projectTimelineOption';
import { readChartTheme } from '@/charts/theme';

import { buildTimeline, projectStats } from './timeline';

const entry = (
  date: string,
  amount: number,
  flow: 'income' | 'expense',
  d = 'x',
): ProjectEntry => ({
  source: 'item',
  id: `${date}-${amount}`,
  project_id: 'p',
  date,
  amount,
  flow,
  description: d,
  category_id: null,
  category_path: null,
  account_id: null,
  account_name: null,
  currency: 'BRL',
  notes: null,
  planned: false,
});

const project = (p: Partial<ProjectView>): ProjectView => ({
  id: 'p',
  name: 'Obra',
  physical_asset_id: null,
  physical_asset_name: null,
  status: 'active',
  budget: null,
  start_date: null,
  end_date: null,
  notes: null,
  spent: 0,
  income: 0,
  net: 0,
  txn_count: 0,
  item_count: 0,
  first_date: null,
  last_date: null,
  remaining: null,
  by_category: [],
  income_by_category: [],
  improvement_ids: [],
  created_at: '',
  updated_at: '',
  ...p,
});

describe('project timeline', () => {
  it('goes month by month from the start to today, with cumulative totals and result', () => {
    const entries = [
      entry('2026-07-10', -400_000, 'expense', 'Cimento'),
      entry('2026-08-02', -250_000, 'expense', 'Pedreiro'),
      entry('2026-08-20', 50_000, 'expense', 'Estorno'),
      entry('2026-09-15', 100_000, 'income', 'Venda de sobras'),
    ];
    const t = buildTimeline(
      { start_date: '2026-06-20', end_date: null, status: 'active' },
      entries,
      '2026-10-14',
    )!;
    expect(t.grain).toBe('month');
    expect(t.periods).toEqual(['2026-06', '2026-07', '2026-08', '2026-09', '2026-10']);
    expect(t.expense).toEqual([0, 400_000, 200_000, 0, 0]);
    expect(t.income).toEqual([0, 0, 0, 100_000, 0]);
    expect(t.cumExpense).toEqual([0, 400_000, 600_000, 600_000, 600_000]);
    expect(t.cumNet).toEqual([0, -400_000, -600_000, -500_000, -500_000]);

    const p = project({
      start_date: '2026-06-20',
      spent: 600_000,
      income: 100_000,
      budget: 1_000_000,
      remaining: 400_000,
    });
    const s = projectStats(p, entries, t, '2026-10-14');
    expect(s.biggestExpense?.description).toBe('Cimento');
    expect(s.biggestIncome?.description).toBe('Venda de sobras');
    expect(s.peak).toEqual({ period: '2026-07', value: 400_000 });
    expect(s.budgetUsed).toBeCloseTo(0.6);
    expect(s.months).toBe(4);
    expect(s.monthlyPace).toBeGreaterThan(140_000);
    // R$ 4k remaining at ~R$ 1.5k/month: ends three months after today.
    expect(s.budgetEnds).toBe('2027-01');
  });

  it('short project goes day by day and stops at the last entry if completed', () => {
    const entries = [
      entry('2026-05-01', -30_000, 'expense'),
      entry('2026-05-03', 80_000, 'income'),
    ];
    const t = buildTimeline(
      { start_date: null, end_date: '2026-05-31', status: 'done' },
      entries,
      '2026-10-14',
    )!;
    expect(t.grain).toBe('day');
    expect(t.periods).toEqual(['2026-05-01', '2026-05-02', '2026-05-03']);
    expect(t.cumNet).toEqual([-30_000, -30_000, 50_000]);
    expect(
      buildTimeline({ start_date: null, end_date: null, status: 'active' }, [], '2026-10-14'),
    ).toBeNull();
    const s = projectStats(project({ status: 'done', spent: 30_000 }), entries, t, '2026-10-14');
    expect(s.budgetEnds).toBeNull();
    expect(s.peak).toEqual({ period: '2026-05-01', value: 30_000 });
  });

  it('the cumulative chart has the budget line; the per-period one has bars', () => {
    const t = buildTimeline(
      { start_date: null, end_date: null, status: 'done' },
      [entry('2026-01-10', -100, 'expense'), entry('2026-04-10', 50, 'income')],
      '2026-10-14',
    )!;
    const theme = readChartTheme('dark', false);
    const acc = projectTimelineOption(theme, {
      ...t,
      mode: 'acumulado',
      budget: 500,
      currency: 'BRL',
    });
    const series = acc.series as { type: string; name: string; markLine?: unknown }[];
    expect(series.map((s) => `${s.type}:${s.name}`)).toEqual([
      'line:Despesa acumulada',
      'line:Receita acumulada',
      'line:Resultado',
    ]);
    expect(series[0]!.markLine).toBeTruthy();
    const per = projectTimelineOption(theme, {
      ...t,
      mode: 'periodo',
      budget: 500,
      currency: 'BRL',
    });
    expect((per.series as { type: string }[]).map((s) => s.type)).toEqual(['bar', 'bar', 'line']);
  });
});
