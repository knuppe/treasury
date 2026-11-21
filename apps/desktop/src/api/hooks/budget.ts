/** Consultas e mutações do orçamento (`budget.*`). */
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import { useCoreMutation } from '@/api/hooks/coreQuery';
import type { BudgetGetArgs } from '@/api/generated/BudgetGetArgs';
import type { BudgetLineInput } from '@/api/generated/BudgetLineInput';
import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { CopyBudgetArgs } from '@/api/generated/CopyBudgetArgs';
import type { GroupTargets } from '@/api/generated/GroupTargets';
import type { SetBudgetArgs } from '@/api/generated/SetBudgetArgs';
import type { SuggestBudgetArgs } from '@/api/generated/SuggestBudgetArgs';

export const budgetKeys = {
  all: ['budget'] as const,
  get: (args: BudgetGetArgs) => ['budget', 'get', args] as const,
  suggest: (args: SuggestBudgetArgs) => ['budget', 'suggest', args] as const,
  months: ['budget', 'list_months', {}] as const,
  groupTargets: ['budget', 'group_targets', {}] as const,
};

export function useBudget(args: BudgetGetArgs) {
  return useQuery({
    queryKey: budgetKeys.get(args),
    queryFn: () => call<BudgetStatus>('budget.get', args),
    placeholderData: keepPreviousData,
  });
}

export function useBudgetMonths() {
  return useQuery({
    queryKey: budgetKeys.months,
    queryFn: () => call<string[]>('budget.list_months', {}),
  });
}

/** Sugestão pela média (só quando pedida). */
export function useBudgetSuggestion(args: SuggestBudgetArgs, enabled: boolean) {
  return useQuery({
    queryKey: budgetKeys.suggest(args),
    queryFn: () => call<BudgetLineInput[]>('budget.suggest', args),
    enabled,
  });
}

/** Depois de gravar: orçamento, painel (previsto do mês) e relatórios de orçamento. */
function useInvalidateBudget() {
  const client = useQueryClient();
  return () =>
    Promise.all([
      client.invalidateQueries({ queryKey: budgetKeys.all }),
      client.invalidateQueries({ queryKey: ['reports', 'dashboard'] }),
    ]);
}

export function useSetBudgetLines() {
  const invalidate = useInvalidateBudget();
  return useMutation({
    mutationFn: (args: SetBudgetArgs) => call<BudgetStatus>('budget.set_lines', args),
    onSuccess: invalidate,
  });
}

export function useCopyBudget() {
  const invalidate = useInvalidateBudget();
  return useMutation({
    mutationFn: (args: CopyBudgetArgs) => call<BudgetStatus>('budget.copy', args),
    onSuccess: invalidate,
  });
}

/** Metas por grupo de gasto (fração da renda; `null` = sem meta). */
export function useGroupTargets() {
  return useQuery({
    queryKey: budgetKeys.groupTargets,
    queryFn: () => call<GroupTargets>('budget.group_targets', {}),
  });
}

/** Grava as metas por grupo: relê o orçamento, os relatórios (grupos) e o histórico. */
export const useSetGroupTargets = () =>
  useCoreMutation<GroupTargets, GroupTargets>('budget.set_group_targets', {
    affects: ['settings'],
  });
