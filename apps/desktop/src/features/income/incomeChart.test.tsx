/**
 * 02 · Família › Income by month: the months with a payslip show as actual, before the forecast
 * from the latest payslip, with the difference from the forecast in the tooltip (demo mode).
 */
import { screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { IncomeMonth } from '@/api/generated/IncomeMonth';
import type { IncomeSummary } from '@/api/generated/IncomeSummary';
import { applyLocale } from '@/i18n';
import { chartOption, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

import { incomeRows } from './incomeChart';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  clearCharts();
  await applyLocale('pt-BR');
});

type Formatter = (p: { dataIndex: number }[]) => string;

describe('02 · Família › Actual and forecast income', () => {
  it('shows the actual from the payslips before the forecast, with the difference', async () => {
    renderApp('unlocked', {}, '#/familia');
    expect(await screen.findByText(/Renda por mês: real e prevista/)).toBeTruthy();
    expect(screen.getByText(/Previsto: último holerite \(/)).toBeTruthy();
    const option = chartOption(/^Renda por mês: real e prevista/);
    const axis = (option.xAxis as { data: string[] }).data;
    // Five months of payslips, the current month (still without a payslip) already forecast and twelve of forecast.
    expect(axis).toHaveLength(18);
    const series = option.series as { name: string; type: string }[];
    expect(series.map((s) => s.type)).toEqual(['bar', 'bar', 'bar', 'line']);
    const fmt = (option.tooltip as { formatter: Formatter }).formatter;
    // The oldest month is before the raise: the actual is below the forecast.
    const first = fmt([{ dataIndex: 0 }]);
    expect(first).toContain('Real');
    expect(first).toContain('Diferença do previsto');
    expect(first).toMatch(/−R\$\s?1\.150,00|-R\$\s?1\.150,00/);
    const recent = fmt([{ dataIndex: 4 }]);
    expect(recent).toContain('igual');
    const forecast = fmt([{ dataIndex: 5 }]);
    expect(forecast).toContain('Previsto');
    expect(forecast).not.toContain('Diferença do previsto');
  });

  it('in English', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/familia');
    expect(await screen.findByText(/Income by month: actual and forecast/)).toBeTruthy();
    expect(screen.getByText(/Forecast: latest payslip \(/)).toBeTruthy();
  });
});

const month = (m: string, gross = 1_000_000): IncomeMonth => ({
  month: m,
  gross,
  social_security: 0,
  income_tax: 0,
  other_deductions: 0,
  net: gross,
  extras: [],
});

/** Today in November: payslips from April to September, October not in yet. */
function novemberSummary(current: IncomeMonth | null): IncomeSummary {
  const real = ['2026-04', '2026-05', '2026-06', '2026-07', '2026-08', '2026-09'];
  return {
    from: '2026-12',
    months: 3,
    currency: 'BRL',
    members: [],
    monthly: ['2026-12', '2027-01', '2027-02'].map((m) => month(m)),
    next: month('next'),
    last_12m: month('12m'),
    actual: real.map((m) => ({ real: month(m), expected: month(m) })),
    payslips: [],
    forecast_base_month: '2026-09',
    current_month: current,
    notes: [],
  };
}

describe('Income by month: months without a payslip', () => {
  it('does not skip from September to December: October empty and November forecast', () => {
    const rows = incomeRows(novemberSummary(month('2026-11', 1_500_000)));
    expect(rows.map((r) => [r.month, r.kind])).toEqual([
      ['2026-04', 'real'],
      ['2026-05', 'real'],
      ['2026-06', 'real'],
      ['2026-07', 'real'],
      ['2026-08', 'real'],
      ['2026-09', 'real'],
      ['2026-10', 'empty'],
      ['2026-11', 'forecast'],
      ['2026-12', 'forecast'],
      ['2027-01', 'forecast'],
      ['2027-02', 'forecast'],
    ]);
  });

  it('without a forecast for the current month, only it drops; the past month without a payslip stays', () => {
    const rows = incomeRows(novemberSummary(null));
    expect(rows.map((r) => r.month)).toEqual([
      '2026-04',
      '2026-05',
      '2026-06',
      '2026-07',
      '2026-08',
      '2026-09',
      '2026-10',
      '2026-12',
      '2027-01',
      '2027-02',
    ]);
    expect(rows.find((r) => r.month === '2026-10')?.kind).toBe('empty');
  });
});
