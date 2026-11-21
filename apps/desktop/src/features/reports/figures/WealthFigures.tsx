/**
 * Figuras 08 · Titulares (barras agrupadas), 09 · Recorrentes (linha do tempo),
 * 10 · Cartões (faturas futuras) e 11 · Patrimônio (área empilhada).
 */
import { useMemo } from 'react';

import {
  useAccounts,
  useByMember,
  useCategories,
  useFutureCommitmentsByCard,
  useNetWorthHistory,
  useRecurrences,
} from '@/api/hooks/reports';
import { ChartFrame } from '@/charts/ChartFrame';
import { GroupedBars } from '@/charts/GroupedBars';
import { RecurrenceTimeline, type TimelineRow } from '@/charts/RecurrenceTimeline';
import { StackedArea } from '@/charts/StackedArea';
import { cssColor } from '@/charts/colorMath';
import { EmptyState, Money, Private, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, formatMonthShort, formatPercent } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { addDays, monthOf } from '@/lib/months';

import { Loading, QueryView } from '../controls';
import { openExplain } from '../explain/store';
import { OTHERS_KEY } from '../model/colors';
import { cardMonthDrill, memberDrill, recurrenceDrill } from '../model/drill';
import { periodLabel } from '../model/period';
import {
  cardsModel,
  membersModel,
  NO_CATEGORY_KEY,
  netWorthModel,
  recurrenceSummary,
  timelineRows,
} from '../model/series';
import {
  NO_MEMBER_KEY,
  useAccountColors,
  useCategoryColors,
  useMemberColors,
} from '../useEntityColors';
import {
  basisLabel,
  FilterNote,
  Question,
  Stat,
  Stats,
  useDrill,
  type FigureProps,
} from './common';

// ── 08 · Titulares ───────────────────────────────────────────────

export function MembersFigure({ period, basis, accountId }: FigureProps) {
  const query = useByMember({ ...period, basis: basis ?? undefined });
  const colors = useMemberColors();
  const drill = useDrill();
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.total <= 0}
      loadingLabel={t('relatorios-carregando-membros')}
    >
      {(data) => {
        const { input, folded } = membersModel(data, colors, NO_MEMBER_KEY);
        type Row = { key: string; label: string; values: number[]; total: number };
        const rows: Row[] = input.rows.map((r, i) => {
          const values = input.groups.map((g) => g.values[i] ?? 0);
          return { key: r.key, label: r.label, values, total: values.reduce((s, v) => s + v, 0) };
        });
        const columns: TechColumn<Row>[] = [
          { key: 'label', header: t('relatorios-categoria') },
          ...input.groups.map((g, j): TechColumn<Row> => ({
            key: g.key,
            header: g.name,
            numeric: true,
            sortValue: (r) => r.values[j] ?? 0,
            cell: (r) => <Money cents={r.values[j] ?? 0} />,
          })),
          {
            key: 'total',
            header: t('relatorios-total'),
            numeric: true,
            sortValue: (r) => r.total,
            cell: (r) => <Money cents={r.total} />,
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-membros-pergunta')}</Question>
            <Stats>
              {data.members.slice(0, 4).map((m) => (
                <Stat
                  key={m.key ?? 'none'}
                  label={
                    <span className="inline-flex items-center gap-1.5">
                      <span
                        aria-hidden
                        className="size-2 rounded-[2px]"
                        style={{
                          background: cssColor(colors.get(m.key ?? NO_MEMBER_KEY) ?? '@gray'),
                        }}
                      />
                      {m.label}
                    </span>
                  }
                  hint={
                    <Private>
                      {t('relatorios-do-total', { pct: formatPercent(m.share, { decimals: 0 }) })}
                    </Private>
                  }
                >
                  <Money cents={m.amount} />
                </Stat>
              ))}
            </Stats>
            <FilterNote memberId={null} accountId={accountId} />
            <ChartFrame
              index="08"
              title={t('relatorios-membros-titulo', { periodo: periodLabel(period) })}
              source={
                folded.length
                  ? t('relatorios-membros-fonte-outras', {
                      base: basisLabel(data.basis),
                      n: input.rows.length,
                      outras: folded.length,
                    })
                  : t('relatorios-membros-fonte', {
                      base: basisLabel(data.basis),
                      n: input.rows.length,
                    })
              }
              chart={
                <GroupedBars
                  input={input}
                  ariaLabel={t('relatorios-membros-aria', { periodo: periodLabel(period) })}
                  onBarClick={(rowKey, groupKey) => {
                    const cats = rowKey === OTHERS_KEY ? folded : [rowKey];
                    const f = memberDrill(
                      { period, basis: data.basis },
                      groupKey === NO_MEMBER_KEY ? null : groupKey,
                      undefined,
                    );
                    const ids = cats.filter((c) => c !== NO_CATEGORY_KEY);
                    if (ids.length) f.category_ids = ids;
                    else if (cats.includes(NO_CATEGORY_KEY)) f.uncategorized = true;
                    drill(f);
                  }}
                />
              }
              table={
                <TechTable
                  label={t('relatorios-membros-tabela')}
                  prefsId="Gasto por membro e categoria"
                  columns={columns}
                  rows={rows}
                  getRowId={(r) => r.key}
                  dense
                />
              }
            />
          </div>
        );
      }}
    </QueryView>
  );
}

