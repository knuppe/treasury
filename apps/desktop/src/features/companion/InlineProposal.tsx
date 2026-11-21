/**
 * Proposta do agente dentro da conversa do painel: o que muda (resumo amigável e, nos
 * detalhes técnicos, o diff exato) e
 * aprovar ou rejeitar ali mesmo. Sensíveis dizem "item a item" e nunca entram em lote.
 */
import { useApprove, useProposal, useReject } from '@/api/hooks/ai';
import type { ReviewResult } from '@/api/generated/ReviewResult';
import { Button, Icon, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { ChangeReview } from '@/features/agent/ChangeReview';
import {
  LEVEL_LABEL,
  PROPOSAL_STATUS_LABEL,
  commandLabel,
  summaryText,
} from '@/features/agent/format';
import { Pill } from '@/features/agent/ui';
import { errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

function notify(results: ReviewResult[], approved: boolean) {
  const failed = results.find((r) => r.status === 'failed');
  if (failed) {
    toast({
      tone: 'error',
      title: t('companheiro-falha-aplicar'),
      description: failed.error ?? t('agente-dados-mudaram'),
    });
    return;
  }
  toast(
    approved
      ? {
          tone: 'success',
          title: t('companheiro-aprovada'),
          description: t('companheiro-aprovada-texto'),
        }
      : { tone: 'info', title: t('companheiro-rejeitada') },
  );
}

export function InlineProposal({ id, label }: { id: string; label: string }) {
  const detail = useProposal(id);
  const approve = useApprove();
  const reject = useReject();
  const p = detail.data;
  const busy = approve.isPending || reject.isPending;
  const title = p ? commandLabel(p.command) : t('agente-proposta-rotulo', { nome: label });
  const run = (kind: 'approve' | 'reject') => {
    const m = kind === 'approve' ? approve : reject;
    m.mutate(
      { ids: [id] },
      {
        onSuccess: (r) => notify(r, kind === 'approve'),
        onError: (e) =>
          toast({
            tone: 'error',
            title: kind === 'approve' ? t('agente-nao-aprovou') : t('agente-nao-rejeitou'),
            description: errorMessage(e),
          }),
      },
    );
  };
  const sensitive = p?.level === 'sensitive';
  const pending = p?.status === 'pending';
  // Decidida (aprovada, rejeitada, retirada, falhou): uma linha no lugar onde foi criada,
  // que abre os detalhes; só a pendente chama atenção.
  if (p && !pending) {
    return (
      <article
        aria-label={t('agente-proposta-rotulo', { nome: p.reason || title })}
        className="rounded-control border border-line px-2.5 py-1.5 text-[12px]"
      >
        <details className="group">
          <summary className="flex cursor-pointer list-none items-center gap-1.5 select-none">
            <Icon
              name="chevronRight"
              size={11}
              className="shrink-0 text-dim transition-transform group-open:rotate-90"
            />
            <span className="shrink-0 text-dim">{title}</span>
            <span className="min-w-0 flex-1 truncate text-dim">{p.reason}</span>
            <Pill
              tone={
                p.status === 'approved' ? 'positive' : p.status === 'failed' ? 'warning' : 'dim'
              }
            >
              {PROPOSAL_STATUS_LABEL[p.status]}
            </Pill>
          </summary>
          <div className="mt-2 flex flex-col gap-1.5">
            {p.reason ? <p className="text-fg">{p.reason}</p> : null}
            <p className="text-[11.5px] text-dim">
              {LEVEL_LABEL[p.level]} ·{' '}
              {summaryText(p.summary) || t('agente-n-alteracoes', { n: p.item_count })}
            </p>
            {p.error ? <p className="text-[11.5px] text-negative">{p.error}</p> : null}
            {p.items.length > 0 ? <ChangeReview items={p.items} maxRows={4} /> : null}
          </div>
        </details>
      </article>
    );
  }
  return (
    <article
      aria-label={t('agente-proposta-rotulo', { nome: p?.reason || title })}
      className={cx(
        'flex flex-col gap-2 rounded-control border border-l-2 bg-[var(--accent-faint)] px-2.5 py-2 text-[12.5px]',
        sensitive ? 'border-line border-l-warning' : 'border-line border-l-accent',
      )}
    >
      <div className="flex flex-wrap items-center gap-1.5">
        <Icon
          name={sensitive ? 'shield' : 'check'}
          size={12}
          className={sensitive ? 'text-warning' : 'text-accent'}
        />
        <span className="font-medium text-fg">{title}</span>
        {p ? <Pill tone={sensitive ? 'warning' : 'accent'}>{LEVEL_LABEL[p.level]}</Pill> : null}
      </div>
      {p?.reason ? <p className="text-fg">{p.reason}</p> : null}
      {p ? (
        <p className="text-[11.5px] text-dim">
          {summaryText(p.summary) || t('agente-n-alteracoes', { n: p.item_count })}
        </p>
      ) : detail.error ? (
        <p className="text-[11.5px] text-negative">{errorMessage(detail.error)}</p>
      ) : (
        <p className="text-[11.5px] text-dim">{t('companheiro-carregando')}</p>
      )}
      {p?.error ? <p className="text-[11.5px] text-negative">{p.error}</p> : null}
      {p && p.items.length > 0 ? (
        <details className="group">
          <summary className="label-caps flex cursor-pointer list-none items-center gap-1 text-[10px] select-none hover:text-fg">
            <Icon
              name="chevronRight"
              size={11}
              className="transition-transform group-open:rotate-90"
            />
            {t('companheiro-ver-alteracoes')}
          </summary>
          <div className="mt-2">
            <ChangeReview items={p.items} maxRows={4} />
          </div>
        </details>
      ) : null}
      {pending ? (
        <div className="flex flex-wrap items-center gap-2">
          {sensitive ? (
            <span className="text-[11px] text-warning">{t('companheiro-sensivel')}</span>
          ) : null}
          <span className="flex-1" />
          <Button size="sm" variant="ghost" disabled={busy} onClick={() => run('reject')}>
            {t('agente-rejeitar')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            disabled={busy}
            loading={approve.isPending}
            icon={<Icon name="check" size={12} />}
            onClick={() => run('approve')}
          >
            {sensitive ? t('agente-aprovar-item') : t('agente-aprovar')}
          </Button>
        </div>
      ) : null}
    </article>
  );
}
