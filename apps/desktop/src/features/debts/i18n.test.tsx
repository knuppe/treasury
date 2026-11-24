/**
 * 07 · Debts in English (docs/i18n-design.md): the list, the summary, the contract detail and the
 * amortization table come from the `dividas.ftl` catalog; contract names (user data)
 * stay as they are.
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

describe('Debts in English', () => {
  it('shows the list, the summary and the contract detail translated', async () => {
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
    expect(within(detail).getByText('CET of what was paid')).toBeTruthy();
    expect(within(detail).getByText(/TR adjustment \(on the anniversary\)/)).toBeTruthy();
    const table = await within(detail).findByRole('grid', {
      name: 'Amortization table · Financiamento do apartamento',
    });
    expect(within(table).getAllByText('Paid')).toHaveLength(3);
    expect(within(detail).getByRole('radio', { name: /^Upcoming/ })).toBeTruthy();
  });
});
