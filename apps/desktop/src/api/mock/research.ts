/**
 * Mock de `research.*` (análise de ativos): um universo de ~34 papéis (as ações, FIIs e
 * ações americanas cadastradas no mock de ativos, com a cotação de lá, mais papéis só do
 * universo), indicadores com fórmula/período/fonte ou motivo de indisponibilidade, preço
 * justo de Graham e Bazin, histórico de 10 anos, documentos IPE, ranking, screener, telas
 * salvas (no banco do mock, com desfazer), comparador, estado e coleta.
 *
 * Números ilustrativos (coerentes entre si: P/L = cotação ÷ LPA, ROE = LPA ÷ VPA…), não
 * dados reais. Mesmas unidades do núcleo: razões em fração, dinheiro dos indicadores em
 * unidades da moeda, histórico em centavos.
 */
import type { DocumentsStatus } from '../generated/DocumentsStatus';
import type { MarketMapPoint } from '../generated/MarketMapPoint';
import type { MethodCheck } from '../generated/MethodCheck';
import type { MethodExplain } from '../generated/MethodExplain';
import type { MethodRun } from '../generated/MethodRun';
import type { ScreenMethod } from '../generated/ScreenMethod';
import type { MarketOverview } from '../generated/MarketOverview';
import type { MarketSector } from '../generated/MarketSector';
import type { AssetClass } from '../generated/AssetClass';
import type { CmpOp } from '../generated/CmpOp';
import type { CompareRow } from '../generated/CompareRow';
import type { CompareView } from '../generated/CompareView';
import type { AssetTag } from '../generated/AssetTag';
import type { Condition } from '../generated/Condition';
import { sourceEnabled } from './sources';
import type { CorporateDocument } from '../generated/CorporateDocument';
import type { NewsItem } from '../generated/NewsItem';
import type { NewsView } from '../generated/NewsView';
import type { FairPriceMethod } from '../generated/FairPriceMethod';
import type { FairPriceView } from '../generated/FairPriceView';
import type { HistoryView } from '../generated/HistoryView';
import type { Indicator } from '../generated/Indicator';
import type { IndicatorGroup } from '../generated/IndicatorGroup';
import type { IndicatorInfo } from '../generated/IndicatorInfo';
import type { IndicatorsView } from '../generated/IndicatorsView';
import type { IndicatorUnit } from '../generated/IndicatorUnit';
import type { Logic } from '../generated/Logic';
import type { MonthRow } from '../generated/MonthRow';
import type { PriceOn } from '../generated/PriceOn';
import type { QuarterRow } from '../generated/QuarterRow';
import type { RankItem } from '../generated/RankItem';
import type { RankResult } from '../generated/RankResult';
import type { ResearchSettings } from '../generated/ResearchSettings';
import type { ResearchSettingsView } from '../generated/ResearchSettingsView';
import type { ResearchStatus } from '../generated/ResearchStatus';
import type { ResearchSyncReport } from '../generated/ResearchSyncReport';
import type { Screen } from '../generated/Screen';
import type { ScreenQuery } from '../generated/ScreenQuery';
import type { ScreenResult } from '../generated/ScreenResult';
import type { SortOrder } from '../generated/SortOrder';
import type { Subject } from '../generated/Subject';
import type { SyncItem } from '../generated/SyncItem';
import type { SyncSource } from '../generated/SyncSource';
import type { UniverseFilter } from '../generated/UniverseFilter';
import type { YearRow } from '../generated/YearRow';
import { extraMockHandler, onMockReset, registerMockHandlers } from '../mockRegistry';
import { closeOn, getAsset, hasAsset, pricePoints } from './assets';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import { conflict, invalid, notFound } from './ledger/errors';
import { del, find, newId, put, rows, write } from './ledger/store';
import { addDays, addMonths, hashSeed, mockToday } from './marketSim';

// ── Catálogo (o mesmo de `research::indicators::CATALOG`) ────────

type Scope = 'stock' | 'fii' | 'both';
interface Def {
  key: string;
  label: string;
  group: IndicatorGroup;
  unit: IndicatorUnit;
  hib: boolean | null;
  positiveOnly: boolean;
  scope: Scope;
  formula: string;
}

const D = (
  key: string,
  label: string,
  group: IndicatorGroup,
  unit: IndicatorUnit,
  hib: boolean | null,
  positiveOnly: boolean,
  scope: Scope,
  formula: string,
): Def => ({ key, label, group, unit, hib, positiveOnly, scope, formula });

const LTM = 'nos últimos 12 meses';
/** Campos do score em lote (`market_score`) e os indicadores dos métodos do explorador. */
const MARKET_DEFS: Def[] = [
  D(
    'score',
    'Score',
    'score',
    'count',
    true,
    false,
    'both',
    'Score de saúde do Treasury (0 a 100)',
  ),
  D('health', 'Saúde (score)', 'score', 'count', true, false, 'both', 'Sub-score Saúde do negócio'),
  D(
    'attractiveness',
    'Atratividade (score)',
    'score',
    'count',
    true,
    false,
    'both',
    'Sub-score Atratividade do preço',
  ),
  D(
    'confidence',
    'Confiança do score',
    'score',
    'percent',
    true,
    false,
    'both',
    'Cobertura × frescor × pares',
  ),
  D(
    'piotroski',
    'Piotroski F-Score',
    'score',
    'count',
    true,
    false,
    'stock',
    'Critérios cumpridos (0 a 9)',
  ),
  D('altman_z', "Altman Z''", 'score', 'multiple', true, false, 'stock', "Z'' de Altman"),
  D('beneish', 'Beneish M-Score', 'score', 'multiple', false, false, 'stock', 'M-Score de Beneish'),
  D(
    'momentum_12_1',
    'Momentum 12-1',
    'score',
    'percent',
    true,
    false,
    'both',
    'Retorno de 12 meses sem o último',
  ),
  D(
    'volatility_1y',
    'Volatilidade 1 ano',
    'score',
    'percent',
    false,
    false,
    'both',
    'Desvio padrão anualizado',
  ),
  D(
    'p_ebit',
    'P/EBIT',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor de mercado ÷ EBIT dos últimos 12 meses',
  ),
  D(
    'p_working_capital',
    'P/Capital de giro',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor de mercado ÷ (ativo circulante − passivo circulante)',
  ),
  D(
    'p_ncav',
    'P/Ativo circulante líquido',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor de mercado ÷ (ativo circulante − passivo total), o "net-net" de Graham',
  ),
  D(
    'ebit_assets',
    'EBIT/Ativo',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'EBIT dos últimos 12 meses ÷ ativo total',
  ),
  D(
    'asset_turnover',
    'Giro dos ativos',
    'profitability',
    'multiple',
    true,
    false,
    'stock',
    'Receita líquida dos últimos 12 meses ÷ ativo total',
  ),
  D(
    'gross_debt_equity',
    'Dívida bruta / PL',
    'debt',
    'multiple',
    false,
    false,
    'stock',
    'Empréstimos e financiamentos ÷ patrimônio líquido dos controladores',
  ),
  D(
    'earnings_yield',
    'EBIT/EV',
    'valuation',
    'percent',
    true,
    true,
    'stock',
    'EBIT ÷ valor da firma',
  ),
  D(
    'magic_roc',
    'ROC (Magic Formula)',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'EBIT ÷ (capital de giro + imobilizado)',
  ),
  D(
    'fcf_yield',
    'Fluxo de caixa livre / valor de mercado',
    'valuation',
    'percent',
    true,
    false,
    'stock',
    '(FCO − capex) ÷ valor de mercado',
  ),
  D(
    'peg',
    'PEG',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'P/L ÷ crescimento do lucro em 5 anos',
  ),
  D(
    'eps_growth_10y',
    'Crescimento do lucro em 10 anos',
    'growth',
    'percent',
    true,
    false,
    'stock',
    'Lucro médio recente ÷ de 10 anos atrás − 1',
  ),
  D(
    'dividend_years',
    'Anos seguidos com proventos',
    'dividends',
    'count',
    true,
    false,
    'both',
    'Anos completos seguidos com provento',
  ),
  D(
    'dividend_cagr_5y',
    'Crescimento dos proventos em 5 anos',
    'dividends',
    'percent',
    true,
    false,
    'both',
    'Crescimento anual dos proventos',
  ),
];

/** Score em lote simulado (determinístico por ticker), como o `market_score` da coleta. */
function marketScore(e: { ticker: string; spec: Spec }): {
  metrics: Record<string, number>;
  band: string;
  flags: string;
} | null {
  if (e.spec.kind === 'stock' && e.spec.us) return null;
  const h = hashSeed(`score:${e.ticker}`);
  const score = 35 + (h % 60);
  const band =
    score >= 80 ? 'robusto' : score >= 65 ? 'saudavel' : score >= 50 ? 'atencao' : 'risco';
  const metrics: Record<string, number> = {
    score,
    health: Math.min(100, score + (h % 11) - 5),
    attractiveness: Math.max(0, score - (h % 17) + 8),
    confidence: 0.6 + (h % 40) / 100,
    momentum_12_1: ((h % 50) - 20) / 100,
    volatility_1y: 0.18 + (h % 25) / 100,
  };
  if (e.spec.kind === 'stock') {
    metrics.piotroski = h % 10;
    metrics.altman_z = 1 + (h % 40) / 10;
    metrics.beneish = -3 + (h % 20) / 10;
  }
  return { metrics, band, flags: score < 45 ? 'divida_alta' : '' };
}

const CATALOG: Def[] = [
  D('price', 'Cotação', 'market', 'money', null, false, 'both', 'Último fechamento disponível'),
  D(
    'market_cap',
    'Valor de mercado',
    'market',
    'money',
    true,
    true,
    'both',
    'Σ ações de cada classe × cotação da classe (FII: cotas × cotação)',
  ),
  D(
    'ev',
    'Valor da firma (EV)',
    'market',
    'money',
    null,
    false,
    'stock',
    'Valor de mercado + dívida bruta − caixa e aplicações financeiras',
  ),
  D(
    'liquidity',
    'Liquidez média diária',
    'market',
    'money',
    true,
    true,
    'both',
    'Média do volume financeiro diário nos últimos pregões (até 21)',
  ),
  D(
    'pl',
    'P/L',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    `Valor de mercado ÷ lucro líquido dos controladores ${LTM}`,
  ),
  D(
    'pvp',
    'P/VP',
    'valuation',
    'multiple',
    false,
    true,
    'both',
    'Valor de mercado ÷ patrimônio líquido dos controladores (FII: cotação ÷ valor patrimonial por cota)',
  ),
  D(
    'ev_ebitda',
    'EV/EBITDA',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor da firma ÷ EBITDA dos últimos 12 meses (EBIT + depreciação e amortização)',
  ),
  D(
    'ev_ebit',
    'EV/EBIT',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor da firma ÷ EBIT dos últimos 12 meses',
  ),
  D(
    'psr',
    'P/Receita (PSR)',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor de mercado ÷ receita líquida dos últimos 12 meses',
  ),
  D(
    'p_ativo',
    'P/Ativo',
    'valuation',
    'multiple',
    false,
    true,
    'stock',
    'Valor de mercado ÷ ativo total',
  ),
  D(
    'dy',
    'Dividend yield (12 meses)',
    'dividends',
    'percent',
    true,
    false,
    'both',
    'Proventos por ação com data-com nos últimos 12 meses ÷ cotação',
  ),
  D(
    'payout',
    'Payout',
    'dividends',
    'percent',
    null,
    false,
    'stock',
    'Dividendos e JCP declarados nos últimos 12 meses ÷ lucro líquido',
  ),
  D(
    'last_dividend',
    'Último rendimento',
    'dividends',
    'money',
    true,
    false,
    'fii',
    'Último provento por cota (agenda; sem agenda, DY do mês × VP por cota do informe)',
  ),
  D(
    'roe',
    'ROE',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'Lucro líquido dos últimos 12 meses ÷ patrimônio líquido',
  ),
  D(
    'roic',
    'ROIC',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'EBIT dos últimos 12 meses × (1 − alíquota nominal de IR) ÷ (patrimônio líquido + dívida líquida)',
  ),
  D(
    'roa',
    'ROA',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'Lucro líquido dos últimos 12 meses ÷ ativo total',
  ),
  D(
    'gross_margin',
    'Margem bruta',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'Lucro bruto ÷ receita líquida (12 meses)',
  ),
  D(
    'ebitda_margin',
    'Margem EBITDA',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'EBITDA ÷ receita líquida (12 meses)',
  ),
  D(
    'ebit_margin',
    'Margem EBIT',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'EBIT ÷ receita líquida (12 meses)',
  ),
  D(
    'net_margin',
    'Margem líquida',
    'profitability',
    'percent',
    true,
    false,
    'stock',
    'Lucro líquido ÷ receita líquida (12 meses)',
  ),
  D(
    'net_debt_equity',
    'Dívida líquida / PL',
    'debt',
    'multiple',
    false,
    false,
    'stock',
    '(Dívida bruta − caixa e aplicações) ÷ patrimônio líquido',
  ),
  D(
    'net_debt_ebitda',
    'Dívida líquida / EBITDA',
    'debt',
    'multiple',
    false,
    false,
    'stock',
    '(Dívida bruta − caixa e aplicações) ÷ EBITDA dos últimos 12 meses',
  ),
  D(
    'current_ratio',
    'Liquidez corrente',
    'debt',
    'multiple',
    true,
    false,
    'stock',
    'Ativo circulante ÷ passivo circulante',
  ),
  D(
    'revenue_cagr_5y',
    'CAGR receita 5 anos',
    'growth',
    'percent',
    true,
    false,
    'stock',
    '(Receita do último exercício ÷ receita de 5 exercícios antes)^(1/5) − 1',
  ),
  D(
    'earnings_cagr_5y',
    'CAGR lucro 5 anos',
    'growth',
    'percent',
    true,
    false,
    'stock',
    '(Lucro do último exercício ÷ lucro de 5 exercícios antes)^(1/5) − 1',
  ),
  D(
    'loss_years_10y',
    'Anos com prejuízo (10 anos)',
    'growth',
    'count',
    false,
    false,
    'stock',
    'Exercícios com lucro líquido negativo entre os últimos 10',
  ),
  D(
    'lpa',
    'LPA',
    'per_share',
    'money',
    true,
    false,
    'stock',
    'Lucro líquido dos últimos 12 meses ÷ ações em circulação (× ações por unit)',
  ),
  D(
    'vpa',
    'VPA',
    'per_share',
    'money',
    true,
    false,
    'stock',
    'Patrimônio líquido ÷ ações em circulação (× ações por unit)',
  ),
  D(
    'nav_per_share',
    'Valor patrimonial por cota',
    'fii',
    'money',
    null,
    false,
    'fii',
    'Patrimônio líquido ÷ cotas emitidas (informe mensal)',
  ),
  D(
    'holders',
    'Número de cotistas',
    'fii',
    'count',
    true,
    false,
    'fii',
    'Total de cotistas no último informe mensal',
  ),
  D('vacancy', 'Vacância', 'fii', 'percent', false, false, 'fii', 'Área vaga ÷ área bruta locável'),
  D(
    'basel',
    'Índice de Basileia',
    'bank',
    'percent',
    true,
    false,
    'stock',
    'Patrimônio de referência ÷ ativos ponderados pelo risco',
  ),
  D(
    'graham',
    'Preço de Graham',
    'fair_price',
    'money',
    null,
    false,
    'stock',
    '√(22,5 × LPA × VPA)',
  ),
  D(
    'graham_upside',
    'Margem até o preço de Graham',
    'fair_price',
    'percent',
    true,
    false,
    'stock',
    'Preço de Graham ÷ cotação − 1',
  ),
  D(
    'bazin',
    'Preço-teto de Bazin',
    'fair_price',
    'money',
    null,
    false,
    'both',
    'Média dos dividendos por ação dos últimos N anos ÷ taxa mínima',
  ),
  D(
    'bazin_upside',
    'Margem até o preço de Bazin',
    'fair_price',
    'percent',
    true,
    false,
    'both',
    'Preço-teto de Bazin ÷ cotação − 1',
  ),
  ...MARKET_DEFS,
];

