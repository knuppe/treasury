import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';
import { useCompanion } from '@/state/companion';

import { useImportHub } from './importHub';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('single import window', () => {
  it('recognizes the format, opens the importer and sends the unknown to the agent', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => useImportHub.getState().setOpen(true));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    const input = within(dialog).getByTestId<HTMLInputElement>('universal-import-input');
    await user.upload(input, [
      new File(['OFXHEADER:100\n<OFX></OFX>'], 'extrato.ofx', { type: 'application/x-ofx' }),
      new File(['%PDF-1.4 NOTA DE CORRETAGEM'], 'nota.pdf', { type: 'application/pdf' }),
      new File(['lista de compras'], 'lista.txt', { type: 'text/plain' }),
    ]);
    const statements = await within(dialog).findByRole('region', { name: 'Extratos e faturas' });
    expect(within(statements).getByText('extrato.ofx')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: /Importar nota/ })).toBeTruthy();
    const unknown = within(dialog).getByRole('region', { name: 'Não reconhecidos' });
    await user.click(await within(unknown).findByRole('button', { name: /Analisar com IA/ }));
    // The file goes as an attachment to the agent panel, with ready-made text to complete.
    await waitFor(() => expect(useCompanion.getState().open).toBe(true));
    const tray = await screen.findByRole('list', { name: 'Anexos da mensagem' });
    expect(within(tray).getByText('lista.txt')).toBeTruthy();
    const box = screen.getByLabelText<HTMLTextAreaElement>('Mensagem para o agente (painel)');
    expect(box.value).toMatch(/lista\.txt/);
  });

  it('images are read on the computer before the AI: a receipt opens the purchase, a payment proof goes to the agent with the stated type', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => useImportHub.getState().setOpen(true));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13]);
    await user.upload(within(dialog).getByTestId<HTMLInputElement>('universal-import-input'), [
      new File([png], 'cupom-mercado.png', { type: 'image/png' }),
      new File([png], 'comprovante-pix.png', { type: 'image/png' }),
      new File([png], 'foto.png', { type: 'image/png' }),
    ]);
    const receipts = await within(dialog).findByRole(
      'region',
      { name: 'Cupons fiscais e notas de compra' },
      { timeout: 15_000 },
    );
    expect(within(receipts).getByText(/Cupom fiscal lido no computador/)).toBeTruthy();
    const payments = within(dialog).getByRole('region', {
      name: 'Comprovantes de pagamento e boletos',
    });
    expect(within(payments).getByText('comprovante-pix.png')).toBeTruthy();
    const images = within(dialog).getByRole('region', { name: /imagens/i });
    expect(within(images).getByText('foto.png')).toBeTruthy();
    // Payment proof: the agent receives the image and the request already saying what it is.
    await user.click(within(payments).getByRole('button', { name: /Registrar com o agente/ }));
    await waitFor(() => expect(useCompanion.getState().open).toBe(true));
    const tray = await screen.findByRole('list', { name: 'Anexos da mensagem' });
    expect(within(tray).getByText('comprovante-pix.png')).toBeTruthy();
    const box = screen.getByLabelText<HTMLTextAreaElement>('Mensagem para o agente (painel)');
    expect(box.value).toMatch(/comprovante ou boleto/);
  });

  it('boleto becomes a planned bill payable, read on the computer', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => useImportHub.getState().setOpen(true));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13]);
    await user.upload(
      within(dialog).getByTestId<HTMLInputElement>('universal-import-input'),
      new File([png], 'boleto-escola.png', { type: 'image/png' }),
    );
    const payments = await within(dialog).findByRole(
      'region',
      { name: 'Comprovantes de pagamento e boletos' },
      { timeout: 15_000 },
    );
    await user.click(within(payments).getByRole('button', { name: 'Revisar' }));
    const pd = await screen.findByRole('dialog', { name: 'Conta a pagar' });
    expect(await within(pd).findByText(/34191\.23456/)).toBeTruthy();
    expect(within(pd).getByDisplayValue('Escola Exemplo')).toBeTruthy();
    await user.click(within(pd).getByRole('combobox', { name: 'Conta' }));
    await user.click((await screen.findAllByRole('option'))[0]!);
    await user.click(within(pd).getByRole('button', { name: 'Criar conta prevista' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Conta a pagar' })).toBeNull());
    const page = (await mockCall('transactions.query', { limit: 500 })) as {
      items: { description: string; status: string; amount: number }[];
    };
    const row = page.items.find((r) => r.description === 'Escola Exemplo');
    expect(row?.status).toBe('planned');
    expect(row?.amount).toBe(-12_345);
  });

  it('payment proof links to the statement transaction, without keeping the image', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => useImportHub.getState().setOpen(true));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13]);
    await user.upload(
      within(dialog).getByTestId<HTMLInputElement>('universal-import-input'),
      new File([png], 'comprovante-pix.png', { type: 'image/png' }),
    );
    const payments = await within(dialog).findByRole(
      'region',
      { name: 'Comprovantes de pagamento e boletos' },
      { timeout: 15_000 },
    );
    await user.click(within(payments).getByRole('button', { name: 'Revisar' }));
    const pd = await screen.findByRole('dialog', { name: 'Comprovante' });
    expect(await within(pd).findByText('E18236120202610221705aBcDeFgHiJk')).toBeTruthy();
    expect(within(pd).getByText(/só os dados lidos vão para a nota/)).toBeTruthy();
    // No matching statement transaction: creates one, in the chosen account.
    await user.click(within(pd).getByRole('combobox', { name: 'Conta' }));
    await user.click((await screen.findAllByRole('option'))[0]!);
    await user.click(within(pd).getByRole('button', { name: 'Guardar os dados do comprovante' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Comprovante' })).toBeNull());
    const page = (await mockCall('transactions.query', { limit: 500 })) as {
      items: { payee: string | null; amount: number }[];
    };
    expect(page.items.some((r) => r.payee === 'Fulano de Teste' && r.amount === -5_000)).toBe(true);
  });

  it('Caixa period statement (with a payment-proof name) goes to statements, without AI', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => useImportHub.getState().setOpen(true));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    await user.upload(
      within(dialog).getByTestId<HTMLInputElement>('universal-import-input'),
      new File(['%PDF-1.3 Extrato por período'], 'Comprovante_2025-03-15_093000.pdf', {
        type: 'application/pdf',
      }),
    );
    const statements = await within(dialog).findByRole('region', { name: 'Extratos e faturas' });
    expect(within(statements).getByText(/Extrato da Caixa/)).toBeTruthy();
    expect(within(statements).getByRole('button', { name: 'Importar' })).toBeTruthy();
    expect(within(dialog).queryByRole('region', { name: 'Não reconhecidos' })).toBeNull();
  });

  it('XML receipt opens the invoice for review, without AI', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => useImportHub.getState().setOpen(true));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    await user.upload(
      within(dialog).getByTestId<HTMLInputElement>('universal-import-input'),
      new File(
        [
          '<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe1"></infNFe></nfeProc>',
        ],
        'nota.xml',
        { type: 'text/xml' },
      ),
    );
    const receipts = await within(dialog).findByRole('region', {
      name: 'Cupons fiscais e notas de compra',
    });
    expect(within(receipts).getByText(/Nota fiscal em XML/)).toBeTruthy();
    expect(within(receipts).getByRole('button', { name: 'Revisar compra' })).toBeTruthy();
  });

  it('the Importar button at the top opens the window', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ }, { timeout: 15_000 });
    const bar = document.querySelector('header.topbar') as HTMLElement;
    await user.click(within(bar).getByRole('button', { name: 'Importar' }));
    expect(await screen.findByRole('dialog', { name: 'Importar arquivos' })).toBeTruthy();
  });
});
