/**
 * Rótulos dos enums da carteira e dos ativos no idioma do app (catálogo `carteira.ftl`; em
 * pt-BR, os mesmos textos do núcleo, `AssetClass::label` e `IncomeKind::label`), e nomes
 * legíveis das fontes de dados. Cada tabela lê o catálogo na hora (getter), então pode ser
 * importada e guardada em constante sem congelar o idioma.
 */
import type { AssetClass } from '@/api/generated/AssetClass';
import type { CorporateActionKind } from '@/api/generated/CorporateActionKind';
import type { Deflator } from '@/api/generated/Deflator';
import type { FixedIncomeKind } from '@/api/generated/FixedIncomeKind';
import type { FixedIncomeTerms } from '@/api/generated/FixedIncomeTerms';
import type { IncomeKind } from '@/api/generated/IncomeKind';
import type { Indexer } from '@/api/generated/Indexer';
import type { Liquidity } from '@/api/generated/Liquidity';
import type { Valuation } from '@/api/generated/Valuation';
import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';
import { baseCurrency, formatDate } from '@/lib/format';

import { formatDecimal } from './decimal';

/** Chaves sem argumentos (as dos rótulos). */
type PlainKey = { [K in MessageKey]: [Messages[K]] extends [never] ? K : never }[MessageKey];

/**
 * Tabela de rótulos que lê o catálogo a cada acesso (enumerável, como um objeto comum). Um
 * valor que não é chave (`'CDB'`, `'JCP'`) fica como está: nome próprio, igual em todo idioma.
 */
function labels<V extends string>(
  entries: Record<V, PlainKey | { fixed: string }>,
): Record<V, string> {
  const out = {} as Record<V, string>;
  for (const [value, entry] of Object.entries(entries) as [V, PlainKey | { fixed: string }][]) {
    Object.defineProperty(out, value, {
      enumerable: true,
      get: typeof entry === 'string' ? () => t(entry) : () => entry.fixed,
    });
  }
  return out;
}

/** Ordem fixa das classes (a do núcleo): é ela que prende a cor de cada classe. */
export const CLASS_ORDER: readonly AssetClass[] = [
  'stock_br',
  'fii',
  'etf_br',
  'bdr',
  'fixed_income',
  'treasury_br',
  'fund',
  'stock_us',
  'etf_us',
  'reit',
  'crypto',
  'pension',
  'other',
  'stock_intl',
  'etf_intl',
];

export const CLASS_LABEL: Record<AssetClass, string> = labels({
  stock_br: 'carteira-classe-stock-br',
  fii: 'carteira-classe-fii',
  etf_br: 'carteira-classe-etf-br',
  bdr: 'carteira-classe-bdr',
  fixed_income: 'carteira-classe-fixed-income',
  treasury_br: 'carteira-classe-treasury-br',
  fund: 'carteira-classe-fund',
  stock_us: 'carteira-classe-stock-us',
  etf_us: 'carteira-classe-etf-us',
  reit: 'carteira-classe-reit',
  crypto: 'carteira-classe-crypto',
  pension: 'carteira-classe-pension',
  other: 'carteira-classe-other',
  stock_intl: 'carteira-classe-stock-intl',
  etf_intl: 'carteira-classe-etf-intl',
});

/**
 * Rótulo da classe no idioma do app; uma classe que a interface não conhece (ou um id que não é
 * classe) fica com o rótulo do núcleo.
 */
export function classLabel(cls: string, fallback?: string | null): string {
  return Object.hasOwn(CLASS_LABEL, cls) ? CLASS_LABEL[cls as AssetClass] : fallback || cls;
}

/**
 * Renda fixa (CDB, RDB, LCI, LCA…): a pessoa só lida com valores. A quantidade existe por dentro
 * (valor × PU 1 na compra, para a curva e o PEPS), mas a interface não a pede nem a mostra.
 */
export function isValueOnly(cls: string | null | undefined): boolean {
  return cls === 'fixed_income';
}

/** Classes de outras bolsas (fora do Brasil e dos EUA): exigem a bolsa no cadastro. */
export function needsExchange(cls: AssetClass): boolean {
  return cls === 'stock_intl' || cls === 'etf_intl';
}

