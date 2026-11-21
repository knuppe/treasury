/** 05 · Budget › "Para onde vai a renda": mock numbers, editable targets and the context. */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { GroupTargets } from '@/api/generated/GroupTargets';
import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import { mockCall } from '@/api/mock';
import { usePageContextStore } from '@/features/companion/pageContext';
import { groupStatus } from '@/features/reports/model/groups';
import { formatMoney, formatPercent, todayIso } from '@/lib/format';
import { monthOf, shiftMonth } from '@/lib/months';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const month = shiftMonth(monthOf(todayIso()), -1);

async function report(): Promise<SpendGroupsReport> {
  return (await mockCall('reports.spend_groups', { from: month, to: month })) as SpendGroupsReport;
}

async function openCard() {
  renderApp('unlocked', {}, `#/orcamento?mes=${month}`);
  const card = await screen.findByRole('region', { name: 'Para onde vai a renda' });
  await within(card).findByRole('list', { name: /^Grupos de gasto de/ });
  return card;
}

describe('05 · Budget › Where the income goes', () => {
  it('shows the four groups with value, share of income and target, like the report', async () => {
    const user = userEvent.setup();
    const card = await openCard();
    const data = await report();
    const m = data.months[0]!;
    const list = within(card).getByRole('list', { name: /^Grupos de gasto de/ });
    const lines = within(list).getAllByRole('listitem');
    expect(lines).toHaveLength(4);
    const names = ['Essenciais', 'Dívidas', 'Futuro', 'Não essencial'];
    m.groups.forEach((g, i) => {
      const line = lines[i]!;
      expect(line.textContent).toContain(names[i]);
      expect(line.textContent).toContain(formatMoney(g.amount));
      if (g.share_of_income != null)
        expect(line.textContent).toContain(
          `${formatPercent(g.share_of_income, { decimals: 0 })} da renda`,
        );
    });
    // Default targets (50/30/20, Debts with no target) and each group's hint.
    expect(lines[0]!.textContent).toContain('meta 50%');
    expect(lines[1]!.textContent).toContain('sem meta');
    expect(lines[2]!.textContent).toContain('meta 20%');
    expect(lines[3]!.textContent).toContain('meta 30%');
    expect(lines[0]!.textContent).toContain('O que não dá para cortar');
    // Future counts the month's contributions, even without a category.
    expect(m.groups[2]!.top.map((c) => c.label)).toContain('Aportes');
    const essential = m.groups[0]!;
    const status = groupStatus('essential', essential.share_of_income, 0.5);
    expect(lines[0]!.textContent).toContain(status === 'over' ? 'acima da meta' : 'dentro da meta');
    // Footer: income and what went to the groups.
    expect(card.textContent).toContain(formatMoney(m.income));
    expect(card.textContent).toContain(formatMoney(m.spending));

    // The group's largest categories, when the row is opened.
    await user.click(within(lines[2]!).getByRole('button', { name: 'Ver as maiores de Futuro' }));
    const top = within(card).getByRole('list', { name: 'Maiores de Futuro' });
    expect(within(top).getByText('Aportes')).toBeTruthy();

    // The page context for the agent carries the per-group summary.
    await waitFor(() => {
      const ctx = usePageContextStore.getState().stack.at(-1)?.ctx;
      expect(ctx?.groups?.map((g) => g.group)).toEqual([
        'income',
        'essential',
        'debt',
        'future',
        'non_essential',
      ]);
    });
  });

  it('edits the targets per group and saves to budget.set_group_targets', async () => {
    const user = userEvent.setup();
    const card = await openCard();
    await user.click(within(card).getByRole('button', { name: /Editar metas/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Metas por grupo' });
    const essential = within(dialog).getByLabelText('Meta de Essenciais (% da renda)');
    expect((essential as HTMLInputElement).value).toBe('50');
    await user.clear(essential);
    await user.type(essential, '55');
    await user.type(within(dialog).getByLabelText('Meta de Dívidas (% da renda)'), '10');
    expect(within(dialog).getByText(/Soma das metas: 115%/)).toBeTruthy();
    expect(within(dialog).getByText(/A soma passa de 100% da renda/)).toBeTruthy();
    // A value outside 0 to 100 blocks saving.
    const future = within(dialog).getByLabelText('Meta de Futuro (% da renda)');
    await user.clear(future);
    await user.type(future, '120');
    expect(within(dialog).getByText('Use um número de 0 a 100.')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Salvar' }).hasAttribute('disabled')).toBe(
      true,
    );
    await user.clear(future);
    await user.type(future, '15');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('Metas por grupo salvas')).toBeTruthy();
    const saved = (await mockCall('budget.group_targets', {})) as GroupTargets;
    expect(saved).toEqual({ essential: 0.55, debt: 0.1, future: 0.15, non_essential: 0.3 });
    const list = within(card).getByRole('list', { name: /^Grupos de gasto de/ });
    await waitFor(() =>
      expect(within(list).getAllByRole('listitem')[0]!.textContent).toContain('meta 55%'),
    );
    expect(within(list).getAllByRole('listitem')[1]!.textContent).toContain('meta 10%');
  });
});
