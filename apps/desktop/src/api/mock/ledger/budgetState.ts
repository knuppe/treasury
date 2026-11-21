/**
 * Orçamento do mock (estado em memória) e o cálculo de previsto × realizado, espelhando
 * `budget/mod.rs`: realizado e comprometido somam as subcategorias; pais sem linha própria
 * somam o previsto dos filhos; categorias com gasto e sem orçamento também aparecem.
 */
import type { Basis } from '@/api/generated/Basis';
import type { BudgetLineInput } from '@/api/generated/BudgetLineInput';
import type { BudgetLineStatus } from '@/api/generated/BudgetLineStatus';
import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { BudgetTotals } from '@/api/generated/BudgetTotals';
import { monthBounds, monthElapsed, monthIndex, shiftMonth } from '@/lib/months';

import {
  aggregate,
  cellAmount,
  decodeKey,
  divCeil,
  firstFlowMonth,
  KEY,
  monthLabel,
  ratio,
  total,
} from './engine';
import { BASE, CATEGORIES, cat, category } from './fixture';
import { getLedger, mockToday } from './ledger';

type Lines = Map<string, number>;

let budgets = new Map<string, Lines>();

/** Linhas iniciais: os três meses anteriores e o atual; o próximo mês fica sem orçamento. */
function seed(): Map<string, Lines> {
  const current = mockToday().slice(0, 7);
  const lines: [string, number][] = [
    [cat('moradia'), 520_000],
    [cat('alimentacao', 'mercado'), 180_000],
    [cat('alimentacao', 'restaurantes'), 60_000],
    [cat('alimentacao', 'delivery'), 40_000],
    [cat('alimentacao', 'padaria'), 30_000],
    [cat('transporte'), 160_000],
    [cat('saude'), 210_000],
    [cat('educacao'), 260_000],
    [cat('lazer'), 120_000],
    [cat('compras'), 140_000],
    [cat('cuidados'), 60_000],
    [cat('assinaturas'), 32_000],
    [cat('impostos'), 6_000],
    [cat('seguros'), 16_000],
    [cat('filhos'), 45_000],
    [cat('salario'), 2_540_000],
    [cat('freelance'), 120_000],
  ];
  const out = new Map<string, Lines>();
  for (let i = -3; i <= 0; i++) out.set(shiftMonth(current, i), new Map(lines));
  return out;
}

export function resetBudgets(): void {
  budgets = seed();
}

resetBudgets();

function linesOf(month: string): Lines {
  return budgets.get(month) ?? new Map<string, number>();
}

export function budgetMonths(): string[] {
  return [...budgets.entries()]
    .filter(([, l]) => l.size > 0)
    .map(([m]) => m)
    .sort()
    .reverse();
}

function ancestorsInclusive(id: string): string[] {
  const out: string[] = [];
  let c = category(id);
  while (c) {
    out.push(c.id);
    c = category(c.parent_id);
  }
  return out;
}

