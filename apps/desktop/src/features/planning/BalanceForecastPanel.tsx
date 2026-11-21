/**
 * Previsão de saldo (docs/forecast.md): o saldo de cada conta do dia a dia e o total, de
 * hoje até 3, 6 ou 12 meses, pelos previstos, recorrências, faturas e parcelas; avisa a
 * primeira data em que uma conta fica negativa.
 */
import { useCallback, useMemo, useState } from 'react';

import type { AccountForecast } from '@/api/generated/AccountForecast';
import type { ForecastSource } from '@/api/generated/ForecastSource';
import { useBalanceForecast } from '@/api/hooks/planning';
import {
  ChartFrame,
  EChart,
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import { Icon, InstrumentCard, Money, TechTable } from '@/design/components';
import { CoverageLine } from '@/features/attention/CoverageLine';
import { tRich } from '@/features/settings/richText';
import { QueryState, Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

type Horizon = '3' | '6' | '12';

const SOURCE: Record<ForecastSource, string> = {
  get planned() {
    return t('planejamento-origem-previsto');
  },
  get recurrence() {
    return t('planejamento-origem-recorrencia');
  },
  get card_statement() {
    return t('planejamento-origem-fatura');
  },
  get detected() {
    return t('planejamento-origem-parcela-detectada');
  },
  get loan() {
    return t('planejamento-origem-parcela');
  },
  get seasonal() {
    return t('planejamento-origem-sazonal');
  },
};

/** Dia e mês, na ordem do idioma (`14/10` ou `10/14`). */
const short = (d: string) => formatDate(d).slice(0, 5);

/** Saldo de uma conta numa data (degraus: vale o último ponto até a data). */
export function balanceOn(a: Pick<AccountForecast, 'points' | 'start'>, date: string): number {
  let v = a.start;
  for (const p of a.points) {
    if (p.date > date) break;
    v = p.balance;
  }
  return v;
}

export function BalanceForecastPanel() {
  const [months, setMonths] = useState<Horizon>('3');
  const q = useBalanceForecast({ months: Number(months) });
  const f = q.data;

  const dates = useMemo(
    () =>
      f
        ? [
            ...new Set([
              ...f.total_points.map((p) => p.date),
              ...f.accounts.flatMap((a) => a.points.map((p) => p.date)),
            ]),
          ]
            .filter((d) => d <= f.to)
            .sort()
        : [],
    [f],
  );
  const option = useCallback(
    (theme: ChartTheme) => {
      if (!f) return {};
      const base = baseOption(theme);
      return {
        ...base,
        legend: { ...(base.legend as object), show: true, top: 0 },
        grid: { left: 8, right: 16, top: 32, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'axis',
          valueFormatter: (v: number) => tooltipMoney(v, theme, f.currency),
        },
        xAxis: (() => {
          const axis = categoryAxis(theme, dates.map(short));
          return {
            ...axis,
            boundaryGap: false,
            axisLabel: { ...(axis.axisLabel as object), hideOverlap: true },
          };
        })(),
        yAxis: valueAxis(theme, (v) => compactMoney(v, theme, f.currency)),
        series: [
          ...f.accounts
            .filter((a) => a.kind !== 'credit_card' && a.currency === f.currency)
            .map((a, i) => ({
              name: a.name,
              type: 'line',
              step: 'end',
              symbol: 'none',
              color: theme.series[i % theme.series.length],
              lineStyle: { width: 1.5 },
              data: dates.map((d) => balanceOn(a, d)),
            })),
          {
            name: t('planejamento-previsao-total'),
            type: 'line',
            step: 'end',
            symbol: 'none',
            color: theme.text,
            lineStyle: { width: 2, type: 'dashed' },
            data: dates.map((d) => balanceOn({ start: f.total_start, points: f.total_points }, d)),
            markLine: {
              symbol: 'none',
              silent: true,
              lineStyle: { color: theme.negative, type: 'dotted' },
              label: { show: false },
              data: [{ yAxis: 0 }],
            },
          },
        ],
      };
    },
    [f, dates],
  );

  const negatives = f?.accounts.filter((a) => a.first_negative) ?? [];
  const events = useMemo(
    () =>
      (f?.accounts ?? [])
        .flatMap((a) =>
          a.events.map((e, i) => ({
            ...e,
            account: a.name,
            currency: a.currency,
            id: `${a.account_id}:${i}`,
          })),
        )
        .sort((x, y) => x.date.localeCompare(y.date))
        .slice(0, 200),
    [f],
  );

  return (
    <InstrumentCard
      index="03.F"
      title={t('planejamento-previsao-titulo')}
      actions={
        <Segmented<Horizon>
          label={t('planejamento-previsao-horizonte')}
          size="sm"
          value={months}
          onChange={setMonths}
          options={[
            { value: '3', label: t('planejamento-n-meses', { n: 3 }) },
            { value: '6', label: t('planejamento-n-meses', { n: 6 }) },
            { value: '12', label: t('planejamento-n-meses', { n: 12 }) },
          ]}
        />
      }
      footer={
        f
          ? [
              {
                label: t('planejamento-previsao-total-hoje'),
                value: <Money cents={f.total_start} currency={f.currency} />,
              },
              {
                label: t('planejamento-previsao-em', { data: formatDate(f.to) }),
                value: <Money cents={f.total_end} currency={f.currency} />,
              },
              {
                label: t('planejamento-previsao-ponto-baixo'),
                value: (
                  <span>
                    {tRich('planejamento-previsao-valor-em', {
                      valor: <Money cents={f.total_min} currency={f.currency} />,
                      data: short(f.total_min_date),
                    })}
                  </span>
                ),
              },
            ]
          : undefined
      }
    >
      <QueryState loading={q.isPending} error={q.error} onRetry={() => void q.refetch()}>
        {f ? (
          <div className="flex flex-col gap-4">
            <CoverageLine onlyPartial />
            {negatives.length ? (
              <ul className="flex flex-col gap-1 rounded-card border border-negative/50 bg-negative/5 px-3 py-2 text-[13px] text-negative">
                {negatives.map((a) => (
                  <li key={a.account_id} className="flex items-center gap-2">
                    <Icon name="alert" size={13} />
                    <span>
                      {a.first_negative === f.from
                        ? tRich('planejamento-previsao-ja-negativa', {
                            conta: a.name,
                            minimo: <Money cents={a.min} currency={a.currency} />,
                            quando: formatDate(a.min_date),
                          })
                        : tRich('planejamento-previsao-negativa', {
                            conta: a.name,
                            data: formatDate(a.first_negative!),
                            minimo: <Money cents={a.min} currency={a.currency} />,
                            quando: formatDate(a.min_date),
                          })}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="flex items-center gap-2 text-[12.5px] text-positive">
                <Icon name="check" size={13} />{' '}
                {t('planejamento-previsao-nenhuma-negativa', { data: formatDate(f.to) })}
              </p>
            )}
            <ChartFrame
              index="03.F1"
              title={t('planejamento-previsao-por-conta')}
              source={t('planejamento-previsao-fonte')}
              chart={
                <EChart
                  option={option}
                  height={260}
                  ariaLabel={t('planejamento-previsao-grafico', {
                    de: formatDate(f.from),
                    ate: formatDate(f.to),
                  })}
                />
              }
              table={
                <TechTable
                  label={t('planejamento-previsao-eventos')}
                  prefsId="Previsão de saldo: eventos"
                  dense
                  rows={events}
                  getRowId={(e) => e.id}
                  columns={[
                    {
                      key: 'date',
                      header: t('planejamento-data'),
                      width: '96px',
                      cell: (e) => formatDate(e.date),
                    },
                    { key: 'account', header: t('planejamento-previsao-conta') },
                    { key: 'description', header: t('planejamento-previsao-descricao') },
                    {
                      key: 'source',
                      header: t('planejamento-previsao-origem'),
                      cell: (e) => SOURCE[e.source],
                    },
                    {
                      key: 'amount',
                      header: t('planejamento-valor'),
                      numeric: true,
                      cell: (e) => <Money cents={e.amount} currency={e.currency} tone="auto" />,
                    },
                  ]}
                />
              }
            />
          </div>
        ) : null}
      </QueryState>
    </InstrumentCard>
  );
}
