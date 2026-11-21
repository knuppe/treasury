/**
 * Linha do tempo do projeto (função pura). `acumulado`: receita e despesa acumuladas, o
 * resultado e o orçamento como referência; `periodo`: receitas e despesas de cada dia ou mês
 * lado a lado, com o resultado acumulado por cima.
 */
import { t as tr } from '@/i18n';
import { formatDate, formatMonthLong, formatMonthShort } from '@/lib/format';

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
import { motion, tooltipHtml, type TooltipRow } from './tooltip';

export type ProjectTimelineMode = 'acumulado' | 'periodo';

export interface ProjectTimelineInput {
  mode: ProjectTimelineMode;
  grain: 'day' | 'month';
  periods: readonly string[];
  income: readonly number[];
  expense: readonly number[];
  cumIncome: readonly number[];
  cumExpense: readonly number[];
  cumNet: readonly number[];
  budget: number | null;
  currency: string;
}

const shortLabel = (grain: 'day' | 'month', p: string) =>
  grain === 'day' ? formatDate(p).slice(0, 5) : formatMonthShort(p);
const longLabel = (grain: 'day' | 'month', p: string) =>
  grain === 'day' ? formatDate(p) : formatMonthLong(p);

export function projectTimelineOption(t: ChartTheme, input: ProjectTimelineInput): EChartsOption {
  const base = baseOption(t);
  const cumulative = input.mode === 'acumulado';
  const money = (v: number) => tooltipMoney(v, t, input.currency);
  const names = cumulative
    ? {
        income: tr('graficos-receita-acumulada'),
        expense: tr('graficos-despesa-acumulada'),
        net: tr('graficos-resultado'),
      }
    : {
        income: tr('graficos-receitas'),
        expense: tr('graficos-despesas'),
        net: tr('graficos-resultado-acumulado'),
      };
  const line = (name: string, color: string, data: readonly number[], dashed = false) => ({
    type: 'line',
    name,
    ...lineStyle(),
    color,
    step: cumulative && input.grain === 'day' ? 'end' : undefined,
    lineStyle: { width: 2, type: dashed ? 'dashed' : 'solid', color },
    symbol: 'circle',
    z: 5,
    data: [...data],
  });
  const bar = (name: string, color: string, data: readonly number[]) => ({
    type: 'bar',
    name,
    ...barStyle(t),
    color,
    emphasis: { focus: 'series' },
    data: [...data],
  });
  const budgetLine =
    cumulative && input.budget
      ? {
          markLine: {
            silent: true,
            symbol: 'none',
            lineStyle: { color: t.warning, type: 'dashed', width: 1 },
            label: {
              color: t.warning,
              fontSize: 11,
              position: 'insideEndTop',
              formatter: () =>
                tr('graficos-orcamento-valor', {
                  valor: compactMoney(input.budget!, t, input.currency),
                }),
            },
            data: [{ yAxis: input.budget }],
          },
        }
      : {};
  return {
    ...base,
    ...motion(),
    grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
    legend: {
      ...(base.legend as object),
      data: cumulative
        ? [names.expense, names.income, names.net]
        : [
            { name: names.income, icon: 'roundRect' },
            { name: names.expense, icon: 'roundRect' },
            { name: names.net, icon: 'path://M0,4 L14,4 L14,6 L0,6 Z' },
          ],
    },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: cumulative
        ? { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } }
        : { type: 'shadow', shadowStyle: { color: t.line, opacity: 0.35 } },
      formatter: (params: { dataIndex: number }[]) => {
        const i = params[0]?.dataIndex ?? 0;
        const rows: TooltipRow[] = cumulative
          ? [
              {
                label: tr('graficos-despesa-acumulada'),
                value: money(input.cumExpense[i] ?? 0),
                color: t.negative,
              },
              {
                label: tr('graficos-receita-acumulada'),
                value: money(input.cumIncome[i] ?? 0),
                color: t.positive,
              },
              {
                label: tr('graficos-resultado'),
                value: money(input.cumNet[i] ?? 0),
                color: t.accent,
                strong: true,
              },
              ...(input.budget
                ? [
                    {
                      label: tr('graficos-resta-orcamento'),
                      value: money(input.budget - (input.cumExpense[i] ?? 0)),
                      color: t.warning,
                      dashed: true,
                    },
                  ]
                : []),
            ]
          : [
              {
                label: tr('graficos-receitas'),
                value: money(input.income[i] ?? 0),
                color: t.positive,
              },
              {
                label: tr('graficos-despesas'),
                value: money(input.expense[i] ?? 0),
                color: t.negative,
              },
              {
                label: tr('graficos-resultado-periodo'),
                value: money((input.income[i] ?? 0) - (input.expense[i] ?? 0)),
                strong: true,
              },
              {
                label: tr('graficos-resultado-acumulado'),
                value: money(input.cumNet[i] ?? 0),
                color: t.accent,
                dashed: true,
              },
            ];
        const p = input.periods[i];
        return tooltipHtml(t, p ? longLabel(input.grain, p) : '', rows);
      },
    },
    xAxis: {
      ...categoryAxis(
        t,
        input.periods.map((p) => shortLabel(input.grain, p)),
      ),
      boundaryGap: !cumulative,
    },
    yAxis: valueAxis(t, (v) => compactMoney(v, t, input.currency)),
    series: cumulative
      ? [
          { ...line(names.expense, t.negative, input.cumExpense), ...budgetLine },
          line(names.income, t.positive, input.cumIncome),
          line(names.net, t.accent, input.cumNet, true),
        ]
      : [
          bar(names.income, t.positive, input.income),
          bar(names.expense, t.negative, input.expense),
          line(names.net, t.accent, input.cumNet, true),
        ],
  };
}
