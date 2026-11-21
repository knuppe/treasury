import { useState } from 'react';
import { Link } from 'react-router';

import type { AccountView } from '@/api/generated/AccountView';
import type { AnnualFeeStatus } from '@/api/generated/AnnualFeeStatus';
import type { FeeIssue } from '@/api/generated/FeeIssue';
import type { WaiverKind } from '@/api/generated/WaiverKind';
import {
  useAddLimitChange,
  useCardOverview,
  useDeleteLimitChange,
  useEndAnnualFee,
  useSetAnnualFee,
} from '@/api/hooks/cards';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, todayIso } from '@/lib/format';
import { txnLink } from '@/lib/txnLink';
import {
  ConfirmDialog,
  InlineError,
  QueryState,
  SectionTitle,
} from '@/features/transactions/ui/controls';

const WAIVER: Record<WaiverKind, () => string> = {
  none: () => t('contas-anuidade-isencao-none'),
  full: () => t('contas-anuidade-isencao-full'),
  spend: () => t('contas-anuidade-isencao-spend'),
  investment: () => t('contas-anuidade-isencao-investment'),
  discount: () => t('contas-anuidade-isencao-discount'),
};

const ISSUE: Record<FeeIssue, () => string> = {
  charged_while_waived: () => t('contas-anuidade-problema-isenta'),
  amount_differs: () => t('contas-anuidade-problema-valor'),
  extra_installment: () => t('contas-anuidade-problema-parcela'),
};

const pct = (bp: number) => `${Math.round(bp / 100)}%`;

/** Barra de uso do limite, com a parte das parcelas futuras. */
function UsageBar({ used, future, limit }: { used: number; future: number; limit: number }) {
  const share = (v: number) => `${Math.min(100, Math.max(0, (v / limit) * 100))}%`;
  const ratio = used / limit;
  return (
    <div
      className="relative h-2.5 w-full overflow-hidden rounded-full bg-line/60"
      role="meter"
      aria-valuemin={0}
      aria-valuemax={limit}
      aria-valuenow={used}
      aria-label={t('contas-limite-uso')}
    >
      <div
        className={cx(
          'absolute inset-y-0 left-0',
          ratio >= 0.95 ? 'bg-negative' : ratio >= 0.8 ? 'bg-warning' : 'bg-accent',
        )}
        style={{ width: share(used) }}
      />
      <div
        className="absolute inset-y-0 bg-fg/25"
        style={{ left: share(used - future), width: share(future) }}
      />
    </div>
  );
}

function LimitDialog({ card, onClose }: { card: AccountView; onClose: () => void }) {
  const add = useAddLimitChange();
  const [date, setDate] = useState<string | null>(todayIso());
  const [limit, setLimit] = useState<number | null>(card.card?.credit_limit ?? null);
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={t('contas-limite-novo')}
      description={t('contas-limite-novo-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={add.isPending}
            disabled={!date || limit == null}
            onClick={() =>
              add.mutate(
                { account_id: card.id, date: date!, credit_limit: limit! },
                {
                  onSuccess: () => {
                    toast({ tone: 'success', title: t('contas-limite-salvo') });
                    onClose();
                  },
                },
              )
            }
          >
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('contas-limite-moeda', { moeda: card.currency })}>
          <MoneyInput currency={card.currency} value={limit} onValueChange={setLimit} />
        </Field>
      </div>
      <InlineError error={add.error} />
    </Dialog>
  );
}

