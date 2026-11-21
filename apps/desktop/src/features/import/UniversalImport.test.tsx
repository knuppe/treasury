import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';
import { useCompanion } from '@/state/companion';

import { useImportHub } from './importHub';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('janela única de importação', () => {
  it('reconhece o formato, abre o importador e manda o desconhecido ao agente', async () => {
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
    // O arquivo vai como anexo para o painel do agente, com um texto pronto para completar.
    await waitFor(() => expect(useCompanion.getState().open).toBe(true));
    const tray = await screen.findByRole('list', { name: 'Anexos da mensagem' });
    expect(within(tray).getByText('lista.txt')).toBeTruthy();
    const box = screen.getByLabelText<HTMLTextAreaElement>('Mensagem para o agente (painel)');
    expect(box.value).toMatch(/lista\.txt/);
  });

  it('imagens são lidas no computador antes da IA: cupom abre a compra, comprovante vai ao agente com o tipo dito', async () => {
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
    // Comprovante: o agente recebe a imagem e o pedido já dizendo o que é.
    await user.click(within(payments).getByRole('button', { name: /Registrar com o agente/ }));
    await waitFor(() => expect(useCompanion.getState().open).toBe(true));
    const tray = await screen.findByRole('list', { name: 'Anexos da mensagem' });
    expect(within(tray).getByText('comprovante-pix.png')).toBeTruthy();
    const box = screen.getByLabelText<HTMLTextAreaElement>('Mensagem para o agente (painel)');
    expect(box.value).toMatch(/comprovante ou boleto/);
  });

  it('boleto vira conta a pagar prevista, lido no computador', async () => {
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

  it('comprovante se liga ao lançamento do extrato, sem guardar a imagem', async () => {
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
    // Sem lançamento do extrato que combine: cria um, na conta escolhida.
    await user.click(within(pd).getByRole('combobox', { name: 'Conta' }));
    await user.click((await screen.findAllByRole('option'))[0]!);
    await user.click(within(pd).getByRole('button', { name: 'Guardar os dados do comprovante' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Comprovante' })).toBeNull());
    const page = (await mockCall('transactions.query', { limit: 500 })) as {
      items: { payee: string | null; amount: number }[];
    };
    expect(page.items.some((r) => r.payee === 'Fulano de Teste' && r.amount === -5_000)).toBe(true);
  });

  it('extrato por período da Caixa (nome de comprovante) vai para extratos, sem IA', async () => {
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

  it('cupom em XML abre a nota para revisar, sem IA', async () => {
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

  it('o botão Importar do topo abre a janela', async () => {
    const user = userEvent.setup();
    act(() => useImportHub.getState().setOpen(false));
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ }, { timeout: 15_000 });
    const bar = document.querySelector('header.topbar') as HTMLElement;
    await user.click(within(bar).getByRole('button', { name: 'Importar' }));
    expect(await screen.findByRole('dialog', { name: 'Importar arquivos' })).toBeTruthy();
  });
});
