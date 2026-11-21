/**
 * Mock de `score.*`: score de saúde por ativo (metodologia 0.1.0) com as mesmas formas e
 * regras do núcleo (`score::assess`). Ações BR saem de um molde com as nove dimensões
 * (deslocado por papel), FIIs de um molde menor, e as demais classes vêm como "não
 * suportada". Casos de propósito: MGLU3 sem uma métrica crítica (sem score), BBAS3 com
 * dimensões imputadas (parcial), WEGE3 com queda de 10 pontos (alerta), PETR4 com red
 * flag e conferência com o Fundamentus, TAEE11 com dados defasados.
 */
import type { Asset } from '@/api/generated/Asset';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { AssetScore } from '@/api/generated/AssetScore';
import type { AssetScoreChange } from '@/api/generated/AssetScoreChange';
import type { AssetScoreDimension } from '@/api/generated/AssetScoreDimension';
import type { AssetScoreEvent } from '@/api/generated/AssetScoreEvent';
import type { AssetScoreEventStatus } from '@/api/generated/AssetScoreEventStatus';
import type { AssetScoreFeedback } from '@/api/generated/AssetScoreFeedback';
import type { AssetScoreFlag } from '@/api/generated/AssetScoreFlag';
import type { AssetScoreMethodology } from '@/api/generated/AssetScoreMethodology';
import type { AssetScoreMetric } from '@/api/generated/AssetScoreMetric';
import type { AssetScoreMissing } from '@/api/generated/AssetScoreMissing';
import type { AssetScorePoint } from '@/api/generated/AssetScorePoint';
import type { AssetScoreReconciliation } from '@/api/generated/AssetScoreReconciliation';
import type { AssetScoreSegment } from '@/api/generated/AssetScoreSegment';
import type { AssetScoreSummary } from '@/api/generated/AssetScoreSummary';
import type { ScoreDirection } from '@/api/generated/ScoreDirection';
import type { ScoreFramework } from '@/api/generated/ScoreFramework';
import type { ScoreUnit } from '@/api/generated/ScoreUnit';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { allAssets, getAsset } from './assets';
import { guarded } from './ledger/boot';
import { nowUtc } from './ledger/dates';
import { invalid, notFound, text } from './ledger/errors';
import { addDays, addMonths, mockToday } from './marketSim';
import { syncedAssetIds } from './portfolio';

const VERSION = '0.1.0';
const DISCLAIMER =
  'Score calculado com dados públicos e regras da literatura, ainda não validado com o histórico (metodologia em rascunho). Não é recomendação de investimento.';
const REF_DATE = '2026-06-30';
const PERIOD = '12 meses até 30/06/2026';

// ── Taxonomia (methodology/v0.1.0/segments.json) ────────────────

const CLASS_LABEL: Record<string, string> = {
  equity_br: 'Ações (Brasil)',
  reit: 'Fundos imobiliários',
  equity_intl: 'Ações no exterior e BDRs',
  fi_sovereign: 'Tesouro Direto',
  fi_bank: 'Renda fixa bancária',
  etf: 'ETFs',
  fund: 'Fundos de investimento',
  crypto: 'Criptoativos',
  other: 'Outros',
};

function classOf(c: AssetClass): string {
  switch (c) {
    case 'stock_br':
      return 'equity_br';
    case 'fii':
    case 'reit':
      return 'reit';
    case 'stock_us':
    case 'stock_intl':
    case 'bdr':
      return 'equity_intl';
    case 'treasury_br':
      return 'fi_sovereign';
    case 'fixed_income':
      return 'fi_bank';
    case 'etf_br':
    case 'etf_us':
    case 'etf_intl':
      return 'etf';
    case 'fund':
    case 'pension':
      return 'fund';
    case 'crypto':
      return 'crypto';
    default:
      return 'other';
  }
}

const SEG = (cls: string, key: string, label: string, profile = `${cls}/${key}`) => ({
  class: cls,
  key,
  label,
  profile,
});

const SEGMENTS: AssetScoreSegment[] = [
  SEG('equity_br', 'default', 'Não financeiras (geral)'),
  SEG('equity_br', 'banks', 'Bancos e financeiras'),
  SEG('equity_br', 'insurance', 'Seguradoras'),
  SEG(
    'equity_br',
    'cyclicals',
    'Commodities (mineração, siderurgia, óleo e gás, papel e celulose)',
  ),
  SEG('equity_br', 'utilities', 'Utilities (energia elétrica e saneamento)'),
  SEG('equity_br', 'retail', 'Varejo e consumo'),
  SEG('equity_br', 'tech', 'Tecnologia e comunicação'),
  SEG('equity_br', 'health', 'Saúde'),
  SEG('equity_br', 'construction', 'Construção civil e incorporação'),
  SEG(
    'equity_br',
    'industrials',
    'Indústria, bens de capital, transporte e infraestrutura',
    'equity_br/default',
  ),
  SEG('equity_br', 'telecom', 'Telecomunicações', 'equity_br/default'),
  SEG('equity_br', 'agro', 'Agronegócio e alimentos', 'equity_br/default'),
  SEG('reit', 'tijolo', 'Tijolo'),
  SEG('reit', 'papel', 'Papel (CRI)'),
  SEG('reit', 'fof', 'Fundo de fundos'),
  SEG('reit', 'hibrido', 'Híbrido'),
];

