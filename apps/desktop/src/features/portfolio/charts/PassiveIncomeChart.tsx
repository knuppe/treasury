import { useCallback, useMemo } from 'react';

import type { PassiveIncome } from '@/api/generated/PassiveIncome';
import {
  barStyle,
  baseOption,
  categoryAxis,
  ChartFrame,
  compactMoney,
  EChart,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';
import { LegendItem, Money, TechTable } from '@/design/components';
import { t as tr } from '@/i18n';
import { formatMonthLong } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { passiveIncomeBars } from '../transform';
import { axisTooltip } from './common';

/** FIG · Proventos líquidos por mês nos últimos 12 meses, com a meta de renda mensal. */
export function PassiveIncomeChart({ index, data }: { index: string; data: PassiveIncome }) {
  const bars = useMemo(() => passiveIncomeBars(data), [data]);
  const target = data.target_monthly;
  const currency = useBaseCurrency();
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      const base = baseOption(t);
      const bs = barStyle(t);
      return {
        ...base,
        legend: { show: false },
        grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          ...axisTooltip(t, (v) => (v == null ? '—' : tooltipMoney(v, t))),
        },
        xAxis: categoryAxis(t, bars.labels),
        yAxis: {
          ...valueAxis(t, (v) => compactMoney(v, t)),
          max: target ? (v: { max: number }) => Math.max(v.max, target * 1.1) : undefined,
        },
        series: [
          {
            ...bs,
            type: 'bar',
            name: tr('carteira-grafico-proventos-liquidos'),
            data: bars.values,
            itemStyle: { ...(bs.itemStyle as object), color: t.accent },
            markLine: target
              ? {
                  silent: true,
                  symbol: 'none',
                  lineStyle: { color: t.warning, width: 2, type: 'dashed' },
                  label: {
                    color: t.warning,
                    fontFamily: t.fontNumeric,
                    fontSize: 10.5,
                    position: 'insideEndTop',
                    formatter: () =>
                      t.privacy
                        ? tr('carteira-meta-curta')
                        : tr('carteira-meta-valor', { valor: compactMoney(target, t) }),
                  },
                  data: [{ yAxis: target }],
                }
              : undefined,
          },
        ],
      };
    },
    [bars, target],
  );
  type Row = { month: string; amount: number };
  return (
    <ChartFrame
      index={index}
      title={tr('carteira-proventos-mes-titulo')}
      source={tr('carteira-proventos-mes-fonte', { moeda: currency })}
      actions={
        <span className="flex gap-3">
          <LegendItem color="var(--accent)">{tr('carteira-recebido')}</LegendItem>
          {target ? (
            <LegendItem color="var(--warning)" dashed>
              {tr('carteira-meta-mensal')}
            </LegendItem>
          ) : null}
        </span>
      }
      chart={<EChart option={option} height={240} ariaLabel={tr('carteira-proventos-mes-aria')} />}
      table={
        <TechTable<Row>
          label={tr('carteira-proventos-mes-tabela')}
          prefsId="Proventos por mês"
          dense
          rows={data.by_month}
          getRowId={(r) => r.month}
          columns={[
            { key: 'month', header: tr('carteira-mes'), cell: (r) => formatMonthLong(r.month) },
            {
              key: 'amount',
              header: tr('carteira-col-liquido'),
              numeric: true,
              cell: (r) => <Money cents={r.amount} />,
            },
          ]}
        />
      }
    />
  );
}
