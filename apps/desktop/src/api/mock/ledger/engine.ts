/**
 * Motor de agregação do mock: as mesmas regras de `reports::engine` aplicadas ao livro
 * fictício. Só `income`/`expense` contam; competência usa a data do lançamento e caixa o
 * vencimento da fatura; divisões contam pelas partes; estornos reduzem o gasto; outras
 * moedas viram a moeda base pelo câmbio da data (EUR fica sem câmbio, de propósito).
 */
import type { Basis } from '@/api/generated/Basis';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import { MONTHS_SHORT } from '@/lib/format';
import { monthIndex, parseIso } from '@/lib/months';

import { ACCOUNTS, BASE, category, rootOf } from './fixture';
import { getLedger, mockToday, type LedgerTxn } from './ledger';
import { db } from './store';

// ── Câmbio ───────────────────────────────────────────────────────

export interface Conversion {
  amount: number;
  rate: string;
  estimated: boolean;
  source: string | null;
}

/** PTAX fictícia do dólar por mês (estável); EUR não tem câmbio cadastrado. */
export function usdRate(date: string): number {
  const i = monthIndex(date.slice(0, 7));
  return (
    Math.round((5.18 + 0.22 * Math.sin(i / 3.1) + 0.0004 * parseIso(date).d) * 10_000) / 10_000
  );
}

/** Reais por unidade da moeda: só o par USD/BRL tem câmbio no mock. */
function brlPer(currency: string, date: string): number | null {
  if (currency === 'BRL') return 1;
  return currency === 'USD' ? usdRate(date) : null;
}

/** Para a moeda base da família: USD → BRL pela PTAX; numa família em dólar, BRL → USD. */
export function convert(amount: number, currency: string, date: string): Conversion | null {
  if (currency === BASE) return { amount, rate: '1', estimated: false, source: null };
  const from = brlPer(currency, date);
  const to = brlPer(BASE, date);
  if (from == null || to == null) return null;
  const rate = from / to;
  // No dia de hoje a PTAX ainda não saiu: usa a do dia anterior, marcada como estimada.
  const estimated = date >= mockToday();
  return {
    amount: Math.round(amount * rate),
    rate: rate.toFixed(4),
    estimated,
    source: 'bcb_ptax_venda',
  };
}

export class Fx {
  missing = new Set<string>();
  estimated = false;
  used = new Map<string, { currency: string; rate: string; date: string; estimated: boolean }>();
  convert(amount: number, currency: string, date: string): Conversion | null {
    const c = convert(amount, currency, date);
    if (!c) {
      this.missing.add(currency);
      return null;
    }
    if (currency !== BASE) {
      this.used.set(`${currency}:${date}`, {
        currency,
        rate: c.rate,
        date,
        estimated: c.estimated,
      });
      this.estimated ||= c.estimated;
    }
    return c;
  }
}

// ── Consulta de fluxo ────────────────────────────────────────────

export interface FlowQuery {
  from: string;
  to: string;
  basis: Basis;
  status?: 'realized' | 'planned';
  kind?: CategoryKind;
  member_ids?: readonly string[] | null;
  account_ids?: readonly string[] | null;
  /** Já expandido com as descendentes. */
  category_ids?: ReadonlySet<string> | null;
}

export interface FlowPart {
  txn: LedgerTxn;
  /** Data efetiva conforme a base. */
  eff: string;
  category_id: string | null;
  member_id: string | null;
  /** Com sinal, na moeda do lançamento. */
  amount: number;
  /** Parte de uma divisão. */
  split: boolean;
}

export function effDate(t: LedgerTxn, basis: Basis): string {
  if (basis === 'cash' && t.statement_id) {
    return getLedger().statements.get(t.statement_id)?.due_date ?? t.date;
  }
  return t.date;
}

/**
 * Tipo no fluxo de caixa, como o núcleo: transferência de uma perna só com categoria conta
 * como despesa (saída) ou receita (entrada); as demais transferências ficam fora.
 */
export function flowKind(t: LedgerTxn): 'income' | 'expense' | null {
  if (t.kind === 'income' || t.kind === 'expense') return t.kind;
  if (t.kind === 'transfer' && !t.transfer_id && t.category_id)
    return t.amount < 0 ? 'expense' : 'income';
  return null;
}

/** Único titular da conta (o núcleo usa como titular dos lançamentos sem titular). */
export function soleOwner(accountId: string): string | null {
  const owners = [...db.t.account_member.values()].filter((m) => m.account_id === accountId);
  return owners.length === 1 ? owners[0]!.member_id : null;
}

export function flowParts(q: FlowQuery): FlowPart[] {
  const status = q.status ?? 'realized';
  const out: FlowPart[] = [];
  for (const t of getLedger().txns) {
    const fk = flowKind(t);
    if (q.kind ? fk !== q.kind : !fk) continue;
    if (status === 'realized' ? t.status === 'planned' : t.status !== 'planned') continue;
    const eff = effDate(t, q.basis);
    if (eff < q.from || eff > q.to) continue;
    if (q.account_ids?.length && !q.account_ids.includes(t.account_id)) continue;
    // Titular: o da divisão, o do lançamento ou, como no núcleo, o único titular da conta.
    const member = t.member_id ?? soleOwner(t.account_id);
    const parts = t.splits?.length
      ? t.splits.map((s) => ({ ...s, member_id: s.member_id ?? member, split: true }))
      : [{ category_id: t.category_id, member_id: member, amount: t.amount, split: false }];
    for (const p of parts) {
      if (q.member_ids?.length && !(p.member_id && q.member_ids.includes(p.member_id))) continue;
      if (q.category_ids && !(p.category_id && q.category_ids.has(p.category_id))) continue;
      out.push({ txn: t, eff, ...p });
    }
  }
  return out;
}

