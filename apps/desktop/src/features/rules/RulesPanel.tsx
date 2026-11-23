import { useState } from 'react';

import type { Rule } from '@/api/generated/Rule';
import type { RuleApplyResult } from '@/api/generated/RuleApplyResult';
import type { RuleInput } from '@/api/generated/RuleInput';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import { useUndo } from '@/api/hooks/changes';
import { useCategories } from '@/api/hooks/categories';
import { useApplyRules, useDeleteRule, useRules, useUpdateRule } from '@/api/hooks/rules';
import {
  Button,
  Dialog,
  EmptyState,
  Icon,
  IconButton,
  InstrumentCard,
  Money,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { formatDate, formatDateTime } from '@/lib/format';
import {
  ConfirmDialog,
  InlineError,
  QueryState,
  Switch,
} from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { TXN_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

import { plainSummary } from './plainSummary';
import { conditionsSummary, RuleDialog } from './RuleDialog';

/** Simula e aplica as regras ativas aos lançamentos existentes, com desfazer. */
export function ApplyRulesDialog({
  open,
  onOpenChange,
  filter,
  ruleIds,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Filtro atual da tabela (opcional: limita o alcance). */
  filter?: TxnFilter;
  ruleIds?: string[];
}) {
  const apply = useApplyRules();
  const undo = useUndo();
  const [onlyUncategorized, setOnlyUncategorized] = useState(true);
  const [useFilter, setUseFilter] = useState(false);
  const [preview, setPreview] = useState<RuleApplyResult | null>(null);
  const [done, setDone] = useState<RuleApplyResult | null>(null);
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    setPreview(null);
    setDone(null);
    apply.reset();
  }
  const args = {
    only_uncategorized: onlyUncategorized,
    ...(useFilter && filter ? { filter: { ...filter, limit: undefined, offset: undefined } } : {}),
    ...(ruleIds ? { rule_ids: ruleIds } : {}),
  };
  const simulate = () => apply.mutate({ ...args, dry_run: true }, { onSuccess: setPreview });
  const run = () =>
    apply.mutate(args, {
      onSuccess: (res) => {
        setDone(res);
        toast({ tone: 'success', title: t('regras-n-alterados', { n: res.updated }) });
      },
    });
  const result = done ?? preview;

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="03"
      size="lg"
      title={t('regras-aplicar')}
      description={t('regras-aplicar-descricao')}
      footer={
        done ? (
          <>
            {done.change_set_id ? (
              <Button
                variant="ghost"
                loading={undo.isPending}
                onClick={() =>
                  undo.mutate(
                    { id: done.change_set_id! },
                    {
                      onSuccess: () => (
                        toast({ tone: 'success', title: t('regras-aplicacao-desfeita') }),
                        onOpenChange(false)
                      ),
                    },
                  )
                }
              >
                {t('regras-desfazer')}
              </Button>
            ) : null}
            <Button variant="primary" onClick={() => onOpenChange(false)}>
              {t('comum-fechar')}
            </Button>
          </>
        ) : (
          <>
            <Button variant="ghost" onClick={() => onOpenChange(false)}>
              {t('comum-cancelar')}
            </Button>
            <Button variant="secondary" loading={apply.isPending && !preview} onClick={simulate}>
              {t('regras-ver-previa')}
            </Button>
            <Button
              variant="primary"
              disabled={!preview || preview.updated === 0}
              loading={apply.isPending && Boolean(preview)}
              onClick={run}
            >
              {t('regras-aplicar-a', { n: preview?.updated ?? 0 })}
            </Button>
          </>
        )
      }
    >
      {!done ? (
        <div className="flex flex-col gap-3">
          <Switch
            checked={onlyUncategorized}
            onChange={(v) => (setOnlyUncategorized(v), setPreview(null))}
            label={t('regras-so-sem-categoria')}
            description={t('regras-so-sem-categoria-descricao')}
          />
          {filter ? (
            <Switch
              checked={useFilter}
              onChange={(v) => (setUseFilter(v), setPreview(null))}
              label={t('regras-so-filtro')}
            />
          ) : null}
        </div>
      ) : null}
      {result ? (
        <section aria-label={t('regras-afetados')} className="rounded-card border border-line">
          <p className="label-caps border-b border-line px-3 py-2" role="status">
            {done
              ? t('regras-resultado-aplicado', { casaram: result.matched, mudam: result.updated })
              : t('regras-resultado-previa', { casaram: result.matched, mudam: result.updated })}
          </p>
          <ul className="max-h-64 overflow-auto">
            {result.sample.map((txn) => (
              <li
                key={txn.id}
                className="grid grid-cols-[88px_1fr_minmax(0,1fr)_120px] items-center gap-3 border-b border-line/60 px-3 py-1.5 text-[12.5px]"
              >
                <span className="num text-dim">{formatDate(txn.date)}</span>
                <span className="truncate">{txn.description}</span>
                <span className="truncate text-accent">
                  {txn.category_path ?? TXN_KIND[txn.kind]}
                </span>
                <Money
                  cents={txn.amount}
                  currency={txn.currency}
                  tone="auto"
                  className="text-right"
                />
              </li>
            ))}
            {result.sample.length === 0 ? (
              <li className="px-3 py-3 text-[12.5px] text-dim">{t('regras-nada-mudar')}</li>
            ) : null}
          </ul>
        </section>
      ) : null}
      <InlineError error={apply.error ?? undo.error} />
    </Dialog>
  );
}

