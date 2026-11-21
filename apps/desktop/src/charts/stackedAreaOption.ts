/**
 * Área empilhada mês a mês (função pura): bandas positivas (contas, investimentos) acima
 * do zero, negativas (dívidas) abaixo, e linhas no mesmo eixo (patrimônio líquido).
 * Preenchimento leve (lavado) com o traço de 2 px na cor da série.
 */
import { formatMonthLong, formatMonthShort } from '@/lib/format';

import { resolveColor, type ColorRef } from './colorMath';
import type { EChartsOption } from './echarts';
import {
  baseOption,
  categoryAxis,
  compactMoney,
  lineStyle,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from './theme';
import { motion, tooltipHtml } from './tooltip';

export interface AreaSeriesInput {
  key: string;
  name: string;
  color: ColorRef;
  values: readonly number[];
  /** Empilha com as séries do mesmo grupo; sem grupo = linha solta. */
  stack?: string;
  dashed?: boolean;
}

export interface StackedAreaInput {
  months: readonly string[];
  series: readonly AreaSeriesInput[];
  currency: string;
  /** Linha extra do tooltip (ex.: total). */
  totalLabel?: string;
  totals?: readonly number[];
}

export function stackedAreaOption(t: ChartTheme, input: StackedAreaInput): EChartsOption {
  const base = baseOption(t);
  const color = (c: ColorRef) => resolveColor(t, c);
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: input.series.map((s) => ({
        name: s.name,
        icon: s.stack ? 'roundRect' : 'path://M0,4 L14,4 L14,6 L0,6 Z',
      })),
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } },
      formatter: (params: { dataIndex: number }[]) => {
        const i = params[0]?.dataIndex ?? 0;
        const rows = [
          ...(input.totals && input.totalLabel
            ? [
                {
                  label: input.totalLabel,
                  value: tooltipMoney(input.totals[i] ?? 0, t, input.currency),
                  strong: true,
                },
              ]
            : []),
          ...input.series.map((s) => ({
            label: s.name,
            value: tooltipMoney(s.values[i] ?? 0, t, input.currency),
            color: color(s.color),
            dashed: s.dashed,
            strong: !s.stack && !input.totals,
          })),
        ];
        const m = input.months[i];
        return tooltipHtml(t, m ? formatMonthLong(m) : '', rows);
      },
    },
    xAxis: {
      ...categoryAxis(
        t,
        input.months.map((m) => formatMonthShort(m)),
      ),
      boundaryGap: false,
    },
    yAxis: valueAxis(t, (v) => compactMoney(v, t, input.currency)),
    series: input.series.map((s) => ({
      type: 'line',
      name: s.name,
      ...lineStyle(),
      color: color(s.color),
      stack: s.stack,
      z: s.stack ? 2 : 5,
      triggerLineEvent: true,
      lineStyle: { width: 2, color: color(s.color), type: s.dashed ? 'dashed' : 'solid' },
      areaStyle: s.stack ? { color: color(s.color), opacity: 0.16 } : undefined,
      emphasis: { focus: 'series' },
      data: [...s.values],
    })),
  };
}
