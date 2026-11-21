import { useState } from 'react';
import { Link } from 'react-router';

import type { AccountView } from '@/api/generated/AccountView';
import type { TxnView } from '@/api/generated/TxnView';
import type { BalanceSync } from '@/api/generated/BalanceSync';
import type { ReconcileResult } from '@/api/generated/ReconcileResult';
import { useReconcile } from '@/api/hooks/accounts';
import { useUndo } from '@/api/hooks/changes';
import { useMembers } from '@/api/hooks/household';
import { useTxnQuery } from '@/api/hooks/transactions';
import { useDeleteFxRate, useFxRates, useSetFxRate } from '@/api/hooks/fx';
import {
  Button,
  DateInput,
  Field,
  InstrumentCard,
  Input,
  Measure,
  Money,
  MoneyInput,
  Private,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatDateTime, todayIso } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import { useBaseCurrency } from '@/state/currency';
import { CardLimitPanel } from '@/features/cards/CardLimitPanel';
import { CardValuePanel } from '@/features/cards/CardValuePanel';
import { PlasticsPanel } from '@/features/cards/Plastics';
import { RewardsPanel } from '@/features/cards/RewardsPanel';
import { InstallmentsPanel, StatementsPanel } from '@/features/cards/Statements';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { ACCOUNT_KIND, TXN_KIND } from '@/features/transactions/ui/labels';
import { slot, withSlots } from '@/features/transactions/ui/slots';

import { BaseBalance } from './AccountCard';
import { FxOperations } from '@/features/fx/FxOperations';

import { BalanceHistory } from './BalanceHistory';
import { CapitalSharesPanel } from './CapitalShares';
import { SyncDifference, SyncHistory, SyncReview } from './BalanceSync';
import { FgtsCheckPanel, FgtsEntries, FgtsSummary, FgtsWithdrawals, FgtsYield } from './FgtsPanel';
import { OverdraftSummary } from './Overdraft';
import { RetirementSummary } from './RetirementSummary';

