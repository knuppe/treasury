/**
 * Analítico de uma linha do fluxo (comparativo e Sankey, `reports.sankey_items`): os
 * lançamentos que compõem cada célula, ao centavo, no período, no anterior e na diferença,
 * com as subcategorias. A categoria de cada lançamento se corrige na própria linha (o
 * relatório recalcula na hora) e "Editar" abre o lançamento completo; na sobra e no déficit,
 * a linha abre o analítico da linha que entra na conta.
 */
import { useState } from 'react';

import type { Basis } from '@/api/generated/Basis';
import type { SankeyItem } from '@/api/generated/SankeyItem';
import type { SankeyItems } from '@/api/generated/SankeyItems';
import type { SankeyItemsPeriod } from '@/api/generated/SankeyItemsPeriod';
import { useSankeyItems } from '@/api/hooks/reports';
import { useTxn, useUpdateTxn } from '@/api/hooks/transactions';
import {
  Button,
  Dialog,
  Icon,
  Money,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { TxnDialog } from '@/features/transactions/TxnDialog';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import { errorText } from '@/features/transactions/ui/errors';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatMoney } from '@/lib/format';

import { capitalize, Loading, QueryError, Segmented } from '../controls';
import { periodLabel, type Period } from '../model/period';

export type FlowItemsView = 'current' | 'previous' | 'delta';

/** Linha (ou linhas, como "Outras") a abrir e a célula clicada. */
export interface FlowItemsTarget {
  nodeIds: string[];
  /** Título no lugar do nome do nó (`Outras`). */
  title?: string;
  view: FlowItemsView;
}

export interface FlowItemsScope {
  period: Period;
  basis: Basis;
  depth: 1 | 2;
}

type ItemRow = SankeyItem & {
  _id: string;
  /** Na diferença, o período do item. */
  _period?: string;
};

interface DeltaRow {
  key: string;
  label: string;
  current: number;
  previous: number;
  delta: number;
  count: number;
}

type GroupBy = 'category' | 'description';

const groupByOptions = () =>
  [
    {
      value: 'category',
      label: t('relatorios-categoria'),
      hint: t('relatorios-agrupar-categoria-dica'),
    },
    {
      value: 'description',
      label: t('relatorios-descricao'),
      hint: t('relatorios-agrupar-descricao-dica'),
    },
  ] as const;

const normalize = (s: string) => s.trim().toLocaleLowerCase('pt-BR').replace(/\s+/g, ' ');

/** Variação por descrição (só o que conta), maior variação primeiro. */
function byDescription(data: SankeyItems): DeltaRow[] {
  const rows = new Map<string, DeltaRow>();
  const add = (items: readonly SankeyItem[], cur: boolean) => {
    for (const i of items) {
      if (!i.counted) continue;
      const key = normalize(i.label);
      let r = rows.get(key);
      if (!r) {
        r = { key, label: i.label, current: 0, previous: 0, delta: 0, count: 0 };
        rows.set(key, r);
      }
      if (cur) r.current += i.amount;
      else r.previous += i.amount;
      r.count += 1;
    }
  };
  add(data.current.items, true);
  add(data.previous.items, false);
  return [...rows.values()]
    .map((r) => ({ ...r, delta: r.current - r.previous }))
    .sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta) || a.label.localeCompare(b.label));
}

function byCategory(data: SankeyItems): DeltaRow[] {
  return data.groups.map((g) => ({
    key: `${g.group}|${g.category_id ?? ''}`,
    label: g.group,
    current: g.current,
    previous: g.previous,
    delta: g.delta,
    count: g.current_count + g.previous_count,
  }));
}

/** Edição de um lançamento do analítico: busca o lançamento e abre o editor completo. */
function EditTxn({ id, onClose }: { id: string; onClose: () => void }) {
  const q = useTxn(id);
  if (!q.data) return null;
  return <TxnDialog open txn={q.data} onOpenChange={(o) => (!o ? onClose() : undefined)} />;
}

/** Item que é um lançamento de receita ou despesa (dá para trocar a categoria). */
const isFlowTxn = (i: SankeyItem) =>
  i.txn_id !== null && i.node_id !== 'investment' && i.node_id !== 'redemption';

