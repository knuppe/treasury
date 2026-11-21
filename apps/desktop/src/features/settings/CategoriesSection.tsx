import { useState, type ReactNode } from 'react';

import { useCountryFeatures } from '@/api/hooks/country';
import { ProductCategoriesPanel } from '@/features/food/ProductCategoriesPanel';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { CategoryNode } from '@/api/generated/CategoryNode';
import type { MissingDefault } from '@/api/generated/MissingDefault';
import type { SpendGroup } from '@/api/generated/SpendGroup';
import {
  useCategoryTree,
  useCategoryUsage,
  useCreateCategory,
  useDeleteCategory,
  useAddDefaultCategories,
  useMergeCategories,
  useMissingDefaultCategories,
  useSetCategoryGroup,
  useUpdateCategory,
} from '@/api/hooks/categories';
import { useUndoToast } from '@/api/hooks/undoToast';
import {
  Button,
  Dialog,
  Field,
  Icon,
  IconButton,
  InstrumentCard,
  Input,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import {
  Checkbox,
  InlineError,
  QueryState,
  Segmented,
  Switch,
} from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { groupName, groupOptions, SPEND_GROUPS } from '@/features/reports/model/groups';
import { useGroupColors } from '@/features/reports/useEntityColors';
import { cssColor } from '@/charts/colorMath';
import { t } from '@/i18n';

const ROOT = '__root__';
const ALL_GROUPS = 'all';

type GroupFilter = SpendGroup | typeof ALL_GROUPS;

/** Seletor compacto do grupo de gasto (na linha da categoria e na seleção em lote). */
function GroupSelect({
  value,
  onChange,
  label,
  placeholder,
  disabled,
  className,
}: {
  value: SpendGroup | undefined;
  onChange: (g: SpendGroup) => void;
  label: string;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
}) {
  const colors = useGroupColors();
  return (
    <div
      className={cx(
        'shrink-0 [&>button]:h-7 [&>button]:gap-1.5 [&>button]:px-2 [&>button]:text-[12px]',
        className ?? 'w-[136px]',
      )}
    >
      <Select
        aria-label={label}
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        onValueChange={(v) => onChange(v as SpendGroup)}
        options={groupOptions().map((o) => ({
          ...o,
          label: (
            <span className="inline-flex items-center gap-1.5">
              <span
                aria-hidden
                className="size-2 shrink-0 rounded-[2px]"
                style={{ background: cssColor(colors.get(o.value) ?? 'var(--line-strong)') }}
              />
              {o.label}
            </span>
          ),
        }))}
      />
    </div>
  );
}

/** A categoria (ou alguma subcategoria dela) está no grupo filtrado? */
function matchesGroup(node: CategoryNode, filter: GroupFilter): boolean {
  if (filter === ALL_GROUPS) return true;
  return node.spend_group === filter || node.children.some((c) => matchesGroup(c, filter));
}

/** Ids da categoria e das subcategorias visíveis (as do filtro). */
function selectableIds(node: CategoryNode, filter: GroupFilter): string[] {
  return [
    ...(filter === ALL_GROUPS || node.spend_group === filter ? [node.id] : []),
    ...node.children.flatMap((c) => selectableIds(c, filter)),
  ];
}

type Action =
  | { kind: 'create'; parent: CategoryNode | null }
  | { kind: 'rename'; node: CategoryNode }
  | { kind: 'move'; node: CategoryNode }
  | { kind: 'merge'; node: CategoryNode }
  | { kind: 'delete'; node: CategoryNode };

function flatRoots(nodes: CategoryNode[]): CategoryNode[] {
  return nodes;
}

function UsageLine({ id }: { id: string }) {
  const { data } = useCategoryUsage(id);
  if (!data) return null;
  const parts = [
    data.transactions && t('ajustes-cat-uso-lancamentos', { n: data.transactions }),
    data.splits && t('ajustes-cat-uso-divisoes', { n: data.splits }),
    data.budget_lines && t('ajustes-cat-uso-orcamento', { n: data.budget_lines }),
    data.rules && t('ajustes-cat-uso-regras', { n: data.rules }),
    data.recurrences && t('ajustes-cat-uso-recorrencias', { n: data.recurrences }),
    data.installment_plans && t('ajustes-cat-uso-parcelamentos', { n: data.installment_plans }),
    data.children && t('ajustes-cat-uso-subcategorias', { n: data.children }),
  ].filter(Boolean);
  return (
    <p className="text-[12.5px] text-dim">
      {parts.length
        ? t('ajustes-cat-em-uso', { usos: parts.join(', ') })
        : t('ajustes-cat-sem-uso')}
    </p>
  );
}

function ActionDialog({
  action,
  kind,
  roots,
  onClose,
}: {
  action: Action | null;
  kind: CategoryKind;
  roots: CategoryNode[];
  onClose: () => void;
}) {
  const create = useCreateCategory();
  const update = useUpdateCategory();
  const merge = useMergeCategories();
  const remove = useDeleteCategory();
  const [name, setName] = useState('');
  const [parent, setParent] = useState<string>(ROOT);
  const [target, setTarget] = useState<string | null>(null);
  const [group, setGroup] = useState<SpendGroup>('non_essential');
  const [groupTouched, setGroupTouched] = useState(false);
  const [last, setLast] = useState<Action | null>(null);
  if (action !== last) {
    setLast(action);
    setName(action?.kind === 'rename' ? action.node.name : '');
    setParent(
      action?.kind === 'move'
        ? (action.node.parent_id ?? ROOT)
        : action?.kind === 'create' && action.parent
          ? action.parent.id
          : ROOT,
    );
    setTarget(null);
    setGroup(
      action?.kind === 'create' ? (action.parent?.spend_group ?? 'non_essential') : 'non_essential',
    );
    setGroupTouched(false);
    for (const m of [create, update, merge, remove]) m.reset();
  }
  if (!action) return null;
  const node = action.kind === 'create' ? null : action.node;
  const pending = create.isPending || update.isPending || merge.isPending || remove.isPending;
  const error = create.error ?? update.error ?? merge.error ?? remove.error;
  const done = (title: string) => () => {
    toast({ tone: 'success', title });
    onClose();
  };
  const parentOptions = [
    { value: ROOT, label: t('ajustes-cat-principal') },
    ...flatRoots(roots)
      .filter((r) => r.id !== node?.id)
      .map((r) => ({ value: r.id, label: r.name })),
  ];
  const titles = {
    create:
      action.kind === 'create' && action.parent
        ? t('ajustes-cat-nova-sub-de', { nome: action.parent.name })
        : t('ajustes-cat-nova'),
    rename: t('ajustes-cat-renomear-nome', { nome: node?.name ?? '' }),
    move: t('ajustes-cat-mover-nome', { nome: node?.name ?? '' }),
    merge: t('ajustes-cat-juntar-titulo', { nome: node?.name ?? '' }),
    delete: t('ajustes-cat-apagar-nome', { nome: node?.name ?? '' }),
  } as const;
  const submit = () => {
    if (action.kind === 'create') {
      create.mutate(
        {
          name,
          kind,
          ...(parent !== ROOT ? { parent_id: parent } : {}),
          ...(kind === 'expense' ? { spend_group: group } : {}),
        },
        { onSuccess: done(t('ajustes-cat-criada')) },
      );
    } else if (action.kind === 'rename')
      update.mutate(
        { id: action.node.id, name },
        {
          onSuccess: done(t('ajustes-cat-renomeada')),
        },
      );
    else if (action.kind === 'move')
      update.mutate(
        { id: action.node.id, parent_id: parent === ROOT ? null : parent },
        { onSuccess: done(t('ajustes-cat-movida')) },
      );
    else if (action.kind === 'merge' && target)
      merge.mutate(
        { from: action.node.id, into: target },
        { onSuccess: done(t('ajustes-cat-juntadas')) },
      );
    else if (action.kind === 'delete') {
      remove.mutate(
        { id: action.node.id, ...(target ? { reassign_to: target } : {}) },
        {
          onSuccess: (moved) => (
            toast({
              tone: 'success',
              title: t('ajustes-cat-apagada'),
              description: target
                ? t('ajustes-cat-reatribuidos', { n: moved.transactions })
                : undefined,
            }),
            onClose()
          ),
        },
      );
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      index="11"
      size="sm"
      title={titles[action.kind]}
      description={
        action.kind === 'merge'
          ? t('ajustes-cat-juntar-texto')
          : action.kind === 'delete'
            ? t('ajustes-cat-apagar-texto')
            : undefined
      }
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant={action.kind === 'delete' ? 'danger' : 'primary'}
            loading={pending}
            disabled={
              ((action.kind === 'create' || action.kind === 'rename') && !name.trim()) ||
              (action.kind === 'merge' && !target)
            }
            onClick={submit}
          >
            {action.kind === 'delete'
              ? t('comum-apagar')
              : action.kind === 'merge'
                ? t('ajustes-cat-juntar')
                : t('comum-salvar')}
          </Button>
        </>
      }
    >
      {action.kind === 'create' || action.kind === 'rename' ? (
        <Field label={t('ajustes-nome')}>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
            onKeyDown={(e) => e.key === 'Enter' && name.trim() && submit()}
          />
        </Field>
      ) : null}
      {action.kind === 'create' || action.kind === 'move' ? (
        <Field label={t('ajustes-cat-dentro-de')}>
          <Select
            value={parent}
            onValueChange={(p) => {
              setParent(p);
              // Sem escolha própria, o grupo acompanha o da categoria-pai.
              if (!groupTouched)
                setGroup(roots.find((r) => r.id === p)?.spend_group ?? 'non_essential');
            }}
            options={parentOptions}
          />
        </Field>
      ) : null}
      {action.kind === 'create' && kind === 'expense' ? (
        <Field label={t('ajustes-cat-grupo')} hint={t('ajustes-cat-grupo-campo-dica')}>
          <Select
            value={group}
            onValueChange={(g) => {
              setGroup(g as SpendGroup);
              setGroupTouched(true);
            }}
            options={groupOptions()}
          />
        </Field>
      ) : null}
      {action.kind === 'merge' || action.kind === 'delete' ? (
        <>
          {node ? <UsageLine id={node.id} /> : null}
          <Field
            label={
              action.kind === 'merge' ? t('ajustes-cat-juntar-em') : t('ajustes-cat-reatribuir')
            }
          >
            <CategoryPicker
              kind={kind}
              value={target}
              onChange={setTarget}
              noneLabel={
                action.kind === 'merge' ? t('ajustes-cat-escolha') : t('ajustes-cat-nao-reatribuir')
              }
              className="[&>button]:h-9 [&>button]:w-full"
            />
          </Field>
        </>
      ) : null}
      <InlineError error={error} />
    </Dialog>
  );
}

interface TreeOptions {
  kind: CategoryKind;
  filter: GroupFilter;
  selected: ReadonlySet<string>;
  onSelect: (ids: readonly string[], on: boolean) => void;
  onGroup: (node: CategoryNode, group: SpendGroup) => void;
  onAction: (a: Action) => void;
}

function NodeRow({
  node,
  depth,
  options,
}: {
  node: CategoryNode;
  depth: number;
  options: TreeOptions;
}) {
  const { kind, filter, selected, onSelect, onGroup, onAction } = options;
  const update = useUpdateCategory();
  const [open, setOpen] = useState(depth > 0 || node.children.length <= 8);
  const toggle = (patch: { archived?: boolean }) =>
    update.mutate(
      { id: node.id, ...patch },
      {
        onError: (e) =>
          toast({ tone: 'error', title: t('ajustes-nao-alterou'), description: errorText(e) }),
      },
    );
  const expense = kind === 'expense';
  const children = node.children.filter((c) => matchesGroup(c, filter));
  // No filtro, a categoria-pai que só aparece por causa de uma subcategoria fica em segundo plano.
  const context = filter !== ALL_GROUPS && node.spend_group !== filter;
  const ids = selectableIds(node, filter);
  const picked = ids.filter((id) => selected.has(id)).length;
  return (
    <li
      role="treeitem"
      aria-expanded={children.length ? open : undefined}
      aria-level={depth + 1}
      aria-selected={expense ? selected.has(node.id) : false}
    >
      <div
        className={cx(
          'group grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 rounded-control py-1 pr-1 hover:bg-surface-2',
          node.archived && 'opacity-60',
        )}
        style={{ paddingLeft: 6 + depth * 20 }}
      >
        <span className="flex min-w-0 items-center gap-1.5">
          {expense && ids.length ? (
            <Checkbox
              checked={picked > 0 && picked === ids.length}
              indeterminate={picked > 0 && picked < ids.length}
              onChange={(on) => onSelect(ids, on)}
              label={t('ajustes-cat-selecionar', { nome: node.name })}
              hideLabel
            />
          ) : null}
          {children.length ? (
            <button
              type="button"
              aria-label={
                open
                  ? t('ajustes-cat-recolher', { nome: node.name })
                  : t('ajustes-cat-expandir', { nome: node.name })
              }
              onClick={() => setOpen(!open)}
              className="inline-flex size-5 items-center justify-center rounded-control text-dim hover:text-fg"
            >
              <Icon name={open ? 'chevronDown' : 'chevronRight'} size={12} />
            </button>
          ) : (
            <span className="inline-block size-5" />
          )}
          <span
            className={cx(
              'truncate text-[13px]',
              depth === 0 ? 'font-medium' : '',
              context ? 'text-dim' : 'text-fg',
            )}
          >
            {node.name}
          </span>
          {node.archived ? <span className="label-caps">{t('ajustes-cat-arquivada')}</span> : null}
        </span>
        <span className="flex items-center gap-2">
          {expense && node.spend_group ? (
            <GroupSelect
              value={node.spend_group}
              onChange={(g) => g !== node.spend_group && onGroup(node, g)}
              label={t('ajustes-cat-grupo-de', { nome: node.name })}
            />
          ) : null}
          <span className="flex items-center gap-0.5 opacity-80 group-focus-within:opacity-100 group-hover:opacity-100">
            {depth === 0 ? (
              <IconButton
                size="sm"
                label={t('ajustes-cat-nova-sub-em', { nome: node.name })}
                icon={<Icon name="plus" size={12} />}
                onClick={() => onAction({ kind: 'create', parent: node })}
              />
            ) : (
              // Mesmo espaço do "+" das principais, para os seletores de grupo alinharem.
              <span aria-hidden className="inline-block size-7" />
            )}
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onAction({ kind: 'rename', node })}
              aria-label={t('ajustes-cat-renomear-nome', { nome: node.name })}
            >
              {t('ajustes-cat-renomear')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onAction({ kind: 'move', node })}
              aria-label={t('ajustes-cat-mover-nome', { nome: node.name })}
            >
              {t('ajustes-cat-mover')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onAction({ kind: 'merge', node })}
              aria-label={t('ajustes-cat-juntar-nome', { nome: node.name })}
            >
              {t('ajustes-cat-juntar')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => toggle({ archived: !node.archived })}
              aria-label={
                node.archived
                  ? t('ajustes-cat-reativar-nome', { nome: node.name })
                  : t('ajustes-cat-arquivar-nome', { nome: node.name })
              }
            >
              {node.archived ? t('ajustes-reativar') : t('ajustes-arquivar')}
            </Button>
            <IconButton
              size="sm"
              label={t('ajustes-cat-apagar-nome', { nome: node.name })}
              icon={<Icon name="x" size={12} />}
              onClick={() => onAction({ kind: 'delete', node })}
            />
          </span>
        </span>
      </div>
      {open && children.length ? (
        <ul role="group">
          {children.map((c) => (
            <NodeRow key={c.id} node={c} depth={depth + 1} options={options} />
          ))}
        </ul>
      ) : null}
    </li>
  );
}