const PROFILE_LABEL: Record<string, string> = {
  'equity_br/default': 'Ações: empresas não financeiras',
  'equity_br/banks': 'Ações: bancos',
  'equity_br/insurance': 'Ações: seguradoras',
  'equity_br/cyclicals': 'Ações: commodities e cíclicas',
  'equity_br/utilities': 'Ações: energia elétrica e saneamento (utilities)',
  'equity_br/retail': 'Ações: varejo e consumo',
  'equity_br/tech': 'Ações: tecnologia',
  'equity_br/health': 'Ações: saúde',
  'equity_br/construction': 'Ações: construção civil',
  'reit/tijolo': 'FIIs: tijolo',
  'reit/papel': 'FIIs: papel (CRI)',
  'reit/fof': 'FIIs: fundo de fundos',
  'reit/hibrido': 'FIIs: híbridos',
};

const BANDS: [key: string, label: string, min: number, meaning: string][] = [
  ['robusto', 'Robusto', 75, 'Fundamentos sólidos nos critérios medidos'],
  ['saudavel', 'Saudável', 60, 'Ok; monitorar trimestralmente'],
  ['atencao', 'Atenção', 40, 'Deterioração ou fraqueza relevante; revisar a tese'],
  ['risco', 'Risco', 0, 'Fundamentos comprometidos nos critérios medidos'],
];
const NO_BAND = { band: 'dados_insuficientes', band_label: 'Dados insuficientes' };

function bandOf(score: number) {
  const b = BANDS.find(([, , min]) => score >= min) ?? BANDS[BANDS.length - 1]!;
  return { band: b[0], band_label: b[1], band_meaning: b[3] };
}

// ── Moldes de métricas ───────────────────────────────────────────

/** Fórmulas das métricas (as mesmas de `methodology/v0.1.0/thresholds.json`). */
const FORMULA: Record<string, string> = {
  roic: 'NOPAT ÷ (dívida líquida + patrimônio líquido), NOPAT = EBIT × (1 − 34%)',
  roe: 'Lucro líquido (12 meses) ÷ patrimônio líquido médio',
  ebit_margin: 'EBIT ÷ receita líquida (12 meses)',
  net_margin: 'Lucro líquido ÷ receita líquida (12 meses)',
  net_debt_ebitda:
    '(Dívida bruta + arrendamentos − caixa) ÷ EBITDA (12 meses), EBITDA = EBIT + D&A recalculado das demonstrações',
  interest_coverage: 'EBIT ÷ despesa financeira líquida (12 meses)',
  altman_z2:
    '6,56·X1 + 3,26·X2 + 6,72·X3 + 1,05·X4 (X1 capital de giro/ativo, X2 lucros retidos/ativo, X3 EBIT/ativo, X4 PL contábil/passivo)',
  current_ratio: 'Ativo circulante ÷ passivo circulante',
  cash_to_st_debt: 'Caixa e aplicações ÷ empréstimos e financiamentos do passivo circulante',
  fcf_margin: '(FCO − capex) ÷ receita líquida (12 meses)',
  fcf_to_net_income:
    '(FCO − capex) ÷ lucro líquido (12 meses); acima de 1 é bom, valores muito altos só indicam lucro pequeno',
  accruals: '(Lucro líquido − FCO) ÷ ativo total médio',
  piotroski:
    'Soma de 9 critérios binários (rentabilidade, alavancagem/liquidez, eficiência) entre os dois últimos exercícios',
  revenue_cagr_5y_real: 'CAGR da receita líquida anual em 5 anos, descontado o IPCA do período',
  eps_cagr_5y: 'CAGR do lucro líquido por ação (lucro ÷ ações em circulação) em 5 exercícios',
  asset_turnover: 'Receita líquida (12 meses) ÷ ativo total médio',
  cash_cycle_days: 'Prazo de estoques + prazo de recebimento − prazo de pagamento',
  earnings_yield: 'EBIT (12 meses) ÷ (valor de mercado + dívida líquida + minoritários)',
  fcf_yield: 'Fluxo de caixa livre (12 meses) ÷ valor de mercado',
  earnings_to_price: 'Lucro líquido (12 meses) ÷ valor de mercado',
  pb: 'Valor de mercado ÷ patrimônio líquido',
  ev_ebitda: '(Valor de mercado + dívida líquida + minoritários) ÷ EBITDA (12 meses)',
  listing_segment:
    'Novo Mercado 100; Nível 2 80; Bovespa Mais 70; Nível 1 60; Básico/Tradicional 40',
  timely_filing:
    'Fração das últimas 4 entregas de DFP/ITR feitas no prazo da CVM (DFP: 3 meses após o exercício; ITR: 45 dias após o trimestre)',
  roe_stability: 'Desvio-padrão do ROE anual nos últimos 5 a 10 exercícios',
  loss_years_10y: 'Exercícios com lucro líquido negativo nos últimos 10 (mínimo de 3 exercícios)',
  volatility_1y: 'Desvio-padrão dos retornos diários de 1 ano × √252',
  payout: 'Dividendos e JCP declarados (12 meses) ÷ lucro líquido',
  dividend_fcf_coverage: 'Fluxo de caixa livre ÷ dividendos e JCP pagos (12 meses)',
  dilution: 'Ações em circulação ÷ ações 12 meses antes − 1 (negativo = recompra)',
  dy_12m: 'Rendimentos distribuídos (12 meses) ÷ cotação',
  income_stability: 'Desvio-padrão ÷ média dos rendimentos mensais em 24 meses',
  vacancy: 'Área vaga ÷ área bruta locável (informe trimestral)',
  tenant_concentration: 'Receita do maior inquilino ÷ receita total',
  leverage: 'Passivo total ÷ patrimônio líquido (informe mensal)',
  avg_volume: 'Volume financeiro médio dos últimos 3 meses',
  holders: 'Número de cotistas no último informe mensal',
  pvp: 'Cotação ÷ valor patrimonial por cota',
};