const DEF = new Map(CATALOG.map((d) => [d.key, d]));

function catalog(): IndicatorInfo[] {
  return CATALOG.map((d) => ({
    key: d.key,
    label: d.label,
    group: d.group,
    unit: d.unit,
    formula: d.formula,
    higher_is_better: d.hib,
    applies_to: d.scope === 'both' ? ['stock', 'fii'] : [d.scope],
  }));
}

const DISCLAIMER =
  'Estimativas a partir de fórmulas clássicas e dos dados informados; não são recomendação de investimento.';

// ── Universo ─────────────────────────────────────────────────────

interface StockSpec {
  kind: 'stock';
  ticker: string;
  name: string;
  issuer: string;
  sector: string;
  segment: string | null;
  bank?: boolean;
  /** Ações por unit. */
  unit?: number;
  us?: boolean;
  /** Cotação dos papéis sem cadastro (os cadastrados usam a do mock de ativos). */
  price?: number;
  /** Milhões de ações (units). */
  shares: number;
  lpa: number;
  vpa: number;
  /** Dividendos por ação de 2021 a 2025. */
  dps: [number, number, number, number, number];
  /** Receita LTM em bilhões. */
  revenue: number;
  margins?: [gross: number, ebitda: number, ebit: number];
  netDebtEbitda?: number;
  currentRatio?: number;
  /** Ativo total em bilhões. */
  assets: number;
  revCagr: number;
  epsCagr: number | null;
  lossYears: number;
  /** Liquidez diária em milhões. */
  liquidity: number;
}

interface FiiSpec {
  kind: 'fii';
  ticker: string;
  name: string;
  issuer: string;
  segment: string;
  price?: number;
  /** Milhões de cotas. */
  shares: number;
  nav: number;
  income: number;
  holders: number;
  liquidity: number;
}

type Spec = StockSpec | FiiSpec;

const S = (s: Omit<StockSpec, 'kind'>): StockSpec => ({ kind: 'stock', ...s });
const F = (s: Omit<FiiSpec, 'kind'>): FiiSpec => ({ kind: 'fii', ...s });

const SPECS: Spec[] = [
  S({
    ticker: 'PETR4',
    name: 'Petrobras PN',
    issuer: 'PETRÓLEO BRASILEIRO S.A. - PETROBRAS',
    sector: 'Petróleo, gás e biocombustíveis',
    segment: 'Exploração, refino e distribuição',
    shares: 13044,
    lpa: 5.8,
    vpa: 33,
    dps: [2.9, 6.7, 4.4, 3.1, 3.4],
    revenue: 490,
    margins: [0.5, 0.45, 0.33],
    netDebtEbitda: 1.5,
    currentRatio: 0.95,
    assets: 1050,
    revCagr: 0.09,
    epsCagr: 0.18,
    lossYears: 1,
    liquidity: 1650,
  }),
  S({
    ticker: 'ITUB4',
    name: 'Itaú Unibanco PN',
    issuer: 'ITAÚ UNIBANCO HOLDING S.A.',
    sector: 'Financeiro',
    segment: 'Bancos',
    bank: true,
    shares: 9800,
    lpa: 4.25,
    vpa: 21,
    dps: [1.1, 1.45, 2.0, 2.7, 3.1],
    revenue: 160,
    assets: 2900,
    revCagr: 0.07,
    epsCagr: 0.11,
    lossYears: 0,
    liquidity: 1200,
  }),
  S({
    ticker: 'WEGE3',
    name: 'WEG ON',
    issuer: 'WEG S.A.',
    sector: 'Bens industriais',
    segment: 'Máquinas e equipamentos',
    shares: 4196,
    lpa: 1.58,
    vpa: 6.1,
    dps: [0.55, 0.62, 0.76, 0.92, 0.98],
    revenue: 39,
    margins: [0.33, 0.22, 0.19],
    netDebtEbitda: -0.3,
    currentRatio: 1.85,
    assets: 52,
    revCagr: 0.19,
    epsCagr: 0.21,
    lossYears: 0,
    liquidity: 410,
  }),
  S({
    ticker: 'BBAS3',
    name: 'Banco do Brasil ON',
    issuer: 'BANCO DO BRASIL S.A.',
    sector: 'Financeiro',
    segment: 'Bancos',
    bank: true,
    shares: 5730,
    lpa: 5.1,
    vpa: 33.5,
    dps: [1.05, 1.6, 2.4, 2.15, 1.7],
    revenue: 140,
    assets: 2300,
    revCagr: 0.08,
    epsCagr: 0.12,
    lossYears: 0,
    liquidity: 520,
  }),
  S({
    ticker: 'MGLU3',
    name: 'Magazine Luiza ON',
    issuer: 'MAGAZINE LUIZA S.A.',
    sector: 'Consumo cíclico',
    segment: 'Varejo',
    shares: 739,
    lpa: -0.25,
    vpa: 13.5,
    dps: [0, 0, 0, 0, 0.02],
    revenue: 39,
    margins: [0.3, 0.07, 0.02],
    netDebtEbitda: 2.1,
    currentRatio: 1.15,
    assets: 39,
    revCagr: 0.08,
    epsCagr: null,
    lossYears: 4,
    liquidity: 160,
  }),
  S({
    ticker: 'VALE3',
    name: 'Vale ON',
    issuer: 'VALE S.A.',
    sector: 'Materiais básicos',
    segment: 'Mineração',
    shares: 4270,
    lpa: 6.9,
    vpa: 44,
    dps: [16.2, 8.9, 7.0, 4.2, 4.6],
    revenue: 215,
    margins: [0.38, 0.4, 0.3],
    netDebtEbitda: 0.9,
    currentRatio: 1.1,
    assets: 480,
    revCagr: 0.05,
    epsCagr: -0.02,
    lossYears: 1,
    liquidity: 1900,
  }),
  S({
    ticker: 'TAEE11',
    name: 'Taesa Unit',
    issuer: 'TRANSMISSORA ALIANÇA DE ENERGIA ELÉTRICA S.A.',
    sector: 'Utilidade pública',
    segment: 'Energia elétrica',
    unit: 3,
    shares: 344,
    lpa: 3.9,
    vpa: 21,
    dps: [3.55, 4.1, 3.4, 3.7, 3.8],
    revenue: 3.9,
    margins: [0.7, 0.78, 0.74],
    netDebtEbitda: 3.4,
    currentRatio: 1.6,
    assets: 25,
    revCagr: 0.06,
    epsCagr: 0.04,
    lossYears: 0,
    liquidity: 85,
  }),
  S({
    ticker: 'ABEV3',
    name: 'Ambev ON',
    issuer: 'AMBEV S.A.',
    sector: 'Consumo não cíclico',
    segment: 'Bebidas',
    price: 13.4,
    shares: 15750,
    lpa: 0.95,
    vpa: 6,
    dps: [0.62, 0.72, 0.75, 0.8, 0.84],
    revenue: 88,
    margins: [0.51, 0.31, 0.24],
    netDebtEbitda: -0.5,
    currentRatio: 1.0,
    assets: 140,
    revCagr: 0.09,
    epsCagr: 0.03,
    lossYears: 0,
    liquidity: 380,
  }),
  S({
    ticker: 'BBDC4',
    name: 'Bradesco PN',
    issuer: 'BANCO BRADESCO S.A.',
    sector: 'Financeiro',
    segment: 'Bancos',
    bank: true,
    price: 16.8,
    shares: 10600,
    lpa: 1.85,
    vpa: 15.4,
    dps: [1.05, 1.25, 1.05, 0.95, 1.1],
    revenue: 150,
    assets: 2000,
    revCagr: 0.03,
    epsCagr: -0.04,
    lossYears: 0,
    liquidity: 720,
  }),
  S({
    ticker: 'B3SA3',
    name: 'B3 ON',
    issuer: 'B3 S.A. - BRASIL, BOLSA, BALCÃO',
    sector: 'Financeiro',
    segment: 'Serviços financeiros diversos',
    price: 13.2,
    shares: 5300,
    lpa: 0.84,
    vpa: 3.6,
    dps: [0.62, 0.55, 0.6, 0.63, 0.66],
    revenue: 10.5,
    margins: [0.8, 0.64, 0.58],
    netDebtEbitda: 0.6,
    currentRatio: 1.4,
    assets: 52,
    revCagr: 0.06,
    epsCagr: 0.05,
    lossYears: 0,
    liquidity: 520,
  }),
  S({
    ticker: 'SUZB3',
    name: 'Suzano ON',
    issuer: 'SUZANO S.A.',
    sector: 'Materiais básicos',
    segment: 'Papel e celulose',
    price: 56.1,
    shares: 1240,
    lpa: 5.6,
    vpa: 36,
    dps: [1.73, 2.49, 1.79, 1.8, 2.0],
    revenue: 49,
    margins: [0.42, 0.45, 0.3],
    netDebtEbitda: 3.1,
    currentRatio: 2.1,
    assets: 165,
    revCagr: 0.14,
    epsCagr: 0.22,
    lossYears: 2,
    liquidity: 300,
  }),
  S({
    ticker: 'RENT3',
    name: 'Localiza ON',
    issuer: 'LOCALIZA RENT A CAR S.A.',
    sector: 'Consumo cíclico',
    segment: 'Aluguel de carros',
    price: 43,
    shares: 1090,
    lpa: 1.6,
    vpa: 26,
    dps: [1.0, 1.2, 1.15, 1.1, 1.25],
    revenue: 38,
    margins: [0.3, 0.28, 0.17],
    netDebtEbitda: 3.2,
    currentRatio: 1.3,
    assets: 100,
    revCagr: 0.3,
    epsCagr: 0.02,
    lossYears: 0,
    liquidity: 380,
  }),
  S({
    ticker: 'EGIE3',
    name: 'Engie Brasil ON',
    issuer: 'ENGIE BRASIL ENERGIA S.A.',
    sector: 'Utilidade pública',
    segment: 'Energia elétrica',
    price: 38.9,
    shares: 815,
    lpa: 3.0,
    vpa: 10.4,
    dps: [2.05, 2.1, 1.95, 2.3, 2.2],
    revenue: 11.5,
    margins: [0.55, 0.6, 0.48],
    netDebtEbitda: 2.9,
    currentRatio: 1.2,
    assets: 50,
    revCagr: 0.04,
    epsCagr: 0.03,
    lossYears: 0,
    liquidity: 110,
  }),
  S({
    ticker: 'CMIG4',
    name: 'Cemig PN',
    issuer: 'CIA ENERGETICA DE MINAS GERAIS - CEMIG',
    sector: 'Utilidade pública',
    segment: 'Energia elétrica',
    price: 11.2,
    shares: 2860,
    lpa: 2.1,
    vpa: 9.4,
    dps: [0.95, 1.3, 1.1, 1.52, 1.2],
    revenue: 39,
    margins: [0.25, 0.23, 0.19],
    netDebtEbitda: 1.1,
    currentRatio: 1.4,
    assets: 68,
    revCagr: 0.07,
    epsCagr: 0.2,
    lossYears: 0,
    liquidity: 190,
  }),
  S({
    ticker: 'PRIO3',
    name: 'PRIO ON',
    issuer: 'PRIO S.A.',
    sector: 'Petróleo, gás e biocombustíveis',
    segment: 'Exploração e produção',
    price: 41,
    shares: 850,
    lpa: 7.1,
    vpa: 26,
    dps: [0, 0, 0, 0, 0.62],
    revenue: 16,
    margins: [0.6, 0.62, 0.47],
    netDebtEbitda: 1.4,
    currentRatio: 1.9,
    assets: 60,
    revCagr: 0.55,
    epsCagr: 0.48,
    lossYears: 2,
    liquidity: 520,
  }),
  S({
    ticker: 'RADL3',
    name: 'Raia Drogasil ON',
    issuer: 'RAIA DROGASIL S.A.',
    sector: 'Saúde',
    segment: 'Farmácias e drogarias',
    price: 23.5,
    shares: 1720,
    lpa: 0.77,
    vpa: 4.9,
    dps: [0.35, 0.38, 0.4, 0.43, 0.45],
    revenue: 44,
    margins: [0.29, 0.08, 0.055],
    netDebtEbitda: 1.0,
    currentRatio: 1.4,
    assets: 30,
    revCagr: 0.16,
    epsCagr: 0.1,
    lossYears: 0,
    liquidity: 230,
  }),
  S({
    ticker: 'LREN3',
    name: 'Lojas Renner ON',
    issuer: 'LOJAS RENNER S.A.',
    sector: 'Consumo cíclico',
    segment: 'Vestuário',
    price: 15.7,
    shares: 960,
    lpa: 1.45,
    vpa: 12,
    dps: [0.42, 0.68, 0.6, 0.62, 0.66],
    revenue: 16,
    margins: [0.57, 0.18, 0.13],
    netDebtEbitda: 0.2,
    currentRatio: 2.0,
    assets: 24,
    revCagr: 0.05,
    epsCagr: 0.01,
    lossYears: 0,
    liquidity: 160,
  }),
  S({
    ticker: 'VIVT3',
    name: 'Telefônica Brasil ON',
    issuer: 'TELEFÔNICA BRASIL S.A.',
    sector: 'Comunicações',
    segment: 'Telecomunicações',
    price: 54.2,
    shares: 1640,
    lpa: 3.4,
    vpa: 43,
    dps: [2.7, 2.85, 2.95, 3.1, 3.3],
    revenue: 56,
    margins: [0.45, 0.41, 0.19],
    netDebtEbitda: 0.5,
    currentRatio: 1.0,
    assets: 125,
    revCagr: 0.05,
    epsCagr: 0.02,
    lossYears: 0,
    liquidity: 140,
  }),
  S({
    ticker: 'GGBR4',
    name: 'Gerdau PN',
    issuer: 'GERDAU S.A.',
    sector: 'Materiais básicos',
    segment: 'Siderurgia',
    price: 17.8,
    shares: 1980,
    lpa: 1.5,
    vpa: 26,
    dps: [3.1, 2.8, 1.3, 0.75, 0.7],
    revenue: 67,
    margins: [0.14, 0.15, 0.1],
    netDebtEbitda: 0.4,
    currentRatio: 2.8,
    assets: 85,
    revCagr: 0.07,
    epsCagr: -0.15,
    lossYears: 1,
    liquidity: 200,
  }),
  S({
    ticker: 'KLBN11',
    name: 'Klabin Unit',
    issuer: 'KLABIN S.A.',
    sector: 'Materiais básicos',
    segment: 'Papel e celulose',
    unit: 5,
    price: 20.4,
    shares: 1220,
    lpa: 1.9,
    vpa: 11.5,
    dps: [1.4, 1.85, 1.2, 0.95, 1.2],
    revenue: 20,
    margins: [0.38, 0.38, 0.27],
    netDebtEbitda: 3.5,
    currentRatio: 2.4,
    assets: 60,
    revCagr: 0.13,
    epsCagr: 0.08,
    lossYears: 0,
    liquidity: 120,
  }),
  S({
    ticker: 'SANB11',
    name: 'Santander Brasil Unit',
    issuer: 'BANCO SANTANDER (BRASIL) S.A.',
    sector: 'Financeiro',
    segment: 'Bancos',
    bank: true,
    unit: 2,
    price: 28.6,
    shares: 3740,
    lpa: 2.6,
    vpa: 24,
    dps: [2.0, 1.65, 1.4, 1.3, 1.45],
    revenue: 60,
    assets: 1100,
    revCagr: 0.04,
    epsCagr: -0.06,
    lossYears: 0,
    liquidity: 180,
  }),
  S({
    ticker: 'AAPL',
    name: 'Apple Inc.',
    issuer: 'Apple Inc.',
    sector: 'Tecnologia',
    segment: 'Hardware e dispositivos',
    us: true,
    shares: 14900,
    lpa: 6.75,
    vpa: 4.4,
    dps: [0.85, 0.9, 0.94, 0.98, 1.02],
    revenue: 405,
    margins: [0.46, 0.34, 0.31],
    netDebtEbitda: 0.1,
    currentRatio: 0.87,
    assets: 345,
    revCagr: 0.09,
    epsCagr: 0.14,
    lossYears: 0,
    liquidity: 11000,
  }),
  S({
    ticker: 'MSFT',
    name: 'Microsoft Corp.',
    issuer: 'Microsoft Corporation',
    sector: 'Tecnologia',
    segment: 'Software',
    us: true,
    shares: 7430,
    lpa: 13.4,
    vpa: 46,
    dps: [2.24, 2.48, 2.72, 3.0, 3.32],
    revenue: 282,
    margins: [0.69, 0.56, 0.45],
    netDebtEbitda: 0.1,
    currentRatio: 1.3,
    assets: 620,
    revCagr: 0.15,
    epsCagr: 0.18,
    lossYears: 0,
    liquidity: 9500,
  }),
  S({
    ticker: 'NVDA',
    name: 'NVIDIA Corp.',
    issuer: 'NVIDIA Corporation',
    sector: 'Tecnologia',
    segment: 'Semicondutores',
    us: true,
    shares: 24400,
    lpa: 3.45,
    vpa: 4.6,
    dps: [0.016, 0.016, 0.016, 0.017, 0.04],
    revenue: 165,
    margins: [0.75, 0.62, 0.6],
    netDebtEbitda: -0.4,
    currentRatio: 4.2,
    assets: 140,
    revCagr: 0.62,
    epsCagr: 0.85,
    lossYears: 0,
    liquidity: 32000,
  }),
  F({
    ticker: 'HGLG11',
    name: 'CSHG Logística FII',
    issuer: 'CSHG LOGÍSTICA FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Logística',
    shares: 33.7,
    nav: 160.5,
    income: 1.1,
    holders: 410_000,
    liquidity: 9.5,
  }),
  F({
    ticker: 'KNRI11',
    name: 'Kinea Renda Imobiliária FII',
    issuer: 'KINEA RENDA IMOBILIÁRIA FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Híbrido',
    shares: 28.2,
    nav: 161,
    income: 1.0,
    holders: 290_000,
    liquidity: 6.8,
  }),
  F({
    ticker: 'MXRF11',
    name: 'Maxi Renda FII',
    issuer: 'MAXI RENDA FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Papel (recebíveis)',
    shares: 390,
    nav: 9.71,
    income: 0.1,
    holders: 1_230_000,
    liquidity: 21,
  }),
  F({
    ticker: 'XPML11',
    name: 'XP Malls FII',
    issuer: 'XP MALLS FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Shoppings',
    shares: 56,
    nav: 112.8,
    income: 0.92,
    holders: 520_000,
    liquidity: 12.4,
  }),
  F({
    ticker: 'BTLG11',
    name: 'BTG Pactual Logística FII',
    issuer: 'BTG PACTUAL LOGÍSTICA FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Logística',
    price: 100.2,
    shares: 42,
    nav: 102.4,
    income: 0.78,
    holders: 175_000,
    liquidity: 7.1,
  }),
  F({
    ticker: 'VISC11',
    name: 'Vinci Shopping Centers FII',
    issuer: 'VINCI SHOPPING CENTERS FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Shoppings',
    price: 106.5,
    shares: 29,
    nav: 120.1,
    income: 0.8,
    holders: 330_000,
    liquidity: 5.4,
  }),
  F({
    ticker: 'HGRU11',
    name: 'CSHG Renda Urbana FII',
    issuer: 'CSHG RENDA URBANA FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Renda urbana',
    price: 121,
    shares: 15,
    nav: 129.6,
    income: 0.92,
    holders: 170_000,
    liquidity: 3.1,
  }),
  F({
    ticker: 'XPLG11',
    name: 'XP Log FII',
    issuer: 'XP LOG FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Logística',
    price: 98,
    shares: 28,
    nav: 113,
    income: 0.78,
    holders: 280_000,
    liquidity: 4.2,
  }),
  F({
    ticker: 'KNCR11',
    name: 'Kinea Rendimentos Imobiliários FII',
    issuer: 'KINEA RENDIMENTOS IMOBILIÁRIOS FUNDO DE INVESTIMENTO IMOBILIÁRIO',
    segment: 'Papel (recebíveis)',
    price: 103.5,
    shares: 64,
    nav: 102,
    income: 1.12,
    holders: 420_000,
    liquidity: 14,
  }),
];

