import { useCallback, useMemo, type ReactNode } from 'react';

import type { Performance } from '@/api/generated/Performance';
import {
  barStyle,
  baseOption,
  categoryAxis,
  ChartFrame,
  compactMoney,
  EChart,
  lineStyle,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';
import { Money, TechTable, type TechColumn } from '@/design/components';
import { t as tr } from '@/i18n';
import { formatMonthShort } from '@/lib/format';
import { usePrivacy } from '@/state/ui';

import { monthlyReturns, performanceLines } from '../transform';
import { axisTooltip, colorsFor, pctAxis, pctText } from './common';

const LINE_KEYS = ['carteira', 'cdi', 'inflacao', 'ibov', 'sp500', 'usd'];

/** Série escolhida para comparar (ativo, índice, CDI + x%), já em %. */
export interface ExtraLine {
  key: string;
  name: string;
  values: (number | null)[];
  dashed?: boolean;
}

/**
 * FIG · Rentabilidade acumulada × CDI × inflação × benchmarks (tudo em %, um eixo só), mais
 * as séries que a pessoa escolheu comparar (`extra`).
 */
export function CumulativeReturnChart({
  index,
  perf,
  extra = [],
  source,
  actions,
}: {
  index: string;
  perf: Performance;
  extra?: readonly ExtraLine[];
  source?: ReactNode;
  actions?: ReactNode;
}) {
  const privacy = usePrivacy();
  const data = useMemo(() => {
    const d = performanceLines(perf);
    return { labels: d.labels, series: [...d.series, ...extra] };
  }, [perf, extra]);
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      // As escolhidas vêm logo depois da inflação, para caberem nas cores distintas do tema.
      const keys = extra.length
        ? ['carteira', 'cdi', 'inflacao', ...extra.map((e) => e.key), 'ibov', 'sp500', 'usd']
        : LINE_KEYS;
      const color = colorsFor(keys, t);
      const base = baseOption(t);
      return {
        ...base,
        grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
        legend: { ...(base.legend as object), data: data.series.map((s) => s.name) },
        tooltip: { ...(base.tooltip as object), ...axisTooltip(t, (v) => pctText(v, t)) },
        xAxis: { ...categoryAxis(t, data.labels), boundaryGap: false },
        yAxis: valueAxis(t, pctAxis),
        series: data.series.map((s) => {
          const c = color(s.key === 'real' ? 'carteira' : s.key);
          const ls = lineStyle();
          return {
            ...ls,
            type: 'line',
            name: s.name,
            data: s.values,
            color: c,
            itemStyle: { color: c },
            lineStyle: {
              ...(ls.lineStyle as object),
              color: c,
              type: s.dashed ? 'dashed' : 'solid',
            },
            connectNulls: false,
            z: s.key === 'carteira' ? 3 : 2,
          };
        }),
      };
    },
    [data, extra],
  );

  type Row = { month: string; values: (number | null)[] };
  const rows: Row[] = perf.months.map((m, i) => ({
    month: m,
    values: data.series.map((s) => s.values[i] ?? null),
  }));
  const columns: TechColumn<Row>[] = [
    {
      key: 'month',
      header: tr('carteira-mes'),
      cell: (r) => formatMonthShort(r.month),
      width: '90px',
    },
    ...data.series.map((s, i): TechColumn<Row> => ({
      key: s.key,
      header: s.name,
      numeric: true,
      cell: (r) => pctText(r.values[i], { privacy }),
    })),
  ];
  const indice = perf.benchmark_label || 'CDI';

  return (
    <ChartFrame
      index={index}
      title={tr('carteira-rent-acumulada-titulo', { indice })}
      source={source}
      actions={actions}
      chart={
        <EChart
          option={option}
          height={300}
          ariaLabel={
            perf.months[0]
              ? tr('carteira-rent-acumulada-aria-desde', {
                  mes: formatMonthShort(perf.months[0]),
                  indice,
                })
              : tr('carteira-rent-acumulada-aria-inicio', { indice })
          }
        />
      }
      table={
        <TechTable
          label={tr('carteira-rent-acumulada-tabela')}
          prefsId="Rentabilidade acumulada por mês"
          columns={columns}
          rows={rows}
          getRowId={(r) => r.month}
          dense
        />
      }
    />
  );
}

