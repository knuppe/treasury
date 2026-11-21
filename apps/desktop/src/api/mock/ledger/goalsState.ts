/**
 * Metas do mock (estado em memória) com progresso e projeção, e a calculadora de reserva
 * de emergência, espelhando `goals/mod.rs`.
 */
import type { GoalSourceRef } from '@/api/generated/GoalSourceRef';
import type { GoalSourceValue } from '@/api/generated/GoalSourceValue';
import type { AnnualExpenses } from '@/api/generated/AnnualExpenses';
import type { EmergencyReserve } from '@/api/generated/EmergencyReserve';
import type { Goal } from '@/api/generated/Goal';
import type { GoalGrowth } from '@/api/generated/GoalGrowth';
import type { GoalInput } from '@/api/generated/GoalInput';
import type { GoalPatch } from '@/api/generated/GoalPatch';
import type { GoalView } from '@/api/generated/GoalView';
import { addMonthsToDate, monthBounds, monthIndex, shiftMonth } from '@/lib/months';

import { MockError } from './budgetState';
import { aggregate, balancesAt, divCeil, divRound, firstFlowMonth, Fx, KEY } from './engine';
import { ACCOUNTS, BASE, category } from './fixture';
import { getLedger, mockToday } from './ledger';

export const CONTRIBUTION_MONTHS = 6;

let goals: Goal[] = [];
let created = new Map<string, number>();
let seq = 0;

function seed(): Goal[] {
  const today = mockToday();
  return [
    {
      id: 'goal-reserva',
      name: 'Reserva de emergência',
      kind: 'emergency',
      target_amount: 9_000_000,
      currency: BASE,
      target_date: addMonthsToDate(today, 30).slice(0, 8) + '01',
      initial_amount: 0,
      monthly_contribution: 120_000,
      account_ids: ['acc-poupanca'],
      archived: false,
      notes: 'Seis meses de gastos essenciais.',
      real_return: null,
    },
    {
      id: 'goal-japao',
      name: 'Viagem ao Japão',
      kind: 'travel',
      target_amount: 3_800_000,
      currency: BASE,
      target_date: addMonthsToDate(today, 11).slice(0, 8) + '15',
      initial_amount: 950_000,
      monthly_contribution: 280_000,
      account_ids: [],
      archived: false,
      notes: null,
      real_return: null,
    },
    {
      id: 'goal-carro',
      name: 'Troca do carro',
      kind: 'purchase',
      target_amount: 12_000_000,
      currency: BASE,
      target_date: addMonthsToDate(today, 20).slice(0, 8) + '01',
      initial_amount: 3_000_000,
      monthly_contribution: 350_000,
      account_ids: [],
      archived: false,
      notes: 'Entrada à vista; o resto financiado.',
      real_return: null,
    },
  ];
}

/** Fontes ligadas além das contas (etiquetas; as posições ficam no mock da educação). */
export const extraSources = new Map<string, GoalSourceRef[]>();

export function resetGoals(): void {
  extraSources.clear();
  goals = seed();
  created = new Map(goals.map((g, i) => [g.id, i]));
  seq = goals.length;
}

resetGoals();

function monthsUntil(a: string, b: string): number {
  return monthIndex(b.slice(0, 7)) - monthIndex(a.slice(0, 7));
}

/** Entradas − saídas efetivadas nas contas, no período, em BRL. */
function netInflow(ids: readonly string[], from: string, to: string, fx: Fx): number {
  let sum = 0;
  for (const t of getLedger().txns) {
    if (!ids.includes(t.account_id) || t.status === 'planned' || t.date < from || t.date > to)
      continue;
    const c = fx.convert(t.amount, t.currency, t.date);
    if (c) sum += c.amount;
  }
  return sum;
}

