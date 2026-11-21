/**
 * Mock de `assets.*` e das cotações (`market.prices`, `market.set_price`): cadastro
 * de ativos fictícios (ações e FIIs da B3, ETF, Tesouro IPCA+, CDB 110% CDI, LCI,
 * ações e ETF nos EUA, cripto) com histórico diário de fechamentos simulado.
 */
import type { Asset } from '../generated/Asset';
import type { AssetClass } from '../generated/AssetClass';
import type { AssetInput } from '../generated/AssetInput';
import type { FixedIncomeTerms } from '../generated/FixedIncomeTerms';
import type { PriceOn } from '../generated/PriceOn';
import type { PricePoint } from '../generated/PricePoint';
import type { ExchangeBrief } from '../generated/ExchangeBrief';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';
import { resolveExchange } from './exchanges';
import { baseCurrency } from './ledger/domain';
import { invoke } from './ledger/investImport';
import {
  addMonths,
  dec,
  mockToday,
  previousBusinessDay,
  valueOn,
  walk,
  type WalkSpec,
  txt,
} from './marketSim';

const LABEL: Record<AssetClass, string> = {
  stock_br: 'Ações',
  fii: 'FIIs',
  etf_br: 'ETFs',
  bdr: 'BDRs',
  fixed_income: 'Renda fixa',
  treasury_br: 'Tesouro Direto',
  fund: 'Fundos',
  stock_us: 'Ações nos EUA',
  etf_us: 'ETFs nos EUA',
  reit: 'REITs',
  crypto: 'Cripto',
  pension: 'Previdência',
  other: 'Outros',
  stock_intl: 'Ações em outras bolsas',
  etf_intl: 'ETFs em outras bolsas',
};

export function classLabel(c: AssetClass): string {
  return LABEL[c];
}

function defaultCurrency(c: AssetClass): string {
  return c === 'stock_us' ||
    c === 'etf_us' ||
    c === 'reit' ||
    c === 'crypto' ||
    c === 'stock_intl' ||
    c === 'etf_intl'
    ? 'USD'
    : 'BRL';
}

const LISTED: ReadonlySet<AssetClass> = new Set([
  'stock_br',
  'fii',
  'etf_br',
  'bdr',
  'stock_us',
  'etf_us',
  'reit',
  'stock_intl',
  'etf_intl',
]);

function brief(market: string | null): ExchangeBrief | null {
  const e = market ? resolveExchange(market) : null;
  return e
    ? {
        code: e.code,
        name: e.name,
        country: e.country,
        timezone: e.timezone,
        yahoo_suffix: e.yahoo_suffix,
      }
    : null;
}

/** Símbolo do Yahoo (`market::sync::default_source`). */
export function quoteSymbol(a: {
  class: AssetClass;
  ticker: string | null;
  currency: string;
  price_provider: string | null;
  provider_symbol: string | null;
  exchange: ExchangeBrief | null;
}): string | null {
  if (a.price_provider === 'manual' || a.price_provider === 'none') return null;
  if (a.provider_symbol) return a.provider_symbol;
  if (!a.ticker) return null;
  const t = a.ticker.toUpperCase();
  const suffixed = a.exchange?.yahoo_suffix ? `${t}${a.exchange.yahoo_suffix}` : null;
  switch (a.class) {
    case 'stock_br':
    case 'fii':
    case 'etf_br':
    case 'bdr':
      return `${t}.SA`;
    case 'stock_us':
    case 'etf_us':
    case 'reit':
      return suffixed ?? t.replace('.', '-');
    case 'stock_intl':
    case 'etf_intl':
      return a.exchange ? (suffixed ?? t.replace('.', '-')) : null;
    case 'crypto':
      return `${t}-${a.currency}`;
    default:
      return null;
  }
}

function defaultMarket(c: AssetClass): string | null {
  if (c === 'stock_br' || c === 'fii' || c === 'etf_br' || c === 'bdr') return 'B3';
  if (c === 'treasury_br') return 'TESOURO';
  if (c === 'fund') return 'CVM';
  if (c === 'crypto') return 'CRYPTO';
  return null;
}

interface Seed {
  id: string;
  ticker: string | null;
  name: string;
  class: AssetClass;
  market?: string;
  country?: string;
  sector?: string;
  segment?: string;
  isin?: string;
  treasury_code?: string;
  fixed_income?: FixedIncomeTerms;
  /** Passeio de preço (na base de hoje, já ajustado por desdobramentos). */
  walk?: WalkSpec;
  /** Desdobramentos reais: antes da data, o preço da base do dia é `× fator`. */
  splits?: { date: string; factor: number }[];
  source?: string;
}

