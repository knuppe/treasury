/**
 * Transformações do comparador (`research.compare`): melhor valor de cada linha (com
 * empates) e as cotações normalizadas numa base comum (100 no primeiro dia em que todos
 * os ativos com cotação têm preço).
 */
import type { CompareRow } from '@/api/generated/CompareRow';
import type { CompareSeries } from '@/api/generated/CompareSeries';
import type { IndicatorGroup } from '@/api/generated/IndicatorGroup';

import { PANEL_GROUPS } from './labels';

/**
 * Índices com o melhor valor da linha. O núcleo escolhe um (`best`, pela ordem natural
 * do indicador); empates com o mesmo valor também são destacados. Linhas sem ordem
 * natural (cotação, valor da firma) não têm melhor.
 */
export function bestOfRow(row: Pick<CompareRow, 'values' | 'best'>): Set<number> {
  const out = new Set<number>();
  if (row.best == null) return out;
  const top = row.values[row.best];
  if (top == null) return out;
  row.values.forEach((v, i) => {
    if (v != null && Math.abs(v - top) <= Math.abs(top) * 1e-9) out.add(i);
  });
  return out;
}

export interface CompareGroup {
  group: IndicatorGroup | 'fair_price';
  rows: CompareRow[];
}

/** Linhas agrupadas na ordem do painel de indicadores (preço justo por último). */
export function groupRows(rows: readonly CompareRow[]): CompareGroup[] {
  const order: (IndicatorGroup | 'fair_price')[] = [...PANEL_GROUPS, 'fair_price'];
  return order
    .map((group) => ({ group, rows: rows.filter((r) => r.group === group) }))
    .filter((g) => g.rows.length > 0);
}

export interface IndexedSeries {
  dates: string[];
  /** Uma linha por série com cotação, na ordem de entrada (`null` = sem pregão). */
  series: { ticker: string; values: (number | null)[] }[];
  /** Data da base 100 comum. */
  base: string | null;
  /** Ativos sem histórico de cotações (ficam fora do gráfico). */
  missing: string[];
}

/**
 * Cotações normalizadas numa base comum. O núcleo normaliza cada série no primeiro dia
 * dela; aqui a base passa a ser o primeiro dia em que todas têm valor, para que todas
 * comecem em 100 no mesmo ponto.
 */
export function rebaseCommon(input: readonly CompareSeries[]): IndexedSeries {
  const withData = input.filter((s) => s.points.length > 0);
  const missing = input.filter((s) => s.points.length === 0).map((s) => s.ticker ?? '—');
  const maps = withData.map((s) => new Map(s.points.map((p) => [p.date, p.value])));
  const all = [...new Set(withData.flatMap((s) => s.points.map((p) => p.date)))].sort();
  const base = all.find((d) => maps.every((m) => m.has(d))) ?? null;
  if (!base) return { dates: [], series: [], base: null, missing };
  const dates = all.filter((d) => d >= base);
  const series = withData.map((s, i) => {
    const m = maps[i]!;
    const b = m.get(base)!;
    return {
      ticker: s.ticker ?? '—',
      values: dates.map((d) => {
        const v = m.get(d);
        return v == null || b === 0 ? null : Number(((v / b) * 100).toFixed(4));
      }),
    };
  });
  return { dates, series, base, missing };
}
