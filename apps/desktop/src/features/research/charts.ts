/**
 * Opções ECharts da análise de ativos (funções puras): barras verticais por exercício e
 * linhas sobre um eixo só. Cada figura tem uma unidade (dinheiro, % ou índice base 100):
 * nada de eixo duplo; medidas de escalas diferentes vão em figuras separadas.
 * Cores da paleta categórica do app (validada), presas à chave da série.
 */
import {
  barStyle,
  baseOption,
  categoryAxis,
  lineStyle,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';

import { axisTooltip, colorsFor } from '../portfolio/charts/common';

export interface ChartSeries {
  key: string;
  name: string;
  values: readonly (number | null)[];
}

export interface SeriesChartInput {
  labels: readonly string[];
  series: readonly ChartSeries[];
  /** Ordem fixa das chaves (prende a cor de cada série). */
  colorKeys: readonly string[];
  /** Rótulo do eixo e do tooltip. */
  format: (v: number) => string;
}

export function barsOption(t: ChartTheme, input: SeriesChartInput): EChartsOption {
  const color = colorsFor(input.colorKeys, t);
  const base = baseOption(t);
  const bs = barStyle(t);
  const many = input.series.length > 1;
  return {
    ...base,
    grid: { left: 8, right: 16, top: many ? 36 : 16, bottom: 8, containLabel: true },
    legend: many
      ? { ...(base.legend as object), data: input.series.map((s) => s.name) }
      : { show: false },
    tooltip: {
      ...(base.tooltip as object),
      ...axisTooltip(t, (v) => (v == null ? '—' : input.format(v))),
      axisPointer: { type: 'shadow', shadowStyle: { color: t.line, opacity: 0.35 } },
    },
    xAxis: categoryAxis(t, [...input.labels]),
    yAxis: valueAxis(t, input.format),
    series: input.series.map((s) => ({
      ...bs,
      type: 'bar',
      name: s.name,
      color: color(s.key),
      barGap: '12%',
      data: s.values.map((v) => ({
        value: v,
        itemStyle: {
          ...(bs.itemStyle as object),
          color: color(s.key),
          borderRadius: (v ?? 0) < 0 ? [0, 0, 4, 4] : [4, 4, 0, 0],
        },
      })),
    })),
  };
}

export function linesOption(
  t: ChartTheme,
  input: SeriesChartInput & { baseline?: number },
): EChartsOption {
  const color = colorsFor(input.colorKeys, t);
  const base = baseOption(t);
  const ls = lineStyle();
  const many = input.series.length > 1;
  return {
    ...base,
    grid: { left: 8, right: 16, top: many ? 36 : 16, bottom: 8, containLabel: true },
    legend: many
      ? { ...(base.legend as object), data: input.series.map((s) => s.name) }
      : { show: false },
    tooltip: {
      ...(base.tooltip as object),
      ...axisTooltip(t, (v) => (v == null ? '—' : input.format(v))),
    },
    xAxis: { ...categoryAxis(t, [...input.labels]), boundaryGap: false },
    yAxis: { ...valueAxis(t, input.format), scale: input.baseline != null },
    series: input.series.map((s, i) => ({
      ...ls,
      type: 'line',
      name: s.name,
      data: [...s.values],
      color: color(s.key),
      itemStyle: { color: color(s.key) },
      lineStyle: { ...(ls.lineStyle as object), color: color(s.key) },
      connectNulls: true,
      ...(i === 0 && input.baseline != null
        ? {
            markLine: {
              silent: true,
              symbol: 'none',
              label: { show: false },
              lineStyle: { color: t.lineStrong, type: 'dashed', width: 1 },
              data: [{ yAxis: input.baseline }],
            },
          }
        : {}),
    })),
  };
}