const ASSET_ID: Record<string, string> = {
  PETR4: 'ast-petr4',
  ITUB4: 'ast-itub4',
  WEGE3: 'ast-wege3',
  BBAS3: 'ast-bbas3',
  MGLU3: 'ast-mglu3',
  VALE3: 'ast-vale3',
  TAEE11: 'ast-taee11',
  HGLG11: 'ast-hglg11',
  KNRI11: 'ast-knri11',
  MXRF11: 'ast-mxrf11',
  XPML11: 'ast-xpml11',
  AAPL: 'ast-aapl',
  MSFT: 'ast-msft',
  NVDA: 'ast-nvda',
};

// ── Estado ───────────────────────────────────────────────────────

const DEFAULT_UA = 'Treasury/0.1 contato@exemplo';
const SEC_HINT =
  'A SEC exige identificação com um e-mail de contato: troque "contato@exemplo" pelo seu e-mail em Ajustes › Fontes de dados (ex.: "Treasury/0.1 seu.nome@email.com").';

interface State {
  settings: ResearchSettings;
  /** Sem coleta: universo vazio (estado de uma instalação nova). */
  empty: boolean;
  lastSync: string | null;
  lastReport: ResearchSyncReport | null;
  syncs: number;
}

function initial(): State {
  return {
    settings: { sec_user_agent: DEFAULT_UA, bazin_rate: 0.06, bazin_years: 5 },
    empty: false,
    lastSync: `${addDays(mockToday(), -1)}T09:12:00Z`,
    lastReport: null,
    syncs: 1,
  };
}

let state = initial();
onMockReset(() => {
  state = initial();
});

/** Testes: começa sem nenhuma coleta (universo vazio, sem fundamentos). */
export function mockResearchEmpty(): void {
  state = { ...initial(), empty: true, lastSync: null, syncs: 0 };
}

// ── Cálculo dos indicadores ──────────────────────────────────────

const REF_DATE = '2026-06-30';
const br = (iso: string) => iso.split('-').reverse().join('/');

function docId(ticker: string, salt = 0): number {
  return 140_000 + ((hashSeed(`${ticker}:${salt}`) >>> 0) % 9000);
}

interface Entry {
  spec: Spec;
  ticker: string;
  assetId: string | null;
  subject: Subject;
  price: number;
  priceOn: PriceOn;
  indicators: Indicator[];
  metrics: Record<string, number>;
  /** Score em lote: faixa e red flags (campos de texto do screener). */
  scoreBand: string | null;
  redFlags: string | null;
}

function specClass(s: Spec): AssetClass {
  return s.kind === 'fii' ? 'fii' : s.us ? 'stock_us' : 'stock_br';
}

function priceOf(s: Spec): { price: number; on: PriceOn } {
  const id = ASSET_ID[s.ticker];
  const today = mockToday();
  const hit = id && hasAsset(id) ? closeOn(id, today) : null;
  if (hit) return { price: hit.close, on: hit.price };
  const p = s.price ?? 10;
  const d = addDays(today, -1);
  return {
    price: p,
    on: { close: String(p), date: d, source: 'b3_cotahist', stale: false, intraday: null },
  };
}

function subjectOf(s: Spec, assetId: string | null): Subject {
  return {
    asset_id: assetId,
    ticker: s.ticker,
    name: s.name,
    class: specClass(s),
    currency: s.kind === 'stock' && s.us ? 'USD' : 'BRL',
    issuer_id: `iss-${s.ticker.slice(0, 4).toLowerCase()}`,
    issuer_name: s.issuer,
    issuer_code:
      s.kind === 'stock' && s.us
        ? `cik:${String(320_000 + (hashSeed(s.ticker) % 900_000)).padStart(10, '0')}`
        : `${String(hashSeed(s.issuer) % 99).padStart(2, '0')}.${String(hashSeed(s.ticker) % 999).padStart(3, '0')}.${String((hashSeed(s.name) >>> 3) % 999).padStart(3, '0')}/0001-${String(hashSeed(s.ticker) % 99).padStart(2, '0')}`,
    sector: s.kind === 'fii' ? 'Fundos imobiliários' : s.sector,
    segment: s.segment,
    mandate:
      s.kind === 'fii'
        ? s.segment.startsWith('Papel')
          ? 'Títulos e valores mobiliários'
          : 'Renda'
        : null,
    layout: s.kind === 'fii' ? 'fund' : s.bank ? 'bank' : 'industrial',
    share_kind:
      s.kind === 'fii'
        ? 'CTF'
        : s.unit
          ? 'UNIT'
          : s.ticker.endsWith('3')
            ? 'ON'
            : s.us
              ? null
              : 'PN',
  };
}

/** Dividendos por ação nos últimos 10 exercícios (2016–2025). */
function dpsByYear(s: Spec): Map<number, number> {
  const out = new Map<number, number>();
  if (s.kind === 'fii') {
    for (let y = 2016; y <= 2025; y += 1) {
      const wobble = 1 + (((y * 7) % 5) - 2) * 0.03;
      out.set(y, Number((s.income * 12 * wobble * (1 - (2025 - y) * 0.015)).toFixed(4)));
    }
    return out;
  }
  for (let y = 2016; y <= 2020; y += 1) {
    out.set(y, Number((s.dps[0] * 0.9 ** (2021 - y)).toFixed(4)));
  }
  s.dps.forEach((v, i) => out.set(2021 + i, v));
  return out;
}

function bazinOf(s: Spec, years: number): { avg: number; byYear: [number, number][] } {
  const all = [...dpsByYear(s).entries()].filter(([y]) => y > 2025 - years);
  const avg = all.reduce((t, [, v]) => t + v, 0) / Math.max(1, all.length);
  return { avg, byYear: all };
}

const BANK_NA = 'não se aplica a bancos e seguradoras (leiaute de instituição financeira)';
const BASEL_NA =
  'o Índice de Basileia não está nos arquivos da CVM (fica nas notas explicativas e no IF.data do Banco Central)';

