/**
 * Closing an account (accounts/closing.rs in the core): the family closed it at the bank or the
 * broker. The dialog asks for the date, shows the balance on it and what stops the closing (a
 * balance left, something recorded after the date, assets still held, card installments to
 * come), with shortcuts to transfer the balance or record an adjustment. When nothing stops it,
 * it lists what the closing does (recurrences paused, planned items removed) and what to change
 * elsewhere (loans and cards paid from the account).
 */
import { useState } from 'react';

import type { AccountCloseBlocker } from '@/api/generated/AccountCloseBlocker';
import type { AccountView } from '@/api/generated/AccountView';
import { useCloseAccount, useClosePreview } from '@/api/hooks/accounts';
import { Button, DateInput, Dialog, Field, Measure, Money, toast } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatMoney, todayIso } from '@/lib/format';

export function blockerText(b: AccountCloseBlocker): string {
  switch (b.kind) {
    case 'future_date':
      return t('contas-encerrar-bloqueio-futuro');
    case 'before_opening':
      return t('contas-encerrar-bloqueio-abertura', { data: formatDate(b.opening_date) });
    case 'already_closed':
      return t('contas-encerrar-bloqueio-encerrada', { data: formatDate(b.closed_on) });
    case 'non_zero_balance':
      return t('contas-encerrar-bloqueio-saldo', { valor: formatMoney(b.balance, b.currency) });
    case 'activity_after':
      return t('contas-encerrar-bloqueio-depois', {
        n: b.count,
        data: formatDate(b.last_date),
      });
    case 'open_positions':
      return t('contas-encerrar-bloqueio-posicoes', { ativos: b.assets.join(', ') });
    case 'pending_installments':
      return t('contas-encerrar-bloqueio-parcelas', { n: b.count });
  }
}

export function CloseAccountDialog({
  account,
  open,
  onOpenChange,
  onTransfer,
  onAdjust,
}: {
  account: AccountView | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Opens a transfer from this account (to move the balance left). */
  onTransfer: () => void;
  /** Opens a balance adjustment in this account. */
  onAdjust: () => void;
}) {
  const [date, setDate] = useState<string | null>(todayIso());
  const preview = useClosePreview(open && account && date ? { id: account.id, date } : null);
  const close = useCloseAccount((_, args) => {
    toast({
      tone: 'success',
      title: t('contas-conta-encerrada'),
      description: t('contas-encerrada-em-nome', {
        nome: account?.name ?? '',
        data: formatDate(args.date ?? todayIso()),
      }),
    });
    onOpenChange(false);
  });
  // While the preview reloads (a date typed, a transfer just made), nothing old shows.
  const p = preview.isFetching ? undefined : preview.data;
  const blocked = !p || p.blockers.length > 0;
  const hasBalance = p?.blockers.some((b) => b.kind === 'non_zero_balance') ?? false;
  const isCard = account?.kind === 'credit_card';

  const confirm = () => {
    if (account && date) close.mutate({ id: account.id, date });
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) close.reset();
        onOpenChange(o);
      }}
      index="03"
      size="md"
      title={account ? t('contas-encerrar-titulo', { nome: account.name }) : ''}
      description={t('contas-encerrar-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="danger" disabled={blocked} loading={close.isPending} onClick={confirm}>
            {t('contas-encerrar-confirmar')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label={t('contas-encerrar-data')} hint={t('contas-encerrar-data-dica')}>
            <DateInput value={date} onValueChange={setDate} />
          </Field>
          {p && account ? (
            <Measure
              size="md"
              label={t('contas-encerrar-saldo', { data: formatDate(p.date) })}
              value={
                <span className="text-[20px]">
                  <Money cents={p.balance} currency={p.currency} />
                </span>
              }
            />
          ) : null}
        </div>

        <InlineError error={preview.error} />

        {p && p.blockers.length > 0 ? (
          <section
            aria-label={t('contas-encerrar-bloqueios')}
            className="flex flex-col gap-2 rounded-card border border-warning/40 p-3"
          >
            <h3 className="label-caps text-warning">{t('contas-encerrar-bloqueios')}</h3>
            <ul className="flex list-disc flex-col gap-1 pl-5 text-[13px] leading-relaxed">
              {p.blockers.map((b) => (
                <li key={b.kind}>{blockerText(b)}</li>
              ))}
            </ul>
            {hasBalance ? (
              <div className="flex flex-wrap gap-2 pt-1">
                {!isCard ? (
                  <Button size="sm" onClick={onTransfer}>
                    {t('contas-encerrar-transferir')}
                  </Button>
                ) : null}
                <Button size="sm" variant="ghost" onClick={onAdjust}>
                  {t('contas-encerrar-ajuste')}
                </Button>
              </div>
            ) : null}
          </section>
        ) : null}

        {p && p.blockers.length === 0 ? (
          <section
            aria-label={t('contas-encerrar-efeitos')}
            className="flex flex-col gap-1 text-[13px] leading-relaxed"
          >
            <p>{t('contas-encerrar-pronto')}</p>
            <ul className="flex list-disc flex-col gap-1 pl-5 text-dim">
              <li>{t('contas-encerrar-sem-avisos')}</li>
              {p.recurrences.length > 0 ? (
                <li>
                  {t('contas-encerrar-recorrencias', {
                    n: p.recurrences.length,
                    nomes: p.recurrences.map((r) => r.name).join(', '),
                  })}
                </li>
              ) : null}
              {p.planned.length > 0 ? (
                <li>{t('contas-encerrar-previstos', { n: p.planned.length })}</li>
              ) : null}
            </ul>
          </section>
        ) : null}

        {p && (p.loans.length > 0 || p.cards_paid.length > 0) ? (
          <ul className="flex list-disc flex-col gap-1 pl-5 text-[13px] leading-relaxed text-warning">
            {p.loans.length > 0 ? (
              <li>
                {t('contas-encerrar-emprestimos', {
                  nomes: p.loans.map((l) => l.name).join(', '),
                })}
              </li>
            ) : null}
            {p.cards_paid.length > 0 ? (
              <li>
                {t('contas-encerrar-cartoes', {
                  nomes: p.cards_paid.map((c) => c.name).join(', '),
                })}
              </li>
            ) : null}
          </ul>
        ) : null}

        <InlineError error={close.error} />
      </div>
    </Dialog>
  );
}
