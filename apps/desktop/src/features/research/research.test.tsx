import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog, resetMock } from '@/api/mock';
import { mockResearchEmpty } from '@/api/mock/research';
import type { CorporateDocument } from '@/api/generated/CorporateDocument';
import type { IncomeEvent } from '@/api/generated/IncomeEvent';
import type { RankResult } from '@/api/generated/RankResult';
import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { Screen } from '@/api/generated/Screen';
import type { ScreenResult } from '@/api/generated/ScreenResult';
import { chartOption } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

import { useDismissedIncome } from '../assets/PendingIncome';
import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../portfolio/state';
import { fixToday, quietCanvas } from '../portfolio/testing';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
  useDismissedIncome.getState().clear();
});
afterEach(() => {
  vi.useRealTimers();
});

const open = (search = '') => renderApp('unlocked', {}, `#/ativos${search}`);

/** "5,42×" / "8,3%" → number. */
const num = (text: string) =>
  Number(
    text
      .replace(/[^\d,−-]/g, '')
      .replace('−', '-')
      .replace(',', '.'),
  );

async function analysis() {
  return screen.findByRole('region', { name: 'Análise fundamentalista' });
}

describe('09 · Ativos · fundamental analysis on the asset page', () => {
  it('indicators per group with formula, period and source on expand', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-petr4&secao=fundamentos');
    const section = await analysis();
    const valuation = await within(section).findByRole('list', { name: 'Indicadores · Valuation' });
    for (const g of ['Mercado', 'Rentabilidade', 'Endividamento', 'Crescimento', 'Dividendos'])
      expect(within(section).getByRole('list', { name: `Indicadores · ${g}` })).toBeTruthy();
    const pl = within(valuation).getByRole('button', { name: 'P/L' });
    expect(pl.getAttribute('aria-expanded')).toBe('false');
    await user.click(pl);
    expect(pl.getAttribute('aria-expanded')).toBe('true');
    const detail = document.getElementById(pl.getAttribute('aria-controls')!)!;
    expect(
      within(detail).getByText(/Valor de mercado ÷ lucro líquido dos controladores/),
    ).toBeTruthy();
    expect(within(detail).getByText('12 meses até 30/06/2026')).toBeTruthy();
    expect(within(detail).getByText(/DFP 2025 \+ ITR 2T26/)).toBeTruthy();
    // DY as a fraction becomes a percentage.
    const dy = within(section).getByLabelText(/^Dividend yield \(12 meses\): /);
    expect(dy.textContent).toMatch(/^\d+,\d%$/);
  });

  it('bank: indicators that do not apply show the reason', async () => {
    open('?ativo=ast-itub4&secao=fundamentos');
    const section = await analysis();
    await within(section).findByRole('list', { name: 'Indicadores · Bancos' });
    expect(within(section).getByLabelText('EV/EBITDA: indisponível')).toBeTruthy();
    expect(
      within(section).getAllByText(/Indisponível: não se aplica a bancos/).length,
    ).toBeGreaterThan(3);
    expect(
      within(section).getAllByText(/Basileia não está nos arquivos da CVM/).length,
    ).toBeGreaterThan(0);
  });

  it('FII: its own set (P/VP, shareholders, vacancy unavailable)', async () => {
    open('?ativo=ast-hglg11&secao=fundamentos');
    const section = await analysis();
    const fii = await within(section).findByRole('list', {
      name: 'Indicadores · Fundo imobiliário',
    });
    expect(within(fii).getByRole('button', { name: 'Número de cotistas' })).toBeTruthy();
    expect(within(fii).getByText(/vacância só está no informe trimestral/)).toBeTruthy();
    expect(within(section).queryByRole('list', { name: 'Indicadores · Valuation' })).toBeTruthy();
    expect(within(section).queryByRole('button', { name: 'P/L' })).toBeNull();
  });

  it('calculated price: Graham and Bazin with assumptions, editable rate and years, warning', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-petr4&secao=fundamentos');
    const section = await analysis();
    await user.click(within(section).getByRole('tab', { name: 'Preço calculado' }));
    expect(await within(section).findByText('Não é recomendação de investimento')).toBeTruthy();
    // Default Bazin: 2021–2025 average (2.90 + 6.70 + 4.40 + 3.10 + 3.40) ÷ 5 = 4.10 ÷ 6%.
    expect(await within(section).findByText('R$ 68,33')).toBeTruthy();
    // Graham: √(22.5 × 5.8 × 33) = 65.62.
    expect(within(section).getByText('R$ 65,62')).toBeTruthy();
    const premissas = within(section).getByRole('grid', { name: 'Premissas · Bazin' });
    expect(within(premissas).getByText('Dividendos por ação em 2021')).toBeTruthy();
    expect(within(premissas).getByText('6%')).toBeTruthy();
    const rate = within(section).getByLabelText('Taxa mínima (Bazin)');
    await user.clear(rate);
    await user.type(rate, '8');
    expect(await within(section).findByText('R$ 51,25')).toBeTruthy();
    await user.click(within(section).getByRole('combobox', { name: 'Média de dividendos' }));
    await user.click(await screen.findByRole('option', { name: '3 anos' }));
    // (4.40 + 3.10 + 3.40) ÷ 3 ÷ 8% = 45.42.
    expect(await within(section).findByText('R$ 45,42')).toBeTruthy();
    expect(within(section).getAllByText(/[▲▼] [+−]\d/).length).toBeGreaterThan(0);
    await user.click(within(section).getByRole('button', { name: 'Usar o padrão' }));
    expect(await within(section).findByText('R$ 68,33')).toBeTruthy();
  });

  it('history: revenue × profit, margins and dividends, one axis per figure', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-wege3&secao=fundamentos');
    const section = await analysis();
    await user.click(within(section).getByRole('tab', { name: 'Histórico' }));
    expect(
      await within(section).findByRole('img', { name: /Receita e lucro líquido de WEGE3/ }),
    ).toBeTruthy();
    const bars = chartOption(/Receita e lucro líquido de WEGE3/) as {
      series: unknown[];
      yAxis: unknown;
      xAxis: { data: string[] };
    };
    expect(bars.series).toHaveLength(2);
    expect(Array.isArray(bars.yAxis)).toBe(false);
    expect(bars.xAxis.data).toHaveLength(10);
    const ratios = chartOption(/Margens e ROE de WEGE3/) as { series: { name: string }[] };
    expect(ratios.series.map((s) => s.name)).toEqual(['Margem EBITDA', 'Margem líquida', 'ROE']);
    expect(within(section).getByRole('img', { name: /Dividendos por ação de WEGE3/ })).toBeTruthy();
    await user.click(within(section).getAllByRole('button', { name: 'Ver tabela' })[0]!);
    const table = await within(section).findByRole('grid', {
      name: 'Receita e lucro por exercício',
    });
    expect(within(table).getByText('2025')).toBeTruthy();
  });

  it('documents: subject as plain text and official CVM link', async () => {
    resetMock('unlocked');
    let target: { id: string; doc: CorporateDocument } | null = null;
    for (const id of [
      'ast-petr4',
      'ast-itub4',
      'ast-wege3',
      'ast-vale3',
      'ast-bbas3',
      'ast-taee11',
      'ast-mglu3',
    ]) {
      const docs = (await mockCall('research.documents', {
        asset_id: id,
        from: '2025-10-14',
      })) as CorporateDocument[];
      const doc = docs.find((d) => d.subject?.includes('<b>'));
      if (doc) {
        target = { id, doc };
        break;
      }
    }
    expect(target).not.toBeNull();
    const user = userEvent.setup();
    open(`?ativo=${target!.id}&secao=fundamentos`);
    const section = await analysis();
    await user.click(within(section).getByRole('tab', { name: 'Documentos' }));
    const grid = await within(section).findByRole('grid', { name: 'Documentos da companhia' });
    expect(within(grid).getByText(target!.doc.subject!)).toBeTruthy();
    expect(grid.querySelector('b')).toBeNull();
    const link = within(grid).getAllByRole('link', { name: /Abrir na CVM/ })[0]!;
    expect(link.getAttribute('href')).toMatch(/^https:\/\/www\.rad\.cvm\.gov\.br\//);
    expect(link.getAttribute('target')).toBe('_blank');
    expect(link.getAttribute('rel')).toContain('noopener');
  });

  it('documents: a FII lists its Fundos.NET filings, with the source in the diagnosis', async () => {
    const user = userEvent.setup();
    open('?ativo=ast-hglg11&secao=fundamentos');
    const section = await analysis();
    await user.click(within(section).getByRole('tab', { name: 'Documentos' }));
    expect(
      (await within(section).findAllByRole('link', { name: /Abrir no Fundos\.NET/ })).length,
    ).toBeGreaterThan(0);
    expect(within(section).getByRole('radio', { name: 'Relatórios gerenciais' })).toBeTruthy();
    expect(within(section).getByText('Diagnóstico dos documentos')).toBeTruthy();
    expect(within(section).getByText(/Fundos\.NET \(CVM e B3\)/)).toBeTruthy();
  });

  it('batch score: the screener filters by score, band and method indicators', async () => {
    resetMock('unlocked');
    const catalog = (await mockCall('research.catalog', {})) as IndicatorInfo[];
    for (const k of ['score', 'piotroski', 'earnings_yield', 'peg', 'dividend_years'])
      expect(catalog.some((c) => c.key === k)).toBe(true);
    expect(catalog.find((c) => c.key === 'score')?.group).toBe('score');
    const r = (await mockCall('research.screen', {
      filters: {
        conditions: [
          { field: 'score', op: 'gte', value: 60 },
          { field: 'score_band', op: 'in', value: ['robusto', 'saudavel'] },
        ],
      },
      fields: ['score', 'earnings_yield'],
    })) as ScreenResult;
    expect(r.total).toBeGreaterThan(0);
    for (const i of r.items) expect(i.metrics.score).toBeGreaterThanOrEqual(60);
  });

  it('fixed income has no analysis section', async () => {
    open('?ativo=ast-cdb-inter');
    await screen.findByRole('heading', { level: 2, name: /CDB Banco Inter/ });
    expect(screen.queryByRole('region', { name: 'Análise fundamentalista' })).toBeNull();
  });
});

