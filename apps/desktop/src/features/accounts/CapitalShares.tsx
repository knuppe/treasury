/**
 * Cotas capital de cooperativa na conta corrente: o usuário informa o saldo das cotas de
 * tempos em tempos e o Treasury apura a rentabilidade de cada período como a variação do
 * saldo menos os aportes (lançamentos da conta que batem com o texto ou a categoria).
 */
import { useState } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import type { CapitalBalancePoint } from '@/api/generated/CapitalBalancePoint';
import {
  useAddCapitalBalance,
  useCapitalShare,
  useDeleteCapitalBalance,
  useMarkCapitalContributions,
  useSetCapitalShare,
} from '@/api/hooks/accounts';
import {
  Button,
  DateInput,
  Field,
  Input,
  Measure,
  Money,
  MoneyInput,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import { InlineError, QueryState, Switch } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';

const pct = (v: number | null | undefined) =>
  v == null ? '—' : t('contas-cotas-pct', { valor: v });

export function CapitalSharesPanel({ account }: { account: AccountView }) {
  const q = useCapitalShare(account.id);
  const set = useSetCapitalShare();
  const add = useAddCapitalBalance();
  const del = useDeleteCapitalBalance();
  const mark = useMarkCapitalContributions();
  const [date, setDate] = useState<string | null>(todayIso());
  const [balance, setBalance] = useState<number | null>(null);
  const [note, setNote] = useState('');
  const v = q.data;
  const [pattern, setPattern] = useState<string | null>(null);

  const columns: TechColumn<CapitalBalancePoint>[] = [
    { key: 'date', header: t('contas-data'), cell: (p) => formatDate(p.date) },
    {
      key: 'balance',
      header: t('contas-cotas-saldo'),
      numeric: true,
      cell: (p) => <Money cents={p.balance} currency={account.currency} />,
    },
    {
      key: 'contrib',
      header: t('contas-cotas-aportes-periodo'),
      numeric: true,
      cell: (p) => <Money cents={p.contributions} currency={account.currency} />,
    },
    {
      key: 'yield',
      header: t('contas-cotas-rentabilidade'),
      numeric: true,
      cell: (p) =>
        p.yield_amount == null ? (
          '—'
        ) : (
          <Money cents={p.yield_amount} currency={account.currency} signed tone="auto" />
        ),
    },
    { key: 'pct', header: '%', numeric: true, cell: (p) => pct(p.yield_pct) },
    { key: 'note', header: t('contas-cotas-nota'), cell: (p) => p.note ?? '' },
    {
      key: 'x',
      header: '',
      cell: (p) => (
        <Button size="sm" variant="ghost" onClick={() => del.mutate({ id: p.id })}>
          {t('comum-apagar')}
        </Button>
      ),
    },
  ];

  return (
    <QueryState loading={q.isLoading} error={q.error}>
      {!v ? (
        <div className="flex max-w-[70ch] flex-col gap-3">
          <p className="text-[13px] leading-relaxed text-dim">{t('contas-cotas-explicacao')}</p>
          <Button
            variant="primary"
            className="self-start"
            loading={set.isPending}
            onClick={() => set.mutate({ account_id: account.id })}
          >
            {t('contas-cotas-ativar')}
          </Button>
          <InlineError error={set.error} />
        </div>
      ) : (
        <div className="flex flex-col gap-5">
          <div className="grid gap-4 @min-[400px]:grid-cols-2 @min-[790px]:grid-cols-4">
            <Measure
              size="md"
              label={t('contas-cotas-saldo')}
              value={
                v.current_balance != null ? (
                  <Money cents={v.current_balance} currency={account.currency} />
                ) : (
                  '—'
                )
              }
              hint={
                v.current_date
                  ? t('contas-cotas-em', { data: formatDate(v.current_date) })
                  : t('contas-cotas-informe-saldo')
              }
            />
            <Measure
              size="md"
              label={t('contas-cotas-aportes')}
              value={<Money cents={v.total_contributions} currency={account.currency} />}
              hint={
                v.contributions_since_last
                  ? withSlots(t('contas-cotas-depois-ultimo', { valor: slot(0) }), [
                      <Money cents={v.contributions_since_last} currency={account.currency} />,
                    ])
                  : undefined
              }
            />
            <Measure
              size="md"
              label={t('contas-cotas-rentabilidade-apurada')}
              value={<Money cents={v.total_yield} currency={account.currency} signed tone="auto" />}
            />
            <Measure size="md" label={t('contas-cotas-12-meses')} value={pct(v.yield_12m_pct)} />
          </div>

          <div className="flex flex-wrap items-end gap-3">
            <Field label={t('contas-cotas-data-saldo')} className="w-44">
              <DateInput value={date} onValueChange={setDate} />
            </Field>
            <Field
              label={t('contas-cotas-saldo-moeda', { moeda: account.currency })}
              className="w-52"
            >
              <MoneyInput currency={account.currency} value={balance} onValueChange={setBalance} />
            </Field>
            <Field label={t('contas-cotas-nota-opcional')} className="min-w-48 flex-1">
              <Input
                value={note}
                onChange={(e) => setNote(e.target.value)}
                placeholder={t('contas-cotas-nota-exemplo')}
              />
            </Field>
            <Button
              variant="primary"
              disabled={!date || balance == null}
              loading={add.isPending}
              onClick={() =>
                date &&
                balance != null &&
                add.mutate(
                  {
                    account_id: account.id,
                    date,
                    balance,
                    ...(note.trim() ? { note: note.trim() } : {}),
                  },
                  {
                    onSuccess: () => {
                      setBalance(null);
                      setNote('');
                      toast({ tone: 'success', title: t('contas-cotas-saldo-informado') });
                    },
                  },
                )
              }
            >
              {t('contas-cotas-informar-saldo')}
            </Button>
          </div>
          <InlineError error={add.error ?? del.error} />

          {v.balances.length ? (
            <TechTable
              label={t('contas-cotas-saldos')}
              prefsId="Saldos das cotas capital"
              columns={columns}
              rows={[...v.balances].reverse()}
              getRowId={(p) => p.id}
            />
          ) : null}

          <section
            aria-label={t('contas-cotas-aportes-identificados')}
            className="flex flex-col gap-2"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h3 className="label-caps">
                {t('contas-cotas-aportes-identificados-n', { n: v.contributions.length })}
              </h3>
              <Button
                size="sm"
                variant="ghost"
                loading={mark.isPending}
                disabled={!v.contributions.length}
                onClick={() =>
                  mark.mutate(
                    { account_id: account.id },
                    {
                      onSuccess: (r) =>
                        toast({
                          tone: 'success',
                          title:
                            r.updated === 0
                              ? t('contas-cotas-ja-fora')
                              : t('contas-cotas-n-tirados', { n: r.updated }),
                        }),
                    },
                  )
                }
              >
                {t('contas-cotas-tirar')}
              </Button>
            </div>
            <ul className="flex max-h-48 flex-col overflow-auto text-[12.5px]">
              {[...v.contributions].reverse().map((c) => (
                <li key={c.txn_id} className="flex gap-3 border-t border-line/60 py-1">
                  <span className="num w-24 text-dim">{formatDate(c.date)}</span>
                  <span className="min-w-0 flex-1 truncate">{c.description}</span>
                  <Money cents={c.amount} currency={account.currency} />
                </li>
              ))}
            </ul>
          </section>

          <section aria-label={t('contas-cotas-como-identificar')} className="flex flex-col gap-3">
            <h3 className="label-caps">{t('contas-cotas-como-identificar')}</h3>
            <div className="grid gap-3 @min-[620px]:grid-cols-2">
              <Field label={t('contas-cotas-texto-extrato')} hint={t('contas-cotas-texto-dica')}>
                <Input
                  value={pattern ?? v.contribution_pattern}
                  onChange={(e) => setPattern(e.target.value)}
                  onBlur={() =>
                    pattern != null &&
                    pattern !== v.contribution_pattern &&
                    set.mutate({
                      account_id: account.id,
                      contribution_pattern: pattern,
                      ...(v.contribution_category_id
                        ? { contribution_category_id: v.contribution_category_id }
                        : {}),
                      include_in_net_worth: v.include_in_net_worth,
                    })
                  }
                />
              </Field>
              <Field label={t('contas-cotas-ou-categoria')}>
                <CategoryPicker
                  kind="expense"
                  label={t('contas-cotas-categoria')}
                  noneLabel={t('contas-cotas-nenhuma')}
                  value={v.contribution_category_id}
                  onChange={(c) =>
                    set.mutate({
                      account_id: account.id,
                      contribution_pattern: v.contribution_pattern,
                      ...(c ? { contribution_category_id: c } : {}),
                      include_in_net_worth: v.include_in_net_worth,
                    })
                  }
                />
              </Field>
            </div>
            <Switch
              checked={v.include_in_net_worth}
              onChange={(on) =>
                set.mutate({
                  account_id: account.id,
                  contribution_pattern: v.contribution_pattern,
                  ...(v.contribution_category_id
                    ? { contribution_category_id: v.contribution_category_id }
                    : {}),
                  include_in_net_worth: on,
                })
              }
              label={t('contas-cotas-somar')}
              description={t('contas-cotas-somar-dica')}
            />
            <InlineError error={set.error ?? mark.error} />
          </section>
        </div>
      )}
    </QueryState>
  );
}
