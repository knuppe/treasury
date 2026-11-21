/**
 * 03 · Accounts in English and Spanish: the screen, the account cards, the registration, the detail and the
 * exchange dialog come from the catalogs in the app language (docs/i18n-design.md).
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

describe('03 · Accounts in other languages', () => {
  it('shows the accounts, the summary and the credit card in English', async () => {
    const user = userEvent.setup();
    await applyLocale('en');
    renderApp('unlocked', {}, '#/contas');
    expect(await screen.findByRole('heading', { level: 1, name: /Accounts/ })).toBeTruthy();
    const grid = await screen.findByRole('grid', { name: 'Accounts' });
    expect(screen.getByRole('region', { name: 'Accounts summary' })).toBeTruthy();
    expect(within(grid).getByRole('columnheader', { name: /Balance/ })).toBeTruthy();
    expect(within(grid).getByRole('row', { name: /Cartão Nubank/ })).toBeTruthy();

    await user.click(screen.getAllByRole('button', { name: 'New account' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: 'New account' });
    expect(within(dialog).getByLabelText(/Opening balance/)).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Create account' })).toBeTruthy();
  });

  it('opens the account detail in Spanish', async () => {
    const user = userEvent.setup();
    await applyLocale('es');
    renderApp('unlocked', {}, '#/contas');
    await user.click(await screen.findByRole('button', { name: 'Abrir Carteira' }));
    expect(await screen.findByRole('tab', { name: 'Historial de saldo' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Movimientos' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Ajustar saldo' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Conciliación' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Nuevo movimiento' })).toBeTruthy();
  });
});