function compute(s: Spec, settings: ResearchSettings): Entry {
  const reg = ASSET_ID[s.ticker];
  const assetId = reg && hasAsset(reg) ? reg : null;
  const subject = subjectOf(s, assetId);
  const { price, on } = priceOf(s);
  const cur = subject.currency;
  const priceSrc =
    on.source === 'b3_cotahist'
      ? `B3 COTAHIST ${br(on.date)}`
      : `Yahoo Finance · fechamento de ${br(on.date)}`;
  const flows =
    s.kind === 'stock' && s.us
      ? `SEC 10-K 2025 + 10-Q Q2 2026 (${subject.issuer_code})`
      : `DFP 2025 + ITR 2T26 (doc. CVM ${docId(s.ticker)})`;
  const balance =
    s.kind === 'stock' && s.us
      ? `SEC 10-Q Q2 2026 (${subject.issuer_code})`
      : `ITR 2T26 (doc. CVM ${docId(s.ticker)})`;
  const ltm = `12 meses até ${br(REF_DATE)}`;
  const bal = `saldo em ${br(REF_DATE)}`;
  const list: Indicator[] = [];
  const push = (
    key: string,
    value: number | null,
    opts: {
      period?: string;
      source?: string;
      reason?: string;
      estimated?: boolean;
      exact?: string;
    } = {},
  ) => {
    const d = DEF.get(key)!;
    const ok = value != null && Number.isFinite(value);
    list.push({
      key,
      label: d.label,
      group: d.group,
      unit: d.unit,
      value: ok ? Number(value.toPrecision(12)) : null,
      exact:
        ok && d.unit === 'money'
          ? (opts.exact ??
            value.toFixed(
              d.key === 'market_cap' || d.key === 'ev' || d.key === 'liquidity' ? 2 : 4,
            ))
          : null,
      currency: d.unit === 'money' ? cur : null,
      formula: d.formula,
      period: ok ? (opts.period ?? null) : null,
      period_end: ok && opts.period ? REF_DATE : null,
      source: ok ? (opts.source ?? null) : null,
      estimated: Boolean(opts.estimated),
      unavailable_reason: ok ? null : (opts.reason ?? 'indisponível'),
      check: ok ? (opts.estimated ? 'warning' : 'verified') : 'unchecked',
      check_notes:
        ok && opts.estimated && key === 'dy'
          ? [
              'sem proventos oficiais da B3 para o papel: estimativa pelos dividendos e JCP pagos no fluxo de caixa (fica fora dos rankings)',
            ]
          : [],
    });
  };
  const today = mockToday();
  push('price', price, {
    period: `fechamento de ${br(on.date)}`,
    source: priceSrc,
    exact: on.close,
  });
  const mc = price * s.shares * 1e6;
  push('market_cap', mc, { source: `${priceSrc}; ${balance}` });

  if (s.kind === 'fii') {
    push('liquidity', s.liquidity * 1e6, { period: '21 pregões', source: 'B3 COTAHIST' });
    const inf = `CVM informe mensal de FII 09/2026`;
    push('pvp', price / s.nav, { period: 'informe de 09/2026', source: `${priceSrc}; ${inf}` });
    const dy = (s.income * 12) / price;
    push('dy', dy, { period: `12 meses até ${br(today)}`, source: 'Yahoo Finance (proventos)' });
    push('last_dividend', s.income, {
      period: 'pagamento de 15/10/2026',
      source: 'Yahoo Finance (proventos)',
    });
    push('nav_per_share', s.nav, { period: 'informe de 09/2026', source: inf });
    push('holders', s.holders, { period: 'informe de 09/2026', source: inf });
    push('vacancy', null, {
      reason: 'a vacância só está no informe trimestral de FII, que ainda não é coletado',
    });
  } else {
    const ni = (s.lpa * s.shares * 1e6) / (s.unit ?? 1);
    const eq = (s.vpa * s.shares * 1e6) / (s.unit ?? 1);
    const rev = s.revenue * 1e9;
    const assets = s.assets * 1e9;
    const bank = Boolean(s.bank);
    const [gm, em, bm] = s.margins ?? [0, 0, 0];
    const ebitda = rev * em;
    const ebit = rev * bm;
    const netDebt = (s.netDebtEbitda ?? 0) * ebitda;
    const ev = mc + netDebt;
    if (bank) push('ev', null, { reason: BANK_NA });
    else push('ev', ev, { source: `${priceSrc}; ${balance}` });
    push('liquidity', s.liquidity * 1e6, {
      period: '21 pregões',
      source: s.us ? 'Yahoo Finance' : 'B3 COTAHIST',
    });
    push(
      'pl',
      ni > 0 ? mc / ni : null,
      ni > 0
        ? { period: ltm, source: `${priceSrc}; ${flows}` }
        : { reason: 'lucro negativo nos últimos 12 meses' },
    );
    push('pvp', mc / eq, { period: bal, source: `${priceSrc}; ${balance}` });
    for (const [k, v] of [
      ['ev_ebitda', ev / ebitda],
      ['ev_ebit', ev / ebit],
    ] as const) {
      if (bank) push(k, null, { reason: BANK_NA });
      else push(k, v, { period: ltm, source: `${priceSrc}; ${flows}` });
    }
    push('psr', mc / rev, { period: ltm, source: `${priceSrc}; ${flows}` });
    push('p_ativo', mc / assets, { period: bal, source: `${priceSrc}; ${balance}` });
    const dps12 = s.dps[4];
    push('dy', dps12 / price, {
      period: `12 meses até ${br(today)}`,
      source: 'Yahoo Finance (proventos)',
    });
    push(
      'payout',
      s.lpa > 0 ? dps12 / s.lpa : null,
      s.lpa > 0
        ? { period: ltm, source: flows }
        : { reason: 'lucro negativo nos últimos 12 meses' },
    );
    push('roe', ni / eq, { period: ltm, source: flows });
    if (bank) push('roic', null, { reason: BANK_NA });
    else push('roic', (ebit * 0.66) / (eq + netDebt), { period: ltm, source: flows });
    push('roa', ni / assets, { period: ltm, source: flows });
    for (const [k, v] of [
      ['gross_margin', gm],
      ['ebitda_margin', em],
      ['ebit_margin', bm],
      ['net_margin', ni / rev],
    ] as const) {
      if (bank) push(k, null, { reason: BANK_NA });
      else push(k, v, { period: ltm, source: flows });
    }
    if (bank) {
      push('net_debt_equity', null, { reason: BANK_NA });
      push('net_debt_ebitda', null, { reason: BANK_NA });
      push('current_ratio', null, { reason: BANK_NA });
      for (const k of [
        'p_ebit',
        'p_working_capital',
        'p_ncav',
        'ebit_assets',
        'asset_turnover',
        'gross_debt_equity',
      ])
        push(k, null, { reason: BANK_NA });
    } else {
      push('net_debt_equity', netDebt / eq, { period: bal, source: balance });
      push('net_debt_ebitda', s.netDebtEbitda ?? null, { period: ltm, source: flows });
      push('current_ratio', s.currentRatio ?? null, { period: bal, source: balance });
      // Saldos aproximados do exemplo: circulante = 40% do ativo, passivo total = 55%.
      const ca = assets * 0.4;
      const cl = s.currentRatio ? ca / s.currentRatio : null;
      const liab = assets * 0.55;
      push('p_ebit', ebit > 0 ? mc / ebit : null, {
        period: ltm,
        source: `${priceSrc}; ${flows}`,
        reason: 'EBIT negativo',
      });
      push('p_working_capital', cl != null && ca > cl ? mc / (ca - cl) : null, {
        period: bal,
        source: `${priceSrc}; ${balance}`,
        reason: 'capital de giro negativo ou zero',
      });
      push('p_ncav', ca > liab ? mc / (ca - liab) : null, {
        period: bal,
        source: `${priceSrc}; ${balance}`,
        reason: 'ativo circulante menor que o passivo total',
      });
      push('ebit_assets', ebit / assets, { period: ltm, source: flows });
      push('asset_turnover', rev / assets, { period: ltm, source: flows });
      push('gross_debt_equity', (netDebt + assets * 0.05) / eq, { period: bal, source: balance });
    }
    const cagrPeriod = 'exercícios de 2020 a 2025';
    push('revenue_cagr_5y', s.revCagr, { period: cagrPeriod, source: 'DFP 2020 e DFP 2025' });
    push(
      'earnings_cagr_5y',
      s.epsCagr,
      s.epsCagr == null
        ? { reason: 'lucro negativo no início ou no fim do período' }
        : { period: cagrPeriod, source: 'DFP 2020 e DFP 2025' },
    );
    push('loss_years_10y', s.lossYears, {
      period: 'exercícios de 2016 a 2025',
      source: 'DFP 2016–2025',
    });
    push('lpa', s.lpa, { period: ltm, source: flows });
    push('vpa', s.vpa, { period: bal, source: balance });
    if (bank) push('basel', null, { reason: BASEL_NA });
    const graham = s.lpa > 0 && s.vpa > 0 ? Math.sqrt(22.5 * s.lpa * s.vpa) : null;
    push(
      'graham',
      graham,
      graham == null
        ? { reason: 'LPA negativo ou zero (prejuízo nos últimos 12 meses)' }
        : { source: flows, exact: graham.toFixed(2) },
    );
    push(
      'graham_upside',
      graham == null ? null : graham / price - 1,
      graham == null
        ? { reason: 'LPA negativo ou zero (prejuízo nos últimos 12 meses)' }
        : { source: flows },
    );
  }
  const bz = bazinOf(s, settings.bazin_years);
  const bazin = bz.avg > 0 ? bz.avg / settings.bazin_rate : null;
  const bzReason = { reason: 'sem dividendos no período' };
  push(
    'bazin',
    bazin,
    bazin == null ? bzReason : { source: 'Yahoo Finance (proventos)', exact: bazin.toFixed(2) },
  );
  push(
    'bazin_upside',
    bazin == null ? null : bazin / price - 1,
    bazin == null ? bzReason : { source: 'Yahoo Finance (proventos)' },
  );

  if (s.kind === 'stock' && !s.bank) {
    const ev = metricOf(list, 'ev');
    const pl = metricOf(list, 'pl');
    const evEbit = metricOf(list, 'ev_ebit');
    push('earnings_yield', evEbit && evEbit > 0 ? 1 / evEbit : null, {
      period: ltm,
      source: flows,
    });
    push('magic_roc', evEbit && ev ? ev / evEbit / (ev * 0.55) : null, {
      period: ltm,
      source: flows,
    });
    push('fcf_yield', s.lpa > 0 ? (s.lpa * 0.7) / price : null, { period: ltm, source: flows });
    push(
      'peg',
      pl && s.epsCagr && s.epsCagr > 0 ? pl / (s.epsCagr * 100) : null,
      pl && s.epsCagr && s.epsCagr > 0
        ? { source: 'P/L e crescimento do lucro em 5 anos' }
        : { reason: 'P/L ou crescimento do lucro negativo: o PEG não se aplica' },
    );
    push('eps_growth_10y', s.epsCagr == null ? null : (1 + s.epsCagr) ** 9 - 1, {
      source: 'DFP 2016–2025',
    });
  }
  if (s.kind === 'stock' && !s.us) {
    const years = s.dps.filter((v) => v > 0).length;
    push('dividend_years', years, { source: 'Yahoo Finance (proventos)' });
    push(
      'dividend_cagr_5y',
      s.dps[0] > 0 && s.dps[4] > 0 ? (s.dps[4] / s.dps[0]) ** (1 / 4) - 1 : null,
      { source: 'Yahoo Finance (proventos)' },
    );
  }

  const metrics: Record<string, number> = {};
  for (const i of list) if (i.value != null) metrics[i.key] = i.value;
  const ms = marketScore({ ticker: s.ticker, spec: s });
  if (ms) Object.assign(metrics, ms.metrics);
  return {
    spec: s,
    ticker: s.ticker,
    assetId,
    subject,
    price,
    priceOn: on,
    indicators: list,
    metrics,
    scoreBand: ms?.band ?? null,
    redFlags: ms?.flags || null,
  };
}

function metricOf(list: Indicator[], key: string): number | null {
  return list.find((i) => i.key === key)?.value ?? null;
}

function universe(): Entry[] {
  if (state.empty) return [];
  return SPECS.map((s) => compute(s, state.settings));
}

function entryOf(a: Record<string, unknown>): Entry {
  const assetId = typeof a.asset_id === 'string' && a.asset_id ? a.asset_id : null;
  const ticker = typeof a.ticker === 'string' && a.ticker ? a.ticker.trim().toUpperCase() : null;
  if (!assetId && !ticker) invalid('informe asset_id ou ticker');
  const spec = SPECS.find((s) => (assetId ? ASSET_ID[s.ticker] === assetId : s.ticker === ticker));
  if (assetId && !hasAsset(assetId)) notFound(`ativo ${assetId}`);
  if (!spec || state.empty) {
    notFound(`${ticker ?? assetId ?? ''} no universo baixado (rode a coleta da análise de ativos)`);
  }
  return compute(spec, state.settings);
}

function indicatorsView(e: Entry): IndicatorsView {
  const notes: string[] = [];
  if (e.spec.kind === 'stock' && e.spec.bank)
    notes.push(
      'Banco: sem EBITDA, valor da firma, dívida líquida, margens nem liquidez corrente; o Índice de Basileia não está nos arquivos da CVM.',
    );
  if (e.spec.kind === 'stock' && e.spec.unit)
    notes.push(`Indicadores por ação multiplicados pelas ${e.spec.unit} ações de cada unit.`);
  if (e.spec.kind === 'stock' && e.spec.us)
    notes.push('Fundamentos da SEC EDGAR (XBRL companyfacts), em dólares.');
  notes.push('Dados ilustrativos do modo de demonstração.');
  return {
    subject: e.subject,
    price: e.priceOn,
    as_of: mockToday(),
    reference_date: e.spec.kind === 'fii' ? null : REF_DATE,
    indicators: e.indicators,
    notes,
  };
}

function fairPrice(e: Entry, rate: number, years: number): FairPriceView {
  const s = e.spec;
  const cur = e.subject.currency;
  const flows = e.indicators.find((i) => i.key === 'lpa')?.source ?? null;
  const balance = e.indicators.find((i) => i.key === 'vpa')?.source ?? null;
  const graham: FairPriceMethod = {
    method: 'graham',
    label: 'Graham',
    value: null,
    currency: cur,
    upside: null,
    formula: '√(22,5 × LPA × VPA); 22,5 = P/L máximo de 15 × P/VP máximo de 1,5',
    assumptions: [],
    unavailable_reason: null,
  };
  if (s.kind === 'fii') graham.unavailable_reason = 'não se aplica a FIIs';
  else {
    graham.assumptions = [
      { label: 'LPA (12 meses)', value: s.lpa.toFixed(4), source: flows },
      { label: 'VPA', value: s.vpa.toFixed(4), source: balance },
      { label: 'Ações em circulação', value: String(Math.round(s.shares * 1e6)), source: balance },
    ];
    if (s.lpa <= 0)
      graham.unavailable_reason = 'LPA negativo ou zero (prejuízo nos últimos 12 meses)';
    else {
      const v = Math.sqrt(22.5 * s.lpa * s.vpa);
      graham.value = v.toFixed(2);
      graham.upside = Number(v.toFixed(2)) / e.price - 1;
    }
  }
  const bz = bazinOf(s, years);
  const pct = Number((rate * 100).toPrecision(10));
  const bazin: FairPriceMethod = {
    method: 'bazin',
    label: 'Bazin',
    value: null,
    currency: cur,
    upside: null,
    formula: 'Média dos dividendos por ação dos últimos N anos ÷ taxa mínima de retorno',
    assumptions: [
      { label: 'Taxa mínima', value: `${pct}%`, source: null },
      { label: 'Anos', value: String(years), source: null },
      ...bz.byYear.map(([y, v]) => ({
        label: `Dividendos por ação em ${y}`,
        value: v.toFixed(4),
        source: 'Yahoo Finance (proventos)',
      })),
      { label: 'Média anual', value: bz.avg.toFixed(4), source: null },
    ],
    unavailable_reason: null,
  };
  if (bz.avg <= 0) bazin.unavailable_reason = 'sem dividendos no período';
  else {
    const v = bz.avg / rate;
    bazin.value = v.toFixed(2);
    bazin.upside = Number(v.toFixed(2)) / e.price - 1;
  }
  return { subject: e.subject, price: e.priceOn, graham, bazin, disclaimer: DISCLAIMER };
}

function history(e: Entry, years: number): HistoryView {
  const s = e.spec;
  const cur = e.subject.currency;
  const c = (v: number) => Math.round(v * 100);
  const rows: YearRow[] = [];
  const quarters: QuarterRow[] = [];
  const months: MonthRow[] = [];
  const dps = dpsByYear(s);
  if (s.kind === 'stock') {
    const unit = s.unit ?? 1;
    const ltmRev = s.revenue * 1e9;
    const ltmNi = (s.lpa * s.shares * 1e6) / unit;
    const margin = ltmRev ? ltmNi / ltmRev : 0;
    const em = s.margins?.[1] ?? null;
    for (let y = 2025 - years + 1; y <= 2025; y += 1) {
      const k = 2025 - y;
      const wobble = 1 + (((y * 13) % 7) - 3) * 0.02;
      const rev = (ltmRev / (1 + s.revCagr) ** k) * wobble;
      const lossYear =
        s.lossYears > 0 && [2016, 2020, 2023, 2024].slice(0, s.lossYears).includes(y);
      const nm = lossYear
        ? -Math.abs(margin || 0.03) * 0.6
        : (margin || 0.02) * (1 + (((y * 7) % 5) - 2) * 0.08);
      const emY = em == null ? null : em * (1 + (((y * 3) % 5) - 2) * 0.05);
      const ni = rev * nm;
      const eq = (s.vpa * s.shares * 1e6) / unit / (1 + Math.max(0.02, s.revCagr * 0.5)) ** k;
      const d = dps.get(y) ?? 0;
      const shares = (s.shares * 1e6) / unit;
      const id = ASSET_ID[s.ticker];
      const pr = id && hasAsset(id) ? closeOn(id, `${y}-12-30`) : null;
      const prOk = pr && pr.price.date >= `${y}-12-15` ? pr.close : null;
      rows.push({
        year: y,
        period_end: `${y}-12-31`,
        revenue: c(rev),
        net_income: c(ni),
        ebitda: emY == null || s.bank ? null : c(rev * emY),
        dividends_declared: c(d * shares),
        dividends_paid: c(d * shares * 0.97),
        equity: c(eq),
        eps: ((ni / shares) * unit).toFixed(4),
        dps: d.toFixed(4),
        net_margin: rev ? ni / rev : null,
        roe: eq > 0 ? ni / eq : null,
        price: prOk == null ? null : prOk.toFixed(2),
        dividend_yield: prOk ? d / prOk : null,
        document: s.us ? `SEC 10-K ${y}` : `DFP ${y} (doc. CVM ${docId(s.ticker, y)})`,
      });
    }
    const qs = ['3T24', '4T24', '1T25', '2T25', '3T25', '4T25', '1T26', '2T26'];
    const ends = [
      '2024-09-30',
      '2024-12-31',
      '2025-03-31',
      '2025-06-30',
      '2025-09-30',
      '2025-12-31',
      '2026-03-31',
      '2026-06-30',
    ];
    qs.forEach((label, i) => {
      const f = 0.25 * (1 + (i - 7) * 0.012) * (1 + (((i * 3) % 4) - 1.5) * 0.02);
      const end = ends[i]!;
      quarters.push({
        label,
        period_start: `${end.slice(0, 5)}${String(Number(end.slice(5, 7)) - 2).padStart(2, '0')}-01`,
        period_end: end,
        revenue: c(ltmRev * f),
        net_income: c(ltmNi * f),
        ebitda: em == null || s.bank ? null : c(ltmRev * f * em),
      });
    });
  } else {
    for (let y = 2025 - years + 1; y <= 2025; y += 1) {
      const d = dps.get(y) ?? 0;
      const shares = s.shares * 1e6;
      const nav = s.nav * (1 - (2025 - y) * 0.01);
      rows.push({
        year: y,
        period_end: `${y}-12-31`,
        revenue: null,
        net_income: c(d * shares * 1.02),
        ebitda: null,
        dividends_declared: c(d * shares),
        dividends_paid: c(d * shares),
        equity: c(nav * shares),
        eps: null,
        dps: d.toFixed(4),
        net_margin: null,
        roe: null,
        price: null,
        dividend_yield: null,
        document: `CVM informe mensal de FII 12/${y}`,
      });
    }
    let m = addMonths(`${mockToday().slice(0, 7)}-01`, -24);
    for (let i = 0; i < 24; i += 1) {
      const month = m.slice(0, 7);
      const wobble = 1 + (((i * 7) % 5) - 2) * 0.02;
      const nav = s.nav * (1 - (23 - i) * 0.0015);
      const inc = s.income * wobble;
      months.push({
        month,
        nav_per_share: nav.toFixed(4),
        income_per_share: inc.toFixed(4),
        dy_month: inc / nav,
        holders: Math.round(s.holders * (1 - (23 - i) * 0.006)),
      });
      m = addMonths(m, 1);
    }
  }
  const notes: string[] = [];
  if (s.kind === 'stock' && s.unit) notes.push(`Valores por papel da unit (${s.unit} ações).`);
  const units_note = 'Valores monetários em centavos da moeda do ativo.';
  return { subject: e.subject, currency: cur, years: rows, quarters, months, notes, units_note };
}

