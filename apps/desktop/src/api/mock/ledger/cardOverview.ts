/**
 * Limite, datas da fatura e anuidade no mock (V156), como `cards/limits.rs` e
 * `cards/annual_fee.rs`. A isenção por investimento não olha os saldos (fica sempre a cumprir).
 */
import type { AnnualFee } from '@/api/generated/AnnualFee';
import type { AnnualFeeStatus } from '@/api/generated/AnnualFeeStatus';
import type { CardOverview } from '@/api/generated/CardOverview';
import type { FeeCharge } from '@/api/generated/FeeCharge';
import type { LimitChange } from '@/api/generated/LimitChange';
import type { RevolvingStatus } from '@/api/generated/RevolvingStatus';

import { familyCountry } from '../country';

import { addDays, addMonths, nowUtc, parseYmd, today } from './dates';
import {
  account,
  datesForReferenceMonth,
  requireCard,
  statementDates,
  statementView,
} from './domain';
import { cleanOpt, invalid, notFound, str } from './errors';
import { type AnnualFeeRow, del, find, newId, patch, put, rows } from './store';
import { normalizeDescription } from './text';

const days = (from: string, to: string) =>
  Math.round((Date.parse(`${to}T00:00:00Z`) - Date.parse(`${from}T00:00:00Z`)) / 86_400_000);

export function limitHistory(accountId: string): LimitChange[] {
  return rows('card_limit_change')
    .filter((c) => c.account_id === accountId)
    .sort((a, b) => b.date.localeCompare(a.date) || b.created_at.localeCompare(a.created_at))
    .map((c) => ({ id: c.id, date: c.date, credit_limit: c.credit_limit, source: c.source }));
}

function syncCurrent(accountId: string): void {
  const latest = limitHistory(accountId)[0];
  const card = find('credit_card', accountId);
  if (latest && card && card.credit_limit !== latest.credit_limit)
    patch('credit_card', accountId, { credit_limit: latest.credit_limit, updated_at: nowUtc() });
}

/** O limite mudou no cadastro: registra no histórico. */
export function recordManualLimit(
  accountId: string,
  old: number | null | undefined,
  next: number | null | undefined,
): void {
  if (next == null || old === next) return;
  const has = rows('card_limit_change').some((c) => c.account_id === accountId);
  const date = has ? today() : account(accountId).opening_date;
  const same = rows('card_limit_change').find(
    (c) => c.account_id === accountId && c.date === date && c.source === 'manual',
  );
  const now = nowUtc();
  if (same) patch('card_limit_change', same.id, { credit_limit: next, updated_at: now });
  else
    put('card_limit_change', {
      id: newId(),
      account_id: accountId,
      date,
      credit_limit: next,
      source: 'manual',
      created_at: now,
      updated_at: now,
    });
}

export function addLimitChange(args: Record<string, unknown>): void {
  const accountId = str(args, 'account_id');
  requireCard(accountId);
  const date = str(args, 'date');
  parseYmd(date);
  const limit = typeof args.credit_limit === 'number' ? args.credit_limit : -1;
  if (limit < 0) invalid('o limite não pode ser negativo');
  const now = nowUtc();
  put('card_limit_change', {
    id: newId(),
    account_id: accountId,
    date,
    credit_limit: limit,
    source: 'manual',
    created_at: now,
    updated_at: now,
  });
  syncCurrent(accountId);
}

export function deleteLimitChange(id: string): void {
  const c = find('card_limit_change', id) ?? notFound('registro de limite');
  del('card_limit_change', id);
  syncCurrent(c.account_id);
}

// ── Anuidade ────────────────────────────────────────────────────

function feeView(r: AnnualFeeRow): AnnualFee {
  return {
    id: r.id,
    account_id: r.account_id,
    amount: r.amount,
    installments: r.installments,
    anniversary_month: r.anniversary_month,
    waiver_kind: r.waiver_kind,
    waiver_threshold: r.waiver_threshold,
    waiver_discount: r.waiver_discount,
    starts_on: r.starts_on,
    ends_on: r.ends_on,
    notes: r.notes,
  };
}

