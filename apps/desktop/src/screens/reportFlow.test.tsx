/**
 * Reports › Flow against the mock: Sankey with contributions and surplus, comparison with the previous
 * period (summary, tooltip and table), period switch via arrows, depth and
 * privacy mode.
 */
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCallLog } from '@/api/mock';
import { formatMonthLong, todayIso } from '@/lib/format';
import { monthOf, shiftMonth } from '@/lib/months';
import { useUi } from '@/state/ui';
import { chartOption, chartProps, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const month = monthOf(todayIso());
const last = shiftMonth(month, -1);

beforeEach(() => clearCharts());

function hashSearch(): URLSearchParams {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '');
}

type AnyRec = Record<string, unknown>;
type Tip = (p: { dataType: string; name: string; data: AnyRec }) => string;

/**
 * Waits for a condition on the chart option without `waitFor`: building the option reads the theme from a
 * DOM element, and the mutation would trigger `waitFor` again, endlessly.
 */
async function eventually(check: () => boolean): Promise<void> {
  const ok = () => {
    try {
      return check();
    } catch {
      return false;
    }
  };
  for (let i = 0; i < 40 && !ok(); i++) await act(() => new Promise((r) => setTimeout(r, 50)));
  expect(check()).toBe(true);
}

function sankeyNodes(): string[] {
  const s = (chartOption(/^Fluxo de/).series as AnyRec[])[0]!;
  return (s.data as { name: string }[]).map((n) => n.name);
}

describe('Reports › Flow', () => {
  it('shows where the income goes, with the previous month alongside', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=fluxo&p=mes-anterior');
    expect(await screen.findByText(/Fig\. 01/)).toBeTruthy();
    expect(screen.getByText(/Para onde vai a receita/)).toBeTruthy();
    // Summary: income, expenses, surplus/deficit and savings rate, each vs the previous one.
    for (const label of ['Receita', 'Despesas', 'Taxa de poupança'])
      expect(screen.getByText(label, { selector: 'dt' })).toBeTruthy();
    expect(screen.getByText(/^(Sobra|Déficit)$/, { selector: 'dt' })).toBeTruthy();
    expect(screen.getAllByText(/^vs /).length).toBeGreaterThanOrEqual(4);

    // Contributions leave the budget separately; transfers between accounts do not appear.
    const nodes = sankeyNodes();
    expect(nodes).toContain('hub');
    expect(nodes).toContain('investment');
    expect(nodes.some((n) => n.startsWith('sub:'))).toBe(true);

    // Tooltip: value, % of income and the change over the month before last.
    const tip = (chartOption(/^Fluxo de/).tooltip as AnyRec).formatter as Tip;
    const html = tip({ dataType: 'node', name: 'investment', data: {} });
    expect(html).toContain('Da receita');
    expect(html).toContain('Variação');

    // Comparison below the chart: current, previous, Δ and Δ%.
    const table = screen.getByRole('grid', { name: /^Comparativo do fluxo/ });
    expect(within(table).getByText('Investimentos')).toBeTruthy();
    expect(within(table).getByText('Δ %')).toBeTruthy();
    expect(within(table).getAllByText('Entradas').length).toBeGreaterThan(0);

    // Clicking the node opens the drill-down: the period's transactions that add up to the node value.
    act(() =>
      chartProps(/^Fluxo de/).onClick?.({
        name: 'expense:cat-moradia',
        dataIndex: 0,
        value: 0,
        data: {},
      }),
    );
    const dialog = await screen.findByRole('dialog', { name: /Analítico · Moradia/ });
    expect(await within(dialog).findByText(/igual ao valor de/)).toBeTruthy();
    expect(mockCallLog()).toContain('reports.sankey_items');
    expect(within(dialog).getByRole('tab', { selected: true }).textContent.toLowerCase()).toContain(
      formatMonthLong(last).toLowerCase(),
    );
  });

  it('arrows switch the period; the detail hides the subcategories', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios?v=fluxo&p=trimestre');
    await screen.findByText(/Fig\. 01/);
    await user.click(screen.getByRole('button', { name: 'Período anterior' }));
    await waitFor(() => expect(hashSearch().get('p')).toBe('trimestre-anterior'));
    await user.click(screen.getByRole('button', { name: 'Período anterior' }));
    await waitFor(() => expect(hashSearch().get('p')).toBe('personalizado'));
    const from = hashSearch().get('from')!;
    expect(from.endsWith('-01')).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Próximo período' }));
    await waitFor(() => expect(hashSearch().get('p')).toBe('trimestre-anterior'));
    expect(mockCallLog().filter((c) => c === 'reports.sankey').length).toBeGreaterThan(2);

    await screen.findByText(/Fig\. 01/);
    // Each period's chart is kept by its label: it starts from zero in the current period.
    clearCharts();
    await user.click(screen.getByRole('radio', { name: 'Categorias' }));
    await eventually(() => !sankeyNodes().some((n) => n.startsWith('sub:')));
    expect(sankeyNodes()).toContain('investment');
  });

  it('privacy mode hides values in the summary, tooltip and table', async () => {
    renderApp('unlocked', {}, '#/relatorios?v=fluxo&p=mes-anterior');
    await screen.findByText(/Fig\. 01/);
    act(() => useUi.getState().setPrivacy(true));
    await waitFor(() =>
      expect(document.querySelectorAll('[data-money="hidden"]').length).toBeGreaterThan(10),
    );
    const table = screen.getByRole('grid', { name: /^Comparativo do fluxo/ });
    expect(within(table).queryByText(/R\$/)).toBeNull();
    const tip = (chartOption(/^Fluxo de/, true).tooltip as AnyRec).formatter as Tip;
    expect(tip({ dataType: 'node', name: 'investment', data: {} })).not.toMatch(/R\$|%/);
  });
});