export function budgetGet(month: string, basis: Basis): BudgetStatus {
  const { from, to } = monthBounds(month);
  const own = linesOf(month);
  const realized = aggregate({ from, to, basis }, KEY.category);
  const plannedTxns = aggregate({ from, to, basis, status: 'planned' }, KEY.category);
  const actual = new Map<string, number>();
  const committed = new Map<string, number>();
  for (const [agg, target] of [
    [realized, actual],
    [plannedTxns, committed],
  ] as const) {
    for (const [k, cell] of agg.cells) {
      const info = category(decodeKey(k)[0]);
      if (!info) continue;
      const v = cellAmount(cell, info.kind);
      if (v === 0) continue;
      for (const a of ancestorsInclusive(info.id)) {
        if (category(a)?.kind === info.kind) target.set(a, (target.get(a) ?? 0) + v);
      }
    }
  }
  const planned = new Map<string, number>();
  for (const c of [...CATEGORIES].reverse()) {
    const v =
      own.get(c.id) ??
      CATEGORIES.filter((x) => x.parent_id === c.id).reduce(
        (s, x) => s + (planned.get(x.id) ?? 0),
        0,
      );
    if (v !== 0) planned.set(c.id, v);
  }
  const lines: BudgetLineStatus[] = [];
  const totals: BudgetTotals = {
    planned_expense: 0,
    actual_expense: 0,
    planned_income: 0,
    actual_income: 0,
    committed_expense: 0,
    committed_income: 0,
    uncategorized_expense: 0,
  };
  for (const c of CATEGORIES) {
    const p = planned.get(c.id) ?? 0;
    const a = actual.get(c.id) ?? 0;
    const cm = committed.get(c.id) ?? 0;
    const hasOwn = own.has(c.id);
    if (p === 0 && a === 0 && cm === 0 && !hasOwn) continue;
    if (!c.parent_id) {
      if (c.kind === 'expense') totals.planned_expense += p;
      else totals.planned_income += p;
    }
    lines.push({
      category_id: c.id,
      category_name: c.name,
      category_path: c.path,
      parent_id: c.parent_id,
      kind: c.kind,
      planned: p,
      actual: a,
      remaining: p - a,
      ratio: ratio(a, p),
      committed: cm,
      has_own_line: hasOwn,
      color: null,
      archived: c.archived,
    });
  }
  const r = total(realized);
  const pl = total(plannedTxns);
  totals.actual_expense = r.expense;
  totals.actual_income = r.income;
  totals.committed_expense = pl.expense;
  totals.committed_income = pl.income;
  totals.uncategorized_expense = realized.cells.get('\u0000|\u0000')?.expense ?? 0;
  const missing = new Set([...realized.missing, ...plannedTxns.missing]);
  return {
    month,
    basis,
    lines,
    totals,
    month_elapsed: monthElapsed(month, getLedger().today),
    currency: BASE,
    missing_fx: [...missing].sort(),
    estimated: realized.estimated || plannedTxns.estimated,
  };
}

export class MockError extends Error {
  readonly code: string;
  constructor(code: string, message: string) {
    super(message);
    this.code = code;
  }
}

export function budgetSetLines(month: string, input: readonly BudgetLineInput[]): void {
  for (const l of input) {
    if (l.amount < 0) throw new MockError('invalid', 'o valor previsto não pode ser negativo');
    if (!category(l.category_id))
      throw new MockError('not_found', `não encontrado: categoria ${l.category_id}`);
  }
  const lines = new Map(linesOf(month));
  for (const l of input) {
    if (l.amount === 0) lines.delete(l.category_id);
    else lines.set(l.category_id, l.amount);
  }
  budgets.set(month, lines);
}

export function budgetCopy(from: string, to: string, overwrite: boolean): number {
  if (from === to) throw new MockError('invalid', 'os meses de origem e destino são iguais');
  const src = linesOf(from);
  if (src.size === 0)
    throw new MockError('not_found', `não encontrado: orçamento de ${monthLabel(from)}`);
  const dest = new Map(linesOf(to));
  let n = 0;
  for (const [id, amount] of src) {
    if (category(id)?.archived || (!overwrite && dest.has(id))) continue;
    dest.set(id, amount);
    n += 1;
  }
  budgets.set(to, dest);
  return n;
}

/** Média dos meses completos anteriores, arredondada para cima ao múltiplo de R$ 10. */
export function budgetSuggest(month: string, monthsBack: number, basis: Basis): BudgetLineInput[] {
  const current = getLedger().today.slice(0, 7);
  const before = monthIndex(month) > monthIndex(current) ? current : month;
  const first = firstFlowMonth();
  if (!first) return [];
  const n = Math.min(Math.max(monthsBack, 1), 24);
  const months: string[] = [];
  for (let i = n; i >= 1; i--) {
    const m = shiftMonth(before, -i);
    if (m >= first) months.push(m);
  }
  if (months.length === 0) return [];
  const agg = aggregate(
    {
      from: monthBounds(months[0]!).from,
      to: monthBounds(months[months.length - 1]!).to,
      basis,
      kind: 'expense',
    },
    KEY.root,
  );
  const stepCents = 1_000;
  const out: BudgetLineInput[] = [];
  for (const c of CATEGORIES) {
    if (c.parent_id || c.kind !== 'expense' || c.archived) continue;
    const t = agg.cells.get(`${c.id}|\u0000`)?.expense ?? 0;
    if (t <= 0) continue;
    out.push({ category_id: c.id, amount: divCeil(t, months.length * stepCents) * stepCents });
  }
  return out;
}
