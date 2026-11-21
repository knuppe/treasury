/**
 * "Vincular a uma operação" a partir de um lançamento (`transactions.link_operation`): o
 * débito da aplicação no CDB, o aporte nas cotas capital da cooperativa, o depósito da
 * poupança programada. Mostra as operações prováveis (`operation_candidates`: compras e
 * vendas da carteira de valor e data perto, as cotas capital e as metas) e liga a escolhida:
 * o lançamento vira aporte (fora de receitas e despesas, sem categoria) e entra no valor da
 * meta ou nos aportes das cotas. Lançamento já vinculado mostra a operação e pode ser
 * desvinculado (volta ao tipo e à categoria de antes). Parcelas de dívida seguem no
 * "Pagamento de parcela"; transferências, na ligação de contas.
 */
import { useState } from 'react';
import { Link } from 'react-router';

import type { OperationCandidate } from '@/api/generated/OperationCandidate';
import type { TxnOperationKind } from '@/api/generated/TxnOperationKind';
import type { TxnView } from '@/api/generated/TxnView';
import {
  useLinkOperation,
  useOperationCandidates,
  useUnlinkOperation,
} from '@/api/hooks/transactions';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, Dialog, Money } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { InlineError } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

export const OPERATION_KIND: Record<TxnOperationKind, string> = {
  get trade() {
    return t('lancamentos-operacao-carteira');
  },
  get capital_share() {
    return t('lancamentos-operacao-cotas');
  },
  get goal() {
    return t('lancamentos-operacao-meta');
  },
};

/** Por que o lançamento não se vincula (o núcleo recusa do mesmo jeito). */
export function operationBlocker(txn: TxnView): string | null {
  if (txn.transfer_id) return t('lancamentos-bloqueio-transferencia');
  if (txn.status === 'planned') return t('lancamentos-bloqueio-previsto');
  if (txn.splits.length > 0) return t('lancamentos-bloqueio-dividido');
  if (txn.loan_id)
    return t('lancamentos-bloqueio-parcela', {
      contrato: txn.loan_name ?? t('lancamentos-um-contrato'),
    });
  return null;
}

export const operationKey = (c: Pick<OperationCandidate, 'kind' | 'target_id'>) =>
  `${c.kind}:${c.target_id}`;
const key = operationKey;

/** Lista de operações para escolher uma (rádio). */
export function OperationChoices({
  candidates,
  chosen,
  onPick,
}: {
  candidates: readonly OperationCandidate[];
  chosen: OperationCandidate | null;
  onPick: (c: OperationCandidate) => void;
}) {
  return (
    <ul aria-label={t('lancamentos-operacoes')} className="flex flex-col gap-1.5" role="radiogroup">
      {candidates.map((c) => {
        const on = chosen != null && key(c) === key(chosen);
        return (
          <li key={key(c)}>
            <button
              type="button"
              role="radio"
              aria-checked={on}
              onClick={() => onPick(c)}
              className={cx(
                'flex w-full items-start gap-3 rounded-card border px-3 py-2 text-left',
                on ? 'border-accent bg-[var(--accent-faint)]' : 'border-line hover:bg-accent-soft',
              )}
            >
              <span className="label-caps mt-0.5 w-24 shrink-0 text-dim">
                {OPERATION_KIND[c.kind]}
              </span>
              <span className="flex min-w-0 flex-col">
                <span className="text-[13px]">{c.label}</span>
                <span className="text-[12px] text-dim">{c.reason}</span>
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

export function OperationLinkDialog({
  txn,
  open,
  onOpenChange,
  onDone,
}: {
  txn: TxnView;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Depois de vincular ou desvincular: o rótulo e o conjunto (para o "Desfazer" da tela). */
  onDone?: (label: string, changeSetId: string | null) => void;
}) {
  const linked = txn.operation;
  const blocked = linked ? null : operationBlocker(txn);
  const list = useOperationCandidates(open && !linked && !blocked ? txn.id : null);
  const candidates = list.data ?? [];
  const [pick, setPick] = useState<string | null>(null);
  const chosen = candidates.find((c) => key(c) === pick) ?? candidates[0] ?? null;
  const link = useLinkOperation();
  const unlink = useUnlinkOperation();
  const undoToast = useUndoToast();
  const pending = link.isPending || unlink.isPending;

  const finish = (label: string, changeSetId: string | null, description?: string) => {
    undoToast({
      tone: 'success',
      title: label,
      changeSetId,
      ...(description ? { description } : {}),
    });
    onDone?.(label, changeSetId);
    onOpenChange(false);
  };
  const submit = () => {
    if (!chosen || pending) return;
    link.mutate(
      { txn_id: txn.id, kind: chosen.kind, target_id: chosen.target_id },
      {
        onSuccess: (r) =>
          finish(
            t('lancamentos-vinculado-toast'),
            r.change_set_id,
            t('lancamentos-vinculado-descricao', { operacao: r.operation.label }),
          ),
      },
    );
  };
  const doUnlink = () =>
    unlink.mutate(
      { txn_id: txn.id },
      {
        onSuccess: (r) =>
          finish(
            t('lancamentos-vinculo-desfeito'),
            r.change_set_id,
            t('lancamentos-vinculo-desfeito-descricao'),
          ),
      },
    );

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={t('lancamentos-vincular-titulo')}
      description={`${formatDate(txn.date)} · ${txn.description || t('lancamentos-sem-descricao')} · ${txn.account_name}`}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {linked ? t('comum-fechar') : t('comum-cancelar')}
          </Button>
          {linked ? null : (
            <Button
              variant="primary"
              loading={link.isPending}
              disabled={!chosen || Boolean(blocked)}
              onClick={submit}
            >
              {t('lancamentos-vincular')}
            </Button>
          )}
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <p className="text-[13px]">
          <Money cents={txn.amount} currency={txn.currency} tone="auto" />{' '}
          <span className="text-dim">
            {txn.amount < 0
              ? t('lancamentos-saiu-de', { conta: txn.account_name })
              : t('lancamentos-entrou-em', { conta: txn.account_name })}
          </span>
        </p>
        {blocked ? <Notice tone="warning" title={blocked} /> : null}
        {linked ? (
          <Notice
            title={t('lancamentos-vinculado', { operacao: linked.label })}
            actions={
              <Button size="sm" variant="ghost" loading={unlink.isPending} onClick={doUnlink}>
                {t('lancamentos-desvincular')}
              </Button>
            }
          >
            {t('lancamentos-vinculado-nota', {
              tipo: OPERATION_KIND[linked.kind].toLowerCase(),
            })}
          </Notice>
        ) : blocked ? null : list.isPending ? (
          <p className="text-[13px] text-dim">{t('lancamentos-procurando-operacoes')}</p>
        ) : candidates.length === 0 ? (
          <p className="text-[13px] text-dim">
            {withSlots(t('lancamentos-nenhuma-operacao', { carteira: slot(0) }), [
              <Link key="carteira" className="text-accent hover:underline" to="/carteira">
                {t('casca-tela-carteira')}
              </Link>,
            ])}
          </p>
        ) : (
          <OperationChoices
            candidates={candidates}
            chosen={chosen}
            onPick={(c) => setPick(key(c))}
          />
        )}
        <InlineError error={link.error ?? unlink.error} />
      </div>
    </Dialog>
  );
}
