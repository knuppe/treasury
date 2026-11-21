import type { Deflator } from '@/api/generated/Deflator';
import { t } from '@/i18n';

import { DEFLATOR_OPTIONS } from '../labels';
import { usePortfolioUi } from '../state';
import { useCountryDeflator } from '../useDeflatorLabel';
import { Segmented } from './Segmented';

/**
 * Seletor do deflator do resultado real: IPCA, inflação do país de referência, US CPI,
 * M2 BR, M2 EUA ou nominal.
 */
export function DeflatorSelect({ size = 'md' }: { size?: 'sm' | 'md' }) {
  const deflator = usePortfolioUi((s) => s.deflator);
  const setDeflator = usePortfolioUi((s) => s.setDeflator);
  const country = useCountryDeflator();
  return (
    <Segmented<Deflator>
      label={t('carteira-deflator-rotulo')}
      value={deflator}
      onChange={setDeflator}
      size={size}
      options={DEFLATOR_OPTIONS.map((o) =>
        o.value === 'country'
          ? {
              value: o.value,
              label: country.code
                ? t('carteira-deflator-pais-de', { pais: country.code })
                : o.label,
              hint:
                country.name && country.seriesName
                  ? t('carteira-deflator-pais-serie', {
                      pais: country.name,
                      serie: country.seriesName,
                      fonte: country.source ?? t('carteira-fonte-oficial'),
                    })
                  : o.hint,
            }
          : { value: o.value, label: o.label, hint: o.hint },
      )}
    />
  );
}
