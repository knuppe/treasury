/**
 * Consultas (TanStack Query) dos relatórios e das leituras de outros módulos que as telas
 * de Painel, Orçamento e Relatórios usam. Chaves no formato `[modulo, acao, args]`, as
 * mesmas das outras telas, para o cache ser compartilhado.
 */
import { keepPreviousData, useMutation, useQueries, useQuery } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { AccountView } from '@/api/generated/AccountView';
import type { CalendarData } from '@/api/generated/CalendarData';
import type { Category } from '@/api/generated/Category';
import type { CategoryListArgs } from '@/api/generated/CategoryListArgs';
import type { CompareArgs } from '@/api/generated/CompareArgs';
import type { Dashboard } from '@/api/generated/Dashboard';
import type { DashboardArgs } from '@/api/generated/DashboardArgs';
import type { ExplainArgs } from '@/api/generated/ExplainArgs';
import type { Explanation } from '@/api/generated/Explanation';
import type { FutureCommitmentsArgs } from '@/api/generated/FutureCommitmentsArgs';
import type { InflationArgs } from '@/api/generated/InflationArgs';
import type { InflationReport } from '@/api/generated/InflationReport';
import type { Member } from '@/api/generated/Member';
import type { MemberBreakdown } from '@/api/generated/MemberBreakdown';
import type { MonthAmount } from '@/api/generated/MonthAmount';
import type { MonthFlow } from '@/api/generated/MonthFlow';
import type { MonthFlowArgs } from '@/api/generated/MonthFlowArgs';
import type { MonthlyEvolution } from '@/api/generated/MonthlyEvolution';
import type { MonthlyEvolutionArgs } from '@/api/generated/MonthlyEvolutionArgs';
import type { NetWorth } from '@/api/generated/NetWorth';
import type { NetWorthArgs } from '@/api/generated/NetWorthArgs';
import type { NetWorthHistoryArgs } from '@/api/generated/NetWorthHistoryArgs';
import type { NetWorthPoint } from '@/api/generated/NetWorthPoint';
import type { PdfArgs } from '@/api/generated/PdfArgs';
import type { PdfResult } from '@/api/generated/PdfResult';
import type { PeriodArgs } from '@/api/generated/PeriodArgs';
import type { PeriodComparison } from '@/api/generated/PeriodComparison';
import type { PortfolioArgs } from '@/api/generated/PortfolioArgs';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { RecurrenceListArgs } from '@/api/generated/RecurrenceListArgs';
import type { SankeyArgs } from '@/api/generated/SankeyArgs';
import type { SankeyData } from '@/api/generated/SankeyData';
import type { SankeyItems } from '@/api/generated/SankeyItems';
import type { SankeyItemsArgs } from '@/api/generated/SankeyItemsArgs';
import type { SpendingArgs } from '@/api/generated/SpendingArgs';
import type { SpendingSummary } from '@/api/generated/SpendingSummary';
import type { SpendGroupsArgs } from '@/api/generated/SpendGroupsArgs';
import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import type { UpcomingArgs } from '@/api/generated/UpcomingArgs';
import type { UpcomingItem } from '@/api/generated/UpcomingItem';

/** Consulta simples `[modulo, acao, args]` → `call('modulo.acao', args)`. */
function useCommand<T>(
  module: string,
  action: string,
  args: object,
  options: { enabled?: boolean; keepPrevious?: boolean } = {},
) {
  return useQuery({
    queryKey: [module, action, args],
    queryFn: () => call<T>(`${module}.${action}`, args),
    enabled: options.enabled ?? true,
    placeholderData: options.keepPrevious === false ? undefined : keepPreviousData,
  });
}

export const reportKeys = {
  all: ['reports'] as const,
};

export const useDashboard = (args: DashboardArgs) =>
  useCommand<Dashboard>('reports', 'dashboard', args);
export const useNetWorth = (args: NetWorthArgs) =>
  useCommand<NetWorth>('reports', 'net_worth', args);
export const useNetWorthHistory = (args: NetWorthHistoryArgs) =>
  useCommand<NetWorthPoint[]>('reports', 'net_worth_history', args);
export const useMonthFlow = (args: MonthFlowArgs, enabled = true) =>
  useCommand<MonthFlow[]>('reports', 'month_flow', args, { enabled });
export const useSpending = (args: SpendingArgs, enabled = true) =>
  useCommand<SpendingSummary>('reports', 'spending', args, { enabled });
export const useSankey = (args: SankeyArgs) => useCommand<SankeyData>('reports', 'sankey', args);
/** Para onde vai a renda: os quatro grupos de gasto por mês e no período, com as metas. */
export const useSpendGroups = (args: SpendGroupsArgs, enabled = true) =>
  useCommand<SpendGroupsReport>('reports', 'spend_groups', args, { enabled });
/** Lançamentos que compõem uma linha do fluxo nos dois períodos (`null` = fechado). */
export const useSankeyItems = (args: SankeyItemsArgs | null) =>
  useCommand<SankeyItems>('reports', 'sankey_items', args ?? {}, {
    enabled: args !== null,
    keepPrevious: false,
  });
export const useCalendar = (args: PeriodArgs) =>
  useCommand<CalendarData>('reports', 'calendar', args);
export const useMonthlyEvolution = (args: MonthlyEvolutionArgs) =>
  useCommand<MonthlyEvolution>('reports', 'monthly_evolution', args);
export const useCompare = (args: CompareArgs) =>
  useCommand<PeriodComparison>('reports', 'compare', args);
export const useByMember = (args: PeriodArgs) =>
  useCommand<MemberBreakdown>('reports', 'by_member', args);
export const useUpcoming = (args: UpcomingArgs) =>
  useCommand<UpcomingItem[]>('reports', 'upcoming', args);
export const useExplain = (args: ExplainArgs | null) =>
  useCommand<Explanation>('reports', 'explain', args ?? {}, {
    enabled: args !== null,
    keepPrevious: false,
  });
/** Figura 07 · Inflação: gasto nominal e corrigido pelo índice do país. */
export const useInflation = (args: InflationArgs) =>
  useCommand<InflationReport>('reports', 'inflation', args);

/** Relatório do mês em PDF, gravado no caminho escolhido (feito no aparelho). */
export const useReportPdf = () =>
  useMutation({ mutationFn: (args: PdfArgs) => call<PdfResult>('reports.pdf', args) });

// ── Leituras de outros módulos (mesmas chaves das telas donas) ──

export const usePortfolioSummary = (args: PortfolioArgs = {}) =>
  useCommand<PortfolioSummary>('portfolio', 'summary', args);
export const useRecurrences = (args: RecurrenceListArgs = {}) =>
  useCommand<Recurrence[]>('recurrences', 'list', args);
export const useCategories = (args: CategoryListArgs = {}) =>
  useCommand<Category[]>('categories', 'list', args);
export const useAccounts = () => useCommand<AccountView[]>('accounts', 'list', {});
export const useMembers = () => useCommand<Member[]>('household', 'list_members', {});

/** Faturas futuras de cada cartão (uma consulta por cartão). */
export function useFutureCommitmentsByCard(cardIds: readonly string[], months: number) {
  return useQueries({
    queries: cardIds.map((id) => {
      const args: FutureCommitmentsArgs = { account_id: id, months };
      return {
        queryKey: ['cards', 'future_commitments', args],
        queryFn: () => call<MonthAmount[]>('cards.future_commitments', args),
      };
    }),
  });
}