const SEEDS: Seed[] = [
  {
    id: 'ast-petr4',
    ticker: 'PETR4',
    name: 'Petrobras PN',
    class: 'stock_br',
    sector: 'Petróleo, gás e biocombustíveis',
    isin: 'BRPETRACNPR6',
    walk: { start: 37.2, anchor: 31.9, vol: 0.27 },
  },
  {
    id: 'ast-itub4',
    ticker: 'ITUB4',
    name: 'Itaú Unibanco PN',
    class: 'stock_br',
    sector: 'Financeiro',
    segment: 'Bancos',
    walk: { start: 33.4, anchor: 38.6, vol: 0.2 },
  },
  {
    id: 'ast-wege3',
    ticker: 'WEGE3',
    name: 'WEG ON',
    class: 'stock_br',
    sector: 'Bens industriais',
    walk: { start: 36.8, anchor: 44.1, vol: 0.26 },
  },
  {
    id: 'ast-bbas3',
    ticker: 'BBAS3',
    name: 'Banco do Brasil ON',
    class: 'stock_br',
    sector: 'Financeiro',
    segment: 'Bancos',
    walk: { start: 27.6, anchor: 23.7, vol: 0.24 },
    splits: [{ date: '2024-04-16', factor: 2 }],
  },
  {
    id: 'ast-mglu3',
    ticker: 'MGLU3',
    name: 'Magazine Luiza ON',
    class: 'stock_br',
    sector: 'Consumo cíclico',
    segment: 'Varejo',
    walk: { start: 21.4, anchor: 8.9, vol: 0.55 },
  },
  {
    id: 'ast-vale3',
    ticker: 'VALE3',
    name: 'Vale ON',
    class: 'stock_br',
    sector: 'Materiais básicos',
    segment: 'Mineração',
    walk: { start: 70.1, anchor: 57.8, vol: 0.25 },
  },
  {
    id: 'ast-taee11',
    ticker: 'TAEE11',
    name: 'Taesa Unit',
    class: 'stock_br',
    sector: 'Utilidade pública',
    segment: 'Energia elétrica',
    walk: { start: 36.2, anchor: 35.1, vol: 0.14 },
  },
  {
    id: 'ast-hglg11',
    ticker: 'HGLG11',
    name: 'CSHG Logística FII',
    class: 'fii',
    segment: 'Logística',
    walk: { start: 165.3, anchor: 158.4, vol: 0.1 },
  },
  {
    id: 'ast-knri11',
    ticker: 'KNRI11',
    name: 'Kinea Renda Imobiliária FII',
    class: 'fii',
    segment: 'Híbrido',
    walk: { start: 160.2, anchor: 151.7, vol: 0.09 },
  },
  {
    id: 'ast-mxrf11',
    ticker: 'MXRF11',
    name: 'Maxi Renda FII',
    class: 'fii',
    segment: 'Papel (recebíveis)',
    walk: { start: 10.62, anchor: 9.64, vol: 0.08 },
  },
  {
    id: 'ast-xpml11',
    ticker: 'XPML11',
    name: 'XP Malls FII',
    class: 'fii',
    segment: 'Shoppings',
    walk: { start: 115.4, anchor: 104.9, vol: 0.11 },
  },
  {
    id: 'ast-bova11',
    ticker: 'BOVA11',
    name: 'iShares Ibovespa',
    class: 'etf_br',
    segment: 'Índice Ibovespa',
    walk: { start: 126.1, anchor: 141.3, vol: 0.18 },
  },
  {
    id: 'ast-ipca35',
    ticker: null,
    name: 'Tesouro IPCA+ 2035',
    class: 'treasury_br',
    treasury_code: 'Tesouro IPCA+ 2035',
    walk: { start: 2210, anchor: 2648, vol: 0.06 },
    source: 'tesouro_transparente',
    fixed_income: {
      kind: 'tesouro_ipca',
      indexer: 'ipca',
      rate: '6.2',
      issue_date: null,
      maturity_date: '2035-05-15',
      grace_until: null,
      liquidity: 'daily',
      ir_exempt: false,
      issuer: 'Tesouro Nacional',
    },
  },
  {
    id: 'ast-cdb-inter',
    ticker: null,
    name: 'CDB Banco Inter 110% CDI',
    class: 'fixed_income',
    fixed_income: {
      kind: 'cdb',
      indexer: 'cdi',
      rate: '110',
      issue_date: '2025-02-10',
      maturity_date: '2028-02-10',
      grace_until: null,
      liquidity: 'at_maturity',
      ir_exempt: false,
      issuer: 'Banco Inter',
    },
  },
  {
    id: 'ast-lci-itau',
    ticker: null,
    name: 'LCI Itaú 95% CDI',
    class: 'fixed_income',
    fixed_income: {
      kind: 'lci',
      indexer: 'cdi',
      rate: '95',
      issue_date: '2025-06-02',
      maturity_date: '2027-06-02',
      grace_until: '2026-03-02',
      liquidity: 'after_grace',
      ir_exempt: true,
      issuer: 'Itaú Unibanco',
    },
  },
  {
    id: 'ast-aapl',
    ticker: 'AAPL',
    name: 'Apple Inc.',
    class: 'stock_us',
    market: 'NASDAQ',
    country: 'US',
    sector: 'Tecnologia',
    walk: { start: 192.5, anchor: 247.8, vol: 0.24 },
  },
  {
    id: 'ast-msft',
    ticker: 'MSFT',
    name: 'Microsoft Corp.',
    class: 'stock_us',
    market: 'NASDAQ',
    country: 'US',
    sector: 'Tecnologia',
    walk: { start: 374.6, anchor: 436.2, vol: 0.22 },
  },
  {
    id: 'ast-nvda',
    ticker: 'NVDA',
    name: 'NVIDIA Corp.',
    class: 'stock_us',
    market: 'NASDAQ',
    country: 'US',
    sector: 'Tecnologia',
    walk: { start: 48.2, anchor: 176.4, vol: 0.48 },
  },
  {
    id: 'ast-voo',
    ticker: 'VOO',
    name: 'Vanguard S&P 500 ETF',
    class: 'etf_us',
    market: 'NYSE Arca',
    country: 'US',
    segment: 'Índice S&P 500',
    walk: { start: 436.9, anchor: 612.3, vol: 0.15 },
  },
  {
    id: 'ast-xpmacro',
    ticker: null,
    name: 'XP Macro FIM',
    class: 'fund',
    segment: 'Multimercado macro',
  },
  {
    id: 'ast-btc',
    ticker: 'BTC',
    name: 'Bitcoin',
    class: 'crypto',
    walk: { start: 42_150, anchor: 106_400, vol: 0.52 },
  },
];

