import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import type { AccountView } from '@/api/generated/AccountView';
import type { Asset } from '@/api/generated/Asset';
import type { FixedIncomeComparison } from '@/api/generated/FixedIncomeComparison';
import type { FixedIncomeSale } from '@/api/generated/FixedIncomeSale';
import type { PerformanceBridge } from '@/api/generated/PerformanceBridge';
import type { SalesReport } from '@/api/generated/SalesReport';
import type { Trade } from '@/api/generated/Trade';
import { formatMoney } from '@/lib/format';
import { renderApp } from '@/test/renderApp';
import { useUi } from '@/state/ui';

import { periodRange } from './components/PeriodSelect';
import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from './state';
import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

function openCarteira(search = '') {
  return renderApp('unlocked', {}, `#/carteira${search}`);
}

describe('08 · Portfolio with the mock', () => {
  it('Sales: the row menu adjusts the sale (edit, new purchase, event, delete)', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=vendas');
    await user.click(await screen.findByRole('combobox', { name: 'Período: Vendas' }));
    await user.click(await screen.findByRole('option', { name: 'Desde o início' }));
    const grid = await screen.findByRole('grid', { name: 'Vendas' });
    await user.pointer({ keys: '[MouseRight]', target: within(grid).getByText('PETR4') });
    const menu = screen.getByRole('menu', { name: /^Ações de Venda de 100 PETR4/ });
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((i) => i.dataset.label),
    ).toEqual([
      'Editar venda…',
      'Registrar compra deste ativo…',
      'Registrar outra venda deste ativo…',
      'Registrar evento (desdobramento, bonificação…)…',
      'Ver extrato',
      'Abrir ativo',
      'Copiar linha',
      'Excluir venda…',
    ]);
    // Edit opens the saved sale.
    await user.click(within(menu).getByRole('menuitem', { name: 'Editar venda…' }));
    expect(await screen.findByRole('dialog', { name: 'Editar operação' })).toBeTruthy();
    await user.keyboard('{Escape}');
    // Delete: the same confirmation as in Operations; the book disappears with the sale.
    await user.pointer({ keys: '[MouseRight]', target: within(grid).getByText('PETR4') });
    await user.click(screen.getByRole('menuitem', { name: 'Excluir venda…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Excluir operação' });
    await user.click(within(dialog).getByRole('button', { name: 'Excluir' }));
    await waitFor(() => expect(within(grid).queryByText('PETR4')).toBeNull());
  });

  it('Sales: each sale with the nominal result and the real one by the chosen deflator', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=vendas');
    await user.click(await screen.findByRole('combobox', { name: 'Período: Vendas' }));
    await user.click(await screen.findByRole('option', { name: 'Desde o início' }));
    const table = await screen.findByRole('grid', { name: 'Vendas' });
    expect(within(table).getByText('PETR4')).toBeTruthy();
    expect(within(table).getByText('MGLU3')).toBeTruthy();
    expect(
      within(screen.getByRole('region', { name: 'Resultado das vendas' })).getByText(
        'Resultado real (IPCA)',
      ),
    ).toBeTruthy();
    const ipca = (await mockCall('portfolio.sales', { deflator: 'ipca' })) as SalesReport;
    const none = (await mockCall('portfolio.sales', { deflator: 'none' })) as SalesReport;
    expect(ipca.totals.count).toBeGreaterThan(1);
    // With inflation, the adjusted cost is higher and the real result lower.
    expect(ipca.totals.inflation_effect).toBeGreaterThan(0);
    expect(ipca.totals.real_result!).toBeLessThan(ipca.totals.result);
    expect(none.totals.real_result).toBe(none.totals.result);
    // With dividends: price + what the sold shares received; the IR stays in the price.
    expect(within(table).getByRole('columnheader', { name: /Com proventos/ })).toBeTruthy();
    expect(within(table).getByRole('columnheader', { name: /Proventos/ })).toBeTruthy();
    for (const r of none.rows) expect(r.total_result).toBe(r.result + r.income);
    expect(none.totals.total_result).toBe(none.totals.result + none.totals.income);
    expect(none.totals.income).toBeGreaterThan(0);
    // IGP-M enters the selector.
    const group = screen.getAllByRole('radiogroup', { name: 'Deflator do resultado real' })[0]!;
    await user.click(within(group).getByRole('radio', { name: 'IGP-M' }));
    expect(
      await within(screen.getByRole('region', { name: 'Resultado das vendas' })).findByText(
        'Resultado real (IGP-M)',
      ),
    ).toBeTruthy();
    // The Result card at the top follows the same deflator.
    expect(
      await within(screen.getByRole('region', { name: 'Resultado' })).findByText(
        'Resultado real (IGP-M)',
      ),
    ).toBeTruthy();
  });

  it('the Result card follows the bar above it: real by IPCA or nominal', async () => {
    const user = userEvent.setup();
    openCarteira();
    const card = await screen.findByRole('region', { name: 'Resultado' });
    // Default: real by IPCA, with the nominal and the index effect in the footer.
    expect(await within(card).findByText('Resultado real (IPCA)')).toBeTruthy();
    expect(within(card).getByText('Efeito do IPCA')).toBeTruthy();
    // No second selector squeezing the title: the bar "Resultado real descontado por" decides.
    expect(within(card).queryByRole('combobox', { name: 'Perspectiva do resultado' })).toBeNull();
    const group = screen.getAllByRole('radiogroup', { name: 'Deflator do resultado real' })[0]!;
    await user.click(within(group).getByRole('radio', { name: 'Nominal' }));
    expect(await within(card).findByText('Resultado total')).toBeTruthy();
    expect(within(card).queryByText('Efeito do IPCA')).toBeNull();
  });

  it('shows the summary, the valued-at-cost notice and the positions grouped by class', async () => {
    openCarteira();
    expect(await screen.findByRole('heading', { level: 1, name: /Carteira/ })).toBeTruthy();
    expect(await screen.findByText(/ativo avaliado pelo custo: XP Macro FIM/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: /Valor de mercado/ })).toBeTruthy();
    expect(screen.getByRole('heading', { name: /Retorno anual/ })).toBeTruthy();
    const acoes = await screen.findByRole('grid', { name: 'Posições · Ações' });
    expect(within(acoes).getByText('ITUB4')).toBeTruthy();
    expect(within(acoes).getByText('PETR4')).toBeTruthy();
    // Closed (MGLU3) only appears with "Incluir posições encerradas".
    expect(screen.queryByText('MGLU3')).toBeNull();
    const rf = screen.getByRole('grid', { name: 'Posições · Renda fixa' });
    expect(within(rf).getAllByText(/na curva/).length).toBeGreaterThan(0);
    expect(within(rf).getAllByText(/líq\./).length).toBe(2);
  });

  it('the deflator selector recomputes the real result and explains when an index is missing', async () => {
    const user = userEvent.setup();
    openCarteira();
    await screen.findByRole('grid', { name: 'Posições · Ações' });
    const group = screen.getByRole('radiogroup', { name: 'Deflator do resultado real' });
    expect(within(group).getByRole('radio', { name: 'IPCA' }).getAttribute('aria-checked')).toBe(
      'true',
    );

    await user.click(within(group).getByRole('radio', { name: 'US CPI' }));
    expect(usePortfolioUi.getState().deflator).toBe('us_cpi');
    expect(await screen.findByText('Real · US CPI')).toBeTruthy();
    // The current month's US CPI is not out yet: the core does not estimate and the real result has no data.
    expect(await screen.findByText(/Sem dados de US CPI até 14\/10\/2026/)).toBeTruthy();

    // Inflation of the reference country (Brazil by default, from the base currency): the IPCA.
    await user.click(within(group).getByRole('radio', { name: 'Inflação do país (BR)' }));
    expect(usePortfolioUi.getState().deflator).toBe('country');
    expect(await screen.findByText('Real · Inflação do país (Brasil)')).toBeTruthy();

    await user.click(within(group).getByRole('radio', { name: 'Nominal' }));
    await waitFor(() => expect(screen.queryByText(/^Real ·/)).toBeNull());
    const summaries = mockCallLog().filter((c) => c === 'portfolio.summary');
    expect(summaries.length).toBeGreaterThanOrEqual(4);
  });

  it('records a purchase and invalidates the portfolio', async () => {
    const user = userEvent.setup();
    openCarteira('?nova=operacao&ativo=ast-petr4');
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await within(dialog).findAllByText(/PETR4 · Petrobras PN/);
    await user.type(within(dialog).getByLabelText('Quantidade'), '10');
    await user.type(within(dialog).getByLabelText('Preço unitário (BRL)'), '31,50');
    // Exact gross: 10 × 31.50.
    expect(within(dialog).getByText('R$ 315,00')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Registrar compra' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Registrar operação' })).toBeNull(),
    );
    expect(await screen.findByText('Operação registrada')).toBeTruthy();
    const trades = (await mockCall('portfolio.list_trades', { asset_id: 'ast-petr4' })) as Trade[];
    expect(trades[0]).toMatchObject({
      date: '2026-10-14',
      side: 'buy',
      quantity: '10',
      price: '31.5',
      gross: 31500,
    });
    expect(window.location.hash).toBe('#/carteira');
  });

  it('CDB at the bank: the operation comes with the account where the asset stays and accepts the total value', async () => {
    const user = userEvent.setup();
    const { queryClient } = openCarteira();
    await screen.findByRole('heading', { level: 1, name: /Carteira/ });
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    const bank = accounts.find((a) => a.kind === 'checking' && !a.archived)!;
    const cdb = (await mockCall('assets.create', {
      name: 'CDB Banco 100% CDI',
      class: 'fixed_income',
      account_id: bank.id,
      fixed_income: {
        kind: 'cdb',
        indexer: 'cdi',
        rate: '100',
        issue_date: '2026-01-02',
        maturity_date: '2036-02-10',
        liquidity: 'at_maturity',
        tax_exempt: false,
      },
    })) as Asset;
    expect(cdb.account_id).toBe(bank.id);
    await queryClient.invalidateQueries();
    window.location.hash = `#/carteira?nova=operacao&ativo=${cdb.id}`;
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    const field = await within(dialog).findByRole('combobox', {
      name: 'Conta (corretora ou banco)',
    });
    await waitFor(() => expect(field.textContent).toContain(bank.name));
    // Fixed income by total value: purchase of R$ 5,000 (PU 1).
    expect(
      within(dialog).getByRole('radio', { name: 'Valor total' }).getAttribute('aria-checked'),
    ).toBe('true');
    await user.type(within(dialog).getByLabelText('Valor aplicado'), '5000');
    await user.click(within(dialog).getByRole('button', { name: 'Registrar compra' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Registrar operação' })).toBeNull(),
    );
    // Sale: R$ 5,315.20 gross, IR of R$ 78.79, the whole position.
    window.location.hash = `#/carteira?nova=operacao&ativo=${cdb.id}`;
    const sell = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await user.click(within(sell).getByRole('radio', { name: /Venda/ }));
    expect(within(sell).getByRole('checkbox', { name: /Resgatar a posição inteira/ })).toBeTruthy();
    await user.type(within(sell).getByLabelText('Valor bruto do resgate'), '5315,20');
    const ir = within(sell).getByLabelText('Impostos retidos (IRRF)');
    await user.clear(ir);
    await user.type(ir, '78,79');
    expect((await within(sell).findAllByText(/R\$ 5\.236,41/)).length).toBeGreaterThan(0);
    await user.click(within(sell).getByRole('button', { name: 'Registrar venda' }));
    await waitFor(async () => {
      const trades = (await mockCall('portfolio.list_trades', { asset_id: cdb.id })) as Trade[];
      expect(trades.find((x) => x.side === 'sell')).toMatchObject({
        quantity: '5000',
        gross: 531_520,
        taxes: 7_879,
      });
    });
    // Unlink the account from the asset.
    const off = (await mockCall('assets.update', { id: cdb.id, account_id: null })) as Asset;
    expect(off.account_id).toBeNull();
  }, 60_000); // purchase and sale in two dialogs: under load it exceeds the default 20 s

  it('RDB: partial redemption by value only, without asking for quantity', async () => {
    const user = userEvent.setup();
    const { queryClient } = openCarteira();
    await screen.findByRole('heading', { level: 1, name: /Carteira/ });
    const accounts = (await mockCall('accounts.list', {})) as AccountView[];
    const bank = accounts.find((a) => a.kind === 'checking' && !a.archived)!;
    const rdb = (await mockCall('assets.create', {
      name: 'RDB Banco Exemplo',
      class: 'fixed_income',
      account_id: bank.id,
      fixed_income: {
        kind: 'rdb',
        indexer: 'cdi',
        rate: '100',
        issue_date: '2026-01-02',
        maturity_date: '2028-01-02',
        liquidity: 'daily',
        tax_exempt: false,
      },
    })) as Asset;
    await queryClient.invalidateQueries();
    window.location.hash = `#/carteira?nova=operacao&ativo=${rdb.id}`;
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await waitFor(() =>
      expect(
        within(dialog).getByRole('combobox', { name: 'Conta (corretora ou banco)' }).textContent,
      ).toContain(bank.name),
    );
    expect(within(dialog).queryByLabelText(/Quantidade/)).toBeNull();
    await user.type(within(dialog).getByLabelText('Valor aplicado'), '5000');
    await user.click(within(dialog).getByRole('button', { name: 'Registrar compra' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Registrar operação' })).toBeNull(),
    );
    // Redemption of R$ 1,000: the fraction of the balance becomes the quantity internally.
    await queryClient.invalidateQueries();
    window.location.hash = `#/carteira?nova=operacao&ativo=${rdb.id}`;
    const sell = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await user.click(within(sell).getByRole('radio', { name: /Venda/ }));
    await user.click(within(sell).getByRole('checkbox', { name: /Resgatar a posição inteira/ }));
    expect(within(sell).queryByLabelText(/Quantidade/)).toBeNull();
    await user.type(within(sell).getByLabelText('Valor bruto do resgate'), '1000');
    expect(await within(sell).findByText(/Resgate parcial: .*% do saldo/)).toBeTruthy();
    await user.click(within(sell).getByRole('button', { name: 'Registrar venda' }));
    await waitFor(async () => {
      const trades = (await mockCall('portfolio.list_trades', { asset_id: rdb.id })) as Trade[];
      const out = trades.find((x) => x.side === 'sell')!;
      expect(out).toMatchObject({ gross: 100_000, by_value: true });
      expect(Number(out.quantity)).toBeGreaterThan(0);
      expect(Number(out.quantity)).toBeLessThan(5000);
    });
  }, 60_000); // purchase and redemption in two dialogs, like the CDB one

  it('blocks a sale larger than the position in the account before calling the core', async () => {
    const user = userEvent.setup();
    openCarteira('?nova=operacao&ativo=ast-petr4');
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await within(dialog).findAllByText(/PETR4 · Petrobras PN/);
    await user.click(within(dialog).getByRole('radio', { name: /Venda/ }));
    expect(await within(dialog).findByText('Posição nesta conta na data: 300')).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Quantidade'), '1000');
    await user.type(within(dialog).getByLabelText('Preço unitário (BRL)'), '32');
    expect(
      await within(dialog).findByText(
        /Venda de 1\.000 maior que a posição de 300 nesta conta em 14\/10\/2026/,
      ),
    ).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Registrar venda' }));
    expect(mockCallLog()).not.toContain('portfolio.create_trade');
    expect(screen.getByRole('dialog', { name: 'Registrar operação' })).toBeTruthy();
  });

  it('shows the core error when it refuses the operation', async () => {
    const user = userEvent.setup();
    openCarteira('?nova=operacao&ativo=ast-petr4');
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await within(dialog).findAllByText(/PETR4 · Petrobras PN/);
    // Date before the first purchase: the position on that date is zero.
    const date = within(dialog).getByLabelText('Data do pregão');
    await user.clear(date);
    await user.type(date, '01/01/2020');
    await user.click(within(dialog).getByRole('radio', { name: /Venda/ }));
    await user.type(within(dialog).getByLabelText('Quantidade'), '1');
    await user.type(within(dialog).getByLabelText('Preço unitário (BRL)'), '10');
    expect(await within(dialog).findByText(/maior que a posição de 0/)).toBeTruthy();
  });

  it('rebalances a new contribution among the classes below target, with the notice', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=alocacao');
    const input = await screen.findByLabelText('Novo aporte');
    await user.type(input, '10.000,00');
    const table = await screen.findByRole('grid', { name: 'Rebalanceamento por classe' });
    await waitFor(() => expect(mockCallLog()).toContain('portfolio.rebalance'));
    // The sum of the suggestions equals the contribution (in the footer).
    await waitFor(() => expect(within(table).getByText('R$ 10.000,00')).toBeTruthy());
    expect(screen.getByText('Não é recomendação de investimento')).toBeTruthy();
    expect(screen.getAllByText(/não é recomendação de investimento/i).length).toBeGreaterThan(1);
  });

  it('edits the target allocation and flags a sum above 100%', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=alocacao');
    const acoes = await screen.findByLabelText('Alvo de Ações (%)');
    await user.clear(acoes);
    await user.type(acoes, '40');
    expect(await screen.findByText(/passa de 100%/)).toBeTruthy();
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Salvar alvos' }).disabled).toBe(
      true,
    );
    await user.clear(acoes);
    await user.type(acoes, '22');
    expect(await screen.findByText(/· completa$/)).toBeTruthy();
  });

  it('projects the cost of fund fees over 10, 20 and 30 years', async () => {
    openCarteira('?aba=taxas');
    const table = await screen.findByRole('table', { name: 'Custo das taxas' });
    expect(within(table).getByText('XP Macro FIM')).toBeTruthy();
    expect(within(table).getByText('Em 30 anos')).toBeTruthy();
    expect(within(table).getAllByText(/com taxa menor/).length).toBeGreaterThan(0);
  });

  it('Fixed income calculator: offers in the typed order, equivalence, no "melhor", and the sale', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=renda-fixa');
    const grid = await screen.findByRole(
      'grid',
      { name: 'Ofertas de renda fixa' },
      { timeout: 15_000 },
    );
    // The two examples, in the order they came, each with the net amount.
    const rows = within(grid).getAllByRole('row');
    const cdb = rows.findIndex((r) => r.textContent?.includes('CDB 110% do CDI'));
    const lci = rows.findIndex((r) => r.textContent?.includes('LCI 90% do CDI'));
    expect(cdb).toBeGreaterThan(0);
    expect(lci).toBeGreaterThan(cdb);
    const data = (await mockCall('portfolio.compare_fixed_income', {
      amount: 1_000_000,
      offers: [
        { kind: 'cdb', index: 'cdi_pct', rate: 110, months: 24 },
        { kind: 'lci', index: 'cdi_pct', rate: 90, months: 24 },
      ],
    })) as FixedIncomeComparison;
    expect(data.offers.map((o) => o.label)).toEqual(['CDB 110% do CDI', 'LCI 90% do CDI']);
    for (const [i, o] of data.offers.entries()) {
      expect(o.net).toBeGreaterThan(o.invested);
      expect(rows[i === 0 ? cdb : lci]!.textContent).toContain(formatMoney(o.net, 'BRL'));
    }
    expect(data.offers[1]!.ir).toBe(0);
    // Taxable: how much an LCI/LCA would need to pay; exempt: how much a CDB would need to pay.
    expect(rows[cdb]!.textContent).toMatch(/equivale a [\d,]+% do CDI numa LCI\/LCA/);
    expect(rows[lci]!.textContent).toMatch(/equivale a [\d,]+% do CDI num CDB/);
    // Vault assumptions with their origin.
    expect(screen.getAllByText(/do cofre/).length).toBeGreaterThan(0);
    // Neutral comparison: no "melhor", winner or highlight.
    const panel = screen.getByRole('tabpanel');
    expect(panel.textContent).not.toMatch(/melhor|vencedor|best/i);

    // Selling before maturity: market rates below the contracted one → mark-to-market gain.
    const sale = screen.getByRole('region', { name: 'Vender antes do vencimento' });
    await user.type(within(sale).getByLabelText('Valor aplicado'), '10.000,00');
    await user.type(within(sale).getByLabelText('Data da aplicação'), '14/10/2024');
    await user.type(within(sale).getByLabelText('Vencimento'), '14/10/2029');
    await user.type(within(sale).getByLabelText('Taxa contratada (% a.a.)'), '12');
    await user.type(within(sale).getByLabelText('Taxa de mercado hoje (% a.a.)'), '10');
    const diff = await within(sale).findByLabelText('Diferença (marcação a mercado)');
    expect(diff.textContent).toMatch(/^\+/);
    expect(within(sale).getByRole('table', { name: 'Vender hoje ou levar até o vencimento' }));
    expect(within(sale).getByText(/um título novo até 14\/10\/2029 precisa pagar/)).toBeTruthy();
    const sim = (await mockCall('portfolio.simulate_fixed_income_sale', {
      amount: 1_000_000,
      start: '2024-10-14',
      maturity: '2029-10-14',
      sale_date: '2026-10-14',
      index: 'prefixed',
      contracted_rate: 12,
      market_rate: 10,
    })) as FixedIncomeSale;
    expect(sim.mark_to_market).toBeGreaterThan(0);
    expect(sim.market_value).toBeGreaterThan(sim.curve_value);
    expect(sim.breakeven_rate!).toBeGreaterThan(9);
    expect(sim.breakeven_rate!).toBeLessThan(11);
  }, 40_000);

  it('measures portfolio risk by class and asset, with correlation', async () => {
    openCarteira('?aba=risco');
    expect(await screen.findByText('Carteira de hoje no período')).toBeTruthy();
    expect(screen.getByRole('table', { name: 'Risco por classe' })).toBeTruthy();
    const assets = screen.getByRole('table', { name: 'Risco por ativo' });
    expect(within(assets).getAllByRole('row').length).toBeGreaterThan(2);
    expect(screen.getByRole('table', { name: 'Correlação entre classes' })).toBeTruthy();
    expect(screen.getAllByText(/Sharpe \(CDI\)/).length).toBe(2);
  });

  it('shows returns, dividends and passive income with a target', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=rentabilidade');
    expect(await screen.findByText(/Rentabilidade acumulada × CDI × inflação/)).toBeTruthy();
    await user.click(screen.getByRole('tab', { name: 'Renda passiva' }));
    const meta = await screen.findByLabelText('Quanto quer receber por mês');
    await user.type(meta, '5.000,00');
    expect(await screen.findByText('Patrimônio necessário')).toBeTruthy();
    await user.click(screen.getByRole('tab', { name: 'Proventos' }));
    expect(await screen.findByRole('grid', { name: 'Proventos' })).toBeTruthy();
  });

  it('Returns: the result bridge goes from the start value to the end, with the series risk', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=rentabilidade');
    // The bridge has its own period (default: year to date), separate from the rest of Returns.
    const bridgePeriod = await screen.findByRole('combobox', {
      name: 'Período: Ponte do resultado',
    });
    expect(bridgePeriod.textContent).toContain('No ano');
    await user.click(bridgePeriod);
    await user.click(await screen.findByRole('option', { name: '12 meses' }));
    const lines = await screen.findByRole('table', { name: 'Linhas da ponte do resultado' });
    const b = (await mockCall('portfolio.bridge', {
      deflator: 'ipca',
      from_month: periodRange('12m').fromMonth,
    })) as PerformanceBridge;
    const l = b.lines;
    // The bridge identity: start + price + dividends − fees − taxes + contributions −
    // withdrawals = end.
    expect(
      l.start_value + l.price_gain + l.income - l.fees - l.taxes + l.contributions - l.withdrawals,
    ).toBe(l.end_value);
    expect(l.delta).toBe(l.end_value - l.start_value - l.contributions + l.withdrawals);
    expect(l.start_value).toBeGreaterThan(0);
    // There are US assets in the mock: part of the price gain is FX.
    expect(l.fx_gain).not.toBe(0);
    const row = (name: string) => within(lines).getByRole('row', { name: new RegExp(`^${name}`) });
    const money = (c: number, signed = false) => formatMoney(c, 'BRL', { signed });
    await waitFor(() => expect(row('Valor no início').textContent).toContain(money(l.start_value)));
    expect(row('Valor no fim').textContent).toContain(money(l.end_value));
    expect(row('Taxas').textContent).toContain(money(-l.fees, true));
    expect(row('Retiradas').textContent).toContain(money(-l.withdrawals, true));
    expect(row('Ganho de preço').textContent).toContain('dos quais câmbio:');
    // The numbers shown add up from start to end.
    const cells = within(lines)
      .getAllByRole('row')
      .map((r) => r.querySelector('td')?.textContent ?? '');
    const parse = (s: string) =>
      Math.round(
        Number(
          s
            .replace(/[^\d,−-]/g, '')
            .replace('−', '-')
            .replace(',', '.'),
        ) * 100,
      );
    const shown = cells.filter(Boolean).map(parse);
    expect(shown).toHaveLength(8);
    expect(shown.slice(0, 7).reduce((a, v) => a + v, 0)).toBe(shown[7]);
    const result = screen.getByRole('group', { name: 'Resultado no período' });
    expect(result.textContent).toContain(formatMoney(l.delta, 'BRL', { signed: true }));
    expect(within(result).getByText(/Perda para a inflação/)).toBeTruthy();
    expect(within(result).getByText('Resultado real')).toBeTruthy();
    // Per asset, collapsed until opened, in the core's order.
    expect(screen.queryByRole('grid', { name: 'Ponte do resultado por ativo' })).toBeNull();
    await user.click(screen.getByText(`Por ativo (${b.assets.length})`));
    expect(await screen.findByRole('grid', { name: 'Ponte do resultado por ativo' })).toBeTruthy();
    // Series risk and the method in the footer.
    expect(screen.getByText('Semidesvio (o tamanho típico das quedas)')).toBeTruthy();
    expect(screen.getByText('Maior tempo abaixo do topo')).toBeTruthy();
    expect(
      screen.getByText(/Como calculamos: Rentabilidade mensal \(Modified Dietz\)/),
    ).toBeTruthy();
    expect(mockCallLog()).toContain('portfolio.bridge');
  });

  it('hides values in privacy mode', async () => {
    openCarteira();
    await screen.findByRole('grid', { name: 'Posições · Ações' });
    useUi.setState({ privacy: true });
    await waitFor(() =>
      expect(document.querySelectorAll('[data-money="hidden"]').length).toBeGreaterThan(10),
    );
    expect(screen.queryByText(/R\$ 3\d\.\d{3},\d{2}/)).toBeNull();
  });

  it('position context menu: new operation already on the asset', async () => {
    const user = userEvent.setup();
    openCarteira();
    const acoes = await screen.findByRole('grid', { name: 'Posições · Ações' });
    await user.pointer({ keys: '[MouseRight]', target: within(acoes).getByText('PETR4') });
    const menu = screen.getByRole('menu', { name: 'Ações de PETR4' });
    expect(within(menu).getByRole('menuitem', { name: 'Fundamentos' })).toBeTruthy();
    await user.click(within(menu).getByRole('menuitem', { name: 'Nova operação' }));
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await within(dialog).findAllByText(/PETR4 · Petrobras PN/);
    expect(window.location.hash).toContain('ativo=ast-petr4');
  });

  it('Returns: compares with CDI + x%, shows the heat map and the contributions', async () => {
    const user = userEvent.setup();
    usePortfolioUi.setState({ compare: [{ kind: 'cdi_plus', spread: 0.02 }] });
    openCarteira('?aba=rentabilidade');
    const chip = await screen.findByRole('button', {
      name: 'Remover CDI + 2,00% ao ano da comparação',
    });
    expect(
      await screen.findByRole('table', { name: 'Mapa de calor dos retornos mensais' }),
    ).toBeTruthy();
    expect(await screen.findByText('Mais somaram')).toBeTruthy();
    expect(mockCallLog()).toContain('portfolio.benchmark_series');
    await user.click(chip);
    expect(usePortfolioUi.getState().compare).toEqual([]);
  });
});

