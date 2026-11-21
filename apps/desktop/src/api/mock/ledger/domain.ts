/**
 * Regras de domínio do livro-caixa compartilhadas pelos mocks dos módulos:
 * caminhos de categoria, câmbio, faturas, inserção de lançamentos e as visões
 * (`AccountView`, `TxnView`, `StatementView`) calculadas como no núcleo.
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { Category } from '@/api/generated/Category';
import type { CategoryNode } from '@/api/generated/CategoryNode';
import type { Converted } from '@/api/generated/Converted';
import type { CurrencyTotal } from '@/api/generated/CurrencyTotal';
import type { StatementStatus } from '@/api/generated/StatementStatus';
import type { StatementView } from '@/api/generated/StatementView';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnPage } from '@/api/generated/TxnPage';
import type { TxnView } from '@/api/generated/TxnView';
import type { AccountKind } from '@/api/generated/AccountKind';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import { retirementOf } from '../retirement';
import { bankDetailsOf } from './bankDetails';
import { freshnessOf } from './freshness';
import { overdraftOf, overdraftStatus } from './overdraft';

import {
  addMonths,
  clampDay,
  diffDays,
  fmt,
  monthEnd,
  monthOf,
  nowUtc,
  parseMonth,
  parseYmd,
  today,
} from './dates';
import { invalid, notFound } from './errors';
import { forgetLoanTxns, loanLinks } from './loanLinks';
import { operationLinks } from './opLinks';
import {
  type AccountRow,
  type CategoryRow,
  type StatementRow,
  type TxnRow,
  db,
  del,
  find,
  newId,
  patch,
  put,
  rows,
} from './store';
import { fingerprint, fold, searchFold } from './text';

export const PATH_SEP = ' › ';

// ── Família ─────────────────────────────────────────────────────

export function household() {
  return rows('household')[0] ?? notFound('família');
}

export function baseCurrency(): string {
  return rows('household')[0]?.base_currency ?? 'BRL';
}

export function requireMember(id: string): void {
  if (!find('member', id)) notFound(`membro ${id}`);
}

// ── Categorias ──────────────────────────────────────────────────

function sortCats(a: CategoryRow, b: CategoryRow): number {
  return (
    a.sort_order - b.sort_order ||
    a.name.localeCompare(b.name, 'pt-BR', { sensitivity: 'base' }) ||
    a.id.localeCompare(b.id)
  );
}

export function categoryPaths(): Map<string, string> {
  const all = db.t.category;
  const out = new Map<string, string>();
  for (const c of all.values()) {
    const parts: string[] = [];
    let cur: CategoryRow | undefined = c;
    const seen = new Set<string>();
    while (cur && !seen.has(cur.id)) {
      seen.add(cur.id);
      parts.unshift(cur.name);
      cur = cur.parent_id ? all.get(cur.parent_id) : undefined;
    }
    out.set(c.id, parts.join(PATH_SEP));
  }
  return out;
}

export function toCategory(r: CategoryRow, paths = categoryPaths()): Category {
  return {
    id: r.id,
    parent_id: r.parent_id,
    name: r.name,
    kind: r.kind,
    icon: r.icon,
    color: r.color,
    is_essential: r.is_essential,
    spend_group: r.spend_group ?? null,
    archived: r.archived,
    sort_order: r.sort_order,
    path: paths.get(r.id) ?? r.name,
  };
}

export function categoryTree(kind?: CategoryKind, includeArchived = false): CategoryNode[] {
  const paths = categoryPaths();
  const all = rows('category').sort(sortCats);
  const ids = new Set(all.map((c) => c.id));
  const byParent = new Map<string | null, CategoryRow[]>();
  for (const c of all) {
    const p = c.parent_id && ids.has(c.parent_id) ? c.parent_id : null;
    const list = byParent.get(p) ?? [];
    list.push(c);
    byParent.set(p, list);
  }
  const build = (parent: string | null): CategoryNode[] =>
    (byParent.get(parent) ?? [])
      .filter((c) => (includeArchived || !c.archived) && (!kind || c.kind === kind))
      .map((c) => ({ ...toCategory(c, paths), children: build(c.id) }));
  const roots = build(null);
  return [...roots.filter((n) => n.kind !== 'income'), ...roots.filter((n) => n.kind === 'income')];
}

export function withDescendants(ids: readonly string[]): string[] {
  const out = new Set(ids);
  let grew = true;
  while (grew) {
    grew = false;
    for (const c of db.t.category.values()) {
      if (c.parent_id && out.has(c.parent_id) && !out.has(c.id)) {
        out.add(c.id);
        grew = true;
      }
    }
  }
  return [...out];
}

export function kindLabel(k: TxnKind): string {
  return {
    income: 'receita',
    expense: 'despesa',
    transfer: 'transferência',
    card_payment: 'pagamento de fatura',
    investment: 'aporte',
    adjustment: 'ajuste',
  }[k];
}

export function statusLabel(s: TxnRow['status']): string {
  return { planned: 'previsto', cleared: 'efetivado', reconciled: 'conciliado' }[s];
}

export const isCashFlow = (k: TxnKind) => k === 'income' || k === 'expense';
export const isTransferLike = (k: TxnKind) =>
  k === 'transfer' || k === 'card_payment' || k === 'investment';

/** Categoria compatível com o tipo do lançamento (como `check_category`). */
export function checkCategory(categoryId: string | null | undefined, kind: TxnKind): void {
  if (!categoryId) return;
  const cat = find('category', categoryId) ?? notFound(`categoria ${categoryId}`);
  const path = categoryPaths().get(cat.id) ?? cat.name;
  if (kind === 'transfer') {
    // Transferência também pode ter categoria (a de uma perna só conta no fluxo por ela).
    if (cat.archived) invalid(`a categoria ${path} está arquivada`);
    return;
  }
  if (!isCashFlow(kind)) invalid(`lançamentos do tipo ${kindLabel(kind)} não usam categoria`);
  if (cat.kind !== kind) {
    invalid(
      `a categoria ${path} é de ${cat.kind === 'income' ? 'receita' : 'despesa'} e o lançamento é uma ${kindLabel(kind)}`,
    );
  }
  if (cat.archived) invalid(`a categoria ${path} está arquivada`);
}

