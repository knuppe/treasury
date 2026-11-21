/**
 * 01 · Painel and 05 · Orçamento in other languages: the cards, financial health, the budget
 * tabs and planning come from the catalogs in the app's language (docs/i18n-design.md).
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  await applyLocale('pt-BR');
});

describe('Painel and Orçamento in other languages', () => {
  it('shows the dashboard in English', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/painel');
    expect(
      await screen.findByRole('heading', { level: 2, name: 'Net worth' }, { timeout: 5_000 }),
    ).toBeTruthy();
    expect(screen.getByRole('heading', { level: 2, name: 'Cash flow this month' })).toBeTruthy();
    expect(screen.getByRole('heading', { level: 2, name: 'Upcoming due dates' })).toBeTruthy();
    const health = await screen.findByRole('region', { name: 'Financial health' });
    expect(within(health).getByRole('list', { name: 'Dimensions' })).toBeTruthy();
    expect(within(health).getByRole('link', { name: /See indicators/ })).toBeTruthy();
    const insights = await screen.findByRole('region', { name: 'What Treasury noticed' });
    expect(within(insights).getAllByRole('button', { name: 'Dismiss' }).length).toBeGreaterThan(0);
  });

  it('shows the budget and the scenarios in Spanish', async () => {
    const user = userEvent.setup();
    await applyLocale('es');
    renderApp('unlocked', {}, '#/orcamento');
    expect(
      await screen.findByRole('tab', { name: 'Previsto frente a real', selected: true }),
    ).toBeTruthy();
    expect(await screen.findByRole('button', { name: /Sugerir por el promedio/ })).toBeTruthy();
    expect(screen.getByRole('table', { name: 'Presupuesto de gastos por categoría' })).toBeTruthy();
    await user.click(screen.getByRole('tab', { name: 'Escenarios' }));
    expect(await screen.findByRole('heading', { name: 'Escenarios "¿y si…?"' })).toBeTruthy();
    expect(screen.getByRole('radio', { name: 'Comprar una vivienda' })).toBeTruthy();
  });
});
