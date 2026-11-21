/**
 * Aba Previdência de um ativo da classe Previdência (VGBL, PGBL ou fundo de pensão): o plano (tipo, regime de
 * tributação, seguradora e taxas), os aportes com a faixa de IR de cada um e a simulação
 * de resgate (bruto, IR e líquido). Os aportes e resgates são as operações do ativo; o
 * valor vem da cota (CVM pelo CNPJ do fundo ou cota informada). O extrato do plano em PDF
 * pode ser importado (docs/brazil/pension-statement.md).
 */
import { useState, type FormEvent } from 'react';

import type { Asset } from '@/api/generated/Asset';
import type { PensionView } from '@/api/generated/PensionView';
import type { WithdrawalSim } from '@/api/generated/WithdrawalSim';
import {
  simulateWithdrawal,
  usePension,
  useSetPensionBalance,
  useSetPensionPlan,
} from '@/api/hooks/pension';
import {
  Button,
  DateInput,
  Field,
  InstrumentCard,
  KeyValue,
  Money,
  MoneyInput,
  TechTable,
  toast,
} from '@/design/components';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';

import { formatDecimal } from '../portfolio/decimal';
import { PensionPlanFields, planDraft, planTerms, type PlanDraft } from './PensionPlanFields';
import { PensionStatementImport } from './PensionStatementImport';

function SimResult({ sim }: { sim: WithdrawalSim }) {
  return (
    <div className="flex flex-col gap-2">
      <KeyValue
        items={[
          { label: t('ativos-prev-bruto'), value: <Money cents={sim.gross} /> },
          { label: t('ativos-prev-base-ir'), value: <Money cents={sim.tax_base} /> },
          { label: t('ativos-termos-ir'), value: <Money cents={sim.tax} /> },
          { label: t('ativos-prev-liquido'), value: <Money cents={sim.net} /> },
          {
            label: t('ativos-prev-aliquota'),
            value: `${formatDecimal(sim.effective_rate_pct)}%`,
          },
        ]}
      />
      <p className="text-[12px] text-dim">{sim.note}</p>
    </div>
  );
}

function PlanForm({ asset, view }: { asset: Asset; view: PensionView }) {
  const save = useSetPensionPlan();
  const [draft, setDraft] = useState<PlanDraft>(() => planDraft(view.plan));
  const submit = (e: FormEvent) => {
    e.preventDefault();
    save.mutate(
      { asset_id: asset.id, ...planTerms(draft) },
      { onSuccess: () => toast({ tone: 'success', title: t('ativos-prev-plano-salvo') }) },
    );
  };
  return (
    <form onSubmit={submit} className="flex flex-col gap-4" aria-label={t('ativos-form-plano')}>
      <PensionPlanFields value={draft} onChange={setDraft} />
      <InlineError error={save.error} />
      <div>
        <Button type="submit" variant="primary" loading={save.isPending}>
          {t('ativos-prev-salvar-plano')}
        </Button>
      </div>
    </form>
  );
}

/**
 * O plano é o ativo; as cotas ficam numa conta (a custódia). Mostra qual conta guarda as
 * cotas, ou que ainda não há uma (a importação do histórico de pagamentos cria), e os aportes
 * conhecidos que ainda esperam a cota da CVM para virar cotas.
 */
function PlanCustody({ view }: { view: PensionView }) {
  return (
    <div
      className="flex flex-col gap-1 rounded-card border border-line px-3 py-2 text-[12.5px]"
      aria-label={t('ativos-prev-custodia')}
    >
      <p>
        {view.accounts.length
          ? t('ativos-prev-custodia-conta', {
              n: view.accounts.length,
              contas: view.accounts.map((a) => a.name).join(', '),
            })
          : t('ativos-prev-custodia-sem-conta')}
      </p>
      {view.pending_payments > 0 ? (
        <p className="text-warning">
          {withSlots(
            t('ativos-prev-aportes-pendentes', {
              n: view.pending_payments,
              valor: slot(0),
              data: view.pending_since ? formatDate(view.pending_since) : '—',
            }),
            [<Money key="v" cents={view.pending_amount} />],
          )}
        </p>
      ) : null}
    </div>
  );
}

/**
 * Os aportes lidos do histórico de pagamentos da seguradora: os que já viraram cotas e os
 * que esperam a cota da CVM (com o valor aportado), e o que os prints não fecharam.
 */
