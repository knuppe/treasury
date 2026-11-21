/**
 * Tabela de amortização do contrato: uma linha por parcela com amortização, juros,
 * correção, seguros e taxas, total e saldo; status (paga, atrasada, próxima, prevista),
 * marca `≈` quando o índice do mês é estimado, o lançamento que pagou a parcela e as ações
 * de pagamento (marcar paga, ligar a um lançamento, desligar), também no menu da linha.
 */
import { memo, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useUndoToast } from '@/api/hooks/undoToast';

import {
  useMarkInstallmentPaid,
  useRecordLoanPayment,
  useUnmarkInstallmentPaid,
} from '@/api/hooks/debt';
import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import {
  Button,
  type ContextMenuEntry,
  DateInput,
  Dialog,
  Field,
  Input,
  Money,
  MoneyInput,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { ConfirmDialog, InlineError, Segmented } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, formatMoney, todayIso } from '@/lib/format';
import { useUi } from '@/state/ui';
import { addDays } from '@/lib/months';
import { txnLink } from '@/lib/txnLink';

import { InstallmentLinkDialog } from './InstallmentLinkDialog';
import { InstallmentBadge } from './paymentParts';
import { installmentStatus, nextOpen, periodRateText, totals } from './transform';

type Filter = 'proximas' | 'abertas' | 'pagas' | 'todas';

/** Lançamentos do dia do pagamento, na conta dele (a tela 03 abre filtrada). */
function paymentLink(i: Installment): string | null {
  return i.txn_account_id && i.txn_date
    ? txnLink({ account_ids: [i.txn_account_id], date_from: i.txn_date, date_to: i.txn_date })
    : null;
}

/** Data · descrição do lançamento que pagou a parcela. */
function txnText(i: Installment): string {
  return `${i.txn_date ? formatDate(i.txn_date) : ''} · ${i.txn_description || t('dividas-sem-descricao')}`;
}

/** O lançamento que pagou a parcela (data · descrição), com link para ele em Lançamentos. */
function PaymentTxn({ installment: i }: { installment: Installment }) {
  if (!i.txn_id) return null;
  const text = txnText(i);
  const to = paymentLink(i);
  const title = i.txn_account_name
    ? t('dividas-paga-pelo-lancamento-titulo-conta', {
        lancamento: text,
        conta: i.txn_account_name,
      })
    : t('dividas-paga-pelo-lancamento-titulo', { lancamento: text });
  const className = 'block max-w-[148px] truncate text-[11px]';
  return to ? (
    <Link className={cx(className, 'text-accent hover:underline')} to={to} title={title}>
      {text}
    </Link>
  ) : (
    <span className={cx(className, 'text-dim')} title={title}>
      {text}
    </span>
  );
}

/** Recorte padrão: as 3 últimas pagas, as atrasadas e as 12 próximas em aberto. */
export function windowAround(installments: readonly Installment[]): Installment[] {
  const paid = installments.filter((i) => i.paid).slice(-3);
  const open = installments.filter((i) => !i.paid);
  const overdue = open.filter((i) => i.overdue);
  const upcoming = open.filter((i) => !i.overdue).slice(0, 12);
  return [...paid, ...overdue, ...upcoming].sort((a, b) => a.number - b.number);
}

