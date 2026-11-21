/**
 * 01 · Painel e 05 · Orçamento em outros idiomas: os cards, a saúde financeira, as abas do
 * orçamento e o planejamento vêm dos catálogos no idioma do app (docs/i18n-design.md).
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

describe('Painel e Orçamento em outros idiomas', () => {
  it('mostra o painel em inglês', async () => {
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

  it('mostra o orçamento e os cenários em espanhol', async () => {
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