interface MetricSeed {
  key: string;
  label: string;
  unit: ScoreUnit;
  direction: ScoreDirection;
  weight: number;
  value: number | null;
  score: number | null;
  percentile: number | null;
  absolute: number | null;
  period?: string | null;
  missing?: string;
  note?: string;
  trend?: number;
  critical?: boolean;
}

/** Linha do molde: chave, rótulo, unidade, direção, peso, valor, nota, percentil, absoluta. */
type Row = [
  key: string,
  label: string,
  unit: ScoreUnit,
  direction: ScoreDirection,
  weight: number,
  value: number | null,
  score: number | null,
  percentile: number | null,
  absolute: number | null,
  extra?: Partial<MetricSeed>,
];

const toSeed = ([
  key,
  label,
  unit,
  direction,
  weight,
  value,
  score,
  percentile,
  absolute,
  extra,
]: Row): MetricSeed => ({
  key,
  label,
  unit,
  direction,
  weight,
  value,
  score,
  percentile,
  absolute,
  ...extra,
});

interface DimSeed {
  key: string;
  label: string;
  weight: number;
  attractiveness?: boolean;
  metrics: MetricSeed[];
}

const dim = (
  key: string,
  label: string,
  weight: number,
  rows: Row[],
  attractiveness = false,
): DimSeed => ({
  key,
  label,
  weight,
  attractiveness,
  metrics: rows.map(toSeed),
});

const SALDO = { period: 'saldo em 30/06/2026' };

// Tabela de dados: uma métrica por linha.
// prettier-ignore
const EQUITY: DimSeed[] = [
  dim('rentabilidade', 'Rentabilidade', 12, [
    ['roic', 'ROIC', 'percent', 'higher', 3, 0.186, 78, 88, 85, { trend: 0.004 }],
    ['roe', 'ROE', 'percent', 'higher', 3, 0.21, 82.5, 90, 86, { critical: true, trend: 0.006 }],
    ['ebit_margin', 'Margem EBIT', 'percent', 'higher', 1.5, 0.212, 85, 84, 90],
    ['net_margin', 'Margem líquida', 'percent', 'higher', 1.5, 0.12, 74, 72, 76, { critical: true }],
  ]),
  dim('solvencia', 'Endividamento e liquidez', 18, [
    ['net_debt_ebitda', 'Dívida líquida / EBITDA', 'multiple', 'lower', 3, 1.42, 72, 66, 78, { critical: true, trend: -0.12 }],
    ['interest_coverage', 'Cobertura de juros', 'multiple', 'higher', 3, 7.06, 88, 90, 91],
    ['altman_z2', "Altman Z''", 'score', 'higher', 1, 4.14, 86, 84, 91],
    ['current_ratio', 'Liquidez corrente', 'multiple', 'range', 1, 1.67, 100, null, 100, SALDO],
    ['cash_to_st_debt', 'Caixa ÷ dívida de curto prazo', 'multiple', 'higher', 1, 2, 65, 50, 100, SALDO],
  ]),
  dim('qualidade_lucros', 'Qualidade dos lucros', 10, [
    ['fcf_margin', 'Margem de fluxo de caixa livre', 'percent', 'higher', 2, 0.13, 80, 82, 83],
    ['fcf_to_net_income', 'Conversão de lucro em caixa (FCF/LL)', 'multiple', 'range', 2, 1.08, 100, null, 100],
    ['accruals', 'Accruals (Sloan)', 'percent', 'lower', 2, -0.044, 58, 33, 87],
    ['piotroski', 'Piotroski F-Score', 'score', 'higher', 2, 8, 86.8, 83, 95, { period: 'exercício até 31/12/2025' }],
  ]),
  dim('crescimento', 'Crescimento', 5, [
    ['revenue_cagr_5y_real', 'Crescimento real da receita (5 anos)', 'percent', 'higher', 3, 0.052, 61.8, 50, 70, { period: '2020 a 2025' }],
    ['eps_cagr_5y', 'Crescimento do lucro por ação (5 anos)', 'percent', 'higher', 2, 0.1, 61.4, 56, 75, { period: '2020 a 2025' }],
  ]),
  dim('eficiencia', 'Eficiência', 8, [
    ['asset_turnover', 'Giro do ativo', 'multiple', 'higher', 2, 0.87, 52, 44, 69],
    ['cash_cycle_days', 'Ciclo de caixa', 'days', 'lower', 1, 48, 70, 64, 78],
  ]),
  dim('valuation', 'Valuation', 12, [
    ['earnings_yield', 'Earnings yield (EBIT/EV)', 'percent', 'higher', 3, 0.094, 52, 48, 60],
    ['fcf_yield', 'FCF yield', 'percent', 'higher', 2, 0.061, 50, 46, 58],
    ['earnings_to_price', 'Lucro ÷ preço (inverso do P/L)', 'percent', 'higher', 1, 0.083, 48, 44, 55],
    ['pb', 'P/VP', 'multiple', 'lower', 1, 1.9, 46, 40, 52],
    ['ev_ebitda', 'EV/EBITDA', 'multiple', 'lower', 1, 6.4, 50, 46, 55],
  ], true),
  dim('governanca', 'Governança', 10, [
    ['listing_segment', 'Segmento de listagem (B3)', 'score', 'higher', 3, 100, 100, null, 100, { period: null, note: 'Novo Mercado' }],
    ['timely_filing', 'Entregas no prazo (DFP/ITR)', 'percent', 'higher', 1, null, null, null, null, { period: null, missing: 'sem datas de entrega das demonstrações' }],
  ]),
  dim('estabilidade', 'Estabilidade', 15, [
    ['roe_stability', 'Volatilidade do ROE (desvio-padrão anual)', 'percent', 'lower', 3, 0.034, 62, 58, 80, { period: '10 exercícios' }],
    ['loss_years_10y', 'Anos com prejuízo (até 10 anos)', 'count', 'lower', 2, 1, 70, 60, 80, { period: '10 exercícios', critical: true }],
    ['volatility_1y', 'Volatilidade anual da cotação', 'percent', 'lower', 1, 0.27, 55, 52, 60, { period: '12 meses até hoje' }],
  ]),
  dim('remuneracao', 'Remuneração ao acionista', 10, [
    ['payout', 'Payout', 'percent', 'range', 2, 0.5, 100, null, 100],
    ['dividend_fcf_coverage', 'Cobertura dos dividendos pelo caixa livre', 'multiple', 'range', 2, 2.17, 100, null, 100],
    ['dilution', 'Diluição líquida (1 ano)', 'percent', 'lower', 1, 0, 60.5, 50, 85, SALDO],
  ]),
];