describe('09 · Ativos · indicators by ticker, rankings, screener and comparator', () => {
  it('analyzes a single security from the universe by ticker', async () => {
    const user = userEvent.setup();
    open('?aba=indicadores');
    await user.type(await screen.findByLabelText('Ticker para analisar'), 'abev3');
    await user.click(screen.getByRole('button', { name: 'Analisar' }));
    // Own page for the security (outside the registry), with the trail back to Ativos.
    expect(await screen.findByRole('heading', { level: 2, name: /^ABEV3/ })).toBeTruthy();
    expect(window.location.hash).toContain('#/ativo/ABEV3');
    expect(screen.getByRole('navigation', { name: 'Trilha' })).toBeTruthy();
    expect(await screen.findByRole('list', { name: 'Indicadores · Valuation' })).toBeTruthy();
  });

  it('ranking in natural order, with a portfolio-only and watchlist cut', async () => {
    const user = userEvent.setup();
    open('?aba=rankings');
    expect(
      await screen.findByRole('grid', { name: 'Ranking · Dividend yield (12 meses)' }),
    ).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Menor P/L' }));
    const grid = await screen.findByRole('grid', { name: 'Ranking · P/L' });
    await waitFor(() => expect(within(grid).queryByText('HGLG11')).toBeNull());
    const values = within(grid)
      .getAllByRole('row')
      .slice(1)
      .map((r) => num(r.querySelectorAll('td')[3]!.textContent));
    expect(values.length).toBeGreaterThan(5);
    expect([...values].sort((a, b) => a - b)).toEqual(values);
    expect(within(grid).getByText('ABEV3')).toBeTruthy();
    await user.click(screen.getByLabelText('Só carteira e watchlist'));
    await waitFor(() => expect(within(grid).queryByText('ABEV3')).toBeNull());
    expect(within(grid).getByText('PETR4')).toBeTruthy();
    expect(within(grid).getByText('VALE3')).toBeTruthy(); // watchlist
    expect(screen.getByText(/papéis com o indicador/)).toBeTruthy();
    const last = mockCallLog().filter((c) => c === 'research.rank');
    expect(last.length).toBeGreaterThan(1);
  });

  it('ranking with an empty universe explains the collection', async () => {
    open('?aba=rankings');
    mockResearchEmpty();
    expect(await screen.findByText('Nenhum papel no universo ainda')).toBeTruthy();
    expect(screen.getByRole('link', { name: /Ir para Dados da análise/ })).toBeTruthy();
  });

  it('screener: runs, saves the view (with undo), reopens and deletes', async () => {
    const user = userEvent.setup();
    open('?aba=screener');
    await user.click(await screen.findByRole('button', { name: 'Rodar screener' }));
    const grid = await screen.findByRole('grid', { name: 'Resultado do screener' });
    expect(screen.getByText('P/L < 12 E Dividend yield (12 meses) ≥ 0.06')).toBeTruthy();
    const rows = within(grid).getAllByRole('row').slice(1);
    expect(rows.length).toBeGreaterThan(0);
    // Columns with the filtered indicators.
    expect(within(grid).getByRole('columnheader', { name: /P\/L/ })).toBeTruthy();
    const first = rows[0]!.querySelector('a')!;
    expect(first.getAttribute('href')).toMatch(/#\/ativo\/[^/?]+$/);

    // Invalid condition points out the error and does not run.
    await user.click(screen.getByRole('button', { name: 'Adicionar condição' }));
    await user.click(screen.getByRole('button', { name: 'Rodar screener' }));
    expect(await screen.findByText(/Informe um número/)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Remover condição 3' }));

    await user.click(screen.getByRole('button', { name: 'Salvar tela…' }));
    const dialog = await screen.findByRole('dialog', { name: 'Salvar tela do screener' });
    await user.type(within(dialog).getByLabelText('Nome'), 'Dividendos baratos');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar tela' }));
    expect(
      await screen.findByText('Tela “Dividendos baratos” salva', { selector: 'p' }),
    ).toBeTruthy();
    let saved = (await mockCall('research.screens.list', {})) as Screen[];
    expect(saved).toHaveLength(1);
    expect(saved[0]!.filters.conditions).toEqual([
      { field: 'pl', op: 'lt', value: 12 },
      { field: 'dy', op: 'gte', value: 0.06 },
    ]);
    expect(screen.getByText(/Tela · Dividendos baratos/)).toBeTruthy();

    // Undo right after saving.
    await user.click(screen.getByRole('button', { name: 'Desfazer' }));
    await waitFor(async () =>
      expect((await mockCall('research.screens.list', {})) as Screen[]).toHaveLength(0),
    );
    // Redo from history is not needed: saves again.
    await user.click(screen.getByRole('button', { name: 'Salvar tela…' }));
    const again = await screen.findByRole('dialog', { name: 'Salvar tela do screener' });
    await user.type(within(again).getByLabelText('Nome'), 'Dividendos baratos');
    await user.click(within(again).getByRole('button', { name: 'Salvar tela' }));
    await waitFor(async () =>
      expect((await mockCall('research.screens.list', {})) as Screen[]).toHaveLength(1),
    );
    // Change the filter and save the changes.
    const value = screen.getByLabelText('Valor da condição 1');
    await user.clear(value);
    await user.type(value, '8');
    await user.click(screen.getByRole('button', { name: 'Salvar alterações' }));
    await waitFor(async () => {
      saved = (await mockCall('research.screens.list', {})) as Screen[];
      expect(saved[0]!.filters.conditions[0]).toEqual({ field: 'pl', op: 'lt', value: 8 });
    });
    // Turn on the new-security notice on the view.
    await user.click(screen.getByRole('checkbox', { name: 'Avisar quando um papel novo entrar' }));
    await waitFor(async () => {
      saved = (await mockCall('research.screens.list', {})) as Screen[];
      expect(saved[0]!.alert).toBe(true);
    });
    await user.click(screen.getByRole('button', { name: 'Excluir tela' }));
    const confirm = await screen.findByRole('dialog', { name: 'Excluir tela' });
    await user.click(within(confirm).getByRole('button', { name: 'Excluir' }));
    await waitFor(async () =>
      expect((await mockCall('research.screens.list', {})) as Screen[]).toHaveLength(0),
    );
    const log = mockCallLog();
    expect(log).toContain('research.screens.create');
    expect(log).toContain('research.screens.update');
    expect(log).toContain('research.screens.delete');
    expect(log).toContain('changes.undo');
  });

  it('screener: opens a saved view in the builder', async () => {
    const user = userEvent.setup();
    const { queryClient } = open('?aba=screener');
    await screen.findByRole('button', { name: 'Rodar screener' });
    await mockCall('research.screens.create', {
      name: 'FIIs de logística',
      filters: {
        logic: 'and',
        conditions: [{ field: 'pvp', op: 'lte', value: 1 }],
        groups: [{ logic: 'or', conditions: [{ field: 'dy', op: 'gte', value: 0.08 }] }],
        universe: { classes: ['fii'], segments: ['logística'] },
      },
    });
    await queryClient.invalidateQueries({ queryKey: ['research'] });
    await user.click(screen.getByRole('combobox', { name: 'Telas salvas' }));
    await user.click(await screen.findByRole('option', { name: 'FIIs de logística' }));
    expect(await screen.findByText(/Tela · FIIs de logística/)).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>('Valor da condição 1').value).toBe('1');
    expect(screen.getByRole('region', { name: 'Grupo 1' })).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>('Segmento contém').value).toBe('logística');
    const grid = await screen.findByRole('grid', { name: 'Resultado do screener' });
    await waitFor(() => expect(within(grid).getByText('HGLG11')).toBeTruthy());
    expect(within(grid).queryByText('KNRI11')).toBeNull();
  });

  it('comparator: best of each row highlighted and quote on a base of 100', async () => {
    const user = userEvent.setup();
    open('?aba=comparador&comparar=PETR4,VALE3,WEGE3');
    const table = await screen.findByRole('table', { name: 'Comparação de indicadores' });
    const plRow = within(table).getByRole('rowheader', { name: 'P/L' }).closest('tr')!;
    const cells = [...plRow.querySelectorAll<HTMLElement>('td:not([aria-hidden])')];
    const values = cells.map((c) => num(c.textContent));
    const bestIdx = cells.findIndex((c) => c.dataset.best);
    expect(values[bestIdx]).toBe(Math.min(...values));
    expect(within(cells[bestIdx]!).getByText('(melhor)')).toBeTruthy();
    // Quote has no natural order.
    const price = within(table).getByRole('rowheader', { name: 'Cotação' }).closest('tr')!;
    expect(price.querySelector('[data-best]')).toBeNull();
    const opt = chartOption(/Cotação normalizada em base 100/) as {
      series: { data: (number | null)[] }[];
      yAxis: unknown;
    };
    expect(opt.series).toHaveLength(3);
    expect(Array.isArray(opt.yAxis)).toBe(false);
    for (const s of opt.series) expect(s.data[0]).toBe(100);
    // Remove one and add another.
    await user.click(screen.getByRole('button', { name: 'Tirar WEGE3 da comparação' }));
    await waitFor(() => expect(window.location.hash).toContain('comparar=PETR4%2CVALE3'));
    await user.type(screen.getByLabelText('Ticker para comparar'), 'itub4{Enter}');
    await waitFor(() => expect(window.location.hash).toContain('ITUB4'));
    await user.type(screen.getByLabelText('Ticker para comparar'), 'zzzz3{Enter}');
    expect(await screen.findByText('ZZZZ3 não está no universo baixado.')).toBeTruthy();
  });
});