/**
 * Categoria do lançamento, editável na linha: escolher outra grava na hora e o relatório
 * recalcula (o lançamento pode sair desta linha). Lançamento dividido abre o editor, onde
 * cada parte tem a sua categoria.
 */
function CategoryCell({ item, onEdit }: { item: SankeyItem; onEdit: (id: string) => void }) {
  const update = useUpdateTxn();
  if (!isFlowTxn(item)) return <span className="text-dim">{item.group}</span>;
  if (item.split) {
    return (
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          onEdit(item.txn_id!);
        }}
        onDoubleClick={(e) => e.stopPropagation()}
        title={t('relatorios-dividido-dica')}
        className="inline-flex min-w-0 items-center gap-1.5 rounded-control text-left text-[12.5px] text-accent hover:underline"
      >
        <span className="truncate">{item.group}</span>
        <span className="shrink-0 text-dim">{t('relatorios-dividido')}</span>
      </button>
    );
  }
  return (
    // A tabela trata ↑/↓/Enter das linhas: o seletor guarda as próprias teclas.
    <span
      className="block max-w-full min-w-0 overflow-hidden"
      onClick={(e) => e.stopPropagation()}
      onDoubleClick={(e) => e.stopPropagation()}
      onKeyDown={(e) => e.stopPropagation()}
    >
      <CategoryPicker
        compact
        kind={item.node_id.startsWith('income:') ? 'income' : 'expense'}
        value={item.category_id}
        label={t('relatorios-categoria-de', { nome: item.label })}
        onChange={(category_id) => {
          if (category_id === item.category_id) return;
          update.mutate(
            { id: item.txn_id!, category_id },
            {
              onSuccess: () =>
                toast({
                  tone: 'success',
                  title: t('relatorios-categoria-alterada'),
                  description: t('relatorios-relatorio-recalculado', { nome: item.label }),
                }),
              onError: (e) =>
                toast({
                  tone: 'error',
                  title: t('relatorios-categoria-nao-mudou'),
                  description: errorText(e),
                }),
            },
          );
        }}
      />
    </span>
  );
}

/** Colunas da lista de lançamentos (com a do período na diferença). */
function itemColumns({
  derived,
  withPeriod,
  onEdit,
}: {
  derived: boolean;
  withPeriod: boolean;
  onEdit: (id: string) => void;
}): TechColumn<ItemRow>[] {
  return [
    ...(withPeriod
      ? [
          {
            key: 'period',
            header: t('relatorios-periodo'),
            width: '170px',
            sortValue: (r: ItemRow) => r._period ?? '',
            cell: (r: ItemRow) => <span className="whitespace-nowrap text-dim">{r._period}</span>,
          },
        ]
      : []),
    ...(derived
      ? []
      : [
          {
            key: 'date',
            header: t('relatorios-data'),
            width: '96px',
            sortValue: (r: ItemRow) => r.date ?? '',
            cell: (r: ItemRow) => (r.date ? formatDate(r.date) : '—'),
          },
        ]),
    {
      key: 'label',
      header: derived ? t('relatorios-linha') : t('relatorios-descricao'),
      sortValue: (r) => r.label,
      cell: (r) => <Description item={r} />,
    },
    {
      key: 'group',
      header: derived ? t('relatorios-grupo') : t('relatorios-categoria'),
      width: derived ? undefined : '240px',
      sortValue: (r) => r.group,
      cell: (r) => <CategoryCell item={r} onEdit={onEdit} />,
    },
    ...(derived
      ? []
      : [
          {
            key: 'account',
            header: t('relatorios-conta'),
            sortValue: (r: ItemRow) => r.account ?? '',
            cell: (r: ItemRow) => <span className="text-dim">{r.account ?? '—'}</span>,
          },
        ]),
    {
      key: 'amount',
      header: t('graficos-valor'),
      width: '140px',
      numeric: true,
      sortValue: (r) => r.amount,
      cell: (r) => <Money cents={r.amount} estimated={r.estimated} signed={r.amount < 0} />,
    },
    ...(derived
      ? []
      : [
          {
            key: 'edit',
            header: '',
            title: t('comum-editar'),
            width: '76px',
            sortable: false,
            hideable: false,
            cell: (r: ItemRow) =>
              r.txn_id ? (
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={t('relatorios-editar-item', { nome: r.label })}
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit(r.txn_id!);
                  }}
                  onDoubleClick={(e) => e.stopPropagation()}
                >
                  {t('comum-editar')}
                </Button>
              ) : null,
          },
        ]),
  ];
}

