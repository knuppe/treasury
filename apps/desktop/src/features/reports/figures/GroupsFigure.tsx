/**
 * Figura 12 · Grupos: para onde vai a renda. Barras empilhadas por grupo de gasto (essenciais,
 * dívidas, futuro, não essencial) mês a mês, em parte da renda ou valor, ou a evolução da parte
 * da renda de cada grupo contra a meta; e as maiores categorias de cada grupo no período.
 */
import { useMemo, useState } from 'react';
import { Link } from 'react-router';

import type { SpendGroup } from '@/api/generated/SpendGroup';
import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import { useBudget } from '@/api/hooks/budget';
import { useCategories, useSpendGroups } from '@/api/hooks/reports';
import { ChartFrame } from '@/charts/ChartFrame';
import { cssColor } from '@/charts/colorMath';
import { SpendGroupsChart } from '@/charts/SpendGroupsChart';
import type { SpendGroupsInput } from '@/charts/spendGroupsOption';
import { Icon, Money, Private, TechTable, type TechColumn } from '@/design/components';
import { StatusText } from '@/features/budget/SpendGroupsCard';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatMonthLong, formatMonthShort, formatPercent } from '@/lib/format';
import { monthBounds, monthOf, shiftMonth } from '@/lib/months';

import { QueryView, Segmented } from '../controls';
import { Explainable } from '../explain/ExplainButton';
import { categoryDrill } from '../model/drill';
import { groupName, groupRows, SPEND_GROUPS } from '../model/groups';
import { periodLabel, trendWindow } from '../model/period';
import type { EntityColors } from '../model/colors';
import { useGroupColors } from '../useEntityColors';
import {
  basisLabel,
  FilterNote,
  Question,
  Stat,
  Stats,
  useDrill,
  type FigureProps,
} from './common';

type Unit = SpendGroupsInput['unit'];
type Form = SpendGroupsInput['form'];

const share = (v: number | null) => (v == null ? '—' : formatPercent(v));

interface MonthRow {
  month: string;
  income: number;
  spending: number;
  groups: Record<SpendGroup, { amount: number; share: number | null }>;
}

interface TopRow {
  id: string;
  group: SpendGroup;
  categoryId: string | null;
  label: string;
  amount: number;
  ofGroup: number | null;
  ofIncome: number | null;
}

function monthRows(data: SpendGroupsReport): MonthRow[] {
  return data.months.map((m) => ({
    month: m.month,
    income: m.income,
    spending: m.spending,
    groups: Object.fromEntries(
      SPEND_GROUPS.map((g) => {
        const x = m.groups.find((y) => y.group === g);
        return [g, { amount: x?.amount ?? 0, share: x?.share_of_income ?? null }];
      }),
    ) as MonthRow['groups'],
  }));
}

function topRows(data: SpendGroupsReport): TopRow[] {
  return SPEND_GROUPS.flatMap((g) => {
    const x = data.total.groups.find((y) => y.group === g);
    if (!x) return [];
    return x.top.map((c, i) => ({
      id: `${g}:${c.category_id ?? `${i}:${c.label}`}`,
      group: g,
      categoryId: c.category_id,
      label: c.label,
      amount: c.amount,
      ofGroup: x.amount > 0 ? c.amount / x.amount : null,
      ofIncome: data.total.income > 0 ? c.amount / data.total.income : null,
    }));
  });
}

