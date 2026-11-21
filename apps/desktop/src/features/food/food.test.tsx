import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { Purchase } from '@/api/generated/Purchase';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';
import { todayIso } from '@/lib/format';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

/** Um cupom inventado (loja e produtos fictícios) pago por um lançamento. */
async function seedReceipt() {
  const acc = ((await mockCall('accounts.list', {})) as AccountView[]).find(
    (a) => a.kind === 'checking',
  )!;
  await mockCall('transactions.create', {
    account_id: acc.id,
    date: todayIso(),
    amount: -4_000,
    kind: 'expense',
    description: 'Mercado fictício',
  });
  await mockCall('receipts.import', {
    manual: {
      date: todayIso(),
      store_name: 'Mercado Fictício',
      items: [
        { description: 'ARROZ EXEMPLO 5KG', total: 2_500 },
        { description: 'REFRIGERANTE EXEMPLO 2L', total: 900 },
        { description: 'DETERGENTE EXEMPLO 500ML', total: 600 },
      ],
    },
  });
}

describe('06 · Consumo', () => {
  it('visão geral por segmento e produtos filtrados pelo segmento', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('heading', { level: 1 });
    await seedReceipt();
    window.location.hash = '#/consumo';
    expect(await screen.findByRole('heading', { level: 1, name: /Consumo/ })).toBeTruthy();
    expect((await screen.findAllByText('Comida em casa')).length).toBeGreaterThan(0);
    const segments = await screen.findByRole('list', { name: 'Segmentos' });
    expect(within(segments).getByText('Alimentação')).toBeTruthy();
    expect(within(segments).getByText('Casa e limpeza')).toBeTruthy();
    expect(screen.getByRole('list', { name: 'Essencial, complementar e supérfluo' })).toBeTruthy();
    // Escolher um segmento detalha as categorias dele.
    await user.click(within(segments).getByText('Casa e limpeza'));
    expect(await screen.findByText('Categorias de Casa e limpeza')).toBeTruthy();

    await user.click(screen.getByRole('tab', { name: 'Produtos' }));
    const grid = await screen.findByRole('grid', { name: 'Produtos' });
    // O segmento escolhido vale também aqui.
    expect(within(grid).getByText('DETERGENTE EXEMPLO 500ML')).toBeTruthy();
    expect(within(grid).queryByText('ARROZ EXEMPLO 5KG')).toBeNull();
    await user.click(
      within(screen.getByRole('group', { name: 'Segmentos' })).getByRole('button', {
        name: 'Todos',
      }),
    );
    const all = await screen.findByRole('grid', { name: 'Produtos' });
    await waitFor(() => expect(within(all).getByText('ARROZ EXEMPLO 5KG')).toBeTruthy());
    // 25,00 por 5 kg: 5,00 o kg.
    expect(within(all).getAllByText(/\/kg/).length).toBeGreaterThan(0);
  });

  it('nova compra à mão fora do Brasil, com a despesa criada junto', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'US' }, '#/consumo?aba=compras');
    expect(await screen.findByRole('heading', { level: 1, name: /Consumo/ })).toBeTruthy();
    expect(document.querySelector('a[href="#/consumo"]')).not.toBeNull();
    // A NFC-e é do Brasil; fora dele, o cupom entra só pela foto (docs/ocr-receipts.md).
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    expect(await screen.findByRole('region', { name: 'Ler cupom pela foto' })).toBeTruthy();
    // Só imagem: PDF e XML são das notas do Brasil.
    expect(screen.getByLabelText<HTMLInputElement>('Arquivo do cupom').accept).toBe('image/*');
    await user.click(screen.getByRole('button', { name: 'Cadastrar à mão' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    await user.type(within(dialog).getByLabelText('Loja'), 'Loja de Roupas Inventada');
    await user.type(within(dialog).getByLabelText('Descrição do item 1'), 'Camiseta básica');
    await user.clear(within(dialog).getByLabelText('Quantidade do item 1'));
    await user.type(within(dialog).getByLabelText('Quantidade do item 1'), '2');
    await user.type(within(dialog).getByLabelText('Preço unitário do item 1'), '15,00');
    await user.click(within(dialog).getByRole('button', { name: 'Adicionar item' }));
    await user.type(within(dialog).getByLabelText('Descrição do item 2'), 'Meia');
    await user.type(within(dialog).getByLabelText('Total do item 2'), '10,00');
    await user.click(within(dialog).getByRole('button', { name: 'Gravar compra' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    const list = screen.getByRole('grid', { name: 'Compras' });
    expect(await within(list).findByText('Loja de Roupas Inventada')).toBeTruthy();
    const r = (await mockCall('food.purchases', {})) as Purchase[];
    expect(r[0]!.receipt.paid).toBe(4_000);
    expect(r[0]!.receipt.txn_created).toBe(true);
    expect(r[0]!.segments[0]!.name).toBe('Vestuário e calçados');
    const txn = (await mockCall('transactions.get', { id: r[0]!.receipt.txn_id })) as TxnView;
    expect(txn.amount).toBe(-4_000);

    // Editar: o lançamento criado junto acompanha o valor.
    await user.click(
      within(list).getByRole('button', { name: 'Editar a compra de Loja de Roupas Inventada' }),
    );
    const edit = await screen.findByRole('dialog', { name: 'Editar compra' });
    await user.click(within(edit).getByRole('button', { name: 'Tirar o item 2' }));
    await user.click(within(edit).getByRole('button', { name: 'Gravar compra' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const after = (await mockCall('transactions.get', { id: r[0]!.receipt.txn_id })) as TxnView;
    expect(after.amount).toBe(-3_000);
  });

  it('as categorias de produto ficam em Ajustes, com os segmentos no topo', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=categorias');
    await user.click(await screen.findByRole('radio', { name: 'Produtos (compras)' }));
    const list = await screen.findByRole('list', { name: 'Categorias de produto' });
    expect(within(list).getByText('Vestuário e calçados')).toBeTruthy();
    expect(within(list).getByText('Hortifrúti')).toBeTruthy();
    await user.click(within(list).getAllByTitle('Nova subcategoria')[0]!);
    await user.type(screen.getByLabelText('Nome da categoria'), 'Frutas da estação');
    await user.click(screen.getByRole('button', { name: 'Criar' }));
    await waitFor(() => expect(within(list).getByText('Frutas da estação')).toBeTruthy());
  });

  it('Preços: as mesmas compras um ano antes e a cesta contra o IPCA de alimentos', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo');
    const card = await screen.findByRole('region', { name: /Inflação da sua cesta/ });
    expect(
      within(card).getByText(/As mesmas compras que você fez nos últimos 3 meses/),
    ).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Ver preços' }));
    expect(await screen.findByRole('tab', { name: 'Preços', selected: true })).toBeTruthy();
    const movers = await screen.findByRole('region', { name: /O que mais mudou de preço/ });
    expect(within(movers).getByText('Café Exemplo Torrado 500g')).toBeTruthy();
    expect(within(movers).getByText('+30%')).toBeTruthy();
  });

  it('código de barras digitado no produto e na linha da compra', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    // Uma compra com um produto, para ele aparecer em Produtos.
    await mockCall('food.purchase_save', {
      date: todayIso(),
      store_name: 'Mercadinho Inventado',
      items: [{ description: 'Café Exemplo 500g', total: 2000 }],
      payment: { mode: 'none' },
    });
    await user.click(await screen.findByRole('tab', { name: 'Produtos' }));
    await user.click(await screen.findByText('Café Exemplo 500g'));
    const field = await screen.findByLabelText('Novo código de barras');
    // Dígito verificador errado: a mensagem diz o que está errado.
    await user.type(field, '7890000000018');
    await user.click(screen.getByRole('button', { name: 'Adicionar' }));
    expect(await screen.findByText(/dígito verificador/)).toBeTruthy();
    await user.clear(field);
    await user.type(field, '7890000000017');
    await user.click(screen.getByRole('button', { name: 'Adicionar' }));
    const codes = await screen.findByRole('list', { name: 'Códigos de barras do produto' });
    expect(within(codes).getByText('7890000000017')).toBeTruthy();

    // Na Nova compra, o campo fica recolhido até a pessoa abrir.
    await user.click(screen.getByRole('tab', { name: 'Compras' }));
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    await user.click(await screen.findByRole('button', { name: 'Cadastrar à mão' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    expect(
      within(dialog).queryByRole('textbox', { name: 'Código de barras do item 1' }),
    ).toBeNull();
    await user.click(within(dialog).getByRole('button', { name: 'Código de barras do item 1' }));
    expect(
      within(dialog).getByRole('textbox', { name: 'Código de barras do item 1' }),
    ).toBeTruthy();
  });

  it('detalhar só parte de um lançamento, a partir de Lançamentos', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo');
    await screen.findByRole('heading', { level: 1, name: /Consumo/ });
    const acc = ((await mockCall('accounts.list', {})) as { id: string }[])[0]!.id;
    const txn = (await mockCall('transactions.create', {
      account_id: acc,
      date: todayIso(),
      amount: -31_247,
      kind: 'expense',
      description: 'Mercado Inventado',
    })) as TxnView;
    window.location.hash = `#/lancamentos?detalhar=${txn.id}`;
    // A Compra abre ligada ao lançamento, com a loja e sem itens.
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    expect(within(dialog).getByDisplayValue('Mercado Inventado')).toBeTruthy();
    expect(
      within(dialog).getByRole<HTMLInputElement>('radio', { name: 'Ligar a um lançamento' })
        .ariaChecked,
    ).toBe('true');
    await user.type(within(dialog).getByLabelText('Descrição do item 1'), 'Patinho moído');
    await user.type(within(dialog).getByLabelText('Total do item 1'), '48,20');
    // O resto do lançamento fica sem detalhe, e isso aparece antes de gravar.
    expect(await within(dialog).findByText(/do lançamento ficam sem detalhe/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Gravar compra' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const r = (await mockCall('food.purchases', {})) as Purchase[];
    expect(r[0]!.receipt.txn_id).toBe(txn.id);
    expect(r[0]!.receipt.paid).toBe(4_820);
    const after = (await mockCall('transactions.get', { id: txn.id })) as TxnView;
    expect(after.amount).toBe(-31_247);
  });

  it('colar os itens como texto e repetir a compra', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    await user.click(await screen.findByText('Colar os itens como texto'));
    await user.type(
      screen.getByRole('textbox', { name: 'Colar os itens como texto' }),
      'Arroz 5kg 24,90{Enter}Feijao 1kg 8,49',
    );
    await user.click(screen.getByRole('button', { name: 'Ler os itens' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    expect(within(dialog).getByDisplayValue('Arroz 5kg')).toBeTruthy();
    expect(within(dialog).getByDisplayValue('Feijao 1kg')).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Loja'), 'Mercado da Esquina Inventado');
    await user.click(within(dialog).getByRole('radio', { name: 'Sem lançamento' }));
    await user.click(within(dialog).getByRole('button', { name: 'Gravar compra' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    // Repetir: os mesmos itens, na mesma loja, numa compra nova.
    const list = screen.getByRole('grid', { name: 'Compras' });
    await user.click(
      await within(list).findByRole('button', {
        name: 'Repetir a compra de Mercado da Esquina Inventado',
      }),
    );
    const again = await screen.findByRole('dialog', { name: 'Nova compra' });
    expect(within(again).getByDisplayValue('Mercado da Esquina Inventado')).toBeTruthy();
    expect(within(again).getByDisplayValue('Arroz 5kg')).toBeTruthy();
    expect(within(again).getByDisplayValue('Feijao 1kg')).toBeTruthy();
  });

  it('EUA: o imposto somado no fim do cupom entra no pago, e as unidades incluem libra', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'US' }, '#/consumo?aba=compras');
    await user.click(await screen.findByRole('button', { name: 'Adicionar compra' }));
    await user.click(await screen.findByRole('button', { name: 'Cadastrar à mão' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova compra' });
    await user.type(within(dialog).getByLabelText('Loja'), 'Fresh Market Exemplo');
    await user.type(within(dialog).getByLabelText('Descrição do item 1'), 'Paper towels');
    await user.type(within(dialog).getByLabelText('Total do item 1'), '10,00');
    // A quantidade com a unidade digitada junto: a libra vai para o campo da unidade.
    const qty = within(dialog).getByLabelText('Quantidade do item 1');
    await user.clear(qty);
    await user.type(qty, '2,5 lb');
    await user.tab();
    expect(
      within(dialog).getByRole('combobox', { name: 'Unidade do item 1' }).textContent,
    ).toContain('lb');
    await user.type(await within(dialog).findByLabelText('Imposto somado'), '0,73');
    await user.click(within(dialog).getByRole('radio', { name: 'Sem lançamento' }));
    await user.click(within(dialog).getByRole('button', { name: 'Gravar compra' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const r = (await mockCall('food.purchases', {})) as Purchase[];
    expect(r[0]!.receipt.paid).toBe(1_073);
    expect(r[0]!.receipt.tax_added).toBe(73);
  });
});
