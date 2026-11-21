import { QueryClientProvider } from '@tanstack/react-query';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { TxnPage } from '@/api/generated/TxnPage';
import { mockCall, resetMock } from '@/api/mock';
import { createQueryClient } from '@/api/queryClient';
import { Toaster, useToastStore } from '@/design/components';

import { useUndoToast } from './undoToast';

function Harness({ title }: { title: string }) {
  const undoToast = useUndoToast();
  return (
    <button type="button" onClick={() => undoToast({ title })}>
      avisar
    </button>
  );
}

function setup() {
  resetMock('unlocked');
  useToastStore.getState().clear();
  const client = createQueryClient();
  render(
    <QueryClientProvider client={client}>
      <Harness title="Lançamento criado" />
      <Toaster />
    </QueryClientProvider>,
  );
}

const count = async (text: string) =>
  ((await mockCall('transactions.query', { text })) as TxnPage).total;

describe('notice with Undo', () => {
  it('undoes the change set just created by the user', async () => {
    const user = userEvent.setup();
    setup();
    const accounts = (await mockCall('accounts.list', {})) as { id: string }[];
    await mockCall('transactions.create', {
      account_id: accounts[0]!.id,
      date: '2026-10-14',
      amount: -1234,
      description: 'Café do aviso',
    });
    await user.click(screen.getByRole('button', { name: 'avisar' }));
    await user.click(await screen.findByRole('button', { name: 'Desfazer' }));
    await waitFor(async () => expect(await count('café do aviso')).toBe(0));
    expect(await screen.findByText('Alteração desfeita')).toBeTruthy();
  });

  it('with no recent user change, the notice has no button', async () => {
    const user = userEvent.setup();
    setup();
    await user.click(screen.getByRole('button', { name: 'avisar' }));
    expect(await screen.findByText('Lançamento criado')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Desfazer' })).toBeNull();
  });
});
