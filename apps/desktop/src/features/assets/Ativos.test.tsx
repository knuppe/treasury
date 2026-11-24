import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import type { Asset } from '@/api/generated/Asset';
import type { AssetScoreSummary } from '@/api/generated/AssetScoreSummary';
import type { Trade } from '@/api/generated/Trade';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../portfolio/state';
import { ASSET_LIST_DEFAULTS, useAssetListUi } from './AssetList';
import { fixToday, quietCanvas } from '../portfolio/testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
  useAssetListUi.setState({ ...ASSET_LIST_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

const open = (search = '') => renderApp('unlocked', {}, `#/ativos${search}`);

describe('09 · Ativos with the mock', () => {
  it('shows only the held positions by default; closed ones and the registry through the filter', async () => {
    const user = userEvent.setup();
    open();
    const grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    expect(within(grid).getByText('PETR4')).toBeTruthy();
    expect(within(grid).queryByText('MGLU3')).toBeNull();
    await user.click(screen.getByRole('combobox', { name: 'Filtrar por posição' }));
    await user.click(await screen.findByRole('option', { name: 'Posições encerradas' }));
    expect(await within(grid).findByText('MGLU3')).toBeTruthy();
    expect(within(grid).queryByText('PETR4')).toBeNull();
  });

  it('opening an asset and going back through the trail returns the list with the filters', async () => {
    const user = userEvent.setup();
    open();
    let grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    await user.click(screen.getByRole('combobox', { name: 'Filtrar por posição' }));
    await user.click(await screen.findByRole('option', { name: 'Todos os cadastrados' }));
    await user.click(screen.getByRole('checkbox', { name: 'Incluir arquivados' }));
    await user.type(screen.getByLabelText('Buscar ativo por ticker ou nome'), 'bbas');
    await waitFor(() => expect(within(grid).queryByText('PETR4')).toBeNull());
    await user.dblClick(within(grid).getByText('BBAS3').closest('tr')!);
    await screen.findByRole('heading', { level: 2, name: /BBAS3/ });

    await user.click(
      within(screen.getByRole('navigation', { name: 'Trilha' })).getByRole('link', {
        name: 'Ativos',
      }),
    );
    grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    expect(screen.getByLabelText<HTMLInputElement>('Buscar ativo por ticker ou nome').value).toBe(
      'bbas',
    );
    expect(screen.getByRole('combobox', { name: 'Filtrar por posição' }).textContent).toContain(
      'Todos os cadastrados',
    );
    expect(
      screen.getByRole<HTMLInputElement>('checkbox', { name: 'Incluir arquivados' }).checked,
    ).toBe(true);
    expect(within(grid).getByText('BBAS3')).toBeTruthy();
    expect(within(grid).queryByText('PETR4')).toBeNull();
  });

  it("shows each asset's score and opens the Saúde tab", async () => {
    const user = userEvent.setup();
    open();
    const grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    const summaries = (await mockCall('score.assets', {
      asset_ids: ['ast-petr4'],
    })) as AssetScoreSummary[];
    const petr = summaries[0]!;
    const row = within(grid).getByText('PETR4').closest('tr')!;
    const cell = await within(row).findByRole('button', { name: /^Score/ });
    if (petr.score != null && petr.status === 'ok') {
      expect(cell.textContent).toContain(String(Math.round(petr.score)));
    }
    await user.click(cell);
    expect(window.location.hash).toContain('#/ativo/ast-petr4');
    expect(window.location.hash).toContain('secao=saude');
  });

  it('fund: come-cotas rule in the statement, automatic or chosen', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-xpmacro&secao=extrato');
    const rule = await screen.findByRole('combobox', { name: 'Regra do come-cotas' });
    expect(screen.getByText(/longo prazo \(15%\)/)).toBeTruthy();
    await user.click(rule);
    await user.click(await screen.findByRole('option', { name: 'Sem come-cotas' }));
    await waitFor(async () => {
      const r = (await mockCall('portfolio.come_cotas', { asset_id: 'ast-xpmacro' })) as {
        rule: { regime: string; origin: string };
      };
      expect(r.rule).toMatchObject({ regime: 'exempt', origin: 'user' });
    });
  });

  it('sorts by ticker alphabetically, with the ones without a code at the end', async () => {
    const user = userEvent.setup();
    open();
    const grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    await user.click(within(grid).getByRole('button', { name: /Ticker/ }));
    const tickers = within(grid)
      .getAllByRole('row')
      .slice(1)
      .map((r) => within(r).getAllByRole('gridcell')[0]!.textContent.trim());
    const coded = tickers.filter((t) => t !== '—');
    expect(coded).toEqual([...coded].sort((a, b) => a.localeCompare(b, 'pt-BR')));
    expect(tickers.slice(coded.length).every((t) => t === '—')).toBe(true);
  });

  it('lists, searches and opens the asset page', async () => {
    const user = userEvent.setup();
    open();
    const grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    expect(within(grid).getByText('PETR4')).toBeTruthy();
    expect(within(grid).getByText('110% do CDI · venc. 10/02/2028')).toBeTruthy();
    await user.type(screen.getByLabelText('Buscar ativo por ticker ou nome'), 'bbas');
    await waitFor(() => expect(within(grid).queryByText('PETR4')).toBeNull());
    const row = within(grid).getByText('BBAS3').closest('tr')!;
    await user.dblClick(row);
    expect(await screen.findByRole('heading', { level: 2, name: /BBAS3/ })).toBeTruthy();
    // Own page for the asset, with the trail back to Ativos.
    expect(window.location.hash).toContain('#/ativo/ast-bbas3');
    const crumb = screen.getByRole('navigation', { name: 'Trilha' });
    expect(within(crumb).getByRole('link', { name: 'Ativos' })).toBeTruthy();
    expect(screen.queryByRole('tablist', { name: 'Seções de ativos' })).toBeNull();
    // Position after the 1:2 split and the sale of 50: 250 shares.
    expect(await screen.findByText('250')).toBeTruthy();
    // Statement: operations and the split interleaved, with the position after each line.
    await user.click(screen.getByRole('tab', { name: 'Extrato' }));
    const statement = await screen.findByRole('grid', { name: 'Extrato do ativo' });
    expect(within(statement).getByText(/Desdobramento 1:2/)).toBeTruthy();
    expect(within(statement).getByText('coleta do Yahoo')).toBeTruthy();
    await user.click(screen.getByRole('tab', { name: 'Gráfico' }));
    expect(await screen.findByText(/Preço, operações e preço médio/)).toBeTruthy();
    expect(await screen.findByRole('img', { name: /Preço de BBAS3 com 3 operações/ })).toBeTruthy();
    // The split came from the collection (Yahoo) and shows its origin.
    await user.click(screen.getByRole('tab', { name: 'Eventos corporativos' }));
    const events = await screen.findByRole('grid', { name: 'Eventos corporativos' });
    expect(within(events).getByText('Desdobramento')).toBeTruthy();
    expect(within(events).getByText('Yahoo Finance')).toBeTruthy();
  });

  it('shows the chart table with the average price after each operation', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-bbas3');
    await screen.findByRole('heading', { level: 2, name: /BBAS3/ });
    await user.click(screen.getByRole('tab', { name: 'Gráfico' }));
    const toggle = await screen.findByRole('button', { name: 'Ver tabela' });
    await user.click(toggle);
    const table = await screen.findByRole('grid', { name: 'Operações e preço médio' });
    expect(within(table).getAllByRole('row').length).toBe(4);
  });

  it('fund without a quote: warns, draws the operations and accepts a manual quote', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-xpmacro');
    expect(await screen.findByText('Pelo custo (sem cotação)')).toBeTruthy();
    await user.click(screen.getByRole('tab', { name: 'Gráfico' }));
    expect(await screen.findByText('Sem histórico de cotações para este ativo')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Informar cotação' }));
    const dialog = await screen.findByRole('dialog', { name: /Informar cotação/ });
    await user.type(within(dialog).getByLabelText('Fechamento (BRL)'), '1,95');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar cotação' }));
    await waitFor(() =>
      expect(screen.queryByText('Sem histórico de cotações para este ativo')).toBeNull(),
    );
    await user.click(screen.getByRole('tab', { name: 'Visão geral' }));
    expect(await screen.findByText('Valor de mercado')).toBeTruthy();
  });

  it('the registry starts with the type and shows only its fields', async () => {
    const user = userEvent.setup();
    open('?novo=ativo');
    const dialog = await screen.findByRole('dialog', { name: 'Cadastrar ativo' });
    expect(within(dialog).queryByLabelText('Ticker')).toBeNull();
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar ativo' }));
    expect(await within(dialog).findByText('Escolha o tipo de ativo.')).toBeTruthy();

    await user.click(within(dialog).getByRole('radio', { name: 'Ações' }));
    expect(within(dialog).getByLabelText('Ticker')).toBeTruthy();
    expect(within(dialog).getByRole('combobox', { name: 'Bolsa' })).toBeTruthy();
    expect(within(dialog).queryByLabelText(/CNPJ/)).toBeNull();
    expect(within(dialog).getByText('Cotação e identificadores')).toBeTruthy();

    await user.click(within(dialog).getByRole('radio', { name: 'Fundos' }));
    expect(within(dialog).getByLabelText('CNPJ do fundo')).toBeTruthy();
    for (const label of ['Ticker', 'Setor', 'Moeda', 'ISIN'])
      expect(within(dialog).queryByLabelText(label)).toBeNull();
    expect(within(dialog).queryByRole('combobox', { name: 'Bolsa' })).toBeNull();
    expect(within(dialog).queryByRole('combobox', { name: 'Buscar ticker' })).toBeNull();

    await user.click(within(dialog).getByRole('radio', { name: 'Tesouro Direto' }));
    expect(within(dialog).getByLabelText('Título do Tesouro')).toBeTruthy();
    expect(within(dialog).getByText('Informar indexador e taxa')).toBeTruthy();
    expect(within(dialog).queryByText('Termos de renda fixa')).toBeNull();

    await user.click(within(dialog).getByRole('radio', { name: 'Cripto' }));
    expect(within(dialog).getByLabelText('Moeda de cotação')).toBeTruthy();
    expect(within(dialog).queryByRole('combobox', { name: 'Bolsa' })).toBeNull();
  });

  it('registers fixed income requiring the rate', async () => {
    const user = userEvent.setup();
    open('?novo=ativo');
    const dialog = await screen.findByRole('dialog', { name: 'Cadastrar ativo' });
    await user.click(within(dialog).getByRole('radio', { name: 'Renda fixa' }));
    await user.type(within(dialog).getByLabelText('Nome'), 'CDB Teste 120% CDI');
    expect(within(dialog).getByText('Termos de renda fixa')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar ativo' }));
    expect(await within(dialog).findByText('Informe a taxa.')).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Taxa'), '120');
    await user.type(within(dialog).getByLabelText('Vencimento'), '15/01/2029');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar ativo' }));
    expect(
      await screen.findByRole('heading', { level: 2, name: /CDB Teste 120% CDI/ }),
    ).toBeTruthy();
    const list = (await mockCall('assets.list', { text: 'CDB Teste' })) as Asset[];
    expect(list[0]?.fixed_income).toMatchObject({
      kind: 'cdb',
      indexer: 'cdi',
      rate: '120',
      maturity_date: '2029-01-15',
    });
  });

  it('watchlist with target price, upside and triggered alerts', async () => {
    open('?aba=watchlist');
    const grid = await screen.findByRole('grid', { name: 'Watchlist' });
    expect(within(grid).getByText('VALE3')).toBeTruthy();
    expect(within(grid).getByText('XPML11')).toBeTruthy();
    expect(within(grid).getAllByText(/▲ acima de|▼ abaixo de/).length).toBeGreaterThan(0);
    expect(within(grid).getByText('R$ 64,00')).toBeTruthy();
  });

  it('dividend calendar: only my portfolio and all', async () => {
    const user = userEvent.setup();
    open('?aba=agenda');
    expect(await screen.findByRole('grid', { name: /Agenda de outubro de 2026/ })).toBeTruthy();
    expect(screen.queryByText('VALE3')).toBeNull();
    await user.click(screen.getByRole('radio', { name: 'Próximos 90' }));
    await user.click(screen.getByLabelText('Só minha carteira'));
    await waitFor(() => expect(screen.getAllByText('TAEE11').length).toBeGreaterThan(0));
    expect(screen.getAllByText('≈ Projeção').length).toBeGreaterThan(0);
  });

  it('indices: series with estimates and collection with a report', async () => {
    const user = userEvent.setup();
    open('?aba=indices');
    const grid = await screen.findByRole('grid', { name: 'Séries de índices' });
    const ipca = within(grid).getByText('IPCA').closest('tr')!;
    expect(within(ipca).getByText('01/09/2026')).toBeTruthy();
    expect(within(ipca).getByText(/≈ 01\/11\/2026/)).toBeTruthy();
    expect(await screen.findByText(/IPCA, 24 meses/)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: /Atualizar índices, câmbio e cotações/ }));
    expect(await screen.findByText('Coleta concluída')).toBeTruthy();
    expect(screen.getByRole('grid', { name: 'Coleta de cotações' })).toBeTruthy();
    expect(mockCallLog()).toContain('market.sync');
  });

  it('the command bar opens an asset by ticker', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('navigation', { name: 'Telas' });
    await user.keyboard('{Control>}k{/Control}');
    const bar = await screen.findByRole('dialog', { name: 'Barra de comando' });
    await user.type(within(bar).getByRole('combobox'), 'hglg');
    const option = await within(bar).findByRole('option', { name: /HGLG11 — CSHG Logística FII/ });
    await user.click(option);
    expect(await screen.findByRole('heading', { level: 2, name: /HGLG11/ })).toBeTruthy();
  });

  it('the command bar opens the new operation form', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('navigation', { name: 'Telas' });
    await user.keyboard('{Control>}k{/Control}');
    const bar = await screen.findByRole('dialog', { name: 'Barra de comando' });
    await user.type(within(bar).getByRole('combobox'), 'nova operacao');
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('dialog', { name: 'Registrar operação' })).toBeTruthy();
  });

  it('statement: repeated purchase flagged, edit and delete from the row menu', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-bbas3');
    await screen.findByRole('heading', { level: 2, name: /BBAS3/ });
    // The same purchase recorded twice (by hand and by the agent, for example).
    const trades = (await mockCall('portfolio.list_trades', { asset_id: 'ast-bbas3' })) as Trade[];
    const buy = trades.find((x) => x.side === 'buy')!;
    await mockCall('portfolio.create_trade', {
      asset_id: buy.asset_id,
      account_id: buy.account_id,
      date: buy.date,
      side: 'buy',
      quantity: buy.quantity,
      price: buy.price,
    });
    await user.click(screen.getByRole('tab', { name: 'Extrato' }));
    const grid = await screen.findByRole('grid', { name: 'Extrato do ativo' });
    expect(await screen.findByText('Lançamentos que parecem repetidos')).toBeTruthy();
    expect(within(grid).getAllByRole('img', { name: /Parece repetida/ })).toHaveLength(2);

    const row = within(grid)
      .getAllByRole('img', { name: /Parece repetida/ })[0]!
      .closest('tr')!;
    await user.pointer({ keys: '[MouseRight]', target: row.querySelector('td')! });
    const menu = screen.getByRole('menu', { name: /^Ações de Compra em/ });
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((i) => i.dataset.label),
    ).toEqual(['Editar…', 'Copiar linha', 'Excluir…']);
    await user.click(within(menu).getByRole('menuitem', { name: 'Editar…' }));
    expect(await screen.findByRole('dialog', { name: 'Editar operação' })).toBeTruthy();
    await user.keyboard('{Escape}');

    // Delete asks for confirmation and offers undo.
    await user.pointer({ keys: '[MouseRight]', target: row.querySelector('td')! });
    await user.click(screen.getByRole('menuitem', { name: 'Excluir…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Excluir operação' });
    await user.click(within(dialog).getByRole('button', { name: 'Excluir' }));
    await waitFor(() => expect(screen.queryByText('Lançamentos que parecem repetidos')).toBeNull());
    // The core returns the change set and the notice offers Desfazer (the mock does not store it).
    expect(await screen.findByText('Operação excluída')).toBeTruthy();
    expect(
      (await mockCall('portfolio.list_trades', { asset_id: 'ast-bbas3' })) as Trade[],
    ).toHaveLength(trades.length);
  });

  it('trades tab: summary of buys, sells, costs and the position above the list', async () => {
    open('?ativo=ast-bbas3&secao=operacoes');
    const summary = await screen.findByRole('region', { name: 'Resumo das operações' });
    for (const title of ['Compras', 'Vendas', 'Custos', 'Posição atual']) {
      expect(within(summary).getByRole('heading', { level: 3, name: title })).toBeTruthy();
    }
    expect(within(summary).getByText('Preço médio de compra')).toBeTruthy();
    expect(within(summary).getByText('Resultado realizado')).toBeTruthy();
    // Two buys of 100 and one sale of 50; the 1:2 split after the first buy puts the
    // quantities in today's units: 300 bought, 250 held, no unexplained difference.
    expect(within(summary).getByText('300')).toBeTruthy();
    expect(await within(summary).findByText('250')).toBeTruthy();
    expect(within(summary).queryByText(/A posição atual considera/)).toBeNull();
    expect(within(summary).getByText(/ajustados por desdobramentos e grupamentos/)).toBeTruthy();
    expect(within(summary).getByText(/^Operações de 15\/01\/2024 a 20\/01\/2026/)).toBeTruthy();
    expect(screen.getByRole('grid', { name: 'Operações' })).toBeTruthy();
  });

  it('asset row context menu: statement and follow', async () => {
    const user = userEvent.setup();
    open();
    const grid = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    await user.pointer({ keys: '[MouseRight]', target: within(grid).getByText('BBAS3') });
    await user.click(
      within(screen.getByRole('menu', { name: 'Ações de BBAS3' })).getByRole('menuitem', {
        name: 'Ver extrato',
      }),
    );
    expect(await screen.findByRole('grid', { name: 'Extrato do ativo' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Extrato' }).getAttribute('aria-selected')).toBe('true');

    await user.click(
      within(screen.getByRole('navigation', { name: 'Trilha' })).getByRole('link', {
        name: 'Ativos',
      }),
    );
    const list = await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    await user.pointer({ keys: '[MouseRight]', target: within(list).getByText('PETR4') });
    await user.click(screen.getByRole('menuitem', { name: 'Acompanhar na watchlist…' }));
    expect(
      await screen.findByRole('dialog', { name: /Watchlist|Adicionar à watchlist/ }),
    ).toBeTruthy();
    // The request leaves the URL after the dialog opens.
    await waitFor(() => expect(window.location.hash).not.toContain('dialogo='));
  });
});