/** Conciliação com o extrato: saldo informado × calculado e "marcar como conciliado". */
export function ReconcilePanel({ account }: { account: AccountView }) {
  const reconcile = useReconcile();
  const undo = useUndo();
  const [date, setDate] = useState<string | null>(todayIso());
  const [balance, setBalance] = useState<number | null>(null);
  const [result, setResult] = useState<ReconcileResult | null>(null);
  const [synced, setSynced] = useState<BalanceSync | null>(null);
  const check = (apply: boolean) => {
    if (!date || balance == null) return;
    reconcile.mutate(
      { account_id: account.id, statement_date: date, statement_balance: balance, apply },
      {
        onSuccess: (r) => {
          setResult(r);
          if (apply && r.marked_reconciled > 0) {
            toast({
              tone: 'success',
              title: t('contas-n-conciliados', { n: r.marked_reconciled }),
              description: t('contas-ate-data', { data: formatDate(date) }),
            });
          }
        },
      },
    );
  };
  return (
    <div className="flex flex-col gap-4">
      <p className="max-w-[70ch] text-[13px] leading-relaxed text-dim">
        {withSlots(t('contas-conciliar-explicacao', { conciliados: slot(0) }), [
          <strong className="text-fg">{t('contas-conciliados-destaque')}</strong>,
        ])}
      </p>
      <div className="flex flex-wrap items-end gap-3">
        <Field label={t('contas-data-extrato')} className="w-44">
          <DateInput
            value={date}
            onValueChange={(d) => (setDate(d), setResult(null), setSynced(null))}
          />
        </Field>
        <Field label={t('contas-saldo-extrato', { moeda: account.currency })} className="w-56">
          <MoneyInput
            currency={account.currency}
            allowNegative
            value={balance}
            onValueChange={(b) => (setBalance(b), setResult(null), setSynced(null))}
          />
        </Field>
        <Button
          variant="secondary"
          loading={reconcile.isPending && !result}
          disabled={!date || balance == null}
          onClick={() => check(false)}
        >
          {t('contas-comparar')}
        </Button>
      </div>
      {result ? (
        <div className="flex flex-col gap-3 rounded-card border border-line p-3" role="status">
          <div className="grid gap-4 @min-[400px]:grid-cols-3">
            <Measure
              size="md"
              label={t('contas-extrato-banco')}
              value={<Money cents={result.statement_balance} currency={account.currency} />}
            />
            <Measure
              size="md"
              label={t('contas-calculado')}
              value={<Money cents={result.computed_balance} currency={account.currency} />}
            />
            <Measure
              size="md"
              label={t('contas-diferenca')}
              value={
                <Money
                  cents={result.difference}
                  currency={account.currency}
                  tone={result.difference === 0 ? 'positive' : 'negative'}
                  signed
                />
              }
            />
          </div>
          {result.difference === 0 ? (
            result.change_set_id ? (
              <div className="flex items-center gap-3">
                <p className="text-[13px] text-positive">
                  {t('contas-n-marcados-conciliados', { n: result.marked_reconciled })}
                </p>
                <Button
                  size="sm"
                  variant="ghost"
                  loading={undo.isPending}
                  onClick={() =>
                    undo.mutate(
                      { id: result.change_set_id! },
                      {
                        onSuccess: () => (
                          toast({ tone: 'success', title: t('contas-conciliacao-desfeita') }),
                          setResult(null)
                        ),
                      },
                    )
                  }
                >
                  {t('contas-desfazer')}
                </Button>
              </div>
            ) : (
              <div className="flex items-center gap-3">
                <p className="text-[13px] text-positive">{t('contas-confere')}</p>
                <Button
                  size="sm"
                  variant="primary"
                  loading={reconcile.isPending}
                  onClick={() => check(true)}
                >
                  {t('contas-marcar-conciliado', { data: date ? formatDate(date) : '' })}
                </Button>
              </div>
            )
          ) : synced ? (
            <SyncReview sync={synced} />
          ) : (
            <>
              <p className="text-[13px] text-warning">
                {withSlots(
                  t('contas-nao-confere', { data: date ? formatDate(date) : '', link: slot(0) }),
                  [
                    <Link
                      className="text-accent underline"
                      to={txnLink({
                        account_ids: [account.id],
                        date_to: date ?? undefined,
                        statuses: ['cleared'],
                      })}
                    >
                      {t('contas-nao-confere-link')}
                    </Link>,
                  ],
                )}
              </p>
              {date && balance != null ? (
                <SyncDifference
                  account={account}
                  date={date}
                  balance={balance}
                  onDone={setSynced}
                />
              ) : null}
            </>
          )}
        </div>
      ) : null}
      <InlineError error={reconcile.error ?? undo.error} />
      <SyncHistory account={account} />
    </div>
  );
}

