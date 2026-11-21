import { QueryClient } from '@tanstack/react-query';

import { ApiError } from './client';

/**
 * Cliente do TanStack Query: cache só em memória (nunca persistido). Erros de
 * domínio do núcleo (`ApiError`) não são repetidos; falhas de rede, uma vez.
 */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: (failureCount, error) =>
          error instanceof ApiError && error.code !== 'network' ? false : failureCount < 1,
        refetchOnWindowFocus: false,
        staleTime: 30_000,
      },
      mutations: {
        retry: false,
      },
    },
  });
}