function activeFee(accountId: string, date: string): AnnualFeeRow | undefined {
  return rows('card_annual_fee')
    .filter(
      (f) => f.account_id === accountId && f.starts_on <= date && (!f.ends_on || f.ends_on >= date),
    )
    .sort((a, b) => b.starts_on.localeCompare(a.starts_on))[0];
}

export function setAnnualFee(args: Record<string, unknown>): void {
  const accountId = str(args, 'account_id');
  requireCard(accountId);
  const amount = typeof args.amount === 'number' ? args.amount : -1;
  if (amount < 0) invalid('o valor da anuidade não pode ser negativo');
  const installments = typeof args.installments === 'number' ? args.installments : 12;
  if (installments < 1 || installments > 12) invalid('a anuidade é cobrada em 1 a 12 parcelas');
  const month = typeof args.anniversary_month === 'number' ? args.anniversary_month : null;
  if (month != null && (month < 1 || month > 12)) invalid('o mês de aniversário vai de 1 a 12');
  const kind = (
    typeof args.waiver_kind === 'string' ? args.waiver_kind : 'none'
  ) as AnnualFeeRow['waiver_kind'];
  const threshold = typeof args.waiver_threshold === 'number' ? args.waiver_threshold : null;
  if ((kind === 'spend' || kind === 'investment') && !(threshold && threshold > 0))
    invalid('informe o valor que isenta a anuidade (gasto na fatura ou valor investido)');
  const discount = kind === 'discount' ? cleanOpt(args.waiver_discount) : null;
  if (kind === 'discount') {
    const d = Number(discount);
    if (!discount || Number.isNaN(d) || d < 0 || d > 1)
      invalid('o desconto é um número entre 0 e 1 (0,5 = 50%)');
  }
  const starts = cleanOpt(args.starts_on) ?? today();
  parseYmd(starts);
  const now = nowUtc();
  for (const f of rows('card_annual_fee')) {
    if (f.account_id !== accountId) continue;
    if (f.starts_on === starts) del('card_annual_fee', f.id);
    else if (f.starts_on < starts && (!f.ends_on || f.ends_on >= starts))
      patch('card_annual_fee', f.id, { ends_on: addDays(starts, -1), updated_at: now });
  }
  put('card_annual_fee', {
    id: newId(),
    account_id: accountId,
    amount,
    installments,
    anniversary_month: month,
    waiver_kind: kind,
    waiver_threshold: kind === 'spend' || kind === 'investment' ? threshold : null,
    waiver_discount: discount,
    starts_on: starts,
    ends_on: null,
    notes: cleanOpt(args.notes),
    created_at: now,
    updated_at: now,
  });
}

export function endAnnualFee(accountId: string): void {
  const T = today();
  const mine = rows('card_annual_fee').filter((f) => f.account_id === accountId);
  if (mine.length === 0) notFound(`anuidade do cartão ${accountId}`);
  for (const f of mine) {
    if (f.starts_on > T) del('card_annual_fee', f.id);
    else if (!f.ends_on || f.ends_on > T)
      patch('card_annual_fee', f.id, { ends_on: T, updated_at: nowUtc() });
  }
}

const isFee = (desc: string) => normalizeDescription(desc).includes('anuidade');

function statementSpend(statementId: string | null): number {
  if (!statementId) return 0;
  return rows('txn')
    .filter((t) => t.statement_id === statementId && t.kind === 'expense' && !isFee(t.description))
    .reduce((s, t) => s - t.amount, 0);
}

function expectedFor(f: AnnualFeeRow, spend: number): number {
  const full = Math.trunc(f.amount / f.installments);
  switch (f.waiver_kind) {
    case 'full':
      return 0;
    case 'spend':
      return spend >= (f.waiver_threshold ?? Infinity) ? 0 : full;
    case 'investment':
      return full;
    case 'discount':
      return Math.round(full * (1 - Number(f.waiver_discount ?? 0)));
    default:
      return full;
  }
}