/** Nome oficial só quando difere do mostrado (como no núcleo). */
function officialOf(v: string | null, name: string): string | null {
  return v && v.toLowerCase() !== name.trim().toLowerCase() ? v : null;
}

function toAsset(s: Seed): Asset {
  const market = s.market ?? defaultMarket(s.class);
  const exchange = brief(market);
  const a: Asset = {
    id: s.id,
    ticker: s.ticker,
    name: s.name,
    official_name: null,
    account_id: null,
    class: s.class,
    class_label: LABEL[s.class],
    market: exchange?.code ?? market,
    currency: defaultCurrency(s.class),
    country: s.country ?? (defaultCurrency(s.class) === 'BRL' ? 'BR' : null),
    isin: s.isin ?? null,
    cnpj: null,
    treasury_code: s.treasury_code ?? null,
    sector: s.sector ?? null,
    segment: s.segment ?? null,
    price_provider: null,
    provider_symbol: null,
    archived: false,
    notes: null,
    fixed_income: s.fixed_income ?? null,
    exchange,
    quote_symbol: null,
  };
  a.quote_symbol = quoteSymbol(a);
  return a;
}

interface AssetsState {
  assets: Map<string, Asset>;
  /** Cotações manuais (`market.set_price`), por ativo e data. */
  manual: Map<string, Map<string, number>>;
  seq: number;
}

let state: AssetsState = initial();

function initial(): AssetsState {
  return { assets: new Map(SEEDS.map((s) => [s.id, toAsset(s)])), manual: new Map(), seq: 0 };
}

onMockReset(() => {
  state = initial();
  priceCache.clear();
});

export function allAssets(): Asset[] {
  return [...state.assets.values()];
}

export function getAsset(id: string): Asset {
  const a = state.assets.get(id);
  if (!a) mockFail('not_found', `Não encontrado: ativo ${id}.`);
  return a;
}

