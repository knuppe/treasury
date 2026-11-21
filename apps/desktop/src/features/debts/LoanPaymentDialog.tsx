/**
 * "Pagamento de parcela" a partir de um lançamento: escolhe o contrato (os ativos) e a
 * parcela que ele pagou — sugere a em aberto de vencimento mais perto da data — e liga os
 * dois (`debt.link_payment`): a parcela fica paga na data e pelo valor do lançamento, que
 * ganha a categoria dos pagamentos do contrato se não tiver uma. Lançamento já ligado
 * mostra a parcela e pode trocar de parcela ou ser desligado (`debt.unmark_paid`).
 */
import { useState } from 'react';
import { Link } from 'react-router';

import type { Installment } from '@/api/generated/Installment';
import type { TxnView } from '@/api/generated/TxnView';
import {
  useLinkLoanPayment,
  useLoanSchedule,
  useLoans,
  useUnmarkInstallmentPaid,
} from '@/api/hooks/debt';
import { useUndoToast } from '@/api/hooks/undoToast';
import {
  Button,
  Dialog,
  Field,
  Money,
  Select,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { screenMeta } from '@/app/screens';
import { InlineError } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { InstallmentBadge, PaymentDiff } from './paymentParts';
import {
  installmentStatus,
  linkableInstallments,
  nextOpen,
  suggestedInstallment,
} from './transform';

const NONE = '__nenhum__';

/** Por que o lançamento não pode pagar parcela (o núcleo recusa do mesmo jeito). */
function blocker(txn: TxnView): string | null {
  if (txn.amount >= 0) return t('dividas-pagamento-so-saida');
  if (txn.transfer_id) return t('dividas-pagamento-transferencia');
  if (txn.status === 'planned') return t('dividas-pagamento-previsto');
  return null;
}

export function LoanPaymentDialog({
  txn,
  open,
  onOpenChange,
  onDone,
}: {
  txn: TxnView;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Depois de ligar ou desligar: o rótulo da ação (para o "Desfazer" da tela). */
  onDone?: (label: string) => void;
}) {
  const loanList = useLoans();
  const base = useBaseCurrency();
  const loans = (loanList.data ?? []).filter((l) => l.currency === txn.currency);
  const [loanPick, setLoanPick] = useState<string | null>(null);
  const [instPick, setInstPick] = useState<string | null>(null);
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) {
      setLoanPick(null);
      setInstPick(null);
    }
  }
  // Contrato sugerido: o da ligação atual, o da categoria dos pagamentos, o da conta de
  // débito ou o único ativo.
  const byDebit = loans.filter((l) => l.debit_account_id === txn.account_id);
  const suggestedLoan =
    loans.find((l) => l.id === txn.loan_id) ??
    (txn.category_id ? loans.find((l) => l.category_id === txn.category_id) : undefined) ??
    (byDebit.length === 1 ? byDebit[0] : undefined) ??
    (loans.length === 1 ? loans[0] : undefined);
  const loanId = loanPick ?? suggestedLoan?.id ?? null;
  const loan = loans.find((l) => l.id === loanId) ?? null;
  const schedule = useLoanSchedule(open ? loanId : null);
  const rows = linkableInstallments(schedule.data ?? [], txn.date, txn.id);
  const next = nextOpen(schedule.data ?? []);
  const current =
    txn.loan_installment_id && txn.loan_id === loanId
      ? (rows.find((i) => i.id === txn.loan_installment_id) ?? null)
      : null;
  const chosen =
    rows.find((i) => i.id === instPick) ?? current ?? suggestedInstallment(rows, txn.date);
  const link = useLinkLoanPayment();
  const unlink = useUnmarkInstallmentPaid();
  const undoToast = useUndoToast();
  const blocked = blocker(txn);
  const linked = txn.loan_installment_id != null;
  const same = linked && chosen?.id === txn.loan_installment_id;
  const pending = link.isPending || unlink.isPending;

  const finish = (label: string, description?: string) => {
    undoToast({ tone: 'success', title: label, ...(description ? { description } : {}) });
    onDone?.(label);
    onOpenChange(false);
  };
  const submit = async (inst: Installment | null) => {
    if (!inst || !loan || blocked || pending) return;
    if (inst.id === txn.loan_installment_id) return;
    try {
      // Trocar de parcela: a anterior volta a ficar em aberto.
      if (txn.loan_installment_id) await unlink.mutateAsync({ id: txn.loan_installment_id });
      await link.mutateAsync({ installment_id: inst.id, txn_id: txn.id });
      finish(
        t('dividas-pagamento-ligado-ok', { n: inst.number, contrato: loan.name }),
        t('dividas-pagamento-paga-em', { data: formatDate(txn.date) }),
      );
    } catch {
      // O erro aparece no diálogo.
    }
  };
  const doUnlink = () => {
    const id = txn.loan_installment_id;
    if (!id) return;
    unlink.mutate(
      { id },
      {
        onSuccess: () =>
          finish(
            t('dividas-pagamento-desligado', {
              n: txn.loan_installment_number ?? '',
              contrato: txn.loan_name ?? t('dividas-contrato-min'),
            }),
            t('dividas-pagamento-voltou'),
          ),
      },
    );
  };

  const columns: TechColumn<Installment>[] = [
    {
      key: 'number',
      header: t('dividas-col-numero'),
      numeric: true,
      width: '64px',
      cell: (i) =>
        i.version > 1 ? (
          <span title={t('dividas-parcela-original', { n: i.number })}>
            {i.version_number}
            <span className="ml-1 text-[11px] text-dim">{`v${i.version}`}</span>
          </span>
        ) : (
          i.number
        ),
    },
    {
      key: 'due',
      header: t('dividas-vencimento'),
      width: '108px',
      cell: (i) => <span className="num">{formatDate(i.due_date)}</span>,
    },
    {
      key: 'total',
      header: t('dividas-parcela'),
      numeric: true,
      width: '136px',
      cell: (i) => <Money cents={i.total} currency={loan?.currency} estimated={i.estimated} />,
    },
    {
      key: 'diff',
      header: t('dividas-diferenca'),
      numeric: true,
      width: '120px',
      cell: (i) => <PaymentDiff paid={-txn.amount} total={i.total} currency={txn.currency} />,
    },
    {
      key: 'status',
      header: t('dividas-situacao'),
      width: '144px',
      cell: (i) =>
        i.id === txn.loan_installment_id ? (
          <span className="label-caps text-positive">
            {'✓ '}
            {t('dividas-pagamento-este')}
          </span>
        ) : (
          <InstallmentBadge status={installmentStatus(i, next?.id ?? null)} />
        ),
    },
  ];

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={t('dividas-pagamento-titulo')}
      description={`${formatDate(txn.date)} · ${txn.description || t('dividas-sem-descricao')} · ${txn.account_name}`}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={pending}
            disabled={!chosen || !loan || Boolean(blocked) || same}
            onClick={() => void submit(chosen)}
          >
            {chosen
              ? linked && !same
                ? t('dividas-pagamento-trocar-para', { n: chosen.number })
                : t('dividas-pagamento-ligar-n', { n: chosen.number })
              : t('dividas-pagamento-ligar')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <p className="text-[13px]">
          <Money cents={txn.amount} currency={txn.currency} tone="auto" />{' '}
          <span className="text-dim">
            {t('dividas-pagamento-saiu-de', { conta: txn.account_name })}
          </span>
        </p>
        {blocked ? <Notice tone="warning" title={blocked} /> : null}
        {linked ? (
          <Notice
            title={t('dividas-pagamento-ligado', {
              n: txn.loan_installment_number ?? '',
              contrato: txn.loan_name ?? t('dividas-um-contrato'),
            })}
            actions={
              <Button size="sm" variant="ghost" loading={unlink.isPending} onClick={doUnlink}>
                {t('dividas-desligar')}
              </Button>
            }
          >
            {t('dividas-pagamento-ligado-texto')}
          </Notice>
        ) : null}
        {loanList.isPending ? (
          <p className="text-[13px] text-dim">{t('dividas-pagamento-carregando-contratos')}</p>
        ) : loans.length === 0 ? (
          <p className="text-[13px] text-dim">
            {withSlots(
              t(
                txn.currency !== base
                  ? 'dividas-pagamento-sem-contrato-moeda'
                  : 'dividas-pagamento-sem-contrato',
                { moeda: txn.currency, tela: slot(0) },
              ),
              [
                <Link key="l" className="text-accent hover:underline" to="/dividas">
                  {`${screenMeta('dividas').index} · ${screenMeta('dividas').title}`}
                </Link>,
              ],
            )}
          </p>
        ) : (
          <>
            <Field label={t('dividas-contrato')}>
              <Select
                aria-label={t('dividas-contrato')}
                value={loanId ?? NONE}
                onValueChange={(v) => {
                  setLoanPick(v === NONE ? null : v);
                  setInstPick(null);
                }}
                options={[
                  ...(loanId ? [] : [{ value: NONE, label: t('dividas-pagamento-escolha') }]),
                  ...loans.map((l) => ({ value: l.id, label: l.name })),
                ]}
              />
            </Field>
            {loan ? (
              schedule.isPending ? (
                <p className="text-[13px] text-dim">{t('dividas-pagamento-carregando-parcelas')}</p>
              ) : (
                <TechTable<Installment>
                  label={t('dividas-pagamento-parcelas-de', { nome: loan.name })}
                  dense
                  customizable={false}
                  maxHeight="300px"
                  rows={rows}
                  getRowId={(i) => i.id}
                  selectedId={chosen?.id ?? null}
                  onSelectedChange={(id) => setInstPick(id)}
                  onActivate={(i) => void submit(i)}
                  columns={columns}
                  empty={t('dividas-pagamento-vazio')}
                />
              )
            ) : null}
            {loan ? (
              <p className="text-[12.5px] text-dim">
                {t(
                  loan.category_id && !txn.category_id
                    ? 'dividas-pagamento-nota-categoria'
                    : 'dividas-pagamento-nota',
                  { data: formatDate(txn.date) },
                )}
              </p>
            ) : null}
          </>
        )}
      </div>
      <InlineError error={loanList.error ?? schedule.error ?? unlink.error ?? link.error} />
    </Dialog>
  );
}
