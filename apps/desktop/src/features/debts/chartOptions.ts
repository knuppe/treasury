/**
 * Opções ECharts das figuras de dívidas, puras (testáveis sem canvas):
 * - saldo devedor ao longo do contrato (uma linha, marca "Hoje"; com a simulação de
 *   amortização, uma segunda linha tracejada no mesmo eixo);
 * - composição das parcelas em barras empilhadas num só eixo: amortização, juros,
 *   correção do saldo e seguros + taxas.
 *
 * Cores pela entidade, em ordem fixa da paleta validada (`@s0`…`@s3`): a mesma série tem a
 * mesma cor mesmo quando outra some (contrato sem indexador não tem correção).
 */
import {
  barStyle,
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
import { motion } from '@/charts/tooltip';
import type { EChartsOption } from '@/charts/echarts';
import { t as tr } from '@/i18n';
import { formatDate, formatMonthShort } from '@/lib/format';

import type { BalancePoint, BreakdownMode, BreakdownRow } from './transform';

/** Linha da simulação de amortização no gráfico do saldo. */
export const SIMULATED_COLOR = '@s4';

export const PART_KEYS = ['amortization', 'interest', 'adjustment', 'charges'] as const;
export type PartKey = (typeof PART_KEYS)[number];

/** Nome de cada parte, no idioma do app (lido a cada acesso). */
export const PART_LABEL: Record<PartKey, string> = {
  get amortization() {
    return tr('dividas-amortizacao');
  },
  get interest() {
    return tr('dividas-juros');
  },
  get adjustment() {
    return tr('dividas-correcao-saldo');
  },
  get charges() {
    return tr('dividas-seguros-taxas');
  },
};

/** Slot fixo da paleta por parte (cor presa à entidade). */
export const PART_COLOR: Record<PartKey, string> = {
  amortization: '@s0',
  interest: '@s1',
  adjustment: '@s2',
  charges: '@s3',
};

/** Partes com algum valor (as outras não entram na legenda nem no gráfico). */
export function visibleParts(rows: readonly BreakdownRow[]): PartKey[] {
  return PART_KEYS.filter((k) => rows.some((r) => r[k] !== 0));
}

function axisLabel(date: string): string {
  return formatMonthShort(date.slice(0, 7));
}

export interface BalanceChartInput {
  points: readonly BalancePoint[];
  todayIndex: number;
  /** Saldo com a amortização simulada (mesmas datas; termina antes se reduzir o prazo). */
  simulated?: readonly BalancePoint[] | null;
}

export function balanceOption(t: ChartTheme, input: BalanceChartInput): EChartsOption {
  const base = baseOption(t);
  const dates = input.points.map((p) => p.date);
  if (input.simulated) {
    for (const p of input.simulated) if (!dates.includes(p.date)) dates.push(p.date);
    dates.sort();
  }
  const byDate = (pts: readonly BalancePoint[]) => new Map(pts.map((p) => [p.date, p]));
  const cur = byDate(input.points);
  const sim = input.simulated ? byDate(input.simulated) : null;
  const today = input.points[input.todayIndex]?.date;
  // Cenário (não é estado): uma cor da paleta, longe do azul do saldo atual.
  const simColor = resolveColor(t, SIMULATED_COLOR);
  return {
    ...base,
    ...motion(),
    // A legenda (quando há a simulação) fica na moldura da figura.
    legend: { show: false },
    grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } },
      formatter: (raw: unknown) => {
        const params = (Array.isArray(raw) ? raw : [raw]) as { dataIndex: number }[];
        const d = dates[params[0]?.dataIndex ?? 0] ?? '';
        const p = cur.get(d);
        const s = sim?.get(d);
        const rows: TooltipRow[] = [];
        if (p)
          rows.push({
            label: p.estimated ? tr('dividas-grafico-saldo-estimado') : tr('dividas-saldo-devedor'),
            value: tooltipMoney(p.balance, t),
            color: t.accent,
            strong: true,
          });
        if (s)
          rows.push({
            label: tr('dividas-com-simulada'),
            value: tooltipMoney(s.balance, t),
            color: simColor,
            dashed: true,
          });
        return tooltipHtml(t, d ? formatDate(d) : '', rows);
      },
    },
    xAxis: { ...categoryAxis(t, dates.map(axisLabel)), boundaryGap: false },
    yAxis: valueAxis(t, (v) => compactMoney(v, t)),
    series: [
      {
        type: 'line',
        name: tr('dividas-saldo-devedor'),
        showSymbol: false,
        symbolSize: 8,
        lineStyle: { width: 2, color: t.accent },
        itemStyle: { color: t.accent },
        areaStyle: { color: t.accent, opacity: 0.08 },
        data: dates.map((d) => cur.get(d)?.balance ?? null),
        markLine:
          today && input.todayIndex >= 0
            ? {
                silent: true,
                symbol: 'none',
                lineStyle: { color: t.lineStrong, width: 1, type: 'dashed' },
                label: {
                  color: t.textDim,
                  fontFamily: t.fontNumeric,
                  fontSize: 10.5,
                  formatter: tr('dividas-hoje'),
                },
                data: [{ xAxis: dates.indexOf(today) }],
              }
            : undefined,
      },
      ...(sim
        ? [
            {
              type: 'line',
              name: tr('dividas-com-simulada'),
              showSymbol: false,
              symbolSize: 8,
              lineStyle: { width: 2, type: 'dashed', color: simColor },
              itemStyle: { color: simColor },
              data: dates.map((d) => sim.get(d)?.balance ?? null),
            },
          ]
        : []),
    ],
  };
}

