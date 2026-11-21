/**
 * Mock das cotações intradiárias (`market.quotes`, `market.quote`, `market.intraday`,
 * `market.refresh_quotes`) e da busca de tickers (`market.search`). As cotações partem do
 * último fechamento simulado com uma variação determinística; a busca junta os ativos
 * cadastrados com uma lista fictícia no formato do Yahoo. Nada disso usa a rede.
 */
import type { Asset } from '../generated/Asset';
import type { AssetClass } from '../generated/AssetClass';
import type { IntradaySeries } from '../generated/IntradaySeries';
import type { MarketQuote } from '../generated/MarketQuote';
import type { QuoteRefreshReport } from '../generated/QuoteRefreshReport';
import type { TickerSearch } from '../generated/TickerSearch';
import type { TickerSuggestion } from '../generated/TickerSuggestion';
import { sessionAt } from '@/lib/marketHours';

import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';
import { allAssets, getAsset, pricePoints } from './assets';
import { exchangeForYahoo, resolveExchange } from './exchanges';
import { dec, hashSeed, prng } from './marketSim';
import { syncedAssetIds } from './portfolio';

const DELAY = 15;

/** Momento da última coleta por ativo (a "rotina" do mock coleta na primeira leitura). */
let fetchedAt = new Map<string, number>();
onMockReset(() => {
  fetchedAt = new Map();
});

function exchangeOf(a: Asset) {
  if (a.exchange) return resolveExchange(a.exchange.code);
  if (['stock_br', 'fii', 'etf_br', 'bdr'].includes(a.class)) return resolveExchange('B3');
  if (['stock_us', 'etf_us', 'reit'].includes(a.class)) return resolveExchange('NYSE');
  return null;
}

function quoteOf(a: Asset, now = Date.now()): MarketQuote | null {
  if (!a.quote_symbol || a.price_provider === 'manual' || a.price_provider === 'none') return null;
  const pts = pricePoints(a.id);
  const last = pts[pts.length - 1];
  if (!last) return null;
  const prev = Number(last.close);
  const rand = prng(hashSeed(`quote:${a.id}:${last.date}`));
  const change = (rand() - 0.45) * 0.04;
  const price = prev * (1 + change);
  const ex = exchangeOf(a);
  const state = a.class === 'crypto' ? 'open' : ex ? sessionAt(ex, new Date(now)) : 'closed';
  const at = new Date(now - DELAY * 60_000);
  const decimals = a.currency === 'JPY' || a.currency === 'KRW' ? 0 : 2;
  return {
    asset_id: a.id,
    symbol: a.quote_symbol,
    price: dec(Number(price.toFixed(decimals))),
    previous_close: dec(prev),
    change: dec(Number((price - prev).toFixed(decimals))),
    change_pct: Number(change.toFixed(6)),
    day_high: dec(Number((Math.max(price, prev) * 1.004).toFixed(decimals))),
    day_low: dec(Number((Math.min(price, prev) * 0.996).toFixed(decimals))),
    volume: null,
    currency: a.currency,
    market_time: at.toISOString().replace(/\.\d+Z$/, 'Z'),
    market_date: at.toISOString().slice(0, 10),
    market_state: state,
    timezone: ex?.timezone ?? null,
    delay_minutes: a.class === 'crypto' ? 0 : DELAY,
    source: 'yahoo_intraday',
    fetched_at: new Date(fetchedAt.get(a.id) ?? now).toISOString(),
  };
}

function trackedQuotes(): MarketQuote[] {
  const out: MarketQuote[] = [];
  for (const id of syncedAssetIds()) {
    const q = quoteOf(getAsset(id));
    if (q) {
      if (!fetchedAt.has(id)) fetchedAt.set(id, Date.now());
      out.push({ ...q, fetched_at: new Date(fetchedAt.get(id)!).toISOString() });
    }
  }
  return out.sort((a, b) => a.asset_id.localeCompare(b.asset_id));
}

function series(a: Asset, range: string): IntradaySeries {
  if (range !== '1d' && range !== '5d') mockFail('invalid', 'período intradiário: 1d ou 5d');
  const q = fetchedAt.has(a.id) ? quoteOf(a) : null;
  const ex = exchangeOf(a);
  const points: IntradaySeries['points'] = [];
  if (q) {
    const end = Math.floor(Date.parse(q.market_time) / 1000 / 300) * 300;
    const days = range === '1d' ? 1 : 5;
    const prev = Number(q.previous_close ?? q.price);
    const target = Number(q.price);
    for (let d = days - 1; d >= 0; d--) {
      const rand = prng(hashSeed(`intraday:${a.id}:${d}`));
      // Ponte do fechamento anterior até o preço da cotação (dias antigos: em torno dele).
      const from = d === 0 ? prev : prev * (1 + (rand() - 0.5) * 0.03);
      const to = d === 0 ? target : from * (1 + (rand() - 0.5) * 0.01);
      const n = 78;
      let noise = 0;
      for (let i = 0; i < n; i++) {
        const ts = end - d * 86_400 - (n - 1 - i) * 300;
        noise = noise * 0.9 + (rand() - 0.5) * 0.002;
        const base = from + ((to - from) * i) / (n - 1);
        const v = i === n - 1 ? to : base * (1 + noise);
        points.push({ ts, price: dec(Number(v.toFixed(4))) });
      }
    }
  }
  return {
    asset_id: a.id,
    range,
    quote: q,
    points,
    timezone: q?.timezone ?? ex?.timezone ?? null,
    exchange: ex?.code ?? null,
    session_now: ex ? sessionAt(ex) : null,
  };
}