function PaymentHistory({ view }: { view: PensionView }) {
  return (
    <InstrumentCard index="03b" title={t('ativos-prev-pagamentos-titulo')}>
      {view.payment_issues.length ? (
        <ul
          className="mb-3 flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
          aria-label={t('ativos-prev-pagamentos-pendencias')}
        >
          {view.payment_issues.map((i) => (
            <li key={`${i.kind}-${i.date ?? ''}-${i.amount ?? ''}`}>{i.message}</li>
          ))}
        </ul>
      ) : null}
      <TechTable
        label={t('ativos-prev-pagamentos-tabela')}
        prefsId="Pagamentos da previdência"
        dense
        rows={view.payments}
        getRowId={(p) => p.id}
        empty={t('ativos-prev-nenhum-aporte')}
        columns={[
          { key: 'date', header: t('ativos-prev-aporte'), cell: (p) => formatDate(p.date) },
          {
            key: 'amount',
            header: t('ativos-prev-aportado'),
            numeric: true,
            cell: (p) => <Money cents={p.amount} />,
          },
          {
            key: 'status',
            header: t('ativos-prev-pagamento-situacao'),
            cell: (p) =>
              p.status === 'bought'
                ? t('ativos-prev-pagamento-cotas', { cotas: formatDecimal(p.quantity ?? '0') })
                : t('ativos-prev-pagamento-sem-cota'),
          },
          { key: 'account', header: t('ativos-prev-pagamento-conta'), cell: (p) => p.account_name },
        ]}
      />
    </InstrumentCard>
  );
}

/**
 * Saldo de hoje informado à mão: para planos sem cota pública (previdência fechada, como a
 * de cooperativa), o que a entidade mostra vira a cota do dia e o valor do plano.
 */
function BalanceForm({ asset }: { asset: Asset }) {
  const save = useSetPensionBalance();
  const [date, setDate] = useState<string | null>(todayIso());
  const [balance, setBalance] = useState<number | null>(null);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!date || !balance) return;
    save.mutate(
      { asset_id: asset.id, date, balance },
      {
        onSuccess: () => {
          toast({ tone: 'success', title: t('ativos-prev-saldo-salvo') });
          setBalance(null);
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      className="mt-3 flex flex-col gap-2 border-t border-line pt-3"
      aria-label={t('ativos-prev-informar-saldo')}
    >
      <p className="label-caps">{t('ativos-prev-informar-saldo')}</p>
      <p className="text-[12px] text-dim">{t('ativos-prev-informar-saldo-dica')}</p>
      <div className="flex flex-wrap items-end gap-2">
        <Field label={t('ativos-prev-saldo-data')} className="w-40">
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('ativos-prev-saldo-valor')} className="w-44">
          <MoneyInput value={balance} onValueChange={setBalance} currency={asset.currency} />
        </Field>
        <Button
          type="submit"
          variant="secondary"
          loading={save.isPending}
          disabled={!date || !balance}
        >
          {t('ativos-prev-salvar-saldo')}
        </Button>
      </div>
      <InlineError error={save.error} />
    </form>
  );
}