/** Câmbio manual (`fx.set`) para contas em moeda estrangeira, com as últimas taxas. */
export function FxPanel({ account }: { account: AccountView }) {
  const base = useBaseCurrency();
  const rates = useFxRates({ base: account.currency, quote: base, limit: 12 });
  const setRate = useSetFxRate();
  const remove = useDeleteFxRate();
  const [date, setDate] = useState<string | null>(todayIso());
  const [rate, setRateText] = useState('');
  return (
    <div className="flex flex-col gap-4">
      <p className="max-w-[70ch] text-[13px] text-dim">{t('contas-cambio-explicacao', { base })}</p>
      <div className="flex flex-wrap items-end gap-3">
        <Field label={t('contas-data')} className="w-44">
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field
          label={t('contas-taxa-rotulo', { moeda: account.currency })}
          hint={t('contas-taxa-dica', { base })}
          className="w-44"
        >
          <Input
            numeric
            inputMode="decimal"
            value={rate}
            onChange={(e) => setRateText(e.target.value.replace(/[^\d.,]/g, ''))}
            suffix={<span className="pr-2 text-[12px] text-dim">{base}</span>}
          />
        </Field>
        <Button
          variant="secondary"
          loading={setRate.isPending}
          disabled={!date || !rate}
          onClick={() =>
            setRate.mutate(
              { base: account.currency, quote: base, date: date!, rate: rate.replace(',', '.') },
              {
                onSuccess: () => (
                  toast({
                    tone: 'success',
                    title: t('contas-cambio-salvo'),
                    description: t('contas-cambio-salvo-descricao', {
                      moeda: account.currency,
                      base,
                      data: formatDate(date!),
                    }),
                  }),
                  setRateText('')
                ),
              },
            )
          }
        >
          {t('contas-salvar-taxa')}
        </Button>
      </div>
      <InlineError error={setRate.error ?? remove.error} />
      <QueryState loading={rates.isLoading} error={rates.error}>
        <ul
          className="flex flex-col divide-y divide-line/60 rounded-card border border-line"
          aria-label={t('contas-taxas-cambio')}
        >
          {(rates.data ?? []).map((r) => (
            <li
              key={r.id}
              className="grid grid-cols-[110px_1fr_1fr_auto] items-center gap-3 px-3 py-1.5 text-[12.5px]"
            >
              <span className="num">{formatDate(r.date)}</span>
              <span className="num">
                {t('contas-taxa-valor', {
                  base: r.base,
                  taxa: Number(r.rate),
                  cotacao: r.quote,
                })}
              </span>
              <span className="text-dim">
                {`${r.source === 'manual' ? t('contas-informada-mao') : r.source} · ${formatDateTime(r.fetched_at)}`}
              </span>
              {r.source === 'manual' ? (
                <Button size="sm" variant="ghost" onClick={() => remove.mutate({ id: r.id })}>
                  {t('contas-remover')}
                </Button>
              ) : (
                <span />
              )}
            </li>
          ))}
          {(rates.data ?? []).length === 0 ? (
            <li className="px-3 py-2 text-[12.5px] text-dim">
              {t('contas-nenhuma-taxa', { moeda: account.currency, base })}
            </li>
          ) : null}
        </ul>
      </QueryState>
    </div>
  );
}

type DetailTab =
  | 'lancamentos'
  | 'historico'
  | 'faturas'
  | 'parcelamentos'
  | 'cartoes'
  | 'limite'
  | 'recompensas'
  | 'conciliacao'
  | 'cambio'
  | 'cotas'
  | 'fgts'
  | 'fgts-conferencia'
  | 'fgts-rendimento'
  | 'fgts-saques';