// ── Contas e câmbio ─────────────────────────────────────────────

export function account(id: string): AccountRow {
  return find('account', id) ?? notFound(`conta ${id}`);
}

export function currencyAndKind(id: string): { currency: string; kind: AccountKind } {
  const a = account(id);
  return { currency: a.currency, kind: a.kind };
}

const SOURCE_RANK: Record<string, number> = { bcb_ptax_venda: 0, manual: 2 };

function lookupRate(base: string, quote: string, date: string) {
  return rows('fx_rate')
    .filter((r) => r.base === base && r.quote === quote && r.date <= date)
    .sort(
      (a, b) =>
        b.date.localeCompare(a.date) || (SOURCE_RANK[a.source] ?? 1) - (SOURCE_RANK[b.source] ?? 1),
    )[0];
}

const MINOR: Record<string, number> = { JPY: 0, KRW: 0, CLP: 0, PYG: 0 };
const minor = (c: string) => MINOR[c] ?? 2;

function roundAway(x: number): number {
  return Math.sign(x) * Math.round(Math.abs(x) + 1e-9);
}

/** `fx::convert`: taxa direta ou inversa da data (ou anterior); estimada se > 7 dias. */
export function convert(amount: number, from: string, to: string, date: string): Converted | null {
  if (from === to) {
    return {
      amount,
      currency: to,
      rate: '1',
      rate_date: date,
      source: 'identity',
      estimated: false,
    };
  }
  let rate: number;
  let rec = lookupRate(from, to, date);
  if (rec) rate = Number(rec.rate);
  else {
    rec = lookupRate(to, from, date);
    if (!rec) return null;
    rate = 1 / Number(rec.rate);
  }
  const major = amount / 10 ** minor(from);
  const out = roundAway(major * rate * 10 ** minor(to));
  return {
    amount: out,
    currency: to,
    rate: String(Number(rate.toFixed(10))),
    rate_date: rec.date,
    source: rec.source,
    estimated: diffDays(date, rec.date) > 7,
  };
}

export function accountView(a: AccountRow, todayIso = today()): AccountView {
  const end = monthEnd(todayIso);
  let bal = 0;
  let proj = 0;
  let rec = 0;
  let committed = 0;
  let last: string | null = null;
  for (const t of db.t.txn.values()) {
    if (t.account_id !== a.id) continue;
    if (t.status !== 'planned' && t.date <= todayIso) {
      bal += t.amount;
      if (!last || t.date > last) last = t.date;
    }
    if (t.date <= end) proj += t.amount;
    if (t.status === 'reconciled') rec += t.amount;
    if (t.status !== 'planned' || t.installment_plan_id) committed += t.amount;
  }
  const balance = a.opening_balance + bal;
  const fresh = freshnessOf(a.kind, a.opening_date, last, todayIso, a.archived);
  const conv = convert(balance, a.currency, baseCurrency(), todayIso);
  const card = cardDetails(a.id);
  const memberIds = rows('account_member')
    .filter((m) => m.account_id === a.id)
    .map((m) => m.member_id)
    .sort((x, y) => (find('member', x)?.sort_order ?? 0) - (find('member', y)?.sort_order ?? 0));
  return {
    id: a.id,
    name: a.name,
    kind: a.kind,
    currency: a.currency,
    institution_id: a.institution_id,
    institution_name: find('institution', a.institution_id)?.name ?? null,
    member_ids: memberIds,
    opening_balance: a.opening_balance,
    opening_date: a.opening_date,
    include_in_net_worth: a.include_in_net_worth,
    archived: a.archived,
    color: a.color,
    notes: a.notes,
    sort_order: a.sort_order,
    card,
    overdraft: overdraftOf(a.id),
    ...bankDetailsOf(a.id),
    ...retirementOf(a.id),
    balance,
    projected_balance: a.opening_balance + proj,
    reconciled_balance: a.opening_balance + rec,
    balance_base: conv?.amount ?? null,
    fx_estimated: conv?.estimated ?? false,
    last_txn_date: last,
    last_import_at: null,
    ...fresh,
    missed_month: null,
    available_credit:
      card?.credit_limit != null ? card.credit_limit + a.opening_balance + committed : null,
    overdraft_status: (() => {
      const o = overdraftOf(a.id);
      return o
        ? {
            ...overdraftStatus(o, balance),
            uncertain: fresh.balance_status != null && fresh.balance_status !== 'fresh',
          }
        : null;
    })(),
  };
}

