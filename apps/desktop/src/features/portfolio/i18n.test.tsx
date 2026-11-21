/**
 * 08 · Carteira em inglês e espanhol: abas, resumo, posições, seletor de deflator e o diálogo
 * de operação vêm dos catálogos no idioma do app, e os decimais seguem o formato do idioma
 * (docs/i18n-design.md).
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from './state';
import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(async () => {
  vi.useRealTimers();
  await applyLocale('pt-BR');
});

describe('08 · Carteira em outros idiomas', () => {
  it('mostra o resumo, as abas e as posições em inglês', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/carteira');
    expect(await screen.findByRole('heading', { level: 1, name: /Portfolio/ })).toBeTruthy();
    const stocks = await screen.findByRole('grid', { name: 'Positions · Stocks' });
    expect(within(stocks).getByText('ITUB4')).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Allocation and rebalancing' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: /Market value/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'New trade' })).toBeTruthy();
    const group = screen.getByRole('radiogroup', { name: 'Real result deflator' });
    expect(within(group).getByRole('radio', { name: 'Country inflation (BR)' })).toBeTruthy();
  });

  it('lê o preço digitado com ponto decimal em inglês', async () => {
    const user = userEvent.setup();
    await applyLocale('en');
    renderApp('unlocked', {}, '#/carteira?nova=operacao&ativo=ast-petr4');
    const dialog = await screen.findByRole('dialog', { name: 'Record trade' });
    await within(dialog).findAllByText(/PETR4 · Petrobras PN/);
    await user.type(within(dialog).getByLabelText('Quantity'), '10');
    await user.type(within(dialog).getByLabelText('Unit price (BRL)'), '31.50');
    // Bruto exato: 10 × 31.50 (o ponto é o decimal em inglês).
    expect(within(dialog).getByText(/^R\$\s?315\.00$/)).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Record purchase' })).toBeTruthy();
  });

  it('mostra as abas e o risco em espanhol', async () => {
    await applyLocale('es');
    renderApp('unlocked', {}, '#/carteira?aba=risco');
    expect(await screen.findByText('Cartera de hoy en el período')).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Rentabilidad' })).toBeTruthy();
    expect(screen.getByRole('table', { name: 'Riesgo por clase' })).toBeTruthy();
    expect(screen.getAllByText(/Sharpe \(CDI\)/).length).toBe(2);
  });
});