/** Aba "Regras": lista por prioridade, ativar/desativar, editar, testar, apagar e aplicar. */
export function RulesPanel({
  onNew,
  draft,
  onDraftConsumed,
  filter,
}: {
  onNew: () => void;
  draft: RuleInput | null;
  onDraftConsumed: () => void;
  filter: TxnFilter;
}) {
  const { data = [], isLoading, error, refetch } = useRules();
  const { data: categories = [] } = useCategories({ include_archived: true });
  const update = useUpdateRule();
  const remove = useDeleteRule();
  const [editing, setEditing] = useState<Rule | null>(null);
  const [deleting, setDeleting] = useState<Rule | null>(null);
  const [applyOpen, setApplyOpen] = useState(false);
  const pathOf = (id?: string) => categories.find((c) => c.id === id)?.path;

  const columns: TechColumn<Rule>[] = [
    {
      key: 'priority',
      header: t('regras-col-prioridade'),
      numeric: true,
      width: '64px',
      sortValue: (r) => r.priority,
      cell: (r) => r.priority,
    },
    {
      key: 'name',
      header: t('regras-col-regra'),
      sortValue: (r) => r.name,
      cell: (r) => (
        <div className="flex min-w-0 flex-col py-1">
          <span className={r.active ? 'break-words' : 'break-words text-dim line-through'}>
            {r.name}
          </span>
          {/* Em palavras, com quebra de linha; a condição exata fica no title. */}
          <span className="break-words text-[11.5px] text-dim" title={conditionsSummary(r)}>
            {plainSummary(r)}
          </span>
        </div>
      ),
    },
    {
      key: 'actions_summary',
      header: t('regras-entao'),
      cell: (r) => (
        <span className="text-[12.5px]">
          {[
            r.actions.category_id
              ? (pathOf(r.actions.category_id) ?? t('regras-acao-categoria'))
              : null,
            r.actions.kind
              ? t('regras-acao-tipo', { tipo: TXN_KIND[r.actions.kind].toLowerCase() })
              : null,
            r.actions.rename_to ? t('regras-acao-renomear', { nome: r.actions.rename_to }) : null,
            r.actions.tags?.length ? r.actions.tags.map((tag) => `#${tag}`).join(' ') : null,
            r.actions.member_id ? t('regras-acao-membro') : null,
          ]
            .filter(Boolean)
            .join(' · ')}
        </span>
      ),
    },
    {
      key: 'hits',
      header: t('regras-col-acertos'),
      numeric: true,
      width: '110px',
      sortValue: (r) => r.hits,
      cell: (r) => (
        <span
          title={
            r.last_hit_at ? t('regras-ultimo', { data: formatDateTime(r.last_hit_at) }) : undefined
          }
        >
          {r.hits}
        </span>
      ),
    },
    {
      key: 'buttons',
      header: <span className="sr-only">{t('regras-acoes')}</span>,
      width: '220px',
      align: 'right',
      cell: (r) => (
        <span className="inline-flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
          <Button
            size="sm"
            variant="ghost"
            onClick={() =>
              update.mutate(
                {
                  id: r.id,
                  name: r.name,
                  priority: r.priority,
                  active: !r.active,
                  match_all: r.match_all,
                  conditions: r.conditions,
                  actions: r.actions,
                },
                {
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('regras-erro-alterar'),
                      description: errorText(e),
                    }),
                },
              )
            }
          >
            {r.active ? t('regras-desativar') : t('regras-ativar')}
          </Button>
          <IconButton
            size="sm"
            label={t('regras-editar', { nome: r.name })}
            icon={<Icon name="arrowRight" size={13} />}
            onClick={() => setEditing(r)}
          />
          <IconButton
            size="sm"
            label={t('regras-apagar-item', { nome: r.name })}
            icon={<Icon name="x" size={13} />}
            onClick={() => setDeleting(r)}
          />
        </span>
      ),
    },
  ];

  return (
    <InstrumentCard
      index="04.3"
      title={t('regras-titulo')}
      padded={false}
      actions={
        <>
          <Button size="sm" variant="ghost" onClick={() => setApplyOpen(true)}>
            {t('regras-aplicar-reticencias')}
          </Button>
          <Button
            size="sm"
            variant="secondary"
            icon={<Icon name="plus" size={12} />}
            onClick={onNew}
          >
            {t('regras-nova')}
          </Button>
        </>
      }
      footer={[
        { label: t('regras-tabela'), value: data.length },
        { label: t('regras-rodape-ativas'), value: data.filter((r) => r.active).length },
        {
          label: t('regras-rodape-ordem'),
          value: t('regras-rodape-ordem-valor'),
        },
      ]}
    >
      <QueryState loading={isLoading} error={error} onRetry={() => void refetch()}>
        {data.length === 0 ? (
          <div className="p-4">
            <EmptyState
              kicker={t('regras-vazio-kicker')}
              title={t('regras-vazio-titulo')}
              description={t('regras-vazio-descricao')}
              actions={
                <Button variant="primary" onClick={onNew}>
                  {t('regras-nova')}
                </Button>
              }
            />
          </div>
        ) : (
          <TechTable
            label={t('regras-tabela')}
            prefsId="Regras"
            columns={columns}
            rows={data}
            getRowId={(r) => r.id}
            onActivate={setEditing}
            dense
          />
        )}
      </QueryState>
      <RuleDialog
        open={Boolean(editing) || Boolean(draft)}
        onOpenChange={(o) => {
          if (!o) {
            setEditing(null);
            onDraftConsumed();
          }
        }}
        rule={editing}
        initial={draft}
      />
      <ApplyRulesDialog open={applyOpen} onOpenChange={setApplyOpen} filter={filter} />
      <ConfirmDialog
        open={Boolean(deleting)}
        onOpenChange={(o) => !o && setDeleting(null)}
        title={t('regras-apagar')}
        description={t('regras-apagar-descricao', { nome: deleting?.name ?? '' })}
        confirmLabel={t('regras-apagar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { id: deleting.id },
            {
              onSuccess: () => (
                toast({ tone: 'success', title: t('regras-apagada') }),
                setDeleting(null)
              ),
            },
          )
        }
      />
    </InstrumentCard>
  );
}