export interface BreakdownChartInput {
  rows: readonly BreakdownRow[];
  mode: BreakdownMode;
}

export function breakdownOption(t: ChartTheme, input: BreakdownChartInput): EChartsOption {
  const base = baseOption(t);
  const parts = visibleParts(input.rows);
  const color = (k: PartKey) => resolveColor(t, PART_COLOR[k]);
  // Ponta arredondada só no segmento de cima de cada coluna.
  const top = input.rows.map((r) => {
    for (let s = parts.length - 1; s >= 0; s--) if (r[parts[s]!] > 0) return s;
    return -1;
  });
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: parts.map((k) => ({ name: PART_LABEL[k], icon: 'roundRect' })),
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'shadow', shadowStyle: { color: t.line, opacity: 0.35 } },
      formatter: (raw: unknown) => {
        const params = (Array.isArray(raw) ? raw : [raw]) as { dataIndex: number }[];
        const r = input.rows[params[0]?.dataIndex ?? 0];
        if (!r) return '';
        const rows: TooltipRow[] = [
          {
            label:
              input.mode === 'ano'
                ? tr('dividas-grafico-parcelas-n', { n: r.count })
                : tr('dividas-parcela'),
            value: tooltipMoney(r.total, t),
            strong: true,
          },
          ...parts
            .slice()
            .reverse()
            .map((k) => ({
              label: PART_LABEL[k],
              value: tooltipMoney(r[k], t),
              color: color(k),
            })),
        ];
        if (r.extra > 0)
          rows.push({
            label: tr('dividas-amortizacao-extraordinaria'),
            value: tooltipMoney(r.extra, t),
          });
        const title =
          input.mode === 'ano'
            ? r.label
            : tr(r.estimated ? 'dividas-grafico-parcela-estimada' : 'dividas-parcela-n-data', {
                n: r.label,
                data: formatDate(r.date),
              });
        return tooltipHtml(
          t,
          title,
          rows,
          parts.includes('adjustment') ? tr('dividas-grafico-correcao-nota') : undefined,
        );
      },
    },
    xAxis: categoryAxis(
      t,
      input.rows.map((r) => r.label),
    ),
    yAxis: valueAxis(t, (v) => compactMoney(v, t)),
    series: parts.map((k, s) => ({
      type: 'bar',
      name: PART_LABEL[k],
      stack: 'parcela',
      ...barStyle(t),
      color: color(k),
      emphasis: { focus: 'series' },
      data: input.rows.map((r, i) => ({
        value: r[k],
        itemStyle: {
          color: color(k),
          borderColor: t.surface,
          borderWidth: 1,
          borderRadius: top[i] === s ? [4, 4, 0, 0] : 0,
          opacity: r.estimated && k === 'adjustment' ? 0.6 : 1,
        },
      })),
    })),
  };
}
