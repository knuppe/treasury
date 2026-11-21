/** Figuras 01 · Fluxo (Sankey com o período anterior), 02 · Composição (treemap/sunburst) e 03 · Ritmo (calendário). */
import { useMemo, useState, type ReactNode } from 'react';
import { Link, useSearchParams } from 'react-router';

import { useCalendar, useCategories, useSankey, useSpending } from '@/api/hooks/reports';
import { CalendarHeatmap } from '@/charts/CalendarHeatmap';
import { ChartFrame } from '@/charts/ChartFrame';
import { CompositionChart } from '@/charts/CompositionChart';
import { SankeyChart } from '@/charts/SankeyChart';
import { topDays, weekdayAverages, weekdays } from '@/charts/calendar';
import type { CompositionForm } from '@/charts/composition';
import { cssColor } from '@/charts/colorMath';
import type { SpendGroup } from '@/api/generated/SpendGroup';
import { EmptyState, Icon, Money, Private, TechTable, type TechColumn } from '@/design/components';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatNumber, formatPercent } from '@/lib/format';
import { daysBetween, weekday } from '@/lib/months';

import { capitalize, QueryView, Segmented } from '../controls';
import { Explainable } from '../explain/ExplainButton';
import { OTHERS_KEY, othersLabel } from '../model/colors';
import { categoryDrill, dayDrill } from '../model/drill';
import { comparisonRows, flowModel, type FlowCompareRow, type FlowGroup } from '../model/flow';
import { calendarWindow, periodLabel, singleMonth } from '../model/period';
import { groupName } from '../model/groups';
import { compositionModel } from '../model/series';
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
import { FlowItemsDialog, type FlowItemsTarget, type FlowItemsView } from './FlowItemsDialog';

// ── 01 · Fluxo ───────────────────────────────────────────────────

interface LinkRow {
  id: string;
  /** Nó cujo analítico a linha abre (o destino; o de origem quando vai para o orçamento). */
  node: string;
  from: string;
  to: string;
  value: number;
  share: number;
}

type FlowDepth = 1 | 2;

const depthOptions = () =>
  [
    {
      value: '1',
      label: t('relatorios-detalhe-categorias'),
      hint: t('relatorios-detalhe-categorias-dica'),
    },
    {
      value: '2',
      label: t('relatorios-detalhe-subcategorias'),
      hint: t('relatorios-detalhe-subcategorias-dica'),
    },
  ] as const;

const groupLabel = (g: FlowGroup) => t(`relatorios-grupo-${g}`);

/** Tom da variação: verde quando o movimento é bom (mais receita, menos despesa). */
function deltaTone(delta: number, goodWhenUp: boolean): 'positive' | 'negative' | 'neutral' {
  if (delta === 0) return 'neutral';
  return delta > 0 === goodWhenUp ? 'positive' : 'negative';
}

/** "vs anterior: ▲ +R$ 120,00 (+8,0%)" abaixo de uma medida (o período anterior na dica). */
function VsPrevious({
  current,
  previous,
  label,
  goodWhenUp,
}: {
  current: number;
  previous: number;
  label: string;
  goodWhenUp: boolean;
}) {
  const delta = current - previous;
  return (
    <span className="flex flex-wrap items-baseline gap-x-1">
      <span title={t('relatorios-periodo-anterior-dica', { periodo: label })}>
        {t('relatorios-vs-anterior')}
      </span>
      <Money cents={delta} tone={deltaTone(delta, goodWhenUp)} trend signed />
      {previous > 0 ? (
        <Private>({formatPercent(delta / previous, { signed: true })})</Private>
      ) : null}
    </span>
  );
}

/** Valor de uma célula do comparativo: o clique abre o analítico (o que compõe o número). */
function CellLink({
  what,
  onOpen,
  children,
}: {
  what: string;
  onOpen: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-haspopup="dialog"
      title={t('relatorios-ver-compoe-dica')}
      onClick={onOpen}
      onDoubleClick={(e) => e.stopPropagation()}
      className="rounded-control decoration-dotted underline-offset-4 hover:underline focus-visible:outline-2 focus-visible:outline-accent"
    >
      {children}
      <span className="sr-only"> {t('relatorios-ver-compoe-sr', { item: what })}</span>
    </button>
  );
}

