/** Consultas e mutações do feedback do agente ao desenvolvedor (`devfeedback.*`). */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { DevFeedback } from '@/api/generated/DevFeedback';
import type { DevFeedbackListArgs } from '@/api/generated/DevFeedbackListArgs';
import type { DevFeedbackPatch } from '@/api/generated/DevFeedbackPatch';
import type { DevFeedbackPriority } from '@/api/generated/DevFeedbackPriority';
import type { DevFeedbackStatusArgs } from '@/api/generated/DevFeedbackStatusArgs';

/** Escritas devolvem o feedback e o conjunto de alterações (para o "Desfazer"). */
export type DevFeedbackWritten = DevFeedback & { change_set_id: string | null };

export const devFeedbackKeys = {
  all: ['devfeedback'] as const,
  list: (args: DevFeedbackListArgs) => ['devfeedback', 'list', args] as const,
};

export function useDevFeedback(args: DevFeedbackListArgs = {}, enabled = true) {
  return useQuery({
    queryKey: devFeedbackKeys.list(args),
    queryFn: () => call<DevFeedback[]>('devfeedback.list', args),
    enabled,
  });
}

function useInvalidate() {
  const client = useQueryClient();
  return () => client.invalidateQueries({ queryKey: devFeedbackKeys.all });
}

export function useSetDevFeedbackStatus() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (args: DevFeedbackStatusArgs) =>
      call<DevFeedbackWritten>('devfeedback.set_status', args),
    onSuccess: invalidate,
  });
}

/** Importância e/ou lugar manual na coluna do quadro. */
export function useSetDevFeedbackPriority() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (args: DevFeedbackPriority) =>
      call<DevFeedbackWritten>('devfeedback.set_priority', args),
    onSuccess: invalidate,
  });
}

export function useUpdateDevFeedback() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (patch: DevFeedbackPatch) => call<DevFeedbackWritten>('devfeedback.update', patch),
    onSuccess: invalidate,
  });
}

/** Grava o Markdown no caminho escolhido no diálogo de salvar (núcleo; só a interface). */
export function useExportDevFeedback() {
  return useMutation({
    mutationFn: (a: { path: string; text: string }) =>
      call<{ path: string; bytes: number }>('devfeedback.export', a),
  });
}

export function useDeleteDevFeedback() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (id: string) =>
      call<{ deleted: string; change_set_id: string | null }>('devfeedback.delete', { id }),
    onSuccess: invalidate,
  });
}
