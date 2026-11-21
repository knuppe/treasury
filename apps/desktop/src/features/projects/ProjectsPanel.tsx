/**
 * 05 · Orçamento › Projetos: obras, reformas, benfeitorias, festas e viagens com orçamento,
 * despesas e receitas (lançamentos ligados e itens lançados à mão), resultado, quanto resta,
 * a linha do tempo de receita × despesa, destaques e as entradas do projeto. Estado na URL
 * (junto de `?aba=projetos`): `projeto=<id>` escolhe o projeto, `novo=projeto` abre o
 * cadastro (com `bem=<id>` já escolhe o bem), `situacao=abertos | concluidos` filtra a lista
 * (padrão: todos, os concluídos no fim).
 */
import { useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router';

import type { ProjectView } from '@/api/generated/ProjectView';
import { useDeleteProject, useProjects } from '@/api/hooks/projects';
import { useUndoToast } from '@/api/hooks/undoToast';
import {
  Button,
  EmptyState,
  Icon,
  Money,
  TechTable,
  type ContextMenuEntry,
} from '@/design/components';
import { ConfirmDialog, QueryState, Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';

import {
  AddTxnsDialog,
  PROJECT_STATUS,
  ProjectFormDialog,
  ProjectItemDialog,
} from './ProjectDialogs';
import { ProjectOverview } from './ProjectOverview';

type Show = 'todos' | 'abertos' | 'concluidos';

const period = (p: ProjectView) =>
  p.start_date || p.end_date
    ? `${p.start_date ? formatDate(p.start_date) : '…'} – ${p.end_date ? formatDate(p.end_date) : t('projetos-hoje')}`
    : p.first_date
      ? `${formatDate(p.first_date)} – ${formatDate(p.last_date ?? p.first_date)}`
      : '—';

function ProjectDetail({
  project,
  onEdit,
  onAdd,
  onNewItem,
  onDelete,
}: {
  project: ProjectView;
  onEdit: () => void;
  onAdd: () => void;
  onNewItem: () => void;
  onDelete: () => void;
}) {
  const p = project;
  return (
    <section
      aria-label={t('projetos-projeto-nome', { nome: p.name })}
      className="flex flex-col gap-4"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-1">
          <h3 className="text-[17px] font-medium text-fg">{p.name}</h3>
          <p className="text-[13px] text-dim">
            {[PROJECT_STATUS[p.status], p.physical_asset_name, period(p)]
              .filter(Boolean)
              .join(' · ')}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="ghost" onClick={onEdit}>
            {t('comum-editar')}
          </Button>
          <Button variant="ghost" onClick={onDelete}>
            {t('projetos-excluir')}
          </Button>
          <Link
            to={txnLink({ project_ids: [p.id] })}
            className="inline-flex h-8 items-center rounded-control px-3 text-[13px] text-accent hover:bg-accent-soft"
          >
            {t('projetos-ver-lancamentos')}
          </Link>
          <Button variant="ghost" icon={<Icon name="plus" size={12} />} onClick={onAdd}>
            {t('projetos-lancamentos')}
          </Button>
          <Button variant="primary" icon={<Icon name="plus" size={12} />} onClick={onNewItem}>
            {t('projetos-novo-item')}
          </Button>
        </div>
      </div>
      <ProjectOverview project={p} onNewItem={onNewItem} onAdd={onAdd} />
    </section>
  );
}

export function ProjectsPanel() {
  const [params, setParams] = useSearchParams();
  const s = params.get('situacao');
  const show: Show = s === 'abertos' || s === 'concluidos' ? s : 'todos';
  // Sempre todos: o filtro é local, e um projeto concluído nunca some sem caminho de volta.
  const q = useProjects({ include_done: true });
  const remove = useDeleteProject();
  const undoToast = useUndoToast();
  const [editing, setEditing] = useState<ProjectView | null>(null);
  const [adding, setAdding] = useState<ProjectView | null>(null);
  const [newItem, setNewItem] = useState<ProjectView | null>(null);
  const [deleting, setDeleting] = useState<ProjectView | null>(null);
  const saved = useRef<Record<string, string | null>>({});
  const update = (patch: Record<string, string | null>) =>
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        for (const [k, v] of Object.entries(patch)) {
          if (v == null) next.delete(k);
          else next.set(k, v);
        }
        return next;
      },
      { replace: true },
    );
  const all = q.data ?? [];
  const done = all.filter((p) => p.status === 'done').length;
  const list = all.filter((p) =>
    show === 'todos' ? true : show === 'concluidos' ? p.status === 'done' : p.status !== 'done',
  );
  const selected = list.find((p) => p.id === params.get('projeto')) ?? list[0] ?? null;
  const creating = params.get('novo') === 'projeto';

  const menu = (p: ProjectView): ContextMenuEntry[] => [
    { heading: p.name },
    {
      label: t('projetos-abrir'),
      icon: 'arrowRight',
      onSelect: () => update({ projeto: p.id }),
    },
    { label: t('projetos-novo-item-menu'), icon: 'plus', onSelect: () => setNewItem(p) },
    { label: t('projetos-por-lancamentos-menu'), icon: 'plus', onSelect: () => setAdding(p) },
    { separator: true },
    { label: t('projetos-editar-menu'), icon: 'sliders', onSelect: () => setEditing(p) },
    {
      label: t('projetos-excluir-menu'),
      icon: 'x',
      danger: true,
      onSelect: () => setDeleting(p),
    },
  ];

  return (
    <div className="flex flex-col gap-5">
      <QueryState loading={q.isPending} error={q.error} onRetry={() => void q.refetch()}>
        {all.length === 0 ? (
          <EmptyState
            kicker={t('projetos-vazio-kicker')}
            title={t('projetos-vazio-titulo')}
            description={t('projetos-vazio-descricao')}
            actions={
              <Button variant="primary" onClick={() => update({ novo: 'projeto' })}>
                {t('projetos-novo')}
              </Button>
            }
          />
        ) : (
          <>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h3 className="label-caps text-fg">{t('projetos-titulo')}</h3>
              <div className="flex flex-wrap items-center gap-3">
                <Segmented<Show>
                  label={t('projetos-situacao-lista')}
                  size="sm"
                  value={show}
                  onChange={(v) => update({ situacao: v === 'todos' ? null : v, projeto: null })}
                  options={[
                    { value: 'todos', label: t('projetos-todos-n', { n: all.length }) },
                    { value: 'abertos', label: t('projetos-abertos-n', { n: all.length - done }) },
                    { value: 'concluidos', label: t('projetos-concluidos-n', { n: done }) },
                  ]}
                />
                <Button
                  variant="primary"
                  icon={<Icon name="plus" size={12} />}
                  onClick={() => update({ novo: 'projeto' })}
                >
                  {t('projetos-novo')}
                </Button>
              </div>
            </div>
            <TechTable<ProjectView>
              label={t('projetos-titulo')}
              prefsId="Projetos"
              rows={list}
              getRowId={(p) => p.id}
              selectedId={selected?.id ?? null}
              onSelectedChange={(id) => id && update({ projeto: id })}
              onActivate={(p) => update({ projeto: p.id })}
              rowMenu={menu}
              rowMenuLabel={(p) => t('projetos-acoes-de', { nome: p.name })}
              empty={
                show === 'abertos' ? t('projetos-nenhum-aberto') : t('projetos-nenhum-concluido')
              }
              columns={[
                {
                  key: 'name',
                  header: t('projetos-projeto'),
                  sortValue: (p) => p.name,
                  cell: (p) => (
                    <span className="flex flex-col">
                      <span className="font-medium">{p.name}</span>
                      <span className="text-[11.5px] text-dim">
                        {p.physical_asset_name ?? t('projetos-sem-bem')}
                      </span>
                    </span>
                  ),
                },
                {
                  key: 'status',
                  header: t('projetos-situacao'),
                  sortValue: (p) => PROJECT_STATUS[p.status],
                  cell: (p) => PROJECT_STATUS[p.status],
                },
                {
                  key: 'period',
                  header: t('projetos-periodo'),
                  sortValue: (p) => p.start_date ?? p.first_date,
                  cell: (p) => <span className="text-dim">{period(p)}</span>,
                },
                {
                  key: 'budget',
                  header: t('projetos-orcamento'),
                  numeric: true,
                  sortValue: (p) => p.budget,
                  cell: (p) => (p.budget != null ? <Money cents={p.budget} /> : '—'),
                },
                {
                  key: 'spent',
                  header: t('projetos-gasto'),
                  numeric: true,
                  sortValue: (p) => p.spent,
                  cell: (p) => <Money cents={p.spent} />,
                },
                {
                  key: 'income',
                  header: t('projetos-receitas'),
                  numeric: true,
                  sortValue: (p) => p.income,
                  cell: (p) => (p.income ? <Money cents={p.income} /> : '—'),
                },
                {
                  key: 'remaining',
                  header: t('projetos-resta'),
                  numeric: true,
                  sortValue: (p) => p.remaining,
                  cell: (p) =>
                    p.remaining != null ? <Money cents={p.remaining} tone="auto" /> : '—',
                },
                {
                  key: 'txn_count',
                  header: t('projetos-entradas'),
                  numeric: true,
                  sortValue: (p) => p.txn_count + p.item_count,
                  cell: (p) => p.txn_count + p.item_count,
                },
              ]}
            />
            {selected ? (
              <ProjectDetail
                key={selected.id}
                project={selected}
                onEdit={() => setEditing(selected)}
                onAdd={() => setAdding(selected)}
                onNewItem={() => setNewItem(selected)}
                onDelete={() => setDeleting(selected)}
              />
            ) : null}
          </>
        )}
      </QueryState>
      <ProjectFormDialog
        open={creating || editing != null}
        onOpenChange={(o) => {
          if (o) return;
          setEditing(null);
          // Uma escrita só na URL (duas seguidas se sobrescrevem): o fechar leva o que o
          // salvar pediu.
          update({ novo: null, bem: null, ...saved.current });
          saved.current = {};
        }}
        project={editing}
        assetId={params.get('bem')}
        onSaved={(p) => {
          // O projeto salvo continua à vista: se saiu do filtro (ex.: concluído), mostra todos.
          const fits = show === 'todos' || (show === 'concluidos') === (p.status === 'done');
          saved.current = { projeto: p.id, ...(fits ? {} : { situacao: null }) };
        }}
      />
      {newItem ? (
        <ProjectItemDialog
          project={newItem}
          open
          onOpenChange={(o) => (!o ? setNewItem(null) : undefined)}
        />
      ) : null}
      {adding ? (
        <AddTxnsDialog
          project={adding}
          open
          onOpenChange={(o) => (!o ? setAdding(null) : undefined)}
        />
      ) : null}
      <ConfirmDialog
        open={deleting != null}
        onOpenChange={(o) => (!o ? setDeleting(null) : undefined)}
        title={t('projetos-excluir-titulo', { nome: deleting?.name ?? t('projetos-o-projeto') })}
        description={t('projetos-excluir-descricao')}
        confirmLabel={t('projetos-excluir-projeto')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { id: deleting.id },
            {
              onSuccess: () => {
                undoToast({ title: t('projetos-excluido'), description: deleting.name });
                setDeleting(null);
                update({ projeto: null });
              },
            },
          )
        }
      />
    </div>
  );
}