/** Diferença de taxas em pontos percentuais: `+3,2 p.p.`. */
function points(diff: number): string {
  const v = Math.round(diff * 1000) / 10;
  const body = formatNumber(Math.abs(v), 1);
  return t('relatorios-pontos', { valor: `${v > 0 ? '+' : v < 0 ? '−' : ''}${body}` });
}

/** Linha de fonte com as moedas sem câmbio no fim, quando houver. */
function withMissingFx(source: string, missing: readonly string[]): string {
  return missing.length
    ? t('relatorios-fonte-sem-cambio', { fonte: source, moedas: missing.join(', ') })
    : source;
}

export function FlowFigure({ period, basis, memberId, accountId, today }: FigureProps) {
  const [depth, setDepth] = useState<FlowDepth>(2);
  const [items, setItems] = useState<FlowItemsTarget | null>(null);
  const query = useSankey({ ...period, basis: basis ?? undefined, depth });
  const colors = useCategoryColors();
  const month = singleMonth(period);
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.links.length === 0 && d.comparison.every((r) => r.previous === 0)}
      loadingLabel={t('relatorios-carregando-fluxo')}
    >
      {(data) => {
        const model = flowModel(data, colors);
        const prev = data.previous;
        const prevLabel = prev ? periodLabel(prev) : null;
        const input = prevLabel ? { ...model.input, previousLabel: prevLabel } : model.input;
        const names = new Map(input.nodes.map((n) => [n.id, n.name]));
        const nodeColor = new Map(input.nodes.map((n) => [n.id, n.color]));
        const tot = data.totals;
        const rate = tot.savings_rate;
        const rows: LinkRow[] = input.links.map((l, i) => ({
          id: String(i),
          node: l.target === 'hub' ? l.source : l.target,
          from: names.get(l.source) ?? l.source,
          to: names.get(l.target) ?? l.target,
          value: l.value,
          share: input.total ? l.value / input.total : 0,
        }));
        // Analítico de um nó: os lançamentos que compõem o valor, ao centavo; "Outras" soma
        // as categorias reunidas.
        const open = (nodeId: string, view: FlowItemsView = 'current') => {
          if (nodeId === 'hub') return;
          if (nodeId === `expense:${OTHERS_KEY}`)
            setItems({
              nodeIds: model.folded.map((c) => `expense:${c}`),
              title: othersLabel(),
              view,
            });
          else setItems({ nodeIds: [nodeId], view });
        };
        const scope = { ...period, basis: data.basis };
        const columns: TechColumn<LinkRow>[] = [
          { key: 'from', header: t('relatorios-de'), sortValue: (r) => r.from },
          { key: 'to', header: t('relatorios-para'), sortValue: (r) => r.to },
          {
            key: 'value',
            header: t('graficos-valor'),
            numeric: true,
            sortValue: (r) => r.value,
            cell: (r) => <Money cents={r.value} />,
          },
          {
            key: 'share',
            header: data.total_income > 0 ? t('relatorios-pct-receita') : t('relatorios-pct-total'),
            numeric: true,
            sortValue: (r) => r.share,
            cell: (r) => <Private>{formatPercent(r.share)}</Private>,
          },
        ];
        const compareRows = comparisonRows(data);
        // Cor do traço: a do nó no diagrama; o que está em "Outras" ou só no anterior, pela
        // categoria de primeiro nível (ou cinza).
        const swatch = (r: FlowCompareRow): string => {
          const fromNode = nodeColor.get(r.id);
          if (fromNode) return cssColor(fromNode);
          const parent = r.parentId ? nodeColor.get(r.parentId) : undefined;
          if (parent) return cssColor(parent);
          const root = r.parentId
            ? data.comparison.find((x) => x.id === r.parentId)?.category_id
            : r.categoryId;
          return cssColor(colors.get(root) ?? '@gray');
        };
        const cellLabel = (r: FlowCompareRow, when: string) =>
          t('relatorios-item-em', { nome: r.name, periodo: when });
        const compareColumns: TechColumn<FlowCompareRow>[] = [
          {
            key: 'name',
            header: t('relatorios-vista-fluxo'),
            cell: (r) => (
              <span className="flex items-center gap-2" style={{ paddingLeft: r.level * 18 }}>
                <span
                  aria-hidden
                  className="size-2 shrink-0 rounded-[2px]"
                  style={{ background: swatch(r), opacity: r.level ? 0.6 : 1 }}
                />
                <span className={r.level ? 'text-dim' : 'text-fg'}>{r.name}</span>
              </span>
            ),
          },
          {
            key: 'group',
            header: t('relatorios-grupo'),
            cell: (r) => <span className="text-dim">{groupLabel(r.group)}</span>,
          },
          {
            key: 'previous',
            header: prevLabel ? capitalize(prevLabel) : t('relatorios-anterior'),
            numeric: true,
            cell: (r) => (
              <CellLink
                what={cellLabel(r, prevLabel ?? t('relatorios-o-periodo-anterior'))}
                onOpen={() => open(r.id, 'previous')}
              >
                <Money cents={r.previous} />
              </CellLink>
            ),
          },
          {
            key: 'current',
            header: capitalize(periodLabel(period)),
            numeric: true,
            cell: (r) => (
              <CellLink what={cellLabel(r, periodLabel(period))} onOpen={() => open(r.id)}>
                <Money cents={r.current} />
              </CellLink>
            ),
          },
          {
            key: 'delta',
            header: 'Δ',
            numeric: true,
            cell: (r) => (
              <CellLink
                what={t('relatorios-diferenca-de', { nome: r.name })}
                onOpen={() => open(r.id, 'delta')}
              >
                <Money cents={r.delta} tone={deltaTone(r.delta, r.goodWhenUp)} trend signed />
              </CellLink>
            ),
          },
          {
            key: 'pct',
            header: 'Δ %',
            numeric: true,
            cell: (r) =>
              r.deltaPct === null ? (
                <span className="text-dim">{r.current > 0 ? t('relatorios-novo') : '—'}</span>
              ) : (
                <Private>{formatPercent(r.deltaPct, { signed: true })}</Private>
              ),
          },
          {
            key: 'share',
            header: t('relatorios-pct-receita'),
            numeric: true,
            cell: (r) => (r.share === null ? '—' : <Private>{formatPercent(r.share)}</Private>),
          },
        ];
        const leftoverLabel = tot.leftover >= 0 ? t('relatorios-sobra') : t('relatorios-deficit');
        const inProgress = period.from <= today && today < period.to;
        return (
          <div className="flex flex-col gap-4">
            <Question>
              {prevLabel
                ? t('relatorios-fluxo-pergunta-comparado', { periodo: prevLabel })
                : t('relatorios-fluxo-pergunta')}
            </Question>
            {data.basis === 'accrual' && data.card_bills_paid > 0 ? (
              <p className="max-w-[80ch] text-[12.5px] text-dim" role="note">
                {withSlots(t('relatorios-fluxo-fatura-competencia', { valor: slot(0) }), [
                  <Money cents={data.card_bills_paid} />,
                ])}
              </p>
            ) : null}
            <Stats>
              <Stat
                label={t('relatorios-receita')}
                hint={
                  prev && prevLabel ? (
                    <VsPrevious
                      current={tot.income}
                      previous={prev.income}
                      label={prevLabel}
                      goodWhenUp
                    />
                  ) : undefined
                }
              >
                <Explainable
                  target={{
                    title: t('relatorios-receitas-de', { periodo: periodLabel(period) }),
                    args: { metric: 'income', params: scope },
                  }}
                >
                  <Money cents={tot.income} estimated={data.estimated} />
                </Explainable>
              </Stat>
              <Stat
                label={t('relatorios-despesas')}
                hint={
                  prev && prevLabel ? (
                    <VsPrevious
                      current={tot.expense}
                      previous={prev.expense}
                      label={prevLabel}
                      goodWhenUp={false}
                    />
                  ) : undefined
                }
              >
                <Explainable
                  target={{
                    title: t('relatorios-despesas-de', { periodo: periodLabel(period) }),
                    args: { metric: 'spending', params: scope },
                  }}
                >
                  <Money cents={tot.expense} estimated={data.estimated} />
                </Explainable>
              </Stat>
              <Stat
                label={leftoverLabel}
                hint={
                  prev && prevLabel ? (
                    <VsPrevious
                      current={tot.leftover}
                      previous={prev.leftover}
                      label={prevLabel}
                      goodWhenUp
                    />
                  ) : undefined
                }
              >
                <Money cents={tot.leftover} tone="auto" trend />
              </Stat>
              <Stat
                label={t('relatorios-taxa-poupanca')}
                hint={
                  prev && prevLabel && rate !== null && prev.savings_rate !== null ? (
                    <span>
                      <span title={t('relatorios-periodo-anterior-dica', { periodo: prevLabel })}>
                        {t('relatorios-vs-anterior')}
                      </span>{' '}
                      <Private>
                        <span
                          className={
                            rate > prev.savings_rate
                              ? 'text-positive'
                              : rate < prev.savings_rate
                                ? 'text-negative'
                                : undefined
                          }
                        >
                          {points(rate - prev.savings_rate)}
                        </span>
                      </Private>
                    </span>
                  ) : month ? undefined : (
                    t('relatorios-no-periodo')
                  )
                }
              >
                {rate !== null ? (
                  month ? (
                    <Explainable
                      target={{
                        title: t('relatorios-taxa-poupanca-de', { periodo: periodLabel(period) }),
                        args: { metric: 'savings_rate', params: { month, basis: data.basis } },
                      }}
                    >
                      <Private>{formatPercent(rate)}</Private>
                    </Explainable>
                  ) : (
                    <Private>{formatPercent(rate)}</Private>
                  )
                ) : (
                  '—'
                )}
              </Stat>
            </Stats>
            {tot.investments || tot.redemptions || tot.debt_payments ? (
              <p className="flex flex-wrap gap-x-5 gap-y-1 text-[12.5px] text-dim">
                {tot.investments ? (
                  <span>
                    {withSlots(t('relatorios-investimentos-aportes', { valor: slot(0) }), [
                      <Money key="v" cents={tot.investments} />,
                    ])}
                  </span>
                ) : null}
                {tot.redemptions ? (
                  <span>
                    {withSlots(t('relatorios-resgates', { valor: slot(0) }), [
                      <Money key="v" cents={tot.redemptions} />,
                    ])}
                  </span>
                ) : null}
                {tot.debt_payments ? (
                  <span>
                    {withSlots(t('relatorios-pagamento-dividas', { valor: slot(0) }), [
                      <Money key="v" cents={tot.debt_payments} />,
                    ])}
                  </span>
                ) : null}
              </p>
            ) : null}
            <FilterNote memberId={memberId} accountId={accountId} />
            {inProgress && prevLabel ? (
              <p role="note" className="text-[12px] text-warning">
                {t('relatorios-fluxo-em-andamento', { periodo: prevLabel })}
              </p>
            ) : null}
            <ChartFrame
              index="01"
              title={t('relatorios-fluxo-titulo', { periodo: periodLabel(period) })}
              source={withMissingFx(
                t('relatorios-fluxo-fonte', { base: basisLabel(data.basis) }),
                data.missing_fx,
              )}
              actions={
                <Segmented
                  label={t('relatorios-detalhe')}
                  value={String(depth) as '1' | '2'}
                  onChange={(v) => setDepth(Number(v) as FlowDepth)}
                  options={depthOptions()}
                />
              }
              chart={
                input.links.length ? (
                  <SankeyChart
                    input={input}
                    ariaLabel={t('relatorios-fluxo-aria', {
                      periodo: periodLabel(period),
                      n: input.nodes.filter((n) => n.id.startsWith('expense:')).length,
                      investimentos: tot.investments ? 'sim' : 'nao',
                      sobra: tot.leftover >= 0 ? 'sim' : 'nao',
                    })}
                    onNodeClick={open}
                    onLinkClick={(l) => open(l.target === 'hub' ? l.source : l.target)}
                  />
                ) : (
                  <EmptyState
                    title={t('relatorios-sem-movimento')}
                    kicker={t('componentes-sem-dados')}
                  />
                )
              }
              table={
                <TechTable
                  label={t('relatorios-fluxo-tabela')}
                  prefsId="Fluxo: origem, destino e valor"
                  columns={columns}
                  rows={rows}
                  getRowId={(r) => r.id}
                  onActivate={(r) => open(r.node)}
                  dense
                />
              }
            />
            {prevLabel ? (
              <section
                aria-labelledby="fluxo-comparativo"
                className="flex flex-col gap-2 border-t border-line pt-4"
              >
                <h3 id="fluxo-comparativo" className="label-caps">
                  {t('relatorios-comparativo-titulo', {
                    anterior: prevLabel,
                    atual: periodLabel(period),
                  })}
                </h3>
                <TechTable
                  label={t('relatorios-comparativo-tabela', {
                    anterior: prevLabel,
                    atual: periodLabel(period),
                  })}
                  prefsId="Comparativo do fluxo"
                  columns={compareColumns}
                  rows={compareRows}
                  getRowId={(r) => r.id}
                  onActivate={(r) => open(r.id)}
                  minWidth="760px"
                  dense
                />
                <p className="text-[12px] text-dim">{t('relatorios-comparativo-dica')}</p>
              </section>
            ) : null}
            {items ? (
              <FlowItemsDialog
                key={`${items.nodeIds.join(',')}|${items.view}`}
                initial={items}
                scope={{ period, basis: data.basis, depth }}
                onClose={() => setItems(null)}
              />
            ) : null}
          </div>
        );
      }}
    </QueryView>
  );
}

