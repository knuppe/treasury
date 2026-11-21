/**
 * Moeda base da família: a do país do cofre (`household.base_currency`). Vale como padrão para
 * os valores que não dizem a moeda (`formatMoney`, `<Money>`, dicas dos gráficos); contas e
 * ativos com moeda própria continuam na deles.
 */
import { create } from 'zustand';

import { setFormatBaseCurrency } from '@/lib/format';

export const useBaseCurrencyStore = create<{ code: string }>()(() => ({ code: 'BRL' }));

export function setBaseCurrency(code: string): void {
  setFormatBaseCurrency(code);
  if (useBaseCurrencyStore.getState().code !== code) useBaseCurrencyStore.setState({ code });
}

/** A moeda base, redesenhando quando ela muda (troca de país ou de cofre). */
export function useBaseCurrency(): string {
  return useBaseCurrencyStore((s) => s.code);
}