// ── Documentos ───────────────────────────────────────────────────

const DOC_TEMPLATES: [category: string, docType: string | null, subject: string][] = [
  ['Fato Relevante', null, 'Aprovação de distribuição de juros sobre capital próprio'],
  [
    'Comunicado ao Mercado',
    'Apresentações a analistas/agentes do mercado',
    'Apresentação de resultados do 2T26',
  ],
  [
    'Aviso aos Acionistas',
    'Outros avisos',
    'Pagamento de proventos <b>R$ 0,21</b> por ação & data-com',
  ],
  ['Fato Relevante', null, 'Plano estratégico 2027–2031'],
  [
    'Comunicado ao Mercado',
    'Esclarecimentos sobre questionamentos da CVM/B3',
    'Esclarecimento sobre notícia veiculada na mídia',
  ],
];

/** Categorias guardadas pela coleta do IPE (como `cvm::IPE_CATEGORIES`). */
const IPE_KEPT = [
  'Fato Relevante',
  'Comunicado ao Mercado',
  'Aviso aos Acionistas',
  'Calendário de Eventos Corporativos',
  'Dados Econômico-Financeiros',
  'Assembleia',
  'Reunião da Administração',
  'Comunicação sobre Transação entre Partes Relacionadas',
  'Informações de Companhias em Recuperação Judicial ou Extrajudicial',
];

/** Diagnóstico dos documentos de um ativo, como `views::documents_status`. */
function documentsStatus(a: Record<string, unknown>): DocumentsStatus {
  const today = mockToday();
  const assetId = typeof a.asset_id === 'string' && a.asset_id ? a.asset_id : null;
  const entry = ((): Entry | null => {
    try {
      return entryOf(a);
    } catch {
      return null;
    }
  })();
  const cls =
    entry?.subject.class ?? (assetId && hasAsset(assetId) ? getAsset(assetId).class : null);
  const covered = cls == null || cls === 'stock_br' || cls === 'bdr';
  const issuer = entry?.subject.issuer_id ?? null;
  const docs =
    issuer && covered
      ? documents({ ...a, from: `${Number(today.slice(0, 4)) - 1}-01-01`, limit: 500 })
      : [];
  const by = new Map<string, { count: number; latest: string }>();
  for (const d of docs) {
    const c = by.get(d.category) ?? { count: 0, latest: d.delivered_at };
    c.count += 1;
    if (d.delivered_at > c.latest) c.latest = d.delivered_at;
    by.set(d.category, c);
  }
  const lastSync = state.empty ? null : `${today}T06:00:00Z`;
  return {
    reason: !covered
      ? 'not_covered'
      : !issuer
        ? 'no_issuer'
        : docs.length
          ? 'ok'
          : lastSync
            ? 'quiet'
            : 'not_collected',
    asset_id: assetId,
    class: cls,
    issuer_id: issuer,
    issuer_name: entry?.subject.issuer_name ?? null,
    source: 'cvm_ipe',
    kept_categories: IPE_KEPT,
    collected_since: `${Number(today.slice(0, 4)) - 1}-01-01`,
    last_sync_at: lastSync,
    stored: docs.length,
    latest: docs[0]?.delivered_at ?? null,
    by_category: [...by.entries()].map(([category, v]) => ({ category, ...v })),
  };
}

function documents(a: Record<string, unknown>): CorporateDocument[] {
  if (state.empty) return [];
  const today = mockToday();
  const from = typeof a.from === 'string' ? a.from : addDays(today, -90);
  const to = typeof a.to === 'string' ? a.to : today;
  let entries: Entry[];
  if (a.asset_id || a.ticker) entries = [entryOf(a)];
  else {
    const mine = mineIds();
    entries = universe().filter((e) => e.assetId && mine.has(e.assetId));
  }
  const cats = Array.isArray(a.categories) ? (a.categories as string[]).map(norm) : [];
  const limit = Math.min(500, Math.max(1, typeof a.limit === 'number' ? a.limit : 50));
  const out: CorporateDocument[] = [];
  for (const e of entries) {
    // IPE: só companhias abertas na CVM (FIIs publicam no Fundos.NET, ainda não coletado).
    if ((e.spec.kind === 'stock' && e.spec.us) || e.spec.kind === 'fii') continue;
    const n = 2 + (hashSeed(e.ticker) % 3);
    for (let i = 0; i < n; i += 1) {
      const t = DOC_TEMPLATES[(hashSeed(e.ticker) + i) % DOC_TEMPLATES.length]!;
      const delivered = addDays(today, -(3 + i * 17 + (hashSeed(`${e.ticker}${i}`) % 9)));
      const id = docId(e.ticker, 100 + i);
      out.push({
        id: `ipe-${id}`,
        issuer_id: e.subject.issuer_id!,
        company_name: e.subject.issuer_name ?? e.subject.name,
        tickers: [e.ticker],
        asset_ids: e.assetId ? [e.assetId] : [],
        category: t[0],
        doc_type: t[1],
        species: null,
        subject: t[2],
        reference_date: delivered,
        delivered_at: delivered,
        link: `https://www.rad.cvm.gov.br/ENETCONSULTA/frmDownloadDocumento.aspx?CodigoInstituicao=1&NumeroSequencialDocumento=${id}`,
        summary: null,
        source: 'cvm_ipe',
      });
    }
  }
  return out
    .filter((d) => d.delivered_at >= from && d.delivered_at <= to)
    .filter((d) => !cats.length || cats.includes(norm(d.category)))
    .sort((x, y) => y.delivered_at.localeCompare(x.delivered_at) || x.id.localeCompare(y.id))
    .slice(0, limit);
}

// ── Carteira e watchlist (pelo mock da carteira) ────────────────

function idsFrom(cmd: string, pick: (row: Record<string, unknown>) => unknown): Set<string> {
  const h = extraMockHandler(cmd);
  if (!h) return new Set();
  const list = h({ limit: 5000 }) as Record<string, unknown>[];
  return new Set(list.map(pick).filter((x): x is string => typeof x === 'string'));
}

const heldIds = () => idsFrom('portfolio.list_trades', (t) => t.asset_id);
const watchedIds = () =>
  idsFrom('portfolio.list_watchlist', (w) => (w.asset as { id?: string } | undefined)?.id);
const mineIds = () => new Set([...heldIds(), ...watchedIds()]);

// ── Tela Mercado (como `research::market`) ───────────────────────

/** Ativos criados pelo Observar (somem ao deixar de observar sem operação). */
let observedIds = new Set<string>();
onMockReset(() => {
  observedIds = new Set();
});

function medianOf(v: number[]): number | null {
  const xs = v.filter(Number.isFinite).sort((a, b) => a - b);
  if (!xs.length) return null;
  const n = xs.length;
  return n % 2 ? xs[(n - 1) / 2]! : (xs[n / 2 - 1]! + xs[n / 2]!) / 2;
}

function sectorStats(sector: string, es: Entry[]): MarketSector {
  const m = (k: string, positive: boolean) =>
    medianOf(
      es.map((e) => e.metrics[k]).filter((v): v is number => v != null && (!positive || v > 0)),
    );
  return {
    sector,
    count: es.length,
    scored: es.filter((e) => e.metrics.score != null).length,
    healthy: es.filter((e) => e.scoreBand === 'robusto' || e.scoreBand === 'saudavel').length,
    median_pl: m('pl', true),
    median_pvp: m('pvp', true),
    median_dy: m('dy', false),
    median_score: m('score', false),
  };
}

function groupStats(es: Entry[], key: (e: Entry) => string | null): MarketSector[] {
  const by = new Map<string, Entry[]>();
  for (const e of es) {
    const k = key(e) ?? '—';
    by.set(k, [...(by.get(k) ?? []), e]);
  }
  return [...by.entries()]
    .map(([k, v]) => sectorStats(k, v))
    .sort((a, b) => b.count - a.count || a.sector.localeCompare(b.sector));
}

function marketOverview(): MarketOverview {
  const all = universe();
  const stocks = all.filter((e) => e.subject.class === 'stock_br');
  const fiis = all.filter((e) => e.subject.class === 'fii');
  return {
    as_of: mockToday(),
    stocks: stocks.length,
    fiis: fiis.length,
    scored: all.filter((e) => e.metrics.score != null).length,
    scores_as_of: all.length ? mockToday() : null,
    last_sync_at: state.empty ? null : `${mockToday()}T06:00:00Z`,
    stock_sectors: groupStats(stocks, (e) => e.subject.sector),
    fii_segments: groupStats(fiis, (e) => e.subject.segment),
  };
}

function marketMap(): MarketMapPoint[] {
  const held = heldIds();
  const watched = watchedIds();
  return universe().flatMap((e) => {
    const h = e.metrics.health;
    const a = e.metrics.attractiveness;
    if (h == null || a == null) return [];
    return [
      {
        ticker: e.ticker,
        name: e.subject.name,
        class: e.subject.class,
        sector: e.subject.sector,
        asset_id: e.assetId,
        health: h,
        attractiveness: a,
        score: e.metrics.score ?? null,
        band: e.scoreBand,
        market_cap: e.metrics.market_cap ?? null,
        held: Boolean(e.assetId && held.has(e.assetId)),
        watched: Boolean(e.assetId && watched.has(e.assetId)),
      },
    ];
  });
}

// ── Métodos (como `research::methods`, receitas de methodology/screens/v0.1.0) ──

interface MockCriterion {
  id: string;
  field: string;
  fields?: string[];
  op: 'gte' | 'gt' | 'lte' | 'eq' | 'between' | 'percentile_gte' | 'product_lte' | 'missing';
  value?: number | [number, number];
  optional?: boolean;
  label: string;
  adapted?: string;
}

interface MockMethod {
  key: string;
  name: string;
  idea: string;
  reference: string;
  notApplicable: string | null;
  classes: AssetClass[];
  exclude: string[];
  minLiquidity: number;
  minLiquidityFii?: number;
  criteria: MockCriterion[];
  order:
    { field: string; dir: 'asc' | 'desc' } | { rankSum: { field: string; dir: 'asc' | 'desc' }[] };
  orderText: string;
}

const FIN = ['banc', 'segur', 'intermedia', 'financ', 'previd'];
const METHODS: MockMethod[] = [
  {
    key: 'graham_defensivo',
    name: 'Graham defensivo',
    idea: 'Empresas grandes, sólidas e baratas.',
    reference: 'Benjamin Graham, O Investidor Inteligente, cap. 14',
    notApplicable: 'Bancos, seguradoras e demais financeiras.',
    classes: ['stock_br'],
    exclude: FIN,
    minLiquidity: 1e6,
    criteria: [
      {
        id: 'tamanho',
        field: 'market_cap',
        op: 'percentile_gte',
        value: 0.6667,
        label: 'Valor de mercado no 67º percentil ou acima',
      },
      {
        id: 'liquidez_corrente',
        field: 'current_ratio',
        op: 'gte',
        value: 2,
        label: 'Liquidez corrente ≥ 2',
      },
      {
        id: 'sem_prejuizo',
        field: 'loss_years_10y',
        op: 'eq',
        value: 0,
        label: 'Anos com prejuízo (10 anos) = 0',
      },
      {
        id: 'proventos',
        field: 'dividend_years',
        op: 'gte',
        value: 10,
        label: 'Anos seguidos com proventos ≥ 10',
        adapted: 'O livro pede proventos em todos os últimos 20 anos.',
      },
      {
        id: 'crescimento_lucro',
        field: 'eps_growth_10y',
        op: 'gte',
        value: 0.3333,
        label: 'Crescimento do lucro em 10 anos ≥ 33,3%',
      },
      { id: 'pl', field: 'pl', op: 'between', value: [0, 15], label: 'P/L 0 – 15' },
      {
        id: 'pl_pvp',
        field: 'pl',
        fields: ['pl', 'pvp'],
        op: 'product_lte',
        value: 22.5,
        label: 'P/L × P/VP ≤ 22,5',
      },
    ],
    order: { field: 'graham_upside', dir: 'desc' },
    orderText: 'Pela margem até o preço de Graham.',
  },
  {
    key: 'bazin',
    name: 'Bazin',
    idea: 'Dividendos altos e constantes.',
    reference: 'Décio Bazin, Faça Fortuna com Ações',
    notApplicable: null,
    classes: ['stock_br'],
    exclude: [],
    minLiquidity: 1e6,
    criteria: [
      { id: 'dy', field: 'dy', op: 'gte', value: 0.06, label: 'DY ≥ 6%' },
      {
        id: 'proventos',
        field: 'dividend_years',
        op: 'gte',
        value: 5,
        label: 'Anos seguidos com proventos ≥ 5',
      },
      {
        id: 'divida',
        field: 'net_debt_equity',
        op: 'lte',
        value: 1,
        optional: true,
        label: 'Dívida líquida / PL ≤ 1',
      },
      {
        id: 'sem_prejuizo',
        field: 'loss_years_10y',
        op: 'eq',
        value: 0,
        label: 'Anos com prejuízo (10 anos) = 0',
        adapted: 'Bazin pede lucro nos últimos 5 anos.',
      },
    ],
    order: { field: 'bazin_upside', dir: 'desc' },
    orderText: 'Pela margem até o preço-teto de Bazin.',
  },
  {
    key: 'magic_formula',
    name: 'Magic Formula',
    idea: 'Bom negócio a preço baixo.',
    reference: 'Joel Greenblatt, The Little Book That Beats the Market',
    notApplicable: 'Bancos, seguradoras, holdings, energia elétrica e saneamento.',
    classes: ['stock_br'],
    exclude: [...FIN, 'holding', 'energia eletrica', 'saneamento'],
    minLiquidity: 1e6,
    criteria: [
      { id: 'ey', field: 'earnings_yield', op: 'gte', value: 0.000001, label: 'EBIT/EV > 0' },
      {
        id: 'roc',
        field: 'magic_roc',
        op: 'gte',
        value: 0.000001,
        label: 'ROC (Magic Formula) > 0',
      },
    ],
    order: {
      rankSum: [
        { field: 'earnings_yield', dir: 'desc' },
        { field: 'magic_roc', dir: 'desc' },
      ],
    },
    orderText: 'Soma das posições em EBIT/EV e em ROC; a menor primeiro.',
  },
  {
    key: 'qualidade_preco_justo',
    name: 'Qualidade e preço',
    idea: 'Saúde alta pelo score do Treasury, ordenada pela atratividade.',
    reference: 'Score de saúde do Treasury',
    notApplicable: 'Classes sem score.',
    classes: ['stock_br', 'fii'],
    exclude: [],
    minLiquidity: 1e6,
    minLiquidityFii: 5e5,
    criteria: [
      { id: 'saude', field: 'health', op: 'gte', value: 75, label: 'Saúde (score) ≥ 75' },
      {
        id: 'confianca',
        field: 'confidence',
        op: 'gte',
        value: 0.7,
        label: 'Confiança do score ≥ 70%',
      },
      { id: 'sem_red_flag', field: 'red_flags', op: 'missing', label: 'Sem red flags do score' },
    ],
    order: { field: 'attractiveness', dir: 'desc' },
    orderText: 'Pela atratividade do score.',
  },
  {
    key: 'fii_desconto',
    name: 'FIIs: renda com desconto',
    idea: 'FIIs abaixo do valor patrimonial, com renda alta e muitos cotistas.',
    reference: 'Critérios usuais de análise de FIIs',
    notApplicable: null,
    classes: ['fii'],
    exclude: [],
    minLiquidity: 5e5,
    criteria: [
      { id: 'pvp', field: 'pvp', op: 'between', value: [0.8, 1.0], label: 'P/VP 0,8 – 1' },
      {
        id: 'dy',
        field: 'dy',
        op: 'gte',
        value: 0.08,
        label: 'DY ≥ 8%',
        adapted: 'A referência é 80% do CDI.',
      },
      { id: 'cotistas', field: 'holders', op: 'gte', value: 5000, label: 'Cotistas ≥ 5.000' },
      {
        id: 'vacancia',
        field: 'vacancy',
        op: 'lte',
        value: 0.1,
        optional: true,
        label: 'Vacância ≤ 10%',
      },
      { id: 'score', field: 'score', op: 'gte', value: 60, optional: true, label: 'Score ≥ 60' },
    ],
    order: { field: 'dy', dir: 'desc' },
    orderText: 'Pelo DY.',
  },
];

