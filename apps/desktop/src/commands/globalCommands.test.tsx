/**
 * ⌘K bar with the global commands: tabs and sections, quick actions, vault search, Brazil-only
 * items hidden outside Brazil, and the most recently used on top.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { t } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { DESTINATIONS } from './useGlobalCommands';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

async function openBar(country?: string) {
  const user = userEvent.setup();
  renderApp('unlocked', country ? { country } : {}, '#/painel');
  await screen.findByRole('navigation', { name: 'Telas' });
  await user.keyboard('{Control>}k{/Control}');
  const dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });
  return { user, dialog, input: within(dialog).getByRole('combobox') };
}

const options = (dialog: HTMLElement) =>
  within(dialog)
    .queryAllByRole('option')
    .map((o) => o.textContent ?? '');

describe('⌘K bar: global commands', () => {
  it('every tab and section in the list has a label and synonyms in the catalog', () => {
    const tk = t as unknown as (k: string) => string;
    for (const d of DESTINATIONS) {
      expect(tk(d.label), d.label).not.toBe(d.label);
      if (d.synonyms) expect(tk(d.synonyms), d.synonyms).not.toBe(d.synonyms);
    }
  });

  it('goes to a tab by synonym and opens the balance adjustment as a transaction', async () => {
    const { user, dialog, input } = await openBar();
    await user.type(input, 'contracheque');
    expect(within(dialog).getByRole('option', { name: /Renda e holerites/ })).toBeTruthy();
    await user.clear(input);
    await user.type(input, 'ajustar saldo');
    await user.keyboard('{Enter}');
    const txn = await screen.findByRole('dialog', { name: 'Novo lançamento' });
    expect(
      within(txn).getByRole('radio', { name: 'Ajuste de saldo' }).getAttribute('aria-checked'),
    ).toBe('true');
  });

  it('finds vault accounts and categories only when the person types', async () => {
    const { user, dialog, input } = await openBar();
    expect(options(dialog).some((o) => o.includes('Nubank Conta'))).toBe(false);
    await user.type(input, 'nubank');
    await waitFor(() => expect(within(dialog).getByRole('group', { name: 'Contas' })).toBeTruthy());
    await user.click(within(dialog).getByRole('option', { name: /Nubank Conta/ }));
    expect(await screen.findByRole('heading', { level: 1, name: /Nubank Conta/ })).toBeTruthy();
  });

  it('in Brazil, B3 and IR show up', async () => {
    const { user, dialog, input } = await openBar();
    await user.type(input, 'b3');
    expect(options(dialog).some((o) => o.includes('Área do Investidor'))).toBe(true);
  });

  it('outside Brazil, what is Brazil-only disappears', async () => {
    const { user, dialog, input } = await openBar('US');
    await user.type(input, 'b3');
    expect(options(dialog).some((o) => o.includes('Área do Investidor'))).toBe(false);
    await user.clear(input);
    await user.type(input, 'imposto de renda');
    expect(options(dialog).some((o) => /declaração/i.test(o))).toBe(false);
    await user.clear(input);
    await user.type(input, 'screener');
    expect(options(dialog).some((o) => o.includes('Screener'))).toBe(false);
  });

  it('the most recently used command shows up in Recents', async () => {
    const { user, input } = await openBar();
    await user.type(input, 'nova meta');
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('dialog', { name: /Nova meta/ })).toBeTruthy();
    await user.keyboard('{Escape}');
    await user.keyboard('{Control>}k{/Control}');
    const again = await screen.findByRole('dialog', { name: 'Barra de comando' });
    const recent = within(again).getByRole('group', { name: 'Recentes' });
    expect(within(recent).getByRole('option', { name: /Nova meta/ })).toBeTruthy();
  });
});
