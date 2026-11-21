/**
 * Transformações puras da tela 07 · Dívidas: progresso, status das parcelas, séries do
 * saldo devedor e da composição das parcelas (por parcela ou por ano) e o resumo das
 * simulações de amortização extraordinária.
 */
import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import type { PrepaySimulation } from '@/api/generated/PrepaySimulation';
import { t } from '@/i18n';
import { formatLocale, MINUS } from '@/lib/format';
import { daysBetween } from '@/lib/months';

import { INDEXER_LABEL } from './labels';

/** Código do `Intl` para o idioma da formatação (o pseudo-idioma formata como pt-BR). */
function intlCode(): string {
  const code = formatLocale();
  return code === 'en' ? 'en-US' : code === 'xx' ? 'pt-BR' : code;
}

/** Número no formato do idioma, com até `max` casas (e pelo menos `min`). */
export function decimalText(value: number, max: number, min = 0): string {
  return value.toLocaleString(intlCode(), {
    minimumFractionDigits: min,
    maximumFractionDigits: max,
  });
}

/** Texto decimal do núcleo (`"9.5"`) com o separador decimal do idioma, sem arredondar. */
export function decimalString(s: string): string {
  return formatLocale() === 'en' ? s : s.replace('.', ',');
}

/** Taxa em texto decimal do núcleo (`"9.5"`, % a.a.) → `9,5%`. */
export function rateText(pct: string | null | undefined, decimals = 2): string {
  if (pct == null || pct.trim() === '') return '—';
  const n = Number(pct);
  if (!Number.isFinite(n)) return pct;
  return `${n < 0 ? MINUS : ''}${decimalText(Math.abs(n), decimals)}%`;
}

/**
 * Taxa do contrato por extenso: `9,5% a.a.`, `TR + 9,5% a.a.`, `CDI + 2% a.a.`,
 * `110% do CDI`, `120% da Selic + 1% a.a. (linear)`. Com `perYear: false`, sem o "a.a."
 * (coluna "Taxa a.a." da lista).
 */
export function loanRateLabel(
  loan: Pick<LoanView, 'indexer' | 'annual_rate' | 'index_pct' | 'spread_mode'>,
  { perYear = true }: { perYear?: boolean } = {},
): string {
  const yearly = (taxa: string) => (perYear ? t('dividas-taxa-aa', { taxa }) : taxa);
  if (loan.indexer === 'cdi' || loan.indexer === 'selic') {
    const name = INDEXER_LABEL[loan.indexer];
    const pct = Number(loan.index_pct);
    const base =
      pct === 100
        ? name
        : t(loan.indexer === 'cdi' ? 'dividas-taxa-pct-cdi' : 'dividas-taxa-pct-selic', {
            pct: rateText(loan.index_pct),
          });
    const spread = Number(loan.annual_rate);
    if (spread === 0) return base;
    const text = `${base} ${spread < 0 ? '−' : '+'} ${yearly(rateText(String(Math.abs(spread))))}`;
    return loan.spread_mode === 'lin360' ? t('dividas-taxa-linear', { taxa: text }) : text;
  }
  const rate = yearly(rateText(loan.annual_rate));
  return loan.indexer === 'none' ? rate : `${INDEXER_LABEL[loan.indexer]} + ${rate}`;
}

/** Fator do período (`"1.012345678"`) → variação `+1,2346%`. */
export function periodRateText(factor: string | null): string | null {
  if (factor == null) return null;
  const pct = (Number(factor) - 1) * 100;
  return `${pct >= 0 ? '+' : MINUS}${decimalText(Math.abs(pct), 4, 2)}%`;
}

/** Parcelas pagas ÷ total do cronograma (0–1). */
export function progress(loan: Pick<LoanView, 'paid_count' | 'remaining_count'>): number {
  const total = loan.paid_count + loan.remaining_count;
  return total === 0 ? 0 : loan.paid_count / total;
}

export type InstallmentStatus = 'paid' | 'overdue' | 'next' | 'planned';

export function installmentStatus(i: Installment, nextId: string | null): InstallmentStatus {
  if (i.paid) return 'paid';
  if (i.overdue) return 'overdue';
  return i.id === nextId ? 'next' : 'planned';
}

