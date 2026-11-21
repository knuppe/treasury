/**
 * Detalhe de um contrato: medidas (saldo, próxima parcela, taxa, CET, custo realizado e
 * real), figuras do saldo e da composição das parcelas, simulador de amortização e a
 * tabela de amortização, com as ações de editar, amortizar, conciliar, arquivar e excluir.
 */
import { useCountryFeatures } from '@/api/hooks/country';
import { useCallback, useState } from 'react';
import { useUndoToast } from '@/api/hooks/undoToast';

import {
  useDeleteLoan,
  useLoanSchedule,
  useMarkPaidUntil,
  useReconcileLoan,
  useUpdateLoan,
} from '@/api/hooks/debt';
import { useAccounts } from '@/api/hooks/accounts';
import { useCategories } from '@/api/hooks/categories';
import { useInstitutions, useMembers } from '@/api/hooks/household';
import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import { Button, Icon, InstrumentCard, KeyValue, Measure, Money, toast } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { Pct } from '@/features/portfolio/components/Pct';
import { ConfirmDialog, QueryState } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';

import { AnticipateCard } from './AnticipateCard';
import { BalanceFigure, BreakdownFigure } from './DebtFigures';
import { INDEXER_LABEL, isPostIndexer, KIND_LABEL, SYSTEM_LABEL } from './labels';
import { ExtraPaymentDialog, LoanFormDialog, type ExtraPreset } from './LoanDialogs';
import { StatementDialog } from './StatementDialog';
import { PortabilitySimulator } from './PortabilitySimulator';
import { PrepaySimulator, type PrepayChoice } from './PrepaySimulator';
import { ScheduleTable } from './ScheduleTable';
import { loanRateLabel, progress, rateText, termText } from './transform';

/** Conciliar e arquivar um contrato (botões do detalhe e menu de contexto da lista). */
export function useLoanActions() {
  const accounts = useAccounts().data ?? [];
  const reconcile = useReconcileLoan();
  const update = useUpdateLoan();
  const undoToast = useUndoToast();
  const canReconcile = (loan: LoanView) => Boolean(loan.debit_account_id || loan.category_id);
  const doReconcile = (loan: LoanView) => {
    const account = accounts.find((a) => a.id === loan.debit_account_id)?.name;
    reconcile.mutate(
      { id: loan.id },
      {
        onSuccess: (r) =>
          undoToast({
            tone: r.matched > 0 ? 'success' : 'info',
            title:
              r.matched > 0
                ? t('dividas-conciliadas', { n: r.matched })
                : t('dividas-conciliar-nada'),
            description: t('dividas-conciliar-procurado', {
              conta: account ?? t('dividas-conta-de-debito-min'),
            }),
          }),
        onError: (e) =>
          toast({ tone: 'error', title: t('dividas-conciliar-erro'), description: e.message }),
      },
    );
  };
  const toggleArchived = (loan: LoanView) =>
    update.mutate(
      { id: loan.id, archived: !loan.archived },
      {
        onSuccess: () =>
          undoToast({
            title: loan.archived ? t('dividas-reativado') : t('dividas-arquivado-ok'),
          }),
      },
    );
  return {
    canReconcile,
    doReconcile,
    reconciling: reconcile.isPending,
    toggleArchived,
    archiving: update.isPending,
  };
}

/** Pedido vindo de fora do detalhe (menu de contexto da lista) para abrir um diálogo. */
export interface LoanIntent {
  loanId: string;
  kind: 'edit' | 'extra' | 'delete';
  /** Cada pedido é novo, mesmo repetindo o tipo. */
  seq: number;
}

