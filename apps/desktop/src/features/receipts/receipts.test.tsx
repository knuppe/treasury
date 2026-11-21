/**
 * 06 · Consumption › Purchases › Add purchase against the mock: read the receipt from the QR link, check
 * the items (locked) with category and tax, save and see what the family buys.
 */
import { cleanup, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { Receipt } from '@/api/generated/Receipt';
import type { ReceiptSummary } from '@/api/generated/ReceiptSummary';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

describe('06 · Consumption › Fiscal receipts', () => {
  it('reads the receipt from the QR link, shows the items and saves', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    await user.type(
      await screen.findByLabelText('Link do QR do cupom'),
      'https://www.fazenda.pr.gov.br/nfce/qrcode?p=41261012345678000190650010000123451000123452|2|1|1|X',
    );
    await user.click(screen.getByRole('button', { name: 'Ler cupom' }));
    // Steps and result before opening the review.
    const steps = await screen.findByRole('list', { name: 'Etapas da leitura' });
    expect(within(steps).getByText('Consultar a nota na SEFAZ')).toBeTruthy();
    const result = await screen.findByRole('region', { name: 'Resultado da leitura' });
    expect(within(result).getByText(/4 item\(ns\) de SUPERMERCADO EXEMPLO LTDA/)).toBeTruthy();
    await user.click(within(result).getByRole('button', { name: 'Revisar e gravar' }));
    const dialog = await screen.findByRole('dialog', { name: /Cupom de SUPERMERCADO EXEMPLO/ });
    // What the document says is locked; each item's category is chosen.
    const first = within(dialog).getByDisplayValue<HTMLInputElement>('ARROZ TIO JOAO T1 5KG');
    expect(first.disabled).toBe(true);
    expect(within(dialog).queryByRole('button', { name: 'Adicionar item' })).toBeNull();
    expect(within(dialog).getByText(/tributos aproximados da nota/)).toBeTruthy();
    await user.click(within(dialog).getByRole('combobox', { name: 'Categoria do item 2' }));
    await user.click(await screen.findByRole('option', { name: 'Outros' }));
    await user.click(within(dialog).getByRole('button', { name: 'Gravar compra' }));
    // Saved: the review and the entry door close.
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    const s = (await mockCall('receipts.summary', {})) as ReceiptSummary;
    expect(s.receipts).toBe(1);
    const saved = (await mockCall('receipts.list', {})) as { id: string }[];
    const full = (await mockCall('receipts.get', { id: saved[0]!.id })) as Receipt;
    expect(full.items[1]!.category_id).toBeTruthy();
    expect(full.items[0]!.category_id).toBeNull();
    const card = await screen.findByRole('region', { name: /O que a família compra/ });
    expect(await within(card).findByText('Limpeza')).toBeTruthy();
    const list = screen.getByRole('region', { name: /Compras/ });
    expect(await within(list).findByText('SUPERMERCADO EXEMPLO LTDA')).toBeTruthy();
  });

  it('Argentine invoice via the AFIP QR: opens the filled purchase, without items', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    const json =
      '{"ver":1,"fecha":"2026-10-20","cuit":30712345678,"ptoVta":3,"tipoCmp":6,"nroCmp":12345,"importe":15234.5,"moneda":"PES","ctz":1}';
    await user.type(
      await screen.findByLabelText('Link do QR do cupom'),
      `https://www.afip.gob.ar/fe/qr/?p=${btoa(json)}`,
    );
    await user.click(screen.getByRole('button', { name: 'Ler cupom' }));
    const result = await screen.findByRole('region', { name: 'Resultado da leitura' });
    expect(within(result).getByText(/Nota da Argentina: Factura B 0003-00012345/)).toBeTruthy();
    await user.click(within(result).getByRole('button', { name: 'Revisar e gravar' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    expect(within(dialog).getByText(/Nota da Argentina lida pelo QR da AFIP/)).toBeTruthy();
    expect(within(dialog).getByText(/não traz os itens/)).toBeTruthy();
    expect(within(dialog).getByDisplayValue('CUIT 30-71234567-8')).toBeTruthy();
    expect(within(dialog).getByDisplayValue('Factura B 0003-00012345')).toBeTruthy();
  });

  it('link from another country: says where it is from and what can be done', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    await user.type(
      await screen.findByLabelText('Link do QR do cupom'),
      'https://www.sii.cl/boleta?x=1',
    );
    await user.click(screen.getByRole('button', { name: 'Ler cupom' }));
    expect(
      await screen.findByText(/não é de uma nota fiscal do Brasil \(www.sii.cl\)/),
    ).toBeTruthy();
  });

  it('photo without QR: reads the receipt text, shows what did not add up and asks the AI for help', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    const photo = new File([new Uint8Array([1, 2, 3])], 'cupom.jpg', { type: 'image/jpeg' });
    await user.upload(await screen.findByLabelText('Arquivo do cupom'), photo);
    // No QR in the photo: reading continues from the text, and the result says the sum does not match.
    const steps = await screen.findByRole('list', { name: 'Etapas da leitura' });
    expect(within(steps).getByText(/sem QR legível/)).toBeTruthy();
    const result = await screen.findByRole('region', { name: 'Resultado da leitura' });
    expect(within(result).getByText(/A soma não bate/)).toBeTruthy();
    await user.click(within(result).getByRole('button', { name: 'Revisar e corrigir' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    const notice = within(dialog).getByRole('region', { name: 'Leitura da foto' });
    expect(
      within(notice).getByText(/A soma dos itens não bate com o total\. Confira e corrija/),
    ).toBeTruthy();
    expect(within(dialog).getByDisplayValue('PADARIA EXEMPLO')).toBeTruthy();
    expect(within(dialog).getByDisplayValue('LEITE INTEGRAL 1L')).toBeTruthy();
    await user.click(within(notice).getByRole('button', { name: 'Pedir ajuda à IA' }));
    const again = await screen.findByRole('dialog', { name: 'Nova compra' });
    expect(await within(again).findByText(/Lido pela IA/)).toBeTruthy();
    expect(within(again).getByDisplayValue('CAFE TORRADO 500G')).toBeTruthy();
    expect(within(again).getByText(/A soma dos itens bate com o total/)).toBeTruthy();
  });

  it('Transactions no longer has the receipts tab, and the old link opens Consumption › Purchases', async () => {
    renderApp('unlocked', {}, '#/lancamentos');
    expect(await screen.findByRole('tab', { name: 'Lançamentos' })).toBeTruthy();
    expect(screen.queryByRole('tab', { name: 'Cupons fiscais' })).toBeNull();
    cleanup();
    renderApp('unlocked', {}, '#/lancamentos?aba=cupons');
    expect(await screen.findByRole('heading', { level: 1, name: /Consumo/ })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Compras', selected: true })).toBeTruthy();
  });
});
