/**
 * Mock de `reports.inflation` (figura 07 · Inflação): o gasto de cada mês da evolução mensal
 * corrigido por um IPCA fictício de 0,4% ao mês. Os dois últimos meses ainda "não saíram" e
 * entram como estimados, como no núcleo.
 */
import type { Basis } from '@/api/generated/Basis';
import type { InflationCategory } from '@/api/generated/InflationCategory';
import type { InflationReport } from '@/api/generated/InflationReport';
import { shiftMonth } from '@/lib/months';

import { divRound } from './engine';
import { monthlyEvolution } from './reports';

const MONTHLY = 0.004;

/** Fator entre o fim do mês `from` e o fim do mês `to` (meses inteiros). */
const factor = (months: number) => (1 + MONTHLY) ** months;

const avg = (v: number[]) =>
  v.length
    ? divRound(
        v.reduce((s, x) => s + x, 0),
        v.length,
      )
    : 0;

export function inflation(
  months: number,
  endMonth: string,
  today: string,
  basis: Basis,
): InflationReport {
  const n = Math.min(Math.max(months, 3), 36);
  const evo = monthlyEvolution(n, endMonth, 6, basis);
  const lastPublished = shiftMonth(today.slice(0, 7), -2);
  const last = evo.months.length - 1;
  const estimatedTail = endMonth > lastPublished;
  const real = evo.totals.map((v, i) => Math.round(v * factor(last - i)));
  const estimated = evo.months.map((_, i) => i < last && estimatedTail);
  // Como o núcleo: a comparação começa no primeiro mês com gasto e precisa de seis meses.
  const start = evo.totals.findIndex((v) => v > 0);
  const from = start >= 0 && evo.months.length - start >= 6 ? start : -1;
  const categories: InflationCategory[] =
    from >= 0
      ? evo.series.map((s) => {
          const first = avg(s.values.slice(from, from + 3));
          const lastAvg = avg(s.values.slice(-3));
          const change = first > 0 ? lastAvg / first - 1 : null;
          const infl = factor(n - from - 3) - 1;
          return {
            key: s.key,
            label: s.label,
            color: s.color,
            first,
            last: lastAvg,
            change,
            inflation: infl,
            above_inflation: change === null ? null : change - infl,
          };
        })
      : [];
  return {
    months: evo.months,
    basis,
    currency: evo.currency,
    nominal: evo.totals,
    real,
    estimated,
    inflation: factor(n) - 1,
    inflation_estimated: estimatedTail,
    index: {
      series_id: 'ipca',
      name: 'IPCA',
      source: 'bcb_sgs:433',
      last_date: `${lastPublished}-01`,
    },
    categories,
    compare_from: from >= 0 ? (evo.months[from] ?? null) : null,
    missing_fx: evo.missing_fx,
    fx_estimated: evo.estimated,
  };
}