export function goalView(goal: Goal): GoalView {
  const today = getLedger().today;
  const fx = new Fx();
  let current = goal.initial_amount;
  const breakdown: GoalSourceValue[] = [];
  if (goal.initial_amount !== 0) {
    breakdown.push({
      kind: 'initial',
      ref_id: null,
      account_id: null,
      name: 'Guardado fora das contas',
      value: goal.initial_amount,
      value_used: 'amount',
      share: null,
      ignored_negative: null,
      stale: false,
    });
  }
  const linked = balancesAt(today).filter((b) => goal.account_ids.includes(b.id));
  for (const b of linked) {
    const c = fx.convert(b.balance, b.currency, today);
    // Conta no vermelho é dívida, não reduz a meta (espelha `goals::sources`).
    if (c) {
      current += Math.max(0, c.amount);
      breakdown.push({
        kind: 'account',
        ref_id: b.id,
        account_id: null,
        name: b.name,
        value: Math.max(0, c.amount),
        value_used: 'balance',
        share: null,
        ignored_negative: c.amount < 0 ? c.amount : null,
        stale: false,
      });
    }
  }
  let avg = 0;
  if (linked.length > 0) {
    const firstOpen = getLedger().firstMonth;
    const months: string[] = [];
    for (let i = CONTRIBUTION_MONTHS; i >= 1; i--) {
      const m = shiftMonth(today.slice(0, 7), -i);
      if (m >= firstOpen) months.push(m);
    }
    if (months.length > 0) {
      const net = netInflow(
        linked.map((b) => b.id),
        monthBounds(months[0]!).from,
        monthBounds(months[months.length - 1]!).to,
        fx,
      );
      avg = divRound(net, months.length);
    }
  }
  const remaining = Math.max(goal.target_amount - current, 0);
  let monthlyNeeded: number | null = null;
  if (goal.target_date) {
    const left = monthsUntil(today, goal.target_date);
    monthlyNeeded = remaining === 0 ? 0 : left <= 0 ? remaining : divCeil(remaining, left);
  }
  const pace = avg > 0 ? avg : (goal.monthly_contribution ?? 0);
  let projected =
    remaining === 0
      ? today
      : pace > 0
        ? addMonthsToDate(today, Math.min(divCeil(remaining, pace), 12 * 200))
        : null;
  // Meta com rendimento: anuidade em dinheiro de hoje (espelha `goals/mod.rs`).
  let growth: GoalGrowth | undefined;
  if (goal.real_return != null) {
    const i = Math.pow(1 + goal.real_return, 1 / 12) - 1;
    const n = goal.target_date ? Math.max(monthsUntil(today, goal.target_date), 0) : 0;
    const linear = monthlyNeeded;
    if (goal.target_date) {
      monthlyNeeded =
        remaining === 0
          ? 0
          : n === 0
            ? remaining
            : annuityPayment(goal.target_amount, current, i, n);
    }
    const m = remaining === 0 ? 0 : monthsToReach(goal.target_amount, current, pace, i);
    projected = m == null ? null : addMonthsToDate(today, m);
    const inflation = 0.045;
    const contributions = monthlyNeeded != null && n > 0 ? current + monthlyNeeded * n : null;
    growth = {
      real_return: goal.real_return,
      inflation,
      months: n,
      from_contributions: contributions,
      from_returns: contributions == null ? null : Math.max(goal.target_amount - contributions, 0),
      target_nominal:
        n > 0 ? Math.round(goal.target_amount * Math.pow(1 + inflation, n / 12)) : null,
      linear_monthly_needed: linear,
    };
  }
  const onTrack = goal.target_date
    ? remaining === 0
      ? true
      : projected
        ? projected <= goal.target_date
        : false
    : null;
  return {
    ...goal,
    current_amount: current,
    progress: current / goal.target_amount,
    remaining,
    monthly_needed: monthlyNeeded,
    avg_monthly_contribution: avg,
    projected_date: projected,
    on_track: onTrack,
    missing_fx: [...fx.missing].sort(),
    estimated: fx.estimated,
    ...(growth ? { growth } : {}),
    sources: [
      ...goal.account_ids.map((id) => ({
        kind: 'account' as const,
        ref_id: id,
        account_id: null,
        value_mode: null,
        share: null,
      })),
      ...(extraSources.get(goal.id) ?? []),
    ],
    breakdown,
    stale_sources: breakdown.filter((s) => s.stale),
  };
}

function annuityPayment(target: number, current: number, i: number, n: number): number {
  const g = Math.pow(1 + i, n);
  const need = target - current * g;
  if (need <= 0) return 0;
  return Math.ceil(Math.abs(i) < 1e-12 ? need / n : (need * i) / (g - 1));
}