export function GroupsFigure({ period, basis, memberId, accountId, today }: FigureProps) {
  const [unit, setUnit] = useState<Unit>('share');
  const [form, setForm] = useState<Form>('bars');
  const win = trendWindow(period);
  const fromMonth = shiftMonth(win.endMonth, -(win.months - 1));
  const member = memberId ?? undefined;
  const chartQuery = useSpendGroups({
    from: fromMonth,
    to: win.endMonth,
    basis: basis ?? undefined,
    member_id: member,
  });
  const periodQuery = useSpendGroups({
    from: monthOf(period.from),
    to: monthOf(period.to),
    basis: basis ?? undefined,
    member_id: member,
  });
  // A base padrão do cofre (o núcleo resolve `budget_basis`), para a linha de fonte.
  const defaults = useBudget({ month: monthOf(today) });
  const shownBasis = basis ?? defaults.data?.basis ?? 'accrual';
  const categories = useCategories({ kind: 'expense', include_archived: true });
  const colors = useGroupColors();
  const drill = useDrill();
  const colorOf = (g: SpendGroup) => colors.get(g) ?? 'var(--line-strong)';
  const idsOf = (g: SpendGroup) =>
    (categories.data ?? []).filter((c) => c.spend_group === g).map((c) => c.id);
  const scopeOf = (from: string, to: string) => ({
    period: { from, to },
    basis,
    memberIds: memberId ? [memberId] : null,
  });

  const openGroup = (g: SpendGroup, from: string, to: string) => {
    const ids = idsOf(g);
    // Futuro sem categorias (só aportes): os lançamentos de investimento do período.
    if (!ids.length && g === 'future') drill(categoryDrill(scopeOf(from, to), [], 'investment'));
    else if (ids.length) drill(categoryDrill(scopeOf(from, to), ids));
  };

  return (
    <QueryView
      query={periodQuery}
      isEmpty={(d) => d.total.income <= 0 && d.total.spending <= 0}
      loadingLabel={t('relatorios-grupos-carregando')}
    >
      {(data) => {
        const rows = groupRows(data.total, data.targets);
        const tops = topRows(data);
        const label = periodLabel(period);
        const topColumns: TechColumn<TopRow>[] = [
          {
            key: 'group',
            header: t('relatorios-grupos-grupo'),
            sortValue: (r) => SPEND_GROUPS.indexOf(r.group),
            cell: (r) => (
              <span className="flex items-center gap-2">
                <span
                  aria-hidden
                  className="size-2 shrink-0 rounded-[2px]"
                  style={{ background: cssColor(colorOf(r.group)) }}
                />
                <span className="text-fg">{groupName(r.group)}</span>
              </span>
            ),
          },
          {
            key: 'label',
            header: t('relatorios-categoria'),
            sortValue: (r) => r.label,
            cell: (r) =>
              r.categoryId ? (
                <button
                  type="button"
                  className="text-left text-fg hover:text-accent focus-visible:outline-2 focus-visible:outline-accent"
                  onClick={() =>
                    drill(categoryDrill(scopeOf(period.from, period.to), [r.categoryId]))
                  }
                >
                  {r.label}
                </button>
              ) : (
                <span className="text-fg">{r.label}</span>
              ),
          },
          {
            key: 'amount',
            header: t('graficos-gasto'),
            numeric: true,
            sortValue: (r) => r.amount,
            cell: (r) => <Money cents={r.amount} />,
          },
          {
            key: 'ofGroup',
            header: t('relatorios-grupos-pct-grupo'),
            numeric: true,
            sortValue: (r) => r.ofGroup,
            cell: (r) => <Private>{share(r.ofGroup)}</Private>,
          },
          {
            key: 'ofIncome',
            header: t('relatorios-grupos-pct-renda'),
            numeric: true,
            sortValue: (r) => r.ofIncome,
            cell: (r) => <Private>{share(r.ofIncome)}</Private>,
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-grupos-pergunta')}</Question>
            <Stats>
              {rows.map((r) => (
                <Stat
                  key={r.group}
                  label={
                    <span className="inline-flex items-center gap-1.5">
                      <span
                        aria-hidden
                        className="size-2 shrink-0 rounded-[2px]"
                        style={{ background: cssColor(colorOf(r.group)) }}
                      />
                      {groupName(r.group)}
                    </span>
                  }
                  hint={
                    <span className="flex flex-col gap-0.5">
                      <span>
                        {r.target == null
                          ? tRich('relatorios-grupos-stat-dica-sem-meta', {
                              valor: <Money cents={r.amount} />,
                            })
                          : tRich('relatorios-grupos-stat-dica', {
                              valor: <Money cents={r.amount} />,
                              meta: formatPercent(r.target, { decimals: 0 }),
                            })}
                      </span>
                      <StatusText status={r.status} />
                    </span>
                  }
                >
                  <Private>{share(r.share)}</Private>
                </Stat>
              ))}
            </Stats>
            <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-[12.5px] text-dim">
              <span className="inline-flex items-baseline gap-1.5">
                {t('relatorios-grupos-renda', { periodo: label })}
                <Explainable
                  target={{
                    title: t('relatorios-grupos-renda', { periodo: label }),
                    args: {
                      metric: 'income',
                      params: { ...period, basis: shownBasis, member_id: member },
                    },
                  }}
                >
                  <Money cents={data.total.income} className="text-fg" />
                </Explainable>
              </span>
              <Link
                to="/orcamento"
                className="label-caps inline-flex items-center gap-1 hover:text-accent"
              >
                {t('relatorios-grupos-metas-link')} <Icon name="arrowRight" size={11} />
              </Link>
              <Link
                to="/ajustes?secao=categorias"
                className="label-caps inline-flex items-center gap-1 hover:text-accent"
              >
                {t('relatorios-grupos-categorias-link')} <Icon name="arrowRight" size={11} />
              </Link>
            </div>
            {data.total.income <= 0 ? (
              <p role="note" className="text-[12px] text-warning">
                {t('relatorios-grupos-sem-renda')}
              </p>
            ) : null}
            <div className="flex flex-wrap items-center gap-3">
              <Segmented
                label={t('relatorios-grupos-medida')}
                value={unit}
                onChange={setUnit}
                options={[
                  { value: 'share', label: t('relatorios-grupos-medida-pct') },
                  { value: 'money', label: t('relatorios-grupos-medida-valor') },
                ]}
              />
              <FilterNote memberId={null} accountId={accountId} />
            </div>
            <QueryView query={chartQuery} loadingLabel={t('relatorios-grupos-carregando')}>
              {(win) => (
                <GroupsChartFrame
                  data={win}
                  unit={unit}
                  form={form}
                  onForm={setForm}
                  basisText={basisLabel(shownBasis)}
                  colors={colors}
                  onSegment={(g, month) => {
                    const b = monthBounds(month);
                    openGroup(g, b.from, b.to);
                  }}
                />
              )}
            </QueryView>
            <p className="text-[12px] text-dim">{t('relatorios-grupos-nota-futuro')}</p>
            <section className="flex flex-col gap-2">
              <h3 className="label-caps text-fg">
                {t('relatorios-grupos-maiores-titulo', { periodo: label })}
              </h3>
              <TechTable
                label={t('relatorios-grupos-maiores-tabela')}
                prefsId="Maiores categorias por grupo"
                columns={topColumns}
                rows={tops}
                getRowId={(r) => r.id}
                dense
              />
            </section>
          </div>
        );
      }}
    </QueryView>
  );
}

function GroupsChartFrame({
  data,
  unit,
  form,
  onForm,
  basisText,
  colors,
  onSegment,
}: {
  data: SpendGroupsReport;
  unit: Unit;
  form: Form;
  onForm: (f: Form) => void;
  basisText: string;
  colors: EntityColors;
  onSegment: (g: SpendGroup, month: string) => void;
}) {
  const rows = useMemo(() => monthRows(data), [data]);
  const months = rows.map((r) => r.month);
  const n = months.length;
  const end = months[n - 1] ?? '';
  const input = useMemo<SpendGroupsInput>(
    () => ({
      months: rows.map((r) => r.month),
      unit,
      form,
      currency: data.currency,
      income: rows.map((r) => r.income),
      series: SPEND_GROUPS.map((g) => ({
        key: g,
        name: groupName(g),
        color: colors.get(g) ?? '@lineStrong',
        values: rows.map((r) => (unit === 'share' ? r.groups[g].share : r.groups[g].amount)),
        target: data.targets[g],
      })),
    }),
    [rows, unit, form, data, colors],
  );
  const columns: TechColumn<MonthRow>[] = [
    {
      key: 'month',
      header: t('relatorios-mes'),
      numeric: true,
      sortValue: (r) => r.month,
      cell: (r) => formatMonthShort(r.month),
    },
    {
      key: 'income',
      header: t('relatorios-receitas'),
      numeric: true,
      sortValue: (r) => r.income,
      cell: (r) => <Money cents={r.income} />,
    },
    ...SPEND_GROUPS.map((g): TechColumn<MonthRow> => ({
      key: g,
      header: groupName(g),
      numeric: true,
      sortValue: (r) => r.groups[g].amount,
      cell: (r) => (
        <span className="inline-flex items-baseline gap-2">
          <Money cents={r.groups[g].amount} hideCurrency />
          <span className="text-dim">
            <Private>{share(r.groups[g].share)}</Private>
          </span>
        </span>
      ),
    })),
    {
      key: 'spending',
      header: t('graficos-grupos-total'),
      numeric: true,
      sortValue: (r) => r.spending,
      cell: (r) => <Money cents={r.spending} />,
    },
  ];
  const aria =
    form === 'bars'
      ? t('relatorios-grupos-aria-barras', { n, mes: end ? formatMonthLong(end) : '' })
      : t('relatorios-grupos-aria-linhas', { n, mes: end ? formatMonthLong(end) : '' });
  return (
    <ChartFrame
      index="12"
      title={t('relatorios-grupos-titulo', { n, mes: end ? formatMonthShort(end) : '' })}
      source={t('relatorios-grupos-fonte', { base: basisText })}
      actions={
        <Segmented
          label={t('relatorios-forma')}
          value={form}
          onChange={onForm}
          options={[
            { value: 'bars', label: t('relatorios-grupos-forma-barras') },
            { value: 'lines', label: t('relatorios-grupos-forma-linhas') },
          ]}
        />
      }
      chart={
        <SpendGroupsChart
          input={input}
          height={360}
          ariaLabel={aria}
          onSegmentClick={(key, i) => {
            const month = months[i];
            if (month && SPEND_GROUPS.includes(key as SpendGroup))
              onSegment(key as SpendGroup, month);
          }}
        />
      }
      table={
        <TechTable
          label={t('relatorios-grupos-tabela')}
          prefsId="Grupos de gasto por mês"
          columns={columns}
          rows={rows}
          getRowId={(r) => r.month}
          dense
        />
      }
    />
  );
}
