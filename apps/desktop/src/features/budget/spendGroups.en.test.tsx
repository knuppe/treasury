/** Grupos de gasto em inglês: o cartão do Orçamento, a figura 12 e Ajustes › Categorias. */
import { screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  clearCharts();
  await applyLocale('pt-BR');
});

describe('Grupos de gasto em inglês', () => {
  it('Budget: where the income goes, with targets', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/orcamento');
    const card = await screen.findByRole('region', { name: 'Where the income goes' });
    const list = await within(card).findByRole('list', { name: /^Spending groups for/ });
    const lines = within(list).getAllByRole('listitem');
    expect(lines.map((l) => within(l).getAllByRole('button')[0]!.textContent)).toEqual([
      'Essentials',
      'Debts',
      'Future',
      'Non-essential',
    ]);
    expect(lines[0]!.textContent).toContain('target 50%');
    expect(lines[1]!.textContent).toContain('no target');
    expect(within(card).getByRole('button', { name: /Edit targets/ })).toBeTruthy();
    expect(screen.queryByText('Para onde vai a renda')).toBeNull();
  });

  it('Reports: the Groups view', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/relatorios?v=grupos&p=12m');
    expect(await screen.findByText(/^Where the income goes: 12 months to/)).toBeTruthy();
    expect(screen.getByRole('tab', { name: /Groups/ })).toBeTruthy();
    expect(screen.getByRole('radiogroup', { name: 'Measure' })).toBeTruthy();
    expect(screen.getByRole('grid', { name: 'Largest categories by group' })).toBeTruthy();
  });

  it('Settings › Categories: group selector and filter', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/ajustes?secao=categorias');
    const tree = await screen.findByRole('tree', { name: 'Expense categories' });
    expect(screen.getByRole('radiogroup', { name: 'Filter by group' })).toBeTruthy();
    const select = within(tree).getAllByRole('combobox')[0]!;
    expect(select.getAttribute('aria-label')).toMatch(/^Group of /);
    expect(select.textContent).toBe('Essentials');
  });
});
