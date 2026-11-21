import { useImperativeHandle, useState, type Ref } from 'react';

import type { BulkResult } from '@/api/generated/BulkResult';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import type { TxnStatus } from '@/api/generated/TxnStatus';
import { useMembers } from '@/api/hooks/household';
import {
  queryTxnIds,
  sameDescription,
  useDeleteTxns,
  useEditTags,
  useSetCategory,
  useSetMember,
  useSetStatus,
  useTags,
} from '@/api/hooks/transactions';
import { Button, Icon, Input, toast } from '@/design/components';
import type { Point } from '@/design/pointer';
import { t } from '@/i18n';

import { LinkToProjectDialog } from '@/features/projects/ProjectDialogs';

import { CategoryPicker } from './ui/CategoryPicker';
import { ConfirmDialog } from './ui/controls';
import { errorText } from './ui/errors';
import { Popover } from './ui/Popover';
import { TXN_STATUS } from './ui/labels';

export interface LastAction {
  label: string;
  changeSetId: string;
}

/** Filtro sem período nem paginação: "mesma descrição" procura em todo o histórico. */
export function allPeriods(f: TxnFilter): TxnFilter {
  const rest = { ...f };
  delete rest.date_from;
  delete rest.date_to;
  delete rest.limit;
  delete rest.offset;
  return rest;
}

/**
 * As ações da barra para o menu de contexto da tabela (várias linhas marcadas): as
 * mesmas, com os mesmos seletores e a mesma confirmação.
 */
export interface BulkBarHandle {
  markSame: () => void;
  markAll: () => void;
  /** Abre o seletor de categoria da barra; com `at`, flutuando nesse ponto (o mouse). */
  categorize: (at?: Point | null) => void;
  setStatus: (status: TxnStatus) => void;
  /** Abre a lista de membros da barra. */
  member: () => void;
  /** Abre o painel de tags da barra. */
  tags: () => void;
  project: () => void;
  /** Pede a confirmação de apagar os marcados. */
  remove: () => void;
}

/**
 * Ações em lote sobre os lançamentos marcados: marcar os de mesma descrição ou todos do
 * filtro, categorizar, membro, tags, projeto, status, regra e apagar; com duas marcadas,
 * ligar (ou desligar) a transferência entre contas.
 */
