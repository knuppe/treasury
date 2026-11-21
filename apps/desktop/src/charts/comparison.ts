/**
 * Comparação A × B (funções puras): barras divergentes da variação (b − a) a partir do
 * zero, ou slope (uma linha por categoria, de A para B). Gastar mais é ruim: ▲ em
 * --negative; gastar menos, ▼ em --positive (sinal e seta, nunca só a cor).
 */
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
import { motion, pct, tooltipHtml } from './tooltip';
import { t as tr } from '@/i18n';

export interface ComparisonInput {
  key: string;
  label: string;
  a: number;
  b: number;
  color: ColorRef;
}

export interface ComparisonLabels {
  a: string;
  b: string;
  currency: string;
}

const deltaRatio = (r: ComparisonInput) => (r.a > 0 ? (r.b - r.a) / r.a : null);

function rowTooltip(t: ChartTheme, r: ComparisonInput, l: ComparisonLabels): string {
  const d = r.b - r.a;
  const ratio = deltaRatio(r);
  return tooltipHtml(
    t,
    r.label,
    [
      { label: l.a, value: tooltipMoney(r.a, t, l.currency) },
      { label: l.b, value: tooltipMoney(r.b, t, l.currency) },
      {
        label: `${d > 0 ? '▲ ' : d < 0 ? '▼ ' : ''}${tr('graficos-variacao')}`,
        value: `${tooltipMoney(d, t, l.currency)}${ratio == null ? '' : ` (${pct(ratio, t)})`}`,
        strong: true,
      },
    ],
    tr('graficos-clique-segundo-periodo'),
  );
}

/** Altura para `n` linhas de barras horizontais. */
export function rowsHeight(n: number, per = 28): number {
  return Math.max(200, n * per + 48);
}

export function divergingOption(
  t: ChartTheme,
  rows: readonly ComparisonInput[],
  l: ComparisonLabels,
): EChartsOption {
  const ordered = [...rows].reverse(); // a maior variação no topo
  const base = baseOption(t);
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 72, top: 8, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'item',
      formatter: (p: { dataIndex: number }) => rowTooltip(t, ordered[p.dataIndex]!, l),
    },
    xAxis: valueAxis(t, (v) => compactMoney(v, t, l.currency)),
    yAxis: {
      ...categoryAxis(
        t,
        ordered.map((r) => r.label),
      ),
      axisLabel: { color: t.text, fontFamily: t.fontText, fontSize: 11.5 },
    },
    series: [
      {
        type: 'bar',
        barMaxWidth: 16,
        data: ordered.map((r) => {
          const d = r.b - r.a;
          return {
            value: d,
            itemStyle: {
              color: d > 0 ? t.negative : t.positive,
              borderRadius: d >= 0 ? [0, 4, 4, 0] : [4, 0, 0, 4],
            },
          };
        }),
        label: {
          show: !t.privacy,
          position: 'right',
          color: t.textDim,
          fontFamily: t.fontNumeric,
          fontSize: 10.5,
          formatter: (p: { value: number }) =>
            `${p.value > 0 ? '▲ +' : p.value < 0 ? '▼ ' : ''}${compactMoney(p.value, t, l.currency)}`,
        },
        markLine: {
          silent: true,
          symbol: 'none',
          lineStyle: { color: t.lineStrong, width: 1, type: 'solid' },
          label: { show: false },
          data: [{ xAxis: 0 }],
        },
      },
    ],
  };
}

export function slopeOption(
  t: ChartTheme,
  rows: readonly ComparisonInput[],
  l: ComparisonLabels,
): EChartsOption {
  const base = baseOption(t);
  // Rótulos diretos só nas 4 maiores variações (o resto fica na legenda e no tooltip).
  const labeled = new Set(
    [...rows]
      .sort((x, y) => Math.abs(y.b - y.a) - Math.abs(x.b - x.a))
      .slice(0, 4)
      .map((r) => r.key),
  );
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 132, top: 36, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'item',
      formatter: (p: { seriesIndex: number }) => rowTooltip(t, rows[p.seriesIndex]!, l),
    },
    xAxis: { ...categoryAxis(t, [l.a, l.b]), boundaryGap: false },
    yAxis: valueAxis(t, (v) => compactMoney(v, t, l.currency)),
    series: rows.map((r) => {
      const color = resolveColor(t, r.color);
      return {
        type: 'line',
        name: r.label,
        ...lineStyle(),
        showSymbol: true,
        symbolSize: 8,
        color,
        itemStyle: { color, borderColor: t.surface, borderWidth: 2 },
        lineStyle: { width: 2, color },
        data: [r.a, r.b],
        endLabel: {
          show: labeled.has(r.key),
          color: t.text,
          fontFamily: t.fontText,
          fontSize: 11,
          formatter: () => r.label,
        },
        emphasis: { focus: 'series' },
      };
    }),
  };
}