/** Negociado em bolsa: a bolsa do catálogo dá moeda, país e símbolo do Yahoo. */
export function isExchangeListed(cls: AssetClass): boolean {
  return (
    cls === 'stock_br' ||
    cls === 'fii' ||
    cls === 'etf_br' ||
    cls === 'bdr' ||
    cls === 'stock_us' ||
    cls === 'etf_us' ||
    cls === 'reit' ||
    needsExchange(cls)
  );
}

/**
 * Moeda padrão do núcleo para cada classe (`default_currency`); com bolsa do catálogo,
 * vale a moeda da bolsa. Renda fixa, previdência e outros ficam na moeda base da família; as
 * classes brasileiras (B3, Tesouro, fundos da CVM), em reais.
 */
export function defaultCurrency(cls: AssetClass, exchangeCurrency?: string | null): string {
  if (exchangeCurrency && isExchangeListed(cls)) return exchangeCurrency;
  if (cls === 'fixed_income' || cls === 'pension' || cls === 'other') return baseCurrency();
  return cls === 'stock_us' ||
    cls === 'etf_us' ||
    cls === 'reit' ||
    cls === 'crypto' ||
    needsExchange(cls)
    ? 'USD'
    : 'BRL';
}

/** Mercado padrão do núcleo (`default_market`). */
export function defaultMarket(cls: AssetClass): string | null {
  switch (cls) {
    case 'stock_br':
    case 'fii':
    case 'etf_br':
    case 'bdr':
      return 'B3';
    case 'treasury_br':
      return 'TESOURO';
    case 'fund':
      return 'CVM';
    case 'crypto':
      return 'CRYPTO';
    default:
      return null;
  }
}

export const INCOME_KIND_LABEL: Record<IncomeKind, string> = labels({
  dividend: 'carteira-provento-dividend',
  jcp: { fixed: 'JCP' },
  fii_income: 'carteira-provento-fii-income',
  coupon: 'carteira-provento-coupon',
  interest: 'carteira-provento-interest',
  rent: 'carteira-provento-rent',
  amortization: 'carteira-provento-amortization',
  other: 'carteira-provento-other',
});

export const ACTION_KIND_LABEL: Record<CorporateActionKind, string> = labels({
  split: 'carteira-evento-split',
  reverse_split: 'carteira-evento-reverse-split',
  bonus: 'carteira-evento-bonus',
  subscription: 'carteira-evento-subscription',
  merger: 'carteira-evento-merger',
  ticker_change: 'carteira-evento-ticker-change',
  spin_off: 'carteira-evento-spin-off',
  capital_return: 'carteira-evento-capital-return',
});

const DEFLATOR_LABEL = labels<Deflator>({
  ipca: { fixed: 'IPCA' },
  igpm: { fixed: 'IGP-M' },
  country: 'carteira-deflator-pais',
  us_cpi: { fixed: 'US CPI' },
  m2_br: { fixed: 'M2 BR' },
  m2_us: 'carteira-deflator-m2-us',
  none: 'carteira-deflator-nominal',
});

const DEFLATOR_HINT = labels<Deflator>({
  ipca: 'carteira-deflator-ipca-dica',
  igpm: 'carteira-deflator-igpm-dica',
  country: 'carteira-deflator-pais-dica',
  us_cpi: 'carteira-deflator-us-cpi-dica',
  m2_br: 'carteira-deflator-m2-br-dica',
  m2_us: 'carteira-deflator-m2-us-dica',
  none: 'carteira-deflator-nominal-dica',
});

export const DEFLATOR_OPTIONS: readonly { value: Deflator; label: string; hint: string }[] = (
  ['ipca', 'igpm', 'country', 'us_cpi', 'm2_br', 'm2_us', 'none'] as const
).map((value) => ({
  value,
  get label() {
    return DEFLATOR_LABEL[value];
  },
  get hint() {
    return DEFLATOR_HINT[value];
  },
}));

/** Rótulo do deflator; `country` leva o nome do país de referência quando conhecido. */
export function deflatorLabel(d: Deflator, countryName?: string | null): string {
  if (d === 'country')
    return countryName ? t('carteira-deflator-pais-de', { pais: countryName }) : DEFLATOR_LABEL[d];
  return (DEFLATOR_LABEL as Record<string, string | undefined>)[d] ?? d;
}

