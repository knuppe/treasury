/**
 * Ajuste de saldo (um tipo de lançamento): a pessoa informa o saldo que a conta tinha numa data
 * e o Treasury lança a diferença como ajuste, fora de receitas, despesas e orçamento, e guarda o
 * marco de sincronia (`accounts.sync_balance`). Serve para quem não lançou tudo ou tem um trecho
 * do histórico sem informação. A prévia mostra o saldo calculado na data e o ajuste antes de
 * gravar.
 */
import { useState, type FormEvent } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import type { BalanceSync } from '@/api/generated/BalanceSync';
import { useBalanceHistory, useSyncBalance } from '@/api/hooks/accounts';
import {
  DateInput,
  Field,
  Input,
  Measure,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

export function BalanceAdjustForm({
  formId,
  accounts,
  accountId,
  onAccountChange,
  onDone,
}: {
  /** Id do `<form>`, para o botão de gravar do rodapé da janela. */
  formId: string;
  accounts: readonly AccountView[];
  accountId: string;
  onAccountChange: (id: string) => void;
  onDone: (sync: BalanceSync) => void;
}) {
  const account = accounts.find((a) => a.id === accountId) ?? null;
  const [date, setDate] = useState<string | null>(todayIso());
  const [balance, setBalance] = useState<number | null>(null);
  const [note, setNote] = useState('');
  const sync = useSyncBalance();
  const history = useBalanceHistory(
    account && date ? { id: account.id, from: date, to: date } : null,
  );
  const computed = history.data?.at(-1)?.balance ?? null;
  const diff = balance != null && computed != null ? balance - computed : null;
  const base = useBaseCurrency();
  const currency = account?.currency ?? base;

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!account || !date || balance == null) return;
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
            title:
              s.difference === 0
                ? t('contas-ajuste-sem-diferenca')
                : t('contas-saldo-sincronizado'),
            description: t('contas-ajuste-lancado', { data: formatDate(s.date) }),
          });
          onDone(s);
        },
      },
    );
  };

  return (
    <form id={formId} className="flex flex-col gap-4" onSubmit={submit} noValidate>
      <p className="max-w-[70ch] text-[13px] leading-relaxed text-dim">
        {t('contas-ajuste-explicacao')}
      </p>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('contas-ajuste-conta')}>
          <Select
            value={accountId || undefined}
            placeholder={t('lancamentos-escolha-conta')}
            onValueChange={onAccountChange}
            options={accounts
              .filter((a) => !a.archived || a.id === accountId)
              .map((a) => ({ value: a.id, label: a.name, hint: a.currency }))}
          />
        </Field>
        <Field label={t('contas-ajuste-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field
          label={t('contas-ajuste-saldo', { moeda: currency })}
          hint={t('contas-ajuste-saldo-dica')}
        >
          <MoneyInput
            currency={currency}
            allowNegative
            value={balance}
            onValueChange={setBalance}
            autoFocus
          />
        </Field>
        <Field label={t('contas-observacao-opcional')}>
          <Input
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder={t('contas-observacao-exemplo')}
          />
        </Field>
      </div>
      {account && date ? (
        <div
          role="status"
          aria-label={t('contas-ajuste-previa')}
          className="grid gap-4 rounded-card border border-line p-3 sm:grid-cols-3"
        >
          <Measure
            size="md"
            label={t('contas-ajuste-calculado', { data: formatDate(date) })}
            value={
              <span className="text-[20px]">
                {computed == null ? '—' : <Money cents={computed} currency={currency} />}
              </span>
            }
          />
          <Measure
            size="md"
            label={t('contas-ajuste-informado')}
            value={
              <span className="text-[20px]">
                {balance == null ? '—' : <Money cents={balance} currency={currency} />}
              </span>
            }
          />
          <Measure
            size="md"
            label={t('contas-ajuste-valor')}
            value={
              <span className="text-[20px]">
                {diff == null ? (
                  '—'
                ) : (
                  <Money
                    cents={diff}
                    currency={currency}
                    signed
                    tone={diff === 0 ? 'positive' : 'auto'}
                  />
                )}
              </span>
            }
            hint={diff === 0 ? t('contas-ajuste-confere') : t('contas-ajuste-fora-relatorios')}
          />
        </div>
      ) : null}
      <InlineError error={sync.error} />
    </form>
  );
}