// prettier-ignore
const REIT: DimSeed[] = [
  dim('qualidade_renda', 'Qualidade da renda', 25, [
    ['dy_12m', 'Dividend yield (12 meses)', 'percent', 'higher', 3, 0.094, 72, 70, 74],
    ['income_stability', 'Estabilidade dos rendimentos', 'percent', 'lower', 2, 0.06, 80, 78, 82, { period: '24 meses', critical: true }],
  ]),
  dim('qualidade_portfolio', 'Qualidade do portfólio', 25, [
    ['vacancy', 'Vacância física', 'percent', 'lower', 3, 0.052, 76, 72, 80, SALDO],
    ['tenant_concentration', 'Concentração no maior inquilino', 'percent', 'lower', 1, 0.18, 70, 64, 72],
  ]),
  dim('alavancagem', 'Alavancagem', 15, [
    ['leverage', 'Passivo ÷ patrimônio líquido', 'percent', 'lower', 1, 0.08, 88, 80, 92, { ...SALDO, critical: true }],
  ]),
  dim('liquidez', 'Liquidez', 15, [
    ['avg_volume', 'Volume médio diário', 'money', 'higher', 1, 9_800_000, 82, 86, 80, { period: '3 meses até hoje' }],
    ['holders', 'Cotistas', 'count', 'higher', 1, 420_000, 90, 92, 88, { period: 'informe de junho/2026' }],
  ]),
  dim('valuation', 'Valuation', 20, [
    ['pvp', 'P/VP', 'multiple', 'lower', 1, 0.97, 58, 52, 62],
  ], true),
];

const PIOTROSKI_ITEMS: [string, boolean][] = [
  ['ROA positivo', true],
  ['Fluxo de caixa operacional positivo', true],
  ['ROA maior que no ano anterior', true],
  ['Caixa operacional ÷ ativo maior que o ROA (qualidade do accrual)', true],
  ['Alavancagem de longo prazo menor', true],
  ['Liquidez corrente maior', false],
  ['Não emitiu ações', true],
  ['Margem bruta maior', true],
  ['Giro do ativo maior', true],
];

function equityFrameworks(): ScoreFramework[] {
  return [
    {
      key: 'piotroski',
      label: 'Piotroski F-Score',
      value: 8,
      reading: 'forte',
      reference: 'Piotroski (2000), Journal of Accounting Research',
      note: null,
      items: PIOTROSKI_ITEMS.map(([label, passed]) => ({ label, value: passed ? 1 : 0, passed })),
    },
    {
      key: 'altman_z2',
      label: "Altman Z''",
      value: 4.14,
      reading: 'zona segura',
      reference: 'Altman (1968); Altman, Hartzell e Peck (1995)',
      note: null,
      items: [
        { label: 'X1 capital de giro ÷ ativo', value: 0.167, passed: null },
        { label: 'X2 lucros retidos ÷ ativo', value: 0.25, passed: null },
        { label: 'X3 EBIT ÷ ativo', value: 0.177, passed: null },
        { label: 'X4 PL contábil ÷ passivo', value: 1, passed: null },
      ],
    },
    {
      key: 'beneish',
      label: 'Beneish M-Score',
      value: -2.59,
      reading: 'sem sinal de manipulação',
      reference: 'Beneish (1999), Financial Analysts Journal',
      note: null,
      items: [
        { label: 'DSRI recebíveis/receita', value: 1, passed: null },
        { label: 'GMI margem bruta', value: 1, passed: null },
        { label: 'AQI qualidade do ativo', value: 1, passed: null },
        { label: 'SGI crescimento da receita', value: 1.1, passed: null },
        { label: 'TATA accruals totais', value: -0.042, passed: null },
      ],
    },
    {
      key: 'magic_formula',
      label: 'Magic Formula (Greenblatt)',
      value: null,
      reading: null,
      reference: 'Greenblatt (2005), The Little Book That Beats the Market',
      note: 'Diagnóstico: o ROC entra em Rentabilidade e o EY em Valuation',
      items: [
        {
          label: 'Retorno sobre o capital (EBIT ÷ capital de giro + imobilizado)',
          value: 0.326,
          passed: null,
        },
        { label: 'Earnings yield (EBIT ÷ EV)', value: 0.094, passed: null },
      ],
    },
  ];
}

// ── Casos por papel ──────────────────────────────────────────────

interface Case {
  segment: string;
  /** Deslocamento das notas das métricas (e dos percentis) em relação ao molde. */
  shift: number;
  /** Variação desde o snapshot anterior. */
  delta?: number;
  flags?: AssetScoreFlag[];
  reconciliation?: AssetScoreReconciliation[];
  /** Dimensões sem cobertura mínima: nota imputada (mediana dos pares × 0,9). */
  imputed?: string[];
  /** Métrica crítica sem dado: sem score. */
  missingCritical?: { key: string; reason: string };
  /** Métricas com a pior nota por denominador negativo. */
  worst?: { key: string; note: string }[];
  stale?: boolean;
}

