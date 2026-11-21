/** 09 · Ativos em inglês (docs/i18n-design.md): abas, filtros, tabela e página do ativo. */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../portfolio/state';
import { fixToday, quietCanvas } from '../portfolio/testing';

beforeAll(quietCanvas);
beforeEach(async () => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
  await applyLocale('en');
});
afterEach(async () => {
  vi.useRealTimers();
  await applyLocale('pt-BR');
});

describe('09 · Assets in English', () => {
  it('shows the tabs, filters and the asset table in English', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos');
    const grid = await screen.findByRole('grid', { name: 'Registered assets' });
    expect(within(grid).getByText('PETR4')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Add asset' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Dividend calendar' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Exchanges' })).toBeTruthy();
    await user.click(screen.getByRole('combobox', { name: 'Filter by position' }));
    expect(await screen.findByRole('option', { name: 'Closed positions' })).toBeTruthy();
  });

  it('opens the asset page with translated tabs', async () => {
    renderApp('unlocked', {}, '#/ativos?ativo=ast-petr4');
    expect(await screen.findByRole('tab', { name: 'Overview' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Health' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'New trade' })).toBeTruthy();
  });
});
