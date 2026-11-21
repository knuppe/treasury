/**
 * Barras empilhadas mês a mês com linhas de referência no mesmo eixo (média móvel,
 * receitas). Função pura. A ponta arredondada vai só no segmento de cima de cada coluna.
 */
import { formatMonthLong, formatMonthShort } from '@/lib/format';

import { resolveColor, type ColorRef } from './colorMath';
import type { EChartsOption } from './echarts';
import {
  barStyle,
  baseOption,
  categoryAxis,
  compactMoney,
  lineStyle,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from './theme';
import { motion, tooltipHtml, type TooltipRow } from './tooltip';
import { t as tr } from '@/i18n';

export interface BarSeriesInput {
  key: string;
  name: string;
  color: ColorRef;
  values: readonly number[];
}

export interface LineSeriesInput {
  key: string;
  name: string;
  color: ColorRef;
  values: readonly number[];
  dashed?: boolean;
}

export interface StackedBarsInput {
  months: readonly string[];
  bars: readonly BarSeriesInput[];
  lines: readonly LineSeriesInput[];
  currency: string;
}

/** Índice da série no topo de cada coluna (a última com valor > 0), ou −1. */
export function topSegments(bars: readonly BarSeriesInput[], n: number): number[] {
  return Array.from({ length: n }, (_, i) => {
    for (let s = bars.length - 1; s >= 0; s--) if ((bars[s]!.values[i] ?? 0) > 0) return s;
    return -1;
  });
}

export function stackedBarsOption(t: ChartTheme, input: StackedBarsInput): EChartsOption {
  const top = topSegments(input.bars, input.months.length);
  const base = baseOption(t);
  const lineColor = (c: ColorRef) => resolveColor(t, c);
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: [
        ...input.bars.map((b) => ({ name: b.name, icon: 'roundRect' })),
        ...input.lines.map((l) => ({ name: l.name, icon: 'path://M0,4 L14,4 L14,6 L0,6 Z' })),
      ],
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'shadow', shadowStyle: { color: t.line, opacity: 0.35 } },
      formatter: (params: { dataIndex: number }[]) => {
        const i = params[0]?.dataIndex ?? 0;
        const total = input.bars.reduce((s, b) => s + (b.values[i] ?? 0), 0);
        const rows: TooltipRow[] = [
          {
            label: tr('graficos-total-despesas'),
            value: tooltipMoney(total, t, input.currency),
            strong: true,
          },
          ...input.bars
            .filter((b) => (b.values[i] ?? 0) !== 0)
            .slice()
            .reverse()
            .map((b) => ({
              label: b.name,
              value: tooltipMoney(b.values[i] ?? 0, t, input.currency),
              color: lineColor(b.color),
            })),
          ...input.lines.map((l) => ({
            label: l.name,
            value: tooltipMoney(l.values[i] ?? 0, t, input.currency),
            color: lineColor(l.color),
            dashed: l.dashed,
          })),
        ];
        const month = input.months[i];
        return tooltipHtml(
          t,
          month ? formatMonthLong(month) : '',
          rows,
          tr('graficos-clique-segmento'),
        );
      },
    },
    xAxis: categoryAxis(
      t,
      input.months.map((m) => formatMonthShort(m)),
    ),
    yAxis: valueAxis(t, (v) => compactMoney(v, t, input.currency)),
    series: [
      ...input.bars.map((b, s) => ({
        type: 'bar',
        name: b.name,
        stack: 'gasto',
        ...barStyle(t),
        color: lineColor(b.color),
        emphasis: { focus: 'series' },
        data: b.values.map((v, i) => ({
          value: v,
          itemStyle: {
            color: lineColor(b.color),
            borderColor: t.surface,
            borderWidth: 1,
            borderRadius: top[i] === s ? [4, 4, 0, 0] : 0,
          },
        })),
      })),
      ...input.lines.map((l) => ({
        type: 'line',
        name: l.name,
        ...lineStyle(),
        color: lineColor(l.color),
        lineStyle: { width: 2, type: l.dashed ? 'dashed' : 'solid', color: lineColor(l.color) },
        symbol: 'circle',
        z: 5,
        data: [...l.values],
      })),
    ],
  };
}