const CASES: Record<string, Case> = {
  PETR4: {
    segment: 'cyclicals',
    shift: -6,
    delta: 2.4,
    flags: [
      {
        key: 'state_control',
        label: 'Controle estatal',
        severity: 'medium',
        cap: null,
        penalty: 5,
        detail: 'A União detém a maioria das ações ordinárias (FRE 2026).',
      },
    ],
    reconciliation: [
      {
        metric: 'roe',
        label: 'ROE',
        ours: 0.212,
        theirs: 0.205,
        diff: 0.034,
        source: 'Fundamentus',
      },
      {
        metric: 'net_debt_ebitda',
        label: 'Dívida líquida / EBITDA',
        ours: 1.42,
        theirs: 1.18,
        diff: 0.203,
        source: 'Fundamentus',
      },
    ],
  },
  ITUB4: { segment: 'banks', shift: 8 },
  WEGE3: { segment: 'industrials', shift: 10, delta: -10.6 },
  BBAS3: {
    segment: 'banks',
    shift: 0,
    imputed: ['crescimento', 'estabilidade', 'remuneracao', 'eficiencia'],
  },
  MGLU3: {
    segment: 'retail',
    shift: -22,
    missingCritical: {
      key: 'net_debt_ebitda',
      reason: 'EBITDA dos últimos 12 meses sem a nota de depreciação e amortização',
    },
    worst: [{ key: 'ev_ebitda', note: 'EBITDA negativo: recebe a pior nota' }],
  },
  VALE3: { segment: 'cyclicals', shift: 2 },
  TAEE11: { segment: 'utilities', shift: 4, stale: true },
};

const SECTOR_OF_SEGMENT: Record<string, string> = {
  banks: 'Bancos',
  cyclicals: 'Petróleo e Gás',
  utilities: 'Energia Elétrica',
  retail: 'Comércio (Atacado e Varejo)',
  industrials: 'Máquinas, Equipamentos, Veículos e Peças',
};

// ── Estado ───────────────────────────────────────────────────────

interface State {
  /** Testes: nenhum ativo com posição ou na watchlist. */
  empty: boolean;
  overrides: Record<string, string>;
  feedback: AssetScoreFeedback[];
  events: AssetScoreEvent[];
  seq: number;
}

const initial = (): State => ({ empty: false, overrides: {}, feedback: [], events: [], seq: 0 });

const EVENT_KINDS: [string, string, number | null, number | null][] = [
  ['recuperacao_judicial', 'Recuperação judicial ou extrajudicial / default', 20, null],
  ['fraude', 'Fraude contábil ou reapresentação de demonstrações', 25, null],
  ['litigio', 'Litígio relevante', null, 10],
  ['ambiental', 'Desastre ambiental ou ESG', null, 15],
  [
    'governanca',
    'Governança (troca abrupta de CEO/CFO, partes relacionadas, interferência)',
    null,
    5,
  ],
  ['regulacao', 'Regulação (tarifa, concessão, intervenção em preços)', null, 5],
  ['outro', 'Outro evento relevante', null, 5],
];

function withEffect(e: AssetScoreEvent): AssetScoreEvent {
  const on = e.status === 'accepted';
  return { ...e, effective_cap: on ? e.cap : null, effective_penalty: on ? e.penalty : null };
}
let state = initial();
onMockReset(() => {
  state = initial();
});

/** Testes: `score.portfolio` sem nenhum ativo (estado vazio da Carteira › Saúde dos ativos). */
export function mockScoreEmpty(): void {
  state.empty = true;
}

// ── Cálculo ──────────────────────────────────────────────────────

const clamp = (v: number) => Math.max(0, Math.min(100, v));
const r1 = (v: number) => Math.round(v * 10) / 10;

function weighted(items: { w: number; v: number | null }[]): number | null {
  const has = items.filter((i) => i.v != null);
  const total = has.reduce((s, i) => s + i.w, 0);
  if (total === 0) return null;
  return has.reduce((s, i) => s + i.w * i.v!, 0) / total;
}

const pctText = (v: number) => `${(v * 100).toFixed(1).replace('.', ',')}%`;
const valueText = (x: AssetScoreMetric) =>
  x.value == null
    ? '—'
    : x.unit === 'percent'
      ? pctText(x.value)
      : x.unit === 'multiple'
        ? `${x.value.toFixed(2).replace('.', ',')}x`
        : String(x.note ?? x.value);

function subjectOf(a: Record<string, unknown>): Asset {
  const id = typeof a.asset_id === 'string' && a.asset_id.trim() ? a.asset_id : null;
  const ticker =
    typeof a.ticker === 'string' && a.ticker.trim() ? a.ticker.trim().toUpperCase() : null;
  if (!id && !ticker) invalid('informe asset_id ou ticker');
  if (id) return getAsset(id);
  const found = allAssets().find((x) => x.ticker?.toUpperCase() === ticker);
  if (!found) notFound(`${ticker} no universo baixado (rode a coleta da análise de ativos)`);
  return found;
}