export function LoanDetail({
  loan,
  onDeleted,
  intent,
}: {
  loan: LoanView;
  onDeleted: () => void;
  intent?: LoanIntent | null;
}) {
  const schedule = useLoanSchedule(loan.id);
  // Portabilidade e juros de mercado do Banco Central: só no Brasil (docs/countries.md).
  const features = useCountryFeatures();
  const institutions = useInstitutions().data ?? [];
  const members = useMembers().data ?? [];
  const accounts = useAccounts().data ?? [];
  const categories = useCategories({ include_archived: true }).data ?? [];
  const actions = useLoanActions();
  const markPaidUntil = useMarkPaidUntil();
  const remove = useDeleteLoan();
  const undoToast = useUndoToast();
  const [editing, setEditing] = useState(false);
  const [statementOpen, setStatementOpen] = useState(false);
  const [extra, setExtra] = useState<ExtraPreset | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [simulated, setSimulated] = useState<readonly Installment[] | null>(null);
  const onSimulated = useCallback((s: readonly Installment[] | null) => setSimulated(s), []);
  const [seenIntent, setSeenIntent] = useState(0);
  if (intent && intent.loanId === loan.id && intent.seq !== seenIntent) {
    setSeenIntent(intent.seq);
    if (intent.kind === 'edit') setEditing(true);
    else if (intent.kind === 'extra') setExtra({});
    else setConfirmDelete(true);
  }

  const institution = institutions.find((i) => i.id === loan.institution_id)?.name;
  const member = members.find((m) => m.id === loan.member_id)?.name;
  const account = accounts.find((a) => a.id === loan.debit_account_id)?.name;
  const category = categories.find((c) => c.id === loan.category_id)?.path;
  const canReconcile = actions.canReconcile(loan);
  const next = loan.next_installment;
  const p = progress(loan);
  const installments = schedule.data ?? [];

  return (
    <section
      aria-label={t('dividas-contrato-aria', { nome: loan.name })}
      className="flex flex-col gap-5"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-1">
          <h2 className="text-[18px] font-medium text-fg">{loan.name}</h2>
          <p className="text-[13px] text-dim">
            {[
              KIND_LABEL[loan.kind],
              institution,
              member ? t('dividas-de-membro', { nome: member }) : null,
              loan.contract_number
                ? t('dividas-contrato-numero-de-data', {
                    numero: loan.contract_number,
                    data: formatDate(loan.contract_date),
                  })
                : t('dividas-contrato-de-data', { data: formatDate(loan.contract_date) }),
              loan.archived ? t('dividas-arquivado') : null,
            ]
              .filter(Boolean)
              .join(' · ')}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="ghost" onClick={() => setEditing(true)}>
            {t('comum-editar')}
          </Button>
          <Button variant="ghost" onClick={() => setStatementOpen(true)}>
            {t('dividas-importar-extrato')}
          </Button>
          <Button
            variant="ghost"
            loading={actions.reconciling}
            disabled={!canReconcile}
            title={canReconcile ? undefined : t('dividas-conciliar-sem-conta')}
            onClick={() => actions.doReconcile(loan)}
          >
            {t('dividas-conciliar')}
          </Button>
          <Button
            variant="ghost"
            loading={actions.archiving}
            onClick={() => actions.toggleArchived(loan)}
          >
            {loan.archived ? t('dividas-reativar') : t('dividas-arquivar')}
          </Button>
          <Button variant="ghost" onClick={() => setConfirmDelete(true)}>
            {t('dividas-excluir')}
          </Button>
          <Button
            variant="primary"
            icon={<Icon name="plus" size={12} />}
            onClick={() => setExtra({})}
          >
            {t('dividas-amortizacao-extra')}
          </Button>
        </div>
      </div>

      {loan.overdue_count > 0 ? (
        <Notice tone="warning" title={t('dividas-parcelas-atrasadas', { n: loan.overdue_count })}>
          {t('dividas-atrasadas-texto', { n: loan.overdue_count })}
          {loan.overdue_count > 1 ? (
            <span className="mt-2 flex">
              <Button
                size="sm"
                variant="secondary"
                loading={markPaidUntil.isPending}
                onClick={() =>
                  markPaidUntil.mutate(
                    { loan_id: loan.id, before: todayIso() },
                    {
                      onSuccess: (r) =>
                        undoToast({
                          tone: 'success',
                          title: t('dividas-marcadas-titulo'),
                          description: t('dividas-marcadas-descricao', { n: r.marked }),
                        }),
                    },
                  )
                }
              >
                {t('dividas-marcar-vencidas', { n: loan.overdue_count })}
              </Button>
            </span>
          ) : null}
        </Notice>
      ) : null}

      <div className="grid gap-4 @min-[530px]:grid-cols-2 @min-[1040px]:grid-cols-4">
        <InstrumentCard index="06.1" title={t('dividas-saldo-devedor')} headingLevel={3}>
          <Measure
            label={t('dividas-hoje')}
            size="md"
            value={<Money cents={loan.balance} />}
            hint={withSlots(t('dividas-de-financiados', { valor: slot(0) }), [
              <Money key="p" cents={loan.principal} />,
            ])}
          />
          <ProgressBar
            value={p}
            label={t('dividas-parcelas-pagas-de', {
              pagas: loan.paid_count,
              total: loan.paid_count + loan.remaining_count,
            })}
          />
        </InstrumentCard>
        <InstrumentCard index="06.2" title={t('dividas-proxima-parcela')} headingLevel={3}>
          {next ? (
            <Measure
              label={t('dividas-numero-data', { n: next.number, data: formatDate(next.due_date) })}
              size="md"
              value={<Money cents={next.total} estimated={next.estimated} />}
              hint={
                next.overdue ? (
                  <span className="text-negative">{t('dividas-atrasada-alerta')}</span>
                ) : (
                  t('dividas-em-aberto', { prazo: termText(loan.remaining_count) })
                )
              }
            />
          ) : (
            <Measure label={t('dividas-contrato')} size="md" value={t('dividas-quitado')} />
          )}
        </InstrumentCard>
        <InstrumentCard index="06.3" title={t('dividas-taxa-custo')} headingLevel={3}>
          <KeyValue
            items={[
              {
                label: t('dividas-taxa'),
                value: isPostIndexer(loan.indexer)
                  ? loanRateLabel(loan)
                  : `${t('dividas-taxa-aa', { taxa: rateText(loan.annual_rate) })} · ${t('dividas-taxa-am', { taxa: rateText(loan.monthly_rate, 4) })}`,
              },
              { label: t('dividas-sistema'), value: systemText(loan) },
              {
                label: t('dividas-cet-contratado'),
                value: loan.cet_annual
                  ? t('dividas-taxa-aa', { taxa: rateText(loan.cet_annual) })
                  : '—',
              },
            ]}
          />
        </InstrumentCard>
        <InstrumentCard index="06.4" title={t('dividas-custo-realizado')} headingLevel={3}>
          <KeyValue
            items={[
              {
                label: t('dividas-cet-realizado'),
                value: <Pct value={loan.realized_cet} decimals={2} />,
              },
              {
                label: t('dividas-custo-real-ipca'),
                value: <Pct value={loan.real_cost} decimals={2} />,
              },
              { label: t('dividas-juros-pagos'), value: <Money cents={loan.interest_paid} /> },
              {
                label: t('dividas-juros-a-pagar'),
                value: <Money cents={loan.interest_remaining} />,
              },
              ...(loan.charges_paid > 0
                ? [
                    {
                      label: t('dividas-seguros-taxas-pagos'),
                      value: <Money cents={loan.charges_paid} />,
                    },
                  ]
                : []),
            ]}
          />
        </InstrumentCard>
      </div>

      <QueryState
        loading={schedule.isPending}
        error={schedule.error}
        onRetry={() => void schedule.refetch()}
        label={t('dividas-montando-cronograma')}
      >
        {installments.length > 0 ? (
          <>
            <div className="grid gap-6 @min-[1040px]:grid-cols-2">
              <BalanceFigure
                index="06.5"
                loan={loan}
                installments={installments}
                simulated={simulated}
              />
              <BreakdownFigure index="06.6" loan={loan} installments={installments} />
            </div>

            {loan.balance > 0 ? (
              <InstrumentCard index="06.7" title={t('dividas-simulador-amortizacao')}>
                <PrepaySimulator
                  loan={loan}
                  onSimulated={onSimulated}
                  onRegister={(c: PrepayChoice) => setExtra(c)}
                />
              </InstrumentCard>
            ) : null}

            {loan.balance > 0 && loan.remaining_count > 0 ? (
              <InstrumentCard index="06.7b" title={t('dividas-antecipar-parcelas')}>
                <AnticipateCard loan={loan} />
              </InstrumentCard>
            ) : null}

            {loan.balance > 0 && features.br_credit_rates ? (
              <InstrumentCard index="06.8" title={t('dividas-simulador-portabilidade')}>
                <PortabilitySimulator loan={loan} />
              </InstrumentCard>
            ) : null}

            <InstrumentCard
              index="06.9"
              title={t('dividas-tabela-amortizacao')}
              footer={[
                {
                  label: t('dividas-conta-de-debito'),
                  value: account ?? t('dividas-nenhuma-min'),
                },
                ...(category
                  ? [{ label: t('dividas-categoria-pagamentos'), value: category }]
                  : []),
                ...(loan.insurance_monthly + loan.fees_monthly > 0
                  ? [
                      {
                        label: t('dividas-seguros-taxas-mes'),
                        value: <Money cents={loan.insurance_monthly + loan.fees_monthly} />,
                      },
                    ]
                  : []),
                { label: t('dividas-origem'), value: t('dividas-origem-valor') },
              ]}
            >
              <ScheduleTable loan={loan} installments={installments} />
            </InstrumentCard>
          </>
        ) : null}
      </QueryState>

      {loan.notes ? (
        <p className="text-[12.5px] text-dim">
          {t('dividas-observacoes-valor', { notas: loan.notes })}
        </p>
      ) : null}

      <LoanFormDialog open={editing} onOpenChange={setEditing} loan={loan} />
      <StatementDialog loan={loan} open={statementOpen} onOpenChange={setStatementOpen} />
      <ExtraPaymentDialog
        open={extra != null}
        onOpenChange={(o) => (!o ? setExtra(null) : undefined)}
        loan={loan}
        preset={extra}
      />
      <ConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title={t('dividas-excluir-titulo', { nome: loan.name })}
        description={t('dividas-excluir-descricao')}
        confirmLabel={t('dividas-excluir-confirmar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          remove.mutate(
            { id: loan.id },
            {
              onSuccess: () => {
                undoToast({
                  tone: 'success',
                  title: t('dividas-excluido'),
                  description: loan.name,
                });
                setConfirmDelete(false);
                onDeleted();
              },
            },
          )
        }
      />
    </section>
  );
}

