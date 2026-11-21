/**
 * Hooks de `debt.*`: contratos de empréstimo e financiamento, cronograma, simuladores
 * (amortização extraordinária e portabilidade) e as escritas (contrato, amortização,
 * parcela paga, ligação parcela × lançamento, conciliação). Chaves `['debt', acao, args]`.
 */
import { keepPreviousData } from '@tanstack/react-query';

import type { PaymentArgs } from '../generated/PaymentArgs';
import type { AmortSystem } from '../generated/AmortSystem';
import type { AnticipateArgs } from '../generated/AnticipateArgs';
import type { AnticipationSimulation } from '../generated/AnticipationSimulation';
import type { DebtSummary } from '../generated/DebtSummary';
import type { Installment } from '../generated/Installment';
import type { LoanInput } from '../generated/LoanInput';
import type { LoanPaymentCandidate } from '../generated/LoanPaymentCandidate';
import type { StatementArgs } from '../generated/StatementArgs';
import type { StatementPreview } from '../generated/StatementPreview';
import type { StatementResult } from '../generated/StatementResult';
import type { LoanView } from '../generated/LoanView';
import type { MarketRates } from '../generated/MarketRates';
import type { PortabilityArgs } from '../generated/PortabilityArgs';
import type { PortabilitySimulation } from '../generated/PortabilitySimulation';
import type { PrepayArgs } from '../generated/PrepayArgs';
import type { PrepayMode } from '../generated/PrepayMode';
import type { PrepaySimulation } from '../generated/PrepaySimulation';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export type { AmortSystem };

export const useDebtSummary = () => useCore<DebtSummary>('debt', 'summary');

export const useLoans = (includeArchived = false) =>
  useCore<LoanView[]>('debt', 'list', { include_archived: includeArchived });

export const useLoan = (id: string | null | undefined) =>
  useCore<LoanView>('debt', 'get', { id }, { enabled: Boolean(id) });

export const useLoanSchedule = (id: string | null | undefined) =>
  useCore<Installment[]>('debt', 'schedule', { id }, { enabled: Boolean(id) });

/** Saídas que podem ter pago a parcela (até 20 dias do vencimento, sem parcela ligada),
 *  as de valor mais próximo primeiro. */
export const usePaymentCandidates = (installmentId: string | null | undefined) =>
  useCore<LoanPaymentCandidate[]>(
    'debt',
    'payment_candidates',
    { installment_id: installmentId },
    { enabled: Boolean(installmentId) },
  );

/** Simulação só de leitura; mantém o resultado anterior enquanto o novo chega. */
export const usePrepaySimulation = (args: PrepayArgs | null) =>
  useCore<PrepaySimulation>('debt', 'simulate_prepayment', args ?? {}, {
    enabled: Boolean(args && args.amount > 0 && args.loan_id),
    placeholderData: keepPreviousData,
    retry: false,
  });

export const usePortabilitySimulation = (args: PortabilityArgs | null) =>
  useCore<PortabilitySimulation>('debt', 'simulate_portability', args ?? {}, {
    enabled: Boolean(args?.loan_id && args.annual_rate),
    placeholderData: keepPreviousData,
    retry: false,
  });

export const useAnticipationSimulation = (args: AnticipateArgs | null) =>
  useCore<AnticipationSimulation>('debt', 'simulate_anticipation', args ?? {}, {
    enabled: Boolean(args?.loan_id && args.count > 0),
    placeholderData: keepPreviousData,
    retry: false,
  });

export interface AnticipateRequest extends AnticipateArgs {
  /** Conta de onde sai o dinheiro (padrão: a conta de débito do contrato). */
  from_account_id?: string;
  without_txn?: boolean;
}

export interface LoanPatch {
  id: string;
  name?: string;
  contract_number?: string | null;
  notes?: string | null;
  debit_account_id?: string | null;
  /** Categoria dos pagamentos (liga os lançamentos às parcelas). */
  category_id?: string | null;
  insurance_monthly?: number;
  fees_monthly?: number;
  /** Financiamento imobiliário: valor do imóvel (DFI) e taxas do MIP/DFI em % a.m. */
  property_value?: number | null;
  mip_rate?: string | null;
  dfi_rate?: string | null;
  archived?: boolean;
  /** Só a taxa efetiva % a.a. (ex.: a implícita no extrato do contrato). */
  annual_rate?: string;
  /**
   * Todos os termos do contrato (valor, taxa ou parcela, sistema, indexador, datas,
   * prazo…): o núcleo recalcula as parcelas em aberto, mantendo as pagas, as amortizações
   * e as antecipações.
   */
  terms?: LoanInput;
}

export interface ExtraPaymentArgs {
  loan_id: string;
  date: string;
  amount: number;
  mode: PrepayMode;
  /** Conta de onde sai o dinheiro (padrão: a conta de débito do contrato). */
  from_account_id?: string;
  /** Não lançar em conta nenhuma (ex.: amortização com FGTS). */
  without_txn?: boolean;
}

export interface MarkPaidArgs {
  installment_id: string;
  date: string;
  amount?: number;
  txn_id?: string;
}

const AFFECTS = { affects: ['debt'] } as const;

export const useCreateLoan = () => useCoreMutation<LoanInput, LoanView>('debt.create', AFFECTS);
export const useUpdateLoan = () => useCoreMutation<LoanPatch, LoanView>('debt.update', AFFECTS);
/** Confere o extrato do contrato colado (sem gravar). */
export const previewLoanStatement = (args: StatementArgs) =>
  call<StatementPreview>('debt.statement_preview', args);
export const useImportLoanStatement = () =>
  useCoreMutation<StatementArgs, StatementResult>('debt.import_statement', AFFECTS);
export const useDeleteLoan = () => useCoreMutation<{ id: string }, null>('debt.delete', AFFECTS);
export const useAddExtraPayment = () =>
  useCoreMutation<ExtraPaymentArgs, LoanView>('debt.add_extra_payment', AFFECTS);
export const useMarkInstallmentPaid = () =>
  useCoreMutation<MarkPaidArgs, null>('debt.mark_paid', AFFECTS);
/** Pagamento com os valores do comprovante; com o saldo devedor depois dele, sincroniza
 *  o contrato (abre uma nova versão quando houve amortização). */
export const useRecordLoanPayment = () =>
  useCoreMutation<PaymentArgs, Installment>('debt.record_payment', AFFECTS);
/** Liga a parcela ao lançamento que a pagou (ela fica paga na data e pelo valor dele). */
export const useLinkLoanPayment = () =>
  useCoreMutation<{ installment_id: string; txn_id: string }, Installment>(
    'debt.link_payment',
    AFFECTS,
  );
export const useUnmarkInstallmentPaid = () =>
  useCoreMutation<{ id: string }, null>('debt.unmark_paid', AFFECTS);
export const useAnticipate = () =>
  useCoreMutation<AnticipateRequest, AnticipationSimulation>('debt.anticipate', AFFECTS);
export const useMarkPaidUntil = () =>
  useCoreMutation<{ loan_id: string; before: string }, { marked: number }>(
    'debt.mark_paid_until',
    AFFECTS,
  );
export const useReconcileLoan = () =>
  useCoreMutation<{ id: string }, { matched: number }>('debt.reconcile', AFFECTS);

/** Taxas de juros do mercado (BCB) e a comparação dos contratos; baixa se tiverem mais de 7 dias. */
export const useMarketRates = (refresh = false) =>
  useCore<MarketRates>('debt', 'market_rates', refresh ? { refresh } : {});