// ── 02 · Composição ──────────────────────────────────────────────

interface CompRow {
  id: string;
  name: string;
  depth: number;
  value: number;
  share: number;
  color: string;
  categories: (string | null)[];
  /** Grupo de gasto da categoria (a "direto" usa o da principal; "Outras" não tem). */
  group: SpendGroup | null;
}

export function CompositionFigure({ period, basis, memberId, accountId }: FigureProps) {
  const [form, setForm] = useState<CompositionForm>('treemap');
  const query = useSpending({
    ...period,
    basis: basis ?? undefined,
    kind: 'expense',
    group_by: 'category',
    member_ids: memberId ? [memberId] : undefined,
    account_ids: accountId ? [accountId] : undefined,
  });
  const categories = useCategories({ kind: 'expense', include_archived: true });
  const byId = useMemo(
    () => new Map((categories.data ?? []).map((c) => [c.id, c])),
    [categories.data],
  );
  const [params] = useSearchParams();
  const groupsLink = useMemo(() => {
    const p = new URLSearchParams(params);
    p.set('v', 'grupos');
    return `/relatorios?${p.toString()}`;
  }, [params]);
  const colors = useCategoryColors();
  const drill = useDrill();
  const scopeFor = (b: FigureProps['basis']) => ({
    period,
    basis: b,
    memberIds: memberId ? [memberId] : null,
    accountIds: accountId ? [accountId] : null,
  });
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.total <= 0}
      loadingLabel={t('relatorios-carregando-composicao')}
    >
      {(data) => {
        const model = compositionModel(data, categories.data ?? [], colors);
        const groupOf = (id: string) => byId.get(id)?.spend_group ?? null;
        const rows: CompRow[] = model.nodes.flatMap((n) => [
          {
            id: n.id,
            name: n.name,
            depth: 0,
            value: n.value,
            share: model.total ? n.value / model.total : 0,
            color: cssColor(n.color),
            categories: model.drill.get(n.id) ?? [],
            group: groupOf(n.id),
          },
          ...(n.children ?? []).map((c) => ({
            id: `${n.id}/${c.id}`,
            name: c.name,
            depth: 1,
            value: c.value,
            share: model.total ? c.value / model.total : 0,
            color: cssColor(n.color),
            categories: model.drill.get(c.id) ?? [c.id],
            group: groupOf(c.id) ?? groupOf(n.id),
          })),
        ]);
        const columns: TechColumn<CompRow>[] = [
          {
            key: 'name',
            header: t('relatorios-categoria'),
            cell: (r) => (
              <span className="flex items-center gap-2" style={{ paddingLeft: r.depth * 18 }}>
                <span
                  aria-hidden
                  className="size-2 rounded-[2px]"
                  style={{ background: r.color, opacity: r.depth ? 0.6 : 1 }}
                />
                <span className={r.depth ? 'text-dim' : 'text-fg'}>{r.name}</span>
              </span>
            ),
          },
          {
            key: 'value',
            header: t('graficos-gasto'),
            numeric: true,
            cell: (r) =>
              r.depth === 0 && r.categories.length === 1 && r.categories[0] ? (
                <Explainable
                  target={{
                    title: t('relatorios-item-em', { nome: r.name, periodo: periodLabel(period) }),
                    args: {
                      metric: 'spending',
                      params: {
                        ...period,
                        basis: data.basis,
                        category_id: r.categories[0],
                        member_id: memberId ?? undefined,
                      },
                    },
                  }}
                >
                  <Money cents={r.value} />
                </Explainable>
              ) : (
                <Money cents={r.value} />
              ),
          },
          {
            key: 'share',
            header: t('relatorios-pct-total'),
            numeric: true,
            cell: (r) => <Private>{formatPercent(r.share)}</Private>,
          },
          {
            key: 'group',
            header: t('relatorios-grupos-grupo'),
            sortValue: (r) => (r.group ? groupName(r.group) : null),
            cell: (r) =>
              r.group ? (
                <span className={r.depth ? 'text-dim' : 'text-fg'}>{groupName(r.group)}</span>
              ) : (
                '—'
              ),
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-composicao-pergunta')}</Question>
            <Stats>
              <Stat label={t('relatorios-despesas')}>
                <Explainable
                  target={{
                    title: t('relatorios-despesas-de', { periodo: periodLabel(period) }),
                    args: {
                      metric: 'spending',
                      params: { ...period, basis: data.basis, member_id: memberId ?? undefined },
                    },
                  }}
                >
                  <Money cents={data.total} estimated={data.estimated} />
                </Explainable>
              </Stat>
              <Stat label={t('graficos-lancamentos')}>
                <Private>{data.count}</Private>
              </Stat>
              <Stat label={t('relatorios-maior-categoria')}>{model.nodes[0]?.name ?? '—'}</Stat>
              <Stat label={t('relatorios-peso-maior')}>
                <Private>
                  {model.nodes[0] && model.total
                    ? formatPercent(model.nodes[0].value / model.total)
                    : '—'}
                </Private>
              </Stat>
            </Stats>
            <ChartFrame
              index="02"
              title={t('relatorios-composicao-titulo', { periodo: periodLabel(period) })}
              source={
                memberId || accountId
                  ? t('relatorios-fonte-despesas-filtro', { base: basisLabel(data.basis) })
                  : t('relatorios-fonte-despesas', { base: basisLabel(data.basis) })
              }
              actions={
                <Segmented
                  label={t('relatorios-forma')}
                  value={form}
                  onChange={setForm}
                  options={[
                    { value: 'treemap', label: t('relatorios-forma-treemap') },
                    { value: 'sunburst', label: t('relatorios-forma-sunburst') },
                  ]}
                />
              }
              chart={
                <CompositionChart
                  nodes={model.nodes}
                  form={form}
                  currency={data.currency}
                  height={form === 'sunburst' ? 460 : 420}
                  ariaLabel={t('relatorios-composicao-aria', {
                    periodo: periodLabel(period),
                    maiores: model.nodes
                      .slice(0, 3)
                      .map((n) => n.name)
                      .join(', '),
                  })}
                  onNodeClick={(id) => {
                    const cats = model.drill.get(id);
                    if (cats) drill(categoryDrill(scopeFor(data.basis), cats));
                  }}
                />
              }
              table={
                <TechTable
                  label={t('relatorios-composicao-tabela')}
                  prefsId="Composição por categoria"
                  columns={columns}
                  rows={rows}
                  getRowId={(r) => r.id}
                  dense
                />
              }
            />
            <Link
              to={groupsLink}
              className="label-caps inline-flex w-fit items-center gap-1 hover:text-accent"
            >
              {t('relatorios-grupos-composicao-link')} <Icon name="arrowRight" size={11} />
            </Link>
          </div>
        );
      }}
    </QueryView>
  );
}

