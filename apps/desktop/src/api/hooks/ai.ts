/**
 * Consultas e mutações do agente (`ai.*`) e da auditoria do MCP (`mcp.audit_list`).
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call, type ApiError } from '@/api/client';
import type { AiSettings } from '@/api/generated/AiSettings';
import type { AiSettingsPatch } from '@/api/generated/AiSettingsPatch';
import type { Alert } from '@/api/generated/Alert';
import type { AlertCounts } from '@/api/generated/AlertCounts';
import type { AlertListArgs } from '@/api/generated/AlertListArgs';
import type { AlertStatusArgs } from '@/api/generated/AlertStatusArgs';
import type { AuditEntry } from '@/api/generated/AuditEntry';
import type { AuditListArgs } from '@/api/generated/AuditListArgs';
import type { CliDetection } from '@/api/generated/CliDetection';
import type { Conversation } from '@/api/generated/Conversation';
import type { ConversationDetail } from '@/api/generated/ConversationDetail';
import type { ConversationExportArgs } from '@/api/generated/ConversationExportArgs';
import type { ConversationExportResult } from '@/api/generated/ConversationExportResult';
import type { Engine } from '@/api/generated/Engine';
import type { EngineInput } from '@/api/generated/EngineInput';
import type { EnginePatch } from '@/api/generated/EnginePatch';
import type { EngineTest } from '@/api/generated/EngineTest';
import type { JobInfo } from '@/api/generated/JobInfo';
import type { CliUserMcp } from '@/api/generated/CliUserMcp';
import type { McpServer } from '@/api/generated/McpServer';
import type { McpServerInput } from '@/api/generated/McpServerInput';
import type { McpTestResult } from '@/api/generated/McpTestResult';
import type { MemoryItem } from '@/api/generated/MemoryItem';
import type { ModelPrice } from '@/api/generated/ModelPrice';
import type { PriceKey } from '@/api/generated/PriceKey';
import type { Procedure } from '@/api/generated/Procedure';
import type { ProcedureListArgs } from '@/api/generated/ProcedureListArgs';
import type { ProcedureSave } from '@/api/generated/ProcedureSave';
import type { ProcedureSaved } from '@/api/generated/ProcedureSaved';
import type { Proposal } from '@/api/generated/Proposal';
import type { ProposalDetail } from '@/api/generated/ProposalDetail';
import type { ProposalListArgs } from '@/api/generated/ProposalListArgs';
import type { ProviderInfo } from '@/api/generated/ProviderInfo';
import type { RememberArgs } from '@/api/generated/RememberArgs';
import type { ReviewArgs } from '@/api/generated/ReviewArgs';
import type { ReviewResult } from '@/api/generated/ReviewResult';
import type { Routine } from '@/api/generated/Routine';
import type { RoutineStatus } from '@/api/generated/RoutineStatus';
import type { SecretStatus } from '@/api/generated/SecretStatus';
import type { SetLimitArgs } from '@/api/generated/SetLimitArgs';
import type { SetPriceArgs } from '@/api/generated/SetPriceArgs';
import type { SetRouteArgs } from '@/api/generated/SetRouteArgs';
import type { SpendLimit } from '@/api/generated/SpendLimit';
import type { Suggestion } from '@/api/generated/Suggestion';
import type { TaskRoute } from '@/api/generated/TaskRoute';
import type { UsageSummary } from '@/api/generated/UsageSummary';
import type { UsageSummaryArgs } from '@/api/generated/UsageSummaryArgs';

export const aiKeys = {
  all: ['ai'] as const,
  proposals: (args: ProposalListArgs) => ['ai', 'proposals', args] as const,
  proposal: (id: string) => ['ai', 'proposal', id] as const,
  pendingCount: ['ai', 'pending_count'] as const,
  memory: ['ai', 'memory'] as const,
  procedures: (args: ProcedureListArgs) => ['ai', 'procedures', args] as const,
  providers: ['ai', 'providers'] as const,
  engines: ['ai', 'engines'] as const,
  clis: ['ai', 'detect_clis'] as const,
  routes: ['ai', 'routes'] as const,
  limits: ['ai', 'limits'] as const,
  prices: ['ai', 'prices'] as const,
  usage: (args: UsageSummaryArgs) => ['ai', 'usage', args] as const,
  settings: ['ai', 'settings'] as const,
  conversations: ['ai', 'conversations'] as const,
  conversation: (id: string) => ['ai', 'conversation', id] as const,
  jobs: ['ai', 'jobs'] as const,
  alerts: (args: AlertListArgs) => ['ai', 'alerts', args] as const,
  alertCount: ['ai', 'alerts_count'] as const,
  routines: ['ai', 'routines'] as const,
  suggestions: (screen: string) => ['ai', 'suggestions', screen] as const,
  mcpExtra: ['ai', 'mcp_extra'] as const,
  cliMcps: ['ai', 'cli_mcps'] as const,
};

export const mcpKeys = {
  audit: (args: AuditListArgs) => ['mcp', 'audit', args] as const,
};

function useInvalidate(...keys: readonly (readonly unknown[])[]) {
  const client = useQueryClient();
  return () => Promise.all(keys.map((queryKey) => client.invalidateQueries({ queryKey })));
}

// --- Sugestões do companheiro (determinísticas, sem motor) ----------------------------

export function useSuggestions(screen: string | null, enabled = true) {
  return useQuery({
    queryKey: aiKeys.suggestions(screen ?? ''),
    queryFn: () => call<Suggestion[]>('ai.suggestions', { screen, limit: 6 }),
    enabled,
    staleTime: 30_000,
  });
}

// --- Fila de aprovação ------------------------------------------------------------------

export function useProposals(args: ProposalListArgs) {
  return useQuery({
    queryKey: aiKeys.proposals(args),
    queryFn: () => call<Proposal[]>('ai.list_proposals', args),
  });
}

export function useProposal(id: string | null) {
  return useQuery({
    queryKey: aiKeys.proposal(id ?? ''),
    queryFn: () => call<ProposalDetail>('ai.get_proposal', { id }),
    enabled: id != null,
  });
}

/** Contador da barra lateral: propostas pendentes. */
export function usePendingCount() {
  return useQuery({
    queryKey: aiKeys.pendingCount,
    queryFn: () => call<number>('ai.pending_count'),
    refetchInterval: 60_000,
  });
}

