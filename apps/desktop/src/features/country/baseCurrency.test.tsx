/**
 * Moeda base da família (a do país do cofre): numa família dos Estados Unidos o painel e as
 * contas saem em dólar, numa de Portugal em euro, sem nenhum `R$` (docs/countries.md).
 */
import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { Household } from '@/api/generated/Household';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

const main = () => screen.getByRole('main');

describe('moeda base da família', () => {
  it('família dos Estados Unidos: painel e contas em USD', async () => {
    renderApp('unlocked', { country: 'US' }, '#/painel');
    const h = (await mockCall('household.get', {})) as Household;
    expect([h.country, h.base_currency]).toEqual(['US', 'USD']);
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    expect(accounts.find((a) => a.id === 'acc-itau')?.currency).toBe('USD');

    const netWorth = await screen.findByRole(
      'heading',
      { level: 2, name: 'Patrimônio líquido' },
      { timeout: 5_000 },
    );
    const card = netWorth.closest('section') ?? main();
    await waitFor(() => expect(within(card).getAllByText(/^−?USD /).length).toBeGreaterThan(0));
    await waitFor(() => expect(main().textContent).toMatch(/USD \d/));
    expect(main().textContent).not.toContain('R$');

    window.location.hash = '#/contas';
    expect(await screen.findByRole('heading', { level: 1, name: /Contas/ })).toBeTruthy();
    const summary = await screen.findByRole('region', { name: 'Resumo das contas' });
    expect(within(summary).getByText('Saldo líquido em USD')).toBeTruthy();
    const grid = await screen.findByRole('grid', { name: 'Contas' });
    const itau = within(grid).getByRole('row', { name: /Itaú Conta Corrente/ });
    expect(itau.textContent).toMatch(/USD\s?\d/);
    // A conta em euro é a "outra moeda" da família: na mesma linha, o valor (ou a falta de câmbio) em USD.
    const eur = within(grid).getByRole('row', { name: /Revolut EUR/ });
    await waitFor(() => expect(eur.textContent).toMatch(/EUR.*USD/));
    expect(main().textContent).not.toContain('R$');
  });

  it('família de Portugal: contas em EUR', async () => {
    renderApp('unlocked', { country: 'PT' }, '#/contas');
    expect(await screen.findByRole('heading', { level: 1, name: /Contas/ })).toBeTruthy();
    const summary = await screen.findByRole('region', { name: 'Resumo das contas' });
    expect(within(summary).getByText('Saldo líquido em EUR')).toBeTruthy();
    await waitFor(() => expect(within(summary).getAllByText(/^−?EUR /).length).toBeGreaterThan(0));
    const grid = await screen.findByRole('grid', { name: 'Contas' });
    const itau = within(grid).getByRole('row', { name: /Itaú Conta Corrente/ });
    expect(itau.textContent).toMatch(/EUR\s?\d/);
    const usd = within(grid).getByRole('row', { name: /Wise USD/ });
    await waitFor(() => expect(usd.textContent).toMatch(/USD.*EUR/));
    expect(main().textContent).not.toContain('R$');

    window.location.hash = '#/painel';
    await screen.findByRole(
      'heading',
      { level: 2, name: 'Patrimônio líquido' },
      { timeout: 5_000 },
    );
    await waitFor(() => expect(main().textContent).toMatch(/EUR \d/));
    expect(main().textContent).not.toContain('R$');
  });
});
