/**
 * Barras agrupadas horizontais (função pura): uma linha por categoria, uma barra por
 * titular, na cor do titular. Barras finas com vão de ~2 px entre as do mesmo grupo.
 */
import { resolveColor, type ColorRef } from './colorMath';
import type { EChartsOption } from './echarts';
import {
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from './theme';
import { motion, pct, tooltipHtml } from './tooltip';
import { t as tr } from '@/i18n';

export interface GroupedBarsInput {
  /** Linhas (categorias), de cima para baixo. */
  rows: readonly { key: string; label: string }[];
  /** Séries (titulares) com um valor por linha. */
  groups: readonly { key: string; name: string; color: ColorRef; values: readonly number[] }[];
  currency: string;
}

export function groupedHeight(input: GroupedBarsInput): number {
  const per = Math.max(22, input.groups.length * 11 + 10);
  return Math.max(220, input.rows.length * per + 56);
}

export function groupedBarsOption(t: ChartTheme, input: GroupedBarsInput): EChartsOption {
  const base = baseOption(t);
  const n = input.rows.length;
  const rev = (i: number) => n - 1 - i; // ECharts desenha a primeira categoria embaixo
  const labels = input.rows.map((r) => r.label).reverse();
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 24, top: 32, bottom: 8, containLabel: true },
    legend: { ...(base.legend as object), data: input.groups.map((g) => g.name) },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'shadow', shadowStyle: { color: t.line, opacity: 0.35 } },
      formatter: (params: { dataIndex: number }[]) => {
        const i = rev(params[0]?.dataIndex ?? 0);
        const total = input.groups.reduce((s, g) => s + (g.values[i] ?? 0), 0);
        return tooltipHtml(
          t,
          input.rows[i]?.label ?? '',
          [
            { label: 'Total', value: tooltipMoney(total, t, input.currency), strong: true },
            ...input.groups.map((g) => ({
              label: g.name,
              value: `${tooltipMoney(g.values[i] ?? 0, t, input.currency)}${total > 0 ? ` · ${pct((g.values[i] ?? 0) / total, t, 0)}` : ''}`,
              color: resolveColor(t, g.color),
            })),
          ],
          tr('graficos-clique-barra'),
        );
      },
    },
    xAxis: valueAxis(t, (v) => compactMoney(v, t, input.currency)),
    yAxis: {
      ...categoryAxis(t, labels),
      axisLabel: { color: t.text, fontFamily: t.fontText, fontSize: 11.5 },
    },
    series: input.groups.map((g) => ({
      type: 'bar',
      name: g.name,
      color: resolveColor(t, g.color),
      barMaxWidth: 12,
      barGap: '20%',
      barCategoryGap: '30%',
      itemStyle: { color: resolveColor(t, g.color), borderRadius: [0, 4, 4, 0] },
      emphasis: { focus: 'series' },
      data: [...g.values].reverse(),
    })),
  };
}
