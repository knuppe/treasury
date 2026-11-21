import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { OverdraftSummary } from './Overdraft';

// The ECharts canvas does not exist in jsdom: the chart becomes a marker.
vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

async function accounts() {
  return (await mockCall('accounts.list', {})) as AccountView[];
}

describe('03 · Accounts', () => {
  it("lists the accounts with type, members and balance, and opens each one's page", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    expect(await screen.findByRole('heading', { level: 1, name: /Contas/ })).toBeTruthy();
    const grid = await screen.findByRole('grid', { name: 'Contas' });
    const usd = within(grid).getByRole('row', { name: /Wise USD/ });
    // Exchange rate older than 7 days: estimated value in the base currency, with an origin badge.
    expect(within(usd).getByRole('button', { name: /Origem: Câmbio/ })).toBeTruthy();
    expect(within(grid).getByRole('row', { name: /Cartão Nubank/ })).toBeTruthy();
    // The card opens on its own page, with available credit and statements.
    await user.click(within(grid).getByRole('button', { name: 'Abrir Cartão Nubank' }));
    expect(await screen.findByRole('heading', { level: 1, name: /Cartão Nubank/ })).toBeTruthy();
    expect(screen.queryByRole('grid', { name: 'Contas' })).toBeNull();
    expect(screen.getByText('Crédito disponível')).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Faturas' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: /Todas as contas/ }));
    expect(await screen.findByRole('grid', { name: 'Contas' })).toBeTruthy();
  });

  it('creates a checking account with holders and opens the detail', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await user.click(await screen.findByRole('button', { name: 'Nova conta' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova conta' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Conta Conjunta Sul');
    await user.type(within(dialog).getByLabelText(/Saldo inicial/), '1.500,00');
    await user.click(within(dialog).getByRole('checkbox', { name: 'Ana' }));
    await user.click(within(dialog).getByRole('button', { name: 'Criar conta' }));

    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Nova conta' })).toBeNull());
    const created = (await accounts()).find((a) => a.name === 'Conta Conjunta Sul');
    expect(created?.balance).toBe(150_000);
    expect(created?.member_ids).toHaveLength(1);
    // The account page opens and the URL keeps the account.
    expect(
      await screen.findByRole('heading', { level: 1, name: /Conta Conjunta Sul/ }),
    ).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Lançamentos' })).toBeTruthy();
    expect(window.location.hash).toContain(`conta=${created!.id}`);
  });

  it('picks the bank from the official list and saves branch and account', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await user.click(await screen.findByRole('button', { name: 'Nova conta' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova conta' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Conta Inter');
    await user.click(within(dialog).getByRole('combobox', { name: 'Instituição' }));
    await user.click(await screen.findByRole('option', { name: '+ Nova instituição…' }));
    await user.type(within(dialog).getByLabelText('Nova instituição'), '077');
    await user.click(await within(dialog).findByRole('button', { name: /Banco Inter S\.A\./ }));
    expect(within(dialog).getByText(/o código reconhece os extratos OFX/)).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Agência'), '0001');
    await user.type(within(dialog).getByLabelText('Conta'), '1234567-8');
    await user.click(within(dialog).getByRole('button', { name: 'Criar conta' }));

    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Nova conta' })).toBeNull());
    const created = (await accounts()).find((a) => a.name === 'Conta Inter');
    expect(created).toMatchObject({ branch: '0001', account_number: '1234567-8' });
    const insts = (await mockCall('household.list_institutions', {})) as {
      id: string;
      code: string | null;
    }[];
    expect(insts.find((i) => i.id === created?.institution_id)?.code).toBe('077');
    await expect(
      mockCall('accounts.update', { id: created!.id, account_number: '12 34 a' }),
    ).rejects.toThrow(/use só números e hífen/);
  });

  it('creates a card with limit, closing day, due day and paying account', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await user.click(await screen.findByRole('button', { name: 'Novo cartão' }));
    const dialog = await screen.findByRole('dialog', { name: 'Nova conta' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Cartão Viagem');
    await user.type(within(dialog).getByLabelText(/Limite/), '8.000,00');
    const closing = within(dialog).getByLabelText('Dia do fechamento');
    await user.clear(closing);
    await user.type(closing, '25');
    const due = within(dialog).getByLabelText(/Dia do vencimento/);
    await user.clear(due);
    await user.type(due, '5');
    expect(within(dialog).getByText('Vence no mês seguinte ao fechamento.')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Criar conta' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Nova conta' })).toBeNull());
    const created = (await accounts()).find((a) => a.name === 'Cartão Viagem');
    expect(created?.card).toMatchObject({ credit_limit: 800_000, closing_day: 25, due_day: 5 });
    expect(created?.available_credit).toBe(800_000);
  });

  it('shows the core error when deleting an account with transactions', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await user.click(await screen.findByRole('button', { name: 'Abrir Carteira' }));
    await user.click(await screen.findByRole('button', { name: 'Editar' }));
    const dialog = await screen.findByRole('dialog', { name: /Editar Carteira/ });
    await user.click(within(dialog).getByRole('button', { name: 'Apagar' }));
    const confirm = await screen.findByRole('dialog', { name: /Apagar Carteira/ });
    await user.click(within(confirm).getByRole('button', { name: 'Apagar conta' }));
    expect(
      await within(confirm).findByText(
        /tem \d+ lançamento\(s\); apague-os antes ou arquive a conta/,
      ),
    ).toBeTruthy();
  });

  it('reconciles with the statement and marks the transactions as reconciled', async () => {
    const user = userEvent.setup();
    const checking = await (async () => {
      renderApp('unlocked', {}, '#/contas');
      return (await accounts())[0]!;
    })();
    await user.click(await screen.findByRole('button', { name: `Abrir ${checking.name}` }));
    await user.click(await screen.findByRole('tab', { name: 'Conciliação' }));
    const today = new Date();
    const d = `${String(today.getDate()).padStart(2, '0')}${String(today.getMonth() + 1).padStart(2, '0')}${today.getFullYear()}`;
    const date = screen.getByLabelText('Data do extrato');
    await user.clear(date);
    await user.type(date, d);
    const now = (await accounts())[0]!;
    await user.type(
      screen.getByLabelText(/Saldo no extrato/),
      (now.balance / 100).toFixed(2).replace('.', ','),
    );
    await user.click(screen.getByRole('button', { name: 'Comparar' }));
    expect(await screen.findByText('Confere ✓')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: /Marcar como conciliado/ }));
    expect(await screen.findByText(/marcados como conciliados/)).toBeTruthy();
    expect((await accounts())[0]!.reconciled_balance).toBe(now.balance);
  });

  it('syncs the balance with the bank: posts the difference as an adjustment and saves the milestone', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    const checking = (await accounts())[0]!;
    await user.click(await screen.findByRole('button', { name: `Abrir ${checking.name}` }));
    await user.click(await screen.findByRole('tab', { name: 'Conciliação' }));
    const today = new Date();
    const d = `${String(today.getDate()).padStart(2, '0')}${String(today.getMonth() + 1).padStart(2, '0')}${today.getFullYear()}`;
    const date = screen.getByLabelText('Data do extrato');
    await user.clear(date);
    await user.type(date, d);
    const now = (await accounts())[0]!;
    const bank = now.balance + 12_345;
    await user.type(
      screen.getByLabelText(/Saldo no extrato/),
      (bank / 100).toFixed(2).replace('.', ','),
    );
    await user.click(screen.getByRole('button', { name: 'Comparar' }));
    await user.click(await screen.findByRole('button', { name: 'Sincronizar com o banco' }));
    expect(await screen.findByText(/Saldo sincronizado com o banco/)).toBeTruthy();
    await waitFor(async () => expect((await accounts())[0]!.balance).toBe(bank));
    // The milestone is kept for investigation: reason noted by hand.
    expect(
      await screen.findByRole('button', { name: 'Pedir ao agente para investigar' }),
    ).toBeTruthy();
    await user.type(
      screen.getByLabelText('Motivo, se você já sabe'),
      'Tarifa que não veio no extrato',
    );
    await user.click(screen.getByRole('button', { name: 'Anotar o motivo' }));
    const list = await screen.findByRole('region', { name: 'Sincronias de saldo' });
    expect(await within(list).findByText(/Tarifa que não veio no extrato/)).toBeTruthy();
    expect(within(list).getByText('explicada')).toBeTruthy();
  });

  it('capital shares: enables, enters balances and computes the yield without counting contributions', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    const checking = (await accounts())[0]!;
    for (const d of ['2026-02-05', '2026-03-05']) {
      await mockCall('transactions.create', {
        account_id: checking.id,
        date: d,
        amount: -5_000,
        description: 'INTEGRALIZACAO COTA CAPITAL',
      });
    }
    await user.click(await screen.findByRole('button', { name: `Abrir ${checking.name}` }));
    await user.click(await screen.findByRole('tab', { name: 'Cotas capital' }));
    await user.click(
      await screen.findByRole('button', { name: 'Ativar cotas capital nesta conta' }),
    );
    const date = await screen.findByLabelText('Data do saldo');
    const inform = async (d: string, value: string) => {
      await user.clear(date);
      await user.type(date, d);
      const bal = screen.getByLabelText(/Saldo das cotas \(/);
      await user.clear(bal);
      await user.type(bal, value);
      await user.click(screen.getByRole('button', { name: 'Informar saldo' }));
      await screen.findAllByText('Saldo das cotas informado');
    };
    await inform('31012026', '1.000,00');
    await inform('31032026', '1.120,00'); // 100 in contributions + 20 in yield
    const table = await screen.findByRole('grid', { name: 'Saldos das cotas capital' });
    await waitFor(() =>
      expect(within(table).getAllByText(/R\$\s?20,00/).length).toBeGreaterThan(0),
    );
    const v = (await mockCall('accounts.capital_share', { account_id: checking.id })) as {
      total_yield: number;
      total_contributions: number;
    };
    expect(v.total_yield).toBe(2_000);
    expect(v.total_contributions).toBe(10_000);
  });

  it('overdraft: setup on the account, limit use and estimated cost', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await screen.findByRole('grid', { name: 'Contas' });
    const created = (await mockCall('accounts.create', {
      name: 'Cooperativa CC',
      kind: 'checking',
      opening_balance: -100_000,
      overdraft: {
        limit: 500_000,
        monthly_rate: '8,00',
        discount_pct: '25',
        annual_rate: null,
        iof_daily_rate: '0,0082',
        iof_additional_rate: '0,38',
        cet_month: null,
        cet_year: null,
      },
    })) as AccountView;
    expect(created.overdraft_status).toMatchObject({
      used: 100_000,
      available: 400_000,
      spendable: 400_000,
      estimated_interest: 6_000,
      estimated_iof: 626,
    });
    // The form shows the fields and turning it off removes the limit.
    const list = await accounts();
    expect(list.find((a) => a.name === 'Cooperativa CC')?.overdraft?.limit).toBe(500_000);
    await mockCall('accounts.update', { id: created.id, overdraft: null });
    expect((await accounts()).find((a) => a.id === created.id)?.overdraft).toBeNull();
    await user.click(screen.getAllByRole('button', { name: /Nova conta/ })[0]!);
    expect(await screen.findByText('A conta tem limite de cheque especial')).toBeTruthy();
  });

  it('credit limit with floating rate "4,75 + TR": indexer, effective rate and index date', async () => {
    renderApp('unlocked', {}, '#/contas');
    await screen.findByRole('grid', { name: 'Contas' });
    const created = (await mockCall('accounts.create', {
      name: 'Limite TR',
      kind: 'checking',
      opening_balance: -100_000,
      overdraft: {
        limit: 500_000,
        monthly_rate: null,
        discount_pct: null,
        annual_rate: '4,75 + TR',
        iof_daily_rate: null,
        iof_additional_rate: null,
        cet_month: null,
        cet_year: null,
      },
    })) as AccountView;
    expect(created.overdraft).toMatchObject({ indexer: 'tr', annual_rate: '4.75' });
    const status = created.overdraft_status!;
    expect(status.index?.indexer).toBe('tr');
    // (1 + 4.75% p.a. on a monthly basis) × (1 + the month's TR) − 1.
    const tr = Number(status.index!.monthly_pct);
    const expected = (1.0475 ** (1 / 12) * (1 + tr / 100) - 1) * 100;
    expect(Number(status.effective_monthly_rate)).toBeCloseTo(expected, 3);
    expect(status.estimated_interest).toBe(
      Math.round(Number(status.effective_monthly_rate) * 1_000),
    );
    // The account summary shows the effective rate with the index and its date.
    render(<OverdraftSummary status={status} currency="BRL" />);
    expect(await screen.findByText(/taxa efetiva ≈ .*% a\.m\. \(com TR .*em /)).toBeTruthy();
  });

  it('row context menu: transaction on the account, reconciliation and archive', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    const grid = await screen.findByRole('grid', { name: 'Contas' });
    await user.pointer({
      keys: '[MouseRight]',
      target: within(grid).getByRole('row', { name: /Cartão Nubank/ }),
    });
    let menu = screen.getByRole('menu', { name: 'Ações de Cartão Nubank' });
    expect(within(menu).getByRole('menuitem', { name: 'Faturas' })).toBeTruthy();
    expect(within(menu).getByRole('menuitem', { name: 'Compra parcelada' })).toBeTruthy();
    // A card has no "Transferir desta conta"; the dialog opens already on the row's account.
    expect(within(menu).queryByRole('menuitem', { name: 'Transferir desta conta' })).toBeNull();
    await user.click(within(menu).getByRole('menuitem', { name: 'Nova compra neste cartão' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo lançamento' });
    expect(within(dialog).getByRole('combobox', { name: /^Conta/ }).textContent).toMatch(
      /Cartão Nubank/,
    );
    await user.keyboard('{Escape}');
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    await user.pointer({
      keys: '[MouseRight]',
      target: within(grid).getByRole('row', { name: /Itaú Conta Corrente/ }),
    });
    menu = screen.getByRole('menu', { name: 'Ações de Itaú Conta Corrente' });
    await user.click(within(menu).getByRole('menuitem', { name: 'Conciliar com o extrato' }));
    expect(window.location.hash).toMatch(/conta=acc-itau/);
    expect(window.location.hash).toMatch(/aba=conciliacao/);
    expect(await screen.findByRole('button', { name: 'Comparar' })).toBeTruthy();

    await user.click(screen.getByRole('button', { name: /Todas as contas/ }));
    await user.pointer({
      keys: '[MouseRight]',
      target: within(await screen.findByRole('grid', { name: 'Contas' })).getByRole('row', {
        name: /Wise USD/,
      }),
    });
    await user.click(screen.getByRole('menuitem', { name: 'Arquivar' }));
    await waitFor(async () =>
      expect(
        (await mockCall('accounts.list', { include_archived: true })) as AccountView[],
      ).toEqual(
        expect.arrayContaining([expect.objectContaining({ name: 'Wise USD', archived: true })]),
      ),
    );
    expect(await screen.findByText('Conta arquivada')).toBeTruthy();
  });

  it('balance adjustment: on the account page and as a transaction type', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas?conta=acc-itau');
    await screen.findByRole('heading', { level: 1, name: /Itaú Conta Corrente/ });
    const before = (await accounts()).find((a) => a.id === 'acc-itau')!.balance;
    await user.click(screen.getByRole('button', { name: 'Ajustar saldo' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo lançamento' });
    expect(
      within(dialog).getByRole('radio', { name: 'Ajuste de saldo' }).getAttribute('aria-checked'),
    ).toBe('true');
    const target = before + 25_000;
    await user.type(
      within(dialog).getByLabelText(/Saldo na data/),
      (target / 100).toFixed(2).replace('.', ','),
    );
    // The preview shows the computed value and the adjustment before saving.
    const preview = within(dialog).getByRole('status', { name: 'Prévia do ajuste' });
    expect(await within(preview).findByText(/250,00/)).toBeTruthy();
    expect(within(preview).getByText(/fora de receitas, despesas e orçamento/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Ajustar saldo' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    await waitFor(async () =>
      expect((await accounts()).find((a) => a.id === 'acc-itau')!.balance).toBe(target),
    );
    expect(await screen.findByText('Saldo sincronizado com o banco')).toBeTruthy();
  });

  it('the balance history table sorts by columns', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas?conta=acc-itau&aba=historico');
    const chart = await screen.findByRole('img', { name: /Saldo de Itaú Conta Corrente/ });
    // The balance forecast, at the top of the screen, has its own "Ver tabela".
    await user.click(within(chart.closest('figure')!).getByRole('button', { name: 'Ver tabela' }));
    const grid = screen.getByRole('grid', { name: 'Saldo de Itaú Conta Corrente por data' });
    const dates = () =>
      within(grid)
        .getAllByRole('row')
        .slice(1)
        .map((r) => {
          const [d, m, y] = (r.querySelector('td')?.textContent ?? '').split('/');
          return `${y}-${m}-${d}`;
        });
    const asc = [...dates()].sort();
    expect(asc.length).toBeGreaterThan(1);
    const byDate = within(grid).getByRole('button', { name: /Data/ });
    await user.click(byDate);
    expect(dates()).toEqual(asc);
    await user.click(byDate);
    expect(dates()).toEqual([...asc].reverse());
    expect(within(grid).getByRole('columnheader', { name: /Data/ }).getAttribute('aria-sort')).toBe(
      'descending',
    );
  });
});