export function hasAsset(id: string): boolean {
  return state.assets.has(id);
}

// ── Cotações ─────────────────────────────────────────────────────

const priceCache = new Map<string, { to: string; series: Map<string, number>; source: string }>();

function seedOf(id: string): Seed | undefined {
  return SEEDS.find((s) => s.id === id);
}

/** Fechamentos simulados (na base de cada dia: antes de um desdobramento, o preço é maior). */
function generated(id: string): { series: Map<string, number>; source: string } | null {
  const today = mockToday();
  const hit = priceCache.get(id);
  if (hit && hit.to === today) return hit;
  const s = seedOf(id);
  if (!s?.walk) return null;
  const raw = walk(`price:${id}`, s.walk, addDaysSafe(today, -1));
  const series = new Map<string, number>();
  const decimals = s.class === 'crypto' ? 2 : s.class === 'treasury_br' ? 2 : 2;
  for (const [d, v] of raw) {
    let p = v;
    for (const sp of s.splits ?? []) if (d < sp.date) p *= sp.factor;
    series.set(d, Number(p.toFixed(decimals)));
  }
  const source = s.source ?? 'yahoo';
  const entry = { to: today, series, source };
  priceCache.set(id, entry);
  return entry;
}

function addDaysSafe(isoDate: string, days: number): string {
  const d = new Date(`${isoDate}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + days);
  return d.toISOString().slice(0, 10);
}

/** Pontos de cotação (gerados + manuais) em ordem de data. */
export function pricePoints(id: string): PricePoint[] {
  const g = generated(id);
  const out = new Map<string, PricePoint>();
  if (g) for (const [d, v] of g.series) out.set(d, { date: d, close: dec(v), source: g.source });
  for (const [d, v] of state.manual.get(id) ?? [])
    out.set(d, { date: d, close: dec(v), source: 'manual' });
  return [...out.values()].sort((a, b) => a.date.localeCompare(b.date));
}

/** Fechamento na data ou o último anterior (`market::close_on`). */
export function closeOn(id: string, date: string): { close: number; price: PriceOn } | null {
  const manual = state.manual.get(id);
  const g = generated(id);
  const fromGen = g ? valueOn(g.series, date) : null;
  let best: [string, number, string] | null = fromGen ? [fromGen[0], fromGen[1], g!.source] : null;
  for (const [d, v] of manual ?? []) {
    if (d <= date && (!best || d >= best[0])) best = [d, v, 'manual'];
  }
  if (!best) return null;
  return {
    close: best[1],
    price: {
      close: dec(best[1]),
      date: best[0],
      source: best[2],
      stale: best[0] !== date,
      intraday: null,
    },
  };
}

/** Símbolo/fonte usados na coleta (para o relatório do `market.sync`). */
export function syncSource(a: Asset): string | null {
  if (a.price_provider === 'manual' || a.price_provider === 'none') return null;
  if (a.class === 'treasury_br') return 'tesouro_transparente';
  const sym = quoteSymbol(a);
  return sym ? `yahoo:${sym}` : null;
}

export function lastPriceDate(id: string): string | null {
  const pts = pricePoints(id);
  return pts.length ? pts[pts.length - 1]!.date : null;
}

// ── Comandos ─────────────────────────────────────────────────────

function str(v: unknown): string | null {
  if (typeof v !== 'string') return null;
  const t = v.trim();
  return t === '' ? null : t;
}

function validDate(v: unknown, what = 'data'): string {
  if (typeof v !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(v)) {
    mockFail('invalid', `${what} inválida: ${String(v)}`);
  }
  return v;
}

function validTerms(t: unknown): FixedIncomeTerms {
  const terms = t as FixedIncomeTerms;
  const rate = txt(terms.rate).trim().replace(',', '.');
  if (!/^-?\d+(\.\d+)?$/.test(rate)) mockFail('invalid', `taxa inválida: ${terms.rate}`);
  for (const d of [terms.issue_date, terms.maturity_date, terms.grace_until]) if (d) validDate(d);
  return {
    kind: terms.kind,
    indexer: terms.indexer,
    rate: dec(Number(rate)),
    issue_date: terms.issue_date ?? null,
    maturity_date: terms.maturity_date ?? null,
    grace_until: terms.grace_until ?? null,
    liquidity: terms.liquidity ?? 'at_maturity',
    ir_exempt: Boolean(terms.ir_exempt),
    issuer: terms.issuer ?? null,
  };
}

const CLASS_ORDER_SQL = (a: Asset, b: Asset) =>
  a.class.localeCompare(b.class) || (a.ticker ?? a.name).localeCompare(b.ticker ?? b.name);

/** Ativos com operações (o mock da carteira informa). */
let heldProbe: (id: string) => boolean = () => false;
export function setHeldProbe(probe: (id: string) => boolean): void {
  heldProbe = probe;
}

/** Ativos com quantidade em carteira hoje (o mock da carteira informa). */
let openProbe: (id: string) => boolean = () => false;
export function setOpenProbe(probe: (id: string) => boolean): void {
  openProbe = probe;
}

let usageProbe: (id: string) => boolean = () => false;
export function setUsageProbe(probe: (id: string) => boolean): void {
  usageProbe = probe;
}

let onDelete: (id: string) => void = () => {};
export function setDeleteHook(hook: (id: string) => void): void {
  onDelete = hook;
}

registerMockHandlers({
  'assets.list': (args) => {
    requireUnlocked();
    const text = str(args.text)?.toLowerCase();
    const classes = Array.isArray(args.classes) ? (args.classes as AssetClass[]) : [];
    return allAssets()
      .filter((a) => args.include_archived === true || !a.archived)
      .filter(
        (a) =>
          !text ||
          (a.ticker ?? '').toLowerCase().includes(text) ||
          a.name.toLowerCase().includes(text) ||
          (a.official_name ?? '').toLowerCase().includes(text),
      )
      .filter((a) => classes.length === 0 || classes.includes(a.class))
      .filter((a) => {
        // Previdência cadastrada sem aporte fica em "Em carteira" (como no núcleo).
        const plan = a.class === 'pension' && !heldProbe(a.id);
        if (args.position === 'open') return plan || openProbe(a.id);
        if (args.held_only !== true && !args.position) return true;
        if (!heldProbe(a.id)) return false;
        return args.position !== 'closed' || !openProbe(a.id);
      })
      .sort(CLASS_ORDER_SQL);
  },

  'assets.get': (args) => {
    requireUnlocked();
    return getAsset(String(args.id));
  },

  'assets.find': (args) => {
    requireUnlocked();
    const t = txt(args.ticker).trim().toUpperCase();
    return (
      allAssets()
        .filter((a) => (a.ticker ?? '').toUpperCase() === t)
        .sort((a, b) => Number(a.archived) - Number(b.archived))[0] ?? null
    );
  },

  'assets.create': (raw) => {
    requireUnlocked();
    const a = raw as unknown as AssetInput;
    const name = txt(a.name).trim();
    if (!name) mockFail('invalid', 'informe o nome do ativo');
    if (a.class === 'fixed_income' && !a.fixed_income) {
      mockFail('invalid', 'renda fixa precisa dos termos (indexador, taxa, vencimento)');
    }
    if (!LABEL[a.class]) mockFail('invalid', `classe de ativo inválida: ${String(a.class)}`);
    if (a.pension && a.class !== 'pension') {
      mockFail('invalid', 'o plano de previdência é para ativos da classe Previdência');
    }
    const ticker = str(a.ticker)?.toUpperCase() ?? null;
    const ex = str(a.market) ? resolveExchange(str(a.market)) : null;
    if ((a.class === 'stock_intl' || a.class === 'etf_intl') && !ex) {
      mockFail(
        'invalid',
        str(a.market)
          ? `bolsa ${str(a.market)} não está no catálogo: cadastre a bolsa (código, país, moeda, fuso e pregão) antes`
          : 'informe a bolsa (LSE, XETRA, TSE…) de ações e ETFs de outras bolsas',
      );
    }
    const market = ex?.code ?? str(a.market)?.toUpperCase() ?? defaultMarket(a.class);
    const listedEx = ex && LISTED.has(a.class) ? ex : null;
    if (
      ticker &&
      allAssets().some(
        (x) => x.ticker?.toUpperCase() === ticker && (x.market ?? '') === (market ?? ''),
      )
    ) {
      mockFail('conflict', `o ativo ${ticker} já está cadastrado`);
    }
    const currency = (
      str(a.currency) ??
      listedEx?.currency ??
      // Como no núcleo: renda fixa, previdência e outros nascem na moeda base da família.
      (a.class === 'fixed_income' || a.class === 'pension' || a.class === 'other'
        ? baseCurrency()
        : defaultCurrency(a.class))
    ).toUpperCase();
    if (!/^[A-Z]{3}$/.test(currency)) mockFail('invalid', `moeda inválida: ${currency}`);
    state.seq += 1;
    const asset: Asset = {
      id: `ast-new-${state.seq}`,
      ticker,
      name,
      official_name: officialOf(str(a.official_name), name),
      account_id: str(a.account_id),
      class: a.class,
      class_label: LABEL[a.class],
      market,
      currency,
      country: str(a.country)?.toUpperCase() ?? listedEx?.country ?? null,
      isin: str(a.isin),
      cnpj: str(a.cnpj),
      treasury_code: str(a.treasury_code),
      sector: str(a.sector),
      segment: str(a.segment),
      price_provider: str(a.price_provider),
      provider_symbol: str(a.provider_symbol),
      archived: false,
      notes: str(a.notes),
      fixed_income: a.fixed_income ? validTerms(a.fixed_income) : null,
      exchange: brief(market),
      quote_symbol: null,
    };
    asset.quote_symbol = quoteSymbol(asset);
    state.assets.set(asset.id, asset);
    if (a.pension) {
      // O plano grava junto: se ele falhar, o ativo não fica (como a transação do núcleo).
      try {
        invoke('pension.set_plan', { ...a.pension, asset_id: asset.id });
      } catch (e) {
        state.assets.delete(asset.id);
        throw e;
      }
    }
    return asset;
  },

  'assets.update': (p) => {
    requireUnlocked();
    const cur = { ...getAsset(String(p.id)) };
    if (p.name !== undefined) {
      const n = txt(p.name).trim();
      if (!n) mockFail('invalid', 'informe o nome do ativo');
      cur.name = n;
    }
    if ('official_name' in p) cur.official_name = officialOf(str(p.official_name), cur.name);
    if ('account_id' in p) cur.account_id = str(p.account_id);
    const opt = (
      k: 'ticker' | 'sector' | 'segment' | 'price_provider' | 'provider_symbol' | 'notes',
    ) => {
      if (k in p) {
        const v = p[k] === null ? null : str(p[k]);
        cur[k] = k === 'ticker' && v ? v.toUpperCase() : v;
      }
    };
    (
      ['ticker', 'sector', 'segment', 'price_provider', 'provider_symbol', 'notes'] as const
    ).forEach(opt);
    if (typeof p.class === 'string') {
      cur.class = p.class as AssetClass;
      cur.class_label = LABEL[cur.class];
    }
    if (typeof p.archived === 'boolean') cur.archived = p.archived;
    if (p.fixed_income) cur.fixed_income = validTerms(p.fixed_income);
    if ((cur.class === 'stock_intl' || cur.class === 'etf_intl') && !cur.exchange) {
      mockFail(
        'invalid',
        'ações e ETFs de outras bolsas precisam de uma bolsa do catálogo (LSE, XETRA, TSE…)',
      );
    }
    cur.quote_symbol = quoteSymbol(cur);
    state.assets.set(cur.id, cur);
    return cur;
  },

  'assets.delete': (args) => {
    requireUnlocked();
    const id = String(args.id);
    getAsset(id);
    if (usageProbe(id)) {
      mockFail('conflict', 'o ativo tem operações ou proventos; arquive em vez de apagar');
    }
    state.assets.delete(id);
    onDelete(id);
    return null;
  },

  'market.prices': (args) => {
    requireUnlocked();
    const id = String(args.asset_id);
    getAsset(id);
    const to = typeof args.to === 'string' ? validDate(args.to) : mockToday();
    const from = typeof args.from === 'string' ? validDate(args.from) : addMonths(to, -12);
    return pricePoints(id).filter((p) => p.date >= from && p.date <= to);
  },

  'market.set_price': (args) => {
    requireUnlocked();
    const id = String(args.asset_id);
    getAsset(id);
    const date = validDate(args.date);
    const close = Number(txt(args.close).trim().replace(',', '.'));
    if (!Number.isFinite(close)) mockFail('invalid', `preço inválido: ${String(args.close)}`);
    if (close <= 0) mockFail('invalid', 'o preço deve ser positivo');
    const m = state.manual.get(id) ?? new Map<string, number>();
    m.set(date, close);
    state.manual.set(id, m);
    return null;
  },
});

/** Último dia útil antes de hoje (data da última cotação simulada). */
export function lastSimulatedClose(): string {
  return previousBusinessDay(addDaysSafe(mockToday(), -1));
}