/** Rótulo da situação da parcela, no idioma do app. */
export function statusLabel(status: InstallmentStatus): string {
  return {
    paid: t('dividas-status-paga'),
    overdue: t('dividas-status-atrasada'),
    next: t('dividas-status-proxima'),
    planned: t('dividas-status-prevista'),
  }[status];
}

/** Primeira parcela em aberto (a próxima a pagar), atrasada ou não. */
export function nextOpen(installments: readonly Installment[]): Installment | null {
  return installments.find((i) => !i.paid) ?? null;
}

/**
 * Parcelas que um lançamento pode pagar: as sem lançamento ligado (ou ligadas a ele
 * mesmo), as `count` de vencimento mais perto da data dele, em ordem de número.
 */
export function linkableInstallments(
  installments: readonly Installment[],
  date: string,
  txnId: string | null,
  count = 12,
): Installment[] {
  const gap = (i: Installment) => Math.abs(daysBetween(i.due_date, date));
  return installments
    .filter((i) => i.id && (i.txn_id == null || i.txn_id === txnId))
    .sort((a, b) => gap(a) - gap(b) || a.number - b.number)
    .slice(0, count)
    .sort((a, b) => a.number - b.number);
}

/** A parcela sugerida para um pagamento: a em aberto de vencimento mais perto da data
 *  (sem nenhuma em aberto, a mais perto). */
export function suggestedInstallment(
  installments: readonly Installment[],
  date: string,
): Installment | null {
  const gap = (i: Installment) => Math.abs(daysBetween(i.due_date, date));
  const closest = (list: readonly Installment[]) =>
    [...list].sort((a, b) => gap(a) - gap(b) || a.number - b.number)[0] ?? null;
  return closest(installments.filter((i) => !i.paid)) ?? closest(installments);
}

export interface BalancePoint {
  date: string;
  balance: number;
  estimated: boolean;
}

/** Saldo devedor ao longo do contrato: o valor financiado na assinatura e o saldo após cada parcela. */
export function balanceSeries(
  installments: readonly Installment[],
  principal: number,
  contractDate: string,
): BalancePoint[] {
  return [
    { date: contractDate, balance: principal, estimated: false },
    ...installments.map((i) => ({
      date: i.due_date,
      balance: i.balance_after,
      estimated: i.estimated,
    })),
  ];
}

/** Índice do último ponto com data até `today` (para a marca "Hoje"). */
export function todayIndex(points: readonly { date: string }[], today: string): number {
  let idx = -1;
  points.forEach((p, i) => {
    if (p.date <= today) idx = i;
  });
  return idx;
}

export interface BreakdownRow {
  key: string;
  label: string;
  /** Data (vencimento ou último vencimento do ano). */
  date: string;
  amortization: number;
  interest: number;
  adjustment: number;
  charges: number;
  total: number;
  extra: number;
  estimated: boolean;
  /** Quantas parcelas o grupo soma. */
  count: number;
}

export type BreakdownMode = 'parcela' | 'ano';

/** Por ano quando o contrato é longo (> 60 parcelas); senão parcela a parcela. */
export function defaultBreakdownMode(count: number): BreakdownMode {
  return count > 60 ? 'ano' : 'parcela';
}

/** Composição das parcelas: amortização, juros, correção do saldo e seguros + taxas. */
export function breakdown(
  installments: readonly Installment[],
  mode: BreakdownMode,
): BreakdownRow[] {
  if (mode === 'parcela') {
    return installments.map((i) => ({
      key: String(i.number),
      label: String(i.number),
      date: i.due_date,
      amortization: i.amortization,
      interest: i.interest,
      adjustment: i.adjustment,
      charges: i.insurance + i.fees,
      total: i.total,
      extra: i.extra_payment,
      estimated: i.estimated,
      count: 1,
    }));
  }
  const byYear = new Map<string, BreakdownRow>();
  for (const i of installments) {
    const y = i.due_date.slice(0, 4);
    const row =
      byYear.get(y) ??
      ({
        key: y,
        label: y,
        date: i.due_date,
        amortization: 0,
        interest: 0,
        adjustment: 0,
        charges: 0,
        total: 0,
        extra: 0,
        estimated: false,
        count: 0,
      } satisfies BreakdownRow);
    row.amortization += i.amortization;
    row.interest += i.interest;
    row.adjustment += i.adjustment;
    row.charges += i.insurance + i.fees;
    row.total += i.total;
    row.extra += i.extra_payment;
    row.estimated ||= i.estimated;
    row.count += 1;
    row.date = i.due_date;
    byYear.set(y, row);
  }
  return [...byYear.values()];
}