/** FIG · Retorno de cada mês (TWR: Modified Dietz no mensal, dias encadeados no diário). */
export function MonthlyReturnChart({ index, perf }: { index: string; perf: Performance }) {
  const privacy = usePrivacy();
  const data = useMemo(() => monthlyReturns(perf), [perf]);
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      const base = baseOption(t);
      const bs = barStyle(t);
      return {
        ...base,
        legend: { show: false },
        grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
        tooltip: { ...(base.tooltip as object), ...axisTooltip(t, (v) => pctText(v, t)) },
        xAxis: categoryAxis(t, data.labels),
        yAxis: valueAxis(t, pctAxis),
        series: [
          {
            ...bs,
            type: 'bar',
            name: tr('carteira-retorno-mes'),
            data: data.values.map((v) => ({
              value: v,
              itemStyle: {
                ...(bs.itemStyle as object),
                color: (v ?? 0) < 0 ? t.negative : t.positive,
                borderRadius: (v ?? 0) < 0 ? [0, 0, 4, 4] : [4, 4, 0, 0],
              },
            })),
          },
        ],
      };
    },
    [data],
  );
  type Row = { month: string; value: number | null };
  const rows: Row[] = perf.months.map((m, i) => ({ month: m, value: data.values[i] ?? null }));
  return (
    <ChartFrame
      index={index}
      title={tr('carteira-retorno-cada-mes')}
      source={
        perf.method === 'daily'
          ? tr('carteira-risco-metodo-diario-fonte')
          : tr('carteira-retorno-metodo')
      }
      chart={<EChart option={option} height={220} ariaLabel={tr('carteira-retorno-aria')} />}
      table={
        <TechTable
          label={tr('carteira-retorno-mensal')}
          prefsId="Retorno mensal"
          dense
          rows={rows}
          getRowId={(r) => r.month}
          columns={[
            { key: 'month', header: tr('carteira-mes'), cell: (r) => formatMonthShort(r.month) },
            {
              key: 'value',
              header: tr('carteira-retorno'),
              numeric: true,
              cell: (r) => pctText(r.value, { privacy }),
            },
          ]}
        />
      }
    />
  );
}

/** FIG · Patrimônio investido × aportes líquidos (os dois em reais, um eixo). */
export function ValueVsContributionsChart({ index, perf }: { index: string; perf: Performance }) {
  const labels = useMemo(() => perf.months.map((m) => formatMonthShort(m)), [perf]);
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      const color = colorsFor(['patrimonio', 'aportes'], t);
      const patrimonio = tr('carteira-patrimonio');
      const aportes = tr('carteira-aportes-liquidos');
      const base = baseOption(t);
      const ls = lineStyle();
      const line = (name: string, key: string, data: number[], dashed = false) => ({
        ...ls,
        type: 'line',
        name,
        data,
        color: color(key),
        itemStyle: { color: color(key) },
        lineStyle: {
          ...(ls.lineStyle as object),
          color: color(key),
          type: dashed ? 'dashed' : 'solid',
        },
      });
      return {
        ...base,
        grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
        legend: { ...(base.legend as object), data: [patrimonio, aportes] },
        tooltip: {
          ...(base.tooltip as object),
          ...axisTooltip(t, (v) => (v == null ? '—' : tooltipMoney(v, t))),
        },
        xAxis: { ...categoryAxis(t, labels), boundaryGap: false },
        yAxis: valueAxis(t, (v) => compactMoney(v, t)),
        series: [
          line(patrimonio, 'patrimonio', perf.value_base),
          line(aportes, 'aportes', perf.net_contributions, true),
        ],
      };
    },
    [perf, labels],
  );
  type Row = { month: string; value: number; contrib: number };
  const rows: Row[] = perf.months.map((m, i) => ({
    month: m,
    value: perf.value_base[i] ?? 0,
    contrib: perf.net_contributions[i] ?? 0,
  }));
  return (
    <ChartFrame
      index={index}
      title={tr('carteira-patrimonio-aportes')}
      source={tr('carteira-patrimonio-aportes-fonte')}
      chart={
        <EChart option={option} height={220} ariaLabel={tr('carteira-patrimonio-aportes-aria')} />
      }
      table={
        <TechTable
          label={tr('carteira-patrimonio-aportes-tabela')}
          prefsId="Patrimônio e aportes por mês"
          dense
          rows={rows}
          getRowId={(r) => r.month}
          columns={[
            { key: 'month', header: tr('carteira-mes'), cell: (r) => formatMonthShort(r.month) },
            {
              key: 'value',
              header: tr('carteira-patrimonio'),
              numeric: true,
              cell: (r) => <Money cents={r.value} />,
            },
            {
              key: 'contrib',
              header: tr('carteira-aportes-liquidos'),
              numeric: true,
              cell: (r) => <Money cents={r.contrib} />,
            },
          ]}
        />
      }
    />
  );
}
