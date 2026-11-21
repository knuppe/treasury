/**
 * Compra do cartão gravada também na conta à vista (mesmo identificador do banco): aviso acima
 * dos lançamentos, revisão com o que fica e o que sai, e só apaga a cópia com confirmação.
 */
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { patch } from '@/api/mock/ledger/store';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

describe('04 · Lançamentos › cópias entre contas', () => {
  it('avisa, mostra o par e apaga só a cópia confirmada', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    const card = accounts.find((a) => a.kind === 'credit_card')!;
    // Conta à vista da mesma instituição do cartão.
    const checking = (await mockCall('accounts.create', {
      name: 'Conta do mesmo banco',
      kind: 'checking',
      institution_id: card.institution_id,
    })) as AccountView;
    const mk = async (account: string) =>
      (
        (await mockCall('transactions.create', {
          account_id: account,
          date: todayIso(),
          amount: -4_590,
          description: 'Padaria Pão Quente',
        })) as { id: string }
      ).id;
    const keep = await mk(card.id);
    const copy = await mk(checking.id);
    const ext = '0b9c2d4e-1111-4b7e-9d2a-00000000aaaa';
    patch('txn', keep, { external_id: ext });
    // Caso comum: a cópia ficou conciliada pelo ajuste de saldo da conta.
    patch('txn', copy, { external_id: ext, status: 'reconciled' });
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    const notice = await screen.findByText(/1 lançamento repete uma compra já gravada/);
    await user.click(
      within(notice.closest('[role="status"]')!).getByRole('button', { name: 'Revisar' }),
    );
    const dialog = await screen.findByRole('dialog', { name: 'Cópias entre contas' });
    const list = within(dialog).getByRole('list', { name: 'Cópias encontradas' });
    expect(within(list).getByText(new RegExp(`Fica: ${card.name}`))).toBeTruthy();
    expect(within(list).getByText(new RegExp(`Cópia: ${checking.name}`))).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Apagar 1 cópia' }));
    expect(await screen.findByText('1 cópia apagada')).toBeTruthy();
    await waitFor(() =>
      expect(screen.queryByText(/lançamento repete uma compra já gravada/)).toBeNull(),
    );
    await expect(mockCall('transactions.get', { id: copy })).rejects.toBeTruthy();
    expect(((await mockCall('transactions.get', { id: keep })) as { id: string }).id).toBe(keep);
  });
});
