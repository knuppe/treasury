/**
 * Hooks dos dados dos ativos com fonte (enriquecimento, fase 1; docs/agent-enrichment.md):
 * campos faltando e sugestões (`assets.data_gaps`), "de onde veio" (`assets.field_sources`),
 * gravar sugestões escolhidas (`assets.enrich_apply`, com histórico e desfazer) e travar um
 * campo (`assets.field_lock`); consultar as fontes da rede (`assets.enrich_lookup`, fase 2).
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { AssetFieldSources } from '@/api/generated/AssetFieldSources';
import type { DataGaps } from '@/api/generated/DataGaps';
import type { DataGapsArgs } from '@/api/generated/DataGapsArgs';
import type { EnrichApplied } from '@/api/generated/EnrichApplied';
import type { EnrichItem } from '@/api/generated/EnrichItem';
import type { FieldLockArgs } from '@/api/generated/FieldLockArgs';
import type { LookupAllArgs } from '@/api/generated/LookupAllArgs';
import type { LookupAllReport } from '@/api/generated/LookupAllReport';
import type { LookupArgs } from '@/api/generated/LookupArgs';
import type { LookupReport } from '@/api/generated/LookupReport';

import { assetKeys } from './assets';
import { cleanArgs, invalidatePortfolio } from './portfolio';

export function useDataGaps(args: DataGapsArgs = {}, options: { enabled?: boolean } = {}) {
  const a = cleanArgs(args);
  return useQuery({
    queryKey: assetKeys.of('data_gaps', a),
    queryFn: () => call<DataGaps>('assets.data_gaps', a),
    enabled: options.enabled ?? true,
  });
}

export function useFieldSources(assetId: string | null | undefined) {
  return useQuery({
    queryKey: assetKeys.of('field_sources', { asset_id: assetId }),
    queryFn: () => call<AssetFieldSources>('assets.field_sources', { asset_id: assetId }),
    enabled: Boolean(assetId),
  });
}

function useDataWrite<A, R>(cmd: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: A) => call<R>(cmd, args),
    onSuccess: () => invalidatePortfolio(client),
  });
}

export const useEnrichApply = () =>
  useDataWrite<{ items: EnrichItem[] }, EnrichApplied>('assets.enrich_apply');
export const useFieldLock = () => useDataWrite<FieldLockArgs, null>('assets.field_lock');

/** Consulta as fontes da rede (B3, GLEIF, BrasilAPI) para um ativo: só grava leituras. */
export const useEnrichLookup = () => useDataWrite<LookupArgs, LookupReport>('assets.enrich_lookup');
/** O mesmo para todos os ativos cobertos (só a interface). */
export const useEnrichLookupAll = () =>
  useDataWrite<LookupAllArgs, LookupAllReport>('assets.enrich_lookup_all');
