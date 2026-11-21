/**
 * 07 · Dívidas em inglês (docs/i18n-design.md): a lista, o resumo, o detalhe do contrato e a
 * tabela de amortização vêm do catálogo `dividas.ftl`; nomes de contrato (dados do usuário)
 * ficam como estão.
 */
import { screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fixToday } from '@/features/portfolio/testing';
import { applyLocale } from '@/i18n';
import { clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => {
  fixToday(); // 14/10/2026
  clearCharts();
});
afterEach(async () => {
  vi.useRealTimers();
  await applyLocale('pt-BR');
});

describe('Dívidas em inglês', () => {
  it('mostra a lista, o resumo e o detalhe do contrato traduzidos', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/dividas');
    expect(await screen.findByRole('heading', { level: 1, name: /Debts/ })).toBeTruthy();
    const list = await screen.findByRole('grid', { name: 'Loans' });
    expect(within(list).getAllByText('Financiamento do apartamento').length).toBeGreaterThan(0);
    expect(within(list).getAllByText('Price (TP) · fixed')).toHaveLength(2);
    expect(screen.getByRole('heading', { name: 'Total outstanding balance' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Monthly commitment' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'New loan' })).toBeTruthy();

    const detail = screen.getByRole('region', { name: 'Loan Financiamento do apartamento' });
    expect(within(detail).getByText('Actual CET')).toBeTruthy();
    expect(within(detail).getByText(/TR adjustment \(on the anniversary\)/)).toBeTruthy();
    const table = await within(detail).findByRole('grid', {
      name: 'Amortization table · Financiamento do apartamento',
    });
    expect(within(table).getAllByText('Paid')).toHaveLength(3);
    expect(within(detail).getByRole('radio', { name: /^Upcoming/ })).toBeTruthy();
  });
});
