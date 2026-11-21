import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import type { Asset } from '@/api/generated/Asset';
import type { TickerSearch } from '@/api/generated/TickerSearch';
import { sessionAt, type TradingHours } from '@/lib/marketHours';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../portfolio/state';
import { fixToday, quietCanvas } from '../portfolio/testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
  useUi.setState({ privacy: false });
});

const open = (path: string) => renderApp('unlocked', {}, `#/${path}`);

describe("trading session in the exchange's time zone", () => {
  const nyse: TradingHours = {
    timezone: 'America/New_York',
    open_time: '09:30',
    close_time: '16:00',
    lunch_start: null,
    lunch_end: null,
    trading_days: '12345',
  };
  const tse: TradingHours = {
    timezone: 'Asia/Tokyo',
    open_time: '09:00',
    close_time: '15:30',
    lunch_start: '11:30',
    lunch_end: '12:30',
    trading_days: '12345',
  };

  it('follows daylight saving time and the lunch break', () => {
    // EDT (UTC−4) in July and EST (UTC−5) in January.
    expect(sessionAt(nyse, new Date('2026-07-15T13:30:00Z'))).toBe('open');
    expect(sessionAt(nyse, new Date('2026-01-15T14:00:00Z'))).toBe('pre');
    expect(sessionAt(nyse, new Date('2026-01-15T14:30:00Z'))).toBe('open');
    expect(sessionAt(nyse, new Date('2026-01-15T21:00:00Z'))).toBe('post');
    expect(sessionAt(nyse, new Date('2026-10-17T15:00:00Z'))).toBe('closed');
    expect(sessionAt(tse, new Date('2026-10-14T02:45:00Z'))).toBe('lunch');
    // Monday in Tokyo is still Sunday in Brazil.
    expect(sessionAt(tse, new Date('2026-10-19T00:30:00Z'))).toBe('open');
  });
});

