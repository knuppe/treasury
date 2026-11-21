import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';
import { todayIso } from '@/lib/format';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

describe('03 · Contas › Aposentadoria', () => {
  it('nos EUA, a conta de aposentadoria mostra o aportado do ano contra o limite', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'US' }, '#/contas');
    await user.click((await screen.findAllByRole('button', { name: /Nova conta/ }))[0]!);
    const form = await screen.findByRole('dialog', { name: 'Nova conta' });
    await user.type(within(form).getByLabelText('Nome'), 'Roth IRA Exemplo');
    await user.click(within(form).getAllByRole('combobox')[0]!);
    await user.click(await screen.findByRole('option', { name: /Aposentadoria/ }));
    await user.click(within(form).getByRole('combobox', { name: 'Plano' }));
    await user.click(await screen.findByRole('option', { name: 'Roth IRA' }));
    await user.click(within(form).getByRole('button', { name: 'Criar conta' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Nova conta' })).toBeNull());
    const acc = ((await mockCall('accounts.list', {})) as AccountView[]).find(
      (a) => a.name === 'Roth IRA Exemplo',
    )!;
    expect(acc.kind).toBe('retirement');
    expect(acc.retirement_plan).toBe('us_roth_ira');
    await mockCall('transactions.create', {
      account_id: acc.id,
      date: todayIso(),
      amount: 200_000,
      kind: 'income',
      description: 'Aporte',
    });
    const panel = await screen.findByRole('region', { name: /Roth IRA em/ });
    await waitFor(() => expect(within(panel).getByText(/de/)).toBeTruthy());
    const s = (await mockCall('retirement.status', { account_id: acc.id })) as {
      contributed: number;
      remaining: number;
    };
    expect(s.contributed).toBe(200_000);
    expect(s.remaining).toBe(550_000);
  });

  it('no Brasil não há o tipo de conta de aposentadoria', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await user.click((await screen.findAllByRole('button', { name: /Nova conta/ }))[0]!);
    const form = await screen.findByRole('dialog', { name: 'Nova conta' });
    await user.click(within(form).getAllByRole('combobox')[0]!);
    expect(screen.queryByRole('option', { name: /Aposentadoria/ })).toBeNull();
  });
});
