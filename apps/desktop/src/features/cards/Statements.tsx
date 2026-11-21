import { useCallback, useMemo, useState } from 'react';
import { Link } from 'react-router';

import type { AccountView } from '@/api/generated/AccountView';
import type { StatementView } from '@/api/generated/StatementView';
import { useAccounts } from '@/api/hooks/accounts';
import {
  useDeleteInstallmentPlan,
  useFutureCommitments,
  useInstallmentPlans,
  usePayStatement,
  useSetInformedTotal,
  useStatement,
  useStatements,
  useUpdateStatement,
} from '@/api/hooks/cards';
import {
  ChartFrame,
  EChart,
  barStyle,
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import {
  Button,
  DateInput,
  Dialog,
  EmptyState,
  Field,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, formatMonthShort, todayIso } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import {
  ConfirmDialog,
  InlineError,
  QueryState,
  SectionTitle,
} from '@/features/transactions/ui/controls';
import { STATEMENT_STATUS } from '@/features/transactions/ui/labels';
import { slot, withSlots } from '@/features/transactions/ui/slots';

import { plasticLabel } from './Plastics';

const STATUS_TONE: Record<StatementView['status'], string> = {
  open: 'border-accent/50 text-accent',
  closed: 'border-warning/60 text-warning',
  paid: 'border-positive/50 text-positive',
  partial: 'border-warning/60 text-warning',
};

export function StatementStatusBadge({ status }: { status: StatementView['status'] }) {
  return (
    <span
      className={cx(
        'label-caps inline-flex h-5 items-center rounded-control border px-1.5 whitespace-nowrap',
        STATUS_TONE[status],
      )}
    >
      {STATEMENT_STATUS[status]}
    </span>
  );
}

/** Pagar fatura: transferência da conta pagadora para o cartão, ligada à fatura. */
export function PayStatementDialog({
  open,
  onOpenChange,
  statement,
  card,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  statement: StatementView;
  card: AccountView;
}) {
  const { data: accounts = [] } = useAccounts();
  const pay = usePayStatement();
  const payers = accounts.filter(
    (a) => a.kind !== 'credit_card' && a.currency === statement.currency,
  );
  const [from, setFrom] = useState(card.card?.payment_account_id ?? payers[0]?.id ?? '');
  const [amount, setAmount] = useState<number | null>(
    statement.remaining > 0 ? statement.remaining : null,
  );
  const [date, setDate] = useState<string | null>(
    statement.due_date < todayIso() ? todayIso() : statement.due_date,
  );
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="02"
      size="sm"
      title={t('contas-pagar-fatura-de', { mes: formatMonthLong(statement.reference_month) })}
      description={t('contas-pagar-fatura-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={pay.isPending}
            disabled={!from || !amount || !date}
            onClick={() =>
              pay.mutate(
                {
                  statement_id: statement.id,
                  from_account_id: from,
                  amount: amount ?? undefined,
                  date: date ?? undefined,
                },
                {
                  onSuccess: (res) => {
                    toast({
                      tone: 'success',
                      title: t('contas-pagamento-lancado'),
                      description: `${res.transfer.from.account_name} → ${res.transfer.to.account_name}`,
                    });
                    onOpenChange(false);
                  },
                },
              )
            }
          >
            {t('contas-pagar')}
          </Button>
        </>
      }
    >
      <KeyValue
        layout="stacked"
        items={[
          {
            label: t('contas-total'),
            value: <Money cents={statement.total} currency={statement.currency} />,
          },
          {
            label: t('contas-ja-pago'),
            value: <Money cents={statement.paid} currency={statement.currency} />,
          },
          {
            label: t('contas-restante'),
            value: <Money cents={statement.remaining} currency={statement.currency} />,
          },
          { label: t('contas-vencimento'), value: formatDate(statement.due_date) },
        ]}
      />
      <Field label={t('contas-pagar-com')}>
        <Select
          value={from || undefined}
          onValueChange={setFrom}
          options={payers.map((a) => ({ value: a.id, label: a.name }))}
          placeholder={t('contas-escolha-conta')}
        />
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-valor')}>
          <MoneyInput currency={statement.currency} value={amount} onValueChange={setAmount} />
        </Field>
        <Field label={t('contas-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
      </div>
      <InlineError error={pay.error} />
    </Dialog>
  );
}

/** Datas reais da fatura (fechamento adiado, vencimento mudado) e o que o banco informou. */
function StatementDatesDialog({
  statement,
  onClose,
}: {
  statement: StatementView;
  onClose: () => void;
}) {
  const update = useUpdateStatement();
  const [closing, setClosing] = useState<string | null>(statement.closing_date);
  const [due, setDue] = useState<string | null>(statement.due_date);
  const [minimum, setMinimum] = useState<number | null>(statement.informed_minimum);
  const [limit, setLimit] = useState<number | null>(statement.informed_limit);
  const [previous, setPrevious] = useState<number | null>(statement.informed_previous_balance);
  const datesChanged = closing !== statement.closing_date || due !== statement.due_date;
  const done = {
    onSuccess: () => {
      toast({ tone: 'success', title: t('contas-fatura-alterada') });
      onClose();
    },
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={t('contas-fatura-ajustar', { mes: formatMonthLong(statement.reference_month) })}
      description={t('contas-fatura-ajustar-descricao')}
      footer={
        <>
          {statement.dates_source !== 'calc' ? (
            <Button
              variant="ghost"
              className="mr-auto"
              loading={update.isPending}
              onClick={() => update.mutate({ id: statement.id, reset_dates: true }, done)}
            >
              {t('contas-fatura-datas-calculadas')}
            </Button>
          ) : null}
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={update.isPending}
            disabled={!closing || !due}
            onClick={() =>
              update.mutate(
                {
                  id: statement.id,
                  ...(datesChanged ? { closing_date: closing!, due_date: due! } : {}),
                  informed_minimum: minimum,
                  informed_limit: limit,
                  informed_previous_balance: previous,
                },
                done,
              )
            }
          >
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-fatura-fechamento')}>
          <DateInput value={closing} onValueChange={setClosing} />
        </Field>
        <Field label={t('contas-vencimento')}>
          <DateInput value={due} onValueChange={setDue} />
        </Field>
        <Field label={t('contas-fatura-minimo')}>
          <MoneyInput currency={statement.currency} value={minimum} onValueChange={setMinimum} />
        </Field>
        <Field label={t('contas-fatura-limite')}>
          <MoneyInput currency={statement.currency} value={limit} onValueChange={setLimit} />
        </Field>
        <Field label={t('contas-fatura-saldo-anterior')}>
          <MoneyInput currency={statement.currency} value={previous} onValueChange={setPrevious} />
        </Field>
      </div>
      <InlineError error={update.error} />
    </Dialog>
  );
}

function StatementDetailPanel({ statementId, card }: { statementId: string; card: AccountView }) {
  const { data, isLoading, error } = useStatement(statementId);
  const setInformed = useSetInformedTotal();
  const [paying, setPaying] = useState(false);
  const [adjusting, setAdjusting] = useState(false);
  const [informed, setInformedValue] = useState<number | null>(null);
  const [syncedFor, setSyncedFor] = useState<string | null>(null);
  if (data && syncedFor !== `${data.statement.id}:${data.statement.informed_total}`) {
    setSyncedFor(`${data.statement.id}:${data.statement.informed_total}`);
    setInformedValue(data.statement.informed_total);
  }
  return (
    <QueryState loading={isLoading} error={error}>
      {data ? (
        <section
          aria-label={t('contas-fatura-de', {
            mes: formatMonthLong(data.statement.reference_month),
          })}
          className="flex flex-col gap-3 rounded-card border border-line p-3"
        >
          <SectionTitle
            aside={
              <span className="flex items-center gap-2">
                <StatementStatusBadge status={data.statement.status} />
                <Link
                  to={txnLink({ statement_id: data.statement.id })}
                  className="label-caps hover:text-accent"
                >
                  {t('contas-ver-em-lancamentos')}
                </Link>
              </span>
            }
          >
            {t('contas-fatura-de', { mes: formatMonthLong(data.statement.reference_month) })}
          </SectionTitle>
          <KeyValue
            items={[
              {
                label: t('contas-compras'),
                value: t('contas-compras-periodo', {
                  inicio: formatDate(data.statement.period_start),
                  fim: formatDate(data.statement.closing_date),
                }),
              },
              {
                label: t('contas-vence'),
                value: (
                  <span>
                    {formatDate(data.statement.due_date)}
                    {data.statement.dates_source !== 'calc' ? (
                      <span className="text-dim">
                        {' · '}
                        {data.statement.dates_source === 'manual'
                          ? t('contas-fatura-datas-manuais')
                          : t('contas-fatura-datas-impressas')}
                      </span>
                    ) : null}
                  </span>
                ),
              },
              { label: t('contas-lancamentos'), value: data.statement.txn_count },
              ...(data.statement.informed_minimum != null
                ? [
                    {
                      label: t('contas-fatura-minimo'),
                      value: (
                        <Money
                          cents={data.statement.informed_minimum}
                          currency={data.statement.currency}
                        />
                      ),
                    },
                  ]
                : []),
              ...(data.statement.informed_previous_balance != null
                ? [
                    {
                      label: t('contas-fatura-saldo-anterior'),
                      value: (
                        <Money
                          cents={data.statement.informed_previous_balance}
                          currency={data.statement.currency}
                        />
                      ),
                    },
                  ]
                : []),
            ]}
          />
          <Button
            size="sm"
            variant="ghost"
            className="self-start"
            onClick={() => setAdjusting(true)}
          >
            {t('contas-fatura-ajustar-botao')}
          </Button>
          <dl className="grid grid-cols-2 gap-3 @min-[400px]:grid-cols-4">
            {(
              [
                ['contas-calculado', data.statement.total],
                ['contas-previsto', data.statement.planned],
                ['contas-pago', data.statement.paid],
                ['contas-restante', data.statement.remaining],
              ] as const
            ).map(([label, cents]) => (
              <div key={label} className="flex flex-col gap-1">
                <dt className="label-caps">{t(label)}</dt>
                <dd className="text-[18px]">
                  <Money cents={cents} currency={data.statement.currency} />
                </dd>
              </div>
            ))}
          </dl>
          {data.by_plastic.length > 0 ? (
            <section aria-label={t('contas-fatura-por-cartao')} className="flex flex-col gap-1">
              <h4 className="label-caps">{t('contas-fatura-por-cartao')}</h4>
              <ul className="grid grid-cols-1 gap-x-6 gap-y-1 @min-[500px]:grid-cols-2">
                {data.by_plastic.map((p) => (
                  <li
                    key={`${p.card_plastic_id ?? ''}|${p.member_id ?? ''}`}
                    className="flex items-baseline justify-between gap-3 text-[12.5px]"
                  >
                    <span className="truncate">
                      {p.kind
                        ? plasticLabel({ kind: p.kind, last4: p.last4, label: p.label })
                        : t('contas-fatura-sem-cartao')}
                      {p.member_name ? <span className="text-dim"> · {p.member_name}</span> : null}
                    </span>
                    <Money cents={p.total} currency={data.statement.currency} />
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
          <div className="flex flex-wrap items-end gap-3 border-t border-line pt-3">
            <Field label={t('contas-total-informado')} className="w-64">
              <MoneyInput
                currency={data.statement.currency}
                value={informed}
                onValueChange={setInformedValue}
              />
            </Field>
            <Button
              size="sm"
              variant="secondary"
              loading={setInformed.isPending}
              disabled={informed === data.statement.informed_total}
              onClick={() =>
                setInformed.mutate(
                  { id: data.statement.id, informed_total: informed },
                  {
                    onSuccess: () =>
                      toast({ tone: 'success', title: t('contas-total-informado-salvo') }),
                  },
                )
              }
            >
              {t('contas-salvar-total-informado')}
            </Button>
            {data.statement.informed_total != null ? (
              <p
                className={cx(
                  'text-[12.5px]',
                  data.statement.informed_total === data.statement.total
                    ? 'text-positive'
                    : 'text-warning',
                )}
                role="status"
              >
                {data.statement.informed_total === data.statement.total
                  ? t('contas-confere-calculado')
                  : withSlots(t('contas-diferenca-banco', { valor: slot(0) }), [
                      <Money
                        cents={data.statement.informed_total - data.statement.total}
                        currency={data.statement.currency}
                        tone="auto"
                      />,
                    ])}
              </p>
            ) : null}
            {data.statement.remaining > 0 ? (
              <Button
                size="sm"
                variant="primary"
                className="ml-auto"
                onClick={() => setPaying(true)}
              >
                {t('contas-pagar-fatura')}
              </Button>
            ) : null}
          </div>
          <InlineError error={setInformed.error} />
          <ul
            className="max-h-64 overflow-auto rounded-card border border-line/60"
            aria-label={t('contas-lancamentos-fatura')}
          >
            {data.transactions.map((x) => (
              <li
                key={x.id}
                className="grid grid-cols-[80px_1fr_minmax(0,180px)_120px] items-center gap-3 border-b border-line/50 px-3 py-1.5 text-[12.5px] last:border-0"
              >
                <span className="num text-dim">{formatDate(x.date)}</span>
                <span className={cx('truncate', x.status === 'planned' && 'text-dim italic')}>
                  {x.description}
                  {x.card_last4 ? (
                    <span className="text-dim">
                      {' · '}
                      {t('contas-plastico-final', { final: x.card_last4 })}
                    </span>
                  ) : null}
                  {x.orig_amount != null && x.orig_currency ? (
                    <span className="text-dim">
                      {' · '}
                      <Money cents={Math.abs(x.orig_amount)} currency={x.orig_currency} />
                    </span>
                  ) : null}
                </span>
                <span className="truncate text-dim">
                  {x.kind === 'card_payment'
                    ? t('contas-pagamento')
                    : (x.category_path ?? t('contas-sem-categoria'))}
                </span>
                <Money cents={x.amount} currency={x.currency} tone="auto" className="text-right" />
              </li>
            ))}
          </ul>
          {adjusting ? (
            <StatementDatesDialog statement={data.statement} onClose={() => setAdjusting(false)} />
          ) : null}
          {paying ? (
            <PayStatementDialog
              open={paying}
              onOpenChange={setPaying}
              statement={data.statement}
              card={card}
            />
          ) : null}
        </section>
      ) : null}
    </QueryState>
  );
}

/** Faturas do cartão: lista com total calculado × informado e o detalhe da escolhida. */
export function StatementsPanel({ card }: { card: AccountView }) {
  const { data = [], isLoading, error, refetch } = useStatements({ account_id: card.id });
  const sorted = useMemo(() => [...data].reverse(), [data]);
  // A fatura "da vez": a mais antiga fechada e não paga; senão a aberta do ciclo atual.
  const current =
    data.find((s) => s.status === 'closed' || s.status === 'partial') ??
    data.find((s) => s.status === 'open') ??
    sorted[0];
  const [selected, setSelected] = useState<string | null>(null);
  const selectedId = selected ?? current?.id ?? null;
  const columns: TechColumn<StatementView>[] = [
    {
      key: 'ref',
      header: t('contas-fatura'),
      cell: (s) => <span className="capitalize">{formatMonthShort(s.reference_month)}</span>,
      sortValue: (s) => s.reference_month,
    },
    {
      key: 'due',
      header: t('contas-vence'),
      numeric: true,
      align: 'left',
      cell: (s) => formatDate(s.due_date),
    },
    {
      key: 'status',
      header: t('contas-situacao'),
      cell: (s) => <StatementStatusBadge status={s.status} />,
    },
    {
      key: 'total',
      header: t('contas-total'),
      numeric: true,
      cell: (s) => <Money cents={s.total} currency={s.currency} />,
    },
    {
      key: 'informed',
      header: t('contas-banco-calc'),
      numeric: true,
      cell: (s) =>
        s.informed_total == null ? (
          <span className="text-dim">—</span>
        ) : s.informed_total === s.total ? (
          <span className="text-positive">{t('contas-confere-curto')}</span>
        ) : (
          <Money cents={s.informed_total - s.total} currency={s.currency} tone="auto" />
        ),
    },
    {
      key: 'remaining',
      header: t('contas-restante'),
      numeric: true,
      cell: (s) => <Money cents={s.remaining} currency={s.currency} />,
    },
  ];
  return (
    <QueryState loading={isLoading} error={error} onRetry={() => void refetch()}>
      {data.length === 0 ? (
        <EmptyState
          kicker={t('contas-faturas-vazio-kicker')}
          title={t('contas-faturas-vazio-titulo')}
          description={t('contas-faturas-vazio-descricao')}
        />
      ) : (
        <div className="grid gap-4 @min-[1300px]:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
          <TechTable
            label={t('contas-faturas')}
            prefsId="Faturas"
            dense
            columns={columns}
            rows={sorted}
            getRowId={(s) => s.id}
            selectedId={selectedId}
            onSelectedChange={(id) => setSelected(id)}
            maxHeight="460px"
          />
          {selectedId ? <StatementDetailPanel statementId={selectedId} card={card} /> : null}
        </div>
      )}
    </QueryState>
  );
}

/** Parcelamentos e compromissos futuros (quanto já está comprometido por mês). */
export function InstallmentsPanel({ card }: { card: AccountView }) {
  const plans = useInstallmentPlans(card.id);
  const commitments = useFutureCommitments({ account_id: card.id, months: 12 });
  const remove = useDeleteInstallmentPlan();
  const [deleting, setDeleting] = useState<string | null>(null);
  const months = useMemo(() => commitments.data ?? [], [commitments.data]);
  const option = useCallback(
    (theme: ChartTheme) => ({
      ...baseOption(theme),
      legend: { show: false },
      grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
      tooltip: {
        ...(baseOption(theme).tooltip as object),
        trigger: 'axis',
        valueFormatter: (v: number) => tooltipMoney(v, theme, card.currency),
      },
      xAxis: categoryAxis(
        theme,
        months.map((m) => formatMonthShort(m.month)),
      ),
      yAxis: valueAxis(theme, (v) => compactMoney(v, theme, card.currency)),
      series: [
        {
          name: t('contas-a-pagar'),
          type: 'bar',
          data: months.map((m) => m.amount),
          color: theme.accent,
          ...barStyle(theme),
        },
      ],
    }),
    [months, card.currency],
  );
  return (
    <div className="flex flex-col gap-5">
      <QueryState loading={commitments.isLoading} error={commitments.error}>
        <ChartFrame
          index="03.3"
          title={t('contas-compromissos-titulo')}
          source={t('contas-compromissos-fonte')}
          chart={
            <EChart option={option} height={200} ariaLabel={t('contas-compromissos-grafico')} />
          }
          table={
            <TechTable
              label={t('contas-compromissos-tabela')}
              prefsId="Compromissos futuros do cartão por mês"
              dense
              rows={months}
              getRowId={(m) => m.month}
              columns={[
                {
                  key: 'month',
                  header: t('contas-mes'),
                  className: 'capitalize',
                  cell: (m) => formatMonthLong(m.month),
                },
                {
                  key: 'amount',
                  header: t('contas-a-pagar'),
                  numeric: true,
                  cell: (m) => <Money cents={m.amount} currency={card.currency} />,
                },
              ]}
            />
          }
        />
      </QueryState>
      <SectionTitle>{t('contas-compras-parceladas')}</SectionTitle>
      <QueryState loading={plans.isLoading} error={plans.error}>
        {(plans.data ?? []).length === 0 ? (
          <p className="text-[13px] text-dim">{t('contas-sem-parceladas')}</p>
        ) : (
          <ul
            className="flex flex-col divide-y divide-line/60 rounded-card border border-line"
            aria-label={t('contas-parcelamentos')}
          >
            {(plans.data ?? []).map((p) => (
              <li
                key={p.id}
                className="grid grid-cols-[minmax(0,1fr)_120px_120px_120px_auto] items-center gap-3 px-3 py-2 text-[13px]"
              >
                <span className="truncate">
                  {p.description}
                  <span className="ml-2 text-[11.5px] text-dim">
                    {t('contas-compra-em', { data: formatDate(p.purchase_date) })}
                  </span>
                </span>
                <span className="num text-dim">
                  {t('contas-parcelas-pagas', {
                    pagas: p.paid_installments,
                    total: p.installments,
                  })}
                </span>
                <Money cents={p.total_amount} currency={p.currency} className="text-right" />
                <span className="text-right">
                  <span className="label-caps mr-1">{t('contas-falta')}</span>
                  <Money cents={p.remaining_amount} currency={p.currency} />
                </span>
                <Button size="sm" variant="ghost" onClick={() => setDeleting(p.id)}>
                  {t('comum-apagar')}
                </Button>
              </li>
            ))}
          </ul>
        )}
      </QueryState>
      <ConfirmDialog
        open={Boolean(deleting)}
        onOpenChange={(o) => !o && setDeleting(null)}
        title={t('contas-apagar-parcelamento')}
        description={t('contas-apagar-parcelamento-descricao')}
        confirmLabel={t('contas-apagar-parcelas')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { id: deleting },
            {
              onSuccess: (r) => (
                toast({
                  tone: 'success',
                  title: t('contas-n-parcelas-apagadas', { n: r.updated }),
                }),
                setDeleting(null)
              ),
            },
          )
        }
      />
    </div>
  );
}