function monthsToReach(target: number, current: number, pace: number, i: number): number | null {
  let b = current;
  for (let m = 1; m <= 12 * 200; m++) {
    b = b * (1 + i) + pace;
    if (b >= target) return m;
    if (pace <= 0 && b <= 0) return null;
  }
  return null;
}

/** Gastos anuais: categorias com despesa em até 3 dos últimos 12 meses (espelha `living.rs`). */
function annualExpenses(fx: Fx): AnnualExpenses {
  const today = getLedger().today;
  const first = firstFlowMonth();
  const months: string[] = [];
  for (let i = 12; i >= 1; i--) {
    const m = shiftMonth(today.slice(0, 7), -i);
    if (first && m >= first) months.push(m);
  }
  const out: AnnualExpenses = {
    items: [],
    monthly: 0,
    essential_monthly: 0,
    months,
    complete_year: months.length === 12,
  };
  if (!out.complete_year) return out;
  const agg = aggregate(
    {
      from: monthBounds(months[0]!).from,
      to: monthBounds(months[11]!).to,
      basis: 'accrual',
      kind: 'expense',
    },
    KEY.category,
    KEY.month,
    fx,
  );
  const byCat = new Map<string, [string, number][]>();
  for (const [k, cell] of agg.cells) {
    const [cat, month] = k.split('|');
    if (!cat || !month || cell.expense <= 0) continue;
    byCat.set(cat, [...(byCat.get(cat) ?? []), [month, cell.expense]]);
  }
  for (const [cat, list] of byCat) {
    const info = category(cat);
    if (list.length > 3 || !info) continue;
    const total = list.reduce((a, [, v]) => a + v, 0);
    out.items.push({
      category_id: cat,
      category: info.path ?? info.name,
      essential: info.is_essential,
      total,
      months: list.map(([m]) => m).sort(),
      monthly: divRound(total, 12),
    });
  }
  out.items.sort((a, b) => b.total - a.total || a.category.localeCompare(b.category));
  out.monthly = divRound(
    out.items.reduce((a, i) => a + i.total, 0),
    12,
  );
  out.essential_monthly = divRound(
    out.items.filter((i) => i.essential).reduce((a, i) => a + i.total, 0),
    12,
  );
  return out;
}

export function goalsList(includeArchived: boolean): GoalView[] {
  return goals
    .filter((g) => includeArchived || !g.archived)
    .sort(
      (a, b) =>
        Number(a.archived) - Number(b.archived) ||
        (created.get(a.id) ?? 0) - (created.get(b.id) ?? 0),
    )
    .map(goalView);
}

export function goalGet(id: string): GoalView {
  const g = goals.find((x) => x.id === id);
  if (!g) throw new MockError('not_found', `não encontrado: meta ${id}`);
  return goalView(g);
}

function validate(g: Goal): void {
  if (!g.name.trim()) throw new MockError('invalid', 'informe o nome da meta');
  if (g.target_amount <= 0)
    throw new MockError('invalid', 'o valor-alvo da meta deve ser positivo');
  if (g.initial_amount < 0)
    throw new MockError('invalid', 'o valor já guardado não pode ser negativo');
  if (g.monthly_contribution != null && g.monthly_contribution < 0)
    throw new MockError('invalid', 'o aporte mensal planejado não pode ser negativo');
  if (g.real_return != null && (g.real_return < -0.05 || g.real_return > 0.15))
    throw new MockError('invalid', 'o retorno real esperado precisa ficar entre −5% e 15% ao ano');
  for (const id of g.account_ids) {
    if (!ACCOUNTS.some((a) => a.id === id))
      throw new MockError('not_found', `não encontrado: conta ${id}`);
  }
}

export function goalCreate(input: GoalInput): GoalView {
  const g: Goal = {
    id: `goal-${++seq}`,
    name: input.name.trim(),
    kind: input.kind,
    target_amount: input.target_amount,
    currency: input.currency ?? BASE,
    target_date: input.target_date ?? null,
    initial_amount: input.initial_amount ?? 0,
    monthly_contribution: input.monthly_contribution ?? null,
    account_ids: [...new Set(input.account_ids ?? [])].sort(),
    archived: false,
    notes: input.notes?.trim() || null,
    real_return: input.real_return ?? null,
  };
  validate(g);
  goals.push(g);
  created.set(g.id, seq);
  return goalView(g);
}

