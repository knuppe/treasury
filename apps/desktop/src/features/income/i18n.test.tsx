/**
 * 02 · Família em inglês: a renda de cada membro, os nomes da folha do Brasil (catálogo) e as
 * contas e cartões de cada um vêm dos catálogos no idioma do app (docs/i18n-design.md).
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

describe('02 · Família em inglês', () => {
  it('mostra as rendas e os holerites com os nomes em inglês', async () => {
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