export function accountViews(includeArchived: boolean): AccountView[] {
  return rows('account')
    .filter((a) => includeArchived || !a.archived)
    .sort(
      (a, b) =>
        Number(a.archived) - Number(b.archived) ||
        a.sort_order - b.sort_order ||
        a.name.localeCompare(b.name, 'pt-BR', { sensitivity: 'base' }),
    )
    .map((a) => accountView(a));
}

/** Saldo numa data: inicial + lançamentos (efetivados e, se pedido, previstos). */
export function balanceAt(accountId: string, asOf: string, includePlanned: boolean): number {
  const a = account(accountId);
  let sum = a.opening_balance;
  for (const t of db.t.txn.values()) {
    if (t.account_id !== accountId || t.date > asOf) continue;
    if (t.status !== 'planned' || includePlanned) sum += t.amount;
  }
  return sum;
}

// ── Cartões e faturas ───────────────────────────────────────────

export function cardDetails(accountId: string) {
  const c = find('credit_card', accountId);
  if (!c) return null;
  return {
    credit_limit: c.credit_limit,
    closing_day: c.closing_day,
    due_day: c.due_day,
    payment_account_id: c.payment_account_id,
    network: c.network,
    last4: c.last4,
  };
}

export interface StatementDates {
  reference_month: string;
  period_start: string;
  closing_date: string;
  due_date: string;
}

/** Fatura em que entra uma compra feita em `date` (como `cards::statement_dates`). */
export function statementDates(date: string, closingDay: number, dueDay: number): StatementDates {
  const { y, m } = parseYmd(date);
  const thisClose = clampDay(y, m, closingDay);
  const closeMonth = date <= thisClose ? date : addMonths(`${date.slice(0, 7)}-01`, 1);
  const cm = parseYmd(closeMonth);
  const closing = clampDay(cm.y, cm.m, closingDay);
  const dueMonth = dueDay > closingDay ? closing : addMonths(`${closing.slice(0, 7)}-01`, 1);
  const dm = parseYmd(dueMonth);
  const due = clampDay(dm.y, dm.m, dueDay);
  const prev = parseYmd(addMonths(`${closing.slice(0, 7)}-01`, -1));
  const prevClose = clampDay(prev.y, prev.m, closingDay);
  return {
    reference_month: monthOf(due),
    period_start: addDaysIso(prevClose, 1),
    closing_date: closing,
    due_date: due,
  };
}

function addDaysIso(iso: string, n: number): string {
  const { y, m, d } = parseYmd(iso);
  const t = new Date(Date.UTC(y, m - 1, d + n));
  return fmt({ y: t.getUTCFullYear(), m: t.getUTCMonth() + 1, d: t.getUTCDate() });
}

export function datesForReferenceMonth(
  month: string,
  closingDay: number,
  dueDay: number,
): StatementDates {
  const { y, m } = parseMonth(month);
  const due = clampDay(y, m, dueDay);
  const closeMonth = dueDay > closingDay ? `${month}-01` : addMonths(`${month}-01`, -1);
  const cm = parseYmd(closeMonth);
  const closing = clampDay(cm.y, cm.m, closingDay);
  const prev = parseYmd(addMonths(closeMonth, -1));
  const prevClose = clampDay(prev.y, prev.m, closingDay);
  return {
    reference_month: month,
    period_start: addDaysIso(prevClose, 1),
    closing_date: closing,
    due_date: due,
  };
}

function findStatement(accountId: string, month: string): StatementRow | undefined {
  return rows('card_statement').find(
    (s) => s.account_id === accountId && s.reference_month === month,
  );
}

function createStatement(accountId: string, d: StatementDates): string {
  const now = nowUtc();
  const id = newId();
  put('card_statement', {
    id,
    account_id: accountId,
    reference_month: d.reference_month,
    period_start: d.period_start,
    closing_date: d.closing_date,
    due_date: d.due_date,
    informed_total: null,
    created_at: now,
    updated_at: now,
  });
  return id;
}

