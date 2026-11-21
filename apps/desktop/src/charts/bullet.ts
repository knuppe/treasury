/**
 * Geometria do bullet previsto × realizado (função pura): barra do realizado, segmento do
 * comprometido (previstos ainda não efetivados), marca do previsto e marca do ritmo (quanto
 * já "deveria" ter saído pela fração do mês decorrida).
 */
import { t } from '@/i18n';

export interface BulletInput {
  planned: number;
  actual: number;
  /** Previstos do mês ainda não efetivados. */
  committed: number;
  /** Fração do mês decorrida (0–1). */
  elapsed: number;
}

/**
 * - `over`: o realizado passou do previsto ("Estourou").
 * - `risk`: realizado + comprometido (previstos já lançados) passa do previsto ("Vai estourar").
 * - `pace`: gasto mais rápido que o mês (informativo: contas fixas costumam vir no começo).
 */
export type BulletStatus = 'none' | 'ok' | 'pace' | 'risk' | 'over';

export interface BulletGeometry {
  /** Larguras em % da trilha. */
  actual: number;
  committed: number;
  /** Posição da marca do previsto (null sem previsto). */
  planned: number | null;
  /** Posição da marca do ritmo (null fora do mês corrente ou sem previsto). */
  pace: number | null;
  status: BulletStatus;
  /** Realizado ÷ previsto (null sem previsto). */
  ratio: number | null;
}

/** Folga antes de considerar o gasto "acima do ritmo" (5% do previsto). */
export const PACE_TOLERANCE = 0.05;

export function bulletStatus(b: BulletInput): BulletStatus {
  if (b.planned <= 0) return 'none';
  if (b.actual > b.planned) return 'over';
  if (b.actual + Math.max(0, b.committed) > b.planned) return 'risk';
  if (b.elapsed > 0 && b.elapsed < 1 && b.actual > b.planned * (b.elapsed + PACE_TOLERANCE))
    return 'pace';
  return 'ok';
}

export function bulletGeometry(b: BulletInput): BulletGeometry {
  const actual = Math.max(0, b.actual);
  const committed = Math.max(0, b.committed);
  const scale = Math.max(b.planned, actual + committed, 1);
  const pctOf = (v: number) => Math.min(100, (v / scale) * 100);
  return {
    actual: pctOf(actual),
    committed: Math.min(100 - pctOf(actual), pctOf(committed)),
    planned: b.planned > 0 ? pctOf(b.planned) : null,
    pace: b.planned > 0 && b.elapsed > 0 && b.elapsed < 1 ? pctOf(b.planned * b.elapsed) : null,
    status: bulletStatus(b),
    ratio: b.planned > 0 ? actual / b.planned : null,
  };
}

export const BULLET_STATUS_LABEL: Record<BulletStatus, string> = {
  get none() {
    return t('graficos-sem-orcamento');
  },
  get ok() {
    return t('graficos-no-ritmo');
  },
  get pace() {
    return t('graficos-acima-ritmo');
  },
  get risk() {
    return t('graficos-vai-estourar');
  },
  get over() {
    return t('graficos-estourou');
  },
};

/** Estados que pedem atenção (cor de status e rótulo na linha). */
export const isAlert = (s: BulletStatus) => s === 'risk' || s === 'over';