function Description({ item }: { item: SankeyItem }) {
  return (
    <span className="flex min-w-0 flex-col">
      <span className={cx('truncate', item.estimated && 'italic')}>
        {item.estimated ? '≈ ' : ''}
        {item.label}
      </span>
      {item.split || item.due_date || item.original_currency ? (
        <span className="text-[11.5px] text-dim">
          {[
            item.split ? t('relatorios-parte-dividido') : null,
            item.due_date
              ? t('relatorios-fatura-vence', { data: formatDate(item.due_date) })
              : null,
            item.original_currency && item.original_amount !== null
              ? formatMoney(item.original_amount, item.original_currency)
              : null,
          ]
            .filter(Boolean)
            .join(' · ')}
        </span>
      ) : null}
    </span>
  );
}

/** Um dos três números do topo (atual, anterior, diferença), que escolhe a vista. */
function PeriodTab({
  id,
  label,
  cents,
  selected,
  onSelect,
  signed = false,
}: {
  id: FlowItemsView;
  label: string;
  cents: number;
  selected: boolean;
  onSelect: (v: FlowItemsView) => void;
  signed?: boolean;
}) {
  return (
    <button
      type="button"
      role="tab"
      id={`flow-items-tab-${id}`}
      aria-selected={selected}
      aria-controls="flow-items-panel"
      onClick={() => onSelect(id)}
      className={cx(
        'flex min-w-0 flex-1 flex-col items-start gap-1 rounded-control border px-3.5 py-2.5 text-left transition-colors',
        selected ? 'border-accent bg-accent-soft/40' : 'border-line hover:border-line-strong',
      )}
    >
      <span className="label-caps">{label}</span>
      <span className="num text-[20px] leading-none font-medium text-fg">
        <Money cents={cents} signed={signed} trend={signed} />
      </span>
    </button>
  );
}

/** Confere a soma dos itens com o valor da tabela. */
function SumCheck({ p, label }: { p: SankeyItemsPeriod; label: string }) {
  const n = p.items.filter((i) => i.counted).length;
  if (p.items_total === p.value) {
    return (
      <p className="flex items-center gap-2 text-[12.5px] text-positive">
        <Icon name="check" size={13} />
        <span>
          {withSlots(t('relatorios-soma-confere', { n, soma: slot(0), periodo: label }), [
            <Money key="soma" cents={p.items_total} />,
          ])}
        </span>
      </p>
    );
  }
  return (
    <p role="alert" className="flex items-center gap-2 text-[12.5px] text-warning">
      <Icon name="alert" size={13} />
      <span>
        {withSlots(t('relatorios-soma-difere', { soma: slot(0), valor: slot(1) }), [
          <Money key="soma" cents={p.items_total} />,
          <Money key="valor" cents={p.value} />,
        ])}
      </span>
    </p>
  );
}

