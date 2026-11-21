/**
 * Figuras 04 · Tendência (barras empilhadas + média móvel), 05 · Orçamento (bullet, com o grupo),
 * 06 · Comparação (A × B). A 07 · Inflação fica em InflationFigure.tsx.
 */
import { useMemo, useState } from 'react';
import { Link } from 'react-router';

import { useBudget } from '@/api/hooks/budget';
import type { SpendGroup } from '@/api/generated/SpendGroup';
import { useCategories, useCompare, useMonthlyEvolution } from '@/api/hooks/reports';
import { BulletBar, BulletStatusLabel } from '@/charts/BulletBar';
import { bulletStatus, type BulletStatus } from '@/charts/bullet';
import { ChartFrame } from '@/charts/ChartFrame';
import { ComparisonChart } from '@/charts/ComparisonChart';
import type { ComparisonInput } from '@/charts/comparison';
import { StackedBars } from '@/charts/StackedBars';
import { cssColor } from '@/charts/colorMath';
import { EmptyState, Icon, Money, Private, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { formatMonthLong, formatMonthShort, formatPercent } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { monthBounds, monthOf } from '@/lib/months';

import { QueryView, Segmented } from '../controls';
import { Explainable } from '../explain/ExplainButton';
import { OTHERS_KEY } from '../model/colors';
import { categoryDrill, monthDrill } from '../model/drill';
import { groupName } from '../model/groups';
import { periodLabel, previousPeriod, trendWindow, yearBeforePeriod } from '../model/period';
import { comparisonRows, NO_CATEGORY_KEY, slopeRows, trendModel } from '../model/series';
import { useCategoryColors } from '../useEntityColors';
import {
  basisLabel,
  FilterNote,
  Question,
  Stat,
  Stats,
  useDrill,
  type FigureProps,
} from './common';

// ── 04 · Tendência ───────────────────────────────────────────────

export function TrendFigure({ period, basis, memberId, accountId }: FigureProps) {
  const win = trendWindow(period);
  const query = useMonthlyEvolution({
    months: win.months,
    end_month: win.endMonth,
    top_n: 99,
    basis: basis ?? undefined,
  });
  const colors = useCategoryColors();
  const drill = useDrill();
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.totals.every((v) => v === 0)}
      loadingLabel={t('relatorios-carregando-tendencia')}
    >
      {(data) => {
        const model = trendModel(data, colors);
        const n = data.months.length;
        const avgAll = Math.round(data.totals.reduce((s, v) => s + v, 0) / Math.max(1, n));
        type Row = { month: string; values: number[]; total: number; avg: number; income: number };
        const rows: Row[] = data.months.map((m, i) => ({
          month: m,
          values: model.input.bars.map((b) => b.values[i] ?? 0),
          total: data.totals[i] ?? 0,
          avg: data.moving_avg_3[i] ?? 0,
          income: data.income[i] ?? 0,
        }));
        const columns: TechColumn<Row>[] = [
          {
            key: 'month',
            header: t('relatorios-mes'),
            numeric: true,
            cell: (r) => formatMonthShort(r.month),
          },
          ...model.input.bars.map((b, j): TechColumn<Row> => ({
            key: b.key,
            header: b.name,
            numeric: true,
            cell: (r) => <Money cents={r.values[j] ?? 0} hideCurrency />,
          })),
          {
            key: 'total',
            header: t('relatorios-total'),
            numeric: true,
            cell: (r) => <Money cents={r.total} />,
          },
          {
            key: 'avg',
            header: t('relatorios-media-3m'),
            numeric: true,
            cell: (r) => <Money cents={r.avg} />,
          },
          {
            key: 'income',
            header: t('relatorios-receitas'),
            numeric: true,
            cell: (r) => <Money cents={r.income} />,
          },
        ];
        const lastAvg = data.moving_avg_3[n - 1] ?? 0;
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-tendencia-pergunta')}</Question>
            <Stats>
              <Stat label={t('relatorios-media-movel-ultimo')}>
                <Money cents={lastAvg} />
              </Stat>
              <Stat
                label={t('relatorios-variacao-media-movel')}
                hint={t('relatorios-contra', {
                  mes: formatMonthShort(data.months[n > 12 ? n - 13 : 0] ?? win.endMonth),
                })}
              >
                {model.trend == null ? (
                  '—'
                ) : (
                  <span className={model.trend > 0 ? 'text-negative' : 'text-positive'}>
                    {model.trend > 0 ? '▲ ' : '▼ '}
                    <Private>{formatPercent(model.trend, { signed: true })}</Private>
                  </span>
                )}
              </Stat>
              <Stat label={t('relatorios-media-meses', { n })}>
                <Money cents={avgAll} />
              </Stat>
              <Stat label={t('relatorios-receitas-periodo')}>
                <Money cents={data.income.reduce((s, v) => s + v, 0)} />
              </Stat>
            </Stats>
            <FilterNote memberId={memberId} accountId={accountId} />
            <ChartFrame
              index="04"
              title={t('relatorios-tendencia-titulo', { n, mes: formatMonthShort(win.endMonth) })}
              source={t('relatorios-tendencia-fonte', { base: basisLabel(data.basis) })}
              chart={
                <StackedBars
                  input={model.input}
                  height={360}
                  ariaLabel={t('relatorios-tendencia-aria', {
                    n,
                    mes: formatMonthLong(win.endMonth),
                  })}
                  onSegmentClick={(key, i) => {
                    const month = data.months[i];
                    if (!month) return;
                    if (key === 'income') drill(monthDrill(month, data.basis, [], 'income'));
                    else if (key === 'avg') drill(monthDrill(month, data.basis));
                    else
                      drill(
                        monthDrill(
                          month,
                          data.basis,
                          key === OTHERS_KEY ? (model.drill.get(key) ?? []) : [key],
                        ),
                      );
                  }}
                />
              }
              table={
                <TechTable
                  label={t('relatorios-tendencia-tabela')}
                  prefsId="Despesas por mês e categoria"
                  columns={columns}
                  rows={rows}
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

// ── 05 · Orçamento ───────────────────────────────────────────────

export function BudgetFigure({ period, basis }: FigureProps) {
  const month = monthOf(period.to);
  const query = useBudget({ month, basis: basis ?? undefined });
  const categories = useCategories({ kind: 'expense', include_archived: true });
  const colors = useCategoryColors();
  const drill = useDrill();
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.lines.length === 0}
      empty={
        <EmptyState
          kicker={t('graficos-sem-orcamento')}
          title={t('relatorios-orcamento-vazio', { mes: formatMonthLong(month) })}
          actions={
            <Link
              to={`/orcamento?mes=${month}`}
              className="label-caps inline-flex items-center gap-1.5 text-accent"
            >
              {t('relatorios-montar-orcamento')} <Icon name="arrowRight" size={12} />
            </Link>
          }
        />
      }
    >
      {(data) => {
        type Row = {
          id: string;
          name: string;
          planned: number;
          actual: number;
          committed: number;
          status: BulletStatus;
          color: string;
          group: SpendGroup | null;
        };
        const groups = new Map((categories.data ?? []).map((c) => [c.id, c.spend_group]));
        const rows: Row[] = data.lines
          .filter((l) => !l.parent_id && l.kind === 'expense')
          .map((l) => ({
            id: l.category_id,
            name: l.category_name,
            planned: l.planned,
            actual: l.actual,
            committed: l.committed,
            status: bulletStatus({ ...l, elapsed: data.month_elapsed }),
            color: colors.get(l.category_id) ?? 'var(--line-strong)',
            group: groups.get(l.category_id) ?? null,
          }))
          .sort(
            (a, b) =>
              Number(b.status === 'over') - Number(a.status === 'over') || b.actual - a.actual,
          );
        const over = rows.filter((r) => r.status === 'over');
        const tot = data.totals;
        const columns: TechColumn<Row>[] = [
          { key: 'name', header: t('relatorios-categoria'), sortValue: (r) => r.name },
          {
            key: 'group',
            header: t('relatorios-grupos-grupo'),
            sortValue: (r) => (r.group ? groupName(r.group) : null),
            cell: (r) => (r.group ? groupName(r.group) : '—'),
          },
          {
            key: 'planned',
            header: t('relatorios-previsto'),
            numeric: true,
            sortValue: (r) => r.planned,
            cell: (r) => (r.planned ? <Money cents={r.planned} /> : '—'),
          },
          {
            key: 'actual',
            header: t('relatorios-realizado'),
            numeric: true,
            sortValue: (r) => r.actual,
            cell: (r) => <Money cents={r.actual} />,
          },
          {
            key: 'committed',
            header: t('relatorios-comprometido'),
            numeric: true,
            sortValue: (r) => r.committed,
            cell: (r) => <Money cents={r.committed} />,
          },
          {
            key: 'diff',
            header: t('relatorios-restante'),
            numeric: true,
            sortValue: (r) => r.planned - r.actual,
            cell: (r) => (r.planned ? <Money cents={r.planned - r.actual} tone="auto" /> : '—'),
          },
          {
            key: 'status',
            header: t('relatorios-situacao'),
            cell: (r) => <BulletStatusLabel status={r.status} />,
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-orcamento-pergunta')}</Question>
            <Stats>
              <Stat label={t('relatorios-previsto')}>
                <Money cents={tot.planned_expense} />
              </Stat>
              <Stat label={t('relatorios-realizado')}>
                <Money cents={tot.actual_expense} />
              </Stat>
              <Stat label={t('relatorios-estouradas')}>
                <Private>{over.length}</Private>
              </Stat>
              <Stat label={t('relatorios-mes-decorrido')}>
                {formatPercent(data.month_elapsed, { decimals: 0 })}
              </Stat>
            </Stats>
            <ChartFrame
              index="05"
              title={t('relatorios-orcamento-titulo', { mes: formatMonthLong(month) })}
              source={t('relatorios-orcamento-fonte', { base: basisLabel(data.basis) })}
              actions={
                <Link
                  to={`/orcamento?mes=${month}`}
                  className="label-caps inline-flex items-center gap-1 hover:text-accent"
                >
                  {t('comum-editar')} <Icon name="arrowRight" size={11} />
                </Link>
              }
              chart={
                <ul
                  className="flex flex-col"
                  aria-label={t('relatorios-orcamento-aria', { mes: formatMonthLong(month) })}
                >
                  {rows.map((r) => (
                    <li
                      key={r.id}
                      className="grid grid-cols-[minmax(120px,200px)_1fr_220px] items-center gap-4 border-b border-line/60 py-2"
                    >
                      <button
                        type="button"
                        onClick={() =>
                          drill(
                            categoryDrill({ period: monthBounds(month), basis: data.basis }, [
                              r.id,
                            ]),
                          )
                        }
                        className="flex min-w-0 items-center gap-2 rounded-control text-left text-[13px] text-fg hover:text-accent focus-visible:outline-2 focus-visible:outline-accent"
                      >
                        <span
                          aria-hidden
                          className="size-2 shrink-0 rounded-[2px]"
                          style={{ background: cssColor(r.color) }}
                        />
                        <span className="truncate">{r.name}</span>
                      </button>
                      <BulletBar
                        label={r.name}
                        planned={r.planned}
                        actual={r.actual}
                        committed={r.committed}
                        elapsed={data.month_elapsed}
                        hint={t('relatorios-clique-nome')}
                      />
                      <span className="num flex items-baseline justify-end gap-2 text-[12.5px]">
                        <Money cents={r.actual} className="text-fg" />
                        <span className="text-dim">{'/'}</span>
                        {r.planned ? (
                          <Money cents={r.planned} className="text-dim" />
                        ) : (
                          <span className="text-dim">{t('relatorios-sem-previsto')}</span>
                        )}
                      </span>
                    </li>
                  ))}
                </ul>
              }
              table={
                <TechTable
                  label={t('relatorios-orcamento-tabela')}
                  prefsId="Previsto × realizado"
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

// ── 06 · Comparação ──────────────────────────────────────────────

type CompareMode = 'anterior' | 'ano';

export function ComparisonFigure({ period, basis, memberId, accountId, today }: FigureProps) {
  const [mode, setMode] = useState<CompareMode>('anterior');
  const [form, setForm] = useState<'diverging' | 'slope'>('diverging');
  const a = useMemo(
    () => (mode === 'anterior' ? previousPeriod(period) : yearBeforePeriod(period)),
    [mode, period],
  );
  const query = useCompare({ a, b: period, basis: basis ?? undefined, kind: 'expense' });
  const colors = useCategoryColors();
  const drill = useDrill();
  const base = useBaseCurrency();
  const currency = query.data?.currency ?? base;
  const labels = useMemo(
    () => ({ a: periodLabel(a), b: periodLabel(period), currency }),
    [a, period, currency],
  );
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.rows.length === 0}
      loadingLabel={t('relatorios-carregando-comparacao')}
    >
      {(data) => {
        const rows = comparisonRows(data, colors);
        const chartRows = form === 'slope' ? slopeRows(rows, colors) : rows;
        const delta = data.total_b - data.total_a;
        const columns: TechColumn<ComparisonInput>[] = [
          { key: 'label', header: t('relatorios-categoria'), sortValue: (r) => r.label },
          {
            key: 'a',
            header: labels.a,
            numeric: true,
            sortValue: (r) => r.a,
            cell: (r) => <Money cents={r.a} />,
          },
          {
            key: 'b',
            header: labels.b,
            numeric: true,
            sortValue: (r) => r.b,
            cell: (r) => <Money cents={r.b} />,
          },
          {
            key: 'delta',
            header: t('graficos-variacao'),
            numeric: true,
            sortValue: (r) => r.b - r.a,
            cell: (r) => (
              <Money
                cents={r.b - r.a}
                tone={r.b > r.a ? 'negative' : r.b < r.a ? 'positive' : 'neutral'}
                trend
                signed
              />
            ),
          },
          {
            key: 'pct',
            header: '%',
            numeric: true,
            sortValue: (r) => (r.a > 0 ? (r.b - r.a) / r.a : null),
            cell: (r) =>
              r.a > 0 ? (
                <Private>{formatPercent((r.b - r.a) / r.a, { signed: true })}</Private>
              ) : (
                '—'
              ),
          },
        ];
        const open = (key: string) => {
          const ids =
            key === OTHERS_KEY ? rows.filter((r) => !colors.get(r.key)).map((r) => r.key) : [key];
          drill(
            categoryDrill(
              { period, basis: data.basis },
              ids.map((id) => (id === NO_CATEGORY_KEY ? null : id)),
            ),
          );
        };
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-comparacao-pergunta', { a: labels.a, b: labels.b })}</Question>
            <Stats>
              <Stat label={t('relatorios-comparacao-a', { periodo: labels.a })}>
                <Money cents={data.total_a} />
              </Stat>
              <Stat label={t('relatorios-comparacao-b', { periodo: labels.b })}>
                <Explainable
                  target={{
                    title: t('relatorios-despesas-de', { periodo: labels.b }),
                    args: { metric: 'spending', params: { ...period, basis: data.basis } },
                  }}
                >
                  <Money cents={data.total_b} />
                </Explainable>
              </Stat>
              <Stat label={t('graficos-variacao')}>
                <Money
                  cents={delta}
                  tone={delta > 0 ? 'negative' : delta < 0 ? 'positive' : 'neutral'}
                  trend
                  signed
                />
              </Stat>
              <Stat label={t('relatorios-variacao-pct')}>
                {data.total_a > 0 ? (
                  <Private>{formatPercent(delta / data.total_a, { signed: true })}</Private>
                ) : (
                  '—'
                )}
              </Stat>
            </Stats>
            <div className="flex flex-wrap items-center gap-3">
              <Segmented
                label={t('relatorios-comparar-com')}
                value={mode}
                onChange={setMode}
                options={[
                  { value: 'anterior', label: t('relatorios-periodo-anterior') },
                  { value: 'ano', label: t('relatorios-um-ano-antes') },
                ]}
              />
              <FilterNote memberId={memberId} accountId={accountId} />
            </div>
            {period.from <= today && today < period.to ? (
              <p role="note" className="text-[12px] text-warning">
                {t('relatorios-comparacao-em-andamento')}
              </p>
            ) : null}
            <ChartFrame
              index="06"
              title={t('relatorios-comparacao-titulo', { a: labels.a, b: labels.b })}
              source={t('relatorios-comparacao-fonte', { base: basisLabel(data.basis) })}
              actions={
                <Segmented
                  label={t('relatorios-forma')}
                  value={form}
                  onChange={setForm}
                  options={[
                    { value: 'diverging', label: t('relatorios-forma-barras') },
                    { value: 'slope', label: t('relatorios-forma-slope') },
                  ]}
                />
              }
              chart={
                <ComparisonChart
                  rows={chartRows}
                  labels={labels}
                  form={form}
                  ariaLabel={t('relatorios-comparacao-aria', { a: labels.a, b: labels.b })}
                  onRowClick={open}
                />
              }
              table={
                <TechTable
                  label={t('relatorios-comparacao-tabela')}
                  prefsId="Comparação por categoria"
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
