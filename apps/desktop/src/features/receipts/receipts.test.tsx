/**
 * 06 · Consumo › Compras › Adicionar compra contra o mock: ler o cupom pelo link do QR, conferir
 * os itens (travados) com a categoria e o imposto, gravar e ver o que a família compra.
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

describe('06 · Consumo › Cupons fiscais', () => {
  it('lê o cupom pelo link do QR, mostra os itens e grava', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    await user.type(
      await screen.findByLabelText('Link do QR do cupom'),
      'https://www.fazenda.pr.gov.br/nfce/qrcode?p=41261012345678000190650010000123451000123452|2|1|1|X',
    );
    await user.click(screen.getByRole('button', { name: 'Ler cupom' }));
    // Etapas e resultado antes de abrir a revisão.
    const steps = await screen.findByRole('list', { name: 'Etapas da leitura' });
    expect(within(steps).getByText('Consultar a nota na SEFAZ')).toBeTruthy();
    const result = await screen.findByRole('region', { name: 'Resultado da leitura' });
    expect(within(result).getByText(/4 item\(ns\) de SUPERMERCADO EXEMPLO LTDA/)).toBeTruthy();
    await user.click(within(result).getByRole('button', { name: 'Revisar e gravar' }));
    const dialog = await screen.findByRole('dialog', { name: /Cupom de SUPERMERCADO EXEMPLO/ });
    // O que o documento diz fica travado; a categoria de cada item se escolhe.
    const first = within(dialog).getByDisplayValue<HTMLInputElement>('ARROZ TIO JOAO T1 5KG');
    expect(first.disabled).toBe(true);
    expect(within(dialog).queryByRole('button', { name: 'Adicionar item' })).toBeNull();
    expect(within(dialog).getByText(/tributos aproximados da nota/)).toBeTruthy();
    await user.click(within(dialog).getByRole('combobox', { name: 'Categoria do item 2' }));
    await user.click(await screen.findByRole('option', { name: 'Outros' }));
    await user.click(within(dialog).getByRole('button', { name: 'Gravar compra' }));
    // Gravou: a revisão e a porta de entrada fecham.
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

  it('nota da Argentina pelo QR da AFIP: abre a compra preenchida, sem os itens', async () => {
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

  it('link de outro país: diz de onde é e o que dá para fazer', async () => {
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

  it('foto sem QR: lê o texto do cupom, mostra o que não fechou e pede ajuda à IA', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    const photo = new File([new Uint8Array([1, 2, 3])], 'cupom.jpg', { type: 'image/jpeg' });
    await user.upload(await screen.findByLabelText('Arquivo do cupom'), photo);
    // Sem QR na foto: a leitura segue pelo texto, e o resultado diz que a soma não bate.
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

  it('Lançamentos não tem mais a aba dos cupons, e o link antigo abre Consumo › Compras', async () => {
    renderApp('unlocked', {}, '#/lancamentos');
    expect(await screen.findByRole('tab', { name: 'Lançamentos' })).toBeTruthy();
    expect(screen.queryByRole('tab', { name: 'Cupons fiscais' })).toBeNull();
    cleanup();
    renderApp('unlocked', {}, '#/lancamentos?aba=cupons');
    expect(await screen.findByRole('heading', { level: 1, name: /Consumo/ })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Compras', selected: true })).toBeTruthy();
  });
});