/** Totais de juros, amortização, correção e encargos de um conjunto de parcelas. */
export function totals(installments: readonly Installment[]) {
  return installments.reduce(
    (s, i) => ({
      amortization: s.amortization + i.amortization,
      interest: s.interest + i.interest,
      adjustment: s.adjustment + i.adjustment,
      charges: s.charges + i.insurance + i.fees,
      total: s.total + i.total,
      extra: s.extra + i.extra_payment,
    }),
    { amortization: 0, interest: 0, adjustment: 0, charges: 0, total: 0, extra: 0 },
  );
}

export interface SimulationSummary {
  interestSaved: number;
  /** Parcelas restantes depois da data da amortização. */
  remainingBefore: number;
  remainingAfter: number;
  monthsSaved: number;
  nextBefore: number;
  nextAfter: number;
  /** Último vencimento com a amortização. */
  payoffDate: string | null;
}

/** Leitura da simulação: quantas parcelas sobram, quanto de juros sai e a nova parcela. */
export function simulationSummary(sim: PrepaySimulation, date: string): SimulationSummary {
  const paidBefore = sim.schedule.filter((r) => r.due_date <= date).length;
  const remainingBefore = Math.max(sim.months_before - paidBefore, 0);
  const remainingAfter = Math.max(sim.months_after - paidBefore, 0);
  return {
    interestSaved: sim.interest_saved,
    remainingBefore,
    remainingAfter,
    monthsSaved: remainingBefore - remainingAfter,
    nextBefore: sim.next_payment_before,
    nextAfter: sim.next_payment_after,
    payoffDate: sim.schedule.at(-1)?.due_date ?? null,
  };
}

/** `12 parcelas` / `1 parcela`; `3 anos e 2 meses` para prazos longos. */
export function termText(months: number): string {
  if (months < 24) return t('dividas-prazo-parcelas', { n: months });
  return t('dividas-prazo-longo', { n: months, anos: Math.floor(months / 12), meses: months % 12 });
}

/** Taxa mensal equivalente à efetiva anual (% a.a. → fração ao mês). */
export function monthlyRateOf(annualPct: number): number {
  return (1 + annualPct / 100) ** (1 / 12) - 1;
}

export interface PaymentPreviewInput {
  principal: number;
  annualRatePct: number;
  system: 'sac' | 'price' | 'bullet';
  term: number;
  grace: number;
  charges: number;
}

/**
 * Primeira parcela depois da carência, sem correção monetária (prévia do formulário; o
 * cronograma de verdade sai do núcleo). Durante a carência a parcela é só juros + encargos.
 */
export function firstPaymentPreview(p: PaymentPreviewInput): {
  grace: number | null;
  first: number;
  last: number;
} | null {
  const n = p.term - p.grace;
  if (!(p.principal > 0) || !(n > 0) || !Number.isFinite(p.annualRatePct) || p.annualRatePct < 0)
    return null;
  const i = monthlyRateOf(p.annualRatePct);
  const interest = p.principal * i;
  const graceTotal = p.grace > 0 ? Math.round(interest) + p.charges : null;
  if (p.system === 'bullet') {
    return {
      grace: graceTotal,
      first: Math.round(interest) + p.charges,
      last: Math.round(p.principal + interest) + p.charges,
    };
  }
  if (p.system === 'sac') {
    const amort = p.principal / n;
    return {
      grace: graceTotal,
      first: Math.round(amort + interest) + p.charges,
      last: Math.round(amort + amort * i) + p.charges,
    };
  }
  const pmt = i === 0 ? p.principal / n : (p.principal * i) / (1 - (1 + i) ** -n);
  return {
    grace: graceTotal,
    first: Math.round(pmt) + p.charges,
    last: Math.round(pmt) + p.charges,
  };
}