/** Fatura (criada se preciso) de um lançamento num cartão; `null` fora de cartão. */
export function statementFor(accountId: string, date: string): string | null {
  const card = find('credit_card', accountId);
  if (!card) return null;
  const d = statementDates(date, card.closing_day, card.due_day);
  return findStatement(accountId, d.reference_month)?.id ?? createStatement(accountId, d);
}

export function requireCard(accountId: string) {
  const { kind } = currencyAndKind(accountId);
  if (kind !== 'credit_card') invalid('a conta escolhida não é um cartão de crédito');
  return (
    find('credit_card', accountId) ?? invalid('o cartão não tem dia de fechamento e vencimento')
  );
}

export function ensureStatement(accountId: string, month: string): string {
  const card = requireCard(accountId);
  return (
    findStatement(accountId, month)?.id ??
    createStatement(accountId, datesForReferenceMonth(month, card.closing_day, card.due_day))
  );
}

function statementSums(id: string) {
  let total = 0;
  let planned = 0;
  let paid = 0;
  let count = 0;
  for (const t of db.t.txn.values()) {
    if (t.statement_id !== id) continue;
    if (t.kind !== 'card_payment') {
      total -= t.amount;
      count++;
      if (t.status === 'planned') planned -= t.amount;
    } else if (t.status !== 'planned') paid += t.amount;
  }
  return { total, planned, paid, count };
}

/** Fatura que um pagamento em `date` quita: a mais antiga fechada com saldo; senão a da data. */
export function statementToPay(accountId: string, date: string): string | null {
  if (!find('credit_card', accountId)) return null;
  const pending = rows('card_statement')
    .filter((s) => s.account_id === accountId && s.closing_date < date)
    .sort((a, b) => a.reference_month.localeCompare(b.reference_month))
    .find((s) => {
      let owed = 0;
      for (const t of db.t.txn.values()) {
        if (t.statement_id !== s.id) continue;
        if (t.kind !== 'card_payment') owed -= t.amount;
        else if (t.status !== 'planned') owed -= t.amount;
      }
      return owed > 0;
    });
  return pending?.id ?? statementFor(accountId, date);
}

export function computeStatus(
  total: number,
  paid: number,
  closing: string,
  todayIso: string,
): StatementStatus {
  const closed = todayIso > closing;
  const remaining = total - paid;
  if (paid > 0 || (closed && total <= 0)) return remaining <= 0 ? 'paid' : 'partial';
  return closed ? 'closed' : 'open';
}

export function statementView(s: StatementRow, todayIso = today()): StatementView {
  const acc = account(s.account_id);
  const sums = statementSums(s.id);
  return {
    id: s.id,
    account_id: s.account_id,
    account_name: acc.name,
    reference_month: s.reference_month,
    period_start: s.period_start,
    closing_date: s.closing_date,
    due_date: s.due_date,
    status: computeStatus(sums.total, sums.paid, s.closing_date, todayIso),
    total: sums.total,
    planned: sums.planned,
    paid: sums.paid,
    remaining: sums.total - sums.paid,
    informed_total: s.informed_total,
    txn_count: sums.count,
    currency: acc.currency,
    dates_source: s.dates_source ?? 'calc',
    informed_minimum: s.informed_minimum ?? null,
    informed_limit: s.informed_limit ?? null,
    informed_previous_balance: s.informed_previous_balance ?? null,
  };
}

// ── Lançamentos ─────────────────────────────────────────────────

export interface NewTxn {
  account_id: string;
  date: string;
  amount: number;
  currency: string;
  kind: TxnKind;
  description: string;
  status?: TxnRow['status'];
  payee?: string | null;
  notes?: string | null;
  category_id?: string | null;
  member_id?: string | null;
  transfer_id?: string | null;
  statement_id?: string | null;
  installment_plan_id?: string | null;
  installment_number?: number | null;
  recurrence_id?: string | null;
  import_id?: string | null;
  external_id?: string | null;
  card_plastic_id?: string | null;
  orig_amount?: number | null;
  orig_currency?: string | null;
  created_at?: string;
}

/** Valor e moeda de origem (compra no exterior), como `transactions::check_orig`. */
export function checkOrig(
  amount: number,
  currency: string,
  origAmount: unknown,
  origCurrency: unknown,
): { orig_amount: number | null; orig_currency: string | null } {
  const cur = typeof origCurrency === 'string' ? origCurrency.trim().toUpperCase() : '';
  const value = typeof origAmount === 'number' ? origAmount : null;
  if (value == null && !cur) return { orig_amount: null, orig_currency: null };
  if (value == null || !cur) return invalid('informe o valor e a moeda de origem juntos');
  if (!/^[A-Z]{3}$/.test(cur)) invalid(`moeda inválida: ${cur}`);
  if (cur === currency)
    invalid('a moeda de origem é a mesma da conta; deixe o valor de origem vazio');
  if (value === 0) invalid('o valor na moeda de origem não pode ser zero');
  return { orig_amount: Math.abs(value) * Math.sign(amount), orig_currency: cur };
}