function PeriodView({
  data,
  view,
  label,
  sentenceLabel,
  onOpenItem,
  onEdit,
}: {
  data: SankeyItems;
  view: 'current' | 'previous';
  label: string;
  /** O período no meio da frase (sem a maiúscula do título). */
  sentenceLabel: string;
  onOpenItem: (item: SankeyItem) => void;
  onEdit: (id: string) => void;
}) {
  const p = data[view];
  const derived = p.items.some((i) => i.component !== null);
  const rows: ItemRow[] = p.items.map((i, n) => ({ ...i, _id: String(n) }));
  const counted = rows.filter((r) => r.counted);
  const left = rows.filter((r) => !r.counted);
  const groups = data.groups
    .map((g) => ({
      ...g,
      value: view === 'current' ? g.current : g.previous,
      count: view === 'current' ? g.current_count : g.previous_count,
    }))
    .filter((g) => g.count > 0)
    .sort((a, b) => b.value - a.value || a.group.localeCompare(b.group));
  const columns = itemColumns({ derived, withPeriod: false, onEdit });
  const leftColumns: TechColumn<ItemRow>[] = [
    ...columns.filter((c) => !['amount', 'account', 'edit'].includes(c.key)),
    {
      key: 'original',
      header: t('graficos-valor'),
      width: '140px',
      numeric: true,
      sortable: false,
      cell: (r) =>
        r.original_currency && r.original_amount !== null ? (
          <Money cents={r.original_amount} currency={r.original_currency} />
        ) : (
          <Money cents={r.amount} signed={r.amount < 0} />
        ),
    },
    {
      key: 'note',
      header: t('relatorios-por-que-fora'),
      sortable: false,
      cell: (r) => <span className="text-[12px] text-dim">{r.note}</span>,
    },
  ];
  return (
    <div className="flex flex-col gap-3">
      {p.note ? (
        <p className="text-[13px] text-dim">{p.note}</p>
      ) : (
        <SumCheck p={p} label={sentenceLabel} />
      )}
      {p.estimated ? (
        <p className="flex items-center gap-2 text-[12.5px] text-warning italic">
          <Icon name="reticleDashed" size={13} /> {t('relatorios-cambio-estimado')}
        </p>
      ) : null}
      {groups.length > 1 && !derived ? (
        <ul
          aria-label={t('relatorios-por-categoria-em', { periodo: label })}
          className="flex flex-col gap-1"
        >
          {groups.map((g) => (
            <li
              key={`${g.group}|${g.category_id ?? ''}`}
              className="flex items-baseline gap-3 text-[13px]"
            >
              <span className="min-w-0 truncate">{g.group}</span>
              <span className="text-[11.5px] text-dim">
                {t('relatorios-itens', { n: g.count })}
              </span>
              <span className="ml-auto">
                <Money cents={g.value} signed={g.value < 0} />
              </span>
            </li>
          ))}
        </ul>
      ) : null}
      {p.note && !counted.length ? null : (
        <TechTable
          label={t('relatorios-o-que-compoe', { nome: data.name, periodo: label })}
          prefsId="O que compõe"
          columns={columns}
          rows={counted}
          getRowId={(r) => r._id}
          onActivate={onOpenItem}
          defaultSort={derived ? null : { key: 'date', direction: 'asc' }}
          footer={{
            label: <span className="label-caps">{t('relatorios-total')}</span>,
            amount: <Money cents={p.items_total} />,
          }}
          maxHeight="340px"
          minWidth={derived ? '520px' : '760px'}
          dense
          empty={t('relatorios-nada-em', { periodo: label })}
        />
      )}
      {counted.length ? (
        <p className="text-[12px] text-dim">
          {derived ? t('relatorios-abre-analitico') : t('relatorios-editar-dica')}
        </p>
      ) : null}
      {left.length ? (
        <section aria-label={t('relatorios-fora-total')} className="flex flex-col gap-2">
          <h4 className="label-caps text-warning">
            {t('relatorios-fora-total-n', { n: left.length })}
          </h4>
          <TechTable
            label={t('relatorios-fora-total-de', { nome: data.name, periodo: label })}
            prefsId="Fora do total"
            columns={leftColumns}
            rows={left}
            getRowId={(r) => r._id}
            onActivate={onOpenItem}
            maxHeight="200px"
            minWidth="760px"
            dense
          />
        </section>
      ) : null}
      {p.missing_fx.length ? (
        <p className="flex items-center gap-2 text-[12.5px] text-warning">
          <Icon name="alert" size={13} />{' '}
          {t('relatorios-sem-cambio', { moedas: p.missing_fx.join(', ') })}
        </p>
      ) : null}
    </div>
  );
}

/** Chave do grupo de um item, a mesma das linhas da diferença. */
function groupKey(i: SankeyItem, groupBy: GroupBy): string {
  return groupBy === 'category' ? `${i.group}|${i.category_id ?? ''}` : normalize(i.label);
}

