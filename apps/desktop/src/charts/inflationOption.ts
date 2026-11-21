/**
 * Gasto nominal × corrigido pela inflação, mês a mês (função pura, figura 07 · Inflação).
 * A distância entre as duas linhas é a correção de cada mês até hoje. Meses com índice ainda
 * não divulgado aparecem com "≈" no eixo, e o ponto corrigido que usou estimativa fica vazado.
 */
import { formatMonthLong, formatMonthShort } from '@/lib/format';
import { t as tr } from '@/i18n';

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
import { motion, pct, tooltipHtml, type TooltipRow } from './tooltip';

export interface InflationChartInput {
  months: readonly string[];
  nominal: readonly number[];
  real: readonly (number | null)[];
  /** A correção do mês usou dado estimado. */
  estimated: readonly boolean[];
  /** Mês ainda sem índice divulgado (marcado com "≈"). */
  unpublished: readonly boolean[];
  currency: string;
}

export function inflationOption(t: ChartTheme, input: InflationChartInput): EChartsOption {
  const base = baseOption(t);
  const nominalName = tr('inflacao-serie-nominal');
  const realName = tr('inflacao-serie-corrigido');
  const realColor = t.warning;
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: [
        { name: nominalName, icon: 'path://M0,4 L14,4 L14,6 L0,6 Z' },
        { name: realName, icon: 'path://M0,4 L14,4 L14,6 L0,6 Z' },
      ],
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } },
      formatter: (params: { dataIndex: number }[]) => {
        const i = params[0]?.dataIndex ?? 0;
        const nominal = input.nominal[i] ?? 0;
        const real = input.real[i] ?? null;
        const est = input.estimated[i] ?? false;
        const rows: TooltipRow[] = [
          { label: nominalName, value: tooltipMoney(nominal, t, input.currency), color: t.accent },
          {
            label: realName,
            value:
              real === null ? '—' : `${est ? '≈ ' : ''}${tooltipMoney(real, t, input.currency)}`,
            color: realColor,
            strong: true,
          },
        ];
        if (real !== null && nominal > 0) {
          rows.push({
            label: tr('inflacao-tooltip-correcao'),
            value: `${est ? '≈ ' : ''}${pct(real / nominal - 1, t)}`,
          });
        }
        const month = input.months[i];
        const note = input.unpublished[i]
          ? tr('inflacao-tooltip-nao-divulgado')
          : est
            ? tr('inflacao-tooltip-estimado')
            : undefined;
        return tooltipHtml(t, month ? formatMonthLong(month) : '', rows, note);
      },
    },
    xAxis: categoryAxis(
      t,
      input.months.map((m, i) => `${formatMonthShort(m)}${input.unpublished[i] ? ' ≈' : ''}`),
    ),
    yAxis: valueAxis(t, (v) => compactMoney(v, t, input.currency)),
    series: [
      {
        type: 'line',
        name: nominalName,
        ...lineStyle(),
        showSymbol: true,
        symbolSize: 6,
        color: t.accent,
        lineStyle: { width: 2, color: t.accent },
        z: 4,
        data: [...input.nominal],
      },
      {
        type: 'line',
        name: realName,
        ...lineStyle(),
        showSymbol: true,
        symbolSize: 7,
        color: realColor,
        lineStyle: { width: 2.5, color: realColor },
        z: 5,
        data: input.real.map((v, i) =>
          v === null
            ? null
            : {
                value: v,
                symbol: input.estimated[i] ? 'emptyCircle' : 'circle',
              },
        ),
      },
    ],
  };
}