function useReview(cmd: 'ai.approve' | 'ai.reject') {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: ReviewArgs) => call<ReviewResult[]>(cmd, args),
    // Aprovar aplica escritas em qualquer módulo: tudo pode ter mudado.
    onSuccess: () => client.invalidateQueries(),
  });
}

export const useApprove = () => useReview('ai.approve');
export const useReject = () => useReview('ai.reject');

/** Desfaz o conjunto de alterações de uma proposta aprovada. */
export function useUndoChangeSet() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => call<unknown>('changes.undo', { id }),
    onSuccess: () => client.invalidateQueries(),
  });
}

// --- Memória do gestor ------------------------------------------------------------------

export function useMemory() {
  return useQuery({ queryKey: aiKeys.memory, queryFn: () => call<MemoryItem[]>('ai.memory') });
}

export function useRemember() {
  const invalidate = useInvalidate(aiKeys.memory);
  return useMutation({
    mutationFn: (args: RememberArgs) => call<{ id: string }>('ai.remember', args),
    onSuccess: invalidate,
  });
}

export function useForget() {
  const invalidate = useInvalidate(aiKeys.memory);
  return useMutation({
    mutationFn: (id: string) => call<null>('ai.forget', { id }),
    onSuccess: invalidate,
  });
}

// --- Memória de procedimentos do agente -------------------------------------------------

export function useProcedures(args: ProcedureListArgs = {}) {
  return useQuery({
    queryKey: aiKeys.procedures(args),
    queryFn: () => call<Procedure[]>('ai.procedures.list', args),
    // A lista anterior fica na tela enquanto a busca nova chega.
    placeholderData: (previous) => previous,
  });
}

export function useSaveProcedure() {
  const invalidate = useInvalidate(['ai', 'procedures']);
  return useMutation({
    mutationFn: (args: ProcedureSave) => call<ProcedureSaved>('ai.procedures.save', args),
    onSuccess: invalidate,
  });
}