describe('08 · Scheduled contributions and portfolio by goal', () => {
  it('the overdue contribution waits for confirmation: confirming records the purchase, skipping removes it from the list', async () => {
    const user = userEvent.setup();
    openCarteira('?aba=operacoes');
    await user.click(await screen.findByRole('button', { name: 'Novo aporte programado' }));
    const dialog = await screen.findByRole('dialog', { name: 'Novo aporte programado' });
    await user.type(within(dialog).getByLabelText('Nome'), 'ITUB4 todo mês');
    await user.click(within(dialog).getByRole('combobox', { name: 'Ativo' }));
    await user.click(await screen.findByRole('option', { name: /^ITUB4/ }));
    await user.click(within(dialog).getByRole('combobox', { name: 'Conta da corretora' }));
    await user.click(await screen.findByRole('option', { name: 'XP Investimentos' }));
    await user.type(within(dialog).getByLabelText('Valor de cada aporte'), '500,00');
    const start = within(dialog).getByLabelText('Início');
    await user.clear(start);
    await user.type(start, '01/08/2026');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const before = ((await mockCall('portfolio.list_trades', {})) as Trade[]).length;
    const list = await screen.findByRole('list', { name: 'Aportes a confirmar' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    expect(screen.getByText(/3 aportes programados para confirmar/)).toBeTruthy();
    // Nothing saved before confirming.
    expect(((await mockCall('portfolio.list_trades', {})) as Trade[]).length).toBe(before);

    const [first, second] = within(list).getAllByRole('listitem');
    await user.click(within(first!).getByRole('button', { name: 'Confirmar' }));
    await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(2));
    expect(((await mockCall('portfolio.list_trades', {})) as Trade[]).length).toBe(before + 1);
    await user.click(within(second!).getByRole('button', { name: 'Pular' }));
    await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(1));
    expect(((await mockCall('portfolio.list_trades', {})) as Trade[]).length).toBe(before + 1);

    // Outside the Operations tab, a notice reminds what is left to confirm.
    await user.click(screen.getByRole('tab', { name: /Posições/ }));
    expect(
      await screen.findByText('1 aporte programado esperando a sua confirmação.'),
    ).toBeTruthy();
  });

  it('Allocation shows the portfolio by goal', async () => {
    openCarteira('?aba=alocacao');
    expect(await screen.findByText('Por objetivo')).toBeTruthy();
    expect(await screen.findByText(/Nenhuma posição ligada a uma meta ainda/)).toBeTruthy();
  });
});
