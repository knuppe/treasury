/**
 * Barra ⌘K com os comandos globais: abas e seções, ações rápidas, busca no cofre, itens só do
 * Brasil escondidos fora dele e os usados por último no topo.
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

describe('barra ⌘K: comandos globais', () => {
  it('toda aba e seção da lista tem rótulo e sinônimos no catálogo', () => {
    const tk = t as unknown as (k: string) => string;
    for (const d of DESTINATIONS) {
      expect(tk(d.label), d.label).not.toBe(d.label);
      if (d.synonyms) expect(tk(d.synonyms), d.synonyms).not.toBe(d.synonyms);
    }
  });

  it('leva a uma aba pelo sinônimo e abre o ajuste de saldo como lançamento', async () => {
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

  it('acha contas e categorias do cofre só quando a pessoa digita', async () => {
    const { user, dialog, input } = await openBar();
    expect(options(dialog).some((o) => o.includes('Nubank Conta'))).toBe(false);
    await user.type(input, 'nubank');
    await waitFor(() => expect(within(dialog).getByRole('group', { name: 'Contas' })).toBeTruthy());
    await user.click(within(dialog).getByRole('option', { name: /Nubank Conta/ }));
    expect(await screen.findByRole('heading', { level: 1, name: /Nubank Conta/ })).toBeTruthy();
  });

  it('no Brasil, a B3 e o IR aparecem', async () => {
    const { user, dialog, input } = await openBar();
    await user.type(input, 'b3');
    expect(options(dialog).some((o) => o.includes('Área do Investidor'))).toBe(true);
  });

  it('fora do Brasil, o que é só do Brasil some', async () => {
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

  it('o comando usado por último aparece em Recentes', async () => {
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