function nextRenewal(f: AnnualFeeRow, T: string): string | null {
  if (!f.anniversary_month) return null;
  const y = Number(T.slice(0, 4));
  const thisYear = `${y}-${String(f.anniversary_month).padStart(2, '0')}-01`;
  return thisYear > T ? thisYear : addMonths(thisYear, 12);
}

export function feeStatus(accountId: string, T = today()): AnnualFeeStatus | null {
  const f = activeFee(accountId, T);
  if (!f) return null;
  const open = rows('card_statement').find(
    (s) => s.account_id === accountId && s.period_start <= T && s.closing_date >= T,
  );
  const openSpend = statementSpend(open?.id ?? null);
  const current = f.waiver_kind === 'spend' ? openSpend : 0;
  const full = Math.trunc(f.amount / f.installments);
  const from = addMonths(T, -12);
  const charges: FeeCharge[] = [];
  let paid = 0;
  for (const t of rows('txn')
    .filter(
      (t) =>
        t.account_id === accountId &&
        t.date > from &&
        t.date <= T &&
        (t.kind === 'expense' || t.kind === 'adjustment') &&
        isFee(t.description),
    )
    .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id))) {
    const charged = -t.amount;
    paid += charged;
    const c: FeeCharge = {
      txn_id: t.id,
      date: t.date,
      description: t.description,
      amount: charged,
      statement_id: t.statement_id,
      expected: null,
      issue: null,
    };
    if (charged > 0 && t.date >= f.starts_on) {
      const expected = expectedFor(f, statementSpend(t.statement_id));
      c.expected = expected;
      const count = charges.filter((x) => x.amount > 0 && x.date >= f.starts_on).length + 1;
      const first = f.amount - full * (f.installments - 1);
      c.issue =
        expected === 0
          ? 'charged_while_waived'
          : count > f.installments
            ? 'extra_installment'
            : Math.abs(charged - expected) > 1 && charged !== first
              ? 'amount_differs'
              : null;
    }
    charges.push(c);
  }
  const met = f.waiver_kind !== 'none' && expectedFor(f, openSpend) === 0;
  return {
    rule: feeView(f),
    installment_amount: full,
    waiver: {
      kind: f.waiver_kind,
      threshold: f.waiver_threshold,
      current,
      remaining:
        (f.waiver_kind === 'spend' || f.waiver_kind === 'investment') && f.waiver_threshold
          ? Math.max(0, f.waiver_threshold - current)
          : 0,
      met,
    },
    paid_12m: paid,
    charges,
    next_renewal: nextRenewal(f, T),
    issues: charges.filter((c) => c.issue).length,
  };
}

// ── Rotativo ────────────────────────────────────────────────────

const CHARGE_WORDS = new Set(['juros', 'encargo', 'encargos', 'rotativo', 'multa', 'mora', 'iof']);

function isCharge(desc: string): boolean {
  const d = normalizeDescription(desc);
  return (
    d.split(/\s+/).some((w) => CHARGE_WORDS.has(w)) &&
    !d.includes('compra internacional') &&
    !d.includes('transacoes no exterior')
  );
}

