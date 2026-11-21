/**
 * Rótulos, tons e formatação do score de saúde por ativo (`score.*`). Não confundir com a
 * saúde financeira da família (`features/health`): aqui a nota é de uma empresa ou fundo.
 *
 * Unidades do núcleo: notas de 0 a 100 (uma casa; mostradas inteiras), confiança e
 * cobertura em fração, métricas pela unidade (`percent` em fração, `multiple` em vezes,
 * `money` em reais, não centavos).
 */
import type { AssetScoreMetric } from '@/api/generated/AssetScoreMetric';
import type { AssetScoreStatus } from '@/api/generated/AssetScoreStatus';
import type { ScoreFlagSeverity } from '@/api/generated/ScoreFlagSeverity';
import type { ScoreUnit } from '@/api/generated/ScoreUnit';
import { t } from '@/i18n';
import { formatNumber, formatPercent, MINUS } from '@/lib/format';

import { compactAmount, formatMultiple } from '../research/format';

export type BandKey = 'robusto' | 'saudavel' | 'atencao' | 'risco' | 'dados_insuficientes';

/** Faixa: sinal (nunca só a cor) e as classes de cor; o rótulo vem do catálogo na hora. */
export const BAND: Record<
  BandKey,
  {
    readonly label: string;
    mark: string;
    text: string;
    chip: string;
    bar: string;
    min: number | null;
  }
> = {
  robusto: {
    get label() {
      return t('score-faixa-robusto');
    },
    mark: '▲',
    text: 'text-positive',
    chip: 'border-positive/50 bg-positive-soft text-positive',
    bar: 'bg-positive',
    min: 75,
  },
  saudavel: {
    get label() {
      return t('score-faixa-saudavel');
    },
    mark: '✓',
    text: 'text-accent',
    chip: 'border-accent/50 bg-accent-soft text-accent',
    bar: 'bg-accent',
    min: 60,
  },
  atencao: {
    get label() {
      return t('score-faixa-atencao');
    },
    mark: '!',
    text: 'text-warning',
    chip: 'border-warning/50 bg-warning-soft text-warning',
    bar: 'bg-warning',
    min: 40,
  },
  risco: {
    get label() {
      return t('score-faixa-risco');
    },
    mark: '▼',
    text: 'text-negative',
    chip: 'border-negative/50 bg-negative-soft text-negative',
    bar: 'bg-negative',
    min: 0,
  },
  dados_insuficientes: {
    get label() {
      return t('score-dados-insuficientes');
    },
    mark: '—',
    text: 'text-dim',
    chip: 'border-line-strong text-dim',
    bar: 'bg-line-strong',
    min: null,
  },
};

export const BAND_ORDER: readonly BandKey[] = ['robusto', 'saudavel', 'atencao', 'risco'];

export function bandKey(band: string | null | undefined): BandKey {
  return band && band in BAND ? (band as BandKey) : 'dados_insuficientes';
}

/** Faixa de uma nota de 0 a 100 (para colorir barras de dimensão e métrica). */
export function bandOfScore(score: number | null | undefined): BandKey {
  if (score == null) return 'dados_insuficientes';
  return BAND_ORDER.find((k) => score >= (BAND[k].min ?? 0)) ?? 'risco';
}

const STATUS_KEY = {
  ok: 'score-status-calculado',
  partial: 'score-status-parcial',
  insufficient_data: 'score-dados-insuficientes',
  not_supported: 'score-status-nao-suportada',
} as const satisfies Record<AssetScoreStatus, string>;

/** Rótulo de cada situação do score, lido do catálogo a cada acesso. */
export const STATUS_LABEL = Object.defineProperties(
  {} as Record<AssetScoreStatus, string>,
  Object.fromEntries(
    Object.entries(STATUS_KEY).map(([k, key]) => [k, { enumerable: true, get: () => t(key) }]),
  ),
);

const SEVERITY_KEY = {
  info: 'score-severidade-informativa',
  low: 'score-severidade-baixa',
  medium: 'score-severidade-media',
  high: 'score-severidade-alta',
  critical: 'score-severidade-critica',
} as const satisfies Record<ScoreFlagSeverity, string>;

const SEVERITY_TEXT: Record<ScoreFlagSeverity, string> = {
  info: 'text-dim',
  low: 'text-dim',
  medium: 'text-warning',
  high: 'text-negative',
  critical: 'text-negative',
};

export const SEVERITY: Record<ScoreFlagSeverity, { readonly label: string; text: string }> =
  Object.fromEntries(
    (Object.keys(SEVERITY_KEY) as ScoreFlagSeverity[]).map((k) => [
      k,
      {
        get label() {
          return t(SEVERITY_KEY[k]);
        },
        text: SEVERITY_TEXT[k],
      },
    ]),
  ) as Record<ScoreFlagSeverity, { readonly label: string; text: string }>;

/** Nota inteira: `69.7` → `70`; sem nota, `—`. */
export const formatScore = (s: number | null | undefined) =>
  s == null || !Number.isFinite(s) ? '—' : String(Math.round(s));

/** Confiança em fração → `76%`. */
export const formatConfidence = (c: number) => formatPercent(c, { decimals: 0 });

/** Variação em pontos com sinal: `+2,4` / `−10,6`. */
export function formatDelta(d: number | null | undefined): string {
  if (d == null || !Number.isFinite(d)) return '—';
  const abs = formatNumber(Math.abs(d), 1);
  return d > 0 ? `+${abs}` : d < 0 ? `${MINUS}${abs}` : '0';
}

/** Valor de uma métrica pela unidade do núcleo. */
export function formatMetricValue(value: number | null | undefined, unit: ScoreUnit): string {
  if (value == null || !Number.isFinite(value)) return '—';
  switch (unit) {
    case 'percent':
      return formatPercent(value, { decimals: 1 });
    case 'multiple':
      return formatMultiple(value);
    case 'score':
      return formatNumber(value, Number.isInteger(value) ? 0 : 2);
    case 'count':
      return formatNumber(value, 0);
    case 'days':
      return t('score-dias', { n: Math.round(value), valor: formatNumber(value, 0) });
    case 'years':
      return t('score-anos', { n: value, valor: formatNumber(value, 1) });
    case 'money':
      // Como no núcleo (`score::engine::fmt_value`): os fundamentos do score são da CVM, em reais.
      return compactAmount(value, 'BRL');
  }
}

/** "p92 de 23 pares do setor X"; sem pares, `null`. */
export function percentileText(m: AssetScoreMetric): string | null {
  if (m.percentile == null) return null;
  const p = `p${Math.round(m.percentile)}`;
  if (m.peers == null) return p;
  // O nível dos pares ("do setor X") vem do núcleo.
  const text = t('score-percentil-pares', { p, n: m.peers });
  return m.peer_level ? `${text} ${m.peer_level}` : text;
}

/** Teto e penalidade de uma red flag: "teto 45 · −10 pontos". */
export function flagEffect(f: { cap: number | null; penalty: number | null }): string | null {
  const parts = [
    f.cap != null ? t('score-teto', { nota: formatScore(f.cap) }) : null,
    f.penalty != null
      ? t('score-penalidade', { n: f.penalty, pontos: `${MINUS}${formatNumber(f.penalty, 0)}` })
      : null,
  ].filter(Boolean);
  return parts.length ? parts.join(' · ') : null;
}
