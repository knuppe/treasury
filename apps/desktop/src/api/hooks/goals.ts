/** Consultas e mutações das metas (`goals.*`). */
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { ObjectiveAllocation } from '@/api/generated/ObjectiveAllocation';
import type { EducationInflation } from '@/api/generated/EducationInflation';
import type { EducationOverview } from '@/api/generated/EducationOverview';
import type { EducationPlanView } from '@/api/generated/EducationPlanView';
import type { EducationResult } from '@/api/generated/EducationResult';
import type { EducationSaveArgs } from '@/api/generated/EducationSaveArgs';
import type { EducationSimulateArgs } from '@/api/generated/EducationSimulateArgs';
import type { EmergencyReserve } from '@/api/generated/EmergencyReserve';
import type { EmergencyReserveArgs } from '@/api/generated/EmergencyReserveArgs';
import type { AssetLabel } from '@/api/generated/AssetLabel';
import type { GoalInput } from '@/api/generated/GoalInput';
import type { GoalLinkArgs } from '@/api/generated/GoalLinkArgs';
import type { GoalLiquidityCandidate } from '@/api/generated/GoalLiquidityCandidate';
import type { GoalUnlinkArgs } from '@/api/generated/GoalUnlinkArgs';
import type { GoalListArgs } from '@/api/generated/GoalListArgs';
import type { GoalPatch } from '@/api/generated/GoalPatch';
import type { GoalView } from '@/api/generated/GoalView';
import type { HoldingsSetArgs } from '@/api/generated/HoldingsSetArgs';
import type { HoldingView } from '@/api/generated/HoldingView';
import type { RetirementDefaults } from '@/api/generated/RetirementDefaults';
import type { RetirementPlan } from '@/api/generated/RetirementPlan';
import type { RetirementScenario } from '@/api/generated/RetirementScenario';
import type { RetirementScenarioInput } from '@/api/generated/RetirementScenarioInput';
import type { RetirementScenarioView } from '@/api/generated/RetirementScenarioView';
import type { RetirementSimulation } from '@/api/generated/RetirementSimulation';

export const goalKeys = {
  all: ['goals'] as const,
  list: (args: GoalListArgs) => ['goals', 'list', args] as const,
  reserve: (args: EmergencyReserveArgs) => ['goals', 'emergency_reserve', args] as const,
};

export function useGoals(args: GoalListArgs = {}) {
  return useQuery({
    queryKey: goalKeys.list(args),
    queryFn: () => call<GoalView[]>('goals.list', args),
  });
}

export function useEmergencyReserve(args: EmergencyReserveArgs) {
  return useQuery({
    queryKey: goalKeys.reserve(args),
    queryFn: () => call<EmergencyReserve>('goals.emergency_reserve', args),
    placeholderData: keepPreviousData,
  });
}

/** Simulação de aposentadoria: o plano salvo com `plan` por cima (sem gravar). */
export function useRetirement(plan: RetirementPlan) {
  return useQuery({
    queryKey: ['goals', 'retirement', plan] as const,
    queryFn: () => call<RetirementSimulation>('goals.retirement', plan),
    placeholderData: keepPreviousData,
    retry: false,
  });
}

/**
 * Padrões do planejador a partir do membro (padrão: o titular): a idade de hoje pela data de
 * nascimento e, com a linha da vida ligada, o horizonte "1 em 10" (docs/life-expectancy.md).
 */
export function useRetirementDefaults(memberId?: string | null) {
  const args = memberId ? { member_id: memberId } : {};
  return useQuery({
    queryKey: ['goals', 'retirement_defaults', args] as const,
    queryFn: () => call<RetirementDefaults>('goals.retirement_defaults', args),
    retry: false,
  });
}

export function useSaveRetirement() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (plan: RetirementPlan) => call<RetirementSimulation>('goals.retirement_save', plan),
    onSuccess: invalidate,
  });
}

/** Cenários de aposentadoria salvos, com o resumo de cada um. */
export function useRetirementScenarios() {
  return useQuery({
    queryKey: ['goals', 'retirement_scenarios'] as const,
    queryFn: () => call<RetirementScenarioView[]>('goals.retirement_scenarios', {}),
  });
}

export function useSaveRetirementScenario() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (input: RetirementScenarioInput) =>
      call<RetirementScenario>('goals.retirement_scenario_save', input),
    onSuccess: invalidate,
  });
}

export function useDeleteRetirementScenario() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (id: string) => call<null>('goals.retirement_scenario_delete', { id }),
    onSuccess: invalidate,
  });
}