function methodOf(key: string): MockMethod {
  const m = METHODS.find((x) => x.key === key);
  if (!m) notFound(`método ${key}`);
  return m;
}

function methodInfo(m: MockMethod): ScreenMethod {
  return {
    key: m.key,
    version: '0.1.0',
    name: m.name,
    idea: m.idea,
    reference: m.reference,
    not_applicable: m.notApplicable,
    classes: m.classes,
    hygiene: m.minLiquidityFii
      ? `Liquidez média diária de R$ ${m.minLiquidity.toLocaleString('pt-BR')} (ações) e R$ ${m.minLiquidityFii.toLocaleString('pt-BR')} (FIIs) ou mais`
      : `Liquidez média diária de R$ ${m.minLiquidity.toLocaleString('pt-BR')} ou mais`,
    criteria: m.criteria.map((c) => ({
      id: c.id,
      label: c.label,
      optional: c.optional ?? false,
      adapted: c.adapted ?? null,
    })),
    order: m.orderText,
  };
}

function methodExcluded(m: MockMethod, e: Entry): string | null {
  if (!m.classes.includes(e.subject.class)) return 'Classe fora do método';
  const sec = norm(e.subject.sector ?? '');
  if (m.exclude.some((x) => sec.includes(x)))
    return `O método não se aplica ao setor ${e.subject.sector}`;
  const min = e.subject.class === 'fii' ? (m.minLiquidityFii ?? m.minLiquidity) : m.minLiquidity;
  if ((e.metrics.liquidity ?? 0) < min)
    return `Liquidez abaixo de R$ ${min.toLocaleString('pt-BR')} por dia`;
  return null;
}

function methodChecks(m: MockMethod, e: Entry, pool: Entry[]): MethodCheck[] {
  return m.criteria.map((c) => {
    if (c.op === 'missing') {
      const v = c.field === 'red_flags' ? e.redFlags : null;
      return {
        id: c.id,
        label: c.label,
        actual: null,
        shown: v ?? '—',
        passed: !v,
        missing: false,
      };
    }
    let x: number | null = e.metrics[c.field] ?? null;
    if (c.op === 'product_lte')
      x = (c.fields ?? []).reduce<number | null>(
        (acc, f) => (acc == null || e.metrics[f] == null ? null : acc * e.metrics[f]),
        1,
      );
    if (c.op === 'percentile_gte' && x != null) {
      const vals = pool
        .filter((p) => p.subject.class === e.subject.class)
        .map((p) => p.metrics[c.field])
        .filter((v): v is number => v != null)
        .sort((a, b) => a - b);
      x = vals.length > 1 ? vals.indexOf(x) / (vals.length - 1) : 1;
    }
    if (x == null)
      return {
        id: c.id,
        label: c.label,
        actual: null,
        shown: '—',
        passed: c.optional ?? false,
        missing: true,
      };
    const v = c.value;
    const passed =
      c.op === 'between' && Array.isArray(v)
        ? x >= v[0] && x <= v[1]
        : typeof v === 'number' &&
          (c.op === 'gte' || c.op === 'percentile_gte'
            ? x >= v
            : c.op === 'gt'
              ? x > v
              : c.op === 'eq'
                ? Math.abs(x - v) < 1e-9
                : x <= v);
    return { id: c.id, label: c.label, actual: x, shown: x.toFixed(2), passed, missing: false };
  });
}

function runMethod(m: MockMethod): MethodRun {
  const all = universe();
  const pool = all.filter((e) => !methodExcluded(m, e));
  const passing = pool
    .map((e) => ({ e, checks: methodChecks(m, e, pool) }))
    .filter((x) => x.checks.every((c) => c.passed));
  const pos = (field: string, dir: 'asc' | 'desc') => {
    const sorted = [...passing].sort((a, b) => {
      const d = (a.e.metrics[field] ?? 0) - (b.e.metrics[field] ?? 0);
      return (dir === 'desc' ? -d : d) || a.e.ticker.localeCompare(b.e.ticker);
    });
    return new Map(sorted.map((x, i) => [x.e.ticker, i + 1]));
  };
  const keyed = passing.map((x) => {
    let v: number;
    let shown: number | null;
    if ('rankSum' in m.order) {
      v = m.order.rankSum.reduce((acc, k) => acc + (pos(k.field, k.dir).get(x.e.ticker) ?? 0), 0);
      shown = v;
    } else {
      v = pos(m.order.field, m.order.dir).get(x.e.ticker) ?? 0;
      shown = x.e.metrics[m.order.field] ?? null;
    }
    return { ...x, v, shown };
  });
  keyed.sort((a, b) => a.v - b.v || a.e.ticker.localeCompare(b.e.ticker));
  return {
    method: methodInfo(m),
    as_of: mockToday(),
    evaluated: pool.length,
    passing: keyed.length,
    rows: keyed.map((x, i) => ({
      ticker: x.e.ticker,
      name: x.e.subject.name,
      class: x.e.subject.class,
      sector: x.e.subject.sector,
      asset_id: x.e.assetId,
      rank: i + 1,
      order_value: x.shown,
      score: x.e.metrics.score ?? null,
      checks: x.checks,
    })),
  };
}

function explainMethod(m: MockMethod, ticker: string): MethodExplain {
  const t = ticker.trim().toUpperCase();
  const e = universe().find((x) => x.ticker === t);
  if (!e) notFound(`${t} no universo`);
  const why = methodExcluded(m, e);
  if (why)
    return { method_key: m.key, ticker: t, passed: false, excluded: why, checks: [], rank: null };
  const pool = universe().filter((x) => !methodExcluded(m, x));
  const checks = methodChecks(m, e, pool);
  const passed = checks.every((c) => c.passed);
  const rank = passed ? (runMethod(m).rows.find((r) => r.ticker === t)?.rank ?? null) : null;
  return { method_key: m.key, ticker: t, passed, excluded: null, checks, rank };
}

function call(cmd: string, args: Record<string, unknown>): unknown {
  const h = extraMockHandler(cmd);
  if (!h) throw new Error(`mock sem ${cmd}`);
  return h(args);
}

function observe(a: Record<string, unknown>): { asset_id: string } {
  const ticker = typeof a.ticker === 'string' ? a.ticker.trim().toUpperCase() : '';
  const e = entryOf({ ticker });
  let id = e.assetId;
  if (!id) {
    const created = call('assets.create', {
      ticker,
      name: e.subject.issuer_name ?? e.subject.name,
      class: e.subject.class,
      market: 'B3',
    }) as { id: string };
    id = created.id;
    ASSET_ID[ticker] = id;
    observedIds.add(id);
  }
  call('portfolio.watch', { asset_id: id });
  return { asset_id: id };
}

function unobserve(a: Record<string, unknown>): { removed: boolean } {
  const id = String(a.asset_id);
  call('portfolio.unwatch', { asset_id: id });
  if (!observedIds.has(id)) return { removed: false };
  observedIds.delete(id);
  if (heldIds().has(id)) return { removed: false };
  call('assets.delete', { id });
  for (const [k, v] of Object.entries(ASSET_ID)) if (v === id) delete ASSET_ID[k];
  return { removed: true };
}

// ── Ranking e screener (as mesmas regras de `research::screen`) ──

function norm(s: string): string {
  return s
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase()
    .trim();
}

function passes(e: Entry, f: UniverseFilter, mine: Set<string>): boolean {
  if (f.classes?.length && !f.classes.includes(e.subject.class)) return false;
  const has = (v: string | null, wanted?: string[]) =>
    !wanted?.length || (v != null && wanted.some((w) => norm(v).includes(norm(w))));
  if (!has(e.subject.sector, f.sectors) || !has(e.subject.segment, f.segments)) return false;
  if (f.min_liquidity != null && !((e.metrics.liquidity ?? -1) >= f.min_liquidity)) return false;
  if (f.only_mine && !(e.assetId && mine.has(e.assetId))) return false;
  return true;
}

const TEXT = [
  'ticker',
  'name',
  'class',
  'sector',
  'segment',
  'currency',
  'score_band',
  'red_flags',
];
const BOOL = ['held', 'watched'];

function validateCondition(c: Condition): void {
  const f = c.field;
  const numeric = DEF.has(f);
  if (!numeric && !TEXT.includes(f) && !BOOL.includes(f))
    invalid(
      `campo desconhecido no filtro: ${f} (indicadores de research.catalog, ou ${[...TEXT, ...BOOL].join(', ')})`,
    );
  const bad = (m: string) => invalid(`filtro ${f}: ${m}`);
  const v = c.value;
  switch (c.op) {
    case 'exists':
    case 'missing':
      return;
    case 'gt':
    case 'gte':
    case 'lt':
    case 'lte':
      if (!numeric) bad('comparação de tamanho só vale para indicadores');
      if (typeof v !== 'number') bad('informe um número em `value`');
      return;
    case 'between':
      if (!(numeric && Array.isArray(v) && v.length === 2 && v.every((x) => typeof x === 'number')))
        bad('`between` precisa de `value: [mínimo, máximo]` num indicador');
      return;
    case 'in':
    case 'not_in':
      if (!Array.isArray(v) || v.length === 0)
        bad('`in`/`not_in` precisam de uma lista em `value`');
      return;
    case 'contains':
      if (numeric || typeof v !== 'string')
        bad('`contains` precisa de um texto num campo de texto');
      return;
    case 'eq':
    case 'ne':
      if (v === undefined || v === null) bad('informe `value`');
  }
}

function validateQuery(q: ScreenQuery): void {
  for (const c of [...(q.conditions ?? []), ...(q.groups ?? []).flatMap((g) => g.conditions)])
    validateCondition(c);
  if ((q.groups ?? []).some((g) => g.conditions.length === 0)) invalid('grupo de filtros vazio');
}

function textOf(e: Entry, f: string): string | null {
  switch (f) {
    case 'ticker':
      return e.ticker;
    case 'name':
      return e.subject.name;
    case 'class':
      return e.subject.class;
    case 'sector':
      return e.subject.sector;
    case 'segment':
      return e.subject.segment;
    case 'currency':
      return e.subject.currency;
    case 'score_band':
      return e.scoreBand;
    case 'red_flags':
      return e.redFlags;
    default:
      return null;
  }
}

interface EvalCtx {
  held: Set<string>;
  watched: Set<string>;
}

function evalCond(e: Entry, c: Condition, cx: EvalCtx): boolean {
  const f = c.field;
  const v = c.value;
  if (BOOL.includes(f)) {
    const actual = Boolean(e.assetId && (f === 'held' ? cx.held : cx.watched).has(e.assetId));
    const want = typeof v === 'boolean' ? v : true;
    if (c.op === 'ne') return actual !== want;
    if (c.op === 'missing') return !actual;
    if (c.op === 'exists') return actual;
    return actual === want;
  }
  if (DEF.has(f)) {
    const x = e.metrics[f];
    if (c.op === 'exists') return x != null;
    if (c.op === 'missing') return x == null;
    if (x == null) return false;
    const n = typeof v === 'number' ? v : null;
    const ops: Partial<Record<CmpOp, () => boolean>> = {
      gt: () => n != null && x > n,
      gte: () => n != null && x >= n,
      lt: () => n != null && x < n,
      lte: () => n != null && x <= n,
      eq: () => n != null && Math.abs(x - n) < 1e-9,
      ne: () => n != null && Math.abs(x - n) >= 1e-9,
      between: () =>
        Array.isArray(v) && x >= Number(v[0] ?? -Infinity) && x <= Number(v[1] ?? Infinity),
      in: () => Array.isArray(v) && v.some((y) => typeof y === 'number' && Math.abs(x - y) < 1e-9),
      not_in: () =>
        !(Array.isArray(v) && v.some((y) => typeof y === 'number' && Math.abs(x - y) < 1e-9)),
    };
    return ops[c.op]?.() ?? false;
  }
  const t = textOf(e, f);
  if (c.op === 'exists') return t != null;
  if (c.op === 'missing') return t == null;
  if (t == null) return false;
  const same = (b: unknown) => typeof b === 'string' && norm(t) === norm(b);
  switch (c.op) {
    case 'eq':
      return same(v);
    case 'ne':
      return v != null && !same(v);
    case 'contains':
      return typeof v === 'string' && norm(t).includes(norm(v));
    case 'in':
      return Array.isArray(v) && v.some(same);
    case 'not_in':
      return !(Array.isArray(v) && v.some(same));
    default:
      return false;
  }
}

function combine(logic: Logic, results: boolean[]): boolean | null {
  if (!results.length) return null;
  return logic === 'and' ? results.every(Boolean) : results.some(Boolean);
}