/** Rotativo em curso, como `cards/revolving.rs`. */
export function revolvingStatus(accountId: string, T = today()): RevolvingStatus | null {
  const tracked = rows('txn').some(
    (t) => t.account_id === accountId && t.kind === 'card_payment' && t.status !== 'planned',
  );
  if (!tracked) return null;
  const views = rows('card_statement')
    .filter((s) => s.account_id === accountId && s.due_date < T)
    .sort((a, b) => a.reference_month.localeCompare(b.reference_month))
    .map((s) => statementView(s, T));
  const run = [];
  for (const v of [...views].reverse()) {
    if (v.remaining > 0) run.push(v);
    else break;
  }
  const first = run.at(-1);
  if (!first) return null;
  let charges = 0;
  for (const t of rows('txn')) {
    if (t.account_id !== accountId || t.kind === 'card_payment' || t.amount >= 0) continue;
    const s = find('card_statement', t.statement_id);
    if (s && s.reference_month > first.reference_month && isCharge(t.description))
      charges -= t.amount;
  }
  return {
    statement_id: first.id,
    reference_month: first.reference_month,
    due_date: first.due_date,
    unpaid: first.remaining,
    statements: run.length,
    charges,
    charges_bp: first.remaining > 0 ? Math.floor((charges * 10_000) / first.remaining) : 0,
    over_cap: charges > first.remaining,
    beyond_next_statement: run.length > 1,
    currency: first.currency,
  };
}

// ── Resumo do cartão ────────────────────────────────────────────

export function cardOverview(accountId: string, T = today()): CardOverview {
  const card = requireCard(accountId);
  const acc = account(accountId);
  let used = -acc.opening_balance;
  let future = 0;
  for (const t of rows('txn')) {
    if (t.account_id !== accountId) continue;
    if (t.status !== 'planned' || t.installment_plan_id) used -= t.amount;
    if (t.installment_plan_id && t.status === 'planned') future -= t.amount;
  }
  const limit = card.credit_limit ?? null;
  const best = statementDates(T, card.closing_day, card.due_day);
  const bestDate = addDays(best.closing_date, 1);
  const bestDue = statementDates(bestDate, card.closing_day, card.due_day).due_date;
  const open = rows('card_statement').find(
    (s) => s.account_id === accountId && s.period_start <= T && s.closing_date >= T,
  );
  return {
    account_id: accountId,
    account_name: acc.name,
    currency: acc.currency,
    credit_limit: limit,
    used,
    committed_installments: future,
    available: limit == null ? null : limit - used,
    utilization_bp: limit && limit > 0 ? Math.floor((Math.max(0, used) * 10_000) / limit) : null,
    best_purchase_date: bestDate,
    best_purchase_due_date: bestDue,
    best_purchase_days: days(bestDate, bestDue),
    open_statement: open ? statementView(open, T) : null,
    limit_history: limitHistory(accountId),
    annual_fee: feeStatus(accountId, T),
    revolving: revolvingStatus(accountId, T),
    brazil_rules: familyCountry() === 'BR',
  };
}

// ── Fatura: datas reais e valores informados ────────────────────

export function updateStatement(args: Record<string, unknown>): void {
  const s = find('card_statement', str(args, 'id')) ?? notFound(`fatura ${String(args.id)}`);
  const now = nowUtc();
  if (args.reset_dates === true) {
    const card = requireCard(s.account_id);
    const d = datesForReferenceMonth(s.reference_month, card.closing_day, card.due_day);
    patch('card_statement', s.id, { ...d, dates_source: 'calc', updated_at: now });
  } else if (typeof args.closing_date === 'string' || typeof args.due_date === 'string') {
    const closing = typeof args.closing_date === 'string' ? args.closing_date : s.closing_date;
    const due = typeof args.due_date === 'string' ? args.due_date : s.due_date;
    parseYmd(closing);
    parseYmd(due);
    if (due < closing) invalid('o vencimento não pode ser antes do fechamento');
    if (closing < s.period_start)
      invalid('o fechamento não pode ser antes do início das compras da fatura');
    patch('card_statement', s.id, {
      closing_date: closing,
      due_date: due,
      dates_source: 'manual',
      updated_at: now,
    });
  }
  for (const k of [
    'informed_total',
    'informed_minimum',
    'informed_limit',
    'informed_previous_balance',
  ] as const) {
    if (!Object.prototype.hasOwnProperty.call(args, k)) continue;
    const raw = args[k];
    const v = typeof raw === 'number' ? raw : null;
    if (v != null && v < 0) invalid('os valores informados da fatura não podem ser negativos');
    patch('card_statement', s.id, { [k]: v, updated_at: now });
  }
}
