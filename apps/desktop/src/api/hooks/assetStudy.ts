/**
 * Hooks do Estudo com IA de um ativo (docs/agent-enrichment.md §7): estado da aba
 * (`assets.study_state`), prévia do pacote (`assets.study_package`), um estudo
 * (`assets.study_get`), aceite do aviso, contexto, apagar e gerar (`assets.study_start`, que
 * devolve um job acompanhado por `ai.chat_poll`).
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { ChatPoll } from '@/api/generated/ChatPoll';
import type { Study } from '@/api/generated/Study';
import type { StudyArgs } from '@/api/generated/StudyArgs';
import type { StudyPackage } from '@/api/generated/StudyPackage';
import type { StudyState } from '@/api/generated/StudyState';

const key = (action: string, args: object) => ['assets', action, args] as const;

export function useStudyState(assetId: string) {
  return useQuery({
    queryKey: key('study_state', { asset_id: assetId }),
    queryFn: () => call<StudyState>('assets.study_state', { asset_id: assetId }),
    // Um estudo pode terminar com a aba fechada: ao voltar, sempre busca de novo.
    staleTime: 0,
  });
}

export function useStudyPackage(args: StudyArgs, enabled: boolean) {
  return useQuery({
    queryKey: key('study_package', args),
    queryFn: () => call<StudyPackage>('assets.study_package', args),
    enabled,
  });
}

export function useStudy(id: string | null) {
  return useQuery({
    queryKey: key('study_get', { id }),
    queryFn: () => call<Study>('assets.study_get', { id }),
    enabled: Boolean(id),
  });
}

/** Acompanha o job do estudo até terminar. */
export function useStudyJob(jobId: string | null) {
  return useQuery({
    queryKey: ['ai', 'chat_poll', { job_id: jobId }],
    queryFn: () => call<ChatPoll>('ai.chat_poll', { job_id: jobId, after_seq: 0 }),
    enabled: Boolean(jobId),
    refetchInterval: (q) => (q.state.data && q.state.data.status !== 'running' ? false : 1000),
  });
}

function useStudyWrite<A, R>(cmd: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: A) => call<R>(cmd, args),
    onSuccess: () =>
      client.invalidateQueries({
        predicate: (q) =>
          q.queryKey[0] === 'assets' &&
          typeof q.queryKey[1] === 'string' &&
          q.queryKey[1].startsWith('study'),
      }),
  });
}

export const useStudyAccept = () =>
  useStudyWrite<Record<string, never>, null>('assets.study_accept');
export const useStudyContext = () =>
  useStudyWrite<{ asset_id: string; text: string }, null>('assets.study_context_set');
export const useStudyDelete = () => useStudyWrite<{ id: string }, null>('assets.study_delete');
export const useStudyStart = () =>
  useStudyWrite<StudyArgs, { job_id: string }>('assets.study_start');