/** Os últimos lançamentos da conta, com o link para a lista completa em Lançamentos. */
function RecentTxns({ account }: { account: AccountView }) {
  // Só o que já aconteceu: os previstos (recorrências futuras) ficam em Lançamentos.
  const q = useTxnQuery({ account_ids: [account.id], date_to: todayIso(), limit: 25 });
  const items = q.data?.items ?? [];
  const columns: TechColumn<TxnView>[] = [
    { key: 'date', header: t('contas-data'), numeric: true, cell: (r) => formatDate(r.date) },
    {
      key: 'description',
      header: t('contas-col-descricao'),
      cell: (r) => (
        <span className="font-sans">
          {r.description}
          {r.kind === 'adjustment' ? (
            <span className="ml-2 rounded-control border border-warning/50 px-1 text-[11px] text-warning">
              {TXN_KIND.adjustment}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'category',
      header: t('contas-col-categoria'),
      cell: (r) => <span className="font-sans text-dim">{r.category_path ?? '—'}</span>,
    },
    {
      key: 'amount',
      header: t('contas-col-valor'),
      numeric: true,
      cell: (r) => <Money cents={r.amount} currency={r.currency} tone="auto" />,
    },
  ];
  return (
    <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
      <div className="flex flex-col gap-3">
        <TechTable
          label={t('contas-ultimos-lancamentos')}
          prefsId="Últimos lançamentos da conta"
          columns={columns}
          rows={items}
          getRowId={(r) => r.id}
          dense
          empty={<span className="text-[12.5px] text-dim">{t('contas-sem-lancamentos')}</span>}
        />
        <Link
          to={txnLink({ account_ids: [account.id] })}
          className="label-caps self-start text-accent hover:underline"
        >
          {t('contas-ver-todos-lancamentos')}
        </Link>
      </div>
    </QueryState>
  );
}

/** Detalhe da conta: histórico de saldo, faturas e parcelamentos (cartão), conciliação e câmbio. */
export function AccountDetail({
  account,
  tab,
  onTabChange,
  onEdit,
  onNewTxn,
  onAdjust,
}: {
  account: AccountView;
  tab: string | null;
  onTabChange: (tab: DetailTab) => void;
  onEdit: () => void;
  onNewTxn: () => void;
  /** Ajuste de saldo numa data (lançamento de ajuste). */
  onAdjust: () => void;
}) {
  const isCard = account.kind === 'credit_card';
  const isFgts = account.kind === 'fgts';
  const base = useBaseCurrency();
  const foreign = account.currency !== base;
  const members = useMembers(true).data ?? [];
  const owners = account.member_ids
    .map((id) => members.find((m) => m.id === id)?.name)
    .filter(Boolean)
    .join(', ');
  const tabs: { id: DetailTab; label: string }[] = [
    ...(isFgts
      ? [
          { id: 'fgts' as const, label: t('fgts-aba-extrato') },
          { id: 'fgts-conferencia' as const, label: t('fgts-aba-conferencia') },
          { id: 'fgts-rendimento' as const, label: t('fgts-aba-rendimento') },
          { id: 'fgts-saques' as const, label: t('fgts-aba-saques') },
        ]
      : []),
    { id: 'lancamentos', label: t('contas-ultimos-lancamentos') },
    { id: 'historico', label: t('contas-historico-saldo') },
    ...(isCard
      ? [
          { id: 'faturas' as const, label: t('contas-faturas') },
          { id: 'parcelamentos' as const, label: t('contas-parcelamentos') },
          { id: 'limite' as const, label: t('contas-limite-anuidade') },
          { id: 'recompensas' as const, label: t('contas-recompensa-aba') },
          { id: 'cartoes' as const, label: t('contas-cartoes') },
        ]
      : []),
    { id: 'conciliacao', label: t('contas-conciliacao') },
    ...(foreign ? [{ id: 'cambio' as const, label: t('contas-cambio') }] : []),
    ...(['checking', 'savings', 'other'].includes(account.kind)
      ? [{ id: 'cotas' as const, label: t('contas-cotas-capital') }]
      : []),
  ];
  const current =
    tabs.find((x) => x.id === tab)?.id ?? (isCard ? 'faturas' : isFgts ? 'fgts' : 'lancamentos');
  return (
    <InstrumentCard
      index="03.0"
      title={t('contas-detalhe-resumo')}
      actions={
        <>
          <Button size="sm" variant="ghost" onClick={onAdjust}>
            {t('contas-ajustar-saldo')}
          </Button>
          <Button size="sm" variant="ghost" onClick={onNewTxn}>
            {t('contas-novo-lancamento')}
          </Button>
          <Button size="sm" variant="secondary" onClick={onEdit}>
            {t('comum-editar')}
          </Button>
        </>
      }
      footer={[
        { label: t('contas-tipo'), value: ACCOUNT_KIND[account.kind] },
        ...(account.institution_name
          ? [{ label: t('contas-instituicao'), value: account.institution_name }]
          : []),
        ...(owners ? [{ label: t('contas-col-membros'), value: owners }] : []),
        { label: t('contas-moeda'), value: account.currency },
        {
          label: t('contas-saldo-inicial'),
          value: <Money cents={account.opening_balance} currency={account.currency} />,
        },
        { label: t('contas-desde'), value: formatDate(account.opening_date) },
        {
          label: t('contas-patrimonio'),
          value: account.include_in_net_worth
            ? t('contas-patrimonio-entra')
            : t('contas-patrimonio-fora'),
        },
        ...(account.card?.last4
          ? [
              {
                label: t('contas-cartao'),
                value: `${account.card.network ?? ''} •••• ${account.card.last4}`.trim(),
              },
            ]
          : []),
      ]}
    >
      <div className="mb-5 grid gap-5 @min-[400px]:grid-cols-2 @min-[790px]:grid-cols-4">
        <Measure
          size="md"
          label={isCard ? t('contas-saldo-devedor') : t('contas-saldo-hoje')}
          value={<Money cents={account.balance} currency={account.currency} />}
          changeKey={account.balance}
        />
        <Measure
          size="md"
          label={t('contas-previsto-ate-fim-mes')}
          value={
            <span className="text-[22px]">
              <Money cents={account.projected_balance} currency={account.currency} />
            </span>
          }
        />
        <Measure
          size="md"
          label={t('contas-conciliado')}
          value={
            <span className="text-[22px]">
              <Money cents={account.reconciled_balance} currency={account.currency} />
            </span>
          }
        />
        {foreign ? (
          <Measure
            size="md"
            label={t('contas-em-moeda', { moeda: base })}
            value={
              <span className="text-[22px]">
                <BaseBalance account={account} />
              </span>
            }
          />
        ) : isCard ? (
          <Measure
            size="md"
            label={t('contas-credito-disponivel')}
            value={
              <span className="text-[22px]">
                {account.available_credit != null ? (
                  <Money cents={account.available_credit} currency={account.currency} />
                ) : (
                  t('contas-sem-limite')
                )}
              </span>
            }
            hint={
              account.card?.credit_limit != null
                ? withSlots(t('contas-limite-de', { limite: slot(0) }), [
                    <Money cents={account.card.credit_limit} currency={account.currency} />,
                  ])
                : undefined
            }
          />
        ) : null}
      </div>
      {isFgts ? (
        <div className="mb-4">
          <Private>
            <FgtsSummary account={account} />
          </Private>
        </div>
      ) : null}
      {account.kind === 'retirement' ? (
        <div className="mb-4 max-w-xl">
          <Private>
            <RetirementSummary account={account} />
          </Private>
        </div>
      ) : null}
      {account.overdraft_status ? (
        <div className="max-w-md">
          <Private>
            <OverdraftSummary status={account.overdraft_status} currency={account.currency} />
          </Private>
        </div>
      ) : null}
      <Tabs value={current} onValueChange={(v) => onTabChange(v as DetailTab)}>
        <TabsList aria-label={t('contas-secoes-de', { nome: account.name })}>
          {tabs.map((x) => (
            <TabsTrigger key={x.id} value={x.id}>
              {x.label}
            </TabsTrigger>
          ))}
        </TabsList>
        {isFgts ? (
          <>
            <TabsContent value="fgts">
              <FgtsEntries account={account} />
            </TabsContent>
            <TabsContent value="fgts-conferencia">
              <FgtsCheckPanel account={account} />
            </TabsContent>
            <TabsContent value="fgts-rendimento">
              <FgtsYield account={account} />
            </TabsContent>
            <TabsContent value="fgts-saques">
              <FgtsWithdrawals account={account} />
            </TabsContent>
          </>
        ) : null}
        <TabsContent value="lancamentos">
          <RecentTxns account={account} />
        </TabsContent>
        <TabsContent value="historico">
          <BalanceHistory account={account} />
        </TabsContent>
        {isCard ? (
          <>
            <TabsContent value="faturas">
              <StatementsPanel card={account} />
            </TabsContent>
            <TabsContent value="parcelamentos">
              <InstallmentsPanel card={account} />
            </TabsContent>
            <TabsContent value="limite">
              <CardLimitPanel card={account} />
            </TabsContent>
            <TabsContent value="recompensas">
              <div className="flex flex-col gap-6">
                <RewardsPanel card={account} />
                <CardValuePanel card={account} />
              </div>
            </TabsContent>
            <TabsContent value="cartoes">
              <PlasticsPanel card={account} />
            </TabsContent>
          </>
        ) : null}
        <TabsContent value="conciliacao">
          <ReconcilePanel account={account} />
        </TabsContent>
        <TabsContent value="cotas">
          <CapitalSharesPanel account={account} />
        </TabsContent>
        {foreign ? (
          <TabsContent value="cambio">
            <div className="flex flex-col gap-6">
              <FxOperations accountId={account.id} />
              <FxPanel account={account} />
            </div>
          </TabsContent>
        ) : null}
      </Tabs>
    </InstrumentCard>
  );
}
