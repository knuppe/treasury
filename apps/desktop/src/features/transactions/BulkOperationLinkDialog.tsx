/**
 * "Vincular a uma operação" com vários lançamentos marcados (os depósitos mensais da
 * poupança programada, os aportes nas cotas capital): escolhe a operação de destino
 * (`operation_candidates` com `txn_ids`: as que combinam com mais lançamentos primeiro) e
 * confirma numa segunda etapa, com a lista dos lançamentos, o total e os que ficam de fora
 * e por quê (`operation_link_preview`: já vinculados, previstos, parcelas de dívida…).
 * Confirmado, `link_operations` vincula os que servem num conjunto só do histórico.
 */
import { useState } from 'react';

import type { OperationCandidate } from '@/api/generated/OperationCandidate';
import type { OperationLinkItem } from '@/api/generated/OperationLinkItem';
import {
  useLinkOperations,
  useOperationCandidatesMany,
  useOperationLinkPreview,
} from '@/api/hooks/transactions';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, Dialog, Money, TechTable, type TechColumn } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { OperationChoices, operationKey } from './OperationLinkDialog';

const columns = (): TechColumn<OperationLinkItem>[] => [
  {
    key: 'date',
    header: t('lancamentos-campo-data'),
    width: '96px',
    cell: (i) => <span className="num">{formatDate(i.date)}</span>,
  },
  {
    key: 'description',
    header: t('lancamentos-campo-descricao'),
    cell: (i) => (
      <span className="flex min-w-0 flex-col">
        <span className="truncate">
          {i.description || t('lancamentos-sem-descricao-parenteses')}
        </span>
        <span className="truncate text-[11.5px] text-dim">{i.account_name}</span>
      </span>
    ),
  },
  {
    key: 'amount',
    header: t('lancamentos-campo-valor'),
    numeric: true,
    width: '120px',
    cell: (i) => <Money cents={i.amount} currency={i.currency} tone="auto" />,
  },
  {
    key: 'status',
    header: t('lancamentos-situacao'),
    width: '220px',
    cell: (i) =>
      i.ok ? (
        <span className="label-caps text-positive">{t('lancamentos-entra')}</span>
      ) : (
        <span className="text-[12px] text-warning" title={i.reason ?? undefined}>
          {t('lancamentos-fica-de-fora', { motivo: i.reason ?? '' })}
        </span>
      ),
  },
];

export function BulkOperationLinkDialog({
  ids,
  open,
  onOpenChange,
  onDone,
}: {
  ids: readonly string[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Depois de vincular: o rótulo e o conjunto (para o "Desfazer" da tela). */
  onDone?: (label: string, changeSetId: string | null) => void;
}) {
  const list = useOperationCandidatesMany(open ? ids : null);
  const candidates = list.data ?? [];
  const [pick, setPick] = useState<string | null>(null);
  const [confirming, setConfirming] = useState<OperationCandidate | null>(null);
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) {
      setPick(null);
      setConfirming(null);
    }
  }
  const chosen = candidates.find((c) => operationKey(c) === pick) ?? candidates[0] ?? null;
  const preview = useOperationLinkPreview(
    open && confirming
      ? { txn_ids: [...ids], kind: confirming.kind, target_id: confirming.target_id }
      : null,
  );
  const link = useLinkOperations();
  const undoToast = useUndoToast();
  const p = preview.data;
  const skipped = p ? p.items.length - p.linkable : 0;

  const submit = () => {
    if (!confirming || !p || p.linkable === 0) return;
    link.mutate(
      { txn_ids: [...ids], kind: confirming.kind, target_id: confirming.target_id },
      {
        onSuccess: (r) => {
          const title = t('lancamentos-n-vinculados', { n: r.linked });
          undoToast({
            tone: 'success',
            title,
            changeSetId: r.change_set_id,
            description:
              r.skipped > 0
                ? t('lancamentos-vinculados-descricao-fora', { operacao: r.label, fora: r.skipped })
                : t('lancamentos-vinculados-descricao', { operacao: r.label }),
          });
          onDone?.(title, r.change_set_id);
          onOpenChange(false);
        },
      },
    );
  };

  const n = ids.length;
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={confirming ? t('lancamentos-confirmar-vinculo') : t('lancamentos-vincular-titulo')}
      description={
        confirming
          ? t('lancamentos-n-marcados-destino', { n, destino: confirming.label })
          : t('lancamentos-n-marcados-escolha', { n })
      }
      footer={
        confirming ? (
          <>
            <Button variant="ghost" onClick={() => setConfirming(null)}>
              {t('comum-voltar')}
            </Button>
            <Button
              variant="primary"
              loading={link.isPending}
              disabled={!p || p.linkable === 0}
              onClick={submit}
            >
              {p ? t('lancamentos-vincular-n', { n: p.linkable }) : t('lancamentos-vincular')}
            </Button>
          </>
        ) : (
          <>
            <Button variant="ghost" onClick={() => onOpenChange(false)}>
              {t('comum-cancelar')}
            </Button>
            <Button variant="primary" disabled={!chosen} onClick={() => setConfirming(chosen)}>
              {t('comum-continuar')}
            </Button>
          </>
        )
      }
    >
      <div className="flex flex-col gap-4">
        {confirming ? (
          preview.isPending ? (
            <p className="text-[13px] text-dim">{t('lancamentos-conferindo')}</p>
          ) : p ? (
            <>
              <p className="text-[13px]">
                <span className="text-dim">{t('lancamentos-destino')}</span> {p.label}
                {p.totals.length > 0 ? (
                  <>
                    {' · '}
                    <span className="text-dim">{t('lancamentos-total')}</span>{' '}
                    {p.totals.map((t, i) => (
                      <span key={t.currency}>
                        {i > 0 ? ' + ' : null}
                        <Money cents={t.amount} currency={t.currency} tone="auto" />
                      </span>
                    ))}
                  </>
                ) : null}
              </p>
              {skipped > 0 ? (
                <Notice tone="warning" title={t('lancamentos-n-ficam-de-fora', { n: skipped })}>
                  {p.linkable > 0
                    ? t('lancamentos-outros-vinculados')
                    : t('lancamentos-nenhum-vinculavel')}
                </Notice>
              ) : null}
              <TechTable
                label={t('lancamentos-a-vincular')}
                prefsId="Lançamentos a vincular"
                columns={columns()}
                rows={p.items}
                getRowId={(i) => i.txn_id}
                dense
                maxHeight="320px"
                customizable={false}
              />
              <p className="text-[12px] text-dim">{t('lancamentos-vinculados-nota')}</p>
            </>
          ) : (
            <InlineError error={preview.error} />
          )
        ) : list.isPending ? (
          <p className="text-[13px] text-dim">{t('lancamentos-procurando-operacoes')}</p>
        ) : candidates.length === 0 ? (
          <p className="text-[13px] text-dim">{t('lancamentos-nenhuma-operacao-marcados')}</p>
        ) : (
          <OperationChoices
            candidates={candidates}
            chosen={chosen}
            onPick={(c) => setPick(operationKey(c))}
          />
        )}
        <InlineError error={link.error} />
      </div>
    </Dialog>
  );
}