export function goalUpdate(p: GoalPatch): GoalView {
  const i = goals.findIndex((x) => x.id === p.id);
  if (i < 0) throw new MockError('not_found', `não encontrado: meta ${p.id}`);
  const g: Goal = { ...goals[i]! };
  if (p.name !== undefined) g.name = p.name.trim();
  if (p.kind !== undefined) g.kind = p.kind;
  if (p.target_amount !== undefined) g.target_amount = p.target_amount;
  if (p.target_date !== undefined) g.target_date = p.target_date;
  if (p.initial_amount !== undefined) g.initial_amount = p.initial_amount;
  if (p.monthly_contribution !== undefined) g.monthly_contribution = p.monthly_contribution;
  if (p.archived !== undefined) g.archived = p.archived;
  if (p.notes !== undefined) g.notes = p.notes?.trim() || null;
  if (p.account_ids !== undefined) g.account_ids = [...new Set(p.account_ids)].sort();
  if (p.real_return !== undefined) g.real_return = p.real_return;
  validate(g);
  goals[i] = g;
  return goalView(g);
}

export function goalDelete(id: string): void {
  const i = goals.findIndex((x) => x.id === id);
  if (i < 0) throw new MockError('not_found', `não encontrado: meta ${id}`);
  goals.splice(i, 1);
}

export function emergencyReserve(
  targetMonths = 6,
  monthsBack = 6,
  includeAnnual: boolean | null = null,
): EmergencyReserve {
  const tm = Math.min(Math.max(targetMonths, 1), 120);
  const mb = Math.min(Math.max(monthsBack, 1), 120);
  const today = getLedger().today;
  const first = firstFlowMonth();
  const months: string[] = [];
  if (first) {
    for (let i = mb; i >= 1; i--) {
      const m = shiftMonth(today.slice(0, 7), -i);
      if (m >= first) months.push(m);
    }
  }
  const fx = new Fx();
  const annual = annualExpenses(fx);
  const annualIds = new Set(annual.items.filter((i) => i.essential).map((i) => i.category_id));
  let essential = 0;
  let essentialAnnual = 0;
  let totalExpense = 0;
  if (months.length > 0) {
    const agg = aggregate(
      {
        from: monthBounds(months[0]!).from,
        to: monthBounds(months[months.length - 1]!).to,
        basis: 'accrual',
        kind: 'expense',
      },
      KEY.category,
      KEY.none,
      fx,
    );
    for (const [k, cell] of agg.cells) {
      totalExpense += cell.expense;
      const id = k.split('|')[0];
      if (category(id)?.is_essential) {
        essential += cell.expense;
        if (id && annualIds.has(id)) essentialAnnual += cell.expense;
      }
    }
  }
  const n = months.length;
  const avgEssential = n > 0 ? divRound(essential, n) : 0;
  const avgTotal = n > 0 ? divRound(totalExpense, n) : 0;
  const avgRegular = n > 0 ? divRound(essential - essentialAnnual, n) : 0;
  const include = (includeAnnual ?? annual.essential_monthly > 0) && annual.complete_year;
  const monthlyCost = include ? avgRegular + annual.essential_monthly : avgEssential;
  const goal = goals.find((g) => g.kind === 'emergency' && !g.archived);
  let current = 0;
  if (goal) {
    const v = goalView(goal);
    current = v.current_amount;
  } else {
    for (const b of balancesAt(today)) {
      const acc = ACCOUNTS.find((a) => a.id === b.id)!;
      if (!acc.include_in_net_worth || !['checking', 'savings', 'cash'].includes(acc.kind))
        continue;
      const c = fx.convert(b.balance, b.currency, today);
      if (c) current += c.amount;
    }
  }
  return {
    avg_essential_monthly: avgEssential,
    avg_essential_regular_monthly: avgRegular,
    monthly_cost: monthlyCost,
    annual,
    include_annual: include,
    avg_total_expense_monthly: avgTotal,
    months_analyzed: n,
    target_months: tm,
    target_amount: monthlyCost * tm,
    current_amount: current,
    coverage_months: monthlyCost > 0 ? current / monthlyCost : null,
    goal_id: goal?.id ?? null,
    currency: BASE,
    months,
    basis: 'accrual',
    missing_fx: [...fx.missing].sort(),
    estimated: fx.estimated,
  };
}
