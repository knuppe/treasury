/**
 * DARFs do ano: período, código, valor (copiável), vencimento e situação, com marcar como
 * pago (opcionalmente lançando a despesa de IR numa conta) e cancelar. O app não emite a
 * guia: os valores vão para o Sicalc da Receita.
 */
import { useState } from 'react';
import { Link } from 'react-router';
import { useUndoToast } from '@/api/hooks/undoToast';

import { useAccounts } from '@/api/hooks/accounts';
import { useCancelDarf, useDarfCharges, useMarkDarfPaid } from '@/api/hooks/tax';
import type { Darf } from '@/api/generated/Darf';
import { Button, DateInput, Dialog, Field, Money, Select, TechTable } from '@/design/components';
import { cx } from '@/design/cx';
import { ConfirmDialog, InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatMoney, todayIso } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';

import { CopyButton } from './CopyValue';
import { DARF_STATUS_LABEL, periodLabel } from './labels';
import { copyText, darfOverdue } from './transform';

const NONE = '__none__';

function DarfBadge({ darf, today }: { darf: Darf; today: string }) {
  const overdue = darfOverdue(darf, today);
  return (
    <span
      className={cx(
        'label-caps inline-flex h-5 items-center gap-1 rounded-control border px-1.5 whitespace-nowrap',
        darf.status === 'paid'
          ? 'border-positive/50 text-positive'
          : darf.status === 'cancelled'
            ? 'border-line text-dim line-through'
            : overdue
              ? 'border-negative/60 text-negative'
              : 'border-accent/60 text-accent',
      )}
    >
      {darf.status === 'paid' ? (
        <span aria-hidden>{'✓'}</span>
      ) : overdue ? (
        <span aria-hidden>!</span>
      ) : null}
      {overdue ? t('ir-darf-vencido') : DARF_STATUS_LABEL[darf.status]}
    </span>
  );
}

export function DarfPanel({ darfs }: { darfs: readonly Darf[] }) {
  const today = todayIso();
  const [paying, setPaying] = useState<Darf | null>(null);
  const [cancelling, setCancelling] = useState<Darf | null>(null);
  const cancel = useCancelDarf();
  const undoToast = useUndoToast();
  const overdue = darfs.filter((d) => darfOverdue(d, today));
  return (
    <div className="flex flex-col gap-3">
      {overdue.length > 0 ? (
        <p role="status" className="text-[12.5px] text-negative">
          {t('ir-darfs-venceram', { n: overdue.length })}
        </p>
      ) : null}
      <TechTable<Darf>
        label={t('ir-secao-darf')}
        prefsId="DARFs"
        dense
        rows={darfs}
        getRowId={(d) => d.id}
        empty={t('ir-darfs-vazio')}
        columns={[
          {
            key: 'period',
            header: t('ir-periodo'),
            cell: (d) => <span className="num">{periodLabel(d.period)}</span>,
          },
          {
            key: 'code',
            header: t('ir-codigo'),
            cell: (d) => <span className="num">{d.revenue_code}</span>,
          },
          {
            key: 'total',
            header: t('ir-valor'),
            numeric: true,
            cell: (d) => (
              <span className="inline-flex flex-col items-end gap-0.5">
                <span className="inline-flex items-center gap-2">
                  <Money cents={d.total} />
                  {d.status === 'pending' ? (
                    <CopyButton
                      text={copyText(d.total)}
                      label={t('ir-valor-darf-de', { periodo: d.period })}
                    />
                  ) : null}
                </span>
                {d.status === 'pending' && d.due_date < today ? <LateCharges darf={d} /> : null}
                {d.status === 'paid' && d.total > d.principal ? (
                  <span className="text-[11.5px] text-dim">
                    {t('ir-darf-com-acrescimos', {
                      multa: formatMoney(d.fine, 'BRL'),
                      juros: formatMoney(d.interest, 'BRL'),
                    })}
                  </span>
                ) : null}
              </span>
            ),
          },
          {
            key: 'due',
            header: t('ir-vencimento'),
            cell: (d) => <span className="num">{formatDate(d.due_date)}</span>,
          },
          {
            key: 'status',
            header: t('ir-situacao'),
            cell: (d) => (
              <span className="flex items-center gap-2">
                <DarfBadge darf={d} today={today} />
                {d.paid_at ? (
                  <span className="num text-[11.5px] text-dim">{formatDate(d.paid_at)}</span>
                ) : null}
              </span>
            ),
          },
          {
            key: 'actions',
            header: <span className="sr-only">{t('ir-acoes')}</span>,
            align: 'right',
            cell: (d) =>
              d.status === 'pending' ? (
                <span className="flex justify-end gap-1">
                  <Button size="sm" variant="ghost" onClick={() => setCancelling(d)}>
                    {t('ir-cancelar')}
                  </Button>
                  <Button
                    size="sm"
                    variant="primary"
                    aria-label={t('ir-marcar-pago-de', { periodo: d.period })}
                    onClick={() => setPaying(d)}
                  >
                    {t('ir-marcar-pago')}
                  </Button>
                </span>
              ) : d.txn_id && d.paid_at ? (
                <Link
                  className="label-caps text-accent hover:underline"
                  to={txnLink({
                    date_from: d.paid_at,
                    date_to: d.paid_at,
                    text: `DARF ${d.revenue_code}`,
                  })}
                >
                  {t('ir-lancamento')}
                </Link>
              ) : null,
          },
        ]}
      />
      {paying ? <PayDialog darf={paying} onClose={() => setPaying(null)} /> : null}
      <ConfirmDialog
        open={cancelling != null}
        onOpenChange={(o) => (!o ? setCancelling(null) : undefined)}
        title={t('ir-cancelar-darf')}
        description={t('ir-cancelar-darf-texto', { periodo: cancelling?.period ?? '' })}
        confirmLabel={t('ir-cancelar-darf')}
        pending={cancel.isPending}
        error={cancel.error}
        onConfirm={() =>
          cancelling &&
          cancel.mutate(
            { id: cancelling.id },
            {
              onSuccess: () => {
                undoToast({ title: t('ir-darf-cancelado-toast') });
                setCancelling(null);
              },
            },
          )
        }
      />
    </div>
  );
}