// ── Busca ────────────────────────────────────────────────────────

/** Resultados fictícios no formato do Yahoo (símbolo, nome, bolsa do Yahoo, tipo). */
const YAHOO: [symbol: string, name: string, exchange: string, type: string][] = [
  ['PETR4.SA', 'Petróleo Brasileiro S.A. - Petrobras', 'SAO', 'EQUITY'],
  ['PETR3.SA', 'Petróleo Brasileiro S.A. - Petrobras', 'SAO', 'EQUITY'],
  ['PBR', 'Petróleo Brasileiro S.A. - Petrobras', 'NYQ', 'EQUITY'],
  ['VALE3.SA', 'Vale S.A.', 'SAO', 'EQUITY'],
  ['XPML11.SA', 'XP Malls Fundo de Investimento Imobiliário', 'SAO', 'EQUITY'],
  ['BOVA11.SA', 'iShares Ibovespa Fundo de Índice', 'SAO', 'ETF'],
  ['AAPL', 'Apple Inc.', 'NMS', 'EQUITY'],
  ['AAPL34.SA', 'Apple Inc. DRN', 'SAO', 'EQUITY'],
  ['MSFT', 'Microsoft Corporation', 'NMS', 'EQUITY'],
  ['VT', 'Vanguard Total World Stock ETF', 'PCX', 'ETF'],
  ['O', 'Realty Income Corporation', 'NYQ', 'EQUITY'],
  ['VOD.L', 'Vodafone Group Public Limited Company', 'LSE', 'EQUITY'],
  ['VOD', 'Vodafone Group Public Limited Company', 'NMS', 'EQUITY'],
  ['CSPX.L', 'iShares Core S&P 500 UCITS ETF', 'LSE', 'ETF'],
  ['SAP.DE', 'SAP SE', 'GER', 'EQUITY'],
  ['MC.PA', 'LVMH Moët Hennessy Louis Vuitton', 'PAR', 'EQUITY'],
  ['ASML.AS', 'ASML Holding N.V.', 'AMS', 'EQUITY'],
  ['NESN.SW', 'Nestlé S.A.', 'EBS', 'EQUITY'],
  ['SHOP.TO', 'Shopify Inc.', 'TOR', 'EQUITY'],
  ['7203.T', 'Toyota Motor Corporation', 'JPX', 'EQUITY'],
  ['TM', 'Toyota Motor Corporation', 'NYQ', 'EQUITY'],
  ['0700.HK', 'Tencent Holdings Limited', 'HKG', 'EQUITY'],
  ['2330.TW', 'Taiwan Semiconductor Manufacturing Company', 'TAI', 'EQUITY'],
  ['005930.KS', 'Samsung Electronics Co., Ltd.', 'KSC', 'EQUITY'],
  ['RELIANCE.NS', 'Reliance Industries Limited', 'NSI', 'EQUITY'],
  ['BHP.AX', 'BHP Group Limited', 'ASX', 'EQUITY'],
  ['NPN.JO', 'Naspers Limited', 'JNB', 'EQUITY'],
  ['WALMEX.MX', 'Wal-Mart de México', 'MEX', 'EQUITY'],
  ['BTC-USD', 'Bitcoin USD', 'CCC', 'CRYPTOCURRENCY'],
  ['VFQS.QA', 'Vodafone Qatar P.Q.S.C.', 'DOH', 'EQUITY'],
];

function typeLabel(c: AssetClass | null, type: string): string {
  if (c === 'fii') return 'FII';
  if (c === 'bdr') return 'BDR';
  if (c === 'reit') return 'REIT';
  if (c === 'crypto' || type === 'CRYPTOCURRENCY') return 'Cripto';
  if (c === 'etf_br' || c === 'etf_us' || c === 'etf_intl' || type === 'ETF') return 'ETF';
  return 'Ação';
}

