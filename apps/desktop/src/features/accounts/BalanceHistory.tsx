import { useCallback, useMemo, useState } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import { useBalanceHistory } from '@/api/hooks/accounts';
import {
  ChartFrame,
  EChart,
  baseOption,
  categoryAxis,
  compactMoney,
  lineStyle,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import { Money, TechTable } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatLocale, todayIso } from '@/lib/format';
import { QueryState, Segmented, Switch } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';

type Range = '3m' | '6m' | '12m' | 'all';

function addMonthsIso(iso: string, months: number): string {
  const [y, m, d] = iso.split('-').map(Number) as [number, number, number];
  const first = new Date(Date.UTC(y, m - 1 + months, 1));
  const dim = new Date(Date.UTC(first.getUTCFullYear(), first.getUTCMonth() + 1, 0)).getUTCDate();
  return `${first.getUTCFullYear()}-${String(first.getUTCMonth() + 1).padStart(2, '0')}-${String(Math.min(d, dim)).padStart(2, '0')}`;
}

/** `2026-10-14` → `14/10` (`10/14` em inglês), para o eixo do gráfico. */
function shortDay(iso: string): string {
  const day = iso.slice(8, 10);
  const month = iso.slice(5, 7);
  return formatLocale() === 'en' ? `${month}/${day}` : `${day}/${month}`;
}

/** FIG. 02.x · saldo da conta ao longo do tempo (com projeção dos previstos, se pedida). */
export function BalanceHistory({ account }: { account: AccountView }) {
  const [range, setRange] = useState<Range>('6m');
  const [projection, setProjection] = useState(false);
  const today = todayIso();
  const from =
    range === 'all'
      ? account.opening_date
      : [addMonthsIso(today, range === '3m' ? -3 : range === '6m' ? -6 : -12), account.opening_date]
          .sort()
          .at(-1)!;
  const to = projection ? addMonthsIso(today, 2) : today;
  const days = (Date.parse(to) - Date.parse(from)) / 86_400_000;
  const interval = days > 400 ? 'month' : 'day';
  const {
    data = [],
    isLoading,
    error,
    refetch,
  } = useBalanceHistory({
    id: account.id,
    from,
    to,
    interval,
    include_planned: projection,
  });

  const labels = useMemo(() => data.map((p) => p.date), [data]);
  const option = useCallback(
    (theme: ChartTheme) => {
      const todayIndex = labels.findIndex((d) => d > today);
      return {
        ...baseOption(theme),
        legend: { show: false },
        grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
        tooltip: {
          ...(baseOption(theme).tooltip as object),
          trigger: 'axis',
          valueFormatter: (v: number) => tooltipMoney(v, theme, account.currency),
        },
        xAxis: (() => {
          const axis = categoryAxis(
            theme,
            labels.map((d) => shortDay(d)),
          );
          return {
            ...axis,
            boundaryGap: false,
            axisLabel: { ...(axis.axisLabel as object), hideOverlap: true },
          };
        })(),
        yAxis: valueAxis(theme, (v) => compactMoney(v, theme, account.currency)),
        series: [
          {
            name: t('contas-saldo'),
            type: 'line',
            data: data.map((p) => p.balance),
            ...lineStyle(),
            color: theme.accent,
            areaStyle: { color: theme.accent, opacity: 0.08 },
            ...(projection && todayIndex > 0
              ? {
                  markLine: {
                    symbol: 'none',
                    silent: true,
                    lineStyle: { color: theme.textDim, type: 'dashed' },
                    label: {
                      color: theme.textDim,
                      formatter: t('contas-hoje'),
                      fontFamily: theme.fontNumeric,
                      fontSize: 10,
                    },
                    data: [{ xAxis: todayIndex - 1 }],
                  },
                }
              : {}),
          },
        ],
      };
    },
    [data, labels, account.currency, projection, today],
  );

  const first = data[0]?.balance ?? 0;
  const last = data.at(-1)?.balance ?? 0;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-4">
        <Segmented<Range>
          label={t('contas-periodo-grafico')}
          size="sm"
          value={range}
          onChange={setRange}
          options={[
            { value: '3m', label: t('contas-n-meses', { n: 3 }) },
            { value: '6m', label: t('contas-n-meses', { n: 6 }) },
            { value: '12m', label: t('contas-n-meses', { n: 12 }) },
            { value: 'all', label: t('contas-desde-inicio') },
          ]}
        />
        <div className="min-w-64">
          <Switch
            checked={projection}
            onChange={setProjection}
            label={t('contas-incluir-previstos')}
          />
        </div>
        {data.length > 1 ? (
          <span className="ml-auto text-[12.5px] text-dim">
            {withSlots(t('contas-variacao-periodo', { valor: slot(0) }), [
              <Money cents={last - first} currency={account.currency} tone="auto" trend />,
            ])}
          </span>
        ) : null}
      </div>
      <QueryState loading={isLoading} error={error} onRetry={() => void refetch()}>
        <ChartFrame
          index="03.1"
          title={t('contas-saldo-de', { nome: account.name })}
          source={t('contas-saldo-fonte', {
            previstos: projection ? 'sim' : 'nao',
            intervalo: interval === 'day' ? 'dia' : 'mes',
          })}
          chart={
            <EChart
              option={option}
              height={240}
              ariaLabel={t('contas-saldo-grafico', {
                nome: account.name,
                de: formatDate(from),
                ate: formatDate(to),
              })}
            />
          }
          table={
            <TechTable
              label={t('contas-saldo-por-data', { nome: account.name })}
              prefsId="Saldo da conta por data"
              dense
              rows={
                interval === 'day'
                  ? data.filter((_, i) => i % 7 === 0 || i === data.length - 1)
                  : data
              }
              getRowId={(p) => p.date}
              columns={[
                { key: 'date', header: t('contas-data'), cell: (p) => formatDate(p.date) },
                {
                  key: 'balance',
                  header: t('contas-saldo'),
                  numeric: true,
                  cell: (p) => <Money cents={p.balance} currency={account.currency} />,
                },
              ]}
            />
          }
        />
      </QueryState>
    </div>
  );
}
