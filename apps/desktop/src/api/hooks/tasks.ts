/**
 * Tarefas em segundo plano do núcleo (docs/background-tasks.md): coletas de índices,
 * cotações e fundamentos, e-mail de ingestão e o download da atualização. O núcleo avisa o
 * início e o fim (`tasks://changed`); o progresso é relido a cada segundo só enquanto há
 * tarefa rodando.
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { call } from '@/api/client';
import type { TaskEvent } from '@/api/generated/TaskEvent';
import type { TaskInfo } from '@/api/generated/TaskInfo';
import { listenApp } from '@/features/agent/tauriBridge';

export const taskKeys = { list: ['tasks', 'list'] as const };

export const TASKS_POLL_RUNNING_MS = 1_000;
export const TASKS_POLL_IDLE_MS = 30_000;

export function useTasks() {
  const client = useQueryClient();
  useEffect(
    () =>
      listenApp<TaskEvent>(
        'tasks://changed',
        () => void client.invalidateQueries({ queryKey: taskKeys.list }),
      ),
    [client],
  );
  return useQuery({
    queryKey: taskKeys.list,
    queryFn: () => call<TaskInfo[]>('tasks.list'),
    refetchInterval: (q: { state: { data?: TaskInfo[] } }) =>
      q.state.data?.some((x) => x.status === 'running')
        ? TASKS_POLL_RUNNING_MS
        : TASKS_POLL_IDLE_MS,
  });
}

/** Pede o cancelamento (a tarefa para na próxima fronteira e guarda o que já veio). */
export function useCancelTask() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => call<TaskInfo>('tasks.cancel', { id }),
    onSuccess: () => client.invalidateQueries({ queryKey: taskKeys.list }),
  });
}
