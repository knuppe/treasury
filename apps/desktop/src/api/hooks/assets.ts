/**
 * Hooks TanStack Query do cadastro de ativos (`assets.*`) e das cotações
 * (`market.prices`, `market.set_price`). Chaves no formato `[modulo, acao, args]`.
 */
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import type { Asset } from '@/api/generated/Asset';
import type { AssetFilter } from '@/api/generated/AssetFilter';
import type { AssetLabel } from '@/api/generated/AssetLabel';
import type { AssetInput } from '@/api/generated/AssetInput';
import type { AssetPatch } from '@/api/generated/AssetPatch';
import type { ManualPriceArgs } from '@/api/generated/ManualPriceArgs';
import type { PricePoint } from '@/api/generated/PricePoint';
import type { SetAssetLabelsArgs } from '@/api/generated/SetAssetLabelsArgs';
import type { UpdateAssetLabelArgs } from '@/api/generated/UpdateAssetLabelArgs';

import { cleanArgs, invalidatePortfolio } from './portfolio';

export const assetKeys = {
  all: ['assets'] as const,
  of: (action: string, args?: object) => ['assets', action, cleanArgs(args)] as const,
};

export function useAssets(filter: AssetFilter = {}, options: { enabled?: boolean } = {}) {
  const args = cleanArgs(filter);
  return useQuery({
    queryKey: assetKeys.of('list', args),
    queryFn: () => call<Asset[]>('assets.list', args),
    placeholderData: keepPreviousData,
    enabled: options.enabled ?? true,
  });
}

export function useAsset(id: string | null | undefined) {
  return useQuery({
    queryKey: assetKeys.of('get', { id }),
    queryFn: () => call<Asset>('assets.get', { id }),
    enabled: Boolean(id),
  });
}

/** Busca exata por ticker (sem diferenciar maiúsculas); `null` se não houver. */
export function findAssetByTicker(ticker: string): Promise<Asset | null> {
  return call<Asset | null>('assets.find', { ticker });
}

function useAssetWrite<A, R>(cmd: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: A) => call<R>(cmd, args),
    onSuccess: () => invalidatePortfolio(client),
  });
}

export const useCreateAsset = () => useAssetWrite<AssetInput, Asset>('assets.create');
export const useUpdateAsset = () => useAssetWrite<AssetPatch, Asset>('assets.update');
export const useDeleteAsset = () => useAssetWrite<{ id: string }, null>('assets.delete');

// ── Etiquetas (assets.labels_*): servem às metas e aos grupos de concentração ──

export function useAssetLabels(options: { enabled?: boolean } = {}) {
  return useQuery({
    queryKey: assetKeys.of('labels_list'),
    queryFn: () => call<AssetLabel[]>('assets.labels_list', {}),
    enabled: options.enabled ?? true,
  });
}

/** Troca as etiquetas de um ativo (as novas são criadas); devolve os nomes. */
export const useSetAssetLabels = () =>
  useAssetWrite<SetAssetLabelsArgs, string[]>('assets.labels_set');
/** Marca a etiqueta como grupo de concentração e ajusta o limite dela. */
export const useUpdateAssetLabel = () =>
  useAssetWrite<UpdateAssetLabelArgs, AssetLabel>('assets.label_update');

// ── Cotações (market.*) ──────────────────────────────────────────

export interface PriceHistoryArgs {
  asset_id: string;
  from?: string;
  to?: string;
}

/** Fechamentos diários (`market.prices`; padrão: últimos 12 meses). */
export function usePriceHistory(args: PriceHistoryArgs | null) {
  const a = cleanArgs(args ?? undefined);
  return useQuery({
    queryKey: ['market', 'prices', a] as const,
    queryFn: () => call<PricePoint[]>('market.prices', a),
    enabled: Boolean(args?.asset_id),
    placeholderData: keepPreviousData,
  });
}

/** Cotação manual (ativos sem fonte automática): `market.set_price`. */
export function useSetPrice() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: ManualPriceArgs) => call<null>('market.set_price', args),
    onSuccess: () =>
      client.invalidateQueries({
        predicate: (q) => ['market', 'portfolio', 'reports'].includes(String(q.queryKey[0])),
      }),
  });
}
