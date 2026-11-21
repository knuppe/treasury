/**
 * Hooks TanStack Query dos índices (`indices.*`) e da coleta de dados de mercado
 * (`market.sync`: índices + câmbio + cotações). Chaves `[modulo, acao, args]`.
 */
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import { baseCurrency } from '@/lib/format';
import type { CountryCatalog } from '@/api/generated/CountryCatalog';
import type { DataSources } from '@/api/generated/DataSources';
import type { DataSyncReport } from '@/api/generated/DataSyncReport';
import type { Deflator } from '@/api/generated/Deflator';
import type { FactorResult } from '@/api/generated/FactorResult';
import type { FxRate } from '@/api/generated/FxRate';
import type { IndexSeries } from '@/api/generated/IndexSeries';
import type { IndexValuePoint } from '@/api/generated/IndexValuePoint';
import type { ProviderStatus } from '@/api/generated/ProviderStatus';

import { cleanArgs } from './portfolio';

export function useIndexSeries() {
  return useQuery({
    queryKey: ['indices', 'list_series', {}] as const,
    queryFn: () => call<IndexSeries[]>('indices.list_series', {}),
  });
}

export interface SeriesArgs {
  id: string;
  from?: string;
  to?: string;
}

/** Pontos de uma série (padrão do núcleo: últimos 24 meses). */
export function useSeriesPoints(args: SeriesArgs | null, options: { enabled?: boolean } = {}) {
  const a = cleanArgs(args ?? undefined);
  return useQuery({
    queryKey: ['indices', 'get_series', a] as const,
    queryFn: () => call<IndexValuePoint[]>('indices.get_series', a),
    enabled: Boolean(args?.id) && (options.enabled ?? true),
    placeholderData: keepPreviousData,
  });
}

export function useDeflatorFactor(args: { deflator: Deflator; from: string; to: string } | null) {
  return useQuery({
    queryKey: ['indices', 'factor', args ?? {}] as const,
    queryFn: () => call<FactorResult>('indices.factor', args),
    enabled: Boolean(args),
  });
}

/** Última taxa de câmbio conhecida (`fx.list`) para a moeda base, ex.: USD → BRL (PTAX). */
export function useLatestFx(base: string, quote: string = baseCurrency()) {
  return useQuery({
    queryKey: ['fx', 'list', { base, quote, limit: 1 }] as const,
    queryFn: () => call<FxRate[]>('fx.list', { base, quote, limit: 1 }),
    select: (rows) => rows[0] ?? null,
  });
}

export function useDataSources() {
  return useQuery({
    queryKey: ['market', 'data_sources', {}] as const,
    queryFn: () => call<DataSources>('market.data_sources', {}),
  });
}

/** Coleta índices, câmbio e cotações (pode levar alguns minutos). */
export function useMarketSync() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => call<DataSyncReport>('market.sync', {}),
    gcTime: 5 * 60_000,
    onSuccess: () =>
      client.invalidateQueries({
        predicate: (q) =>
          ['indices', 'market', 'fx', 'portfolio', 'assets', 'reports', 'accounts'].includes(
            String(q.queryKey[0]),
          ),
      }),
  });
}

/** Cadastra (ou apaga, com texto vazio) a chave gratuita da FRED. Só pela interface. */
export function useSetFredKey() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (key: string) => call<null>('market.set_fred_key', { key }),
    // A chave não fica no cache de mutações (não reaparece em nenhum lugar).
    gcTime: 0,
    onSuccess: () => client.invalidateQueries({ queryKey: ['market', 'data_sources'] }),
  });
}

/** País de referência e catálogo de países (inflação, juros e câmbio de cada um). */
export function useCountryCatalog() {
  return useQuery({
    queryKey: ['indices', 'catalog', {}] as const,
    queryFn: () => call<CountryCatalog>('indices.catalog', {}),
    staleTime: 5 * 60_000,
  });
}

/** Provedores de dados com uso, última coleta e último erro. */
export function useProviderStatus() {
  return useQuery({
    queryKey: ['indices', 'providers', {}] as const,
    queryFn: () => call<ProviderStatus[]>('indices.providers', {}),
  });
}

/** Escolhe o país de referência (`null` volta ao automático, pela moeda base). */
export function useSetReferenceCountry() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (country: string | null) =>
      call<CountryCatalog>('indices.set_country', { country }),
    onSuccess: () =>
      client.invalidateQueries({
        predicate: (q) => ['indices', 'portfolio', 'reports'].includes(String(q.queryKey[0])),
      }),
  });
}