/** Sistema e indexador por extenso: `SAC · correção IPCA (M − 2, piso zero)`. */
function systemText(loan: LoanView): string {
  const system = SYSTEM_LABEL[loan.system];
  if (loan.indexer === 'none') return `${system} · ${t('dividas-sistema-pre')}`;
  const indice = INDEXER_LABEL[loan.indexer];
  if (isPostIndexer(loan.indexer)) {
    const text = loan.index_lag_days
      ? t('dividas-sistema-pos-defasagem', { indice, dias: loan.index_lag_days })
      : t('dividas-sistema-pos', { indice });
    return `${system} · ${text}`;
  }
  const detalhes = [
    loan.correction_reference === 'anniversary'
      ? t('dividas-correcao-aniversario')
      : loan.correction_lag_months === 0
        ? t('dividas-correcao-mes')
        : `M − ${loan.correction_lag_months}`,
    loan.deflation_floor ? t('dividas-correcao-piso') : null,
    loan.correction_prorata !== 'none' ? t('dividas-correcao-prorata') : null,
  ]
    .filter(Boolean)
    .join(', ');
  return `${system} · ${t('dividas-sistema-correcao', { indice, detalhes })}`;
}

export function ProgressBar({ value, label }: { value: number; label: string }) {
  const pct = Math.round(Math.max(0, Math.min(1, value)) * 100);
  return (
    <div className="mt-3 flex flex-col gap-1.5">
      <div
        role="progressbar"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
        className="h-1.5 w-full overflow-hidden rounded-full bg-surface-2"
      >
        <div className="h-full rounded-full bg-accent" style={{ width: `${pct}%` }} />
      </div>
      <span className="label-caps">{label}</span>
    </div>
  );
}
