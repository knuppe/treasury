import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { TxnPage } from '@/api/generated/TxnPage';
import { mockCall } from '@/api/mock';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

import { useImportHub } from './importHub';

function ofx(): File {
  const d = (n: number) => {
    const t = new Date();
    t.setDate(t.getDate() - n);
    return `${t.getFullYear()}${String(t.getMonth() + 1).padStart(2, '0')}${String(t.getDate()).padStart(2, '0')}`;
  };
  const body = [
    'OFXHEADER:100',
    'DATA:OFXSGML',
    'CHARSET:1252',
    '',
    '<OFX><BANKMSGSRSV1><STMTTRNRS><STMTRS><CURDEF>BRL',
    '<BANKACCTFROM><BANKID>0077<BRANCHID>0001<ACCTID>98765-4<ACCTTYPE>CHECKING</BANKACCTFROM>',
    `<BANKTRANLIST><DTSTART>${d(6)}<DTEND>${d(1)}`,
    `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${d(5)}<TRNAMT>-38.90<FITID>T-1<MEMO>UBER TRIP CENTRO</STMTTRN>`,
    `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${d(4)}<TRNAMT>-210.00<FITID>T-2<MEMO>LIVRARIA CULTURA VIVA</STMTTRN>`,
    `<STMTTRN><TRNTYPE>CREDIT<DTPOSTED>${d(3)}<TRNAMT>450.00<FITID>T-3<MEMO>PIX RECEBIDO CARLA</STMTTRN>`,
    `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${d(2)}<TRNAMT>0.00<FITID>T-4<MEMO>SALDO DO DIA</STMTTRN>`,
    `</BANKTRANLIST><LEDGERBAL><BALAMT>9876.54<DTASOF>${d(1)}</LEDGERBAL></STMTRS></STMTTRNRS></BANKMSGSRSV1></OFX>`,
  ].join('\n');
  return new File([body], 'extrato-teste.ofx', { type: 'application/x-ofx' });
}

/** Fatura de cartão cujo `ACCTID` é um UUID (sem o final do cartão). Dados fictícios. */
function cardOfx(name: string, fitid: string, amount: string): File {
  const t = new Date();
  t.setDate(t.getDate() - 2);
  const day = `${t.getFullYear()}${String(t.getMonth() + 1).padStart(2, '0')}${String(t.getDate()).padStart(2, '0')}`;
  const body = [
    'OFXHEADER:100',
    'DATA:OFXSGML',
    'CHARSET:1252',
    '',
    '<OFX><CREDITCARDMSGSRSV1><CCSTMTTRNRS><CCSTMTRS><CURDEF>BRL',
    '<CCACCTFROM><ACCTID>5612467a-3f1e-4c2b-9d7a-0e8b1c2d4321</CCACCTFROM>',
    `<BANKTRANLIST><DTSTART>${day}<DTEND>${day}`,
    `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${day}<TRNAMT>-${amount}<FITID>${fitid}<MEMO>PADARIA FICTICIA</STMTTRN>`,
    '</BANKTRANLIST></CCSTMTRS></CCSTMTTRNRS></CREDITCARDMSGSRSV1></OFX>',
  ].join('\n');
  return new File([body], name, { type: 'application/x-ofx' });
}

async function importedCount() {
  const page = (await mockCall('transactions.query', { text: 'livraria cultura viva' })) as TxnPage;
  return page.total;
}

