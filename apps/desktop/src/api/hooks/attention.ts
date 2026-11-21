/**
 * "Para resolver" (`attention.list`, docs/to-resolve.md): a lista única do que precisa da
 * pessoa e a cobertura dos números. O sino da barra superior, o cartão do Painel e as linhas
 * de cobertura leem a mesma consulta.
 */
import { useMutation, useQueryClient } from '@tanstack/react-query';

import { call } from '../client';
import type { Attention } from '../generated/Attention';
import type { MonthClose } from '../generated/MonthClose';
import type { MonthCloseSetArgs } from '../generated/MonthCloseSetArgs';
import type { NextStep } from '../generated/NextStep';
import { useCore } from './coreQuery';

export const ATTENTION_KEY = ['attention'] as const;

export const useAttention = (enabled = true) =>
  useCore<Attention>(
    'attention',
    'list',
    {},
    { enabled, staleTime: 30_000, refetchInterval: 5 * 60_000 },
  );

/** Dispensa um item: o alerta vira resolvido; os demais ficam adiados no cofre. */
export function useDismissAttention() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => call<unknown>('attention.dismiss', { id }),
    onSuccess: () =>
      Promise.all([
        client.invalidateQueries({ queryKey: ATTENTION_KEY }),
        client.invalidateQueries({ queryKey: ['ai', 'alerts'] }),
        client.invalidateQueries({ queryKey: ['ai', 'alerts_count'] }),
      ]),
  });
}

/** "Seu próximo passo": o degrau atual e uma coisa para fazer agora (`attention.next_step`). */
export const useNextStep = (enabled = true) =>
  useCore<NextStep>('attention', 'next_step', {}, { enabled, staleTime: 60_000 });

/** O roteiro de fechar o mês que terminou (`attention.month_close`). */
export const useMonthClose = (enabled = true) =>
  useCore<MonthClose>('attention', 'month_close', {}, { enabled, staleTime: 30_000 });

/** Marca um passo manual do mês, fecha o mês ou reabre. */
export function useSetMonthClose() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: MonthCloseSetArgs) => call<unknown>('attention.month_close_set', args),
    onSuccess: () => client.invalidateQueries({ queryKey: ATTENTION_KEY }),
  });
}