describe('search for tickers, exchanges and intraday quotes (mock)', () => {
  it('search merges registered ones and Yahoo and fills in the registry for another exchange', async () => {
    const user = userEvent.setup();
    open('ativos?novo=ativo');
    const dialog = await screen.findByRole('dialog', { name: 'Cadastrar ativo' });
    const box = within(dialog).getByRole('combobox', { name: 'Buscar ticker' });
    await user.type(box, 'vodafone');
    const list = await screen.findByRole('listbox', { name: 'Sugestões de ativos' });
    const vod = await within(list).findByRole('option', { name: /^VOD — .*LSE · Ação · GBP/ });
    // Keyboard: ↓ to the LSE option and Enter.
    const options = within(list).getAllByRole('option');
    const idx = options.indexOf(vod);
    for (let i = 0; i < idx; i++) await user.keyboard('{ArrowDown}');
    await user.keyboard('{Enter}');
    expect(screen.queryByRole('listbox', { name: 'Sugestões de ativos' })).toBeNull();
    expect(within(dialog).getByLabelText<HTMLInputElement>('Ticker').value).toBe('VOD');
    expect(within(dialog).getByLabelText<HTMLInputElement>('Nome').value).toMatch(/Vodafone/);
    expect(
      within(dialog)
        .getByRole('radio', { name: 'Ações em outras bolsas' })
        .getAttribute('aria-checked'),
    ).toBe('true');
    expect(within(dialog).getByRole('combobox', { name: 'Bolsa' }).textContent).toContain('LSE');
    expect(within(dialog).getByLabelText<HTMLInputElement>('Moeda').value).toBe('GBP');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar ativo' }));
    expect(await screen.findByRole('heading', { level: 2, name: /VOD/ })).toBeTruthy();
    const [a] = (await mockCall('assets.list', { text: 'Vodafone' })) as Asset[];
    expect(a).toMatchObject({
      class: 'stock_intl',
      market: 'LSE',
      currency: 'GBP',
      country: 'GB',
      quote_symbol: 'VOD.L',
    });
    expect(mockCallLog()).toContain('market.search');
  });

  it('another exchange with no exchange chosen does not register', async () => {
    const user = userEvent.setup();
    open('ativos?novo=ativo');
    const dialog = await screen.findByRole('dialog', { name: 'Cadastrar ativo' });
    await user.click(within(dialog).getByRole('radio', { name: 'ETFs em outras bolsas' }));
    await user.type(within(dialog).getByLabelText('Nome'), 'ETF sem bolsa');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar ativo' }));
    expect(await within(dialog).findByText(/ou cadastre-a em Ativos › Bolsas/)).toBeTruthy();
  });

  it('the operation registers and picks a new asset from the search', async () => {
    const user = userEvent.setup();
    open('carteira?nova=operacao');
    const dialog = await screen.findByRole('dialog', { name: 'Registrar operação' });
    await user.type(within(dialog).getByRole('combobox', { name: 'Buscar ticker' }), 'toyota');
    await user.click(await screen.findByRole('option', { name: /^7203 — .*TSE · Ação · JPY/ }));
    const group = within(dialog).getByRole('group', { name: 'Ativo novo' });
    expect(within(group).getByText(/7203\.T · Ações em outras bolsas/)).toBeTruthy();
    await user.click(within(group).getByRole('button', { name: 'Cadastrar e usar' }));
    await waitFor(() =>
      expect(within(dialog).queryByRole('group', { name: 'Ativo novo' })).toBeNull(),
    );
    const [a] = (await mockCall('assets.list', { text: 'Toyota' })) as Asset[];
    expect(a).toMatchObject({ ticker: '7203', market: 'TSE', currency: 'JPY' });
    await waitFor(() =>
      expect(within(dialog).getByRole('combobox', { name: 'Ativo' }).textContent).toContain('7203'),
    );
  });

  it('the watchlist picks an already registered asset from the search', async () => {
    const user = userEvent.setup();
    open('ativos?aba=watchlist');
    await screen.findByRole('grid', { name: 'Watchlist' });
    await user.click(screen.getByRole('button', { name: 'Adicionar' }));
    const dialog = await screen.findByRole('dialog', { name: 'Adicionar à watchlist' });
    await user.type(within(dialog).getByRole('combobox', { name: 'Buscar ticker' }), 'wege');
    const opt = await screen.findByRole('option', { name: /^WEGE3 — .*\(cadastrado\)/ });
    await user.click(opt);
    expect(within(dialog).queryByRole('group', { name: 'Ativo novo' })).toBeNull();
    expect(within(dialog).getByRole('combobox', { name: 'Ativo' }).textContent).toContain('WEGE3');
  });

  it("the watchlist and the positions show the day's change with the delay badge", async () => {
    open('ativos?aba=watchlist');
    const grid = await screen.findByRole('grid', { name: 'Watchlist' });
    await waitFor(() =>
      expect(
        within(grid).getAllByRole('button', { name: /Cotação com ≈15 min/ }).length,
      ).toBeGreaterThan(0),
    );
  });

  it('asset page: 1D/5D intraday chart, refresh and privacy', async () => {
    const user = userEvent.setup();
    open('ativos?ativo=ast-petr4');
    await screen.findByRole('heading', { level: 2, name: /PETR4/ });
    expect(screen.getByText(/Hoje · PETR4\.SA/)).toBeTruthy();
    // With no recent quote, the page asks for the collection (market.refresh_quotes).
    await waitFor(() => expect(mockCallLog()).toContain('market.refresh_quotes'));
    const badge = await screen.findByRole('button', { name: /Cotação com atraso de ~15 min/ });
    expect(badge).toBeTruthy();
    expect(screen.getByRole('img', { name: 'Cotação intradiária de PETR4 (1 dia)' })).toBeTruthy();
    await user.click(screen.getByRole('radio', { name: '5D' }));
    expect(
      await screen.findByRole('img', { name: 'Cotação intradiária de PETR4 (5 dias)' }),
    ).toBeTruthy();
    const series = (await mockCall('market.intraday', {
      asset_id: 'ast-petr4',
      range: '5d',
    })) as { points: unknown[] };
    expect(series.points.length).toBe(5 * 78);
    // Privacy mode: the price disappears.
    const fig = screen.getByLabelText('Cotação intradiária');
    expect(within(fig).getByText(/R\$/)).toBeTruthy();
    useUi.setState({ privacy: true });
    await waitFor(() => expect(within(fig).queryByText(/R\$/)).toBeNull());
  });

  it('Bolsas tab: catalog with the session right now and exchange registry', async () => {
    const user = userEvent.setup();
    open('ativos?aba=bolsas');
    const grid = await screen.findByRole('grid', { name: 'Bolsas' });
    expect(within(grid).getByText('LSE')).toBeTruthy();
    expect(within(grid).getByText('Bolsa de Tóquio (JPX)')).toBeTruthy();
    expect(within(grid).getAllByText('câmbio manual').length).toBeGreaterThan(0);
    await user.click(screen.getByRole('button', { name: 'Cadastrar bolsa' }));
    const dialog = await screen.findByRole('dialog', { name: 'Cadastrar bolsa' });
    await user.type(within(dialog).getByLabelText('Código'), 'pse');
    await user.type(within(dialog).getByLabelText('Nome'), 'Philippine Stock Exchange');
    await user.type(within(dialog).getByLabelText('País'), 'ph');
    await user.type(within(dialog).getByLabelText('Moeda'), 'php');
    await user.type(within(dialog).getByLabelText('Sufixo no Yahoo'), '.ps');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar bolsa' }));
    expect(await within(dialog).findByText(/Informe o fuso/)).toBeTruthy();
    await user.type(within(dialog).getByLabelText('Fuso (IANA)'), 'Asia/Manila');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar bolsa' }));
    await waitFor(() => expect(within(grid).getByText('PSE')).toBeTruthy());
  });

  it("the mock's search classifies exchanges and classes like the core", async () => {
    const r = (await mockCall('market.search', { query: 'petr' })) as TickerSearch;
    expect(r.results[0]).toMatchObject({ origin: 'asset', ticker: 'PETR4', symbol: 'PETR4.SA' });
    expect(r.results.filter((s) => s.symbol === 'PETR4.SA')).toHaveLength(1);
    const pbr = r.results.find((s) => s.symbol === 'PBR');
    expect(pbr).toMatchObject({ class: 'stock_us', exchange: 'NYSE', currency: 'USD' });
    const q = (await mockCall('market.search', { query: 'qatar' })) as TickerSearch;
    expect(q.results[0]).toMatchObject({ class: null, exchange: null, exchange_name: 'DOH' });
  });
});
