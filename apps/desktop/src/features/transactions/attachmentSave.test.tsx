/**
 * Transaction attachment in the app (WKWebView): a Blob link downloads nothing there, so clicking the
 * attachment asks where to save (system dialog) and the core writes the file.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

const invoke = vi.hoisted(() => vi.fn());
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
});

describe('04 · Transactions › attachments in the app', () => {
  it('saves the attachment through the system dialog and the core', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const acc = ((await mockCall('accounts.list', {})) as AccountView[])[0]!;
    const txn = (await mockCall('transactions.create', {
      account_id: acc.id,
      date: todayIso(),
      amount: -5_000,
      description: 'Farmácia com recibo',
    })) as { id: string };
    await mockCall('transactions.add_attachment', {
      txn_id: txn.id,
      filename: 'recibo.pdf',
      mime: 'application/pdf',
      content_base64: 'UmVjaWJv',
    });
    await queryClient.invalidateQueries();
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    const row = (await within(grid).findByText('Farmácia com recibo')).closest('tr')!;
    await user.dblClick(row);
    const dialog = await screen.findByRole('dialog', { name: 'Editar lançamento' });
    // Only now it "becomes" the app: the dialog and the core call go through invoke.
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    const calls: { cmd: string; args: Record<string, unknown> }[] = [];
    let dialogOptions: Record<string, unknown> | null = null;
    invoke.mockImplementation(async (cmd: string, a: Record<string, unknown>) => {
      if (cmd === 'plugin:dialog|save') {
        dialogOptions = a.options as Record<string, unknown>;
        return '/Users/teste/Downloads/recibo.pdf';
      }
      if (cmd === 'api_call') {
        const c = a as { cmd: string; args: Record<string, unknown> };
        calls.push(c);
        return mockCall(c.cmd, c.args);
      }
      return null;
    });
    const createUrl = vi.spyOn(URL, 'createObjectURL');
    await user.click(await within(dialog).findByRole('button', { name: 'recibo.pdf' }));
    await waitFor(() =>
      expect(calls.some((c) => c.cmd === 'transactions.save_attachment')).toBe(true),
    );
    const save = calls.find((c) => c.cmd === 'transactions.save_attachment')!;
    expect(save.args.path).toBe('/Users/teste/Downloads/recibo.pdf');
    expect(dialogOptions).toMatchObject({
      defaultPath: 'recibo.pdf',
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    });
    expect(createUrl).not.toHaveBeenCalled();
    expect(await screen.findByText('Anexo salvo')).toBeTruthy();
  });
});
