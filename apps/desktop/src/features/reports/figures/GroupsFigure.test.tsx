/** Relatórios › 12 · Grupos: bars per group (share of income or amount), trend and table. */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import { mockCall } from '@/api/mock';
import { todayIso } from '@/lib/format';
import { monthOf, shiftMonth } from '@/lib/months';
import { chartOption, chartProps, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => clearCharts());

type Series = {
  type: string;
  name: string;
  stack?: string;
  data: unknown[];
  markLine?: { data: { yAxis: number }[] };
};

const series = (label: RegExp) => chartOption(label).series as Series[];
const value = (d: unknown) =>
  d && typeof d === 'object' && 'value' in d ? (d as { value: number | null }).value : d;

describe('Relatórios › 12 · Grupos', () => {
  it('stacked bars per group as share of income and as amount, trend with targets and drill-down', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios?v=grupos&p=12m');
    expect(await screen.findByText(/Fig\. 12/)).toBeTruthy();
    const end = monthOf(todayIso());
    const data = (await mockCall('reports.spend_groups', {
      from: shiftMonth(end, -11),
      to: end,
    })) as SpendGroupsReport;

    // Share of income (default): four stacked series, fractions of each month's income.
    const bars = series(/^Grupos de gasto por mês/);
    expect(bars.map((s) => s.name)).toEqual(['Essenciais', 'Dívidas', 'Futuro', 'Não essencial']);
    expect(bars.every((s) => s.type === 'bar' && s.stack === 'grupos')).toBe(true);
    expect(bars[0]!.data.map(value)).toEqual(data.months.map((m) => m.groups[0]!.share_of_income));

    // As amount: cents, with income as a reference line.
    await user.click(screen.getByRole('radio', { name: 'Valor' }));
    await waitFor(() =>
      expect(series(/^Grupos de gasto por mês/)[2]!.data.map(value)).toEqual(
        data.months.map((m) => m.groups[2]!.amount),
      ),
    );
    expect(series(/^Grupos de gasto por mês/).at(-1)!.name).toBe('Renda');

    // Clicking a segment opens the month's transactions in the group's categories.
    chartProps(/^Grupos de gasto por mês/).onClick?.({
      name: '',
      seriesName: 'Essenciais',
      dataIndex: 11,
      value: 0,
      data: {},
    });
    await waitFor(() => expect(window.location.hash).toMatch(/^#\/lancamentos\?/));
    expect(window.location.hash).toContain('cat-moradia');
    expect(window.location.hash).not.toContain('cat-lazer');
  });

  it('trend of the share of income with the dashed target and the table of the largest categories', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios?v=grupos&p=12m');
    await screen.findByText(/Fig\. 12/);
    await user.click(screen.getByRole('radio', { name: 'Evolução' }));
    await waitFor(() => expect(series(/^Parte da renda de cada grupo/)).toHaveLength(4));
    const lines = series(/^Parte da renda de cada grupo/);
    expect(lines.every((s) => s.type === 'line')).toBe(true);
    expect(lines[0]!.markLine?.data[0]?.yAxis).toBe(0.5);
    expect(lines[1]!.markLine).toBeUndefined();

    const table = screen.getByRole('grid', { name: 'Maiores categorias por grupo' });
    expect(within(table).getAllByText('Aportes').length).toBeGreaterThan(0);
    expect(within(table).getAllByText('Essenciais').length).toBeGreaterThan(0);
    // Chart table view: one month per row, with the four groups.
    await user.click(screen.getByRole('button', { name: 'Ver tabela' }));
    const months = screen.getByRole('grid', { name: 'Grupos de gasto por mês' });
    expect(within(months).getAllByRole('row').length).toBeGreaterThan(12);
  });
});
