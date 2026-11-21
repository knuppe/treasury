/**
 * Corpo do projeto escolhido: números (despesas, receitas, resultado, orçamento, ritmo), a
 * linha do tempo de receita × despesa (acumulada ou por período), destaques, as categorias e
 * as entradas (lançamentos ligados e itens lançados à mão) com editar e tirar do projeto.
 */
import { useCallback, useMemo, useState, type ReactNode } from 'react';
import { useNavigate } from 'react-router';

import type { ProjectEntry } from '@/api/generated/ProjectEntry';
import type { ProjectView } from '@/api/generated/ProjectView';
import { useProjectEntries, useRemoveProjectEntries } from '@/api/hooks/projects';
import { useTxn } from '@/api/hooks/transactions';
import { useUndoToast } from '@/api/hooks/undoToast';
import { EChart } from '@/charts/EChart';
import { projectTimelineOption, type ProjectTimelineMode } from '@/charts/projectTimelineOption';
import type { ChartTheme } from '@/charts/theme';
import {
  InstrumentCard,
  Money,
  Private,
  TechTable,
  type ContextMenuEntry,
  type TechTableMenuContext,
} from '@/design/components';
import { ProgressMeter } from '@/features/goals/GoalsPanel';
import { Stat, Stats } from '@/features/reports/figures/common';
import { TxnDialog } from '@/features/transactions/TxnDialog';
import { ConfirmDialog, Segmented } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { formatDate, formatMonthLong, formatPercent, todayIso } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';

import { ProjectItemDialog } from './ProjectDialogs';
import { buildTimeline, projectStats, type ProjectTimeline } from './timeline';

const periodLabel = (tl: ProjectTimeline, p: string) =>
  tl.grain === 'day' ? formatDate(p) : formatMonthLong(p);

function TimelineCard({
  project,
  timeline,
  loading,
  onNewItem,
}: {
  project: ProjectView;
  timeline: ProjectTimeline | null;
  loading: boolean;
  onNewItem: () => void;
}) {
  const [mode, setMode] = useState<ProjectTimelineMode>('acumulado');
  // Os valores do projeto são na moeda base da família (como no núcleo).
  const currency = useBaseCurrency();
  const option = useCallback(
    (theme: ChartTheme) =>
      projectTimelineOption(theme, {
        mode,
        ...timeline!,
        budget: project.budget,
        currency,
      }),
    [mode, timeline, project.budget, currency],
  );
  const daily = timeline?.grain === 'day';
  const by = daily ? t('projetos-por-dia') : t('projetos-por-mes');
  return (
    <InstrumentCard
      index="05.P1"
      title={t('projetos-linha-tempo')}
      headingLevel={4}
      actions={
        timeline ? (
          <Segmented<ProjectTimelineMode>
            label={t('projetos-visao-linha')}
            size="sm"
            value={mode}
            onChange={setMode}
            options={[
              { value: 'acumulado', label: t('projetos-acumulado') },
              { value: 'periodo', label: by },
            ]}
          />
        ) : null
      }
    >
      {timeline ? (
        <EChart
          option={option}
          height={300}
          ariaLabel={
            mode === 'acumulado'
              ? t(
                  project.budget
                    ? 'projetos-grafico-acumulado-orcamento'
                    : 'projetos-grafico-acumulado',
                  {
                    nome: project.name,
                    de: periodLabel(timeline, timeline.periods[0]!),
                    ate: periodLabel(timeline, timeline.periods[timeline.periods.length - 1]!),
                  },
                )
              : t(daily ? 'projetos-grafico-periodo-dia' : 'projetos-grafico-periodo-mes', {
                  nome: project.name,
                })
          }
        />
      ) : (
        <p className="text-[13px] text-dim">
          {loading
            ? t('projetos-montando')
            : withSlots(t('projetos-linha-vazia', { lance: slot(0) }), [
                <button
                  key="lance"
                  type="button"
                  className="text-accent hover:underline"
                  onClick={onNewItem}
                >
                  {t('projetos-linha-vazia-lance')}
                </button>,
              ])}
        </p>
      )}
    </InstrumentCard>
  );
}

function entryLine(e: ProjectEntry) {
  return (
    <span className="flex min-w-0 flex-col">
      <span className="truncate">{e.description || t('projetos-sem-descricao')}</span>
      <span className="flex items-baseline gap-2 text-[12px] text-dim">
        {formatDate(e.date)}
        <Money cents={e.amount} tone="auto" />
      </span>
    </span>
  );
}