// ── 09 · Recorrentes ─────────────────────────────────────────────

const TIMELINE_DAYS = 92;

export function RecurringFigure({ today }: FigureProps) {
  const query = useRecurrences({});
  const categories = useCategories({ include_archived: true });
  const colors = useCategoryColors();
  const drill = useDrill();
  const window = useMemo(() => ({ from: today, to: addDays(today, TIMELINE_DAYS) }), [today]);
  return (
    <QueryView
      query={query}
      isEmpty={(d) => !d.some((r) => r.active && r.kind === 'expense')}
      empty={
        <EmptyState
          kicker={t('relatorios-sem-recorrencias')}
          title={t('relatorios-sem-recorrencias-titulo')}
          description={t('relatorios-sem-recorrencias-descricao')}
        />
      }
      loadingLabel={t('relatorios-carregando-recorrencias')}
    >
      {(list) => {
        const summary = recurrenceSummary(list);
        const rows = timelineRows(list, categories.data ?? [], colors, window);
        const columns: TechColumn<TimelineRow>[] = [
          { key: 'label', header: t('graficos-recorrencia'), sortValue: (r) => r.label },
          {
            key: 'detail',
            header: t('relatorios-regra'),
            cell: (r) => <span className="text-dim">{r.detail}</span>,
          },
          {
            key: 'amount',
            header: t('graficos-valor'),
            numeric: true,
            sortValue: (r) => r.amount,
            cell: (r) => <Money cents={r.amount} currency={r.currency} />,
          },
          {
            key: 'monthly',
            header: t('graficos-por-mes'),
            numeric: true,
            sortValue: (r) => r.monthly,
            cell: (r) => <Money cents={r.monthly} currency={r.currency} />,
          },
          {
            key: 'next',
            header: t('relatorios-proximas'),
            cell: (r) =>
              r.dates
                .slice(0, 3)
                .map((d) => formatDate(d).slice(0, 5))
                .join(' · '),
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-recorrentes-pergunta')}</Question>
            <Stats>
              <Stat label={t('relatorios-comprometido-mes')}>
                <Money cents={summary.expenseMonthly} />
              </Stat>
              <Stat label={t('relatorios-receitas-recorrentes')}>
                <Money cents={summary.incomeMonthly} />
              </Stat>
              <Stat label={t('relatorios-das-receitas-recorrentes')}>
                <Private>
                  {summary.committedShare == null ? '—' : formatPercent(summary.committedShare)}
                </Private>
              </Stat>
              <Stat label={t('relatorios-despesas-recorrentes')}>
                <Private>{summary.count}</Private>
              </Stat>
            </Stats>
            <ChartFrame
              index="09"
              title={t('relatorios-recorrentes-titulo', { n: TIMELINE_DAYS })}
              source={t('relatorios-recorrentes-fonte')}
              chart={
                <RecurrenceTimeline
                  rows={rows}
                  from={window.from}
                  to={window.to}
                  today={today}
                  label={t('relatorios-recorrentes-aria', { n: TIMELINE_DAYS })}
                  onOccurrenceClick={(r) => {
                    const rec = list.find((x) => x.id === r.id);
                    if (rec) drill(recurrenceDrill(rec.description, rec.account_id));
                  }}
                />
              }
              table={
                <TechTable
                  label={t('relatorios-recorrencias')}
                  prefsId="Recorrências"
                  columns={columns}
                  rows={rows}
                  getRowId={(r) => r.id}
                  dense
                />
              }
            />
          </div>
        );
      }}
    </QueryView>
  );
}

// ── 10 · Cartões ─────────────────────────────────────────────────

const CARD_MONTHS = 12;
const isCard = (kind: string) => kind === 'credit_card';

export function CardsFigure({ today }: FigureProps) {
  const accounts = useAccounts();
  const cards = useMemo(
    () => (accounts.data ?? []).filter((a) => isCard(a.kind) && !a.archived),
    [accounts.data],
  );
  const results = useFutureCommitmentsByCard(
    cards.map((c) => c.id),
    CARD_MONTHS,
  );
  const colors = useAccountColors(isCard);
  const drill = useDrill();
  const base = useBaseCurrency();
  if (accounts.isPending || results.some((r) => r.isPending))
    return <Loading label={t('relatorios-carregando-cartoes')} />;
  if (cards.length === 0)
    return (
      <EmptyState
        kicker={t('relatorios-sem-cartoes')}
        title={t('relatorios-sem-cartoes-titulo')}
        description={t('relatorios-sem-cartoes-descricao')}
      />
    );
  const model = cardsModel(
    cards.map((c, i) => ({ id: c.id, name: c.name, data: results[i]?.data })),
    colors,
    base,
  );
  const next = model.totals[1] ?? 0;
  type Row = { month: string; values: number[]; total: number };
  const rows: Row[] = model.months.map((m, i) => ({
    month: m,
    values: model.series.map((s) => s.values[i] ?? 0),
    total: model.totals[i] ?? 0,
  }));
  const columns: TechColumn<Row>[] = [
    {
      key: 'month',
      header: t('relatorios-fatura'),
      numeric: true,
      cell: (r) => formatMonthShort(r.month),
    },
    ...model.series.map((s, j): TechColumn<Row> => ({
      key: s.key,
      header: s.name,
      numeric: true,
      cell: (r) => <Money cents={r.values[j] ?? 0} />,
    })),
    {
      key: 'total',
      header: t('relatorios-total'),
      numeric: true,
      cell: (r) => <Money cents={r.total} />,
    },
  ];
  return (
    <div className="flex flex-col gap-4">
      <Question>{t('relatorios-cartoes-pergunta')}</Question>
      <Stats>
        <Stat label={t('relatorios-proximos-meses', { n: CARD_MONTHS })}>
          <Money cents={model.total} />
        </Stat>
        <Stat
          label={t('relatorios-a-pagar-em', {
            mes: model.months[0] ? formatMonthShort(model.months[0]) : '—',
          })}
        >
          <Money cents={model.totals[0] ?? 0} />
        </Stat>
        <Stat
          label={t('relatorios-proxima-fatura')}
          hint={model.months[1] ? formatMonthLong(model.months[1]) : undefined}
        >
          <Money cents={next} />
        </Stat>
        <Stat label={t('relatorios-vista-cartoes')}>
          <Private>{cards.length}</Private>
        </Stat>
      </Stats>
      <ChartFrame
        index="10"
        title={t('relatorios-cartoes-titulo', { mes: formatMonthShort(monthOf(today)) })}
        source={t('relatorios-cartoes-fonte')}
        chart={
          <StackedArea
            input={model}
            height={300}
            ariaLabel={t('relatorios-cartoes-aria', { n: CARD_MONTHS })}
            onPointClick={(key, i) => {
              const month = model.months[i];
              if (month) drill(cardMonthDrill(key, month));
            }}
          />
        }
        table={
          <TechTable
            label={t('relatorios-cartoes-tabela')}
            prefsId="Faturas futuras por cartão"
            columns={columns}
            rows={rows}
            getRowId={(r) => r.month}
            dense
          />
        }
      />
    </div>
  );
}

// ── 11 · Patrimônio ──────────────────────────────────────────────

const NET_WORTH_MONTHS = 24;

export function NetWorthFigure({ period }: FigureProps) {
  const end = monthOf(period.to);
  const query = useNetWorthHistory({ months: NET_WORTH_MONTHS, end_month: end });
  const base = useBaseCurrency();
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.length === 0}
      loadingLabel={t('relatorios-carregando-patrimonio')}
    >
      {(points) => {
        const input = netWorthModel(points, base);
        const first = points[0]!;
        const last = points[points.length - 1]!;
        const change = last.net - first.net;
        const estimated = points.some((p) => p.estimated);
        const missing = [...new Set(points.flatMap((p) => p.missing_fx))];
        type Row = (typeof points)[number];
        const columns: TechColumn<Row>[] = [
          {
            key: 'month',
            header: t('relatorios-mes'),
            numeric: true,
            cell: (r) => formatMonthShort(r.month),
          },
          {
            key: 'accounts',
            header: t('relatorios-contas'),
            numeric: true,
            cell: (r) => <Money cents={r.assets - r.investments - r.physical} />,
          },
          {
            key: 'investments',
            header: t('relatorios-investimentos'),
            numeric: true,
            cell: (r) => <Money cents={r.investments} />,
          },
          {
            key: 'physical',
            header: t('relatorios-bens'),
            numeric: true,
            cell: (r) => <Money cents={r.physical} />,
          },
          {
            key: 'liabilities',
            header: t('relatorios-dividas'),
            numeric: true,
            cell: (r) => <Money cents={-r.liabilities} />,
          },
          {
            key: 'net',
            header: t('relatorios-liquido'),
            numeric: true,
            cell: (r) => <Money cents={r.net} estimated={r.estimated} />,
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-patrimonio-pergunta')}</Question>
            <Stats>
              <Stat label={t('relatorios-liquido-em', { mes: formatMonthShort(last.month) })}>
                <button
                  type="button"
                  className="rounded-control text-left hover:underline focus-visible:outline-2 focus-visible:outline-accent"
                  onClick={() =>
                    openExplain({
                      title: t('relatorios-patrimonio-liquido-em', {
                        data: formatDate(last.as_of),
                      }),
                      args: { metric: 'net_worth', params: { as_of: last.as_of } },
                    })
                  }
                  aria-label={t('relatorios-explicar-patrimonio', { data: formatDate(last.as_of) })}
                >
                  <Money cents={last.net} estimated={last.estimated} />
                </button>
              </Stat>
              <Stat label={t('relatorios-variacao-desde', { mes: formatMonthShort(first.month) })}>
                <Money cents={change} tone="auto" trend />
              </Stat>
              <Stat label={t('relatorios-investimentos')}>
                <Money cents={last.investments} />
              </Stat>
              <Stat label={t('relatorios-bens')}>
                <Money cents={last.physical} />
              </Stat>
              <Stat label={t('relatorios-dividas')}>
                <Money cents={-last.liabilities} />
              </Stat>
            </Stats>
            <ChartFrame
              index="11"
              title={t('relatorios-patrimonio-titulo', {
                n: points.length,
                mes: formatMonthShort(end),
              })}
              source={t('relatorios-patrimonio-fonte', {
                estimado: estimated ? 'sim' : 'nao',
                semcambio: missing.length ? 'sim' : 'nao',
                moedas: missing.join(', '),
              })}
              chart={
                <StackedArea
                  input={input}
                  height={340}
                  ariaLabel={t('relatorios-patrimonio-aria', {
                    n: points.length,
                    mes: formatMonthLong(end),
                  })}
                  onPointClick={(_key, i) => {
                    const p = points[i];
                    if (p)
                      openExplain({
                        title: t('relatorios-patrimonio-liquido-em', { data: formatDate(p.as_of) }),
                        args: { metric: 'net_worth', params: { as_of: p.as_of } },
                      });
                  }}
                />
              }
              table={
                <TechTable
                  label={t('relatorios-patrimonio-tabela')}
                  prefsId="Patrimônio por mês"
                  columns={columns}
                  rows={points}
                  getRowId={(r) => r.month}
                  dense
                />
              }
            />
          </div>
        );
      }}
    </QueryView>
  );
}