function matches(e: Entry, q: ScreenQuery, cx: EvalCtx): boolean {
  const parts = [
    ...(q.conditions ?? []).map((c) => evalCond(e, c, cx)),
    ...(q.groups ?? []).map(
      (g) =>
        combine(
          g.logic,
          g.conditions.map((c) => evalCond(e, c, cx)),
        ) ?? true,
    ),
  ];
  return combine(q.logic ?? 'and', parts) ?? true;
}

const OP_TEXT: Record<CmpOp, string> = {
  gt: '>',
  gte: '≥',
  lt: '<',
  lte: '≤',
  eq: '=',
  ne: '≠',
  between: 'entre',
  in: 'em',
  not_in: 'fora de',
  contains: 'contém',
  exists: 'disponível',
  missing: 'indisponível',
};

function describeCondition(c: Condition): string {
  const label = DEF.get(c.field)?.label ?? c.field;
  const v = c.value == null ? '' : typeof c.value === 'string' ? c.value : JSON.stringify(c.value);
  return c.op === 'exists' || c.op === 'missing'
    ? `${label} ${OP_TEXT[c.op]}`
    : `${label} ${OP_TEXT[c.op]} ${v}`;
}

function describe(q: ScreenQuery): string {
  const join = (l: Logic) => (l === 'and' ? ' E ' : ' OU ');
  const parts = (q.conditions ?? []).map(describeCondition);
  for (const g of q.groups ?? [])
    parts.push(`(${g.conditions.map(describeCondition).join(join(g.logic))})`);
  if (q.tags?.length)
    parts.push(`características: ${q.tags.map((k) => TAG_LABEL[k] ?? k).join(', ')}`);
  return parts.length ? parts.join(join(q.logic ?? 'and')) : 'todos os papéis';
}

function sortByMetric(list: Entry[], key: string, order: SortOrder): Entry[] {
  return [...list].sort((x, y) => {
    const a = x.metrics[key];
    const b = y.metrics[key];
    if (a != null && b != null && a !== b) return order === 'asc' ? a - b : b - a;
    if (a != null && b == null) return -1;
    if (a == null && b != null) return 1;
    return x.ticker.localeCompare(y.ticker);
  });
}

function infoOf(key: string): IndicatorInfo {
  const i = catalog().find((c) => c.key === key);
  if (!i) invalid(`indicador desconhecido: ${key} (veja research.catalog)`);
  return i;
}

function rank(a: Record<string, unknown>): RankResult {
  const key = typeof a.indicator === 'string' ? a.indicator : '';
  const info = infoOf(key);
  const def = DEF.get(key)!;
  const order: SortOrder =
    a.order === 'asc' || a.order === 'desc' ? a.order : def.hib === false ? 'asc' : 'desc';
  const f = (a.filters ?? {}) as UniverseFilter;
  const mine = mineIds();
  const scoped = universe().filter((e) => passes(e, f, mine));
  const withV = scoped.filter((e) => {
    const v = e.metrics[key];
    return v != null && Number.isFinite(v) && (!def.positiveOnly || v > 0);
  });
  const sorted = sortByMetric(withV, key, order);
  const limit = Math.min(500, Math.max(1, typeof a.limit === 'number' ? a.limit : 20));
  return {
    indicator: info,
    order,
    as_of: mockToday(),
    total: withV.length,
    without_value: scoped.length - withV.length,
    items: sorted.slice(0, limit).map((e, i): RankItem => ({
      position: i + 1,
      ticker: e.ticker,
      asset_id: e.assetId,
      name: e.subject.name,
      class: e.subject.class,
      sector: e.subject.sector,
      segment: e.subject.segment,
      value: e.metrics[key]!,
      price: e.metrics.price ?? null,
      liquidity: e.metrics.liquidity ?? null,
    })),
  };
}

const DEFAULT_FIELDS = ['price', 'market_cap', 'liquidity', 'pl', 'pvp', 'dy', 'roe'];

function screenQuery(a: Record<string, unknown>): ScreenQuery {
  if (a.filters) return a.filters as ScreenQuery;
  if (typeof a.screen_id === 'string') return getScreen(a.screen_id).filters;
  return { logic: 'and', conditions: [], groups: [] };
}

// ── Características como tags (como `research::tags`, methodology/tags/v0.1.0) ──

interface MockTag {
  key: string;
  group: string;
  rules?: { classes: string[]; all: Condition[] }[];
  method?: string;
}

const TAGS: MockTag[] = [
  {
    key: 'nunca_prejuizo',
    group: 'historico',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'loss_years_10y',
            op: 'eq',
            value: 0,
          },
          {
            field: 'eps_growth_10y',
            op: 'exists',
          },
        ],
      },
    ],
  },
  {
    key: 'lucro_crescente',
    group: 'historico',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'earnings_cagr_5y',
            op: 'gte',
            value: 0.05,
          },
        ],
      },
    ],
  },
  {
    key: 'receita_crescente',
    group: 'historico',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'revenue_cagr_5y',
            op: 'gte',
            value: 0.05,
          },
        ],
      },
    ],
  },
  {
    key: 'proventos_10_anos',
    group: 'proventos',
    rules: [
      {
        classes: ['stock_br', 'stock_us', 'fii'],
        all: [
          {
            field: 'dividend_years',
            op: 'gte',
            value: 10,
          },
        ],
      },
    ],
  },
  {
    key: 'proventos_5_anos',
    group: 'proventos',
    rules: [
      {
        classes: ['stock_br', 'stock_us', 'fii'],
        all: [
          {
            field: 'dividend_years',
            op: 'gte',
            value: 5,
          },
        ],
      },
    ],
  },
  {
    key: 'proventos_crescentes',
    group: 'proventos',
    rules: [
      {
        classes: ['stock_br', 'stock_us', 'fii'],
        all: [
          {
            field: 'dividend_cagr_5y',
            op: 'gt',
            value: 0,
          },
        ],
      },
    ],
  },
  {
    key: 'dy_alto',
    group: 'proventos',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'dy',
            op: 'gte',
            value: 0.06,
          },
        ],
      },
      {
        classes: ['fii'],
        all: [
          {
            field: 'dy',
            op: 'gte',
            value: 0.1,
          },
        ],
      },
    ],
  },
  {
    key: 'divida_baixa',
    group: 'solidez',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'net_debt_ebitda',
            op: 'lte',
            value: 1,
          },
        ],
      },
    ],
  },
  {
    key: 'caixa_liquido',
    group: 'solidez',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'net_debt_equity',
            op: 'lt',
            value: 0,
          },
        ],
      },
    ],
  },
  {
    key: 'folga_curto_prazo',
    group: 'solidez',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'current_ratio',
            op: 'gte',
            value: 1.5,
          },
        ],
      },
    ],
  },
  {
    key: 'roe_alto',
    group: 'rentabilidade',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'roe',
            op: 'gte',
            value: 0.15,
          },
        ],
      },
    ],
  },
  {
    key: 'margem_alta',
    group: 'rentabilidade',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'net_margin',
            op: 'gte',
            value: 0.15,
          },
        ],
      },
    ],
  },
  {
    key: 'piotroski_forte',
    group: 'rentabilidade',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'piotroski',
            op: 'gte',
            value: 7,
          },
        ],
      },
    ],
  },
  {
    key: 'score_saudavel',
    group: 'score',
    rules: [
      {
        classes: ['stock_br', 'fii'],
        all: [
          {
            field: 'score_band',
            op: 'in',
            value: ['robusto', 'saudavel'],
          },
        ],
      },
    ],
  },
  {
    key: 'sem_alertas',
    group: 'score',
    rules: [
      {
        classes: ['stock_br', 'fii'],
        all: [
          {
            field: 'score',
            op: 'exists',
          },
          {
            field: 'red_flags',
            op: 'missing',
          },
        ],
      },
    ],
  },
  {
    key: 'abaixo_vp',
    group: 'preco',
    rules: [
      {
        classes: ['stock_br', 'stock_us', 'fii'],
        all: [
          {
            field: 'pvp',
            op: 'between',
            value: [1e-6, 0.999999],
          },
        ],
      },
    ],
  },
  {
    key: 'pl_baixo',
    group: 'preco',
    rules: [
      {
        classes: ['stock_br', 'stock_us'],
        all: [
          {
            field: 'pl',
            op: 'between',
            value: [1e-6, 10],
          },
        ],
      },
    ],
  },
  {
    key: 'liquidez_alta',
    group: 'mercado',
    rules: [
      {
        classes: ['stock_br'],
        all: [
          {
            field: 'liquidity',
            op: 'gte',
            value: 10000000,
          },
        ],
      },
      {
        classes: ['fii'],
        all: [
          {
            field: 'liquidity',
            op: 'gte',
            value: 1000000,
          },
        ],
      },
    ],
  },
  {
    key: 'grande',
    group: 'mercado',
    rules: [
      {
        classes: ['stock_br'],
        all: [
          {
            field: 'market_cap',
            op: 'gte',
            value: 10000000000,
          },
        ],
      },
      {
        classes: ['fii'],
        all: [
          {
            field: 'market_cap',
            op: 'gte',
            value: 1000000000,
          },
        ],
      },
    ],
  },
  {
    key: 'baixa_volatilidade',
    group: 'mercado',
    rules: [
      {
        classes: ['stock_br', 'fii'],
        all: [
          {
            field: 'volatility_1y',
            op: 'lte',
            value: 0.25,
          },
        ],
      },
    ],
  },
  {
    key: 'em_alta_12m',
    group: 'mercado',
    rules: [
      {
        classes: ['stock_br', 'fii'],
        all: [
          {
            field: 'momentum_12_1',
            op: 'gt',
            value: 0,
          },
        ],
      },
    ],
  },
  {
    key: 'muitos_cotistas',
    group: 'fii',
    rules: [
      {
        classes: ['fii'],
        all: [
          {
            field: 'holders',
            op: 'gte',
            value: 100000,
          },
        ],
      },
    ],
  },
  {
    key: 'vacancia_baixa',
    group: 'fii',
    rules: [
      {
        classes: ['fii'],
        all: [
          {
            field: 'vacancy',
            op: 'lte',
            value: 0.05,
          },
        ],
      },
    ],
  },
  {
    key: 'metodo_graham_defensivo',
    group: 'metodos',
    method: 'graham_defensivo',
  },
  {
    key: 'metodo_bazin',
    group: 'metodos',
    method: 'bazin',
  },
  {
    key: 'metodo_magic_formula',
    group: 'metodos',
    method: 'magic_formula',
  },
  {
    key: 'metodo_qualidade_preco_justo',
    group: 'metodos',
    method: 'qualidade_preco_justo',
  },
  {
    key: 'metodo_fii_desconto',
    group: 'metodos',
    method: 'fii_desconto',
  },
];

const TAG_LABEL: Record<string, string> = {
  nunca_prejuizo: 'Nunca deu prejuízo',
  lucro_crescente: 'Lucro crescendo',
  receita_crescente: 'Receita crescendo',
  proventos_10_anos: 'Proventos há 10 anos',
  proventos_5_anos: 'Proventos há 5 anos',
  proventos_crescentes: 'Proventos crescendo',
  dy_alto: 'Dividend yield alto',
  divida_baixa: 'Dívida baixa',
  caixa_liquido: 'Caixa maior que a dívida',
  folga_curto_prazo: 'Folga no curto prazo',
  roe_alto: 'ROE alto',
  margem_alta: 'Margem alta',
  piotroski_forte: 'Piotroski forte',
  score_saudavel: 'Saudável no score',
  sem_alertas: 'Sem alertas no score',
  abaixo_vp: 'Abaixo do valor patrimonial',
  pl_baixo: 'P/L baixo',
  liquidez_alta: 'Liquidez alta',
  grande: 'Grande',
  baixa_volatilidade: 'Pouca oscilação',
  em_alta_12m: 'Em alta em 12 meses',
  muitos_cotistas: 'Muitos cotistas',
  vacancia_baixa: 'Vacância baixa',
  metodo_graham_defensivo: 'Cumpre Graham defensivo',
  metodo_bazin: 'Cumpre Bazin',
  metodo_magic_formula: 'Cumpre Magic Formula',
  metodo_qualidade_preco_justo: 'Cumpre Qualidade e preço',
  metodo_fii_desconto: 'Cumpre FIIs com desconto',
};

const TAG_GROUP: Record<string, string> = {
  historico: 'Histórico',
  proventos: 'Proventos',
  solidez: 'Solidez',
  rentabilidade: 'Rentabilidade',
  score: 'Score Treasury',
  preco: 'Preço',
  mercado: 'Mercado',
  fii: 'FIIs',
  metodos: 'Métodos',
};

const TAG_CLASS: Record<string, string> = {
  stock_br: 'Ações',
  fii: 'FIIs',
  stock_us: 'Ações dos EUA',
};

/** Tags de cada papel do universo (ticker → chaves, na ordem do catálogo). */
function tagsOfUniverse(): Record<string, string[]> {
  const es = universe();
  const badges: Record<string, string[]> = {};
  for (const m of METHODS) for (const r of runMethod(m).rows) (badges[r.ticker] ??= []).push(m.key);
  const none = { held: new Set<string>(), watched: new Set<string>() };
  const out: Record<string, string[]> = {};
  for (const tag of TAGS)
    for (const e of es) {
      const has = tag.method
        ? (badges[e.ticker] ?? []).includes(tag.method)
        : (tag.rules ?? []).some(
            (r) =>
              r.classes.includes(e.subject.class) &&
              matches(e, { logic: 'and', conditions: r.all, groups: [] }, none),
          );
      if (has) (out[e.ticker] ??= []).push(tag.key);
    }
  return out;
}

function tagCatalog(): AssetTag[] {
  const tagged = Object.values(tagsOfUniverse());
  return TAGS.map((tag) => {
    const method = tag.method ? METHODS.find((m) => m.key === tag.method) : undefined;
    const many = (tag.rules ?? []).length > 1;
    const rule = method
      ? `cumpre todos os critérios do método ${method.name} (aba Métodos)`
      : (tag.rules ?? [])
          .map((r) => {
            const text = describe({ logic: 'and', conditions: r.all, groups: [] });
            return many ? `${r.classes.map((c) => TAG_CLASS[c] ?? c).join(', ')}: ${text}` : text;
          })
          .join(' · ');
    const classes = method
      ? method.classes
      : [...new Set((tag.rules ?? []).flatMap((r) => r.classes))];
    return {
      key: tag.key,
      label: TAG_LABEL[tag.key] ?? tag.key,
      group: tag.group,
      group_label: TAG_GROUP[tag.group] ?? tag.group,
      rule,
      classes: classes as AssetClass[],
      count: tagged.filter((v) => v.includes(tag.key)).length,
      version: '0.1.0',
    };
  });
}

