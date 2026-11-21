/**
 * Cascata da ponte do resultado: barras horizontais do valor no início ao valor no fim,
 * cada linha (preço, proventos, taxas, impostos, aportes, retiradas) flutuando do acumulado
 * anterior. Duas séries empilhadas com `stackStrategy: 'all'` (a base invisível e o degrau),
 * para a faixa cruzar o zero quando o período começa sem carteira. Sem animação.
 */
import { useCallback } from 'react';

import {
  baseOption,
  compactMoney,
  EChart,
  tooltipHtml,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';

import type { BridgeStep } from '../transform';

export function BridgeChart({
  steps,
  labels,
  currency,
  fxNote,
  ariaLabel,
}: {
  steps: BridgeStep[];
  /** Nome de cada degrau, na ordem de `steps`. */
  labels: string[];
  currency: string;
  /** "dos quais câmbio: X" no tooltip do ganho de preço (ou nada). */
  fxNote?: string;
  ariaLabel: string;
}) {
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      const base = baseOption(t);
      const color = (s: BridgeStep) =>
        s.total ? t.accent : s.amount < 0 ? t.negative : t.positive;
      return {
        ...base,
        animation: false,
        legend: { show: false },
        grid: { left: 8, right: 24, top: 8, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'item',
          formatter: (raw: unknown) => {
            const i = (raw as { dataIndex?: number }).dataIndex ?? 0;
            const s = steps[i];
            if (!s) return '';
            const value = tooltipMoney(s.amount, t, currency);
            const sign = !s.total && s.amount > 0 && !t.privacy ? '+' : '';
            return tooltipHtml(
              t,
              labels[i] ?? '',
              [{ label: labels[i] ?? '', value: `${sign}${value}`, color: color(s), strong: true }],
              s.key === 'preco' ? fxNote : undefined,
            );
          },
        },
        xAxis: valueAxis(t, (v) => compactMoney(v, t, currency)),
        yAxis: {
          type: 'category',
          data: labels,
          inverse: true,
          axisLine: { lineStyle: { color: t.line } },
          axisTick: { show: false },
          axisLabel: { color: t.textDim, fontFamily: t.fontText, fontSize: 11 },
        },
        series: [
          {
            type: 'bar',
            name: 'base',
            stack: 'ponte',
            stackStrategy: 'all',
            silent: true,
            tooltip: { show: false },
            itemStyle: { color: 'transparent' },
            emphasis: { disabled: true },
            data: steps.map((s) => s.low),
          },
          {
            type: 'bar',
            name: 'ponte',
            stack: 'ponte',
            stackStrategy: 'all',
            barMaxWidth: 18,
            emphasis: { disabled: true },
            data: steps.map((s) => ({
              value: s.high - s.low,
              itemStyle: { color: color(s), borderRadius: 3 },
            })),
          },
        ],
      };
    },
    [steps, labels, currency, fxNote],
  );
  return <EChart option={option} height={Math.max(220, steps.length * 30)} ariaLabel={ariaLabel} />;
}