function PayDialog({ darf, onClose }: { darf: Darf; onClose: () => void }) {
  const today = todayIso();
  const [date, setDate] = useState<string | null>(darf.due_date < today ? today : darf.due_date);
  const [account, setAccount] = useState<string>(NONE);
  const accounts = (useAccounts().data ?? []).filter(
    (a) => !a.archived && (a.kind === 'checking' || a.kind === 'savings'),
  );
  const pay = useMarkDarfPaid();
  const undoToast = useUndoToast();
  return (
    <Dialog
      open
      onOpenChange={(o) => (!o ? onClose() : undefined)}
      size="sm"
      title={t('ir-darf-pago-titulo', { periodo: darf.period })}
      description={t('ir-darf-pago-descricao', {
        codigo: darf.revenue_code,
        vencimento: formatDate(darf.due_date),
      })}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-voltar')}
          </Button>
          <Button
            variant="primary"
            loading={pay.isPending}
            disabled={!date}
            onClick={() =>
              pay.mutate(
                {
                  id: darf.id,
                  paid_at: date!,
                  ...(account !== NONE ? { from_account_id: account } : {}),
                },
                {
                  onSuccess: () => {
                    undoToast({
                      tone: 'success',
                      title: t('ir-darf-pago'),
                      description: account !== NONE ? t('ir-despesa-lancada') : undefined,
                    });
                    onClose();
                  },
                },
              )
            }
          >
            {t('ir-marcar-como-pago')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="flex items-baseline justify-between rounded-control border border-line bg-bg px-3 py-2">
          <span className="label-caps">{t('ir-valor')}</span>
          <Money cents={darf.total} className="text-[15px] font-medium" />
        </div>
        <Field label={t('ir-data-pagamento')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('ir-lancar-despesa')} hint={t('ir-lancar-despesa-dica')}>
          <Select
            value={account}
            onValueChange={(v) => v && setAccount(v)}
            options={[
              { value: NONE, label: t('ir-nao-lancar') },
              ...accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency })),
            ]}
          />
        </Field>
        {darf.due_date < (date ?? today) ? (
          <p className="text-[12.5px] text-warning">{t('ir-pago-atrasado')}</p>
        ) : null}
        <InlineError error={pay.error} />
      </div>
    </Dialog>
  );
}

/** Multa e juros de um DARF vencido, se pago hoje (Lei 9.430/1996 art. 61). */
function LateCharges({ darf }: { darf: Darf }) {
  const q = useDarfCharges(darf.id);
  if (!q.data || q.data.total <= darf.principal) return null;
  return (
    <span className="text-[11.5px] text-negative">
      {t(q.data.estimated ? 'ir-darf-atraso-hoje-estimado' : 'ir-darf-atraso-hoje', {
        total: formatMoney(q.data.total, 'BRL'),
        multa: formatMoney(q.data.fine, 'BRL'),
        juros: formatMoney(q.data.interest, 'BRL'),
      })}
    </span>
  );
}
