import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { TransferCandidate } from '@/api/generated/TransferCandidate';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall } from '@/api/mock';
import { todayIso } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('transferências entre contas próprias', () => {
  it('acha o salário passado para outra conta, liga e recusa', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const day = todayIso();
    const out = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: day,
      amount: -321_000,
      description: 'PIX ENVIADO NUBANK',
    })) as TxnView;
    const inn = (await mockCall('transactions.create', {
      account_id: 'acc-nuconta',
      date: day,
      amount: 321_000,
      description: 'PIX RECEBIDO',
    })) as TxnView;
    const list = (await mockCall('transactions.transfer_candidates', {})) as TransferCandidate[];
    const ours = list.find((c) => c.from.txn_id === out.id);
    expect(ours?.to.txn_id).toBe(inn.id);
    expect(ours?.reason).toMatch(/mesmo valor no mesmo dia/);

    // Escrita por fora da interface (como o agente): a tela relê.
    await queryClient.invalidateQueries({ queryKey: ['transactions'] });
    const banner = await screen.findByText(/parecem? transferências? entre suas contas/);
    await user.click(
      within(banner.closest('[role="status"]')!).getByRole('button', { name: 'Revisar' }),
    );
    const dialog = await screen.findByRole('dialog', { name: 'Transferências entre suas contas' });
    const pairs = within(dialog).getByRole('list', { name: 'Pares sugeridos' });
    expect(within(pairs).getByText(/PIX ENVIADO NUBANK/)).toBeTruthy();
    const link = within(dialog).getByRole('button', { name: /^Ligar \d+ pa/ });
    await user.click(link);
    await waitFor(async () =>
      expect(((await mockCall('transactions.get', { id: out.id })) as TxnView).kind).toBe(
        'transfer',
      ),
    );
    const after = (await mockCall('transactions.get', { id: inn.id })) as TxnView;
    expect(after.kind).toBe('transfer');
    expect(after.category_id).toBeNull();
    expect(
      ((await mockCall('transactions.transfer_candidates', {})) as TransferCandidate[]).some(
        (c) => c.from.txn_id === out.id,
      ),
    ).toBe(false);
  });

  it('"não é" tira o par da lista', async () => {
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const day = todayIso();
    const out = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: day,
      amount: -7_700,
      description: 'X',
    })) as TxnView;
    const inn = (await mockCall('transactions.create', {
      account_id: 'acc-nuconta',
      date: day,
      amount: 7_700,
      description: 'Y',
    })) as TxnView;
    await mockCall('transactions.dismiss_transfer', { from_id: out.id, to_id: inn.id });
    const list = (await mockCall('transactions.transfer_candidates', {})) as TransferCandidate[];
    expect(list.some((c) => c.from.txn_id === out.id && c.to.txn_id === inn.id)).toBe(false);
  });

  it('o seletor de categoria da linha abre fora da tabela (não é cortado pela célula)', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    const grid = await screen.findByRole('grid', { name: 'Lançamentos' });
    const cell = within(grid).getAllByRole('button', { name: /^Categoria de / })[0]!;
    await user.click(cell);
    const search = await screen.findByRole('combobox', { name: /^Buscar categoria de / });
    expect(grid.contains(search)).toBe(false);
    expect(document.activeElement).toBe(search);
    await user.keyboard('{Escape}');
    await waitFor(() =>
      expect(screen.queryByRole('combobox', { name: /^Buscar categoria de / })).toBeNull(),
    );
  });

  it('na edição, despesa vira transferência para terceiros ou para outra conta da família', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const out = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: todayIso(),
      amount: -1_000_000,
      description: 'DEBITO PIX - CORRETORA',
    })) as TxnView;
    await queryClient.invalidateQueries();
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    await user.dblClick((await within(grid).findByText('DEBITO PIX - CORRETORA')).closest('tr')!);
    const dialog = await screen.findByRole('dialog', { name: 'Editar lançamento' });
    const tipo = within(dialog).getByRole('radiogroup', { name: 'Tipo do lançamento' });
    await user.click(within(tipo).getByRole('radio', { name: 'Transferência' }));
    expect(within(dialog).getByLabelText('Destinatário')).toBeTruthy();
    expect(within(dialog).queryByRole('checkbox', { name: /Dividir/ })).toBeNull();
    await user.click(within(dialog).getByRole('combobox', { name: 'Para a conta da família' }));
    const option = (await screen.findAllByRole('option')).find(
      (o) => /Nu/.test(o.textContent ?? '') && !/Cart/.test(o.textContent ?? ''),
    )!;
    const target = option.textContent.replace(/BRL$/, '').trim();
    await user.click(option);
    await user.click(within(dialog).getByRole('button', { name: 'Salvar alterações' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Editar lançamento' })).toBeNull(),
    );
    const saved = (await mockCall('transactions.get', { id: out.id })) as TxnView;
    expect(saved.kind).toBe('transfer');
    expect(saved.transfer_id).toBeTruthy();
    expect(saved.counterpart_account_name).toBe(target);
  });

  it('transferência mostra e aceita categoria na lista', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const t = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: todayIso(),
      amount: -8_000,
      kind: 'transfer',
      description: 'DEBITO PIX - DINHO ESTACIONAMENTO',
    })) as TxnView;
    await queryClient.invalidateQueries();
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    await within(grid).findByText('DEBITO PIX - DINHO ESTACIONAMENTO');
    await user.click(
      within(grid).getByRole('button', {
        name: /Categoria de DEBITO PIX - DINHO ESTACIONAMENTO/,
      }),
    );
    await user.type(
      screen.getByRole('combobox', {
        name: 'Buscar categoria de debito pix - dinho estacionamento',
      }),
      'estacionamento',
    );
    await user.keyboard('{Enter}');
    await waitFor(async () =>
      expect(((await mockCall('transactions.get', { id: t.id })) as TxnView).category_path).toMatch(
        /Estacionamento/,
      ),
    );
  });

  it('pelo menu da linha liga à entrada que já existe na outra conta', async () => {
    const user = userEvent.setup();
    const day = todayIso();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos?q=GUARDAR');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const out = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: day,
      amount: -50_000,
      description: 'PIX GUARDAR',
    })) as TxnView;
    const inn = (await mockCall('transactions.create', {
      account_id: 'acc-poupanca',
      date: day,
      amount: 50_000,
      description: 'GUARDAR RECEBIDO',
    })) as TxnView;
    await queryClient.invalidateQueries({ queryKey: ['transactions'] });
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    await user.pointer({
      keys: '[MouseRight]',
      target: await within(grid).findByText('PIX GUARDAR'),
    });
    await user.click(
      within(screen.getByRole('menu', { name: 'Ações de PIX GUARDAR' })).getByRole('menuitem', {
        name: /Transferência para outra conta/,
      }),
    );
    const dialog = await screen.findByRole('dialog', { name: 'Transferência para outra conta' });
    await user.click(within(dialog).getByRole('combobox', { name: 'Para a conta' }));
    await user.click(await screen.findByRole('option', { name: /Poupança Caixa/ }));
    const status = within(dialog).getByRole('status', { name: 'O que vai acontecer' });
    expect(await within(status).findByText(/Achei a entrada em Poupança Caixa/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Ligar as contas' }));
    await waitFor(async () => {
      const t = (await mockCall('transactions.get', { id: out.id })) as TxnView;
      expect(t).toMatchObject({ kind: 'transfer', counterpart_account_id: 'acc-poupanca' });
    });
    const back = (await mockCall('transactions.get', { id: inn.id })) as TxnView;
    expect(back.kind).toBe('transfer');
  });

  it('pagamento de fatura: liga a saída ao cartão e mostra a fatura que quita', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos?q=PAGTO FATURA');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const bill = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: todayIso(),
      amount: -120_000,
      description: 'PAGTO FATURA',
    })) as TxnView;
    await queryClient.invalidateQueries({ queryKey: ['transactions'] });
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    await user.pointer({
      keys: '[MouseRight]',
      target: await within(grid).findByText('PAGTO FATURA'),
    });
    await user.click(
      within(screen.getByRole('menu', { name: 'Ações de PAGTO FATURA' })).getByRole('menuitem', {
        name: /Pagamento de fatura/,
      }),
    );
    const dialog = await screen.findByRole('dialog', { name: 'Pagamento de fatura' });
    await user.click(within(dialog).getByRole('combobox', { name: 'Cartão' }));
    await user.click(await screen.findByRole('option', { name: /Cartão Nubank/ }));
    const status = within(dialog).getByRole('status', { name: 'O que vai acontecer' });
    expect(await within(status).findByText(/Vira pagamento de fatura/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Ligar ao cartão' }));
    await waitFor(async () => {
      const t = (await mockCall('transactions.get', { id: bill.id })) as TxnView;
      expect(t).toMatchObject({ kind: 'card_payment', counterpart_account_id: 'acc-nucard' });
    });
  });

  it('duas marcadas (saída e entrada) viram transferência pelo menu do lote', async () => {
    const user = userEvent.setup();
    const day = todayIso();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos?q=PAR MANUAL');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const out = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: day,
      amount: -9_900,
      description: 'PAR MANUAL SAIDA',
    })) as TxnView;
    await mockCall('transactions.create', {
      account_id: 'acc-nuconta',
      date: day,
      amount: 9_900,
      description: 'PAR MANUAL ENTRADA',
    });
    await queryClient.invalidateQueries({ queryKey: ['transactions'] });
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    for (const d of ['PAR MANUAL SAIDA', 'PAR MANUAL ENTRADA'])
      await user.click(await within(grid).findByRole('checkbox', { name: new RegExp(d) }));
    await user.pointer({
      keys: '[MouseRight]',
      target: within(grid).getByText('PAR MANUAL SAIDA'),
    });
    const menu = screen.getByRole('menu', { name: 'Ações de 2 lançamentos marcados' });
    await user.click(
      within(menu).getByRole('menuitem', { name: 'Ligar as duas como transferência' }),
    );
    expect(await screen.findByText('As duas pernas ficaram ligadas.')).toBeTruthy();
    const t = (await mockCall('transactions.get', { id: out.id })) as TxnView;
    expect(t).toMatchObject({ kind: 'transfer', counterpart_account_id: 'acc-nuconta' });
  });

  it('a barra do lote liga as duas, a lista mostra a ligação e o menu desliga', async () => {
    const user = userEvent.setup();
    const day = todayIso();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos?q=PIX PROPRIO');
    await screen.findByRole('grid', { name: 'Lançamentos' });
    const out = (await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: day,
      amount: -700_000,
      kind: 'transfer',
      description: 'PIX PROPRIO SAIDA',
    })) as TxnView;
    const inn = (await mockCall('transactions.create', {
      account_id: 'acc-nuconta',
      date: day,
      amount: 700_000,
      kind: 'transfer',
      description: 'PIX PROPRIO ENTRADA',
    })) as TxnView;
    await mockCall('transactions.create', {
      account_id: 'acc-itau',
      date: day,
      amount: 700_000,
      description: 'PIX PROPRIO ESTORNO',
    });
    await queryClient.invalidateQueries({ queryKey: ['transactions'] });
    const grid = screen.getByRole('grid', { name: 'Lançamentos' });
    const row = async (d: string) => (await within(grid).findByText(d)).closest('tr')!;
    // Soltas: a lista avisa.
    expect(
      within(await row('PIX PROPRIO SAIDA')).getByLabelText('Transferência solta'),
    ).toBeTruthy();

    // Duas na mesma conta: o botão diz por que não liga.
    for (const d of ['PIX PROPRIO SAIDA', 'PIX PROPRIO ESTORNO'])
      await user.click(within(grid).getByRole('checkbox', { name: new RegExp(d) }));
    const blocked = screen.getByRole('button', { name: 'Ligar como transferência' });
    expect(blocked).toHaveProperty('disabled', true);
    expect(blocked.getAttribute('title')).toMatch(/mesma conta/);
    await user.click(within(grid).getByRole('checkbox', { name: /PIX PROPRIO ESTORNO/ }));
    await user.click(within(grid).getByRole('checkbox', { name: /PIX PROPRIO ENTRADA/ }));
    await user.click(screen.getByRole('button', { name: 'Ligar como transferência' }));
    expect(await screen.findByText('As duas pernas ficaram ligadas.')).toBeTruthy();
    const linked = (await mockCall('transactions.get', { id: inn.id })) as TxnView;
    expect(linked.counterpart_account_id).toBe('acc-itau');
    expect(
      await within(await row('PIX PROPRIO ENTRADA')).findByLabelText(/^Ligada a /),
    ).toBeTruthy();

    // Menu da linha: desligar deixa as duas soltas.
    await user.pointer({
      keys: '[MouseRight]',
      target: within(grid).getByText('PIX PROPRIO SAIDA'),
    });
    await user.click(await screen.findByRole('menuitem', { name: /^Desligar de / }));
    expect(await screen.findByText(/As duas pernas ficaram soltas/)).toBeTruthy();
    for (const id of [out.id, inn.id]) {
      const t = (await mockCall('transactions.get', { id })) as TxnView;
      expect(t).toMatchObject({ kind: 'transfer', transfer_id: null });
    }
  });
});