/** Insere sem regras; associa a fatura do cartão pela data quando `statement_id` vier vazio. */
/** Único membro da conta (o que os lançamentos sem membro herdam), como no núcleo. */
export function soleOwnerOf(accountId: string): string | null {
  const owners = rows('account_member').filter((m) => m.account_id === accountId);
  return owners.length === 1 ? owners[0]!.member_id : null;
}

/** Membro herdado: o do plástico do cartão, se tiver; senão o titular único da conta. */
export function inheritedMember(accountId: string, plasticId?: string | null): string | null {
  const own = plasticId ? find('card_plastic', plasticId)?.member_id : null;
  return own ?? soleOwnerOf(accountId);
}

/** A conta (ou um plástico) mudou de membros: os lançamentos com o membro herdado acompanham. */
export function reinheritMembers(accountId: string): void {
  for (const t of rows('txn')) {
    if (t.account_id !== accountId) continue;
    reinheritOne(t);
  }
}

/** Confere que o plástico existe e é do cartão do lançamento. */
export function requirePlasticOn(plasticId: string, accountId: string): void {
  const p = find('card_plastic', plasticId) ?? notFound(`cartão ${plasticId}`);
  if (p.account_id !== accountId) invalid('o plástico é de outro cartão');
}

/** Refaz o membro herdado de um lançamento (não mexe no escolhido à mão). */
export function reinheritOne(t: TxnRow): void {
  const auto = t.member_origin === 'auto' || (!t.member_origin && !t.member_id);
  const owner = inheritedMember(t.account_id, t.card_plastic_id);
  if (!auto || t.member_id === owner) return;
  patch('txn', t.id, {
    member_id: owner,
    member_origin: owner ? 'auto' : null,
    updated_at: nowUtc(),
  });
}

export function insertTxn(t: NewTxn): TxnRow {
  parseYmd(t.date);
  const now = t.created_at ?? nowUtc();
  const inherited = t.member_id ? null : inheritedMember(t.account_id, t.card_plastic_id);
  const row: TxnRow = {
    id: newId(),
    account_id: t.account_id,
    date: t.date,
    amount: t.amount,
    currency: t.currency,
    kind: t.kind,
    status: t.status ?? 'cleared',
    description: t.description.trim(),
    payee: t.payee ?? null,
    notes: t.notes ?? null,
    category_id: t.category_id ?? null,
    member_id: t.member_id ?? inherited,
    member_origin: t.member_id ? 'manual' : inherited ? 'auto' : null,
    transfer_id: t.transfer_id ?? null,
    statement_id: t.statement_id ?? statementFor(t.account_id, t.date),
    installment_plan_id: t.installment_plan_id ?? null,
    installment_number: t.installment_number ?? null,
    recurrence_id: t.recurrence_id ?? null,
    import_id: t.import_id ?? null,
    external_id: t.external_id ?? null,
    card_plastic_id: t.card_plastic_id ?? null,
    orig_amount: t.orig_amount ?? null,
    orig_currency: t.orig_currency ?? null,
    fingerprint: fingerprint(t.date, t.amount, t.description),
    created_at: now,
    updated_at: now,
  };
  return put('txn', row);
}

export function txnRow(id: string): TxnRow {
  return find('txn', id) ?? notFound(`lançamento ${id}`);
}

export function otherLegs(row: TxnRow): TxnRow[] {
  if (!row.transfer_id) return [];
  return rows('txn').filter((t) => t.transfer_id === row.transfer_id && t.id !== row.id);
}

export function ensureTag(name: string): string {
  const clean = name.trim();
  if (!clean) invalid('tags não podem ficar vazias');
  const existing = rows('tag').find((t) => t.name.toLowerCase() === clean.toLowerCase());
  if (existing) return existing.id;
  const now = nowUtc();
  return put('tag', { id: newId(), name: clean, color: null, created_at: now, updated_at: now }).id;
}

export function replaceTags(txnId: string, names: readonly string[]): void {
  const wanted = new Set(names.map(ensureTag));
  for (const link of rows('txn_tag')) {
    if (link.txn_id === txnId && !wanted.has(link.tag_id)) del('txn_tag', link.id);
  }
  for (const tagId of wanted) {
    const id = `${txnId}|${tagId}`;
    if (!find('txn_tag', id)) put('txn_tag', { id, txn_id: txnId, tag_id: tagId });
  }
}

export interface SplitIn {
  category_id?: string | null;
  member_id?: string | null;
  amount: number;
  notes?: string | null;
}

export function validateSplits(total: number, splits: readonly SplitIn[]): void {
  if (splits.length === 0) return;
  if (splits.length < 2) invalid('uma divisão precisa de pelo menos duas partes');
  const sum = splits.reduce((s, p) => s + p.amount, 0);
  if (sum !== total) {
    invalid(`a soma das partes (${brlCore(sum)}) difere do total (${brlCore(total)})`);
  }
  if (splits.some((p) => p.amount === 0 || p.amount > 0 !== total > 0)) {
    invalid('as partes da divisão devem ter o mesmo sinal do total e não podem ser zero');
  }
}

