/**
 * Composição da estimativa por tipo de imposto (função pura): uma barra horizontal por
 * parte, de cima para baixo na ordem do núcleo, numa cor só (o eixo dá o nome de cada
 * parte); a fração do total fica no fim da barra e o valor, na dica.
 */
import type { EChartsOption } from '@/charts/echarts';
import {
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts/theme';
import { motion, pct, tooltipHtml } from '@/charts/tooltip';
import { t as tr } from '@/i18n';

export interface PartBar {
  key: string;
  label: string;
  amount: number;
  /** Tem estimativa (e não só valores lançados ou informados). */
  estimated: boolean;
}

export function partsHeight(n: number): number {
  return Math.max(160, n * 40 + 24);
}

export function partsOption(
  t: ChartTheme,
  bars: readonly PartBar[],
  currency: string,
): EChartsOption {
  const base = baseOption(t);
  const total = bars.reduce((s, b) => s + b.amount, 0);
  const rows = [...bars].reverse(); // ECharts desenha a primeira categoria embaixo
  const color = t.series[0] ?? t.accent;
  return {
    ...base,
    ...motion(),
    legend: { show: false },
    grid: { left: 8, right: 8, top: 8, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'item',
      formatter: (p: { dataIndex: number }) => {
        const b = rows[p.dataIndex];
        if (!b) return '';
        return tooltipHtml(
          t,
          b.label,
          [
            {
              label: tr('graficos-valor'),
              value: tooltipMoney(b.amount, t, currency),
              strong: true,
            },
            { label: tr('graficos-do-total'), value: total > 0 ? pct(b.amount / total, t) : '—' },
          ],
          b.estimated ? tr('impostos-grafico-inclui-estimados') : tr('impostos-grafico-lancados'),
        );
      },
    },
    // Folga à direita para o rótulo (fração do total) da maior barra.
    xAxis: {
      ...valueAxis(t, (v) => compactMoney(v, t, currency)),
      show: false,
      max: (v: { max: number }) => v.max * 1.15,
    },
    yAxis: {
      ...categoryAxis(
        t,
        rows.map((b) => b.label),
      ),
      axisLabel: { color: t.text, fontFamily: t.fontText, fontSize: 12 },
    },
    series: [
      {
        type: 'bar',
        name: tr('impostos-imposto-estimado'),
        barMaxWidth: 18,
        barCategoryGap: '35%',
        itemStyle: { color, borderRadius: [0, 4, 4, 0] },
        label: {
          show: true,
          position: 'right',
          color: t.textDim,
          fontFamily: t.fontNumeric,
          fontSize: 11,
          formatter: (p: { dataIndex: number }) => {
            const b = rows[p.dataIndex];
            // Só a fração: os valores ficam na dica, na tabela e nos três cenários ao lado.
            return b ? pct(total > 0 ? b.amount / total : 0, t, 0) : '';
          },
        },
        data: rows.map((b) => b.amount),
      },
    ],
  };
}
