import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { mockInboxAdd } from '@/api/mock/import';
import { renderApp } from '@/test/renderApp';

const OFX = [
  'OFXHEADER:100',
  'DATA:OFXSGML',
  '<OFX><BANKMSGSRSV1><STMTTRNRS><STMTRS><CURDEF>BRL',
  '<BANKACCTFROM><BANKID>0341<ACCTID>11111</BANKACCTFROM><BANKTRANLIST>',
  '<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>20261001<TRNAMT>-10.00<FITID>IN-1<MEMO>PADARIA EXEMPLO',
  '</STMTTRN></BANKTRANLIST></STMTRS></STMTTRNRS></BANKMSGSRSV1></OFX>',
].join('\n');

describe('caixa de entrada de arquivos', () => {
  it('lista o que chegou na pasta, abre a importação e descarta', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    expect(mockInboxAdd('extrato-exemplo.ofx', OFX)).not.toBeNull();
    // O mesmo conteúdo de novo não vira outro item.
    expect(mockInboxAdd('copia.ofx', OFX)).toBeNull();
    mockInboxAdd('planilha.bin', '\u0000\u0001 nada');
    mockInboxAdd('extrato-exemplo.pdf', '%PDF-1.4 Saldo final 1.234,56');
    await user.click(screen.getByRole('tab', { name: 'Caixa de entrada' }));
    const list = await screen.findByRole('list', { name: 'Arquivos na caixa de entrada' });
    await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(3));
    const unknown = within(list).getByText('planilha.bin').closest('li')!;
    expect(within(unknown).getByText(/tipo não reconhecido/)).toBeTruthy();
    expect(within(unknown).queryByRole('button', { name: 'Importar' })).toBeNull();

    const ofx = within(list).getByText('extrato-exemplo.ofx').closest('li')!;
    await user.click(within(ofx).getByRole('button', { name: 'Importar' }));
    const hub = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    expect(await within(hub).findByText('extrato-exemplo.ofx')).toBeTruthy();
    await user.keyboard('{Escape}');

    // PDF: "Ver como texto" mostra o que os assistentes de IA leem.
    const pdfRow = within(list).getByText('extrato-exemplo.pdf').closest('li')!;
    await user.click(
      within(pdfRow).getByRole('button', { name: 'Ver extrato-exemplo.pdf como texto' }),
    );
    const text = await screen.findByRole('dialog', { name: 'extrato-exemplo.pdf como texto' });
    expect((await within(text).findByLabelText('Conteúdo do documento')).textContent).toContain(
      'Saldo final 1.234,56',
    );
    await user.keyboard('{Escape}');

    await user.click(within(unknown).getByRole('button', { name: 'Descartar planilha.bin' }));
    await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(2));
  });
});
