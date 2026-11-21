/**
 * 02 · Família contra o mock: a renda de cada membro, as contas e cartões de cada um e a
 * escolha do membro de uma conta sem membro (os lançamentos dela passam a herdá-lo).
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

async function account(name: string) {
  const list = (await mockCall('accounts.list', {})) as AccountView[];
  return list.find((a) => a.name === name)!;
}

describe('02 · Família', () => {
  it('mostra as rendas e as contas de cada membro e dá membro à conta sem membro', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    expect(await screen.findByRole('heading', { level: 1, name: /Família/ })).toBeTruthy();
    expect((await screen.findAllByText('Salário · Alfa Engenharia')).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Contas e cartões:/).length).toBeGreaterThan(0);

    // A Carteira (dinheiro) não tem membro: escolher a Ana.
    const loose = await screen.findByRole('region', { name: /Contas e cartões sem membro/ });
    await user.click(within(loose).getByRole('combobox', { name: 'Membro de Carteira' }));
    await user.click(await screen.findByRole('option', { name: 'Ana' }));
    await waitFor(async () => expect((await account('Carteira')).member_ids).toEqual(['mem-ana']));
  });

  it('a tabela de holerites diz o tipo de cada um', async () => {
    renderApp('unlocked', {}, '#/familia');
    const grid = await screen.findByRole('grid', { name: 'Holerites importados' });
    expect(within(grid).getAllByText('Tipo').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('Mensal').length).toBeGreaterThan(0);
  });

  it('escolhe as contas e cartões de um membro', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    await screen.findByText('Studio Norte (PJ)');
    const before = await account('Poupança Caixa');
    expect(before.member_ids).toEqual(['mem-ana']);
    const buttons = screen.getAllByRole('button', { name: 'Escolher' });
    // A ordem dos cartões segue os membros: Ana, depois Bruno.
    await user.click(buttons[1]!);
    const dialog = await screen.findByRole('dialog', { name: /Contas e cartões de Bruno/ });
    await user.click(within(dialog).getByRole('checkbox', { name: 'Poupança Caixa' }));
    await waitFor(async () =>
      expect((await account('Poupança Caixa')).member_ids).toEqual(['mem-ana', 'mem-bruno']),
    );
    // Com a Ana e o Bruno, a poupança vira conjunta: os lançamentos dela ficam da família.
    const row = within(dialog).getByRole('checkbox', { name: 'Poupança Caixa' }).closest('div')!;
    await waitFor(() => expect(row.textContent).toContain('conjunta com Ana'));
  });
});

describe('02 · Família › Simular demissão', () => {
  it('abre a simulação numa renda CLT e mostra rescisão, FGTS, seguro e o total', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    await screen.findAllByText('Salário · Alfa Engenharia');
    await user.click(screen.getAllByRole('button', { name: 'Simular demissão' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: /Simular demissão · / });
    // Sem admissão na renda, pede as datas antes de calcular.
    const admission = within(dialog).queryByText('Informe as datas para ver a simulação.');
    if (admission) {
      await user.type(within(dialog).getByLabelText('Admissão'), '01/03/2022');
    }
    const total = await within(dialog).findByRole('region', { name: 'Total disponível' });
    expect(total.textContent).toMatch(/R\$/);
    for (const s of ['Rescisão', 'FGTS', 'Seguro-desemprego'])
      expect(within(dialog).getByRole('region', { name: s })).toBeTruthy();
    expect(within(dialog).getByText('Aviso prévio indenizado')).toBeTruthy();
    // Pedido de demissão: sem seguro.
    await user.click(within(dialog).getByRole('combobox', { name: 'Motivo' }));
    await user.click(await screen.findByRole('option', { name: 'Pedido de demissão' }));
    expect(
      await within(dialog).findByText('O seguro-desemprego é só para dispensa sem justa causa.'),
    ).toBeTruthy();
  });
});