export type KeyFn = (p: FlowPart) => string | null;

export const KEY = {
  none: () => null,
  category: (p: FlowPart) => p.category_id,
  root: (p: FlowPart) => (p.category_id ? rootOf(p.category_id) : null),
  member: (p: FlowPart) => p.member_id,
  account: (p: FlowPart) => p.txn.account_id,
  month: (p: FlowPart) => p.eff.slice(0, 7),
  day: (p: FlowPart) => p.eff,
} satisfies Record<string, KeyFn>;

export interface Cell {
  income: number;
  expense: number;
  incomeIds: Set<string>;
  expenseIds: Set<string>;
  missing: Set<string>;
  estimated: boolean;
}

export function emptyCell(): Cell {
  return {
    income: 0,
    expense: 0,
    incomeIds: new Set(),
    expenseIds: new Set(),
    missing: new Set(),
    estimated: false,
  };
}

export const cellAmount = (c: Cell, kind: CategoryKind) =>
  kind === 'income' ? c.income : c.expense;
export const cellCount = (c: Cell, kind: CategoryKind) =>
  kind === 'income' ? c.incomeIds.size : c.expenseIds.size;

const NUL = '\u0000';
export const encodeKey = (a: string | null, b: string | null = null) => `${a ?? NUL}|${b ?? NUL}`;
export function decodeKey(k: string): [string | null, string | null] {
  const [a, b] = k.split('|');
  return [a === NUL ? null : (a ?? null), b === NUL ? null : (b ?? null)];
}

export interface Agg {
  cells: Map<string, Cell>;
  missing: Set<string>;
  estimated: boolean;
  fx: Fx;
}

export function aggregate(
  q: FlowQuery,
  k1: KeyFn = KEY.none,
  k2: KeyFn = KEY.none,
  fx = new Fx(),
): Agg {
  const cells = new Map<string, Cell>();
  for (const p of flowParts(q)) {
    const key = encodeKey(k1(p), k2(p));
    let cell = cells.get(key);
    if (!cell) cells.set(key, (cell = emptyCell()));
    const c = fx.convert(p.amount, p.txn.currency, p.txn.date);
    if (!c) {
      cell.missing.add(p.txn.currency);
      continue;
    }
    if (c.estimated) cell.estimated = true;
    const v = c.amount;
    if (flowKind(p.txn) === 'income') {
      cell.income += v;
      cell.incomeIds.add(p.txn.id);
    } else {
      cell.expense -= v;
      cell.expenseIds.add(p.txn.id);
    }
  }
  return { cells, missing: fx.missing, estimated: fx.estimated, fx };
}

export function total(agg: Agg): Cell {
  const t = emptyCell();
  for (const c of agg.cells.values()) {
    t.income += c.income;
    t.expense += c.expense;
    c.incomeIds.forEach((id) => t.incomeIds.add(id));
    c.expenseIds.forEach((id) => t.expenseIds.add(id));
    c.missing.forEach((m) => t.missing.add(m));
    t.estimated ||= c.estimated;
  }
  agg.missing.forEach((m) => t.missing.add(m));
  t.estimated ||= agg.estimated;
  return t;
}

// ── Saldos ───────────────────────────────────────────────────────

export interface BalanceRow {
  id: string;
  name: string;
  currency: string;
  balance: number;
}

export function balancesAt(asOf: string): BalanceRow[] {
  const { start, txns } = getLedger();
  return ACCOUNTS.map((a) => {
    let balance = start <= asOf ? a.opening_balance : 0;
    for (const t of txns) {
      if (t.account_id === a.id && t.status !== 'planned' && t.date <= asOf) balance += t.amount;
    }
    return { id: a.id, name: a.name, currency: a.currency, balance };
  });
}

// ── Rótulos ──────────────────────────────────────────────────────

/** `2026-10` → `out/2026` (como `engine::month_label`). */
export function monthLabel(month: string): string {
  const { y, m } = parseIso(month);
  return `${MONTHS_SHORT[m - 1] ?? ''}/${y}`;
}

export function brDate(iso: string): string {
  const { y, m, d } = parseIso(iso);
  return `${String(d).padStart(2, '0')}/${String(m).padStart(2, '0')}/${y}`;
}

export function basisLabel(basis: Basis): string {
  return basis === 'cash' ? 'caixa' : 'competência';
}

export function categoryLabel(id: string | null): {
  label: string;
  path: string | null;
  parent: string | null;
} {
  const c = category(id);
  if (!c) return { label: id ?? 'Sem categoria', path: null, parent: null };
  return { label: c.name, path: c.path, parent: c.parent_id };
}

export const ratio = (a: number, b: number): number | null => (b > 0 ? a / b : null);

/** Arredondamento como `engine::div_round` (meio para longe do zero). */
export function divRound(a: number, b: number): number {
  if (b === 0) return 0;
  const q = a / b;
  return q >= 0 ? Math.floor(q + 0.5) : -Math.floor(-q + 0.5);
}

export function divCeil(a: number, b: number): number {
  if (b <= 0) return 0;
  return Math.ceil(a / b);
}

/** Primeiro mês com receita/despesa efetivada. */
export function firstFlowMonth(): string | null {
  const t = getLedger().txns.find(
    (x) => (x.kind === 'income' || x.kind === 'expense') && x.status !== 'planned',
  );
  return t ? t.date.slice(0, 7) : null;
}