export function useDeleteProcedure() {
  const invalidate = useInvalidate(['ai', 'procedures']);
  return useMutation({
    mutationFn: (id: string) => call<null>('ai.procedures.delete', { id }),
    onSuccess: invalidate,
  });
}

// --- Motores, chaves, CLIs -------------------------------------------------------------

export function useProviders() {
  return useQuery({
    queryKey: aiKeys.providers,
    queryFn: () => call<ProviderInfo[]>('ai.engines.providers'),
    staleTime: Infinity,
  });
}

export function useEngines() {
  return useQuery({ queryKey: aiKeys.engines, queryFn: () => call<Engine[]>('ai.engines.list') });
}

export function useDetectClis(enabled: boolean) {
  return useQuery({
    queryKey: aiKeys.clis,
    queryFn: () => call<CliDetection[]>('ai.detect_clis'),
    enabled,
    staleTime: 60_000,
  });
}

export function useCreateEngine() {
  const invalidate = useInvalidate(aiKeys.engines, aiKeys.routes);
  return useMutation({
    mutationFn: (input: EngineInput) => call<Engine>('ai.engines.create', input),
    onSuccess: invalidate,
  });
}

export function useUpdateEngine() {
  const invalidate = useInvalidate(aiKeys.engines);
  return useMutation({
    mutationFn: (patch: EnginePatch) => call<Engine>('ai.engines.update', patch),
    onSuccess: invalidate,
  });
}

export function useDeleteEngine() {
  const invalidate = useInvalidate(aiKeys.engines, aiKeys.routes);
  return useMutation({
    mutationFn: (id: string) => call<null>('ai.engines.delete', { id }),
    onSuccess: invalidate,
  });
}

export function useSetSecret() {
  const invalidate = useInvalidate(aiKeys.engines);
  return useMutation({
    mutationFn: (args: { engine_id: string; value: string }) =>
      call<SecretStatus>('ai.set_secret', args),
    // A chave não fica no cache de mutações depois que o diálogo fecha.
    gcTime: 0,
    onSuccess: invalidate,
  });
}

export function useDeleteSecret() {
  const invalidate = useInvalidate(aiKeys.engines);
  return useMutation({
    mutationFn: (engineId: string) =>
      call<SecretStatus>('ai.delete_secret', { engine_id: engineId }),
    onSuccess: invalidate,
  });
}

export function useTestEngine() {
  const invalidate = useInvalidate(['ai', 'usage']);
  return useMutation({
    mutationFn: (engineId: string) => call<EngineTest>('ai.test_engine', { engine_id: engineId }),
    onSuccess: invalidate,
  });
}

// --- MCPs extras deste cofre -----------------------------------------------------------

/** Servidores MCP extras do cofre (os segredos nunca vêm: só `set` e `hint`). */
export function useMcpExtras() {
  return useQuery({
    queryKey: aiKeys.mcpExtra,
    queryFn: () => call<McpServer[]>('ai.mcp_extra.list'),
  });
}

export function useSaveMcpExtra() {
  const invalidate = useInvalidate(aiKeys.mcpExtra);
  return useMutation({
    mutationFn: (input: McpServerInput) => call<McpServer>('ai.mcp_extra.save', input),
    // Os valores secretos não ficam no cache de mutações depois que o diálogo fecha.
    gcTime: 0,
    onSuccess: invalidate,
  });
}

export function useDeleteMcpExtra() {
  const invalidate = useInvalidate(aiKeys.mcpExtra);
  return useMutation({
    mutationFn: (id: string) => call<null>('ai.mcp_extra.delete', { id }),
    onSuccess: invalidate,
  });
}

/** Conecta no servidor e lista as ferramentas (pode levar até ~20 s). */
export function useTestMcpExtra() {
  return useMutation({
    mutationFn: (id: string) => call<McpTestResult>('ai.mcp_extra.test', { id }),
  });
}

/** MCPs que as CLIs instaladas carregam por conta própria (Claude, Cursor, Grok…), com os
 * liberados para o agente deste cofre (padrão: nenhum). */
export function useCliMcps() {
  return useQuery({
    queryKey: aiKeys.cliMcps,
    queryFn: () => call<CliUserMcp[]>('ai.cli_mcps.list'),
  });
}

