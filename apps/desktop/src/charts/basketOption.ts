/**
 * Índice da sua cesta × índice oficial de alimentos, base 100 no primeiro mês (função pura,
 * 06 · Consumo › Preços, docs/gtin-inflation.md §5). Mês sem cobertura fica sem ponto na linha da
 * cesta.
 */
import { formatMonthLong, formatMonthShort } from '@/lib/format';
import { t as tr } from '@/i18n';

import type { EChartsOption } from './echarts';
import { baseOption, categoryAxis, lineStyle, valueAxis, type ChartTheme } from './theme';
import { motion, tooltipHtml, type TooltipRow } from './tooltip';

export interface BasketChartInput {
  months: readonly string[];
  basket: readonly (number | null)[];
  official: readonly (number | null)[];
  officialName: string | null;
}

const fmt = (v: number) => v.toLocaleString(undefined, { maximumFractionDigits: 1 });

export function basketOption(t: ChartTheme, input: BasketChartInput): EChartsOption {
  const base = baseOption(t);
  const mine = tr('alimentacao-cesta-serie');
  const official = input.officialName ?? tr('alimentacao-cesta-oficial');
  const hasOfficial = input.official.some((v) => v !== null);
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: hasOfficial ? [mine, official] : [mine],
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      formatter: (params: unknown) => {
        const list = params as { dataIndex: number }[];
        const i = list[0]?.dataIndex ?? 0;
        const rows: TooltipRow[] = [];
        const b = input.basket[i];
        rows.push({ label: mine, value: b == null ? '—' : fmt(b), color: t.accent, strong: true });
        const o = input.official[i];
        if (hasOfficial)
          rows.push({ label: official, value: o == null ? '—' : fmt(o), color: t.warning });
        const month = input.months[i];
        return tooltipHtml(t, month ? formatMonthLong(month) : '', rows);
      },
    },
    xAxis: categoryAxis(
      t,
      input.months.map((m) => formatMonthShort(m)),
    ),
    yAxis: { ...valueAxis(t, (v) => fmt(v)), scale: true },
    series: [
      {
        type: 'line',
        name: mine,
        ...lineStyle(),
        showSymbol: true,
        symbolSize: 6,
        connectNulls: true,
        color: t.accent,
        lineStyle: { width: 2.5, color: t.accent },
        z: 5,
        data: [...input.basket],
      },
      ...(hasOfficial
        ? [
            {
              type: 'line' as const,
              name: official,
              ...lineStyle(),
              showSymbol: false,
              color: t.warning,
              lineStyle: { width: 2, color: t.warning, type: 'dashed' as const },
              z: 4,
              data: [...input.official],
            },
          ]
        : []),
    ],
  };
}