describe('09 · Ativos · analysis data and income to confirm', () => {
  it('status per source, SEC notice and collection with confirmation', async () => {
    const user = userEvent.setup();
    open('?aba=dados');
    expect(await screen.findByText('Configure o e-mail da SEC')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Coletar agora' }));
    const dialog = await screen.findByRole('dialog', { name: 'Coletar dados da análise' });
    expect(within(dialog).getByText(/só os arquivos que mudaram/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Coletar' }));
    const grid = await screen.findByRole('grid', { name: 'Coleta por fonte' });
    expect(within(grid).getByText('CVM DFP · demonstrações anuais')).toBeTruthy();
    expect(within(grid).getByText('1 com problema')).toBeTruthy(); // SEC without email
    expect(mockCallLog()).toContain('research.sync');

    const ua = screen.getByLabelText('User-Agent da SEC EDGAR');
    await user.clear(ua);
    await user.type(ua, 'Treasury/0.1 ana@familia.com');
    const rate = screen.getByLabelText('Taxa mínima (Bazin)');
    await user.clear(rate);
    await user.type(rate, '7,5');
    await user.click(screen.getByRole('button', { name: 'Salvar ajustes' }));
    await waitFor(() => expect(screen.queryByText('Configure o e-mail da SEC')).toBeNull());
    const view = (await mockCall('research.get_settings', {})) as {
      settings: { bazin_rate: number; sec_user_agent: string };
    };
    expect(view.settings).toMatchObject({
      bazin_rate: 0.075,
      sec_user_agent: 'Treasury/0.1 ana@familia.com',
    });
  });

  it('first collection warns about the ~230 MB', async () => {
    const user = userEvent.setup();
    open('?aba=dados');
    mockResearchEmpty();
    expect(await screen.findByText('nunca')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Coletar agora' }));
    const dialog = await screen.findByRole('dialog', { name: 'Coletar dados da análise' });
    expect(within(dialog).getByText(/cerca de 230 MB/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Baixar ~230 MB' }));
    expect(await screen.findByRole('grid', { name: 'Coleta por fonte' })).toBeTruthy();
    const rank = (await mockCall('research.rank', { indicator: 'dy' })) as RankResult;
    expect(rank.total).toBeGreaterThan(20);
  });

  it('income to confirm: confirms with the input ready and dismisses for the session', async () => {
    const user = userEvent.setup();
    open('?aba=agenda');
    const card = await screen.findByRole('region', { name: /Proventos a confirmar \(2\)/ });
    const grid = within(card).getByRole('grid', { name: 'Proventos a confirmar' });
    // WEGE3 with no payment date at the source: the date is editable.
    const pay = within(grid).getByLabelText('Data de pagamento de WEGE3');
    expect(within(grid).getByText('≈ palpite: data-ex')).toBeTruthy();
    await user.click(within(grid).getByRole('button', { name: 'Confirmar provento de PETR4' }));
    expect(await screen.findByText('Provento de PETR4 lançado')).toBeTruthy();
    await waitFor(() =>
      expect(screen.getByRole('region', { name: /Proventos a confirmar \(1\)/ })).toBeTruthy(),
    );
    const petr = (await mockCall('portfolio.list_income', {
      asset_id: 'ast-petr4',
    })) as IncomeEvent[];
    expect(
      petr.some((i) => i.ex_date === '2026-09-18' && i.gross === Math.round(300 * 0.38 * 100)),
    ).toBe(true);
    await user.clear(pay);
    await user.type(pay, '30/09/2026');
    await user.click(within(grid).getByRole('button', { name: 'Dispensar provento de WEGE3' }));
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: /Proventos a confirmar/ })).toBeNull(),
    );
    expect(mockCallLog()).toContain('portfolio.pending_income');
  });
});