export function useSetCliMcps() {
  const invalidate = useInvalidate(aiKeys.cliMcps);
  return useMutation({
    mutationFn: (allowed: string[]) => call<string[]>('ai.cli_mcps.set', { allowed }),
    onSuccess: invalidate,
  });
}

/** Resposta ao pedido de confirmação de uma ferramenta externa na conversa. */
export function useDecideExternal() {
  return useMutation({
    mutationFn: (args: { request_id: string; allow: boolean; scope?: 'conversation' }) =>
      call<null>('ai.mcp_extra.decide', args),
  });
}

/** Resposta à pergunta que o agente fez na conversa (`ask_user`). */
export function useAnswerQuestion() {
  return useMutation({
    mutationFn: (args: { request_id: string; selected: string[]; other?: string }) =>
      call<null>('ai.question.answer', args),
  });
}

// --- Roteamento, limites, preços, uso ---------------------------------------------------

export function useRoutes() {
  return useQuery({ queryKey: aiKeys.routes, queryFn: () => call<TaskRoute[]>('ai.routes.list') });
}

export function useSetRoute() {
  const invalidate = useInvalidate(aiKeys.routes, aiKeys.routines);
  return useMutation({
    mutationFn: (args: SetRouteArgs) => call<TaskRoute>('ai.routes.set', args),
    onSuccess: invalidate,
  });
}

export function useLimits() {
  return useQuery({ queryKey: aiKeys.limits, queryFn: () => call<SpendLimit[]>('ai.limits.list') });
}

export function useSetLimit() {
  const invalidate = useInvalidate(aiKeys.limits, ['ai', 'usage']);
  return useMutation({
    mutationFn: (args: SetLimitArgs) => call<SpendLimit>('ai.limits.set', args),
    onSuccess: invalidate,
  });
}

export function usePrices() {
  return useQuery({ queryKey: aiKeys.prices, queryFn: () => call<ModelPrice[]>('ai.prices.list') });
}

export function useSetPrice() {
  const invalidate = useInvalidate(aiKeys.prices);
  return useMutation({
    mutationFn: (args: SetPriceArgs) => call<ModelPrice>('ai.prices.set', args),
    onSuccess: invalidate,
  });
}

export function useDeletePrice() {
  const invalidate = useInvalidate(aiKeys.prices);
  return useMutation({
    mutationFn: (key: PriceKey) => call<null>('ai.prices.delete', key),
    onSuccess: invalidate,
  });
}

export function useUsageSummary(args: UsageSummaryArgs) {
  return useQuery({
    queryKey: aiKeys.usage(args),
    queryFn: () => call<UsageSummary>('ai.usage_summary', args),
  });
}

// --- Ajustes do agente ------------------------------------------------------------------

export function useAiSettings() {
  return useQuery({
    queryKey: aiKeys.settings,
    queryFn: () => call<AiSettings>('ai.settings.get'),
  });
}

export function useSetAiSettings() {
  const invalidate = useInvalidate(aiKeys.settings, aiKeys.routines);
  return useMutation({
    mutationFn: (patch: AiSettingsPatch) => call<AiSettings>('ai.settings.set', patch),
    onSuccess: invalidate,
  });
}

// --- Conversas e jobs -------------------------------------------------------------------

export function useConversations() {
  return useQuery({
    queryKey: aiKeys.conversations,
    queryFn: () => call<Conversation[]>('ai.conversations.list', { limit: 100 }),
  });
}

/** Conversas começadas na página de um ativo (docs/asset-page.md). */
export function useAssetConversations(assetId: string | null) {
  return useQuery({
    queryKey: [...aiKeys.conversations, 'asset', assetId ?? ''],
    queryFn: () => call<Conversation[]>('ai.conversations.list', { limit: 50, asset_id: assetId }),
    enabled: assetId != null,
  });
}

export function useConversation(id: string | null) {
  return useQuery({
    queryKey: aiKeys.conversation(id ?? ''),
    queryFn: () => call<ConversationDetail>('ai.conversations.get', { id }),
    enabled: id != null,
  });
}