/** Cria (sem `goal_id`) ou liga uma meta ao plano principal. */
export function useLinkRetirementGoal() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (goalId?: string) =>
      call<GoalView>('goals.retirement_link_goal', goalId ? { goal_id: goalId } : {}),
    onSuccess: invalidate,
  });
}

export function useUnlinkRetirementGoal() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: () => call<null>('goals.retirement_unlink_goal', {}),
    onSuccess: invalidate,
  });
}

/** Metas e o plano também são marcos da linha da vida. */
function useInvalidateGoals() {
  const client = useQueryClient();
  return () =>
    Promise.all([
      client.invalidateQueries({ queryKey: goalKeys.all }),
      client.invalidateQueries({ queryKey: ['longevity'] }),
    ]);
}

export function useCreateGoal() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (input: GoalInput) => call<GoalView>('goals.create', input),
    onSuccess: invalidate,
  });
}

export function useUpdateGoal() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (patch: GoalPatch) => call<GoalView>('goals.update', patch),
    onSuccess: invalidate,
  });
}

export function useDeleteGoal() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (id: string) => call<null>('goals.delete', { id }),
    onSuccess: invalidate,
  });
}

// ── Meta de faculdade (docs/design/college-goal.md) ────────────

export function useEducationPlan(goalId: string | null) {
  return useQuery({
    queryKey: ['goals', 'education', 'plan', goalId] as const,
    queryFn: () => call<EducationPlanView>('goals.education_get', { goal_id: goalId }),
    enabled: goalId != null,
  });
}

/** Conta do plano sem gravar (o assistente mostra o resultado a cada passo). */
export function useEducationSimulate(args: EducationSimulateArgs | null) {
  return useQuery({
    queryKey: ['goals', 'education', 'simulate', args] as const,
    queryFn: () => call<EducationResult>('goals.education_simulate', args),
    enabled: args != null,
    placeholderData: keepPreviousData,
    retry: false,
  });
}

export function useEducationInflation() {
  return useQuery({
    queryKey: ['goals', 'education', 'inflation'] as const,
    queryFn: () => call<EducationInflation>('goals.education_inflation', {}),
  });
}

export function useEducationOverview() {
  return useQuery({
    queryKey: ['goals', 'education', 'overview'] as const,
    queryFn: () => call<EducationOverview>('goals.education_overview', {}),
  });
}

export function useSaveEducation() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (args: EducationSaveArgs) => call<EducationPlanView>('goals.education_save', args),
    onSuccess: invalidate,
  });
}

/** Classificação "Objetivo": a carteira dividida pelas metas (pesos das posições ligadas). */
export function useObjectives() {
  return useQuery({
    queryKey: ['goals', 'objectives'] as const,
    queryFn: () => call<ObjectiveAllocation>('goals.objectives', {}),
  });
}

export function useGoalHoldings(goalId: string | null) {
  return useQuery({
    queryKey: ['goals', 'holdings', goalId] as const,
    queryFn: () => call<HoldingView[]>('goals.holdings_get', { goal_id: goalId }),
    enabled: goalId != null,
  });
}

export function useSetGoalHoldings() {
  const invalidate = useInvalidateGoals();
  return useMutation({
    mutationFn: (args: HoldingsSetArgs) => call<HoldingView[]>('goals.holdings_set', args),
    onSuccess: invalidate,
  });
}

/** Liga uma fonte à meta: conta, ativo ou etiqueta (docs/goal-sources.md). */
export function useLinkGoalSource() {
  const invalidate = useInvalidateGoals();
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: GoalLinkArgs) => call<GoalView>('goals.link_source', args),
    // A reserva entra na saúde financeira.
    onSuccess: () =>
      Promise.all([invalidate(), client.invalidateQueries({ queryKey: ['health'] })]),
  });
}

export function useUnlinkGoalSource() {
  const invalidate = useInvalidateGoals();
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: GoalUnlinkArgs) => call<GoalView>('goals.unlink_source', args),
    // A reserva entra na saúde financeira.
    onSuccess: () =>
      Promise.all([invalidate(), client.invalidateQueries({ queryKey: ['health'] })]),
  });
}

/** Investimentos de liquidez diária que ainda não estão na meta. */
export function useLiquidityCandidates(goalId: string, enabled: boolean) {
  return useQuery({
    queryKey: ['goals', 'liquidity', goalId] as const,
    queryFn: () =>
      call<GoalLiquidityCandidate[]>('goals.liquidity_candidates', { goal_id: goalId }),
    enabled,
  });
}

/** Etiquetas de ativos da família (Ativos › etiquetas). */
export function useAssetLabels() {
  return useQuery({
    queryKey: ['assets', 'labels_list'] as const,
    queryFn: () => call<AssetLabel[]>('assets.labels_list', {}),
  });
}
