/**
 * 09 · Ativos › fundamental analysis in English and Spanish: tabs, indicator groups and
 * labels come from the `pesquisa.ftl` catalog in the app's language (docs/i18n-design.md); the
 * indicator names still come from the core.
 */
import { screen, within } from '@testing-library/react';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { STATUS_LABEL } from '../devfeedback/format';
import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../portfolio/state';
import { fixToday, quietCanvas } from '../portfolio/testing';
import { compactAmount } from './format';
import { GROUP_LABEL } from './labels';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(async () => {
  vi.useRealTimers();
  await applyLocale('pt-BR');
});

describe('09 · Ativos › analysis in other languages', () => {
  it('fundamental analysis in English', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/ativos?ativo=ast-petr4&secao=fundamentos');
    const section = await screen.findByRole('region', { name: 'Fundamental analysis' });
    expect(within(section).getByRole('tab', { name: 'Calculated price' })).toBeTruthy();
    expect(within(section).getByRole('tab', { name: 'History' })).toBeTruthy();
    expect(
      await within(section).findByRole('list', { name: 'Indicators · Valuation' }),
    ).toBeTruthy();
    expect(within(section).getByRole('list', { name: 'Indicators · Profitability' })).toBeTruthy();
    expect(compactAmount(2_500_000_000, 'USD')).toBe('$ 2.5B');
  });

  it('labels in Spanish', async () => {
    await applyLocale('es');
    expect(GROUP_LABEL.debt).toBe('Endeudamiento');
    expect(STATUS_LABEL.em_andamento).toBe('En curso');
  });
});
