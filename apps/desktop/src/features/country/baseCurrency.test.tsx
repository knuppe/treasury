/**
 * Family base currency (that of the vault country): in a United States family the dashboard and
 * accounts come out in dollars, in a Portugal one in euros, with no `R$` (docs/countries.md).
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

describe('family base currency', () => {
  it('United States family: dashboard and accounts in USD', async () => {
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
    // The euro account is the family "other currency": on the same line, the value (or the missing exchange rate) in USD.
    const eur = within(grid).getByRole('row', { name: /Revolut EUR/ });
    await waitFor(() => expect(eur.textContent).toMatch(/EUR.*USD/));
    expect(main().textContent).not.toContain('R$');
  });

  it('Portugal family: accounts in EUR', async () => {
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
