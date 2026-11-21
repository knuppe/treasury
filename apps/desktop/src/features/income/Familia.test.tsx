/**
 * 02 · Família against the mock: each member's income, each one's accounts and cards, and
 * choosing the member of an account without a member (its transactions start inheriting it).
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
  it("shows each member's income and accounts and assigns a member to the account without one", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    expect(await screen.findByRole('heading', { level: 1, name: /Família/ })).toBeTruthy();
    expect((await screen.findAllByText('Salário · Alfa Engenharia')).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Contas e cartões:/).length).toBeGreaterThan(0);

    // The Carteira (cash) has no member: choose Ana.
    const loose = await screen.findByRole('region', { name: /Contas e cartões sem membro/ });
    await user.click(within(loose).getByRole('combobox', { name: 'Membro de Carteira' }));
    await user.click(await screen.findByRole('option', { name: 'Ana' }));
    await waitFor(async () => expect((await account('Carteira')).member_ids).toEqual(['mem-ana']));
  });

  it('the payslip table states the type of each one', async () => {
    renderApp('unlocked', {}, '#/familia');
    const grid = await screen.findByRole('grid', { name: 'Holerites importados' });
    expect(within(grid).getAllByText('Tipo').length).toBeGreaterThan(0);
    expect(within(grid).getAllByText('Mensal').length).toBeGreaterThan(0);
  });

  it("chooses a member's accounts and cards", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    await screen.findByText('Studio Norte (PJ)');
    const before = await account('Poupança Caixa');
    expect(before.member_ids).toEqual(['mem-ana']);
    const buttons = screen.getAllByRole('button', { name: 'Escolher' });
    // The order of the cards follows the members: Ana, then Bruno.
    await user.click(buttons[1]!);
    const dialog = await screen.findByRole('dialog', { name: /Contas e cartões de Bruno/ });
    await user.click(within(dialog).getByRole('checkbox', { name: 'Poupança Caixa' }));
    await waitFor(async () =>
      expect((await account('Poupança Caixa')).member_ids).toEqual(['mem-ana', 'mem-bruno']),
    );
    // With Ana and Bruno, the savings account becomes joint: its transactions belong to the family.
    const row = within(dialog).getByRole('checkbox', { name: 'Poupança Caixa' }).closest('div')!;
    await waitFor(() => expect(row.textContent).toContain('conjunta com Ana'));
  });
});

describe('02 · Família › Simular demissão', () => {
  it('opens the simulation on a CLT income and shows severance, FGTS, insurance and the total', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    await screen.findAllByText('Salário · Alfa Engenharia');
    await user.click(screen.getAllByRole('button', { name: 'Simular demissão' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: /Simular demissão · / });
    // With no hire date on the income, it asks for the dates before computing.
    const admission = within(dialog).queryByText('Informe as datas para ver a simulação.');
    if (admission) {
      await user.type(within(dialog).getByLabelText('Admissão'), '01/03/2022');
    }
    const total = await within(dialog).findByRole('region', { name: 'Total disponível' });
    expect(total.textContent).toMatch(/R\$/);
    for (const s of ['Rescisão', 'FGTS', 'Seguro-desemprego'])
      expect(within(dialog).getByRole('region', { name: s })).toBeTruthy();
    expect(within(dialog).getByText('Aviso prévio indenizado')).toBeTruthy();
    // Resignation: no insurance.
    await user.click(within(dialog).getByRole('combobox', { name: 'Motivo' }));
    await user.click(await screen.findByRole('option', { name: 'Pedido de demissão' }));
    expect(
      await within(dialog).findByText('O seguro-desemprego é só para dispensa sem justa causa.'),
    ).toBeTruthy();
  });
});