function brlCore(cents: number): string {
  const neg = cents < 0;
  const abs = Math.abs(cents);
  const whole = String(Math.floor(abs / 100)).replace(/\B(?=(\d{3})+(?!\d))/g, '.');
  return `${neg ? '-' : ''}R$ ${whole},${String(abs % 100).padStart(2, '0')}`;
}

export function checkSplits(total: number, kind: TxnKind, splits: readonly SplitIn[]): void {
  if (splits.length === 0) return;
  if (!isCashFlow(kind)) invalid(`lançamentos do tipo ${kindLabel(kind)} não podem ser divididos`);
  validateSplits(total, splits);
  for (const s of splits) {
    checkCategory(s.category_id, kind);
    if (s.member_id) requireMember(s.member_id);
  }
}

let splitSeq = 0;
export function replaceSplits(txnId: string, splits: readonly SplitIn[]): void {
  for (const s of rows('txn_split')) if (s.txn_id === txnId) del('txn_split', s.id);
  const now = nowUtc();
  for (const s of splits) {
    splitSeq += 1;
    put('txn_split', {
      id: newId(),
      txn_id: txnId,
      category_id: s.category_id ?? null,
      member_id: s.member_id ?? null,
      amount: s.amount,
      notes: cleanNotes(s.notes),
      seq: splitSeq,
      created_at: now,
      updated_at: now,
    });
  }
}

const cleanNotes = (v: unknown) => (typeof v === 'string' && v.trim() ? v.trim() : null);

/** Apaga linhas sem checagens, com divisões, tags e anexos. */
export function deleteRows(list: readonly TxnRow[]): number {
  const ids = new Set(list.map((r) => r.id));
  // Parcelas de dívida pagas por estes lançamentos voltam a ficar em aberto.
  forgetLoanTxns(ids);
  for (const id of ids) operationLinks().delete(id);
  for (const s of rows('txn_split')) if (ids.has(s.txn_id)) del('txn_split', s.id);
  for (const l of rows('txn_tag')) if (ids.has(l.txn_id)) del('txn_tag', l.id);
  for (const a of rows('attachment')) if (a.txn_id && ids.has(a.txn_id)) del('attachment', a.id);
  for (const id of ids) del('txn', id);
  return ids.size;
}

// ── Visões de lançamento ────────────────────────────────────────

export function txnViews(list: readonly TxnRow[]): TxnView[] {
  const paths = categoryPaths();
  const loans = loanLinks();
  const ops = operationLinks();
  const splitsBy = new Map<string, TxnView['splits']>();
  for (const s of [...db.t.txn_split.values()].sort((a, b) => a.seq - b.seq)) {
    const l = splitsBy.get(s.txn_id) ?? [];
    l.push({
      id: s.id,
      category_id: s.category_id,
      member_id: s.member_id,
      amount: s.amount,
      notes: s.notes,
    });
    splitsBy.set(s.txn_id, l);
  }
  const tagsBy = new Map<string, TxnView['tags']>();
  for (const link of db.t.txn_tag.values()) {
    const tag = find('tag', link.tag_id);
    if (!tag) continue;
    const l = tagsBy.get(link.txn_id) ?? [];
    l.push({ id: tag.id, name: tag.name, color: tag.color });
    tagsBy.set(link.txn_id, l);
  }
  const attachments = new Map<string, number>();
  for (const a of db.t.attachment.values()) {
    if (a.txn_id) attachments.set(a.txn_id, (attachments.get(a.txn_id) ?? 0) + 1);
  }
  const legsBy = new Map<string, TxnRow[]>();
  for (const t of db.t.txn.values()) {
    if (!t.transfer_id) continue;
    const l = legsBy.get(t.transfer_id) ?? [];
    l.push(t);
    legsBy.set(t.transfer_id, l);
  }
  return list.map((t) => {
    const other = t.transfer_id
      ? (legsBy.get(t.transfer_id) ?? [])
          .filter((o) => o.id !== t.id)
          .sort((a, b) => a.id.localeCompare(b.id))[0]
      : undefined;
    const st = find('card_statement', t.statement_id);
    const plan = find('installment_plan', t.installment_plan_id);
    return {
      id: t.id,
      account_id: t.account_id,
      account_name: find('account', t.account_id)?.name ?? '?',
      date: t.date,
      amount: t.amount,
      currency: t.currency,
      kind: t.kind,
      status: t.status,
      description: t.description,
      payee: t.payee,
      notes: t.notes,
      category_id: t.category_id,
      category_path: t.category_id ? (paths.get(t.category_id) ?? null) : null,
      member_id: t.member_id,
      member_name: find('member', t.member_id)?.name ?? null,
      transfer_id: t.transfer_id,
      counterpart_account_id: other?.account_id ?? null,
      counterpart_account_name: other ? (find('account', other.account_id)?.name ?? null) : null,
      statement_id: t.statement_id,
      statement_due_date: st?.due_date ?? null,
      installment_plan_id: t.installment_plan_id,
      installment_number: t.installment_number,
      installment_total: plan?.installments ?? null,
      recurrence_id: t.recurrence_id,
      import_id: t.import_id,
      external_id: t.external_id,
      splits: splitsBy.get(t.id) ?? [],
      tags: (tagsBy.get(t.id) ?? []).sort((a, b) =>
        a.name.localeCompare(b.name, 'pt-BR', { sensitivity: 'base' }),
      ),
      attachment_count: attachments.get(t.id) ?? 0,
      created_at: t.created_at,
      updated_at: t.updated_at,
      project_id: t.project_id ?? null,
      project_name: find('project', t.project_id ?? null)?.name ?? null,
      card_plastic_id: t.card_plastic_id ?? null,
      card_last4: find('card_plastic', t.card_plastic_id ?? null)?.last4 ?? null,
      orig_amount: t.orig_amount ?? null,
      orig_currency: t.orig_currency ?? null,
      loan_id: loans.get(t.id)?.loan_id ?? null,
      loan_name: loans.get(t.id)?.loan_name ?? null,
      loan_installment_id: loans.get(t.id)?.installment_id ?? null,
      loan_installment_number: loans.get(t.id)?.installment_number ?? null,
      operation: ops.has(t.id)
        ? {
            kind: ops.get(t.id)!.kind,
            target_id: ops.get(t.id)!.target_id,
            label: ops.get(t.id)!.label,
          }
        : null,
    };
  });
}

