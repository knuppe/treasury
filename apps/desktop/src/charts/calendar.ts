/**
 * Heatmap de calendário (função pura): um quadrado por dia, intensidade = gasto, numa
 * rampa sequencial de um matiz; faixas por quantis dos dias com gasto.
 */
import { t as tr } from '@/i18n';
import { formatDate, monthName } from '@/lib/format';
import { weekday } from '@/lib/months';

import type { EChartsOption } from './echarts';
import { baseOption, compactMoney, tooltipMoney, type ChartTheme } from './theme';
import { motion, tooltipHtml } from './tooltip';

export interface CalendarDayInput {
  date: string;
  value: number;
  count: number;
}

/** Dias da semana (domingo primeiro), no idioma do app. */
export function weekdays(): string[] {
  return tr('graficos-dias-semana').split(',');
}

/** Arredonda para um valor "redondo" em centavos (1, 2, 2,5 ou 5 × 10ⁿ). */
export function niceCeil(cents: number): number {
  if (cents <= 0) return 0;
  const p = 10 ** Math.floor(Math.log10(cents));
  for (const m of [1, 2, 2.5, 5, 10]) if (cents <= m * p) return m * p;
  return 10 * p;
}

/**
 * Limites das faixas: quantis dos dias com gasto, arredondados e sem repetição.
 * Devolve os limites superiores (o último é o máximo).
 */
export function calendarBins(values: readonly number[], steps: number): number[] {
  const positive = values.filter((v) => v > 0).sort((a, b) => a - b);
  if (positive.length === 0) return [];
  const out: number[] = [];
  for (let i = 1; i <= steps; i++) {
    const q =
      positive[Math.min(positive.length - 1, Math.ceil((i / steps) * positive.length) - 1)]!;
    const v = i === steps ? positive[positive.length - 1]! : niceCeil(q);
    if (out.length === 0 || v > out[out.length - 1]!) out.push(v);
  }
  return out;
}

/** Média de gasto por dia da semana (dias do período, com ou sem gasto). */
export function weekdayAverages(
  days: readonly CalendarDayInput[],
): { weekday: number; avg: number; total: number }[] {
  const sums = Array.from({ length: 7 }, () => ({ total: 0, n: 0 }));
  for (const d of days) {
    const w = weekday(d.date);
    sums[w]!.total += d.value;
    sums[w]!.n += 1;
  }
  return sums.map((s, i) => ({
    weekday: i,
    total: s.total,
    avg: s.n ? Math.round(s.total / s.n) : 0,
  }));
}

/** Os `n` dias de maior gasto. */
export function topDays(days: readonly CalendarDayInput[], n: number): CalendarDayInput[] {
  return [...days]
    .filter((d) => d.value > 0)
    .sort((a, b) => b.value - a.value || a.date.localeCompare(b.date))
    .slice(0, n);
}

export function calendarOption(
  t: ChartTheme,
  days: readonly CalendarDayInput[],
  range: { from: string; to: string },
  currency: string,
): EChartsOption {
  const bins = calendarBins(
    days.map((d) => d.value),
    5,
  );
  const ramp = t.sequential;
  const zero = ramp[0] === 'transparent' ? t.surface2 : ramp[0]!;
  const steps = ramp.slice(1);
  const pieces = [
    { value: 0, color: zero, label: tr('graficos-sem-gasto') },
    ...bins.map((hi, i) => ({
      gt: i === 0 ? 0 : bins[i - 1]!,
      lte: hi,
      color:
        steps[
          Math.min(
            steps.length - 1,
            Math.round((i * (steps.length - 1)) / Math.max(1, bins.length - 1)),
          )
        ]!,
      label: t.privacy
        ? tr('graficos-faixa', { n: i + 1 })
        : tr('graficos-ate', { valor: compactMoney(hi, t, currency) }),
    })),
  ];
  const byDate = new Map(days.map((d) => [d.date, d]));
  return {
    ...baseOption(t),
    ...motion(),
    tooltip: {
      ...(baseOption(t).tooltip as object),
      trigger: 'item',
      formatter: (p: { data: [string, number] }) => {
        const [date] = p.data;
        const d = byDate.get(date);
        return tooltipHtml(
          t,
          `${weekdays()[weekday(date)]}, ${formatDate(date)}`,
          [
            {
              label: tr('graficos-gasto'),
              value: tooltipMoney(d?.value ?? 0, t, currency),
              strong: true,
            },
            {
              label: tr('graficos-lancamentos'),
              value: t.privacy ? '•••••' : String(d?.count ?? 0),
            },
          ],
          d && d.value > 0 ? tr('graficos-clique-dia') : undefined,
        );
      },
    },
    visualMap: {
      type: 'piecewise',
      pieces,
      orient: 'horizontal',
      left: 0,
      bottom: 0,
      itemWidth: 12,
      itemHeight: 12,
      itemGap: 12,
      textStyle: { color: t.textDim, fontFamily: t.fontNumeric, fontSize: 10.5 },
      selectedMode: false,
    },
    calendar: {
      range: [range.from, range.to],
      top: 26,
      left: 28,
      right: 6,
      bottom: 40,
      cellSize: ['auto', 15],
      orient: 'horizontal',
      splitLine: { show: true, lineStyle: { color: t.line, width: 1, type: 'solid' } },
      itemStyle: { color: zero, borderColor: t.surface, borderWidth: 2 },
      dayLabel: {
        firstDay: 0,
        nameMap: weekdays().map((d) => d.charAt(0).toUpperCase()),
        color: t.textDim,
        fontFamily: t.fontNumeric,
        fontSize: 10,
      },
      monthLabel: {
        nameMap: Array.from({ length: 12 }, (_, i) => monthName(i + 1, true)),
        color: t.textDim,
        fontFamily: t.fontNumeric,
        fontSize: 10.5,
      },
      yearLabel: { show: false },
    },
    series: [
      {
        type: 'heatmap',
        coordinateSystem: 'calendar',
        data: days.map((d) => [d.date, d.value]),
        emphasis: { itemStyle: { borderColor: t.text, borderWidth: 1 } },
      },
    ],
  };
}
