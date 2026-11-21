/**
 * Painel de uso: tokens e custo estimado por mês, provider, motor e tarefa. CLIs e
 * modelos locais aparecem com custo zero no app.
 */
import { useCallback, useMemo, useState } from 'react';

import type { UsageGroup } from '@/api/generated/UsageGroup';
import { useUsageSummary } from '@/api/hooks/ai';
import { ChartFrame, EChart, type ChartTheme } from '@/charts';
import {
  InstrumentCard,
  Measure,
  Private,
  Select,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { ErrorBlock, LoadingBlock } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDate, formatNumber, todayIso } from '@/lib/format';

import { ENGINE_KIND_LABEL, formatCost, formatTokens } from '../format';
import { usageBarsOption, usageMonths } from './usageChart';

const periods = () => [
  { value: '1', label: t('agente-periodo-1') },
  { value: '3', label: t('agente-periodo-3') },
  { value: '6', label: t('agente-periodo-6') },
  { value: '12', label: t('agente-periodo-12') },
];

type GroupBy = 'provider' | 'engine' | 'task';

/** Nome de cada tabela "Uso por…" (e o das preferências, que fica em pt-BR). */
const GROUP_TABLE = {
  provider: ['agente-uso-por-provider', 'Uso por provider'],
  engine: ['agente-uso-por-motor', 'Uso por motor'],
  task: ['agente-uso-por-tarefa', 'Uso por tarefa'],
} as const;

/** Primeiro dia do mês, `n - 1` meses antes de hoje. */
export function periodFrom(today: string, months: number): string {
  let y = Number(today.slice(0, 4));
  let m = Number(today.slice(5, 7)) - (months - 1);
  while (m < 1) {
    m += 12;
    y -= 1;
  }
  return `${y}-${String(m).padStart(2, '0')}-01`;
}

const groupColumns = (): TechColumn<UsageGroup>[] => [
  {
    key: 'label',
    header: t('agente-nome'),
    cell: (g) => (
      <span>
        {g.label}
        {g.kind ? (
          <span className="ml-2 text-[11px] text-dim">{ENGINE_KIND_LABEL[g.kind]}</span>
        ) : null}
      </span>
    ),
  },
  {
    key: 'calls',
    header: t('agente-chamadas'),
    numeric: true,
    cell: (g) => <Private>{g.totals.calls}</Private>,
  },
  {
    key: 'tokens',
    header: t('agente-tokens-entrada-saida'),
    numeric: true,
    cell: (g) => (
      <Private>
        {`${formatTokens(g.totals.input_tokens)} / ${formatTokens(g.totals.output_tokens)}`}
      </Private>
    ),
  },
  {
    key: 'cost',
    header: t('agente-col-custo'),
    numeric: true,
    cell: (g) => (
      <Private>
        {formatCost(g.totals.cost_micros)}
        {g.totals.unpriced_calls > 0 ? <span className="text-dim">{' *'}</span> : null}
      </Private>
    ),
  },
  {
    key: 'errors',
    header: t('agente-erros'),
    numeric: true,
    cell: (g) =>
      g.totals.errors > 0 ? <span className="text-negative">{g.totals.errors}</span> : '0',
  },
];

function GroupTable({ by, rows }: { by: GroupBy; rows: readonly UsageGroup[] }) {
  const [key, prefsId] = GROUP_TABLE[by];
  return (
    <TechTable
      label={t(key)}
      prefsId={prefsId}
      columns={groupColumns()}
      rows={rows}
      getRowId={(g) => g.key}
      dense
      empty={t('agente-sem-uso')}
    />
  );
}

