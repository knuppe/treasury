/**
 * Transferências entre contas próprias que chegaram soltas (ex.: o salário cai numa conta e
 * parte vai para outra; cada extrato traz uma perna). Um aviso acima dos lançamentos abre a
 * revisão dos pares: marcar, recusar ("não é transferência") e ligar sozinho as próximas
 * entre as mesmas contas.
 */
import { useMemo, useState } from 'react';

import type { TransferCandidate } from '@/api/generated/TransferCandidate';
import {
  useDismissTransfer,
  useLinkTransfers,
  useTransferCandidates,
} from '@/api/hooks/transactions';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, Dialog, Icon, Money } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { Checkbox, InlineError, Switch } from './ui/controls';
import { TXN_KIND } from './ui/labels';

function kindLabel(kind: string): string {
  if (kind === 'transfer') return TXN_KIND.transfer;
  if (kind === 'card_payment') return TXN_KIND.card_payment;
  if (kind === 'investment') return t('lancamentos-aporte-ou-resgate');
  return kind;
}

const key = (c: TransferCandidate) => `${c.from.txn_id}|${c.to.txn_id}`;

function PairRow({
  c,
  checked,
  onToggle,
  onDismiss,
  dismissing,
}: {
  c: TransferCandidate;
  checked: boolean;
  onToggle: (on: boolean) => void;
  onDismiss: () => void;
  dismissing: boolean;
}) {
  return (
    <li
      className={cx(
        'grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-3 border-b border-line/60 py-2.5 last:border-b-0',
        !checked && 'opacity-70',
      )}
    >
      <Checkbox
        hideLabel
        label={t('lancamentos-ligar-par-rotulo', {
          de: c.from.account_name,
          para: c.to.account_name,
          data: formatDate(c.from.date),
        })}
        checked={checked}
        onChange={onToggle}
      />
      <div className="flex min-w-0 flex-col gap-1">
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-[13.5px] text-fg">
          <span className="font-medium">{c.from.account_name}</span>
          <Icon name="arrowRight" size={12} className="text-dim" />
          <span className="font-medium">{c.to.account_name}</span>
          <span className="text-dim">{'·'}</span>
          <Money cents={c.amount} currency={c.currency} />
          <span className="label-caps text-dim">{kindLabel(c.kind)}</span>
          {c.auto ? (
            <span className="label-caps text-accent">{t('lancamentos-sempre-ligar')}</span>
          ) : null}
        </div>
        <div className="grid gap-x-4 gap-y-0.5 text-[12px] text-dim @min-[620px]:grid-cols-2">
          <span className="truncate" title={c.from.description}>
            {t('lancamentos-sai-em', {
              data: formatDate(c.from.date),
              descricao: c.from.description || '—',
            })}
            {c.from.category_path ? ` (${c.from.category_path})` : ''}
          </span>
          <span className="truncate" title={c.to.description}>
            {t('lancamentos-entra-em', {
              data: formatDate(c.to.date),
              descricao: c.to.description || '—',
            })}
            {c.to.category_path ? ` (${c.to.category_path})` : ''}
          </span>
        </div>
        <span className="text-[11.5px] text-dim first-letter:uppercase">{`${c.reason}.`}</span>
      </div>
      <Button size="sm" variant="ghost" loading={dismissing} onClick={onDismiss}>
        {t('lancamentos-nao-e')}
      </Button>
    </li>
  );
}

/** Aviso com a contagem; abre a revisão. Some quando não há pares. */
export function TransferSuggestions({ className }: { className?: string }) {
  const q = useTransferCandidates();
  const list = useMemo(() => q.data ?? [], [q.data]);
  const [open, setOpen] = useState(false);
  if (list.length === 0) return null;
  return (
    <>
      <div
        role="status"
        className={cx(
          'flex flex-wrap items-center gap-3 rounded-card border border-accent/40 bg-accent-soft/40 px-3 py-2 text-[13px]',
          className,
        )}
      >
        <Icon name="arrowRight" size={14} className="text-accent" />
        <span className="min-w-0 flex-1 text-fg">
          {t('lancamentos-pares-aviso', { n: list.length })}
          <span className="text-dim"> {t('lancamentos-pares-aviso-nota')}</span>
        </span>
        <Button size="sm" variant="primary" onClick={() => setOpen(true)}>
          {t('lancamentos-revisar')}
        </Button>
      </div>
      <TransferReview open={open} onOpenChange={setOpen} candidates={list} />
    </>
  );
}

export function TransferReview({
  open,
  onOpenChange,
  candidates,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  candidates: readonly TransferCandidate[];
}) {
  // Marcados por padrão: os de nota alta. O usuário desmarca o que não for.
  const [picked, setPicked] = useState<Set<string> | null>(null);
  const [remember, setRemember] = useState(true);
  const [dismissing, setDismissing] = useState<string | null>(null);
  const link = useLinkTransfers();
  const dismiss = useDismissTransfer();
  const undoToast = useUndoToast();
  const selected = picked ?? new Set(candidates.filter((c) => c.score >= 60).map(key));
  const chosen = candidates.filter((c) => selected.has(key(c)));
  const toggle = (c: TransferCandidate, on: boolean) => {
    const next = new Set(selected);
    if (on) next.add(key(c));
    else next.delete(key(c));
    setPicked(next);
  };
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) setPicked(null);
        onOpenChange(o);
      }}
      index="03"
      size="xl"
      resizable
      resizeId="transfer-review"
      title={t('lancamentos-revisao-titulo')}
      description={t('lancamentos-revisao-descricao')}
      footer={
        <>
          <div className="mr-auto">
            <Switch
              checked={remember}
              onChange={setRemember}
              label={t('lancamentos-ligar-sozinho')}
              description={t('lancamentos-ligar-sozinho-descricao')}
            />
          </div>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('lancamentos-agora-nao')}
          </Button>
          <Button
            variant="primary"
            loading={link.isPending}
            disabled={chosen.length === 0}
            onClick={() =>
              link.mutate(
                {
                  pairs: chosen.map((c) => ({ from_id: c.from.txn_id, to_id: c.to.txn_id })),
                  remember,
                },
                {
                  onSuccess: (r) => {
                    undoToast({
                      tone: 'success',
                      title: t('lancamentos-n-transferencias-ligadas', { n: r.linked }),
                    });
                    setPicked(null);
                    onOpenChange(false);
                  },
                },
              )
            }
          >
            {t('lancamentos-ligar-n-pares', { n: chosen.length })}
          </Button>
        </>
      }
    >
      <ul aria-label={t('lancamentos-pares-sugeridos')} className="flex flex-col">
        {candidates.map((c) => (
          <PairRow
            key={key(c)}
            c={c}
            checked={selected.has(key(c))}
            onToggle={(on) => toggle(c, on)}
            dismissing={dismissing === key(c)}
            onDismiss={() => {
              setDismissing(key(c));
              dismiss.mutate(
                { from_id: c.from.txn_id, to_id: c.to.txn_id },
                { onSettled: () => setDismissing(null) },
              );
            }}
          />
        ))}
      </ul>
      <InlineError error={link.error ?? dismiss.error} />
    </Dialog>
  );
}
