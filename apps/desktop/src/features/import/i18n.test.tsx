/**
 * Importação em inglês: a janela única, o assistente de extrato e o aviso da pasta Downloads vêm
 * do catálogo `importacao.ftl` no idioma do app (docs/i18n-design.md).
 */
import { act, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

import { resetDownloadOffers, simulateDownloadOffer } from './downloadsWatch';
import { useImportHub } from './importHub';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

/** Extrato OFX mínimo de conta corrente (dados fictícios). */
const OFX = [
  'OFXHEADER:100',
  '<OFX><BANKMSGSRSV1><STMTTRNRS><STMTRS><CURDEF>BRL',
  '<BANKACCTFROM><BANKID>0077<BRANCHID>0001<ACCTID>98765-4<ACCTTYPE>CHECKING</BANKACCTFROM>',
  '<BANKTRANLIST><DTSTART>20261001<DTEND>20261002',
  '<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>20261001<TRNAMT>-10.00<FITID>X-1<MEMO>PADARIA</STMTTRN>',
  '</BANKTRANLIST></STMTRS></STMTTRNRS></BANKMSGSRSV1></OFX>',
].join('\n');

afterEach(async () => {
  resetDownloadOffers();
  useImportHub.setState({ open: false, pending: [], origins: {} });
  await applyLocale('pt-BR');
});

describe('importação em inglês', () => {
  it('janela única e assistente de extrato', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Transactions' });
    act(() => useImportHub.getState().setOpen(true));
    const hub = await screen.findByRole('dialog', { name: 'Import files' });
    expect(within(hub).getByText('Drop the files here or click to choose')).toBeTruthy();
    expect(within(hub).getByRole('button', { name: 'Set up email import' })).toBeTruthy();
    await user.upload(within(hub).getByTestId<HTMLInputElement>('universal-import-input'), [
      new File([OFX], 'extrato.ofx', { type: 'application/x-ofx' }),
      new File(['%PDF-1.4 NOTA DE CORRETAGEM'], 'nota.pdf', { type: 'application/pdf' }),
    ]);
    const statements = await within(hub).findByRole('region', {
      name: 'Statements and card bills',
    });
    expect(within(statements).getByText('extrato.ofx')).toBeTruthy();
    expect(within(hub).getByRole('button', { name: 'Import note' })).toBeTruthy();
    await user.click(within(statements).getByRole('button', { name: 'Import' }));

    const wizard = await screen.findByRole('dialog', { name: 'Import statement' });
    expect(within(wizard).getByRole('list', { name: 'Steps' }).textContent).toContain('Preview');
    expect(await within(wizard).findByText('Account statement (OFX)')).toBeTruthy();
    expect(within(wizard).getByText('Encoding')).toBeTruthy();
    expect(within(wizard).getByRole('combobox', { name: 'Destination account' })).toBeTruthy();
  });

  it('aviso da pasta Downloads em espanhol', async () => {
    await applyLocale('es');
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Movimientos/ });
    act(() =>
      simulateDownloadOffer(
        { path: '/Downloads/a.ofx', name: 'a.ofx', size: 10, kind: 'statement', label: 'OFX' },
        btoa('OFXHEADER:100'),
      ),
    );
    const card = await screen.findByRole('region', { name: 'Archivos nuevos en Descargas' });
    expect(within(card).getByText('Archivo nuevo en Descargas')).toBeTruthy();
    expect(within(card).getByRole('button', { name: 'Ahora no' })).toBeTruthy();
  });
});
