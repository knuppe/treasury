import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

async function all() {
  return (await mockCall('accounts.list', { include_archived: true })) as AccountView[];
}

/** Opens the app, creates the account and goes to its page. */
async function openNewAccount(name: string, opening: number) {
  renderApp('unlocked', {}, '#/contas');
  await screen.findByRole('grid', { name: 'Contas' });
  const acc = (await mockCall('accounts.create', {
    name,
    kind: 'checking',
    opening_balance: opening,
    opening_date: '2026-01-02',
  })) as AccountView;
  act(() => {
    window.location.hash = `#/contas?conta=${acc.id}`;
  });
  await screen.findByRole('heading', { level: 1, name: new RegExp(name) });
  return acc;
}

describe('03 · Accounts · closing an account', () => {
  it('blocks while the account has money and offers the transfer', async () => {
    const user = userEvent.setup();
    await openNewAccount('Banco Fechado', 12_345);
    await user.click(await screen.findByRole('button', { name: 'Encerrar conta' }));
    const dialog = await screen.findByRole('dialog', { name: /Encerrar Banco Fechado/ });
    const blockers = await within(dialog).findByRole('region', { name: 'Antes de encerrar' });
    expect(within(blockers).getByText(/Ainda há R\$\s?123,45 na conta/)).toBeTruthy();
    expect(within(blockers).getByRole('button', { name: 'Transferir o saldo' })).toBeTruthy();
    const confirm = within(dialog).getAllByRole('button', { name: 'Encerrar conta' }).at(-1)!;
    expect((confirm as HTMLButtonElement).disabled).toBe(true);
  });

  it('closes an empty account, shows the date and reopens it', async () => {
    const user = userEvent.setup();
    const acc = await openNewAccount('Corretora Antiga', 0);
    await user.click(await screen.findByRole('button', { name: 'Encerrar conta' }));
    const dialog = await screen.findByRole('dialog', { name: /Encerrar Corretora Antiga/ });
    expect(await within(dialog).findByText(/pode ser encerrada/)).toBeTruthy();
    await user.click(within(dialog).getAllByRole('button', { name: 'Encerrar conta' }).at(-1)!);
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: /Encerrar Corretora Antiga/ })).toBeNull(),
    );

    const closed = (await all()).find((a) => a.id === acc.id)!;
    expect(closed).toMatchObject({ archived: true, closed_on: todayIso() });
    expect(await screen.findByText(/^Encerrada em /)).toBeTruthy();
    // The page stays open (the account left the list) and says it is closed.
    await expect(mockCall('accounts.close_preview', { id: acc.id })).resolves.toMatchObject({
      blockers: [{ kind: 'already_closed', closed_on: todayIso() }],
    });

    await user.click(screen.getByRole('button', { name: 'Reabrir conta' }));
    await waitFor(async () =>
      expect((await all()).find((a) => a.id === acc.id)).toMatchObject({
        archived: false,
        closed_on: null,
      }),
    );
  });
});