export function PensionPanel({ asset }: { asset: Asset }) {
  const q = usePension(asset.id);
  const [amount, setAmount] = useState<number | null>(null);
  const [sim, setSim] = useState<WithdrawalSim | null>(null);
  const [simError, setSimError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const runSim = (e: FormEvent) => {
    e.preventDefault();
    if (!amount) return;
    setBusy(true);
    setSimError(null);
    simulateWithdrawal({ asset_id: asset.id, amount })
      .then(setSim)
      .catch(setSimError)
      .finally(() => setBusy(false));
  };
  return (
    <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
      {q.data ? (
        <div className="flex flex-col gap-5">
          <div className="flex justify-end">
            <PensionStatementImport asset={asset} />
          </div>
          {q.data.warnings.length ? (
            <ul
              className="flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
              aria-label={t('ativos-prev-avisos')}
            >
              {q.data.warnings.map((w) => (
                <li key={w} className="first-letter:uppercase">
                  {w}.
                </li>
              ))}
            </ul>
          ) : null}
          <PlanCustody view={q.data} />
          <div className="grid gap-4 @min-[880px]:grid-cols-2">
            <InstrumentCard
              index="01"
              title={t('ativos-prev-saldo')}
              footer={[
                {
                  label: t('ativos-prev-cota'),
                  value: q.data.price
                    ? q.data.price_date
                      ? t('ativos-cotacao-informada-em', {
                          preco: formatDecimal(q.data.price),
                          data: formatDate(q.data.price_date),
                        })
                      : formatDecimal(q.data.price)
                    : '—',
                },
              ]}
            >
              <KeyValue
                items={[
                  { label: t('ativos-prev-aportado'), value: <Money cents={q.data.contributed} /> },
                  { label: t('ativos-prev-valor-hoje'), value: <Money cents={q.data.value} /> },
                  {
                    label: t('ativos-prev-rendimento'),
                    value: <Money cents={q.data.gain} tone="auto" signed />,
                  },
                ]}
              />
              <div className="mt-3 border-t border-line pt-3">
                <p className="label-caps mb-2">{t('ativos-prev-resgatar-tudo')}</p>
                <SimResult sim={q.data.full_withdrawal} />
              </div>
              <BalanceForm asset={asset} />
              {q.data.changing_band_soon > 0 ? (
                <p className="mt-2 text-[12.5px] text-accent">
                  {withSlots(t('ativos-prev-mudam-faixa', { valor: slot(0) }), [
                    <Money key="v" cents={q.data.changing_band_soon} />,
                  ])}
                </p>
              ) : null}
            </InstrumentCard>
            <InstrumentCard index="02" title={t('ativos-prev-plano')}>
              <PlanForm key={q.data.plan?.plan_type ?? 'novo'} asset={asset} view={q.data} />
            </InstrumentCard>
          </div>
          <InstrumentCard index="03" title={t('ativos-prev-aportes-titulo')}>
            <TechTable
              label={t('ativos-prev-aportes-tabela')}
              prefsId="Aportes da previdência"
              dense
              rows={q.data.lots}
              getRowId={(l) => `${l.date}-${l.quantity}`}
              empty={t('ativos-prev-nenhum-aporte')}
              columns={[
                { key: 'date', header: t('ativos-prev-aporte'), cell: (l) => formatDate(l.date) },
                {
                  key: 'cost',
                  header: t('ativos-prev-aportado'),
                  numeric: true,
                  cell: (l) => <Money cents={l.cost} />,
                },
                {
                  key: 'value',
                  header: t('ativos-prev-valor-hoje'),
                  numeric: true,
                  cell: (l) => <Money cents={l.value} />,
                },
                {
                  key: 'gain',
                  header: t('ativos-prev-rendimento'),
                  numeric: true,
                  cell: (l) => <Money cents={l.gain} tone="auto" signed />,
                },
                {
                  key: 'regressive_rate_pct',
                  header: t('ativos-prev-ir-hoje'),
                  numeric: true,
                  cell: (l) => `${l.regressive_rate_pct}%`,
                },
                {
                  key: 'next_rate_date',
                  header: t('ativos-prev-proxima-faixa'),
                  cell: (l) =>
                    l.next_rate_date
                      ? t('ativos-prev-proxima-em', {
                          pct: `${l.next_rate_pct}%`,
                          data: formatDate(l.next_rate_date),
                        })
                      : t('ativos-prev-ja-menor'),
                },
              ]}
            />
          </InstrumentCard>
          {q.data.payments.length || q.data.payment_issues.length ? (
            <PaymentHistory view={q.data} />
          ) : null}
          <InstrumentCard index="04" title={t('ativos-prev-simular-titulo')}>
            <form
              onSubmit={runSim}
              className="flex flex-wrap items-end gap-3"
              aria-label={t('ativos-prev-simular-titulo')}
            >
              <Field label={t('ativos-prev-valor-bruto')} className="w-56">
                <MoneyInput value={amount} onValueChange={setAmount} />
              </Field>
              <Button type="submit" variant="secondary" loading={busy} disabled={!amount}>
                {t('ativos-prev-simular')}
              </Button>
            </form>
            <p className="mt-2 text-[12px] text-dim">{t('ativos-prev-simular-dica')}</p>
            <InlineError error={simError} />
            {sim ? (
              <div className="mt-3" role="status" aria-label={t('ativos-prev-resultado')}>
                <SimResult sim={sim} />
              </div>
            ) : null}
          </InstrumentCard>
        </div>
      ) : null}
    </QueryState>
  );
}