export function useDeleteConversation() {
  const invalidate = useInvalidate(aiKeys.conversations);
  return useMutation({
    mutationFn: (id: string) => call<null>('ai.conversations.delete', { id }),
    onSuccess: invalidate,
  });
}

export function usePinConversation() {
  const invalidate = useInvalidate(aiKeys.conversations);
  return useMutation({
    mutationFn: (a: { id: string; pinned: boolean }) => call<null>('ai.conversations.pin', a),
    onSuccess: invalidate,
  });
}

export function useClearConversations() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => call<{ deleted: number }>('ai.conversations.clear'),
    onSuccess: () => {
      client.removeQueries({ queryKey: ['ai', 'conversation'] });
      return client.invalidateQueries({ queryKey: aiKeys.conversations });
    },
  });
}

/**
 * Jobs do agente; com `poll`, consulta a cada 2 s enquanto houver algum rodando. Sem nada
 * rodando, a cada 30 s, só para achar rotinas que o núcleo começou sozinho: as conversas
 * abertas pelo app já invalidam esta consulta ao começar e ao terminar (useChat), e com a
 * janela escondida o React Query não consulta.
 */
export const JOBS_POLL_RUNNING_MS = 2_000;
export const JOBS_POLL_IDLE_MS = 30_000;
export function useJobs({ poll = false }: { poll?: boolean } = {}) {
  return useQuery({
    queryKey: aiKeys.jobs,
    queryFn: () => call<JobInfo[]>('ai.jobs.list'),
    ...(poll
      ? {
          refetchInterval: (q: { state: { data?: JobInfo[] } }) =>
            q.state.data?.some((j) => j.status === 'running')
              ? JOBS_POLL_RUNNING_MS
              : JOBS_POLL_IDLE_MS,
        }
      : {}),
  });
}

/** Cancela um job do agente (conversa ou rotina). */
export function useCancelJob() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (jobId: string) => call<null>('ai.chat_cancel', { job_id: jobId }),
    onSuccess: () => client.invalidateQueries({ queryKey: aiKeys.jobs }),
  });
}

// --- Alertas e rotinas ------------------------------------------------------------------

export function useAlerts(args: AlertListArgs) {
  return useQuery({
    queryKey: aiKeys.alerts(args),
    queryFn: () => call<Alert[]>('ai.alerts.list', args),
  });
}

export function useAlertCounts() {
  return useQuery({
    queryKey: aiKeys.alertCount,
    queryFn: () => call<AlertCounts>('ai.alerts.count'),
    refetchInterval: 60_000,
  });
}

export function useSetAlertStatus() {
  const invalidate = useInvalidate(['ai', 'alerts'], aiKeys.alertCount);
  return useMutation({
    mutationFn: (args: AlertStatusArgs) => call<unknown>('ai.alerts.set_status', args),
    onSuccess: invalidate,
  });
}

export function useDeleteAlerts() {
  const invalidate = useInvalidate(['ai', 'alerts'], aiKeys.alertCount);
  return useMutation({
    mutationFn: (ids: string[]) => call<{ deleted: number }>('ai.alerts.delete', { ids }),
    onSuccess: invalidate,
  });
}

export function useRoutines() {
  return useQuery({
    queryKey: aiKeys.routines,
    queryFn: () => call<RoutineStatus[]>('ai.routines.status'),
  });
}

export function useRunRoutine() {
  const invalidate = useInvalidate(aiKeys.routines, aiKeys.jobs);
  return useMutation({
    mutationFn: (routine: Routine) => call<{ job_id: string }>('ai.routines.run', { routine }),
    onSuccess: invalidate,
  });
}

// --- Auditoria do MCP -------------------------------------------------------------------

export function useAuditLog(args: AuditListArgs) {
  return useQuery({
    queryKey: mcpKeys.audit(args),
    queryFn: () => call<AuditEntry[]>('mcp.audit_list', args),
  });
}

/** Exporta uma conversa em Markdown ou PDF no caminho escolhido (só a interface). */
export function useExportConversation() {
  return useMutation<ConversationExportResult, ApiError, ConversationExportArgs>({
    mutationFn: (args) => call<ConversationExportResult>('ai.conversations.export', args),
  });
}
