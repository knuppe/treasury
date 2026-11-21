/**
 * Ligar uma parcela ao lançamento que a pagou, a partir do contrato: lista as saídas até
 * 20 dias do vencimento ainda sem parcela (`debt.payment_candidates`, as de valor mais
 * próximo primeiro) e liga a escolhida (`debt.link_payment`): a parcela fica paga na data
 * e pelo valor do lançamento. Parcela já ligada mostra o lançamento e pode ser desligada.
 */
import { useState } from 'react';

import type { Installment } from '@/api/generated/Installment';
import type { LoanPaymentCandidate } from '@/api/generated/LoanPaymentCandidate';
import type { LoanView } from '@/api/generated/LoanView';
import {
  useLinkLoanPayment,
  usePaymentCandidates,
  useUnmarkInstallmentPaid,
} from '@/api/hooks/debt';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, Dialog, Money, TechTable, type TechColumn } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { PaymentDiff } from './paymentParts';

export function InstallmentLinkDialog({
  loan,
  installment,
  onClose,
}: {
  loan: LoanView;
  installment: Installment;
  onClose: () => void;
}) {
  const candidates = usePaymentCandidates(installment.id);
  const link = useLinkLoanPayment();
  const unlink = useUnmarkInstallmentPaid();
  const undoToast = useUndoToast();
  const list = candidates.data ?? [];
  const [picked, setPicked] = useState<string | null>(null);
  const chosen = list.find((c) => c.txn_id === picked) ?? list[0] ?? null;
  const n = installment.number;
  const submit = (c: LoanPaymentCandidate | null) => {
    if (!c) return;
    link.mutate(
      { installment_id: installment.id, txn_id: c.txn_id },
      {
        onSuccess: () => {
          undoToast({
            tone: 'success',
            title: t('dividas-ligar-ok', { n }),
            description: `${formatDate(c.date)} · ${c.description} · ${c.account_name}`,
          });
          onClose();
        },
      },
    );
  };
  const columns: TechColumn<LoanPaymentCandidate>[] = [
    {
      key: 'date',
      header: t('dividas-data'),
      width: '96px',
      cell: (c) => <span className="num">{formatDate(c.date)}</span>,
    },
    {
      key: 'description',
      header: t('dividas-descricao'),
      width: '220px',
      cell: (c) => (
        <span className="block max-w-[360px] truncate">
          {c.description || t('dividas-sem-descricao')}
        </span>
      ),
    },
    {
      key: 'account',
      header: t('dividas-conta'),
      width: '160px',
      cell: (c) => (
        <span className="flex min-w-0 items-center gap-1.5">
          <span className="max-w-[200px] truncate">{c.account_name}</span>
          {c.in_debit_account ? (
            <span
              className="label-caps shrink-0 text-accent"
              title={t('dividas-conta-debito-contrato')}
            >
              {t('dividas-debito')}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'amount',
      header: t('dividas-valor'),
      numeric: true,
      width: '112px',
      cell: (c) => <Money cents={c.amount} currency={c.currency} />,
    },
    {
      key: 'diff',
      header: t('dividas-diferenca'),
      numeric: true,
      width: '104px',
      cell: (c) => <PaymentDiff paid={-c.amount} total={installment.total} />,
    },
  ];
  const linked = installment.txn_id != null;
  return (
    <Dialog
      open
      onOpenChange={(o) => (!o ? onClose() : undefined)}
      size="xl"
      title={t('dividas-ligar-titulo', { n })}
      description={withSlots(
        t('dividas-ligar-descricao', {
          contrato: loan.name,
          data: formatDate(installment.due_date),
          valor: slot(0),
        }),
        [<Money key="v" cents={installment.total} currency={loan.currency} />],
      )}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={link.isPending}
            disabled={!chosen}
            onClick={() => submit(chosen)}
          >
            {linked ? t('dividas-ligar-trocar') : t('dividas-ligar-ao-lancamento')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        {linked ? (
          <div
            role="note"
            aria-label={t('dividas-lancamento-ligado')}
            className="flex flex-wrap items-center justify-between gap-3 rounded-card border border-line border-l-2 border-l-positive px-4 py-3 text-[13px]"
          >
            <span className="min-w-0">
              <span className="text-dim">{t('dividas-paga-pelo-lancamento')} </span>
              {installment.txn_date ? formatDate(installment.txn_date) : ''} ·{' '}
              {installment.txn_description || t('dividas-sem-descricao')}
              {installment.txn_account_name ? (
                <span className="text-dim"> · {installment.txn_account_name}</span>
              ) : null}
            </span>
            <Button
              size="sm"
              variant="ghost"
              loading={unlink.isPending}
              onClick={() =>
                unlink.mutate(
                  { id: installment.id },
                  {
                    onSuccess: () => {
                      undoToast({ title: t('dividas-desligada', { n }) });
                      onClose();
                    },
                  },
                )
              }
            >
              {t('dividas-desligar-lancamento')}
            </Button>
          </div>
        ) : null}
        {candidates.isPending ? (
          <p className="text-[13px] text-dim">{t('dividas-ligar-procurando')}</p>
        ) : (
          <TechTable<LoanPaymentCandidate>
            label={t('dividas-ligar-tabela', { n })}
            dense
            customizable={false}
            maxHeight="320px"
            rows={list}
            getRowId={(c) => c.txn_id}
            selectedId={chosen?.txn_id ?? null}
            onSelectedChange={(id) => setPicked(id)}
            onActivate={(c) => submit(c)}
            columns={columns}
            empty={t('dividas-ligar-vazio')}
          />
        )}
      </div>
      <InlineError error={candidates.error ?? link.error ?? unlink.error} />
    </Dialog>
  );
}
