/**
 * Fila de aprovação: propostas pendentes da IA com o que muda em linguagem simples (nomes,
 * valores e datas formatados) e o diff exato (antes/depois de cada linha) em "Ver detalhes
 * técnicos". Reversíveis podem ser aprovadas em lote; sensíveis, só item a item. O estado do
 * modo autônomo aparece aqui só para leitura (muda-se em Configurar).
 */
import { useQueries } from '@tanstack/react-query';
import { useEffect, useMemo, useRef, useState } from 'react';
import { Link } from 'react-router';

import { call } from '@/api/client';
import type { Proposal } from '@/api/generated/Proposal';
import type { ProposalDetail } from '@/api/generated/ProposalDetail';
import type { ReviewResult } from '@/api/generated/ReviewResult';
import {
  aiKeys,
  useAiSettings,
  useApprove,
  useProposals,
  useReject,
  useUndoChangeSet,
} from '@/api/hooks/ai';
import { Button, Icon, InstrumentCard, toast } from '@/design/components';
import { cx } from '@/design/cx';
import {
  ErrorBlock,
  LoadingBlock,
  Notice,
  errorMessage,
} from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { ChangeReview } from './ChangeReview';
import { LEVEL_LABEL, PROPOSAL_STATUS_LABEL, commandLabel, summaryText } from './format';
import { Pill } from './ui';

function reviewToast(results: ReviewResult[], approved: boolean) {
  const failed = results.filter((r) => r.status === 'failed');
  const withdrawn = results.filter((r) => r.status === 'withdrawn');
  const ok = results.length - failed.length - withdrawn.length;
  if (withdrawn.length > 0) {
    toast({
      tone: 'info',
      title: t('agente-retiradas', { n: withdrawn.length }),
      description: t('agente-retiradas-texto'),
    });
  }
  if (ok > 0) {
    toast({
      tone: 'success',
      title: t(approved ? 'agente-aprovadas' : 'agente-rejeitadas', { n: ok }),
      description: approved ? t('agente-aprovadas-texto') : undefined,
    });
  }
  for (const f of failed) {
    toast({
      tone: 'error',
      title: t('agente-falha-aplicar'),
      description: f.error ?? t('agente-dados-mudaram'),
    });
  }
}

