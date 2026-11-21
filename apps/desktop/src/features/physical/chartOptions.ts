/**
 * Opção ECharts da figura "valor ao longo do tempo" (pura, testável sem canvas): valor
 * estimado (linha cheia), custo acumulado (linha em degraus, tracejada) e as avaliações
 * registradas (pontos com anel da cor da superfície, numa série de linha invisível), tudo
 * num eixo só.
 *
 * Cores pela entidade em ordem fixa da paleta: valor `@s0`, custo `@s2`, avaliações `@s3`.
 */
import {
  baseOption,
  categoryAxis,
  compactMoney,
  resolveColor,
  tooltipHtml,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
  type TooltipRow,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';
import { motion } from '@/charts/tooltip';
import { t as tr } from '@/i18n';
import { formatDate, formatMonthLong, formatMonthShort } from '@/lib/format';

import type { ValueChartData } from './transform';

export const VALUE_COLOR = '@s0';
export const COST_COLOR = '@s2';
export const APPRAISAL_COLOR = '@s3';

export function valueOption(t: ChartTheme, d: ValueChartData): EChartsOption {
  const base = baseOption(t);
  const value = resolveColor(t, VALUE_COLOR);
  const cost = resolveColor(t, COST_COLOR);
  const appraisal = resolveColor(t, APPRAISAL_COLOR);
  return {
    ...base,
    ...motion(),
    legend: { show: false },
    grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } },
      formatter: (raw: unknown) => {
        const params = (Array.isArray(raw) ? raw : [raw]) as { dataIndex: number }[];
        const i = params[0]?.dataIndex ?? 0;
        const rows: TooltipRow[] = [
          {
            label: d.estimated[i] ? tr('bens-valor-estimado') : tr('bens-valor'),
            value: tooltipMoney(d.value[i] ?? 0, t),
            color: value,
            strong: true,
          },
          {
            label: tr('bens-custo-acumulado'),
            value: tooltipMoney(d.cost[i] ?? 0, t),
            color: cost,
            dashed: true,
          },
        ];
        for (const a of d.appraisals.filter((x) => x.index === i))
          rows.push({
            label: tr('bens-avaliacao-em', { fonte: a.source, data: formatDate(a.date) }),
            value: tooltipMoney(a.value, t),
            color: appraisal,
          });
        const m = d.months[i];
        return tooltipHtml(t, m ? formatMonthLong(m) : '', rows);
      },
    },
    xAxis: { ...categoryAxis(t, d.months.map(formatMonthShort)), boundaryGap: false },
    yAxis: valueAxis(t, (v) => compactMoney(v, t)),
    series: [
      {
        type: 'line',
        name: tr('bens-custo-acumulado'),
        step: 'end',
        showSymbol: false,
        lineStyle: { width: 2, type: 'dashed', color: cost },
        itemStyle: { color: cost },
        data: d.cost,
      },
      {
        type: 'line',
        name: tr('bens-valor'),
        showSymbol: false,
        symbolSize: 8,
        lineStyle: { width: 2, color: value },
        itemStyle: { color: value },
        areaStyle: { color: value, opacity: 0.08 },
        data: d.value,
      },
      {
        // Só os pontos das avaliações (linha invisível; sem módulo de dispersão no bundle).
        type: 'line',
        name: tr('bens-avaliacoes'),
        symbol: 'circle',
        symbolSize: 10,
        showSymbol: true,
        showAllSymbol: true,
        connectNulls: false,
        lineStyle: { opacity: 0 },
        itemStyle: { color: appraisal, borderColor: t.surface, borderWidth: 2 },
        z: 5,
        data: d.months.map((_, i) => {
          const hits = d.appraisals.filter((a) => a.index === i);
          return hits.length ? hits[hits.length - 1]!.value : null;
        }),
      },
    ],
  };
}