function classify([symbol, name, yex, type]: (typeof YAHOO)[number]): TickerSuggestion {
  if (type === 'CRYPTOCURRENCY') {
    const [base, quote] = symbol.split('-');
    return {
      origin: 'yahoo',
      asset_id: null,
      ticker: base!,
      symbol,
      name,
      class: 'crypto',
      exchange: 'CRYPTO',
      exchange_name: yex,
      country: null,
      currency: quote ?? null,
      type_label: 'Cripto',
      sector: null,
    };
  }
  const ex = exchangeForYahoo(symbol, yex);
  const ticker =
    ex?.yahoo_suffix && symbol.endsWith(ex.yahoo_suffix)
      ? symbol.slice(0, -ex.yahoo_suffix.length)
      : ex?.country === 'US'
        ? symbol.replace('-', '.')
        : symbol;
  let cls: AssetClass | null = null;
  if (ex?.country === 'BR') {
    cls =
      type === 'ETF' || /ÍNDICE|INDICE/i.test(name)
        ? 'etf_br'
        : /IMOBILI/i.test(name)
          ? 'fii'
          : /3[1-59]$/.test(ticker) && ticker.length === 6
            ? 'bdr'
            : 'stock_br';
  } else if (ex?.country === 'US') {
    cls = type === 'ETF' ? 'etf_us' : ticker === 'O' ? 'reit' : 'stock_us';
  } else if (ex) {
    cls = type === 'ETF' ? 'etf_intl' : 'stock_intl';
  }
  return {
    origin: 'yahoo',
    asset_id: null,
    ticker,
    symbol,
    name,
    class: cls,
    exchange: ex?.code ?? null,
    exchange_name: ex?.name ?? yex,
    country: ex?.country ?? null,
    currency: ex?.currency ?? null,
    type_label: typeLabel(cls, type),
    sector: null,
  };
}

function search(query: string, limit: number): TickerSearch {
  const q = query.trim();
  const lower = q.toLowerCase();
  if (!q) return { query: q, results: [], online: false, error: null };
  const local: TickerSuggestion[] = allAssets()
    .filter((a) => !a.archived && a.ticker)
    .filter((a) => a.ticker!.toLowerCase().includes(lower) || a.name.toLowerCase().includes(lower))
    .map((a) => ({
      origin: 'asset',
      asset_id: a.id,
      ticker: a.ticker!,
      symbol: a.quote_symbol,
      name: a.name,
      class: a.class,
      exchange: a.exchange?.code ?? a.market,
      exchange_name: a.exchange?.name ?? null,
      country: a.country,
      currency: a.currency,
      type_label: typeLabel(a.class, ''),
      sector: a.sector,
    }));
  const online =
    q.length >= 2
      ? YAHOO.filter(
          ([s, n]) => s.toLowerCase().startsWith(lower) || n.toLowerCase().includes(lower),
        ).map(classify)
      : [];
  const merged = [...local];
  for (const o of online) {
    const dup = merged.find(
      (l) => l.ticker.toUpperCase() === o.ticker.toUpperCase() && l.exchange === o.exchange,
    );
    if (!dup) merged.push(o);
  }
  const Q = q.toUpperCase();
  const rank = (s: TickerSuggestion) =>
    (s.ticker === Q || s.symbol === Q ? 0 : s.ticker.startsWith(Q) ? 3 : 6) +
    (s.origin === 'asset' ? 0 : s.origin === 'research' ? 1 : 2);
  const results = merged
    .map((s, i) => ({ s, i }))
    .sort((a, b) => rank(a.s) - rank(b.s) || a.i - b.i)
    .map((x) => x.s)
    .slice(0, limit);
  return { query: q, results, online: q.length >= 2, error: null };
}

registerMockHandlers({
  'market.quotes': (args) => {
    requireUnlocked();
    const ids = Array.isArray(args.asset_ids) ? (args.asset_ids as string[]) : [];
    const all = trackedQuotes();
    return ids.length ? all.filter((q) => ids.includes(q.asset_id)) : all;
  },

  'market.quote': (args) => {
    requireUnlocked();
    const id = String(args.asset_id);
    const a = getAsset(id);
    return fetchedAt.has(id) ? quoteOf(a) : null;
  },

  'market.intraday': (args) => {
    requireUnlocked();
    return series(
      getAsset(String(args.asset_id)),
      typeof args.range === 'string' ? args.range : '1d',
    );
  },

  'market.refresh_quotes': (args) => {
    requireUnlocked();
    const ids =
      Array.isArray(args.asset_ids) && args.asset_ids.length
        ? (args.asset_ids as string[])
        : syncedAssetIds();
    const report: QuoteRefreshReport = {
      refreshed: [],
      errors: [],
      finished_at: new Date().toISOString(),
    };
    for (const id of ids) {
      const a = getAsset(id);
      if (quoteOf(a)) {
        fetchedAt.set(id, Date.now());
        report.refreshed.push(id);
      } else if (a.quote_symbol) {
        report.errors.push({
          asset_id: id,
          ticker: a.ticker,
          symbol: a.quote_symbol,
          error: 'sem fechamento simulado para este ativo',
        });
      }
    }
    return report;
  },

  'market.search': (args) => {
    requireUnlocked();
    const limit = typeof args.limit === 'number' ? Math.min(Math.max(args.limit, 1), 30) : 12;
    return search(typeof args.query === 'string' ? args.query : '', limit);
  },
});