export function txnView(id: string): TxnView {
  return txnViews([txnRow(id)])[0]!;
}

function isUncategorized(t: TxnRow, parts: readonly SplitRowLite[] | undefined): boolean {
  const loneTransfer = t.kind === 'transfer' && !t.transfer_id;
  if ((!isCashFlow(t.kind) && !loneTransfer) || t.category_id) return false;
  return !parts || parts.length === 0 || parts.some((s) => !s.category_id);
}

interface SplitRowLite {
  category_id: string | null;
  member_id: string | null;
}

/** Filtro de `transactions.query` (mesma semântica do `build_where` do núcleo). */
export function filterTxns(f: TxnFilter): TxnRow[] {
  for (const d of [f.date_from, f.date_to]) if (d) parseYmd(d);
  if (f.date_from && f.date_to && f.date_from > f.date_to) {
    invalid('a data inicial é posterior à data final');
  }
  const cats =
    f.category_ids && f.category_ids.length > 0
      ? new Set(
          f.include_subcategories === false ? f.category_ids : withDescendants(f.category_ids),
        )
      : null;
  const words = f.text ? searchFold(f.text).split(/\s+/).filter(Boolean) : [];
  const splitsBy = new Map<string, SplitRowLite[]>();
  for (const sp of db.t.txn_split.values()) {
    const list = splitsBy.get(sp.txn_id) ?? [];
    list.push(sp);
    splitsBy.set(sp.txn_id, list);
  }
  const tagsBy = new Map<string, string[]>();
  for (const l of db.t.txn_tag.values()) {
    const list = tagsBy.get(l.txn_id) ?? [];
    list.push(l.tag_id);
    tagsBy.set(l.txn_id, list);
  }
  const inRange = (d: string) =>
    (!f.date_from || d >= f.date_from) && (!f.date_to || d <= f.date_to);
  return rows('txn').filter((t) => {
    if (f.basis === 'cash' && (f.date_from || f.date_to)) {
      const own = (!t.statement_id || t.kind === 'card_payment') && inRange(t.date);
      const due = find('card_statement', t.statement_id)?.due_date;
      const byDue = t.kind !== 'card_payment' && !!t.statement_id && !!due && inRange(due);
      if (!own && !byDue) return false;
    } else if (!inRange(t.date)) return false;
    if (f.account_ids?.length && !f.account_ids.includes(t.account_id)) return false;
    if (cats) {
      const hit =
        (t.category_id && cats.has(t.category_id)) ||
        (splitsBy.get(t.id) ?? []).some((s) => s.category_id && cats.has(s.category_id));
      if (!hit) return false;
    }
    if (f.member_ids?.length) {
      const m = f.member_ids;
      const hit =
        (t.member_id && m.includes(t.member_id)) ||
        (splitsBy.get(t.id) ?? []).some((s) => s.member_id && m.includes(s.member_id));
      if (!hit) return false;
    }
    if (f.kinds?.length && !f.kinds.includes(t.kind)) return false;
    if (f.statuses?.length && !f.statuses.includes(t.status)) return false;
    if (words.length) {
      const hay = searchFold(`${t.description}\u0001${t.payee ?? ''}\u0001${t.notes ?? ''}`);
      if (!words.every((w) => hay.includes(w))) return false;
    }
    if (f.amount_min != null && Math.abs(t.amount) < Math.abs(f.amount_min)) return false;
    if (f.amount_max != null && Math.abs(t.amount) > Math.abs(f.amount_max)) return false;
    if (f.tag_ids?.length && !(tagsBy.get(t.id) ?? []).some((id) => f.tag_ids!.includes(id))) {
      return false;
    }
    if (f.project_ids?.length && !(t.project_id && f.project_ids.includes(t.project_id))) {
      return false;
    }
    if (f.import_id && t.import_id !== f.import_id) return false;
    if (f.statement_id && t.statement_id !== f.statement_id) return false;
    if (f.uncategorized != null && isUncategorized(t, splitsBy.get(t.id)) !== f.uncategorized) {
      return false;
    }
    return true;
  });
}