function FeeDialog({
  card,
  status,
  onClose,
}: {
  card: AccountView;
  status: AnnualFeeStatus | null;
  onClose: () => void;
}) {
  const save = useSetAnnualFee();
  const rule = status?.rule;
  const [amount, setAmount] = useState<number | null>(rule?.amount ?? null);
  const [installments, setInstallments] = useState(String(rule?.installments ?? 12));
  const [month, setMonth] = useState(String(rule?.anniversary_month ?? ''));
  const [kind, setKind] = useState<WaiverKind>(rule?.waiver_kind ?? 'none');
  const [threshold, setThreshold] = useState<number | null>(rule?.waiver_threshold ?? null);
  const [discount, setDiscount] = useState(
    rule?.waiver_discount ? String(Math.round(Number(rule.waiver_discount) * 100)) : '',
  );
  const [starts, setStarts] = useState<string | null>(todayIso());
  const [notes, setNotes] = useState(rule?.notes ?? '');
  return (
    <Dialog
      open
      onOpenChange={(o) => (o ? null : onClose())}
      index="02"
      size="sm"
      title={t('contas-anuidade-regra')}
      description={t('contas-anuidade-regra-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={save.isPending}
            disabled={amount == null}
            onClick={() =>
              save.mutate(
                {
                  account_id: card.id,
                  amount: amount!,
                  installments: Number(installments),
                  ...(month ? { anniversary_month: Number(month) } : {}),
                  waiver_kind: kind,
                  ...(threshold != null && (kind === 'spend' || kind === 'investment')
                    ? { waiver_threshold: threshold }
                    : {}),
                  ...(kind === 'discount' && discount
                    ? { waiver_discount: String(Number(discount) / 100) }
                    : {}),
                  ...(starts ? { starts_on: starts } : {}),
                  ...(notes.trim() ? { notes: notes.trim() } : {}),
                },
                {
                  onSuccess: () => {
                    toast({ tone: 'success', title: t('contas-anuidade-salva') });
                    onClose();
                  },
                },
              )
            }
          >
            {t('contas-plastico-salvar')}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-2 gap-3">
        <Field label={t('contas-anuidade-valor-ano')}>
          <MoneyInput currency={card.currency} value={amount} onValueChange={setAmount} />
        </Field>
        <Field label={t('contas-anuidade-parcelas')}>
          <Select
            value={installments}
            onValueChange={setInstallments}
            options={Array.from({ length: 12 }, (_, i) => ({
              value: String(i + 1),
              label: `${i + 1}×`,
            }))}
          />
        </Field>
        <Field label={t('contas-anuidade-mes')}>
          <Select
            value={month || '0'}
            onValueChange={(v) => setMonth(v === '0' ? '' : v)}
            options={[
              { value: '0', label: t('contas-anuidade-mes-nao-sei') },
              ...Array.from({ length: 12 }, (_, i) => ({
                value: String(i + 1),
                label: formatMonthLong(`2026-${String(i + 1).padStart(2, '0')}`).split(' ')[0]!,
              })),
            ]}
          />
        </Field>
        <Field label={t('contas-anuidade-inicio')}>
          <DateInput value={starts} onValueChange={setStarts} />
        </Field>
      </div>
      <Field label={t('contas-anuidade-isencao')}>
        <Select
          value={kind}
          onValueChange={(v) => setKind(v as WaiverKind)}
          options={(Object.keys(WAIVER) as WaiverKind[]).map((k) => ({
            value: k,
            label: WAIVER[k](),
          }))}
        />
      </Field>
      {kind === 'spend' || kind === 'investment' ? (
        <Field
          label={
            kind === 'spend'
              ? t('contas-anuidade-gasto-minimo')
              : t('contas-anuidade-investimento-minimo')
          }
        >
          <MoneyInput currency={card.currency} value={threshold} onValueChange={setThreshold} />
        </Field>
      ) : null}
      {kind === 'discount' ? (
        <Field label={t('contas-anuidade-desconto')}>
          <Input
            value={discount}
            inputMode="numeric"
            onChange={(e) => setDiscount(e.target.value.replace(/\D/g, '').slice(0, 3))}
            placeholder="50"
          />
        </Field>
      ) : null}
      <Field label={t('contas-anuidade-notas')}>
        <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
      </Field>
      <InlineError error={save.error} />
    </Dialog>
  );
}

function FeeSection({ card, status }: { card: AccountView; status: AnnualFeeStatus | null }) {
  const [editing, setEditing] = useState(false);
  const [ending, setEnding] = useState(false);
  const end = useEndAnnualFee();
  const cur = card.currency;
  return (
    <section className="flex flex-col gap-3" aria-label={t('contas-anuidade')}>
      <SectionTitle
        aside={
          <span className="flex gap-2">
            <Button size="sm" variant="secondary" onClick={() => setEditing(true)}>
              {status ? t('contas-anuidade-mudar') : t('contas-anuidade-informar')}
            </Button>
            {status ? (
              <Button size="sm" variant="ghost" onClick={() => setEnding(true)}>
                {t('contas-anuidade-sem')}
              </Button>
            ) : null}
          </span>
        }
      >
        {t('contas-anuidade')}
      </SectionTitle>
      {!status ? (
        <p className="max-w-[70ch] text-[13px] text-dim">{t('contas-anuidade-vazio')}</p>
      ) : (
        <>
          <KeyValue
            items={[
              {
                label: t('contas-anuidade-valor-ano'),
                value: (
                  <span>
                    <Money cents={status.rule.amount} currency={cur} />{' '}
                    <span className="text-dim">
                      {t('contas-anuidade-em-parcelas', { n: status.rule.installments })}{' '}
                      <Money cents={status.installment_amount} currency={cur} />
                    </span>
                  </span>
                ),
              },
              { label: t('contas-anuidade-isencao'), value: WAIVER[status.waiver.kind]() },
              {
                label: t('contas-anuidade-pago-12m'),
                value: <Money cents={status.paid_12m} currency={cur} />,
              },
              {
                label: t('contas-anuidade-renova'),
                value: status.next_renewal
                  ? formatDate(status.next_renewal)
                  : t('contas-anuidade-mes-nao-sei'),
              },
            ]}
          />
          {status.waiver.kind === 'spend' || status.waiver.kind === 'investment' ? (
            <p
              className={cx('text-[13px]', status.waiver.met ? 'text-positive' : 'text-warning')}
              role="status"
            >
              {status.waiver.met ? (
                t('contas-anuidade-isenta-agora')
              ) : (
                <>
                  {t(
                    status.waiver.kind === 'spend'
                      ? 'contas-anuidade-falta-gastar'
                      : 'contas-anuidade-falta-investir',
                  )}{' '}
                  <Money cents={status.waiver.remaining} currency={cur} />
                </>
              )}
            </p>
          ) : null}
          {status.charges.length > 0 ? (
            <ul
              className="flex flex-col divide-y divide-line/60 rounded-card border border-line"
              aria-label={t('contas-anuidade-cobrancas')}
            >
              {status.charges.map((c) => (
                <li
                  key={c.txn_id}
                  className="flex flex-wrap items-baseline gap-x-4 gap-y-1 px-3 py-1.5 text-[12.5px]"
                >
                  <span className="num text-dim">{formatDate(c.date)}</span>
                  <Link
                    to={txnLink({ text: c.description, account_ids: [card.id] })}
                    className="truncate hover:text-accent"
                  >
                    {c.description}
                  </Link>
                  <Money cents={c.amount} currency={cur} className="ml-auto" />
                  {c.issue ? (
                    <span className="w-full text-warning">
                      {ISSUE[c.issue]()}
                      {c.expected != null ? (
                        <>
                          {' '}
                          {t('contas-anuidade-esperado')}{' '}
                          <Money cents={c.expected} currency={cur} />
                        </>
                      ) : null}
                    </span>
                  ) : null}
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-[12.5px] text-dim">{t('contas-anuidade-sem-cobrancas')}</p>
          )}
        </>
      )}
      {editing ? <FeeDialog card={card} status={status} onClose={() => setEditing(false)} /> : null}
      {ending ? (
        <ConfirmDialog
          open
          tone="primary"
          onOpenChange={(o) => (o ? null : setEnding(false))}
          title={t('contas-anuidade-sem-titulo')}
          description={t('contas-anuidade-sem-descricao')}
          confirmLabel={t('contas-anuidade-sem')}
          pending={end.isPending}
          error={end.error}
          onConfirm={() =>
            end.mutate({ account_id: card.id }, { onSuccess: () => setEnding(false) })
          }
        />
      ) : null}
    </section>
  );
}

/** Limite e anuidade do cartão: uso, melhor dia de compra, histórico do limite e anuidade. */
export function CardLimitPanel({ card }: { card: AccountView }) {
  const { data, isLoading, error } = useCardOverview(card.id);
  const remove = useDeleteLimitChange();
  const [adding, setAdding] = useState(false);
  return (
    <QueryState loading={isLoading} error={error}>
      {data ? (
        <div className="flex flex-col gap-5">
          {data.revolving ? (
            <section
              role="alert"
              className="flex flex-col gap-1 rounded-card border border-negative/50 p-3 text-[13px]"
            >
              <strong className="text-negative">
                {t('contas-rotativo-titulo', {
                  mes: formatMonthLong(data.revolving.reference_month),
                })}
              </strong>
              <span>
                {t('contas-rotativo-sem-pagar')}{' '}
                <Money cents={data.revolving.unpaid} currency={data.revolving.currency} />
                {' · '}
                {t('contas-rotativo-encargos')}{' '}
                <Money cents={data.revolving.charges} currency={data.revolving.currency} />{' '}
                <span className="text-dim">({pct(data.revolving.charges_bp)})</span>
              </span>
              <span className="text-dim">
                {data.brazil_rules ? t('contas-rotativo-regras-br') : t('contas-rotativo-regras')}
              </span>
              {data.brazil_rules && data.revolving.over_cap ? (
                <span className="text-negative">{t('contas-rotativo-acima-teto')}</span>
              ) : null}
              {data.brazil_rules && data.revolving.beyond_next_statement ? (
                <span className="text-warning">{t('contas-rotativo-alem-da-fatura')}</span>
              ) : null}
            </section>
          ) : null}
          <section className="flex flex-col gap-2" aria-label={t('contas-limite-uso')}>
            {data.credit_limit != null && data.credit_limit > 0 ? (
              <>
                <div className="flex flex-wrap items-baseline justify-between gap-2 text-[13px]">
                  <span>
                    {t('contas-limite-usado')} <Money cents={data.used} currency={data.currency} />{' '}
                    <span className="text-dim">
                      {t('contas-limite-de-total')}{' '}
                      <Money cents={data.credit_limit} currency={data.currency} />
                    </span>
                  </span>
                  <span
                    className={cx(
                      'num',
                      (data.utilization_bp ?? 0) >= 9500
                        ? 'text-negative'
                        : (data.utilization_bp ?? 0) >= 8000
                          ? 'text-warning'
                          : 'text-dim',
                    )}
                  >
                    {pct(data.utilization_bp ?? 0)}
                  </span>
                </div>
                <UsageBar
                  used={data.used}
                  future={data.committed_installments}
                  limit={data.credit_limit}
                />
                <p className="text-[12.5px] text-dim">
                  {t('contas-limite-parcelas-futuras')}{' '}
                  <Money cents={data.committed_installments} currency={data.currency} />
                  {' · '}
                  {t('contas-limite-disponivel')}{' '}
                  <Money cents={data.available ?? 0} currency={data.currency} />
                </p>
              </>
            ) : (
              <p className="text-[13px] text-dim">{t('contas-limite-sem')}</p>
            )}
            <p className="text-[13px]">
              {t('contas-melhor-dia', {
                data: formatDate(data.best_purchase_date),
                vencimento: formatDate(data.best_purchase_due_date),
                dias: data.best_purchase_days,
              })}
            </p>
          </section>

          <section className="flex flex-col gap-2" aria-label={t('contas-limite-historico')}>
            <SectionTitle
              aside={
                <Button size="sm" variant="secondary" onClick={() => setAdding(true)}>
                  {t('contas-limite-novo')}
                </Button>
              }
            >
              {t('contas-limite-historico')}
            </SectionTitle>
            {data.limit_history.length === 0 ? (
              <p className="text-[12.5px] text-dim">{t('contas-limite-historico-vazio')}</p>
            ) : (
              <ul className="flex flex-col divide-y divide-line/60 rounded-card border border-line">
                {data.limit_history.map((c) => (
                  <li key={c.id} className="flex items-center gap-4 px-3 py-1.5 text-[12.5px]">
                    <span className="num w-24 text-dim">{formatDate(c.date)}</span>
                    <Money cents={c.credit_limit} currency={data.currency} />
                    <span className="label-caps">
                      {c.source === 'import'
                        ? t('contas-limite-da-fatura')
                        : t('contas-limite-informado')}
                    </span>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="ml-auto"
                      loading={remove.isPending && remove.variables?.id === c.id}
                      onClick={() => remove.mutate({ id: c.id })}
                    >
                      {t('contas-remover')}
                    </Button>
                  </li>
                ))}
              </ul>
            )}
            <InlineError error={remove.error} />
          </section>

          <FeeSection card={card} status={data.annual_fee} />
          {adding ? <LimitDialog card={card} onClose={() => setAdding(false)} /> : null}
        </div>
      ) : null}
    </QueryState>
  );
}