function shell(a: Asset, cls: string): AssetScore {
  return {
    asset_id: a.id,
    ticker: a.ticker,
    name: a.name,
    asset_class: cls,
    class_label: CLASS_LABEL[cls] ?? cls,
    segment: '',
    segment_label: '',
    profile: '',
    profile_label: '',
    sector: a.sector ?? null,
    as_of: mockToday(),
    methodology_version: VERSION,
    status: 'not_supported',
    score: null,
    health_raw: null,
    events: state.events.filter((e) => e.asset_id === a.id).map(withEffect),
    event_suggestions: [],
    ...NO_BAND,
    band_meaning: '',
    confidence: 0,
    health: null,
    attractiveness: null,
    raw_score: null,
    imputed_share: 0,
    dimensions: [],
    frameworks: [],
    flags: [],
    missing: [],
    top_drivers: [],
    top_detractors: [],
    peers: null,
    data_date: null,
    stale: false,
    reconciliation: [],
    change: null,
    notes: [],
    disclaimer: DISCLAIMER,
  };
}

function segmentInfo(cls: string, key: string) {
  const s = SEGMENTS.find((x) => x.class === cls && x.key === key) ?? SEGMENTS[0]!;
  return { segment: s.key, segment_label: s.label, profile: s.profile };
}

function buildMetric(
  seed: MetricSeed,
  c: Case,
  peerLevel: string,
  peers: number,
): AssetScoreMetric {
  const missing = c.missingCritical?.key === seed.key ? c.missingCritical.reason : seed.missing;
  const worst = c.worst?.find((w) => w.key === seed.key);
  const has = !missing;
  return {
    key: seed.key,
    label: seed.label,
    unit: seed.unit,
    direction: seed.direction,
    formula: FORMULA[seed.key] ?? '',
    weight: seed.weight,
    value: has ? (worst && seed.value != null ? -Math.abs(seed.value) : seed.value) : null,
    score: !has || seed.score == null ? null : worst ? 0 : r1(clamp(seed.score + c.shift)),
    percentile:
      !has || seed.percentile == null
        ? null
        : worst
          ? 0
          : Math.round(clamp(seed.percentile + c.shift)),
    absolute:
      !has || seed.absolute == null
        ? null
        : worst
          ? 0
          : Math.round(clamp(seed.absolute + c.shift / 2)),
    peers: has && seed.percentile != null ? peers : null,
    peer_level: has && seed.percentile != null ? peerLevel : null,
    average: has && seed.trend != null && seed.value != null ? seed.value * 0.97 : null,
    trend: has ? (seed.trend ?? null) : null,
    period: has ? (seed.period === undefined ? PERIOD : seed.period) : null,
    source: has ? 'exercício: DFP 2025-12-31; ITR 2026-06-30' : null,
    note: worst?.note ?? seed.note ?? null,
    missing: missing ?? null,
    critical: Boolean(seed.critical),
    worst: Boolean(worst),
  };
}

function scoreOf(a: Asset): AssetScore {
  const cls = classOf(a.class);
  const s = shell(a, cls);
  if (cls !== 'equity_br' && cls !== 'reit') {
    s.band_meaning = `O score de ${s.class_label.toLowerCase()} ainda não foi implementado (primeira entrega: ações BR, bancos e FIIs).`;
    return s;
  }
  const ticker = a.ticker ?? '';
  const c: Case | undefined =
    cls === 'reit'
      ? { segment: ticker === 'MXRF11' ? 'papel' : 'tijolo', shift: ticker === 'MXRF11' ? -8 : 2 }
      : CASES[ticker];
  const seg = segmentInfo(
    cls,
    state.overrides[ticker] ?? c?.segment ?? (cls === 'reit' ? 'hibrido' : 'default'),
  );
  Object.assign(s, seg, { profile_label: PROFILE_LABEL[seg.profile] ?? '' });
  if (!c) {
    s.status = 'insufficient_data';
    s.band_meaning = 'Sem score: sem demonstrações da CVM baixadas para este papel.';
    s.missing.push({
      key: 'source_data',
      label: 'Demonstrações financeiras',
      dimension: 'Dados',
      reason: 'sem demonstrações da CVM baixadas para este papel',
      critical: true,
    });
    return s;
  }

  const sector =
    cls === 'reit'
      ? (a.segment ?? 'Logística')
      : (SECTOR_OF_SEGMENT[c.segment] ?? a.sector ?? null);
  const peers = cls === 'reit' ? 12 : 9;
  const peerLevel = `do setor ${sector}`;
  s.sector = sector;
  s.peers = {
    sector,
    segment: seg.segment,
    segment_label: seg.segment_label,
    sector_size: peers,
    segment_size: peers + 5,
    class_size: cls === 'reit' ? 38 : 64,
  };
  s.data_date = c.stale ? '2025-12-31' : REF_DATE;
  s.stale = Boolean(c.stale);
  if (c.stale) {
    s.notes.push('Dados defasados: os mais recentes são de 31/12/2025 (mais de 2 trimestres).');
  }

  const seeds = cls === 'reit' ? REIT : EQUITY;
  const dims: AssetScoreDimension[] = seeds.map((d) => {
    const metrics = d.metrics.map((x) => buildMetric(x, c, peerLevel, peers));
    const total = metrics.reduce((t, x) => t + x.weight, 0);
    const covered = metrics.filter((x) => x.score != null).reduce((t, x) => t + x.weight, 0);
    const imputed = Boolean(c.imputed?.includes(d.key));
    const own = weighted(metrics.map((x) => ({ w: x.weight, v: x.score })));
    return {
      key: d.key,
      label: d.label,
      weight: d.weight,
      score: imputed ? 49.5 : own == null ? null : r1(own),
      coverage: imputed ? 0.4 : total ? covered / total : 0,
      imputed,
      attractiveness: Boolean(d.attractiveness),
      metrics,
    };
  });
  s.dimensions = dims;
  s.imputed_share = dims.filter((d) => d.imputed).reduce((t, d) => t + d.weight, 0) / 100;

  const all = dims.flatMap((d) => d.metrics.map((x) => ({ ...x, dimension: d.label })));
  s.missing = all
    .filter((x) => x.missing)
    .map((x): AssetScoreMissing => ({
      key: x.key,
      label: x.label,
      dimension: x.dimension,
      reason: x.missing!,
      critical: x.critical,
    }))
    .sort((x, y) => Number(y.critical) - Number(x.critical));
  if (cls === 'equity_br' && seg.segment !== 'banks') s.frameworks = equityFrameworks();

  const ranked = all.filter((x) => x.score != null).sort((x, y) => y.score! - x.score!);
  const why = (x: AssetScoreMetric) =>
    x.percentile != null
      ? `${x.label} ${valueText(x)}: p${x.percentile} entre ${x.peers} pares ${x.peer_level}`
      : `${x.label} ${valueText(x)}: nota ${Math.round(x.score!)} pela referência absoluta`;
  s.top_drivers = ranked.slice(0, 3).map(why);
  s.top_detractors = ranked.slice(-3).reverse().map(why);

  const critical = s.missing.filter((x) => x.critical);
  s.confidence =
    r1(
      Math.max(0.2, 0.82 - s.imputed_share * 0.6 - (c.stale ? 0.12 : 0) - critical.length * 0.2) *
        100,
    ) / 100;
  if (critical.length > 0) {
    s.status = 'insufficient_data';
    s.band_meaning = `Sem score: falta ${critical.map((x) => x.label).join(', ')} (métrica crítica).`;
    s.dimensions = dims.map((d) => ({ ...d, score: null }));
    s.top_drivers = [];
    s.top_detractors = [];
    return s;
  }

  const total = weighted(dims.map((d) => ({ w: d.weight, v: d.score })))!;
  const health = weighted(
    dims.filter((d) => !d.attractiveness).map((d) => ({ w: d.weight, v: d.score })),
  );
  const attract = weighted(
    dims.filter((d) => d.attractiveness).map((d) => ({ w: d.weight, v: d.score })),
  );
  s.raw_score = r1(total);
  s.flags = c.flags ?? [];
  let final = total;
  for (const f of s.flags) {
    if (f.penalty != null) final -= f.penalty;
    if (f.cap != null) final = Math.min(final, f.cap);
  }
  s.score = r1(clamp(final));
  s.health = health == null ? null : r1(health);
  s.attractiveness = attract == null ? null : r1(attract);
  if (s.imputed_share > 0.3) {
    s.status = 'partial';
    s.band_meaning = `Score parcial: ${Math.round(s.imputed_share * 100)}% do peso veio de notas imputadas pela mediana dos pares; sem faixa até os dados chegarem.`;
  } else {
    s.status = 'ok';
    Object.assign(s, bandOf(s.score));
  }
  s.reconciliation = c.reconciliation ?? [];
  if (c.delta != null) s.change = changeOf(s, c.delta);
  return s;
}