const suggestionKey = (m: MissingDefault) => `${m.kind}:${m.path}`;

/** Oferece as categorias do conjunto inicial que faltam, sem mudar as existentes. */
function SuggestedCategories() {
  const { data = [] } = useMissingDefaultCategories();
  const add = useAddDefaultCategories();
  const [open, setOpen] = useState(false);
  // As desmarcadas (as sugestões novas vêm marcadas).
  const [skipped, setSkipped] = useState<ReadonlySet<string>>(new Set());
  if (!data.length) return null;
  const picked = data.filter((m) => !skipped.has(suggestionKey(m)));
  const incomes = data.filter((m) => m.kind === 'income').length;
  const toggle = (m: MissingDefault, on: boolean) => {
    const next = new Set(skipped);
    const group = m.path.split(' › ')[0]!;
    const isGroup = !m.parent_exists && !m.path.includes(' › ');
    // Grupo novo leva as subcategorias junto; subcategoria marcada traz o grupo novo.
    const affected = data.filter(
      (o) =>
        o.kind === m.kind &&
        (o.path === m.path ||
          (isGroup && o.path.startsWith(`${group} › `)) ||
          (on && !m.parent_exists && o.path === group)),
    );
    for (const o of affected) {
      if (on) next.delete(suggestionKey(o));
      else next.add(suggestionKey(o));
    }
    setSkipped(next);
  };
  const all = picked.length === data.length;
  return (
    <div
      role="status"
      className="mb-3 flex flex-col gap-2 rounded-card border border-accent/40 bg-accent-soft/40 p-3"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-[13px] text-fg">
          {incomes
            ? t('ajustes-cat-sugeridas-receita', { n: data.length, receitas: incomes })
            : t('ajustes-cat-sugeridas-despesa', { n: data.length })}
        </p>
        <div className="flex gap-2">
          <Button size="sm" variant="ghost" onClick={() => setOpen((o) => !o)}>
            {open ? t('ajustes-cat-esconder-lista') : t('ajustes-cat-ver-lista')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            loading={add.isPending}
            disabled={!picked.length}
            onClick={() =>
              add.mutate(all ? {} : { paths: picked.map((m) => m.path) }, {
                onSuccess: (n) => {
                  setSkipped(new Set());
                  toast({
                    tone: 'success',
                    title: t('ajustes-cat-adicionadas', { n }),
                    description: t('ajustes-cat-desfazer-dica'),
                  });
                },
              })
            }
          >
            {all
              ? t('ajustes-cat-adicionar-sugeridas')
              : t('ajustes-cat-adicionar-selecionadas', { n: picked.length })}
          </Button>
        </div>
      </div>
      {open ? (
        <ul aria-label={t('ajustes-cat-sugeridas')} className="columns-2 text-[12.5px] text-dim">
          {data.map((m) => (
            <li key={suggestionKey(m)} className="break-inside-avoid">
              <Checkbox
                checked={!skipped.has(suggestionKey(m))}
                onChange={(on) => toggle(m, on)}
                label={
                  m.kind === 'income' ? t('ajustes-cat-receita-item', { caminho: m.path }) : m.path
                }
              />
            </li>
          ))}
        </ul>
      ) : null}
      <InlineError error={add.error} />
    </div>
  );
}

/**
 * 12.16 · Categorias: árvore editável (criar, renomear, mover, grupo de gasto, arquivar, juntar,
 * apagar), com filtro por grupo e mudança de grupo em lote.
 */
function TxnCategories({ header }: { header?: ReactNode }) {
  const [kind, setKind] = useState<CategoryKind>('expense');
  const [archived, setArchived] = useState(false);
  const [filter, setFilter] = useState<GroupFilter>(ALL_GROUPS);
  const [selected, setSelected] = useState<ReadonlySet<string>>(new Set());
  const {
    data = [],
    isLoading,
    error,
    refetch,
  } = useCategoryTree({ kind, include_archived: archived });
  const [action, setAction] = useState<Action | null>(null);
  const setGroup = useSetCategoryGroup();
  const undoToast = useUndoToast();
  const expense = kind === 'expense';
  const visible = expense ? data.filter((n) => matchesGroup(n, filter)) : data;

  const select = (ids: readonly string[], on: boolean) =>
    setSelected((cur) => {
      const next = new Set(cur);
      for (const id of ids) {
        if (on) next.add(id);
        else next.delete(id);
      }
      return next;
    });

  const apply = (
    ids: string[],
    group: SpendGroup,
    title: (updated: number) => string,
    onDone?: () => void,
  ) =>
    setGroup.mutate(
      { ids, group },
      {
        onSuccess: ({ updated }) => {
          onDone?.();
          if (updated > 0) undoToast({ title: title(updated) });
          else toast({ title: t('ajustes-cat-grupo-lote', { n: 0, grupo: groupName(group) }) });
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('ajustes-nao-alterou'), description: errorText(e) }),
      },
    );

  /** Na linha: a categoria e as subcategorias que seguiam o grupo dela (as exceções ficam). */
  const changeRow = (node: CategoryNode, group: SpendGroup) => {
    const followers = node.children.filter((c) => c.spend_group === node.spend_group);
    apply([node.id, ...followers.map((c) => c.id)], group, () =>
      followers.length
        ? t('ajustes-cat-grupo-alterado-subs', {
            nome: node.name,
            n: followers.length,
            grupo: groupName(group),
          })
        : t('ajustes-cat-grupo-alterado', { nome: node.name, grupo: groupName(group) }),
    );
  };

  const changeSelected = (group: SpendGroup) =>
    apply(
      [...selected],
      group,
      (updated) => t('ajustes-cat-grupo-lote', { n: updated, grupo: groupName(group) }),
      () => setSelected(new Set()),
    );

  const options: TreeOptions = {
    kind,
    filter: expense ? filter : ALL_GROUPS,
    selected,
    onSelect: select,
    onGroup: changeRow,
    onAction: setAction,
  };

  return (
    <InstrumentCard
      index="13.16"
      title={t('ajustes-secao-categorias')}
      actions={
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setAction({ kind: 'create', parent: null })}
        >
          {t('ajustes-cat-nova')}
        </Button>
      }
      footer={[
        { label: t('ajustes-cat-principais'), value: data.length },
        {
          label: t('ajustes-cat-subcategorias'),
          value: data.reduce((n, c) => n + c.children.length, 0),
        },
        {
          label: t('ajustes-cat-grupos-rodape'),
          value: t('ajustes-cat-grupos-rodape-valor'),
        },
      ]}
    >
      {header}
      <div className="mb-3 flex flex-wrap items-center gap-4">
        <Segmented<CategoryKind>
          label={t('ajustes-cat-tipo')}
          value={kind}
          onChange={(k) => {
            setKind(k);
            setSelected(new Set());
          }}
          options={[
            { value: 'expense', label: t('ajustes-cat-despesas') },
            { value: 'income', label: t('ajustes-cat-receitas') },
          ]}
        />
        {expense ? (
          <Segmented<GroupFilter>
            label={t('ajustes-cat-filtro-grupo')}
            size="sm"
            value={filter}
            onChange={setFilter}
            options={[
              { value: ALL_GROUPS, label: t('ajustes-cat-todos-grupos') },
              ...SPEND_GROUPS.map((g) => ({ value: g, label: groupName(g) })),
            ]}
          />
        ) : null}
        <div className="w-fit">
          <Switch
            checked={archived}
            onChange={setArchived}
            label={t('ajustes-cat-mostrar-arquivadas')}
          />
        </div>
      </div>
      {expense ? (
        <p className="mb-3 text-[12.5px] text-dim">{t('ajustes-cat-grupos-explicacao')}</p>
      ) : null}
      <SuggestedCategories />
      {expense && selected.size > 0 ? (
        <div
          role="region"
          aria-label={t('ajustes-cat-selecao-aria')}
          className="sticky top-2 z-10 mb-2 flex flex-wrap items-center gap-3 rounded-card border border-accent/50 bg-surface-2 px-3 py-2 shadow-[var(--shadow-pop)]"
        >
          <span className="text-[13px] text-fg">
            {t('ajustes-cat-selecionadas', { n: selected.size })}
          </span>
          <GroupSelect
            value={undefined}
            placeholder={t('ajustes-cat-mudar-grupo')}
            label={t('ajustes-cat-mudar-grupo-aria')}
            onChange={changeSelected}
            disabled={setGroup.isPending}
            className="w-[160px]"
          />
          <Button size="sm" variant="ghost" onClick={() => setSelected(new Set())}>
            {t('ajustes-cat-limpar-selecao')}
          </Button>
        </div>
      ) : null}
      <QueryState loading={isLoading} error={error} onRetry={() => void refetch()}>
        {expense && filter !== ALL_GROUPS && visible.length === 0 ? (
          <p className="py-2 text-[13px] text-dim">
            {t('ajustes-cat-filtro-vazio', { grupo: groupName(filter) })}
          </p>
        ) : null}
        <ul
          role="tree"
          aria-label={expense ? t('ajustes-cat-arvore-despesa') : t('ajustes-cat-arvore-receita')}
          aria-multiselectable={expense || undefined}
          className="flex flex-col"
        >
          {visible.map((n) => (
            <NodeRow key={n.id} node={n} depth={0} options={options} />
          ))}
        </ul>
      </QueryState>
      <ActionDialog action={action} kind={kind} roots={data} onClose={() => setAction(null)} />
    </InstrumentCard>
  );
}

/**
 * Categorias de lançamento e, no Brasil, as categorias de produto da tela 06 · Alimentação
 * (aba "Produtos", docs/consumption.md).
 */
export function CategoriesSection() {
  const features = useCountryFeatures();
  const [view, setView] = useState<'txn' | 'products'>('txn');
  const header = features.receipts ? (
    <div className="mb-3">
      <Segmented<'txn' | 'products'>
        label={t('ajustes-cat-o-que')}
        value={view}
        onChange={setView}
        options={[
          { value: 'txn', label: t('ajustes-cat-lancamentos') },
          { value: 'products', label: t('ajustes-cat-produtos') },
        ]}
      />
    </div>
  ) : null;
  if (view === 'products' && features.receipts) {
    return (
      <InstrumentCard index="13.16" title={t('ajustes-secao-categorias')}>
        {header}
        <ProductCategoriesPanel />
      </InstrumentCard>
    );
  }
  return <TxnCategories header={header} />;
}