export const FIXED_INCOME_KIND_LABEL: Record<FixedIncomeKind, string> = labels({
  cdb: { fixed: 'CDB' },
  rdb: { fixed: 'RDB' },
  lc: { fixed: 'LC' },
  lci: { fixed: 'LCI' },
  lca: { fixed: 'LCA' },
  cri: { fixed: 'CRI' },
  cra: { fixed: 'CRA' },
  debenture: 'carteira-rf-debenture',
  debenture_incentivada: 'carteira-rf-debenture-incentivada',
  tesouro_selic: { fixed: 'Tesouro Selic' },
  tesouro_prefixado: { fixed: 'Tesouro Prefixado' },
  tesouro_ipca: { fixed: 'Tesouro IPCA+' },
  tesouro_renda: { fixed: 'Tesouro RendA+' },
  poupanca: 'carteira-rf-poupanca',
  other: 'carteira-rf-outro',
});

/** Isento de IR para pessoa física por padrão (`FixedIncomeKind::default_ir_exempt`). */
export function defaultIrExempt(kind: FixedIncomeKind): boolean {
  return (
    kind === 'lci' ||
    kind === 'lca' ||
    kind === 'cri' ||
    kind === 'cra' ||
    kind === 'debenture_incentivada' ||
    kind === 'poupanca'
  );
}

export const INDEXER_LABEL: Record<Indexer, string> = labels({
  cdi: { fixed: 'CDI' },
  selic: { fixed: 'Selic' },
  ipca: { fixed: 'IPCA' },
  igpm: { fixed: 'IGP-M' },
  prefixado: 'carteira-indexador-prefixado',
});

export const LIQUIDITY_LABEL: Record<Liquidity, string> = labels({
  daily: 'carteira-liquidez-diaria',
  at_maturity: 'carteira-liquidez-vencimento',
  after_grace: 'carteira-liquidez-carencia',
});

/** Como ler a taxa de cada indexador (ajuda do campo). */
export function rateHint(indexer: Indexer): string {
  switch (indexer) {
    case 'cdi':
    case 'selic':
      return t('carteira-taxa-dica-percentual', { indice: INDEXER_LABEL[indexer] });
    case 'prefixado':
      return t('carteira-taxa-dica-prefixado');
    default:
      return t('carteira-taxa-dica-real', { indice: INDEXER_LABEL[indexer] });
  }
}

/** Resumo dos termos: `110% do CDI · venc. 15/01/2028`, `IPCA + 6,2% a.a.`. */
export function termsSummary(terms: FixedIncomeTerms): string {
  const taxa = formatDecimal(terms.rate, { maxDecimals: 4 });
  const head =
    terms.indexer === 'cdi' || terms.indexer === 'selic'
      ? t('carteira-termos-percentual', { taxa, indice: INDEXER_LABEL[terms.indexer] })
      : terms.indexer === 'prefixado'
        ? t('carteira-termos-pre', { taxa })
        : t('carteira-termos-indice', { taxa, indice: INDEXER_LABEL[terms.indexer] });
  const parts = [head];
  if (terms.maturity_date)
    parts.push(t('carteira-termos-vencimento', { data: safeDate(terms.maturity_date) }));
  return parts.join(' · ');
}

export function safeDate(iso: string | null | undefined): string {
  if (!iso) return '—';
  try {
    return formatDate(iso);
  } catch {
    return iso;
  }
}

export const VALUATION_LABEL: Record<Valuation, string> = labels({
  market: 'carteira-avaliacao-mercado',
  curve: 'carteira-avaliacao-curva',
  missing: 'carteira-avaliacao-sem-cotacao',
});