function changeOf(s: AssetScore, delta: number): AssetScoreChange {
  const prevScore = r1(s.score! - delta);
  const prevBand = bandOf(prevScore).band;
  const previous_as_of = addDays(s.as_of, -30);
  const alerts: string[] = [];
  if (delta <= -10) {
    alerts.push(
      `Queda de ${Math.abs(delta).toFixed(1).replace('.', ',')} pontos desde ${previous_as_of.split('-').reverse().join('/')}`,
    );
  }
  if (prevBand !== s.band)
    alerts.push(`Mudou de faixa: ${bandOf(prevScore).band_label} → ${s.band_label}`);
  return {
    previous_as_of,
    previous_score: prevScore,
    previous_band: prevBand,
    delta,
    band_changed: prevBand !== s.band,
    new_flags: [],
    alerts,
  };
}

function summaryOf(s: AssetScore): AssetScoreSummary {
  return {
    asset_id: s.asset_id,
    ticker: s.ticker,
    name: s.name,
    asset_class: s.asset_class,
    segment_label: s.segment_label,
    status: s.status,
    score: s.score,
    band: s.band,
    band_label: s.band_label,
    confidence: s.confidence,
    health: s.health,
    attractiveness: s.attractiveness,
    delta: s.change?.delta ?? null,
    flags: s.flags.map((f) => f.label),
    missing_critical: s.missing.filter((x) => x.critical).map((x) => x.label),
    alerts: s.change?.alerts ?? [],
    error: null,
  };
}

const RANK: Record<AssetScore['status'], number> = {
  ok: 0,
  partial: 0,
  insufficient_data: 1,
  not_supported: 2,
};

function portfolio(): AssetScoreSummary[] {
  if (state.empty) return [];
  const known = new Set(allAssets().map((a) => a.id));
  return syncedAssetIds()
    .filter((id) => known.has(id))
    .map((id) => summaryOf(scoreOf(getAsset(id))))
    .sort((a, b) => RANK[a.status] - RANK[b.status] || (b.score ?? -1) - (a.score ?? -1));
}

function history(s: AssetScore): AssetScorePoint[] {
  if (s.status === 'not_supported') return [];
  // Variação de cada mês para o seguinte; a última é a do snapshot anterior até hoje.
  const steps = [1.5, -2.6, 1.1, s.change?.delta ?? 0.8];
  const scores: (number | null)[] = [s.score];
  for (let i = steps.length - 1; i >= 0; i--) {
    const next = scores[0];
    scores.unshift(next == null ? null : r1(next - steps[i]!));
  }
  return scores.map((score, i) => {
    const back = scores.length - 1 - i;
    return {
      as_of: back === 0 ? s.as_of : addMonths(s.as_of, -back),
      methodology_version: VERSION,
      score,
      band: score != null && s.status === 'ok' ? bandOf(score).band : s.band,
      confidence: s.confidence,
    };
  });
}