// ── 03 · Ritmo ───────────────────────────────────────────────────

interface DayRow {
  date: string;
  value: number;
  count: number;
}

export function RhythmFigure({ period, basis, memberId, accountId, today }: FigureProps) {
  const window = useMemo(() => calendarWindow(period, today), [period, today]);
  const query = useCalendar({ ...window, basis: basis ?? undefined });
  const drill = useDrill();
  return (
    <QueryView
      query={query}
      isEmpty={(d) => d.total_expense <= 0}
      loadingLabel={t('relatorios-carregando-ritmo')}
    >
      {(data) => {
        const days = data.days.map((d) => ({ date: d.date, value: d.expense, count: d.count }));
        const elapsed = days.filter((d) => d.date <= today);
        const nDays = Math.max(1, Math.min(elapsed.length, daysBetween(window.from, today) + 1));
        const byWeekday = weekdayAverages(elapsed);
        const maxWd = Math.max(1, ...byWeekday.map((w) => w.avg));
        const top = topDays(days, 5);
        const rows: DayRow[] = days.filter((d) => d.value > 0);
        const columns: TechColumn<DayRow>[] = [
          {
            key: 'date',
            header: t('relatorios-dia'),
            numeric: true,
            sortValue: (r) => r.date,
            cell: (r) => formatDate(r.date),
          },
          {
            key: 'wd',
            header: t('relatorios-semana'),
            sortValue: (r) => weekday(r.date),
            cell: (r) => weekdays()[weekday(r.date)],
          },
          {
            key: 'value',
            header: t('graficos-gasto'),
            numeric: true,
            sortValue: (r) => r.value,
            cell: (r) => <Money cents={r.value} />,
          },
          {
            key: 'count',
            header: t('graficos-lancamentos'),
            numeric: true,
            sortValue: (r) => r.count,
            cell: (r) => <Private>{r.count}</Private>,
          },
        ];
        return (
          <div className="flex flex-col gap-4">
            <Question>{t('relatorios-ritmo-pergunta')}</Question>
            <Stats>
              <Stat label={t('relatorios-despesas-12-meses')}>
                <Money cents={data.total_expense} estimated={data.estimated} />
              </Stat>
              <Stat label={t('relatorios-media-por-dia')}>
                <Money cents={Math.round(data.total_expense / nDays)} />
              </Stat>
              <Stat
                label={t('relatorios-dia-mais-caro')}
                hint={top[0] ? formatDate(top[0].date) : undefined}
              >
                {top[0] ? <Money cents={top[0].value} /> : '—'}
              </Stat>
              <Stat label={t('relatorios-dias-sem-gasto')}>
                <Private>{elapsed.filter((d) => d.value === 0).length}</Private>
              </Stat>
            </Stats>
            <FilterNote memberId={memberId} accountId={accountId} />
            <ChartFrame
              index="03"
              title={t('relatorios-ritmo-titulo', { periodo: periodLabel(window) })}
              source={t('relatorios-ritmo-fonte', { base: basisLabel(data.basis) })}
              chart={
                <div className="flex flex-col gap-6">
                  <CalendarHeatmap
                    days={days}
                    range={window}
                    currency={data.currency}
                    ariaLabel={t('relatorios-ritmo-aria', {
                      periodo: periodLabel(window),
                      dia: top[0] ? formatDate(top[0].date) : t('relatorios-nenhum'),
                    })}
                    onDayClick={(d) => drill(dayDrill(d, data.basis))}
                  />
                  <div className="grid gap-6 @min-[530px]:grid-cols-2">
                    <section
                      aria-label={t('relatorios-media-dia-semana-aria')}
                      className="flex flex-col gap-2"
                    >
                      <span className="label-caps">{t('relatorios-media-dia-semana')}</span>
                      <ul className="flex flex-col gap-1.5">
                        {byWeekday.map((w) => (
                          <li
                            key={w.weekday}
                            className="grid grid-cols-[72px_1fr_96px] items-center gap-3 text-[12.5px]"
                          >
                            <span className="text-dim">{weekdays()[w.weekday]}</span>
                            <span aria-hidden className="h-2 rounded-[2px] bg-line/50">
                              <span
                                className="block h-full rounded-r-[4px] bg-accent"
                                style={{ width: `${(w.avg / maxWd) * 100}%` }}
                              />
                            </span>
                            <Money cents={w.avg} className="text-right text-fg" />
                          </li>
                        ))}
                      </ul>
                    </section>
                    <section
                      aria-label={t('relatorios-dias-maior-gasto')}
                      className="flex flex-col gap-2"
                    >
                      <span className="label-caps">{t('relatorios-dias-maior-gasto')}</span>
                      <ol className="flex flex-col">
                        {top.map((d, i) => (
                          <li key={d.date}>
                            <button
                              type="button"
                              onClick={() => drill(dayDrill(d.date, data.basis))}
                              className="grid w-full grid-cols-[20px_1fr_auto] items-center gap-3 rounded-control px-1 py-1 text-left text-[12.5px] hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-accent"
                            >
                              <span className="num text-[11px] text-accent">
                                {String(i + 1).padStart(2, '0')}
                              </span>
                              <span className="text-fg">
                                {t('relatorios-dia-semana-data', {
                                  semana: weekdays()[weekday(d.date)] ?? '',
                                  data: formatDate(d.date),
                                })}
                              </span>
                              <Money cents={d.value} className="text-fg" />
                            </button>
                          </li>
                        ))}
                      </ol>
                    </section>
                  </div>
                </div>
              }
              table={
                <TechTable
                  label={t('relatorios-gasto-por-dia')}
                  prefsId="Gasto por dia"
                  columns={columns}
                  rows={rows}
                  getRowId={(r) => r.date}
                  defaultSort={{ key: 'value', direction: 'desc' }}
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
