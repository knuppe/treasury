import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCallLog } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from './state';
import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

async function openTab(asset: string, tab: string, grid: string, view?: string) {
  const user = userEvent.setup();
  renderApp('unlocked', {}, `#/ativos?ativo=${asset}`);
  await user.click(await screen.findByRole('tab', { name: tab }));
  // The Income tab shows the distributions first; the records are one of its views.
  if (view) await user.click(await screen.findByRole('radio', { name: view }));
  return { user, grid: await screen.findByRole('grid', { name: grid }) };
}

/** Record rows (without the header, the totals footer and the empty state). */
const bodyRows = (grid: HTMLElement) => [
  ...grid.querySelectorAll<HTMLTableRowElement>('tbody tr[data-row-id]'),
];
const selected = (grid: HTMLElement) =>
  bodyRows(grid).filter((r) => r.getAttribute('aria-selected') === 'true');

describe('asset records: multi-row selection', () => {
  it('operations: Shift+↓ selects, the selection menu copies and deletes in bulk', async () => {
    const { user, grid } = await openTab('ast-bbas3', 'Operações', 'Operações');
    expect(grid.getAttribute('aria-multiselectable')).toBe('true');
    expect(bodyRows(grid)).toHaveLength(3);

    await user.click(within(grid).getByText('20/01/2026'));
    await user.keyboard('{Shift>}{ArrowDown}{/Shift}');
    expect(selected(grid)).toHaveLength(2);
    expect(screen.getByText('2 operações selecionadas')).toBeTruthy();

    await user.pointer({ keys: '[MouseRight]', target: within(grid).getByText('14/03/2025') });
    const menu = screen.getByRole('menu', { name: 'Ações de 2 operações' });
    expect(within(menu).getByText('2 operações selecionadas')).toBeTruthy();
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((i) => i.dataset.label),
    ).toEqual(['Copiar 2 linhas', 'Excluir 2 operações…']);
    await user.keyboard('{Escape}');

    // ⌘C/Ctrl+C copies the visible columns, one line per operation, without opening the menu.
    await user.keyboard('{Control>}c{/Control}');
    await waitFor(async () => {
      const lines = (await navigator.clipboard.readText()).split('\n');
      expect(lines[0]).toBe('Data\tTipo\tQtd.\tPreço\tCustos\tBruto\tCaixa');
      expect(lines.slice(1).map((l) => l.split('\t').slice(0, 3))).toEqual([
        ['20/01/2026', 'Venda', '50'],
        ['14/03/2025', 'Compra', '100'],
      ]);
    });

    // ⌘⌫/Ctrl+Backspace asks for confirmation with the count; deletes in a single batch.
    await user.keyboard('{Control>}{Backspace}{/Control}');
    const dialog = await screen.findByRole('dialog', { name: 'Excluir 2 operações' });
    const list = within(dialog).getByRole('list', { name: 'Registros a excluir' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
    await user.click(within(dialog).getByRole('button', { name: 'Excluir 2' }));
    await waitFor(() => expect(bodyRows(grid)).toHaveLength(1));
    expect(within(grid).getByText('15/01/2024')).toBeTruthy();
    expect(mockCallLog().filter((c) => c === 'portfolio.delete_trades')).toHaveLength(1);
    expect(mockCallLog()).not.toContain('portfolio.delete_trade');
    expect(await screen.findByText('2 operações excluídas')).toBeTruthy();
  });

  it('operations: row menu with Editar (Enter) and Excluir for a single one', async () => {
    const { user, grid } = await openTab('ast-bbas3', 'Operações', 'Operações');
    await user.pointer({ keys: '[MouseRight]', target: within(grid).getByText('15/01/2024') });
    const menu = screen.getByRole('menu', { name: /^Ações de Compra de 100 BBAS3/ });
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((i) => i.dataset.label),
    ).toEqual(['Editar…', 'Copiar linha', 'Excluir…']);
    await user.keyboard('{Escape}');
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('dialog', { name: /operação/i })).toBeTruthy();
  });

  it('income: ⌘A selects all and the delete shortcut deletes all at once', async () => {
    const { user, grid } = await openTab('ast-hglg11', 'Proventos', 'Proventos', 'Lançamentos');
    const total = bodyRows(grid).length;
    expect(total).toBeGreaterThan(2);
    await user.click(bodyRows(grid)[0]!.querySelector('td')!);
    await user.keyboard('{Control>}a{/Control}');
    expect(selected(grid)).toHaveLength(total);
    expect(screen.getByText(`${total} proventos selecionados`)).toBeTruthy();

    await user.keyboard('{Control>}{Backspace}{/Control}');
    const dialog = await screen.findByRole('dialog', { name: `Excluir ${total} proventos` });
    expect(within(dialog).getByText(`e mais ${total - 8}`)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: `Excluir ${total}` }));
    expect(await within(grid).findByText('Nenhum provento no período.')).toBeTruthy();
    expect(mockCallLog().filter((c) => c === 'portfolio.delete_incomes')).toHaveLength(1);
  });

  it("corporate events: the row's Excluir button uses the same confirmation, in a batch of one", async () => {
    const { user, grid } = await openTab(
      'ast-bbas3',
      'Eventos corporativos',
      'Eventos corporativos',
    );
    expect(bodyRows(grid)).toHaveLength(1);
    await user.click(within(grid).getByRole('button', { name: 'Excluir' }));
    const dialog = await screen.findByRole('dialog', { name: 'Excluir evento corporativo' });
    expect(within(dialog).getByText(/^Desdobramento de BBAS3/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Excluir' }));
    expect(await within(grid).findByText('Nenhum evento corporativo.')).toBeTruthy();
    expect(mockCallLog()).toContain('portfolio.delete_actions');
  });

  it('statement (read-only): Shift+End selects to the end and ⌘C copies', async () => {
    const { user, grid } = await openTab('ast-bbas3', 'Extrato', 'Extrato do ativo');
    const n = bodyRows(grid).length;
    expect(n).toBeGreaterThan(2);
    await user.click(bodyRows(grid)[0]!.querySelector('td')!);
    await user.keyboard('{Shift>}{End}{/Shift}');
    expect(selected(grid)).toHaveLength(n);
    await user.pointer({ keys: '[MouseRight]', target: bodyRows(grid)[1]!.querySelector('td')! });
    const menu = screen.getByRole('menu', { name: `Ações de ${n} linhas` });
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((i) => i.dataset.label),
    ).toEqual([`Copiar ${n} linhas`]);
    await user.keyboard('{Escape}');
    await user.keyboard('{Control>}c{/Control}');
    await waitFor(async () =>
      expect((await navigator.clipboard.readText()).split('\n')).toHaveLength(n + 1),
    );
  });
});