/** Destaques: rótulo em cima, valor embaixo (cabem descrições longas em cards estreitos). */
function Highlights({ items }: { items: readonly { label: string; value: ReactNode }[] }) {
  return (
    <dl className="grid gap-x-6 gap-y-3 @min-[420px]:grid-cols-2">
      {items.map((h) => (
        <div key={h.label} className="flex min-w-0 flex-col gap-0.5">
          <dt className="label-caps">{h.label}</dt>
          <dd className="min-w-0 text-[13px] text-fg">{h.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function CategoryList({
  title,
  label,
  items,
  empty,
}: {
  title: string;
  label: string;
  items: ProjectView['by_category'];
  empty: string;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <h5 className="label-caps">{title}</h5>
      {items.length === 0 ? (
        <p className="text-[12.5px] text-dim">{empty}</p>
      ) : (
        <ul aria-label={label} className="flex flex-col gap-1.5">
          {items.map((c) => (
            <li key={c.category_id ?? 'sem'} className="flex items-baseline gap-3 text-[13px]">
              <span className="min-w-0 truncate">
                {c.category_path ?? t('controles-sem-categoria')}
              </span>
              <span className="ml-auto">
                <Money cents={c.spent} />
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** Edição de um lançamento da conta: busca o lançamento e abre o editor completo. */
function EditTxn({ id, onClose }: { id: string; onClose: () => void }) {
  const q = useTxn(id);
  if (!q.data) return null;
  return <TxnDialog open txn={q.data} onOpenChange={(o) => (!o ? onClose() : undefined)} />;
}

export function ProjectOverview({
  project,
  onNewItem,
  onAdd,
}: {
  project: ProjectView;
  onNewItem: () => void;
  onAdd: () => void;
}) {
  const p = project;
  const q = useProjectEntries(p.id);
  const entries = useMemo(() => q.data ?? [], [q.data]);
  const today = todayIso();
  const timeline = useMemo(() => buildTimeline(p, entries, today), [p, entries, today]);
  const stats = projectStats(p, entries, timeline, today);
  const navigate = useNavigate();
  const remove = useRemoveProjectEntries();
  const undoToast = useUndoToast();
  const [editingItem, setEditingItem] = useState<ProjectEntry | null>(null);
  const [editingTxn, setEditingTxn] = useState<string | null>(null);
  const [removing, setRemoving] = useState<readonly ProjectEntry[] | null>(null);
  const over = p.remaining != null && p.remaining < 0;

  const edit = (e: ProjectEntry) => (e.source === 'item' ? setEditingItem(e) : setEditingTxn(e.id));
  const menu = (e: ProjectEntry, ctx: TechTableMenuContext): ContextMenuEntry[] => [
    { heading: e.description || t('projetos-entrada-sem-descricao') },
    {
      label: e.source === 'item' ? t('projetos-editar-item') : t('projetos-editar-lancamento'),
      icon: 'sliders',
      shortcut: ['enter'],
      onSelect: () => edit(e),
    },
    e.source === 'txn' && {
      label: t('projetos-ver-em-lancamentos'),
      icon: 'arrowRight',
      onSelect: () =>
        void navigate(txnLink({ project_ids: [p.id], date_from: e.date, date_to: e.date })),
    },
    {
      label: t('componentes-copiar-linha'),
      icon: 'copy',
      shortcut: ['mod', 'C'],
      onSelect: ctx.copy,
    },
    { separator: true },
    {
      label: e.source === 'item' ? t('projetos-excluir-item') : t('projetos-tirar-menu'),
      icon: 'x',
      danger: true,
      shortcut: ['mod', 'Backspace'],
      onSelect: () => setRemoving([e]),
    },
  ];
  const selectionMenu = (rows: ProjectEntry[], ctx: TechTableMenuContext): ContextMenuEntry[] => [
    { heading: t('projetos-n-entradas-selecionadas', { n: rows.length }) },
    {
      label: t('componentes-copiar-linhas', { n: rows.length }),
      icon: 'copy',
      shortcut: ['mod', 'C'],
      onSelect: ctx.copy,
    },
    { separator: true },
    {
      label: t('projetos-tirar-n', { n: rows.length }),
      icon: 'x',
      danger: true,
      shortcut: ['mod', 'Backspace'],
      onSelect: () => setRemoving(rows),
    },
  ];

  const removeCount = removing?.length ?? 0;
  const removeItems = removing?.filter((e) => e.source === 'item').length ?? 0;
  const highlights: { label: string; value: ReactNode }[] = [
    {
      label: t('projetos-maior-despesa'),
      value: stats.biggestExpense ? entryLine(stats.biggestExpense) : '—',
    },
    {
      label: t('projetos-maior-receita'),
      value: stats.biggestIncome ? entryLine(stats.biggestIncome) : '—',
    },
    {
      label:
        timeline?.grain === 'day' ? t('projetos-dia-maior-gasto') : t('projetos-mes-maior-gasto'),
      value:
        stats.peak && timeline ? (
          <span className="flex flex-col">
            <span>{periodLabel(timeline, stats.peak.period)}</span>
            <span className="text-[12px] text-dim">
              <Money cents={stats.peak.value} />
            </span>
          </span>
        ) : (
          '—'
        ),
    },
    ...(stats.budgetEnds
      ? [
          {
            label: t('projetos-orcamento-acaba'),
            value: t('projetos-orcamento-acaba-em', { mes: formatMonthLong(stats.budgetEnds) }),
          },
        ]
      : []),
  ];

  return (
    <div className="flex flex-col gap-4">
      <Stats>
        <Stat
          label={t('projetos-despesas')}
          hint={over ? t('projetos-acima-orcamento') : undefined}
        >
          <Money cents={p.spent} />
        </Stat>
        <Stat label={t('projetos-receitas')}>
          <Money cents={p.income} />
        </Stat>
        <Stat label={t('projetos-resultado')} hint={t('projetos-resultado-dica')}>
          <Money cents={p.net} tone="auto" />
        </Stat>
        <Stat
          label={t('projetos-orcamento')}
          hint={
            p.remaining != null
              ? withSlots(
                  t(over ? 'projetos-passou' : 'projetos-resta-valor', { valor: slot(0) }),
                  [<Money key="resta" cents={Math.abs(p.remaining)} />],
                )
              : t('projetos-sem-orcamento')
          }
        >
          {stats.budgetUsed != null ? (
            <Private>{formatPercent(stats.budgetUsed, { decimals: 0 })}</Private>
          ) : (
            '—'
          )}
          {p.budget ? (
            <div className="mt-1.5">
              <ProgressMeter
                value={p.spent / p.budget}
                label={t('projetos-pct-orcamento', { pct: Math.round((p.spent / p.budget) * 100) })}
                tone={over ? 'var(--negative)' : undefined}
              />
            </div>
          ) : null}
        </Stat>
        <Stat
          label={t('projetos-gasto-mes')}
          hint={timeline ? t('projetos-media-meses', { n: stats.months }) : undefined}
        >
          {timeline ? <Money cents={stats.monthlyPace} /> : '—'}
        </Stat>
        <Stat
          label={t('projetos-entradas')}
          hint={t('projetos-entradas-dica', { lancamentos: p.txn_count, itens: p.item_count })}
        >
          {p.txn_count + p.item_count}
        </Stat>
      </Stats>
      <TimelineCard project={p} timeline={timeline} loading={q.isPending} onNewItem={onNewItem} />
      <div className="grid gap-4 @min-[760px]:grid-cols-2">
        <InstrumentCard index="05.P2" title={t('projetos-destaques')} headingLevel={4}>
          <Highlights items={highlights} />
          {p.improvement_ids.length > 0 ? (
            <p className="mt-3 text-[12.5px] text-dim">
              {t('projetos-alimenta', {
                n: p.improvement_ids.length,
                bem: p.physical_asset_name ?? t('projetos-um-bem'),
              })}
            </p>
          ) : null}
          {p.notes ? <p className="mt-3 text-[12.5px] text-dim">{p.notes}</p> : null}
        </InstrumentCard>
        <InstrumentCard index="05.P3" title={t('projetos-por-categoria')} headingLevel={4}>
          <div className="flex flex-col gap-4">
            <CategoryList
              title={t('projetos-despesas')}
              label={t('projetos-despesas-categoria', { nome: p.name })}
              items={p.by_category}
              empty={t('projetos-nenhuma-despesa')}
            />
            <CategoryList
              title={t('projetos-receitas')}
              label={t('projetos-receitas-categoria', { nome: p.name })}
              items={p.income_by_category}
              empty={t('projetos-nenhuma-receita')}
            />
          </div>
        </InstrumentCard>
      </div>
      <InstrumentCard index="05.P4" title={t('projetos-entradas')} headingLevel={4} padded={false}>
        <TechTable<ProjectEntry>
          label={t('projetos-entradas-de', { nome: p.name })}
          prefsId="Entradas do projeto"
          rows={entries}
          getRowId={(e) => `${e.source}:${e.id}`}
          defaultSort={{ key: 'date', direction: 'desc' }}
          onActivate={edit}
          rowMenu={menu}
          rowMenuLabel={(e) =>
            t('projetos-acoes-de', {
              nome: e.description || t('projetos-entrada-sem-descricao-min'),
            })
          }
          multiSelect
          selectionMenu={selectionMenu}
          selectionMenuLabel={(rows) => t('projetos-acoes-n-entradas', { n: rows.length })}
          selectionStatus={(rows) => {
            const income = rows
              .filter((e) => e.flow === 'income')
              .reduce((sum, e) => sum + e.amount, 0);
            const expense = rows
              .filter((e) => e.flow === 'expense')
              .reduce((sum, e) => sum - e.amount, 0);
            return (
              <>
                <span className="num text-accent">
                  {t('projetos-n-entradas-selecionadas', { n: rows.length })}
                </span>
                {expense ? (
                  <span className="flex items-baseline gap-1.5">
                    <span className="label-caps">{t('projetos-despesas')}</span>
                    <Money cents={expense} />
                  </span>
                ) : null}
                {income ? (
                  <span className="flex items-baseline gap-1.5">
                    <span className="label-caps">{t('projetos-receitas')}</span>
                    <Money cents={income} />
                  </span>
                ) : null}
              </>
            );
          }}
          maxHeight="420px"
          dense
          empty={
            <span>
              {withSlots(t('projetos-entradas-vazio', { item: slot(0), lancamentos: slot(1) }), [
                <button
                  key="item"
                  type="button"
                  className="text-accent hover:underline"
                  onClick={onNewItem}
                >
                  {t('projetos-entradas-vazio-item')}
                </button>,
                <button
                  key="lancamentos"
                  type="button"
                  className="text-accent hover:underline"
                  onClick={onAdd}
                >
                  {t('projetos-entradas-vazio-lancamentos')}
                </button>,
              ])}
            </span>
          }
          columns={[
            {
              key: 'date',
              header: t('projetos-data'),
              numeric: true,
              sortValue: (e) => e.date,
              cell: (e) => formatDate(e.date),
            },
            {
              key: 'description',
              header: t('projetos-descricao'),
              sortValue: (e) => e.description,
              cell: (e) => (
                <span className="flex min-w-0 items-center gap-2">
                  <span className="min-w-0 truncate">
                    {e.description || t('projetos-sem-descricao')}
                  </span>
                  {e.planned ? (
                    <span className="label-caps text-warning">{t('projetos-previsto')}</span>
                  ) : null}
                </span>
              ),
            },
            {
              key: 'category',
              header: t('controles-categoria'),
              sortValue: (e) => e.category_path,
              cell: (e) => <span className="text-dim">{e.category_path ?? '—'}</span>,
            },
            {
              key: 'origin',
              header: t('projetos-origem'),
              sortValue: (e) => e.account_name ?? '',
              cell: (e) => (
                <span className="text-dim">{e.account_name ?? t('projetos-item-do-projeto')}</span>
              ),
            },
            {
              key: 'flow',
              header: t('projetos-tipo'),
              sortValue: (e) => e.flow,
              cell: (e) =>
                e.flow === 'income' ? t('controles-tipo-receita') : t('controles-tipo-despesa'),
            },
            {
              key: 'amount',
              header: t('projetos-valor'),
              numeric: true,
              sortValue: (e) => e.amount,
              cell: (e) => <Money cents={e.amount} currency={e.currency} tone="auto" />,
            },
          ]}
          footer={
            entries.length
              ? {
                  description: t('projetos-n-entradas', { n: entries.length }),
                  amount: <Money cents={entries.reduce((s, e) => s + e.amount, 0)} tone="auto" />,
                }
              : undefined
          }
        />
      </InstrumentCard>
      {editingItem ? (
        <ProjectItemDialog
          project={p}
          entry={editingItem}
          open
          onOpenChange={(o) => (!o ? setEditingItem(null) : undefined)}
        />
      ) : null}
      {editingTxn ? <EditTxn id={editingTxn} onClose={() => setEditingTxn(null)} /> : null}
      <ConfirmDialog
        open={removing != null}
        onOpenChange={(o) => (!o ? setRemoving(null) : undefined)}
        title={
          removeCount === 1
            ? removeItems
              ? t('projetos-excluir-item-titulo')
              : t('projetos-tirar-lancamento-titulo')
            : t('projetos-tirar-n-titulo', { n: removeCount })
        }
        description={
          removeItems === removeCount
            ? t('projetos-tirar-itens-descricao')
            : removeItems
              ? t('projetos-tirar-misto-descricao')
              : t('projetos-tirar-lancamento-descricao')
        }
        confirmLabel={removeItems === removeCount ? t('projetos-excluir') : t('projetos-tirar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          removing &&
          remove.mutate(
            {
              txn_ids: removing.filter((e) => e.source === 'txn').map((e) => e.id),
              item_ids: removing.filter((e) => e.source === 'item').map((e) => e.id),
            },
            {
              onSuccess: () => {
                undoToast({
                  title: t('projetos-entradas-tiradas'),
                  description: t('projetos-entradas-tiradas-descricao', {
                    n: removeCount,
                    nome: p.name,
                  }),
                });
                setRemoving(null);
              },
            },
          )
        }
      />
    </div>
  );
}
