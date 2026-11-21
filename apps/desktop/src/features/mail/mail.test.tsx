/**
 * 04 · Lançamentos › E-mail contra o mock: configurar a caixa (o catálogo preenche o
 * servidor e a senha fica só no cofre), criar uma regra, buscar, revisar a fila e o modo de
 * teste da exclusão.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { MailAccount } from '@/api/generated/MailAccount';
import type { MailMessage } from '@/api/generated/MailMessage';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

async function account(): Promise<MailAccount> {
  const list = (await mockCall('mail.accounts.list', {})) as MailAccount[];
  return list[0]!;
}

describe('04 · Lançamentos › E-mail', () => {
  it('configura a caixa pelo catálogo, sem mostrar a senha de volta', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=email');
    await user.click(await screen.findByRole('button', { name: 'Configurar o e-mail' }));
    const dialog = await screen.findByRole('dialog', { name: /Nova conta do e-mail/ });
    await user.type(
      within(dialog).getByLabelText('Endereço da caixa'),
      'familia.treasury@gmail.com',
    );
    await user.click(within(dialog).getByRole('button', { name: 'Procurar a configuração' }));
    await waitFor(() =>
      expect(within(dialog).getByLabelText<HTMLInputElement>('Servidor IMAP').value).toBe(
        'imap.gmail.com',
      ),
    );
    expect(within(dialog).getByLabelText<HTMLInputElement>('Usuário').value).toBe(
      'familia.treasury@gmail.com',
    );
    expect(within(dialog).getByText(/verificação em duas etapas/)).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Senha de app'), 'abcd efgh ijkl mnop');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar e testar' }));
    expect(await within(dialog).findByText(/Conectou\. Lê: INBOX/)).toBeTruthy();
    expect(within(dialog).getByText(/Lixeira/)).toBeTruthy();
    const a = await account();
    expect(a.password?.set).toBe(true);
    expect(JSON.stringify(a)).not.toContain('abcd efgh');
    expect(a.allowed_folders).toEqual(['INBOX']);
    expect(a.default_action).toBe('ignore');
  });

  it('busca, revisa a fila e lança o aviso de compra', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await mockCall('mail.accounts.save', {
      name: 'Caixa',
      kind: 'imap',
      provider: 'gmail',
      username: 'familia@gmail.com',
    });
    const a = await account();
    await mockCall('mail.accounts.set_password', { id: a.id, password: 'senha-de-app' });
    await mockCall('mail.rules.save', {
      name: 'Nubank',
      account_id: a.id,
      action: 'review',
      conditions: {
        folders: [],
        from: ['nubank.com.br'],
        from_block: [],
        subject: { contains: [], all: false, not_contains: [] },
      },
    });
    await user.click(await screen.findByRole('tab', { name: 'E-mail' }));
    const contas = await screen.findByRole('region', { name: /Contas/ });
    await user.click(within(contas).getByRole('button', { name: 'Buscar agora' }));
    const fila = screen.getByRole('region', { name: /Fila do e-mail/ });
    const compra = await within(fila).findByRole('article', { name: 'Compra aprovada' });
    expect(within(compra).getByText('Aviso de compra: PADARIA PAO QUENTE')).toBeTruthy();
    expect(within(compra).getByText('Remetente autenticado')).toBeTruthy();
    await user.click(within(compra).getByRole('button', { name: 'Lançar' }));
    await waitFor(async () => {
      const list = (await mockCall('mail.list', {})) as MailMessage[];
      expect(list.find((m) => m.subject === 'Compra aprovada')?.status).toBe('imported');
    });
    // A fatura encaminhada mostra quem encaminhou.
    const fatura = within(fila).getByRole('article', { name: /Sua fatura de outubro/ });
    expect(within(fatura).getByText(/encaminhada por ana@gmail.com/)).toBeTruthy();
    expect(within(fatura).getByRole('button', { name: 'Abrir no importador' })).toBeTruthy();
  });

  it('a exclusão começa em modo de teste e só vale depois da confirmação', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await mockCall('mail.accounts.save', {
      name: 'Pasta',
      kind: 'folder',
      folder_path: '/tmp/treasury-email',
      retention: {
        mode: 'after_success',
        days: 30,
        grace_days: 7,
        local: 'move',
        purge_trash: false,
      },
    });
    let a = await account();
    expect(a.retention_state).toBe('testing');
    await user.click(await screen.findByRole('tab', { name: 'E-mail' }));
    expect(await screen.findByText(/Modo de teste da exclusão até/)).toBeTruthy();
    await expect(mockCall('mail.accounts.confirm_retention', { id: a.id })).rejects.toThrow(
      /modo de teste/,
    );
    a = await account();
    expect(a.retention_confirmed).toBe(false);
  });
});
