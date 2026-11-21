/**
 * 04 · Transactions in English: the screen, the filters, the shared labels (`controles.ftl`) and the
 * rules tab come from the catalogs in the app language (docs/i18n-design.md).
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { TXN_KIND } from './ui/labels';

afterEach(async () => {
  await applyLocale('pt-BR');
});

describe('04 · Transactions in English', () => {
  it('shows the table, filters and totals in English', async () => {
    await applyLocale('en');
    renderApp('unlocked', {}, '#/lancamentos');
    expect(await screen.findByRole('grid', { name: 'Transactions' })).toBeTruthy();
    expect(screen.getByRole('button', { name: /New transaction/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Period: This month/ })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Recurrences' })).toBeTruthy();
    const totals = screen.getByLabelText('Filter totals');
    expect(within(totals).getAllByText('Net').length).toBeGreaterThan(0);
    expect(TXN_KIND.card_payment).toBe('Card bill payment');
  });

  it('opens the rules tab in Spanish', async () => {
    const user = userEvent.setup();
    await applyLocale('es');
    renderApp('unlocked', {}, '#/lancamentos?aba=regras');
    expect(await screen.findByRole('grid', { name: 'Reglas' })).toBeTruthy();
    await user.click(screen.getAllByRole('button', { name: 'Nueva regla' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: 'Nueva regla' });
    expect(within(dialog).getByLabelText('Texto de la condición 1')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Probar' })).toBeTruthy();
  });
});
