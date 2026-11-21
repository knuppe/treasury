/** Hooks de `fx.*`: taxas de câmbio informadas e conversão. */
import type { ConvertArgs } from '../generated/ConvertArgs';
import type { Converted } from '../generated/Converted';
import type { ExchangeArgs } from '../generated/ExchangeArgs';
import type { ExchangeQuote } from '../generated/ExchangeQuote';
import type { FxOperation } from '../generated/FxOperation';
import type { FxRate } from '../generated/FxRate';
import type { ReferenceRate } from '../generated/ReferenceRate';
import type { FxRateInput } from '../generated/FxRateInput';
import { useBaseCurrency } from '@/state/currency';

import { useCore, useCoreMutation } from './coreQuery';

export const useFxRates = (args: { base?: string; quote?: string; limit?: number }) =>
  useCore<FxRate[]>('fx', 'list', args);

export const useConvert = (args: ConvertArgs | null) =>
  useCore<Converted | null>('fx', 'convert', args, { enabled: Boolean(args) });

export const useSetFxRate = () =>
  useCoreMutation<FxRateInput, FxRate>('fx.set', { affects: ['fx'] });

export const useDeleteFxRate = () =>
  useCoreMutation<{ id: string }, null>('fx.delete', { affects: ['fx'] });

/**
 * Câmbio de referência do dia na moeda base da família: no Brasil, a PTAX (boletim mais
 * recente); senão, o último fechamento coletado.
 */
export function useReferenceRate(currency: string | null, date: string | null) {
  const base = useBaseCurrency();
  return useCore<ReferenceRate>(
    'fx',
    'reference_rate',
    { currency, ...(date ? { date } : {}) },
    { enabled: Boolean(currency && currency !== base), staleTime: 5 * 60_000, retry: false },
  );
}

/** Valores, IOF, tarifa e VET de uma operação de câmbio (sem gravar). */
export const useExchangeQuote = (args: ExchangeArgs | null) =>
  useCore<ExchangeQuote>('fx', 'exchange_quote', args, { enabled: Boolean(args), retry: false });

export const useExchange = () =>
  useCoreMutation<ExchangeArgs, FxOperation | null>('fx.exchange', { affects: ['ledger', 'fx'] });

export const useFxOperations = (accountId?: string | null) =>
  useCore<FxOperation[]>('fx', 'operations', accountId ? { account_id: accountId } : {});
