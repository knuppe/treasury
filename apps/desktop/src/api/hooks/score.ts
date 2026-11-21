/**
 * Hooks do score de saúde por ativo (`score.*`). Chaves `['score', acao, args]`.
 *
 * Não confundir com a saúde financeira da família (`health.*`): aqui a nota é de um
 * ativo (empresa ou fundo), calculada com os fundamentos públicos da análise de ativos.
 * Os comandos aceitam `asset_id` (cadastrado) ou `ticker` (papel do universo).
 *
 * Unidades: notas de 0 a 100, confiança e cobertura em fração (0–1), valores das
 * métricas pela unidade (`percent` em fração, `multiple` em vezes, `money` em reais).
 */
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call, type ApiError } from '@/api/client';
import type { AssetScore } from '@/api/generated/AssetScore';
import type { AssetScoreArgs } from '@/api/generated/AssetScoreArgs';
import type { AssetScoreEventInput } from '@/api/generated/AssetScoreEventInput';
import type { AssetScoreEventStatusArgs } from '@/api/generated/AssetScoreEventStatusArgs';
import type { AssetScoreFeedback } from '@/api/generated/AssetScoreFeedback';
import type { AssetScoreFeedbackInput } from '@/api/generated/AssetScoreFeedbackInput';
import type { AssetScoreMethodology } from '@/api/generated/AssetScoreMethodology';
import type { AssetScorePoint } from '@/api/generated/AssetScorePoint';
import type { AssetScoreSegmentArgs } from '@/api/generated/AssetScoreSegmentArgs';
import type { AssetScoreSummary } from '@/api/generated/AssetScoreSummary';

import { cleanArgs } from './portfolio';

export const scoreKeys = {
  all: ['score'] as const,
  of: (action: string, args?: object) => ['score', action, cleanArgs(args)] as const,
};

function read<T>(action: string, args?: object, enabled = true) {
  const a = cleanArgs(args);
  return {
    queryKey: scoreKeys.of(action, a),
    queryFn: () => call<T>(`score.${action}`, a),
    enabled,
  };
}

const hasSubject = (s: AssetScoreArgs | null | undefined) => Boolean(s?.asset_id || s?.ticker);

// ── Leituras ─────────────────────────────────────────────────────

/** Score de um ativo (refeito no núcleo a cada consulta; grava o snapshot do dia). */
export function useAssetScore(subject: AssetScoreArgs | null) {
  return useQuery(read<AssetScore>('get', subject ?? {}, hasSubject(subject)));
}

/** Scores dos ativos com posição ou na watchlist, na ordem do núcleo. */
export function useScorePortfolio() {
  return useQuery(read<AssetScoreSummary[]>('portfolio'));
}

/** Resumo do score de vários ativos cadastrados (lista de Ativos), na ordem pedida. */
export function useScoreAssets(assetIds: readonly string[]) {
  return useQuery({
    ...read<AssetScoreSummary[]>('assets', { asset_ids: [...assetIds] }, assetIds.length > 0),
    placeholderData: keepPreviousData,
    staleTime: 5 * 60_000,
  });
}

export function useScoreHistory(subject: AssetScoreArgs | null) {
  return useQuery(read<AssetScorePoint[]>('history', subject ?? {}, hasSubject(subject)));
}

/** Perfis, pesos, segmentos e faixas da metodologia em uso. */
export function useScoreMethodology() {
  return useQuery(read<AssetScoreMethodology>('methodology'));
}

export function useScoreFeedback(subject: AssetScoreArgs | null) {
  return useQuery(read<AssetScoreFeedback[]>('feedback.list', subject ?? {}, hasSubject(subject)));
}

// ── Escritas (só pela interface) ─────────────────────────────────

function useScoreWrite<A, R>(cmd: string) {
  const client = useQueryClient();
  return useMutation<R, ApiError, A>({
    mutationFn: (args: A) => call<R>(cmd, args),
    onSuccess: () => client.invalidateQueries({ queryKey: scoreKeys.all }),
  });
}

/** Troca o segmento de um papel (`segment: null` volta ao automático); os scores são refeitos. */
export const useSetScoreSegment = () =>
  useScoreWrite<AssetScoreSegmentArgs, AssetScoreMethodology>('score.set_segment');

/** "Discordo"/"concordo" com a nota; devolve o id do registro. */
export const useAddScoreFeedback = () =>
  useScoreWrite<AssetScoreFeedbackInput, string>('score.feedback.add');

/** Registra um evento qualitativo (entra aceito, salvo `accepted: false`); devolve o id. */
export const useAddScoreEvent = () =>
  useScoreWrite<AssetScoreEventInput, string>('score.events.add');

/** Aceita, rejeita ou resolve um evento. */
export const useSetScoreEventStatus = () =>
  useScoreWrite<AssetScoreEventStatusArgs, null>('score.events.set_status');

export const useDeleteScoreEvent = () => useScoreWrite<{ id: string }, null>('score.events.delete');
