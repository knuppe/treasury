/**
 * 02 · Família in English: each member's income, the Brazilian payroll names (catalog) and
 * each one's accounts and cards come from the catalogs in the app's language (docs/i18n-design.md).
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

describe('02 · Família in English', () => {
  it('shows the incomes and payslips with the names in English', async () => {
    const user = userEvent.setup();
    await applyLocale('en');
    renderApp('unlocked', {}, '#/familia');
    expect(await screen.findByRole('heading', { level: 1, name: /Family/ })).toBeTruthy();
    expect((await screen.findAllByText('Salário · Alfa Engenharia')).length).toBeGreaterThan(0);
    expect(screen.getAllByText('Accounts and cards:').length).toBeGreaterThan(0);
    expect(screen.getAllByRole('button', { name: 'Import payslip' }).length).toBeGreaterThan(0);
    expect(
      screen.getByRole('region', { name: /Accounts and cards without a member/ }),
    ).toBeTruthy();

    await user.click(screen.getAllByRole('button', { name: 'New income' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: 'New income' });
    expect(within(dialog).getByText('Receives 13º salary')).toBeTruthy();
    expect(within(dialog).getByRole('combobox', { name: 'Type' })).toBeTruthy();
  });
});