function DeltaView({
  data,
  currentLabel,
  previousLabel,
  currentSentence,
  previousSentence,
  onOpenItem,
  onEdit,
}: {
  data: SankeyItems;
  currentLabel: string;
  previousLabel: string;
  /** Os períodos no meio da frase (sem a maiúscula do título). */
  currentSentence: string;
  previousSentence: string;
  onOpenItem: (item: SankeyItem) => void;
  onEdit: (id: string) => void;
}) {
  const [groupBy, setGroupBy] = useState<GroupBy>('category');
  const [selected, setSelected] = useState<string | null>(null);
  const rows = groupBy === 'category' ? byCategory(data) : byDescription(data);
  // A linha escolhida (ou a de maior variação) mostra os lançamentos dela abaixo.
  const row = rows.find((r) => r.key === selected) ?? rows[0] ?? null;
  const derived = data.current.items.concat(data.previous.items).some((i) => i.component);
  const detail: ItemRow[] = row
    ? [
        ...data.current.items.map((i, n) => ({ ...i, _id: `c${n}`, _period: currentLabel })),
        ...data.previous.items.map((i, n) => ({ ...i, _id: `p${n}`, _period: previousLabel })),
      ].filter((i) => i.counted && groupKey(i, groupBy) === row.key)
    : [];
  const cur = data.current.items_total;
  const prev = data.previous.items_total;
  const columns: TechColumn<DeltaRow>[] = [
    {
      key: 'label',
      header: groupBy === 'category' ? t('relatorios-categoria') : t('relatorios-descricao'),
      sortValue: (r) => r.label,
    },
    {
      key: 'previous',
      header: previousLabel,
      numeric: true,
      sortValue: (r) => r.previous,
      cell: (r) => <Money cents={r.previous} signed={r.previous < 0} />,
    },
    {
      key: 'current',
      header: currentLabel,
      numeric: true,
      sortValue: (r) => r.current,
      cell: (r) => <Money cents={r.current} signed={r.current < 0} />,
    },
    {
      key: 'delta',
      header: 'Δ',
      numeric: true,
      sortValue: (r) => r.delta,
      cell: (r) => <Money cents={r.delta} trend signed />,
    },
  ];
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[13px] text-fg">
        {withSlots(
          t('relatorios-diferenca-resumo', {
            anterior: previousSentence,
            antes: slot(0),
            atual: currentSentence,
            depois: slot(1),
            diferenca: slot(2),
          }),
          [
            <Money key="antes" cents={prev} />,
            <Money key="depois" cents={cur} />,
            <Money key="diferenca" cents={cur - prev} trend signed />,
          ],
        )}
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[12px] text-dim">{t('relatorios-agrupar-por')}</span>
        <Segmented
          label={t('relatorios-agrupar-diferenca')}
          value={groupBy}
          onChange={setGroupBy}
          options={groupByOptions()}
        />
      </div>
      <TechTable
        label={
          groupBy === 'category'
            ? t('relatorios-diferenca-por-categoria', { nome: data.name })
            : t('relatorios-diferenca-por-descricao', { nome: data.name })
        }
        prefsId={groupBy === 'category' ? 'Diferença por categoria' : 'Diferença por descrição'}
        columns={columns}
        rows={rows}
        getRowId={(r) => r.key}
        selectedId={row?.key ?? null}
        onSelectedChange={(id) => setSelected(id)}
        footer={{
          label: <span className="label-caps">{t('relatorios-total')}</span>,
          previous: <Money cents={prev} />,
          current: <Money cents={cur} />,
          delta: <Money cents={cur - prev} trend signed />,
        }}
        maxHeight="240px"
        minWidth="640px"
        dense
        empty={t('relatorios-nada-mudou')}
      />
      {row ? (
        <section
          aria-label={t('relatorios-lancamentos-de', { nome: row.label })}
          className="flex flex-col gap-2"
        >
          <h4 className="label-caps">
            {derived
              ? t('relatorios-linhas-dois-periodos', { nome: row.label })
              : t('relatorios-lancamentos-dois-periodos', { nome: row.label })}
          </h4>
          <TechTable
            label={t('relatorios-lancamentos-de', { nome: row.label })}
            prefsId="Lançamentos nos dois períodos"
            columns={itemColumns({ derived, withPeriod: true, onEdit })}
            rows={detail}
            getRowId={(r) => r._id}
            onActivate={onOpenItem}
            defaultSort={derived ? null : { key: 'date', direction: 'asc' }}
            maxHeight="260px"
            minWidth="860px"
            dense
            empty={t('relatorios-nenhum-lancamento')}
          />
          <p className="text-[12px] text-dim">
            {derived ? t('relatorios-abre-analitico') : t('relatorios-editar-dica')}
          </p>
        </section>
      ) : null}
    </div>
  );
}

