/**
 * Painel: junta `reports.dashboard` (que usa a base padrão do cofre) com as consultas na
 * base escolhida pelo usuário quando ela é outra (o comando do painel não recebe `basis`).
 * Funções puras.
 */
import type { Basis } from '@/api/generated/Basis';
import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { Dashboard } from '@/api/generated/Dashboard';
import type { GroupAmount } from '@/api/generated/GroupAmount';
import type { MonthFlow } from '@/api/generated/MonthFlow';
import type { SpendingSummary } from '@/api/generated/SpendingSummary';

/** Taxa de poupança de vários meses: (Σ receitas − Σ despesas) ÷ Σ receitas. */
export function savingsRate(history: readonly MonthFlow[]): number | null {
  const income = history.reduce((s, f) => s + f.income, 0);
  const expense = history.reduce((s, f) => s + f.expense, 0);
  return income > 0 ? (income - expense) / income : null;
}

export interface DashboardView {
  month: string;
  basis: Basis;
  /** Base padrão do cofre (a do `reports.dashboard`). */
  defaultBasis: Basis;
  dashboard: Dashboard;
  flow: MonthFlow;
  flowHistory: MonthFlow[];
  savingsRate12m: number | null;
  topCategories: GroupAmount[];
  /** Orçamento na base escolhida (com ritmo e comprometido), se já carregou. */
  budget: BudgetStatus | undefined;
}

/**
 * Painel na base `basis`. Se a base for a padrão, usa tudo do `dashboard`; senão, troca
 * fluxo, histórico e principais categorias pelas consultas na outra base (quando vierem).
 */
export function composeDashboard(
  dashboard: Dashboard,
  basis: Basis,
  overrides: { flowHistory?: MonthFlow[]; spending?: SpendingSummary; budget?: BudgetStatus },
): DashboardView {
  const useOverride = basis !== dashboard.basis;
  const history =
    useOverride && overrides.flowHistory?.length ? overrides.flowHistory : dashboard.flow_history;
  const top =
    useOverride && overrides.spending
      ? overrides.spending.groups.filter((g) => g.amount > 0).slice(0, 5)
      : dashboard.top_categories;
  return {
    month: dashboard.month,
    basis: useOverride && overrides.flowHistory ? basis : dashboard.basis,
    defaultBasis: dashboard.basis,
    dashboard,
    flow: history[history.length - 1] ?? dashboard.flow,
    flowHistory: history,
    savingsRate12m:
      useOverride && overrides.flowHistory ? savingsRate(history) : dashboard.savings_rate_12m,
    topCategories: top,
    budget: overrides.budget,
  };
}

/** Situação do orçamento do mês para o card: gasto, previsto, ritmo e estado. */
export function budgetPace(b: BudgetStatus): {
  planned: number;
  actual: number;
  committed: number;
  elapsed: number;
  /** Gasto esperado até hoje no ritmo linear. */
  expectedByNow: number;
  ratio: number | null;
} {
  const planned = b.totals.planned_expense;
  return {
    planned,
    actual: b.totals.actual_expense,
    committed: b.totals.committed_expense,
    elapsed: b.month_elapsed,
    expectedByNow: Math.round(planned * b.month_elapsed),
    ratio: planned > 0 ? b.totals.actual_expense / planned : null,
  };
}
