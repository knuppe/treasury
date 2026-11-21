/**
 * Grupos de gasto mês a mês (essenciais, dívidas, futuro, não essencial): barras empilhadas
 * ou linhas, em parte da renda (fração) ou em valor. Um eixo só; a renda entra como linha de
 * referência nas barras em valor e as metas como tracejado nas linhas em parte da renda.
 * Função pura.
 */
import { t as tr } from '@/i18n';
import { formatMonthLong, formatMonthShort, formatPercent } from '@/lib/format';

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
import { motion, pct, tooltipHtml, type TooltipRow } from './tooltip';

export interface SpendGroupSeries {
  key: string;
  name: string;
  color: ColorRef;
  /** Centavos (valor) ou fração da renda (parte); `null` = sem renda no mês. */
  values: readonly (number | null)[];
  /** Meta em fração da renda (só desenhada nas linhas em parte da renda). */
  target?: number | null;
}

export interface SpendGroupsInput {
  months: readonly string[];
  series: readonly SpendGroupSeries[];
  unit: 'money' | 'share';
  form: 'bars' | 'lines';
  currency: string;
  /** Renda de cada mês (centavos): linha de referência nas barras em valor. */
  income?: readonly number[];
}

/** Índice da série no topo de cada coluna (a última com valor > 0), ou −1. */
function topSegments(series: readonly SpendGroupSeries[], n: number): number[] {
  return Array.from({ length: n }, (_, i) => {
    for (let s = series.length - 1; s >= 0; s--) if ((series[s]!.values[i] ?? 0) > 0) return s;
    return -1;
  });
}

export function spendGroupsOption(t: ChartTheme, input: SpendGroupsInput): EChartsOption {
  const base = baseOption(t);
  const color = (c: ColorRef) => resolveColor(t, c);
  const share = input.unit === 'share';
  const fmt = (v: number) => (share ? pct(v, t) : tooltipMoney(v, t, input.currency));
  const axis = (v: number) =>
    share ? formatPercent(v, { decimals: 0 }) : compactMoney(v, t, input.currency);
  const bars = input.form === 'bars';
  const incomeLine = bars && !share && input.income ? input.income : null;
  const top = topSegments(input.series, input.months.length);
  const incomeName = tr('graficos-grupos-renda');
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: [
        ...input.series.map((s) => ({ name: s.name, ...(bars ? { icon: 'roundRect' } : {}) })),
        ...(incomeLine ? [{ name: incomeName, icon: 'path://M0,4 L14,4 L14,6 L0,6 Z' }] : []),
      ],
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: bars
        ? { type: 'shadow', shadowStyle: { color: t.line, opacity: 0.35 } }
        : { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } },
      formatter: (params: { dataIndex: number }[]) => {
        const i = params[0]?.dataIndex ?? 0;
        const values = input.series.map((s) => s.values[i]);
        const known = values.filter((v): v is number => v != null);
        const rows: TooltipRow[] = [
          ...(known.length
            ? [
                {
                  label: tr('graficos-grupos-total'),
                  value: fmt(known.reduce((a, v) => a + v, 0)),
                  strong: true,
                },
              ]
            : []),
          // Nas barras, de cima para baixo como a pilha; nas linhas, na ordem dos grupos.
          ...(bars
            ? input.series.map((s, j) => ({ s, v: values[j] })).reverse()
            : input.series.map((s, j) => ({ s, v: values[j] }))
          ).map(({ s, v }) => ({
            label: s.name,
            value: v == null ? '—' : fmt(v),
            color: color(s.color),
          })),
          ...(incomeLine
            ? [
                {
                  label: incomeName,
                  value: tooltipMoney(incomeLine[i] ?? 0, t, input.currency),
                  color: t.lineStrong,
                  dashed: true,
                },
              ]
            : []),
          ...(!bars && share
            ? input.series
                .filter((s) => s.target != null)
                .map((s) => ({
                  label: tr('graficos-grupos-meta', { grupo: s.name }),
                  value: pct(s.target!, t),
                  color: color(s.color),
                  dashed: true,
                }))
            : []),
        ];
        const month = input.months[i];
        return tooltipHtml(
          t,
          month ? formatMonthLong(month) : '',
          rows,
          bars ? tr('graficos-grupos-clique') : undefined,
        );
      },
    },
    xAxis: {
      ...categoryAxis(
        t,
        input.months.map((m) => formatMonthShort(m)),
      ),
      ...(bars ? {} : { boundaryGap: false }),
    },
    yAxis: valueAxis(t, axis),
    series: [
      ...input.series.map((s, k) =>
        bars
          ? {
              type: 'bar',
              name: s.name,
              stack: 'grupos',
              ...barStyle(t),
              color: color(s.color),
              emphasis: { focus: 'series' },
              data: s.values.map((v, i) => ({
                value: v,
                itemStyle: {
                  color: color(s.color),
                  borderColor: t.surface,
                  borderWidth: 1,
                  borderRadius: top[i] === k ? [4, 4, 0, 0] : 0,
                },
              })),
              // Em parte da renda, o tracejado em 100%: acima dele, gastou mais do que entrou.
              ...(share && k === input.series.length - 1
                ? {
                    markLine: {
                      silent: true,
                      symbol: 'none',
                      label: { show: false },
                      lineStyle: { type: 'dashed', width: 1, color: t.lineStrong },
                      data: [{ yAxis: 1 }],
                    },
                  }
                : {}),
            }
          : {
              type: 'line',
              name: s.name,
              ...lineStyle(),
              showSymbol: input.months.length <= 1,
              color: color(s.color),
              lineStyle: { width: 2, color: color(s.color) },
              symbol: 'circle',
              connectNulls: false,
              emphasis: { focus: 'series' },
              data: [...s.values],
              ...(share && s.target != null
                ? {
                    markLine: {
                      silent: true,
                      symbol: 'none',
                      label: { show: false },
                      lineStyle: { type: 'dashed', width: 1, color: color(s.color), opacity: 0.8 },
                      data: [{ yAxis: s.target }],
                    },
                  }
                : {}),
            },
      ),
      ...(incomeLine
        ? [
            {
              type: 'line',
              name: incomeName,
              ...lineStyle(),
              color: t.lineStrong,
              lineStyle: { width: 2, type: 'dashed', color: t.lineStrong },
              symbol: 'circle',
              z: 5,
              data: [...incomeLine],
            },
          ]
        : []),
    ],
  };
}