function methodology(): AssetScoreMethodology {
  const profile = (key: string, label: string, seeds: DimSeed[]) => ({
    key,
    label,
    dimensions: seeds.map((d) => ({
      key: d.key,
      label: d.label,
      weight: d.weight,
      attractiveness: Boolean(d.attractiveness),
      metrics: d.metrics.map((x) => ({
        key: x.key,
        label: x.label,
        weight: x.weight,
        formula: FORMULA[x.key] ?? '',
        critical: Boolean(x.critical),
        unavailable: null,
      })),
    })),
  });
  return {
    version: VERSION,
    profiles: [
      ...Object.entries(PROFILE_LABEL)
        .filter(([k]) => k.startsWith('equity_br/'))
        .map(([k, l]) => profile(k, l, EQUITY)),
      ...Object.entries(PROFILE_LABEL)
        .filter(([k]) => k.startsWith('reit/'))
        .map(([k, l]) => profile(k, l, REIT)),
    ],
    segments: SEGMENTS,
    bands: BANDS.map(([, label, min, meaning]) => [label, min, meaning]),
    overrides: { ...state.overrides },
    event_kinds: EVENT_KINDS.map(([k, l]) => [k, l]),
  };
}

registerMockHandlers(
  guarded({
    'score.get': (a) => scoreOf(subjectOf(a)),
    'score.portfolio': () => portfolio(),
    'score.assets': (a) => {
      const ids = Array.isArray(a.asset_ids) ? (a.asset_ids as unknown[]) : [];
      const known = new Set(allAssets().map((x) => x.id));
      return ids
        .filter((id): id is string => typeof id === 'string' && known.has(id))
        .map((id) => summaryOf(scoreOf(getAsset(id))));
    },
    'score.history': (a) => history(scoreOf(subjectOf(a))),
    'score.methodology': () => methodology(),
    'score.set_segment': (a) => {
      const ticker = typeof a.ticker === 'string' ? a.ticker.trim().toUpperCase() : '';
      if (!ticker) invalid('informe o ticker');
      const cls = text(a.class);
      if (cls !== 'equity_br' && cls !== 'reit') invalid(`classe "${cls}" sem segmentos`);
      if (a.segment == null) {
        delete state.overrides[ticker];
      } else {
        const seg = text(a.segment);
        if (!SEGMENTS.some((x) => x.class === cls && x.key === seg)) {
          invalid(`segmento "${seg}" não existe para esta classe`);
        }
        state.overrides[ticker] = seg;
      }
      return methodology();
    },
    'score.feedback.add': (a) => {
      const asset = subjectOf(a);
      const verdict = text(a.verdict);
      if (verdict !== 'discordo' && verdict !== 'concordo') {
        invalid('veredito deve ser "discordo" ou "concordo"');
      }
      const reason = typeof a.reason === 'string' ? a.reason.trim() : '';
      if (!reason) invalid('conte o motivo');
      const expected =
        typeof a.expected_band === 'string' && a.expected_band ? a.expected_band : null;
      if (expected && !BANDS.some(([k]) => k === expected))
        invalid(`faixa "${expected}" desconhecida`);
      const s = scoreOf(asset);
      const id = `sfb-${++state.seq}`;
      state.feedback.unshift({
        id,
        subject: asset.id,
        asset_id: asset.id,
        ticker: asset.ticker,
        as_of: s.as_of,
        methodology_version: VERSION,
        score: s.score,
        band: s.band,
        verdict,
        expected_band: expected,
        reason,
        created_at: nowUtc(),
      });
      return id;
    },
    'score.events.list': (a) => {
      const asset = subjectOf(a);
      return state.events.filter((e) => e.asset_id === asset.id).map(withEffect);
    },
    'score.events.add': (a) => {
      const asset = subjectOf(a);
      const kind = EVENT_KINDS.find(([k]) => k === text(a.kind));
      if (!kind) invalid(`tipo de evento desconhecido: ${text(a.kind)}`);
      const title = typeof a.title === 'string' ? a.title.trim() : '';
      if (!title) invalid('descreva o evento');
      const id = `sev-${++state.seq}`;
      state.events.unshift({
        id,
        asset_id: asset.id,
        ticker: asset.ticker,
        kind: kind[0],
        kind_label: kind[1],
        title,
        event_date: text(a.event_date),
        severity: kind[2] != null ? 'critical' : 'medium',
        cap: kind[2],
        penalty: kind[3],
        evidence_url: typeof a.evidence_url === 'string' ? a.evidence_url : null,
        source: typeof a.source === 'string' ? a.source : 'user',
        status: a.accepted === false ? 'pending' : 'accepted',
        resolved_at: null,
        note: typeof a.note === 'string' ? a.note : null,
        effective_cap: null,
        effective_penalty: null,
      });
      return id;
    },
    'score.events.set_status': (a) => {
      const e = state.events.find((x) => x.id === text(a.id));
      if (!e) notFound('evento');
      e.status = text(a.status) as AssetScoreEventStatus;
      e.resolved_at = e.status === 'resolved' ? nowUtc() : null;
      return null;
    },
    'score.events.delete': (a) => {
      const before = state.events.length;
      state.events = state.events.filter((x) => x.id !== text(a.id));
      if (state.events.length === before) notFound('evento');
      return null;
    },
    'score.feedback.list': (a) => {
      const asset = a.asset_id || a.ticker ? subjectOf(a) : null;
      return state.feedback.filter((f) => !asset || f.asset_id === asset.id);
    },
  }),
);