describe('importação de extrato', () => {
  it('detecta, mostra a prévia com regras e ajustes, grava e fecha a janela', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    await user.keyboard('i');
    const dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    expect(
      within(dialog).getByRole('combobox', { name: 'Conta de destino' }).textContent,
    ).toContain('Itaú Conta Corrente');
    await user.upload(within(dialog).getByLabelText('Arquivo do banco'), ofx());

    // Leitura: dados do OFX.
    expect(await within(dialog).findByText('Extrato de conta (OFX)')).toBeTruthy();
    expect(within(dialog).getByText('98765-4', { exact: false })).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Gerar prévia' }));

    // Prévia: linha de saldo ignorada, regra do UBER sugerida, ajuste por linha.
    const table = await within(dialog).findByRole('table', { name: 'Prévia da importação' });
    expect(within(dialog).getByRole('button', { name: /Nova 3/ })).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: /Ignorada 1/ })).toBeTruthy();
    expect(
      within(table).getByRole('button', { name: /Categoria da linha 1: Transporte › Aplicativos/ }),
    ).toBeTruthy();
    const desc = within(table).getByLabelText('Descrição da linha 2');
    await user.clear(desc);
    await user.type(desc, 'Livraria Cultura Viva (livros)');
    await user.tab();
    await user.click(within(table).getByRole('checkbox', { name: 'Importar linha 3' }));
    await user.click(within(dialog).getByRole('button', { name: 'Importar 2 lançamento(s)' }));

    // Importou: a janela fecha e o resultado fica no aviso.
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar extrato' })).toBeNull(),
    );
    expect(
      await screen.findByText(/2 lançamento\(s\) importado\(s\) em Itaú Conta Corrente/),
    ).toBeTruthy();
    expect(await importedCount()).toBe(1);
    const renamed = (await mockCall('transactions.query', {
      text: 'livraria cultura viva',
    })) as TxnPage;
    expect(renamed.items[0]!.description).toBe('Livraria Cultura Viva (livros)');
    expect(renamed.items[0]!.notes).toBe('Descrição no extrato: LIVRARIA CULTURA VIVA');
  });

  it('grava agência e conta do OFX e reconhece a conta na próxima vez', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    await user.keyboard('i');
    let dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    await user.upload(within(dialog).getByLabelText('Arquivo do banco'), ofx());
    expect(await within(dialog).findByText(/Não reconheci a conta \(nenhuma conta/)).toBeTruthy();
    expect(within(dialog).getByText(/Banco do arquivo: 077/)).toBeTruthy();
    await user.click(within(dialog).getByRole('combobox', { name: 'Conta de destino' }));
    await user.click(await screen.findByRole('option', { name: /Nubank Conta/ }));
    await user.click(
      within(dialog).getByRole('button', { name: 'Gravar agência e conta em Nubank Conta' }),
    );
    await waitFor(async () =>
      expect(
        ((await mockCall('accounts.get', { id: 'acc-nuconta' })) as { account_number: string })
          .account_number,
      ).toBe('98765-4'),
    );
    await user.click(within(dialog).getByRole('button', { name: 'Fechar' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar extrato' })).toBeNull(),
    );

    await user.keyboard('i');
    dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    await user.upload(within(dialog).getByLabelText('Arquivo do banco'), ofx());
    expect(
      await within(dialog).findByText(/Reconhecida pelo banco, agência e número/),
    ).toBeTruthy();
    expect(
      within(dialog).getByRole('combobox', { name: 'Conta de destino' }).textContent,
    ).toContain('Nubank Conta');
  });

  it('lembra o cartão escolhido para um OFX com ACCTID que o cadastro não reconhece', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    await user.keyboard('i');
    let dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    await user.upload(
      within(dialog).getByLabelText('Arquivo do banco'),
      cardOfx('fatura-1.ofx', 'C-1', '42.50'),
    );
    expect(
      await within(dialog).findByText(
        /Não reconheci a conta \(nenhum cartão com esse final.*a escolha fica lembrada/,
      ),
    ).toBeTruthy();
    // Fatura detectada: nenhuma conta pré-escolhida e só cartões na lista.
    const destination = within(dialog).getByRole('combobox', { name: 'Conta de destino' });
    expect(destination.textContent).not.toContain('Itaú Conta Corrente');
    await user.click(destination);
    const options = (await screen.findAllByRole('option')).map((o) => o.textContent ?? '');
    expect(options.length).toBeGreaterThan(0);
    expect(options.every((o) => o.includes('Cartão'))).toBe(true);
    expect(options.some((o) => /Nubank Conta|Conta Corrente|Poupança/.test(o))).toBe(false);
    await user.click(await screen.findByRole('option', { name: /Cartão Nubank/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Gerar prévia' }));
    await user.click(
      await within(dialog).findByRole('button', { name: 'Importar 1 lançamento(s)' }),
    );
    expect(
      await screen.findByText(/1 lançamento\(s\) importado\(s\) em Cartão Nubank/),
    ).toBeTruthy();
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar extrato' })).toBeNull(),
    );

    // Próxima fatura do mesmo cartão: a conta já vem escolhida, com o motivo.
    await user.keyboard('i');
    dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    await user.upload(
      within(dialog).getByLabelText('Arquivo do banco'),
      cardOfx('fatura-2.ofx', 'C-2', '17.30'),
    );
    expect(
      await within(dialog).findByText(/Conta lembrada da última importação deste arquivo/),
    ).toBeTruthy();
    expect(
      within(dialog).getByRole('combobox', { name: 'Conta de destino' }).textContent,
    ).toContain('Cartão Nubank');
  });

  it('arquivo vindo de fora: escolhe a conta, maximiza a janela e ajusta colunas', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    act(() => useImportHub.getState().setOpen(true));
    const hub = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    await user.upload(within(hub).getByTestId<HTMLInputElement>('universal-import-input'), ofx());
    const statements = await within(hub).findByRole('region', { name: 'Extratos e faturas' });
    await user.click(within(statements).getAllByRole('button')[0]!);
    const dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    // Não reconheceu a conta: o campo fica vazio e a prévia espera a escolha.
    expect(await within(dialog).findByText(/Não reconheci a conta/)).toBeTruthy();
    const generate = within(dialog).getByRole('button', { name: 'Gerar prévia' });
    expect((generate as HTMLButtonElement).disabled).toBe(true);
    await user.click(within(dialog).getByRole('combobox', { name: 'Conta de destino' }));
    await user.click(await screen.findByRole('option', { name: /Nubank Conta/ }));
    expect((generate as HTMLButtonElement).disabled).toBe(false);

    await user.click(within(dialog).getByRole('button', { name: 'Maximizar' }));
    expect(dialog.style.width).not.toBe('');
    await user.click(within(dialog).getByRole('button', { name: 'Restaurar tamanho' }));
    expect(within(dialog).getByRole('separator', { name: /Redimensionar a janela/ })).toBeTruthy();

    await user.click(generate);
    await within(dialog).findByRole('table', { name: 'Prévia da importação' });
    const col = within(dialog).getByRole('separator', { name: 'Largura da coluna Descrição' });
    const before = Number(col.getAttribute('aria-valuenow'));
    col.focus();
    await user.keyboard('{ArrowRight}{ArrowRight}');
    expect(Number(col.getAttribute('aria-valuenow'))).toBe(before + 32);
    // Na prévia dá para trocar a conta (refaz a prévia).
    expect(
      within(dialog).getByRole('combobox', { name: 'Conta de destino' }).textContent,
    ).toContain('Nubank Conta');
    act(() => useImportHub.getState().setOpen(false));
  });

  it('reimportar o mesmo arquivo marca duplicados', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=importacoes');
    await user.click(await screen.findByRole('button', { name: 'Importar extrato' }));
    let dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    await user.upload(within(dialog).getByLabelText('Arquivo do banco'), ofx());
    await user.click(await within(dialog).findByRole('button', { name: 'Gerar prévia' }));
    await user.click(
      await within(dialog).findByRole('button', { name: 'Importar 3 lançamento(s)' }),
    );
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar extrato' })).toBeNull(),
    );
    expect(await screen.findByText('extrato-teste.ofx')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Importar extrato' }));
    dialog = await screen.findByRole('dialog', { name: 'Importar extrato' });
    await user.upload(within(dialog).getByLabelText('Arquivo do banco'), ofx());
    await user.click(await within(dialog).findByRole('button', { name: 'Gerar prévia' }));
    expect(await within(dialog).findByRole('button', { name: /Duplicada 3/ })).toBeTruthy();
    expect(
      within(dialog).getByText('este mesmo arquivo já foi importado nesta conta'),
    ).toBeTruthy();
    expect(
      within(dialog)
        .getByRole('button', { name: 'Importar 0 lançamento(s)' })
        .hasAttribute('disabled'),
    ).toBe(true);
    expect(todayIso()).toMatch(/^\d{4}-/);
  });
});
