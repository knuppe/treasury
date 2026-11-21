/**
 * Hooks da saúde financeira (`health.*`). Chaves `['health', acao, args]`.
 *
 * A nota depende de tudo o que entra nos relatórios (lançamentos, parcelas, faturas,
 * saldos): é refeita a cada vez que uma tela que a mostra abre (`staleTime: 0`), em vez de
 * entrar nos grupos de invalidação de cada escrita.
 */
import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { HealthModel } from '@/api/generated/HealthModel';
import type { HealthReport } from '@/api/generated/HealthReport';
import type { HealthScoreArgs } from '@/api/generated/HealthScoreArgs';
import type { HealthSettingsView } from '@/api/generated/HealthSettingsView';

import { useCore, useCoreMutation } from './coreQuery';

export const healthKeys = {
  all: ['health'] as const,
};

export function useHealthScore(args: HealthScoreArgs, enabled = true) {
  return useQuery({
    queryKey: ['health', 'score', args],
    queryFn: () => call<HealthReport>('health.score', args),
    enabled,
    staleTime: 0,
    placeholderData: keepPreviousData,
  });
}

export const useHealthSettings = () => useCore<HealthSettingsView>('health', 'get_settings');

/** Troca o modelo (só pela interface); a nota e os ajustes são refeitos. */
export function useSetHealthModel() {
  return useCoreMutation<{ model: HealthModel }, HealthSettingsView>('health.set_settings', {
    affects: ['health'],
  });
}