const SOURCE_NAMES: Record<string, string> = labels({
  b3_cotahist: { fixed: 'B3 COTAHIST' },
  b3_listados: 'carteira-fonte-b3-listados',
  fundamentus: { fixed: 'Fundamentus' },
  brapi: { fixed: 'brapi.dev' },
  yahoo: { fixed: 'Yahoo Finance' },
  yahoo_intraday: 'carteira-fonte-yahoo-intraday',
  coingecko: { fixed: 'CoinGecko' },
  defillama: { fixed: 'DefiLlama' },
  tiingo: { fixed: 'Tiingo' },
  eodhd: { fixed: 'EODHD' },
  tesouro: { fixed: 'Tesouro Transparente' },
  tesouro_transparente: { fixed: 'Tesouro Transparente' },
  cvm: { fixed: 'CVM Dados Abertos' },
  manual: 'carteira-fonte-manual-voce',
  mock: 'carteira-fonte-mock',
  radar: 'carteira-fonte-radar',
  bcb: 'carteira-fonte-bcb',
  bcb_ptax: { fixed: 'BCB PTAX' },
  focus: { fixed: 'BCB Focus' },
  fred: { fixed: 'FRED' },
  eurostat: { fixed: 'Eurostat' },
  oecd: 'carteira-fonte-oecd',
  worldbank: 'carteira-fonte-worldbank',
  ecb_ref: 'carteira-fonte-bce-ref',
  boc: { fixed: 'Bank of Canada' },
  boe: { fixed: 'Bank of England' },
});

/**
 * Nome legível de uma fonte: `bcb_sgs:433` → `BCB/SGS 433`, `fred:CPIAUCSL` →
 * `FRED CPIAUCSL`, `b3_cotahist` → `B3 COTAHIST`.
 */
export function sourceName(source: string | null | undefined): string {
  if (!source) return t('carteira-fonte-nao-informada');
  const [head = '', code] = source.split(':');
  if (head === 'bcb_sgs') return `BCB/SGS ${code ?? ''}`.trim();
  if (head === 'ibge_sidra') return `IBGE/SIDRA ${code ?? ''}`.trim();
  if (head === 'fred') return `FRED ${code ?? ''}`.trim();
  if (head === 'bcb_olinda') return `BCB Olinda ${code ?? ''}`.trim();
  if (head === 'yahoo' && code) return `Yahoo Finance ${code}`;
  if (head === 'ecb' && code)
    return `${t('carteira-fonte-bce')} ${code.split('/')[0] ?? ''}`.trim();
  if (head === 'boc' && code) return `Bank of Canada ${code}`;
  if (head === 'ons' && code) return `ONS ${code.split('/').pop()?.toUpperCase() ?? ''}`.trim();
  if (head === 'boe' && code) return `Bank of England ${code}`;
  return SOURCE_NAMES[head] ?? source;
}

/** Rótulo curto do ativo: ticker ou nome. */
export function assetLabel(a: { ticker: string | null; name: string }): string {
  return a.ticker ?? a.name;
}

/**
 * De onde vem a cotação do ativo, como a coleta do núcleo decide (`market::sync::default_source`):
 * Yahoo Finance para B3, EUA e cripto (com ticker), Tesouro Transparente para o Tesouro,
 * curva para a renda fixa; fundos, previdência e outros ficam sem fonte automática.
 */
export function quoteSourceLabel(a: {
  class: AssetClass;
  market?: string | null;
  ticker: string | null;
  price_provider: string | null;
  provider_symbol: string | null;
}): string {
  if (a.price_provider === 'manual') return t('carteira-fonte-manual');
  if (a.price_provider === 'none') return t('carteira-fonte-sem-cotacao');
  if (a.class === 'fixed_income') return t('carteira-fonte-na-curva');
  if (a.price_provider === 'tesouro' || a.class === 'treasury_br') return 'Tesouro Transparente';
  if (a.price_provider === 'yahoo' || a.provider_symbol) return 'Yahoo Finance';
  if (!a.ticker) return t('carteira-fonte-manual-sem-fonte');
  switch (a.class) {
    case 'stock_br':
    case 'fii':
    case 'etf_br':
    case 'bdr':
    case 'stock_us':
    case 'etf_us':
    case 'reit':
    case 'crypto':
      return 'Yahoo Finance';
    case 'stock_intl':
    case 'etf_intl':
      return a.market ? 'Yahoo Finance' : t('carteira-fonte-manual-bolsa');
    default:
      return t('carteira-fonte-manual-sem-fonte');
  }
}