export function BulkBar({
  ids,
  filter,
  total,
  onSelect,
  onClear,
  onDone,
  onSuggestRule,
  transferPair,
  onLinkOperation,
  ref,
}: {
  ids: string[];
  /** Filtro da tela (para "mesma descrição" e "todos do filtro"). */
  filter?: TxnFilter;
  /** Lançamentos no filtro. */
  total?: number;
  /** Troca a seleção (ao marcar os de mesma descrição ou todos do filtro). */
  onSelect?: (ids: string[], from?: string) => void;
  onClear: () => void;
  onDone: (action: LastAction | null) => void;
  onSuggestRule?: () => void;
  /**
   * Duas marcadas: ligar como transferência (uma saída e uma entrada soltas em contas
   * diferentes), desligar (as duas pernas da mesma transferência) ou, sem par, o motivo.
   */
  transferPair?: { onLink?: () => void; onUnlink?: () => void; issue?: string; pending?: boolean };
  /** Vincula os marcados a uma operação (meta, cotas capital, carteira), com confirmação. */
  onLinkOperation?: () => void;
  ref?: Ref<BulkBarHandle>;
}) {
  const setCategory = useSetCategory();
  const setStatus = useSetStatus();
  const setMember = useSetMember();
  const editTags = useEditTags();
  const remove = useDeleteTxns();
  const { data: members = [] } = useMembers();
  const { data: tags = [] } = useTags();
  const [tagName, setTagName] = useState('');
  const [marking, setMarking] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [projecting, setProjecting] = useState(false);
  const [categoryOpen, setCategoryOpen] = useState(false);
  const [categoryAt, setCategoryAt] = useState<Point | null>(null);
  const [memberOpen, setMemberOpen] = useState(false);
  const [tagsOpen, setTagsOpen] = useState(false);
  const n = ids.length;

  const finish = (label: string) => (res: BulkResult) => {
    toast({
      tone: 'success',
      title: label,
      description: res.skipped
        ? t('lancamentos-n-alterados-n-fora', { n: res.updated, fora: res.skipped })
        : t('lancamentos-n-alterados', { n: res.updated }),
    });
    onDone(res.change_set_id ? { label, changeSetId: res.change_set_id } : null);
  };
  const fail = (title: string) => (e: unknown) =>
    toast({ tone: 'error', title, description: errorText(e) });

  const markSame = async () => {
    if (!onSelect) return;
    setMarking(true);
    try {
      const r = await sameDescription(ids, allPeriods(filter ?? {}));
      onSelect(r.ids, ids[0]);
      toast({
        tone: 'success',
        title: t('lancamentos-n-marcados-toast', { n: r.ids.length }),
        description: t('lancamentos-mesma-descricao-lista', {
          descricoes: r.descriptions.map((d) => `“${d}”`).join(', '),
        }),
      });
    } catch (e) {
      fail(t('lancamentos-erro-marcar'))(e);
    } finally {
      setMarking(false);
    }
  };
  const markAll = async () => {
    if (!onSelect || !filter) return;
    setMarking(true);
    try {
      onSelect(await queryTxnIds(filter));
    } catch (e) {
      fail(t('lancamentos-erro-marcar'))(e);
    } finally {
      setMarking(false);
    }
  };
  const applyStatus = (s: TxnStatus) =>
    setStatus.mutate(
      { ids, status: s },
      {
        onSuccess: finish(t('lancamentos-status-rotulo', { status: TXN_STATUS[s].toLowerCase() })),
        onError: fail(t('lancamentos-erro-status')),
      },
    );
  useImperativeHandle(ref, () => ({
    markSame: () => void markSame(),
    markAll: () => void markAll(),
    categorize: (at) => {
      setCategoryAt(at ?? null);
      setCategoryOpen(true);
    },
    setStatus: applyStatus,
    member: () => setMemberOpen(true),
    tags: () => setTagsOpen(true),
    project: () => setProjecting(true),
    remove: () => setConfirm(true),
  }));
  const tagAction = (add: boolean, name: string, close: () => void) => {
    const clean = name.trim();
    if (!clean) return;
    close();
    setTagName('');
    editTags.mutate(
      { ids, ...(add ? { add: [clean] } : { remove: [clean] }) },
      {
        onSuccess: finish(
          add
            ? t('lancamentos-tag-posta', { tag: clean })
            : t('lancamentos-tag-tirada', { tag: clean }),
        ),
        onError: fail(t('lancamentos-erro-tags')),
      },
    );
  };

  return (
    <div
      role="toolbar"
      aria-label={t('lancamentos-acoes-lote')}
      className="flex flex-wrap items-center gap-2 rounded-card border border-accent/60 bg-[var(--accent-faint)] px-3 py-2"
    >
      <span className="num text-[13px] text-accent" role="status">
        {t('lancamentos-n-marcado', { n })}
      </span>
      {onSelect ? (
        <Button size="sm" variant="ghost" loading={marking} onClick={() => void markSame()}>
          {t('lancamentos-mesma-descricao')}
        </Button>
      ) : null}
      {onSelect && filter && total != null && total > n ? (
        <Button size="sm" variant="ghost" loading={marking} onClick={() => void markAll()}>
          {t('lancamentos-todos-do-filtro', { n: total })}
        </Button>
      ) : null}
      <CategoryPicker
        value={null}
        label={t('lancamentos-categorizar-marcados')}
        noneLabel={t('lancamentos-tirar-categoria')}
        open={categoryOpen}
        onOpenChange={(o) => {
          setCategoryOpen(o);
          if (!o) setCategoryAt(null);
        }}
        anchor={categoryAt}
        onChange={(category_id) =>
          setCategory.mutate(
            { ids, category_id },
            {
              onSuccess: finish(
                category_id
                  ? t('lancamentos-categoria-definida')
                  : t('lancamentos-categoria-removida'),
              ),
              onError: fail(t('lancamentos-erro-categorizar')),
            },
          )
        }
        compact
      />
      <Popover
        ariaLabel={t('lancamentos-mudar-status-marcados')}
        label={
          <>
            {t('lancamentos-campo-status')}{' '}
            <Icon name="chevronDown" size={12} className="text-dim" />
          </>
        }
      >
        {(close) => (
          <ul className="flex flex-col">
            {(['planned', 'cleared', 'reconciled'] as TxnStatus[]).map((s) => (
              <li key={s}>
                <button
                  type="button"
                  className="h-8 w-full rounded-control px-2 text-left text-[13px] text-fg hover:bg-accent-soft"
                  onClick={() => {
                    close();
                    applyStatus(s);
                  }}
                >
                  {TXN_STATUS[s]}
                </button>
              </li>
            ))}
          </ul>
        )}
      </Popover>
      <Button size="sm" variant="ghost" onClick={() => setProjecting(true)}>
        {t('lancamentos-projeto-reticencias')}
      </Button>
      <Popover
        ariaLabel={t('lancamentos-membro-marcados')}
        open={memberOpen}
        onOpenChange={setMemberOpen}
        label={
          <>
            {t('lancamentos-campo-membro')}{' '}
            <Icon name="chevronDown" size={12} className="text-dim" />
          </>
        }
      >
        {(close) => (
          <ul className="flex flex-col">
            {[
              ...members.map((m) => ({ id: m.id, name: m.name })),
              {
                id: null,
                name: t('lancamentos-sem-membro'),
              },
            ].map((m) => (
              <li key={m.id ?? 'nenhum'}>
                <button
                  type="button"
                  className="h-8 w-full rounded-control px-2 text-left text-[13px] text-fg hover:bg-accent-soft"
                  onClick={() => {
                    close();
                    setMember.mutate(
                      { ids, member_id: m.id },
                      {
                        onSuccess: finish(
                          m.id
                            ? t('lancamentos-membro-definido', { nome: m.name })
                            : t('lancamentos-membro-removido'),
                        ),
                        onError: fail(t('lancamentos-erro-membro')),
                      },
                    );
                  }}
                >
                  {m.name}
                </button>
              </li>
            ))}
          </ul>
        )}
      </Popover>
      <Popover
        ariaLabel={t('lancamentos-tags-marcados')}
        open={tagsOpen}
        onOpenChange={setTagsOpen}
        label={
          <>
            {t('lancamentos-campo-tags')} <Icon name="chevronDown" size={12} className="text-dim" />
          </>
        }
        panelClassName="w-64"
      >
        {(close) => (
          <div className="flex flex-col gap-2">
            <form
              className="flex gap-1.5"
              onSubmit={(e) => {
                e.preventDefault();
                tagAction(true, tagName, close);
              }}
            >
              <Input
                aria-label={t('lancamentos-tag')}
                value={tagName}
                onChange={(e) => setTagName(e.target.value)}
                placeholder={t('lancamentos-nova-tag')}
              />
              <Button size="sm" type="submit" disabled={!tagName.trim()}>
                {t('lancamentos-por')}
              </Button>
            </form>
            {tags.length > 0 ? (
              <ul
                className="flex max-h-48 flex-col overflow-y-auto"
                aria-label={t('lancamentos-tags-existentes')}
              >
                {tags.map((tag) => (
                  <li key={tag.id} className="flex items-center gap-1 text-[13px]">
                    <span className="min-w-0 flex-1 truncate">{tag.name}</span>
                    <Button
                      size="sm"
                      variant="ghost"
                      aria-label={t('lancamentos-por-tag', { tag: tag.name })}
                      onClick={() => tagAction(true, tag.name, close)}
                    >
                      {t('lancamentos-por')}
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      aria-label={t('lancamentos-tirar-tag', { tag: tag.name })}
                      onClick={() => tagAction(false, tag.name, close)}
                    >
                      {t('lancamentos-tirar')}
                    </Button>
                  </li>
                ))}
              </ul>
            ) : null}
          </div>
        )}
      </Popover>
      {transferPair?.onUnlink ? (
        <Button
          size="sm"
          variant="ghost"
          loading={transferPair.pending}
          onClick={transferPair.onUnlink}
        >
          {t('lancamentos-desligar-transferencia')}
        </Button>
      ) : transferPair && (transferPair.onLink || transferPair.issue) ? (
        <Button
          size="sm"
          variant={transferPair.onLink ? 'primary' : 'ghost'}
          icon={<Icon name="arrowRight" size={12} />}
          loading={transferPair.pending}
          disabled={!transferPair.onLink}
          title={transferPair.onLink ? undefined : transferPair.issue}
          aria-description={transferPair.onLink ? undefined : transferPair.issue}
          onClick={transferPair.onLink}
        >
          {t('lancamentos-ligar-como-transferencia')}
        </Button>
      ) : null}
      {onLinkOperation ? (
        <Button size="sm" variant="ghost" onClick={onLinkOperation}>
          {t('lancamentos-vincular-operacao')}
        </Button>
      ) : null}
      {onSuggestRule ? (
        <Button size="sm" variant="ghost" onClick={onSuggestRule}>
          {t('lancamentos-criar-regra')}
        </Button>
      ) : null}
      <Button size="sm" variant="danger" onClick={() => setConfirm(true)}>
        {t('comum-apagar')}
      </Button>
      <Button size="sm" variant="ghost" className="ml-auto" onClick={onClear}>
        {t('controles-limpar-selecao')}
      </Button>
      <LinkToProjectDialog
        ids={ids}
        open={projecting}
        onOpenChange={setProjecting}
        onDone={(label, changeSetId) => onDone(changeSetId ? { label, changeSetId } : null)}
      />
      <ConfirmDialog
        open={confirm}
        onOpenChange={setConfirm}
        title={t('lancamentos-apagar-n', { n })}
        description={t('lancamentos-apagar-n-descricao')}
        confirmLabel={t('comum-apagar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          remove.mutate(
            { ids },
            {
              onSuccess: (res) => {
                setConfirm(false);
                onClear();
                finish(t('lancamentos-apagados'))(res);
              },
            },
          )
        }
      />
    </div>
  );
}