export function UsagePanel({ index = '04' }: { index?: string }) {
  const [period, setPeriod] = useState<string>('6');
  const today = todayIso();
  const from = periodFrom(today, Number(period));
  const usage = useUsageSummary({ from, to: today });
  const [by, setBy] = useState<GroupBy>('provider');

  const months = useMemo(
    () => (usage.data ? usageMonths(usage.data.by_month, from, today) : []),
    [usage.data, from, today],
  );
  const option = useCallback(
    (theme: ChartTheme) => usageBarsOption(theme, months, usage.data?.currency),
    [months, usage.data?.currency],
  );
  const s = usage.data;

  return (
    <InstrumentCard
      index={index}
      title={t('agente-uso-titulo')}
      actions={
        <div className="w-[190px]">
          <Select
            aria-label={t('agente-periodo')}
            value={period}
            onValueChange={setPeriod}
            options={periods()}
          />
        </div>
      }
      footer={
        s
          ? [
              { label: t('agente-periodo'), value: `${formatDate(s.from)} – ${formatDate(s.to)}` },
              { label: t('agente-moeda'), value: s.currency },
              { label: t('agente-fonte'), value: t('agente-fonte-uso') },
            ]
          : undefined
      }
    >
      {usage.isLoading ? (
        <LoadingBlock />
      ) : usage.error ? (
        <ErrorBlock error={usage.error} onRetry={() => void usage.refetch()} />
      ) : s ? (
        <div className="flex flex-col gap-5">
          <div className="grid grid-cols-2 gap-4 @min-[530px]:grid-cols-4">
            <Measure
              size="md"
              label={t('agente-custo-estimado')}
              value={<Private>{formatCost(s.totals.cost_micros, s.currency)}</Private>}
            />
            <Measure
              size="md"
              label={t('agente-chamadas')}
              value={<Private>{formatNumber(s.totals.calls)}</Private>}
            />
            <Measure
              size="md"
              label={t('agente-tokens')}
              value={
                <Private>{formatTokens(s.totals.input_tokens + s.totals.output_tokens)}</Private>
              }
            />
            <Measure size="md" label={t('agente-erros')} value={formatNumber(s.totals.errors)} />
          </div>
          {s.totals.unpriced_calls > 0 ? (
            <p className="text-[12px] text-warning">
              {t('agente-sem-preco', { n: s.totals.unpriced_calls })}
            </p>
          ) : null}
          <ChartFrame
            index="01"
            title={t('agente-custo-mes')}
            source={t('agente-custo-fonte')}
            chart={<EChart option={option} height={220} ariaLabel={t('agente-custo-ia-mes')} />}
            table={
              <TechTable
                label={t('agente-custo-ia-mes')}
                prefsId="Custo estimado de IA por mês"
                dense
                rows={months}
                getRowId={(m) => m.key}
                columns={[
                  { key: 'label', header: t('agente-mes'), sortValue: (m) => m.key },
                  { key: 'calls', header: t('agente-chamadas'), numeric: true },
                  {
                    key: 'tokens',
                    header: t('agente-tokens'),
                    numeric: true,
                    sortValue: (m) => m.inputTokens + m.outputTokens,
                    cell: (m) => formatTokens(m.inputTokens + m.outputTokens),
                  },
                  {
                    key: 'cost',
                    header: t('agente-col-custo'),
                    numeric: true,
                    sortValue: (m) => m.costMicros,
                    cell: (m) => <Private>{formatCost(m.costMicros, s.currency)}</Private>,
                  },
                ]}
              />
            }
          />
          <div className="flex flex-col gap-2">
            <div className="flex items-center gap-3">
              <span className="label-caps">{t('agente-por')}</span>
              <div className="w-[160px]">
                <Select
                  aria-label={t('agente-agrupar')}
                  value={by}
                  onValueChange={(v) => setBy(v as GroupBy)}
                  options={[
                    { value: 'provider', label: t('agente-provider') },
                    { value: 'engine', label: t('agente-motor') },
                    { value: 'task', label: t('agente-tarefa') },
                  ]}
                />
              </div>
            </div>
            <GroupTable
              by={by}
              rows={by === 'provider' ? s.by_provider : by === 'engine' ? s.by_engine : s.by_task}
            />
          </div>
        </div>
      ) : null}
    </InstrumentCard>
  );
}
