/**
 * 04 · Lançamentos · "Vincular a uma operação" against the mock: the goal cited in the statement comes
 * first, linking becomes a contribution without a category and unlinking restores the previous type.
 */
import { QueryClientProvider } from '@tanstack/react-query';
import { cleanup, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router';
import { describe, expect, it, vi } from 'vitest';

import type { GoalView } from '@/api/generated/GoalView';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall, resetMock } from '@/api/mock';
import { createQueryClient } from '@/api/queryClient';
import { Toaster } from '@/design/components';

import { BulkOperationLinkDialog } from './BulkOperationLinkDialog';
import { OperationLinkDialog } from './OperationLinkDialog';
import {
  bulkMenuItems,
  txnMenuItems,
  type BulkMenuHandlers,
  type TxnMenuHandlers,
} from './txnMenu';

const txnView = async (id: string) => (await mockCall('transactions.get', { id })) as TxnView;

function openDialog(txn: TxnView) {
  const onOpenChange = vi.fn();
  const onDone = vi.fn();
  render(
    <QueryClientProvider client={createQueryClient()}>
      <MemoryRouter>
        <OperationLinkDialog txn={txn} open onOpenChange={onOpenChange} onDone={onDone} />
        <Toaster />
      </MemoryRouter>
    </QueryClientProvider>,
  );
  return { onOpenChange, onDone };
}

describe('OperationLinkDialog', () => {
  it('links the scheduled savings deposit to the goal and unlinks', async () => {
    const user = userEvent.setup();
    resetMock('unlocked');
    const goal = (await mockCall('goals.create', {
      name: 'Reforma da cozinha',
      kind: 'purchase',
      target_amount: 1_000_000,
    })) as GoalView;
    const created = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: '2026-10-05',
      amount: -20_000,
      kind: 'expense',
      description: 'POUPANCA PROGRAMADA REFORMA',
    })) as TxnView;

    const first = openDialog(created);
    let dialog = await screen.findByRole('dialog', { name: 'Vincular a uma operação' });
    const options = await within(dialog).findAllByRole('radio');
    expect(options[0]!.getAttribute('aria-checked')).toBe('true');
    expect(within(options[0]!).getByText('Meta Reforma da cozinha')).toBeTruthy();
    expect(within(options[0]!).getByText('o texto do extrato cita a meta')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Vincular' }));
    await waitFor(() => expect(first.onDone).toHaveBeenCalled());
    expect(first.onDone.mock.calls[0]![0]).toBe('Lançamento vinculado');
    let t = await txnView(created.id);
    expect(t).toMatchObject({ kind: 'investment', category_id: null });
    expect(t.operation).toMatchObject({ kind: 'goal', target_id: goal.id });

    // From the row menu, the item shows the link.
    const items = txnMenuItems(t, { linkOperation: () => undefined } as unknown as TxnMenuHandlers);
    expect(
      items.some((i) => i && 'label' in i && i.label === 'Vínculo: Meta Reforma da cozinha…'),
    ).toBe(true);

    cleanup();
    const second = openDialog(t);
    dialog = await screen.findByRole('dialog', { name: 'Vincular a uma operação' });
    expect(within(dialog).getByText('Vinculado: Meta Reforma da cozinha')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Desvincular' }));
    await waitFor(() =>
      expect(second.onDone).toHaveBeenCalledWith('Vínculo desfeito', expect.anything()),
    );
    t = await txnView(created.id);
    expect(t.kind).toBe('expense');
    expect(t.operation).toBeNull();
  });

  it('a linked transfer cannot be linked: the dialog explains', async () => {
    resetMock('unlocked');
    const created = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: '2026-10-05',
      amount: -20_000,
      kind: 'expense',
      description: 'APLICACAO',
    })) as TxnView;
    openDialog({ ...created, transfer_id: 'x' });
    const dialog = await screen.findByRole('dialog', { name: 'Vincular a uma operação' });
    expect(within(dialog).getByText(/desligue a transferência/)).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Vincular' }).hasAttribute('disabled')).toBe(
      true,
    );
  });
});

describe('BulkOperationLinkDialog (several checked transactions)', () => {
  it('picks the goal, confirms with the list and the total and skips the planned one', async () => {
    const user = userEvent.setup();
    resetMock('unlocked');
    await mockCall('goals.create', {
      name: 'Reforma da cozinha',
      kind: 'purchase',
      target_amount: 1_000_000,
    });
    const make = async (date: string, status?: string) =>
      (
        (await mockCall('transactions.create', {
          account_id: 'acc-itau',
          date,
          amount: -20_000,
          kind: 'expense',
          description: 'POUPANCA PROGRAMADA REFORMA',
          ...(status ? { status } : {}),
        })) as TxnView
      ).id;
    const ids = [
      await make('2026-08-05'),
      await make('2026-09-05'),
      await make('2026-12-05', 'planned'),
    ];
    const onDone = vi.fn();
    render(
      <QueryClientProvider client={createQueryClient()}>
        <MemoryRouter>
          <BulkOperationLinkDialog ids={ids} open onOpenChange={vi.fn()} onDone={onDone} />
          <Toaster />
        </MemoryRouter>
      </QueryClientProvider>,
    );
    let dialog = await screen.findByRole('dialog', { name: 'Vincular a uma operação' });
    const options = await within(dialog).findAllByRole('radio');
    expect(within(options[0]!).getByText('Meta Reforma da cozinha')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Continuar' }));

    dialog = await screen.findByRole('dialog', { name: 'Confirmar o vínculo' });
    const grid = await within(dialog).findByRole('grid', { name: 'Lançamentos a vincular' });
    expect(within(grid).getAllByText('✓ Entra')).toHaveLength(2);
    expect(within(grid).getByText(/Fica de fora: .*previsto/)).toBeTruthy();
    expect(within(dialog).getByText('1 lançamento fica de fora')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Vincular 2 lançamentos' }));
    await waitFor(() => expect(onDone).toHaveBeenCalled());
    expect(onDone.mock.calls[0]![0]).toBe('2 lançamentos vinculados');
    expect((await txnView(ids[0]!)).kind).toBe('investment');
    expect((await txnView(ids[1]!)).operation?.label).toBe('Meta Reforma da cozinha');
    expect((await txnView(ids[2]!)).operation).toBeNull();
  });

  it('the batch menu has the link', () => {
    const items = bulkMenuItems(3, {
      linkOperation: () => undefined,
    } as unknown as BulkMenuHandlers);
    expect(items.some((i) => i && 'label' in i && i.label === 'Vincular a uma operação…')).toBe(
      true,
    );
  });
});