function runScreen(a: Record<string, unknown>): ScreenResult {
  const q = screenQuery(a);
  validateQuery(q);
  const sortBy = typeof a.sort_by === 'string' ? a.sort_by : 'market_cap';
  const def = DEF.get(sortBy);
  if (!def) invalid(`indicador desconhecido para ordenar: ${sortBy}`);
  const order: SortOrder =
    a.order === 'asc' || a.order === 'desc' ? a.order : def.hib === false ? 'asc' : 'desc';
  const mine = mineIds();
  const cx = { held: heldIds(), watched: watchedIds() };
  const wanted = q.tags ?? [];
  for (const k of wanted)
    if (!TAGS.some((x) => x.key === k)) invalid(`característica desconhecida: ${k}`);
  const tagged = tagsOfUniverse();
  const hits = sortByMetric(
    universe().filter(
      (e) =>
        passes(e, q.universe ?? {}, mine) &&
        matches(e, q, cx) &&
        wanted.every((k) => (tagged[e.ticker] ?? []).includes(k)),
    ),
    sortBy,
    order,
  );
  const keep = new Set([...DEFAULT_FIELDS, sortBy]);
  for (const c of [...(q.conditions ?? []), ...(q.groups ?? []).flatMap((g) => g.conditions)])
    keep.add(c.field);
  if (Array.isArray(a.fields)) for (const f of a.fields) keep.add(String(f));
  const limit = Math.min(500, Math.max(1, typeof a.limit === 'number' ? a.limit : 50));
  // Contagem facetada: quantos do recorte (antes do limite) têm cada característica.
  const tagCounts: Record<string, number> = {};
  for (const e of hits)
    for (const k of tagged[e.ticker] ?? []) tagCounts[k] = (tagCounts[k] ?? 0) + 1;
  return {
    as_of: mockToday(),
    description: describe(q),
    total: hits.length,
    tag_counts: tagCounts,
    items: hits.slice(0, limit).map((e) => ({
      ticker: e.ticker,
      asset_id: e.assetId,
      name: e.subject.name,
      class: e.subject.class,
      sector: e.subject.sector,
      segment: e.subject.segment,
      currency: e.subject.currency,
      metrics: Object.fromEntries(Object.entries(e.metrics).filter(([k]) => keep.has(k))),
      tags: tagged[e.ticker] ?? [],
    })),
  };
}

// ── Telas salvas (no banco do mock: entram no histórico) ────────

function toScreen(r: {
  id: string;
  name: string;
  description: string | null;
  query: ScreenQuery;
  created_at: string;
  updated_at: string;
  alert?: boolean;
}): Screen {
  return {
    id: r.id,
    name: r.name,
    description: r.description,
    filters: r.query,
    alert: r.alert ?? false,
    summary: describe(r.query),
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

function getScreen(id: string): Screen {
  const r = find('screen', id);
  if (!r) notFound(`tela ${id}`);
  return toScreen(r);
}

function checkName(name: unknown, except?: string): string {
  const n = typeof name === 'string' ? name.trim() : '';
  if (!n) invalid('informe o nome da tela');
  if (rows('screen').some((r) => r.name.toLowerCase() === n.toLowerCase() && r.id !== except))
    conflict(`já existe uma tela chamada ${n}`);
  return n;
}

function normQuery(q: unknown): ScreenQuery {
  if (!q || typeof q !== 'object') invalid('argumentos inválidos: missing field `filters`');
  const x = q as Partial<ScreenQuery>;
  const out: ScreenQuery = {
    logic: x.logic ?? 'and',
    conditions: x.conditions ?? [],
    groups: (x.groups ?? []).map((g) => ({ logic: g.logic ?? 'and', conditions: g.conditions })),
    ...(x.universe ? { universe: x.universe } : {}),
  };
  validateQuery(out);
  return out;
}

// ── Comparador ───────────────────────────────────────────────────

function compare(a: Record<string, unknown>): CompareView {
  const refs = [
    ...((Array.isArray(a.asset_ids) ? a.asset_ids : []) as string[]).map((id) => ({
      asset_id: id,
    })),
    ...((Array.isArray(a.tickers) ? a.tickers : []) as string[]).map((t) => ({ ticker: t })),
  ];
  if (refs.length < 2) invalid('compare pelo menos 2 ativos');
  if (refs.length > 6) invalid('compare no máximo 6 ativos');
  const entries = refs.map((r) => entryOf(r));
  const rowsOut: CompareRow[] = [];
  for (const def of CATALOG) {
    if (!entries.some((e) => e.indicators.some((i) => i.key === def.key))) continue;
    const values = entries.map((e) => e.metrics[def.key] ?? null);
    let best: number | null = null;
    if (def.hib != null) {
      values.forEach((v, i) => {
        if (v == null || (def.positiveOnly && v <= 0)) return;
        const b = best == null ? null : values[best]!;
        if (b == null || (def.hib ? v > b : v < b)) best = i;
      });
    }
    rowsOut.push({
      key: def.key,
      label: def.label,
      group: def.group,
      unit: def.unit,
      values,
      best,
    });
  }
  const today = mockToday();
  const from = addMonths(today, -12);
  return {
    as_of: today,
    subjects: entries.map((e) => e.subject),
    rows: rowsOut,
    price_index: entries.map((e) => {
      const pts = e.assetId
        ? pricePoints(e.assetId).filter((p) => p.date >= from && p.date <= today)
        : [];
      const base = pts.length ? Number(pts[0]!.close) : 0;
      return {
        ticker: e.ticker,
        points: base
          ? pts.map((p) => ({
              date: p.date,
              value: Number(((Number(p.close) / base) * 100).toFixed(4)),
            }))
          : [],
      };
    }),
  };
}

// ── Estado, ajustes e coleta ─────────────────────────────────────

function settingsView(): ResearchSettingsView {
  const ua = state.settings.sec_user_agent.trim();
  const isDefault = !ua || ua === DEFAULT_UA || ua.includes('@exemplo');
  return {
    settings: { ...state.settings },
    sec_user_agent_is_default: isDefault,
    sec_hint: isDefault ? SEC_HINT : null,
    last_sync_at: state.lastSync,
  };
}

const MB = 1_000_000;

function syncReport(force: boolean, sources: SyncSource[] | null): ResearchSyncReport {
  const first = state.syncs === 0 || force;
  const y = Number(mockToday().slice(0, 4));
  const want = (s: SyncSource) => !sources?.length || sources.includes(s);
  const item = (
    source: SyncSource,
    label: string,
    bytes: number,
    rows: number,
    url: string | null = null,
  ): SyncItem => ({
    source,
    label,
    url,
    status: first ? 'updated' : 'unchanged',
    bytes: first ? bytes : 0,
    rows: first ? rows : 0,
    error: null,
  });
  const items: SyncItem[] = [];
  if (want('fca'))
    for (const yy of [y - 1, y]) items.push(item('fca', `CVM FCA ${yy}`, 0.4 * MB, 2_480));
  if (want('dfp'))
    for (let yy = y - 10; yy < y; yy += 1)
      items.push(item('dfp', `CVM DFP ${yy}`, 12.1 * MB, 41_000));
  if (want('itr'))
    for (const yy of [y - 2, y - 1, y])
      items.push(item('itr', `CVM ITR ${yy}`, 26.4 * MB, 118_000));
  if (want('fii'))
    for (let yy = y - 5; yy <= y; yy += 1)
      items.push(item('fii', `CVM informe mensal de FII ${yy}`, 1.6 * MB, 6_100));
  if (want('ipe'))
    for (const yy of [y - 1, y]) items.push(item('ipe', `CVM IPE ${yy}`, 2.1 * MB, 9_800));
  if (want('b3')) {
    const it = item(
      'b3',
      `B3 COTAHIST ${addDays(mockToday(), -1).split('-').reverse().join('')}`,
      0.6 * MB,
      4_200,
    );
    items.push({ ...it, status: 'updated', bytes: 0.6 * MB, rows: 4_200 });
  }
  if (want('b3_listados'))
    items.push({
      source: 'b3_listados',
      label: 'B3 empresas e fundos listados · proventos e eventos (66 de 880 emissores e fundos)',
      url: null,
      status: 'updated',
      bytes: 0.4 * MB,
      rows: 1_350,
      error: null,
    });
  const warnings: string[] = [];
  if (want('sec')) {
    if (settingsView().sec_user_agent_is_default) {
      warnings.push(SEC_HINT);
      items.push({
        source: 'sec',
        label: 'SEC company_tickers.json',
        url: null,
        status: 'skipped',
        bytes: 0,
        rows: 0,
        error: SEC_HINT,
      });
    } else {
      items.push(item('sec', 'SEC company_tickers.json', 1.0 * MB, 10_300));
      for (const t of ['AAPL', 'MSFT', 'NVDA'])
        items.push(item('sec', `SEC companyfacts ${t}`, 0.3 * MB, 1_900));
    }
  }
  const now = new Date();
  return {
    started_at: new Date(now.getTime() - (first ? 84_000 : 6_000)).toISOString(),
    finished_at: now.toISOString(),
    ok: items.every((i) => i.status !== 'error'),
    items: items.map((i) => ({ ...i, bytes: Math.round(i.bytes) })),
    bytes: Math.round(items.reduce((t, i) => t + i.bytes, 0)),
    universe: SPECS.length,
    radar: 18,
    warnings,
  };
}

function status(): ResearchStatus {
  const empty = state.empty;
  return {
    last_sync_at: state.lastSync,
    issuers: empty ? 0 : 2_412,
    securities: empty ? 0 : 1_187,
    facts: empty ? 0 : 1_284_000,
    documents: empty ? 0 : 19_600,
    quotes_last_date: empty ? null : addDays(mockToday(), -1),
    universe_as_of: empty ? null : mockToday(),
    universe_size: empty ? 0 : SPECS.length,
    last_report: state.lastReport,
  };
}

function normalizeRate(r: unknown): number {
  const n = typeof r === 'number' ? r : Number.NaN;
  const v = n > 1 ? n / 100 : n;
  if (!(v > 0 && v <= 1) || !Number.isFinite(v))
    invalid('a taxa mínima do Bazin deve ser positiva (ex.: 0.06 ou 6 para 6%)');
  return v;
}

function validateYears(y: unknown): number {
  const n = typeof y === 'number' ? y : Number.NaN;
  if (!Number.isInteger(n) || n < 1 || n > 10) invalid('os anos da média do Bazin vão de 1 a 10');
  return n;
}

registerMockHandlers(
  guarded({
    'research.catalog': () => catalog(),
    'research.status': () => status(),
    'research.get_settings': () => settingsView(),
    'research.set_settings': (a) => {
      const next = { ...state.settings };
      for (const [k, v] of Object.entries(a)) {
        if (!(k in next)) invalid(`ajuste desconhecido: ${k}`);
        (next as Record<string, unknown>)[k] = v;
      }
      const ua = String(next.sec_user_agent ?? '').trim();
      if (!ua) invalid('informe o User-Agent da SEC (nome do app e e-mail)');
      if (ua.length > 200) invalid('User-Agent inválido');
      if (!ua.includes('@'))
        invalid(
          'a SEC exige um e-mail de contato no User-Agent (ex.: Treasury/0.1 seu.nome@email.com)',
        );
      state.settings = {
        sec_user_agent: ua,
        bazin_rate: normalizeRate(next.bazin_rate),
        bazin_years: validateYears(next.bazin_years),
      };
      return settingsView();
    },
    'research.fundamentals': (a) => {
      const e = entryOf(a);
      return {
        subject: e.subject,
        issuer: null,
        securities: [],
        reference_date: REF_DATE,
        latest: [],
        annual: [],
        last_sync_at: state.lastSync,
      };
    },
    'research.indicators': (a) => indicatorsView(entryOf(a)),
    'research.fair_price': (a) => {
      const e = entryOf(a);
      const rate = a.bazin_rate == null ? state.settings.bazin_rate : normalizeRate(a.bazin_rate);
      const years = a.years == null ? state.settings.bazin_years : validateYears(a.years);
      return fairPrice(e, rate, years);
    },
    'research.history': (a) => {
      const y = typeof a.years === 'number' ? Math.min(15, Math.max(1, a.years)) : 10;
      return history(entryOf(a), Math.min(10, y));
    },
    'research.documents': (a) => documents(a),
    // Notícias do ativo (docs/asset-overview.md): manchetes inventadas com o ticker.
    'research.news': (a): NewsView => {
      const ticker = ((): string | null => {
        try {
          return entryOf(a).subject.ticker;
        } catch {
          return typeof a.ticker === 'string' ? a.ticker.toUpperCase() : null;
        }
      })();
      const feeds = sourceEnabled('news_feeds');
      const day = mockToday();
      const items: NewsItem[] =
        ticker && feeds
          ? [
              {
                title: `${ticker} sobe após resultado do trimestre`,
                link: `https://portal.exemplo.invalid/${ticker.toLowerCase()}-resultado`,
                source: 'Portal Exemplo',
                date: day,
              },
            ]
          : [];
      return {
        items,
        sources: [{ source: 'Portal Exemplo', ok: true }],
        ...(ticker
          ? {
              search_url: `https://news.google.com/search?q=${ticker}&hl=pt-BR&gl=BR&ceid=BR:pt-419`,
            }
          : {}),
        feeds_enabled: feeds,
      };
    },
    'research.documents_status': (a) => documentsStatus(a),
    'research.market_overview': () => marketOverview(),
    'research.methods': () => METHODS.map(methodInfo),
    'research.method_counts': () =>
      Object.fromEntries(METHODS.map((m) => [m.key, runMethod(m).passing])),
    'research.method_run': (a) => runMethod(methodOf(String(a.key))),
    'research.method_explain': (a) => explainMethod(methodOf(String(a.key)), String(a.ticker)),
    'research.method_badges': () => {
      const out: Record<string, string[]> = {};
      for (const m of METHODS)
        for (const r of runMethod(m).rows) (out[r.ticker] ??= []).push(m.key);
      return out;
    },
    'research.market_map': () => marketMap(),
    'research.observed': (a) => observedIds.has(String(a.id)),
    'research.observe': (a) => observe(a),
    'research.unobserve': (a) => unobserve(a),
    'research.rank': (a) => rank(a),
    'research.screen': (a) => runScreen(a),
    'research.tags': () => tagCatalog(),
    'research.screens.list': () =>
      rows('screen')
        .map(toScreen)
        .sort((x, y) => x.name.toLowerCase().localeCompare(y.name.toLowerCase())),
    'research.screens.get': (a) => getScreen(String(a.id)),
    'research.screens.create': (a) => {
      const name = checkName(a.name);
      const query = normQuery(a.filters);
      const r = write(`Tela do screener criada: ${name}`, () => {
        const now = nowUtc();
        const description =
          typeof a.description === 'string' && a.description.trim() ? a.description.trim() : null;
        return put('screen', {
          id: newId(),
          name,
          description,
          query,
          created_at: now,
          updated_at: now,
          alert: a.alert === true,
        });
      });
      return toScreen(r.value);
    },
    'research.screens.update': (a) => {
      const id = String(a.id);
      const cur = find('screen', id);
      if (!cur) notFound(`tela ${id}`);
      write('Tela do screener alterada', () => {
        const next = { ...cur, updated_at: nowUtc() };
        if (a.name !== undefined) next.name = checkName(a.name, id);
        if (a.description !== undefined)
          next.description =
            typeof a.description === 'string' && a.description.trim() ? a.description.trim() : null;
        if (a.filters !== undefined) next.query = normQuery(a.filters);
        if (typeof a.alert === 'boolean') next.alert = a.alert;
        put('screen', next);
      });
      return getScreen(id);
    },
    'research.screens.delete': (a) => {
      const id = String(a.id);
      if (!find('screen', id)) notFound(`tela ${id}`);
      write('Tela do screener removida', () => del('screen', id));
      return null;
    },
    'research.compare': (a) => compare(a),
    'research.dividend_radar': () => ({
      horizon_end: addMonths(mockToday(), 12),
      assets: [],
      written: 0,
      removed: 0,
    }),
    'research.refresh_universe': () => ({ universe: universe().length, as_of: mockToday() }),
    'research.sync': (a) => {
      const sources = Array.isArray(a.sources) ? (a.sources as SyncSource[]) : null;
      const report = syncReport(a.force === true, sources);
      state.syncs += 1;
      state.empty = false;
      state.lastSync = report.finished_at;
      state.lastReport = report;
      return report;
    },
  }),
);
