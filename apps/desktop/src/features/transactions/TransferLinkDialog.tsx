/**
 * Liga um lançamento solto a outra conta da família (`transactions.update` com
 * `transfer_account_id`): usa a perna que já existe lá (mesmo valor, até 4 dias) ou cria essa
 * perna. Antes de gravar, mostra o que vai acontecer (`transactions.transfer_match`): o par
 * encontrado, se vira transferência, pagamento de fatura ou aporte, e a fatura quitada.
 * `mode="card"` é o "Pagamento de fatura": só os cartões como destino.
 */
import { useState } from 'react';

import type { TransferMatch } from '@/api/generated/TransferMatch';
import type { TxnView } from '@/api/generated/TxnView';
import { useAccounts } from '@/api/hooks/accounts';
import { useCore } from '@/api/hooks/coreQuery';
import { useUpdateTxn } from '@/api/hooks/transactions';
import { Button, Dialog, Field, Money, MoneyInput, Select, toast } from '@/design/components';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { InlineError } from './ui/controls';
import { slot, withSlots } from './ui/slots';

export type TransferLinkMode = 'transfer' | 'card';

const NONE = '__nenhuma__';

export function TransferLinkDialog({
  txn,
  mode,
  open,
  onOpenChange,
  onDone,
}: {
  txn: TxnView;
  mode: TransferLinkMode;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Depois de ligar: o rótulo da ação (para o "Desfazer" da tela). */
  onDone?: (label: string) => void;
}) {
  const { data: accounts = [] } = useAccounts();
  const options = accounts.filter(
    (a) =>
      a.id !== txn.account_id && !a.archived && (mode === 'card' ? a.kind === 'credit_card' : true),
  );
  const [accountId, setAccountId] = useState<string | null>(null);
  const [otherAmount, setOtherAmount] = useState<number | null>(null);
  const [lastOpen, setLastOpen] = useState(open);
  if (open !== lastOpen) {
    setLastOpen(open);
    if (open) {
      setAccountId(null);
      setOtherAmount(null);
    }
  }
  // Um cartão só: já escolhido.
  const chosen = accountId ?? (mode === 'card' && options.length === 1 ? options[0]!.id : null);
  const account = options.find((a) => a.id === chosen);
  const crossCurrency = Boolean(account && account.currency !== txn.currency);
  const match = useCore<TransferMatch>(
    'transactions',
    'transfer_match',
    {
      txn_id: txn.id,
      account_id: chosen,
      ...(crossCurrency && otherAmount ? { other_amount: otherAmount } : {}),
    },
    { enabled: open && Boolean(chosen) },
  );
  const update = useUpdateTxn();
  const out = txn.amount < 0;
  const m = match.data;
  const ready = Boolean(chosen && m && m.amount != null);
  const title =
    mode === 'card'
      ? t('lancamentos-pagamento-fatura')
      : out
        ? t('lancamentos-transferencia-para')
        : t('lancamentos-transferencia-de');
  const accountLabel =
    mode === 'card'
      ? t('lancamentos-cartao')
      : out
        ? t('lancamentos-para-a-conta')
        : t('lancamentos-da-conta');
  const submit = () => {
    if (!chosen) return;
    update.mutate(
      {
        id: txn.id,
        transfer_account_id: chosen,
        ...(crossCurrency && otherAmount ? { transfer_amount: otherAmount } : {}),
      },
      {
        onSuccess: () => {
          const label =
            m?.kind === 'card_payment'
              ? t('lancamentos-fatura-ligada')
              : t('lancamentos-transferencia-ligada');
          toast({
            tone: 'success',
            title: label,
            description: m?.existing
              ? t('lancamentos-ligado-a', {
                  descricao: m.existing.description,
                  conta: m.existing.account_name,
                })
              : t('lancamentos-criado-em', {
                  conta: account?.name ?? t('lancamentos-outra-conta'),
                }),
          });
          onDone?.(label);
          onOpenChange(false);
        },
      },
    );
  };
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="sm"
      title={title}
      description={`${formatDate(txn.date)} · ${txn.description || t('lancamentos-sem-descricao')} · ${txn.account_name}`}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" loading={update.isPending} disabled={!ready} onClick={submit}>
            {mode === 'card' ? t('lancamentos-ligar-cartao') : t('lancamentos-ligar-contas')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <p className="text-[13px]">
          <Money cents={txn.amount} currency={txn.currency} tone="auto" />{' '}
          <span className="text-dim">
            {out
              ? t('lancamentos-saiu-em', { conta: txn.account_name })
              : t('lancamentos-entrou-em', { conta: txn.account_name })}
          </span>
        </p>
        {options.length === 0 ? (
          <p className="text-[13px] text-dim">
            {mode === 'card'
              ? t('lancamentos-nenhum-cartao')
              : t('lancamentos-nenhuma-outra-conta')}
          </p>
        ) : (
          <Field label={accountLabel}>
            <Select
              aria-label={accountLabel}
              value={chosen ?? NONE}
              onValueChange={(v) => {
                setAccountId(v === NONE ? null : v);
                setOtherAmount(null);
              }}
              options={[
                ...(chosen ? [] : [{ value: NONE, label: t('lancamentos-escolha-conta') }]),
                ...options.map((a) => ({ value: a.id, label: a.name, hint: a.currency })),
              ]}
            />
          </Field>
        )}
        {crossCurrency && account ? (
          <Field
            label={t('lancamentos-valor-em', { conta: account.name, moeda: account.currency })}
          >
            <MoneyInput
              currency={account.currency}
              value={otherAmount}
              onValueChange={setOtherAmount}
            />
          </Field>
        ) : null}
        {chosen && account ? (
          <div
            role="status"
            aria-label={t('lancamentos-o-que-acontece')}
            className="flex flex-col gap-1.5 text-[13px]"
          >
            {match.isPending ? (
              <p className="text-dim">{t('lancamentos-procurando', { conta: account.name })}</p>
            ) : m ? (
              <>
                {m.existing ? (
                  <p>
                    {t(out ? 'lancamentos-achei-entrada' : 'lancamentos-achei-saida', {
                      conta: account.name,
                      data: formatDate(m.existing.date),
                      descricao: m.existing.description,
                    })}
                  </p>
                ) : m.amount != null ? (
                  <p>
                    {withSlots(
                      t(out ? 'lancamentos-crio-entrada' : 'lancamentos-crio-saida', {
                        valor: slot(0),
                        conta: account.name,
                        data: formatDate(txn.date),
                      }),
                      [<Money key="valor" cents={Math.abs(m.amount)} currency={m.currency} />],
                    )}
                  </p>
                ) : (
                  <p className="text-dim">
                    {out
                      ? t('lancamentos-informe-entrou', { conta: account.name })
                      : t('lancamentos-informe-saiu', { conta: account.name })}
                  </p>
                )}
                <p className="text-dim">
                  {m.kind === 'card_payment'
                    ? m.statement_due_date
                      ? `${t('lancamentos-vira-fatura')} ${t('lancamentos-quita-fatura', { data: formatDate(m.statement_due_date) })}`
                      : t('lancamentos-vira-fatura')
                    : m.kind === 'investment'
                      ? t('lancamentos-vira-aporte')
                      : t('lancamentos-vira-transferencia')}
                </p>
              </>
            ) : null}
          </div>
        ) : null}
      </div>
      <InlineError error={match.error ?? update.error} />
    </Dialog>
  );
}