export const ScheduleTable = memo(function ScheduleTable({
  loan,
  installments,
}: {
  loan: LoanView;
  installments: readonly Installment[];
}) {
  const [filter, setFilter] = useState<Filter>('proximas');
  const [paying, setPaying] = useState<Installment | null>(null);
  const [undoing, setUndoing] = useState<Installment | null>(null);
  const [linking, setLinking] = useState<Installment | null>(null);
  const navigate = useNavigate();
  const next = nextOpen(installments);
  const rows = useMemo(
    () =>
      filter === 'proximas'
        ? windowAround(installments)
        : installments.filter((i) =>
            filter === 'todas' ? true : filter === 'pagas' ? i.paid : !i.paid,
          ),
    [installments, filter],
  );
  const sum = totals(rows);
  const hasAdjustment = installments.some((i) => i.adjustment !== 0);
  const hasCharges = installments.some((i) => i.insurance + i.fees !== 0);
  const hasPeriodFactor = installments.some((i) => i.index_factor != null);
  const overdue = installments.filter((i) => i.overdue).length;

  const privacy = useUi((u) => u.privacy);
  // Composição dos encargos no título da célula (fora do modo privacidade).
  const money = (c: number) => formatMoney(c);
  const columns: TechColumn<Installment>[] = [
    {
      key: 'number',
      header: t('dividas-col-numero'),
      numeric: true,
      width: '72px',
      // Como o banco: cada amortização abre uma nova versão do contrato e a contagem
      // recomeça em 1; a ordenação segue a parcela do contrato original.
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
      width: '112px',
      cell: (i) => <span className="num">{formatDate(i.due_date)}</span>,
    },
    {
      key: 'amortization',
      header: t('dividas-amortizacao'),
      numeric: true,
      cell: (i) => (
        <span className="flex flex-col items-end">
          <Money cents={i.amortization} />
          {i.extra_payment > 0 ? (
            <span className="text-[11px] text-accent">
              {withSlots(t('dividas-mais-extra', { valor: slot(0) }), [
                <Money key="e" cents={i.extra_payment} />,
              ])}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'interest',
      header: t('dividas-juros'),
      numeric: true,
      cell: (i) => <Money cents={i.interest} />,
    },
    ...(hasPeriodFactor
      ? [
          {
            key: 'period',
            header: t('dividas-juros-periodo'),
            numeric: true,
            cell: (i: Installment) =>
              i.index_factor ? (
                <span
                  className="flex flex-col items-end leading-tight"
                  title={t('dividas-juros-periodo-dica')}
                >
                  <span className="num">{periodRateText(i.index_factor)}</span>
                  <span className="text-[11px] text-dim">
                    {t('dividas-dias-uteis', { n: i.business_days ?? '' })}
                  </span>
                </span>
              ) : (
                '—'
              ),
          } satisfies TechColumn<Installment>,
        ]
      : []),
    ...(hasAdjustment
      ? [
          {
            key: 'adjustment',
            header: t('dividas-correcao'),
            numeric: true,
            cell: (i: Installment) => <Money cents={i.adjustment} estimated={i.estimated} />,
          },
        ]
      : []),
    ...(hasCharges
      ? [
          {
            key: 'charges',
            header: t('dividas-seguros-taxas'),
            numeric: true,
            cell: (i: Installment) => (
              <span
                title={
                  !privacy && (i.insurance_mip || i.insurance_dfi)
                    ? t('dividas-encargos-detalhe', {
                        mip: money(i.insurance_mip),
                        dfi: money(i.insurance_dfi),
                        outros: money(i.insurance - i.insurance_mip - i.insurance_dfi),
                        taxas: money(i.fees),
                      })
                    : undefined
                }
              >
                <Money cents={i.insurance + i.fees} />
              </span>
            ),
          },
        ]
      : []),
    {
      key: 'total',
      header: t('dividas-parcela'),
      numeric: true,
      cell: (i) => (
        <span className="flex flex-col items-end">
          <Money cents={i.total} estimated={i.estimated && !i.paid} className="font-medium" />
          {i.paid && i.paid_amount != null && i.paid_amount !== i.total ? (
            <span className="text-[11px] text-dim">
              {withSlots(t('dividas-pago-valor', { valor: slot(0) }), [
                <Money key="p" cents={i.paid_amount} />,
              ])}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'balance',
      header: t('dividas-saldo-apos'),
      numeric: true,
      cell: (i) => <Money cents={i.balance_after} estimated={i.estimated} />,
    },
    {
      key: 'status',
      header: t('dividas-situacao'),
      width: '160px',
      cell: (i) => {
        const status = installmentStatus(i, next?.id ?? null);
        return (
          <span className="flex flex-col items-start gap-0.5">
            <span
              className="flex items-center gap-1.5"
              title={
                i.paid && i.paid_date
                  ? t('dividas-paga-em', { data: formatDate(i.paid_date) })
                  : undefined
              }
            >
              <InstallmentBadge status={status} />
              {i.estimated && !i.paid ? (
                <span className="label-caps italic" title={t('dividas-indice-estimado-dica')}>
                  <span aria-hidden>{'≈'}</span>
                  <span className="sr-only">{t('dividas-indice-estimado')}</span>
                </span>
              ) : null}
            </span>
            <PaymentTxn installment={i} />
          </span>
        );
      },
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('dividas-acoes')}</span>,
      align: 'right',
      width: '176px',
      cell: (i) => {
        const soon = i.due_date <= addDays(todayIso(), 45);
        const linkButton = (
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('dividas-ligar-titulo', { n: i.number })}
            onClick={() => setLinking(i)}
          >
            {t('dividas-ligar-reticencias')}
          </Button>
        );
        if (i.paid && i.txn_id)
          return (
            <span className="flex justify-end gap-1">
              <Button
                size="sm"
                variant="ghost"
                aria-label={t('dividas-desligar-parcela', { n: i.number })}
                onClick={() => setUndoing(i)}
              >
                {t('dividas-desligar')}
              </Button>
            </span>
          );
        if (i.paid)
          return (
            <span className="flex justify-end gap-1">
              {linkButton}
              <Button
                size="sm"
                variant="ghost"
                aria-label={t('dividas-desfazer-parcela', { n: i.number })}
                onClick={() => setUndoing(i)}
              >
                {t('dividas-desfazer')}
              </Button>
            </span>
          );
        return soon ? (
          <span className="flex justify-end gap-1">
            {linkButton}
            <Button
              size="sm"
              variant={i.overdue ? 'primary' : 'ghost'}
              aria-label={t('dividas-marcar-parcela-aria', { n: i.number })}
              onClick={() => setPaying(i)}
            >
              {t('dividas-marcar-paga')}
            </Button>
          </span>
        ) : null;
      },
    },
  ];

  const rowMenu = (i: Installment): ContextMenuEntry[] => {
    if (!i.id) return [];
    const to = paymentLink(i);
    return [
      { heading: t('dividas-parcela-n-data', { n: i.number, data: formatDate(i.due_date) }) },
      !i.paid && { label: t('dividas-menu-marcar-paga'), onSelect: () => setPaying(i) },
      {
        label: i.txn_id ? t('dividas-menu-ligar-outro') : t('dividas-menu-ligar-um'),
        onSelect: () => setLinking(i),
      },
      Boolean(to) && {
        label: t('dividas-menu-ver-lancamento'),
        onSelect: () => to && void navigate(to),
      },
      { separator: true },
      i.paid && i.txn_id
        ? { label: t('dividas-desligar-lancamento'), onSelect: () => setUndoing(i) }
        : i.paid && { label: t('dividas-menu-desfazer'), onSelect: () => setUndoing(i) },
    ];
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Segmented<Filter>
          size="sm"
          label={t('dividas-filtro-aria')}
          value={filter}
          onChange={setFilter}
          options={[
            { value: 'proximas', label: t('dividas-filtro-proximas') },
            {
              value: 'abertas',
              label: t('dividas-filtro-abertas', {
                n: installments.filter((i) => !i.paid).length,
              }),
            },
            {
              value: 'pagas',
              label: t('dividas-filtro-pagas', { n: installments.filter((i) => i.paid).length }),
            },
            { value: 'todas', label: t('dividas-filtro-todas', { n: installments.length }) },
          ]}
        />
        {overdue > 0 ? (
          <span className="label-caps text-negative">
            {'! '}
            {t('dividas-parcelas-atrasadas', { n: overdue })}
          </span>
        ) : null}
      </div>
      <TechTable<Installment>
        label={t('dividas-tabela-de', { nome: loan.name })}
        prefsId={`Tabela de amortização · ${loan.name}`}
        dense
        maxHeight="460px"
        rows={rows}
        getRowId={(i) => i.id || String(i.number)}
        defaultSelectedId={next?.id ?? null}
        columns={columns}
        rowMenu={rowMenu}
        rowMenuLabel={(i) => t('dividas-acoes-parcela', { n: i.number })}
        empty={t('dividas-filtro-vazio')}
        footer={{
          number: 'Σ',
          amortization: <Money cents={sum.amortization} />,
          interest: <Money cents={sum.interest} />,
          adjustment: <Money cents={sum.adjustment} />,
          charges: <Money cents={sum.charges} />,
          total: <Money cents={sum.total} />,
        }}
      />
      {paying ? (
        <MarkPaidDialog loanId={loan.id} installment={paying} onClose={() => setPaying(null)} />
      ) : null}
      {linking ? (
        <InstallmentLinkDialog loan={loan} installment={linking} onClose={() => setLinking(null)} />
      ) : null}
      <UnmarkDialog installment={undoing} onClose={() => setUndoing(null)} />
    </div>
  );
});

function MarkPaidDialog({
  loanId,
  installment,
  onClose,
}: {
  loanId: string;
  installment: Installment;
  onClose: () => void;
}) {
  const mark = useMarkInstallmentPaid();
  const record = useRecordLoanPayment();
  const undoToast = useUndoToast();
  const today = todayIso();
  // Quase sempre se paga no vencimento: já vem com ele (se ainda não venceu, hoje).
  const [date, setDate] = useState<string | null>(
    installment.due_date <= today ? installment.due_date : today,
  );
  const [amount, setAmount] = useState<number | null>(installment.total);
  // Saldo devedor depois do pagamento, como no comprovante do banco: sincroniza o saldo
  // e recalcula as parcelas seguintes (amortização extra abre uma nova versão).
  const [balance, setBalance] = useState<number | null>(null);
  const [amort, setAmort] = useState<number | null>(null);
  const [term, setTerm] = useState('');
  const pending = mark.isPending || record.isPending;
  const done = () => {
    undoToast({
      tone: 'success',
      title:
        balance != null
          ? t('dividas-paga-sincronizada', { n: installment.number })
          : t('dividas-marcada-paga', { n: installment.number }),
    });
    onClose();
  };
  const submit = () => {
    if (balance != null) {
      const n = Number(term);
      record.mutate(
        {
          id: loanId,
          installment_id: installment.id,
          date: date!,
          ...(amount != null ? { amount } : {}),
          balance_after: balance,
          ...(amort != null ? { amortization: amort } : {}),
          ...(term.trim() && Number.isInteger(n) && n >= 0 ? { remaining_term: n } : {}),
        },
        { onSuccess: done },
      );
      return;
    }
    mark.mutate(
      {
        installment_id: installment.id,
        date: date!,
        ...(amount != null && amount !== installment.total ? { amount } : {}),
      },
      { onSuccess: done },
    );
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => (!o ? onClose() : undefined)}
      size="sm"
      title={t('dividas-pagar-titulo', { n: installment.number })}
      description={t('dividas-pagar-descricao', { data: formatDate(installment.due_date) })}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={pending}
            disabled={!date || amount == null}
            onClick={submit}
          >
            {t('dividas-marcar-como-paga')}
          </Button>
        </>
      }
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('dividas-data-pagamento')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('dividas-valor-pago')} hint={t('dividas-valor-pago-dica')}>
          <MoneyInput value={amount} onValueChange={setAmount} />
        </Field>
        <Field label={t('dividas-saldo-depois')} hint={t('dividas-saldo-depois-dica')}>
          <MoneyInput value={balance} onValueChange={setBalance} />
        </Field>
        <Field
          label={t('dividas-amortizacao-parcela')}
          hint={t('dividas-amortizacao-parcela-dica')}
        >
          <MoneyInput value={amort} onValueChange={setAmort} disabled={balance == null} />
        </Field>
        <Field label={t('dividas-parcelas-que-faltam')} hint={t('dividas-faltam-dica')}>
          <Input
            numeric
            inputMode="numeric"
            value={term}
            onChange={(e) => setTerm(e.target.value.replace(/\D/g, ''))}
            disabled={balance == null}
          />
        </Field>
      </div>
      <InlineError error={mark.error ?? record.error} />
    </Dialog>
  );
}

function UnmarkDialog({
  installment,
  onClose,
}: {
  installment: Installment | null;
  onClose: () => void;
}) {
  const unmark = useUnmarkInstallmentPaid();
  const undoToast = useUndoToast();
  const linked = installment?.txn_id != null;
  const txn = installment ? txnText(installment) : '';
  const n = installment?.number ?? '';
  return (
    <ConfirmDialog
      open={installment != null}
      onOpenChange={(o) => (!o ? onClose() : undefined)}
      title={linked ? t('dividas-desligar-parcela', { n }) : t('dividas-desfazer-parcela', { n })}
      description={
        linked
          ? t('dividas-desligar-descricao', { lancamento: txn })
          : t('dividas-desfazer-descricao')
      }
      confirmLabel={linked ? t('dividas-desligar-lancamento') : t('dividas-desfazer-pagamento')}
      tone="primary"
      pending={unmark.isPending}
      error={unmark.error}
      onConfirm={() =>
        installment &&
        unmark.mutate(
          { id: installment.id },
          {
            onSuccess: () => {
              undoToast({
                title: linked
                  ? t('dividas-desligada', { n: installment.number })
                  : t('dividas-em-aberto-de-novo', { n: installment.number }),
              });
              onClose();
            },
          },
        )
      }
    />
  );
}