function ProposalCard({
  proposal,
  detail,
  selected,
  onSelect,
  onApprove,
  onReject,
  busy,
  highlighted,
}: {
  proposal: Proposal;
  detail: { data?: ProposalDetail; error: unknown; isLoading: boolean };
  selected: boolean;
  onSelect: (checked: boolean) => void;
  onApprove: () => void;
  onReject: () => void;
  busy: boolean;
  highlighted: boolean;
}) {
  const sensitive = proposal.level === 'sensitive';
  const ref = useRef<HTMLElement>(null);
  useEffect(() => {
    if (highlighted) ref.current?.scrollIntoView({ block: 'nearest' });
  }, [highlighted]);
  const title = commandLabel(proposal.command);
  return (
    <article
      ref={ref}
      aria-label={t('agente-proposta-rotulo', { nome: proposal.reason || title })}
      className={cx(
        'flex flex-col gap-3 rounded-card border bg-surface p-4',
        highlighted ? 'border-accent' : 'border-line',
      )}
    >
      <header className="flex flex-wrap items-start gap-3">
        {sensitive ? (
          <span className="mt-0.5 text-warning" title={t('agente-sensivel-dica')}>
            <Icon name="shield" size={14} />
          </span>
        ) : (
          <input
            type="checkbox"
            className="mt-1 size-3.5 accent-[var(--accent)]"
            aria-label={t('agente-selecionar-lote', { nome: proposal.reason || title })}
            checked={selected}
            onChange={(e) => onSelect(e.target.checked)}
          />
        )}
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-[14px] font-medium text-fg">{title}</h3>
            <Pill tone={sensitive ? 'warning' : 'accent'}>{LEVEL_LABEL[proposal.level]}</Pill>
          </div>
          {proposal.reason ? <p className="text-[13px] text-fg">{proposal.reason}</p> : null}
          <p className="text-[12px] text-dim">
            {[
              summaryText(proposal.summary) || t('agente-n-alteracoes', { n: proposal.item_count }),
              proposal.client,
              formatDateTime(proposal.created_at),
            ].join(' · ')}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant="ghost" onClick={onReject} disabled={busy}>
            {t('agente-rejeitar')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            onClick={onApprove}
            disabled={busy}
            icon={<Icon name="check" size={12} />}
          >
            {sensitive ? t('agente-aprovar-item') : t('agente-aprovar')}
          </Button>
        </div>
      </header>
      {detail.isLoading ? (
        <LoadingBlock label={t('agente-carregando-diff')} className="min-h-0 py-3" />
      ) : detail.error ? (
        <ErrorBlock error={detail.error} title={t('agente-erro-diff')} />
      ) : detail.data ? (
        <ChangeReview items={detail.data.items} />
      ) : null}
    </article>
  );
}

function ReviewedList({ items }: { items: Proposal[] }) {
  const undo = useUndoChangeSet();
  if (items.length === 0) return null;
  return (
    <InstrumentCard index="02" title={t('agente-revisadas')} padded={false}>
      <ul className="divide-y divide-line">
        {items.map((p) => (
          <li key={p.id} className="flex flex-wrap items-center gap-3 px-4 py-2.5">
            <Pill
              tone={
                p.status === 'approved' ? 'positive' : p.status === 'failed' ? 'negative' : 'dim'
              }
            >
              {PROPOSAL_STATUS_LABEL[p.status]}
            </Pill>
            <span className="min-w-0 flex-1 text-[13px] text-fg">
              {commandLabel(p.command)}
              {p.reason ? <span className="text-dim">{` · ${p.reason}`}</span> : null}
              {p.error ? <span className="block text-[12px] text-negative">{p.error}</span> : null}
              {p.status === 'withdrawn' && p.note ? (
                <span className="block text-[12px] text-dim">{p.note}</span>
              ) : null}
            </span>
            <span className="num text-[11px] text-dim">
              {p.reviewed_at ? formatDateTime(p.reviewed_at) : ''}
            </span>
            {p.status === 'approved' ? (
              <Button
                size="sm"
                variant="ghost"
                loading={undo.isPending && undo.variables === p.change_set_id}
                onClick={() =>
                  undo.mutate(p.change_set_id, {
                    onSuccess: () => toast({ tone: 'success', title: t('agente-desfeita') }),
                    onError: (e) =>
                      toast({
                        tone: 'error',
                        title: t('agente-nao-desfez'),
                        description: errorMessage(e),
                      }),
                  })
                }
              >
                {t('componentes-desfazer')}
              </Button>
            ) : null}
          </li>
        ))}
      </ul>
    </InstrumentCard>
  );
}

export function ApprovalQueue({
  focusId,
  settingsHref,
}: {
  focusId?: string | null;
  settingsHref: string;
}) {
  const pending = useProposals({ status: 'pending' });
  const all = useProposals({ limit: 30 });
  const settings = useAiSettings();
  const approve = useApprove();
  const reject = useReject();
  const [selected, setSelected] = useState<ReadonlySet<string>>(new Set());

  const list = useMemo(
    // Mais antigas primeiro: é a ordem em que o núcleo aplica.
    () => [...(pending.data ?? [])].sort((a, b) => a.created_at.localeCompare(b.created_at)),
    [pending.data],
  );
  const details = useQueries({
    queries: list.map((p) => ({
      queryKey: aiKeys.proposal(p.id),
      queryFn: () => call<ProposalDetail>('ai.get_proposal', { id: p.id }),
    })),
  });
  const reversibleIds = list.filter((p) => p.level === 'reversible').map((p) => p.id);
  const chosen = reversibleIds.filter((id) => selected.has(id));
  const busy = approve.isPending || reject.isPending;

  const run = (kind: 'approve' | 'reject', ids: string[]) => {
    const m = kind === 'approve' ? approve : reject;
    m.mutate(
      { ids },
      {
        onSuccess: (r) => {
          reviewToast(r, kind === 'approve');
          setSelected((s) => new Set([...s].filter((id) => !ids.includes(id))));
        },
        onError: (e) =>
          toast({
            tone: 'error',
            title: kind === 'approve' ? t('agente-nao-aprovou') : t('agente-nao-rejeitou'),
            description: errorMessage(e),
          }),
      },
    );
  };

  const reviewed = (all.data ?? []).filter((p) => p.status !== 'pending').slice(0, 10);
  const autonomous = settings.data?.autonomous;

  return (
    <div className="flex flex-col gap-4">
      <Notice
        tone={autonomous ? 'warning' : 'info'}
        title={
          autonomous === undefined
            ? t('agente-autonomo')
            : autonomous
              ? t('agente-autonomo-ligado')
              : t('agente-autonomo-desligado')
        }
        actions={
          <Link
            className="text-[12.5px] text-accent underline-offset-2 hover:underline"
            to={settingsHref}
          >
            {t('agente-alterar-configurar')}
          </Link>
        }
      >
        {t('agente-sensiveis-texto')}
      </Notice>

      <InstrumentCard
        index="01"
        title={list.length ? t('agente-pendentes-n', { n: list.length }) : t('agente-pendentes')}
        actions={
          reversibleIds.length > 0 ? (
            <>
              <Button
                size="sm"
                variant="ghost"
                onClick={() =>
                  setSelected(
                    chosen.length === reversibleIds.length ? new Set() : new Set(reversibleIds),
                  )
                }
              >
                {chosen.length === reversibleIds.length
                  ? t('controles-limpar-selecao')
                  : t('agente-selecionar-reversiveis')}
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={chosen.length === 0 || busy}
                onClick={() => run('reject', chosen)}
              >
                {t('agente-rejeitar-selecionadas')}
              </Button>
              <Button
                size="sm"
                variant="primary"
                disabled={chosen.length === 0 || busy}
                loading={approve.isPending && chosen.length > 1}
                onClick={() => run('approve', chosen)}
              >
                {t('agente-aprovar-selecionadas', { n: chosen.length })}
              </Button>
            </>
          ) : null
        }
      >
        {pending.isLoading ? (
          <LoadingBlock />
        ) : pending.error ? (
          <ErrorBlock error={pending.error} onRetry={() => void pending.refetch()} />
        ) : list.length === 0 ? (
          <p className="py-6 text-center text-[13px] text-dim">{t('agente-fila-vazia')}</p>
        ) : (
          <div className="flex flex-col gap-3">
            {list.map((p, i) => (
              <ProposalCard
                key={p.id}
                proposal={p}
                detail={details[i] ?? { error: null, isLoading: true }}
                selected={selected.has(p.id)}
                onSelect={(c) =>
                  setSelected((s) => {
                    const next = new Set(s);
                    if (c) next.add(p.id);
                    else next.delete(p.id);
                    return next;
                  })
                }
                onApprove={() => run('approve', [p.id])}
                onReject={() => run('reject', [p.id])}
                busy={busy}
                highlighted={focusId === p.id}
              />
            ))}
          </div>
        )}
      </InstrumentCard>

      <ReviewedList items={reviewed} />
    </div>
  );
}
