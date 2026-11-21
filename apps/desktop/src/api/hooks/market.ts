/**
 * Hooks do catálogo de bolsas, das cotações intradiárias (Yahoo, ~15 min de atraso) e da
 * busca de tickers (`market.exchanges`, `market.quotes`, `market.intraday`,
 * `market.refresh_quotes`, `market.search`).
 */
import { keepPreviousData, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';

import { call } from '@/api/client';
import type { Exchange } from '@/api/generated/Exchange';
import type { ExchangeInput } from '@/api/generated/ExchangeInput';
import type { IntradaySeries } from '@/api/generated/IntradaySeries';
import type { MarketQuote } from '@/api/generated/MarketQuote';
import type { QuoteRefreshReport } from '@/api/generated/QuoteRefreshReport';
import type { TickerSearch } from '@/api/generated/TickerSearch';
import { listenApp } from '@/features/agent/tauriBridge';

import { coreKey, useCore, useCoreMutation } from './coreQuery';

/** Cotações intradiárias são relidas a cada 5 min (a rotina do app coleta nesse ritmo). */
export const QUOTE_REFRESH_MS = 5 * 60_000;

export const useExchanges = () =>
  useCore<Exchange[]>('market', 'exchanges', undefined, { staleTime: 10 * 60_000 });

export const useSaveExchange = () =>
  useCoreMutation<ExchangeInput, Exchange>('market.exchange_save', { affects: ['market'] });

export const useDeleteExchange = () =>
  useCoreMutation<{ code: string }, null>('market.exchange_delete', { affects: ['market'] });

/** Últimas cotações intradiárias (todas as coletadas). */
export function useQuotes(options: { enabled?: boolean } = {}) {
  return useCore<MarketQuote[]>(
    'market',
    'quotes',
    {},
    {
      refetchInterval: QUOTE_REFRESH_MS,
      placeholderData: keepPreviousData,
      enabled: options.enabled ?? true,
    },
  );
}

/** Mapa ativo → cotação intradiária. */
export function useQuoteMap(): Map<string, MarketQuote> {
  const q = useQuotes();
  return useMemo(() => new Map((q.data ?? []).map((x) => [x.asset_id, x])), [q.data]);
}

export function useIntraday(assetId: string | null | undefined, range: '1d' | '5d') {
  return useQuery({
    queryKey: coreKey('market', 'intraday', { asset_id: assetId, range }),
    queryFn: () => call<IntradaySeries>('market.intraday', { asset_id: assetId, range }),
    enabled: Boolean(assetId),
    refetchInterval: QUOTE_REFRESH_MS,
    placeholderData: keepPreviousData,
  });
}

/** Coleta agora (rede): sem ids, todos os ativos acompanhados, mesmo fora do pregão. */
export const useRefreshQuotes = () =>
  useCoreMutation<{ asset_ids?: string[] }, QuoteRefreshReport>('market.refresh_quotes', {
    affects: ['market'],
  });

/** Busca de tickers com espera de digitação (`delayMs`) e cache curto do TanStack Query. */
export function useTickerSearch(query: string, delayMs = 300) {
  const [debounced, setDebounced] = useState(query.trim());
  useEffect(() => {
    const t = setTimeout(() => setDebounced(query.trim()), delayMs);
    return () => clearTimeout(t);
  }, [query, delayMs]);
  const q = useQuery({
    queryKey: coreKey('market', 'search', { query: debounced }),
    queryFn: () => call<TickerSearch>('market.search', { query: debounced, limit: 12 }),
    enabled: debounced.length >= 1,
    staleTime: 5 * 60_000,
    placeholderData: keepPreviousData,
  });
  return { ...q, pending: debounced !== query.trim() || q.isFetching, query: debounced };
}

/** O app avisa quando a rotina coletou cotações novas: relê cotações e carteira. */
export function useQuoteEvents(): void {
  const client = useQueryClient();
  useEffect(
    () =>
      listenApp<number>('market://quotes', () => {
        for (const m of ['market', 'portfolio', 'reports']) {
          void client.invalidateQueries({ queryKey: [m] });
        }
      }),
    [client],
  );
}
