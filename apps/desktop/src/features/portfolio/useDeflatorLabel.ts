/**
 * Rótulos do deflator com o país de referência (`indices.catalog`): "Inflação do país
 * (Portugal)" e a série que ele usa (ex.: HICP Portugal, BCE).
 */
import { useCallback } from 'react';

import type { Deflator } from '@/api/generated/Deflator';
import { useCountryInfo } from '@/api/hooks/country';
import { useCountryCatalog } from '@/api/hooks/indices';

import { deflatorLabel, sourceName } from './labels';

export interface CountryDeflator {
  /** ISO do país de referência (ex.: `PT`), ou `null` enquanto carrega. */
  code: string | null;
  name: string | null;
  /** Série principal de inflação do país e a fonte, para dicas. */
  seriesName: string | null;
  source: string | null;
}

export function useCountryDeflator(): CountryDeflator {
  const q = useCountryCatalog();
  const p = q.data?.countries.find((c) => c.country === q.data?.country);
  const s = p?.inflation[0];
  return {
    code: q.data?.country ?? null,
    name: p?.name ?? null,
    seriesName: s?.name ?? null,
    source: s ? sourceName(s.source) : null,
  };
}

/** `(d) => rótulo` com o nome do país de referência em `country`. Fora do Brasil o IPCA (o
 * padrão das telas) vira a inflação do país da família, como no núcleo. */
export function useDeflatorLabel(): (d: Deflator) => string {
  const { name } = useCountryDeflator();
  const info = useCountryInfo();
  return useCallback(
    (d: Deflator) =>
      info && info.code !== 'BR' && (d === 'ipca' || d === 'country')
        ? `${info.inflation_label} (${info.name})`
        : deflatorLabel(d, name),
    [name, info],
  );
}
