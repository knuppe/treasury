/**
 * Sincronia de saldo: quando o extrato não confere e o motivo é desconhecido, lança a
 * diferença como ajuste (fora de receitas e despesas) e guarda o marco. Depois, as causas
 * prováveis ajudam a achar o motivo, que pode ser anotado ou investigado pelo agente.
 */
import { useState } from 'react';
import { Link } from 'react-router';

import type { AccountView } from '@/api/generated/AccountView';
import type { BalanceSync } from '@/api/generated/BalanceSync';
import {
  useBalanceSyncs,
  useResolveSync,
  useSyncBalance,
  useSyncHints,
} from '@/api/hooks/accounts';
import { Button, Field, Input, Money, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import { useCompanion } from '@/state/companion';
import { InlineError } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';

function statusLabel(status: BalanceSync['status']): string {
  switch (status) {
    case 'open':
      return t('contas-sincronia-sem-motivo');
    case 'explained':
      return t('contas-sincronia-explicada');
    case 'ignored':
      return t('contas-sincronia-aceita');
  }
}

/** Botão "Sincronizar com o banco" para a diferença que não confere. */
export function SyncDifference({
  account,
  date,
  balance,
  onDone,
}: {
  account: AccountView;
  date: string;
  balance: number;
  onDone: (s: BalanceSync) => void;
}) {
  const sync = useSyncBalance();
  const [note, setNote] = useState('');
  return (
    <div className="flex flex-col gap-2 rounded-card border border-line p-3">
      <p className="text-[13px] text-fg">
        {withSlots(t('contas-sincronizar-explicacao', { ajuste: slot(0) }), [
          <strong>{t('contas-ajuste-destaque')}</strong>,
        ])}
      </p>
      <div className="flex flex-wrap items-end gap-2">
        <Field label={t('contas-observacao-opcional')} className="min-w-64 flex-1">
          <Input
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder={t('contas-observacao-exemplo')}
          />
        </Field>
        <Button
          variant="primary"
          loading={sync.isPending}
          onClick={() =>
            sync.mutate(
              {
                account_id: account.id,
                date,
                balance,
                ...(note.trim() ? { note: note.trim() } : {}),
              },
              {
                onSuccess: (s) => {
                  toast({
                    tone: 'success',
                    title: t('contas-saldo-sincronizado'),
                    description: t('contas-ajuste-lancado', { data: formatDate(s.date) }),
                  });
                  onDone(s);
                },
              },
            )
          }
        >
          {t('contas-sincronizar')}
        </Button>
      </div>
      <InlineError error={sync.error} />
    </div>
  );
}

/** Causas prováveis, o motivo e a investigação pelo agente de um marco. */
export function SyncReview({ sync }: { sync: BalanceSync }) {
  const hints = useSyncHints(sync.status === 'open' ? sync.id : null);
  const resolve = useResolveSync();
  const ask = useCompanion((s) => s.ask);
  const [why, setWhy] = useState('');
  const period = txnLink({
    account_ids: [sync.account_id],
    ...(hints.data?.since ? { date_from: hints.data.since } : {}),
    date_to: sync.date,
  });
  return (
    <div className="flex flex-col gap-3 rounded-card border border-warning/50 bg-warning/5 p-3">
      <p className="text-[13px] text-fg">
        {withSlots(
          t('contas-ajuste-resumo', {
            diferenca: slot(0),
            data: formatDate(sync.date),
            banco: slot(1),
            treasury: slot(2),
          }),
          [
            <Money cents={sync.difference} currency={sync.currency} signed tone="auto" />,
            <Money cents={sync.statement_balance} currency={sync.currency} />,
            <Money cents={sync.computed_balance} currency={sync.currency} />,
          ],
        )}
      </p>
      {hints.data?.hints.length ? (
        <ul
          aria-label={t('contas-causas-provaveis')}
          className="flex flex-col gap-1 text-[12.5px] text-fg"
        >
          {hints.data.hints.map((h) => (
            <li key={h.kind} className="flex gap-2">
              <span aria-hidden className="text-warning">
                {'•'}
              </span>
              <span>{h.message}</span>
            </li>
          ))}
        </ul>
      ) : null}
      <div className="flex flex-wrap items-center gap-2">
        <Link className="text-[12.5px] text-accent underline" to={period}>
          {t('contas-ver-lancamentos-periodo')}
        </Link>
        <Button
          size="sm"
          variant="secondary"
          onClick={() =>
            ask(
              t('contas-pedido-agente', {
                conta: sync.account_name,
                data: formatDate(sync.date),
                id: sync.id,
                banco: (sync.statement_balance / 100).toFixed(2),
                treasury: (sync.computed_balance / 100).toFixed(2),
              }),
            )
          }
        >
          {t('contas-pedir-agente')}
        </Button>
      </div>
      <div className="flex flex-wrap items-end gap-2">
        <Field label={t('contas-motivo')} className="min-w-64 flex-1">
          <Input
            value={why}
            onChange={(e) => setWhy(e.target.value)}
            placeholder={t('contas-motivo-exemplo')}
          />
        </Field>
        <Button
          size="sm"
          variant="primary"
          disabled={!why.trim()}
          loading={resolve.isPending && resolve.variables?.status === 'explained'}
          onClick={() =>
            resolve.mutate({ id: sync.id, status: 'explained', explanation: why.trim() })
          }
        >
          {t('contas-anotar-motivo')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          loading={resolve.isPending && resolve.variables?.status === 'ignored'}
          onClick={() => resolve.mutate({ id: sync.id, status: 'ignored' })}
        >
          {t('contas-aceitar-sem-investigar')}
        </Button>
      </div>
      <InlineError error={hints.error ?? resolve.error} />
    </div>
  );
}

/** Histórico dos marcos da conta, com o que ainda está sem motivo para revisar. */
export function SyncHistory({ account }: { account: AccountView }) {
  const q = useBalanceSyncs(account.id);
  const list = q.data ?? [];
  const [openId, setOpenId] = useState<string | null>(null);
  if (!list.length) return null;
  return (
    <section aria-label={t('contas-sincronias')} className="flex flex-col gap-2">
      <h3 className="label-caps">{t('contas-sincronias')}</h3>
      <ul className="flex flex-col">
        {list.map((s) => (
          <li key={s.id} className="flex flex-col gap-2 border-t border-line py-2">
            <div className="flex flex-wrap items-center gap-3 text-[13px]">
              <span className="num w-24 text-dim">{formatDate(s.date)}</span>
              <span className="min-w-0 flex-1">
                {s.difference === 0
                  ? t('contas-conferia')
                  : withSlots(t('contas-ajuste-de', { valor: slot(0) }), [
                      <Money cents={s.difference} currency={s.currency} signed tone="auto" />,
                    ])}
                {s.explanation ? <span className="text-dim">{` · ${s.explanation}`}</span> : null}
              </span>
              <span className={cx('label-caps', s.status === 'open' ? 'text-warning' : 'text-dim')}>
                {statusLabel(s.status)}
              </span>
              {s.status === 'open' ? (
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setOpenId(openId === s.id ? null : s.id)}
                >
                  {openId === s.id ? t('comum-fechar') : t('contas-investigar')}
                </Button>
              ) : null}
            </div>
            {openId === s.id ? <SyncReview sync={s} /> : null}
          </li>
        ))}
      </ul>
    </section>
  );
}