/** O analítico de uma linha do fluxo; `onClose` fecha. */
export function FlowItemsDialog({
  initial,
  scope,
  onClose,
}: {
  initial: FlowItemsTarget;
  scope: FlowItemsScope;
  onClose: () => void;
}) {
  const [stack, setStack] = useState<FlowItemsTarget[]>([initial]);
  const [editing, setEditing] = useState<string | null>(null);
  const target = stack[stack.length - 1]!;
  const query = useSankeyItems({
    ...scope.period,
    basis: scope.basis,
    depth: scope.depth,
    node_ids: target.nodeIds,
  });
  const data = query.data;
  const setView = (view: FlowItemsView) =>
    setStack((s) => [...s.slice(0, -1), { ...s[s.length - 1]!, view }]);
  const title = target.title ?? data?.name ?? '';
  const currentSentence = data ? periodLabel(data.current) : '';
  const previousSentence = data ? periodLabel(data.previous) : '';
  const currentLabel = capitalize(currentSentence);
  const previousLabel = capitalize(previousSentence);
  const openItem = (item: SankeyItem) => {
    if (item.component) setStack((s) => [...s, { nodeIds: [item.component!], view: target.view }]);
    else if (item.txn_id) setEditing(item.txn_id);
  };
  const back = stack.length > 1 ? stack[stack.length - 2]! : null;
  return (
    <>
      <Dialog
        open
        onOpenChange={(o) => {
          if (!o) onClose();
        }}
        index="≡"
        size="xl"
        title={title ? t('relatorios-analitico-de', { titulo: title }) : t('relatorios-analitico')}
        footer={
          back ? (
            <Button
              variant="secondary"
              icon={<Icon name="arrowLeft" size={13} />}
              onClick={() => setStack((s) => s.slice(0, -1))}
            >
              {t('comum-voltar')}
            </Button>
          ) : undefined
        }
      >
        {query.error ? (
          <QueryError error={query.error} onRetry={() => void query.refetch()} />
        ) : !data ? (
          <Loading label={t('relatorios-carregando-lancamentos')} />
        ) : (
          <div className="flex flex-col gap-4">
            <div
              role="tablist"
              aria-label={t('relatorios-periodo')}
              className="flex flex-wrap gap-2"
            >
              <PeriodTab
                id="previous"
                label={previousLabel}
                cents={data.previous.value}
                selected={target.view === 'previous'}
                onSelect={setView}
              />
              <PeriodTab
                id="current"
                label={currentLabel}
                cents={data.current.value}
                selected={target.view === 'current'}
                onSelect={setView}
              />
              <PeriodTab
                id="delta"
                label={t('relatorios-diferenca-delta')}
                cents={data.current.value - data.previous.value}
                selected={target.view === 'delta'}
                onSelect={setView}
                signed
              />
            </div>
            <div
              id="flow-items-panel"
              role="tabpanel"
              aria-labelledby={`flow-items-tab-${target.view}`}
            >
              {target.view === 'delta' ? (
                <DeltaView
                  data={data}
                  currentLabel={currentLabel}
                  previousLabel={previousLabel}
                  currentSentence={currentSentence}
                  previousSentence={previousSentence}
                  onOpenItem={openItem}
                  onEdit={setEditing}
                />
              ) : (
                <PeriodView
                  data={data}
                  view={target.view}
                  label={target.view === 'current' ? currentLabel : previousLabel}
                  sentenceLabel={target.view === 'current' ? currentSentence : previousSentence}
                  onOpenItem={openItem}
                  onEdit={setEditing}
                />
              )}
            </div>
          </div>
        )}
      </Dialog>
      {editing ? <EditTxn id={editing} onClose={() => setEditing(null)} /> : null}
    </>
  );
}
