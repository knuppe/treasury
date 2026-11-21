/** Hooks de `country.*` (docs/countries.md): o país da família e o que muda com ele. */
import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';

import { call } from '../client';
import type { CountryChange } from '../generated/CountryChange';
import type { CountryChoice } from '../generated/CountryChoice';
import type { CountryFeatures } from '../generated/CountryFeatures';
import type { CountryInfo } from '../generated/CountryInfo';
import type { CountryState } from '../generated/CountryState';
import type { SetCountryArgs } from '../generated/SetCountryArgs';
import { useCore, useCoreMutation } from './coreQuery';

export const useCountry = () => useCore<CountryState>('country', 'get');

/** O país da família (`null` enquanto carrega). */
export const useCountryInfo = (): CountryInfo | null => useCountry().data?.info ?? null;

/** Recursos do Brasil enquanto o país carrega: os cofres antigos são todos do Brasil e as
 * telas não piscam. */
const BRAZIL: CountryFeatures = {
  br_market: true,
  br_income_tax: true,
  br_credit_rates: true,
  fipe: true,
  fgts: true,
  tax_burden: true,
  payroll: true,
  receipts: true,
  iptu: true,
};

export function useCountryFeatures(): CountryFeatures {
  const f = useCountry().data?.info.features;
  // Mesma referência enquanto os valores não mudam: o país chegar (igual ao padrão) não
  // recalcula o menu nem as telas.
  const key = JSON.stringify(f ?? BRAZIL);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  return useMemo(() => f ?? BRAZIL, [key]);
}

/** Países para escolher, mesmo sem cofre (criação do cofre). */
export const useCountryChoices = () =>
  useQuery({
    queryKey: ['app', 'countries'],
    queryFn: () => call<CountryChoice[]>('app.countries', {}),
    staleTime: Infinity,
  });

/** Troca o país: fontes, moeda, índices, impostos e telas mudam juntos. */
export const useSetCountry = () =>
  useCoreMutation<SetCountryArgs, CountryChange>('country.set', { affects: 'all' });