export function sortTxns(list: TxnRow[], sort: TxnFilter['sort']): TxnRow[] {
  const byDateDesc = (a: TxnRow, b: TxnRow) =>
    b.date.localeCompare(a.date) ||
    b.created_at.localeCompare(a.created_at) ||
    b.id.localeCompare(a.id);
  const cmp: Record<string, (a: TxnRow, b: TxnRow) => number> = {
    date_desc: byDateDesc,
    date_asc: (a, b) => -byDateDesc(a, b),
    amount_desc: (a, b) =>
      b.amount - a.amount || b.date.localeCompare(a.date) || b.id.localeCompare(a.id),
    amount_asc: (a, b) =>
      a.amount - b.amount || b.date.localeCompare(a.date) || b.id.localeCompare(a.id),
  };
  // Por texto, como o núcleo: sem acentos e maiúsculas, vazios por último, empate pela data.
  const text: Record<string, (t: TxnRow) => string | null> = {
    description: (t) => t.description,
    account: (t) => find('account', t.account_id)?.name ?? null,
    category: (t) => (t.category_id ? (categoryPaths().get(t.category_id) ?? null) : null),
    member: (t) => (t.member_id ? (find('member', t.member_id)?.name ?? null) : null),
    status: (t) => String({ planned: 0, cleared: 1, reconciled: 2 }[t.status]),
  };
  for (const [k, get] of Object.entries(text))
    for (const dir of [1, -1]) {
      cmp[`${k}_${dir === 1 ? 'asc' : 'desc'}`] = (a, b) => {
        const x = get(a);
        const y = get(b);
        if (!x || !y) return x === y ? byDateDesc(a, b) : x ? -1 : 1;
        return dir * fold(x).localeCompare(fold(y), 'pt-BR') || byDateDesc(a, b);
      };
    }
  return [...list].sort(cmp[sort ?? 'date_desc']);
}

/** Ids de todos os lançamentos do filtro, na ordem pedida (sem paginação). */
export function queryTxnIds(f: TxnFilter): string[] {
  return sortTxns(filterTxns(f), f.sort).map((t) => t.id);
}

/** Lançamentos com a mesma descrição dos escolhidos (sem caixa, acento e espaços extras). */
export function sameDescription(
  ids: readonly string[],
  f: TxnFilter = {},
): { ids: string[]; descriptions: string[] } {
  const key = (s: string) => searchFold(s).split(/\s+/).filter(Boolean).join(' ');
  if (ids.length === 0) invalid('escolha ao menos um lançamento');
  const keys = new Set<string>();
  const descriptions: string[] = [];
  for (const id of ids) {
    const t = find('txn', id) ?? notFound(`lançamento ${id}`);
    const k = key(t.description);
    if (k && !keys.has(k)) {
      keys.add(k);
      descriptions.push(t.description.trim());
    }
  }
  const out = [...new Set(ids)];
  const seen = new Set(out);
  for (const t of sortTxns(filterTxns({ ...f, limit: undefined, offset: undefined }), f.sort)) {
    if (keys.has(key(t.description)) && !seen.has(t.id)) {
      seen.add(t.id);
      out.push(t.id);
    }
  }
  return { ids: out, descriptions };
}

export function queryTxns(f: TxnFilter): TxnPage {
  const all = sortTxns(filterTxns(f), f.sort);
  const byCur = new Map<string, CurrencyTotal>();
  for (const t of all) {
    const tot = byCur.get(t.currency) ?? { currency: t.currency, inflow: 0, outflow: 0, net: 0 };
    if (t.amount > 0) tot.inflow += t.amount;
    else tot.outflow -= t.amount;
    tot.net = tot.inflow - tot.outflow;
    byCur.set(t.currency, tot);
  }
  const limit = Math.min(Math.max(f.limit ?? 100, 1), 2000);
  const offset = Math.max(f.offset ?? 0, 0);
  return {
    items: txnViews(all.slice(offset, offset + limit)),
    total: all.length,
    totals: [...byCur.values()].sort((a, b) => a.currency.localeCompare(b.currency)),
  };
}
