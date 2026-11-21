/**
 * Anexo do lançamento no app (WKWebView): link de Blob não baixa nada lá, então clicar no
 * anexo pergunta onde salvar (diálogo do sistema) e o núcleo grava o arquivo.
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

describe('04 · Lançamentos › anexos no app', () => {
  it('salva o anexo pelo diálogo do sistema e pelo núcleo', async () => {
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
    // Só agora "vira" o app: o diálogo e a chamada ao núcleo passam pelo invoke.
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
