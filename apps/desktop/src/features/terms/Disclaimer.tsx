/**
 * Avisos das telas (docs/terms.md §5): rodapés curtos e fixos, nunca modais. Impostos citam a
 * fonte, a data e o órgão do país do cofre, e as limitações conhecidas nos EUA e em Portugal;
 * investimentos lembram que são ferramenta de estudo.
 */
import type { ReactNode } from 'react';

import { Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

/** País com órgão fiscal próprio no aviso; o resto vira o genérico. */
const TAX_COUNTRIES = ['BR', 'US', 'PT'];

export function FooterNotice({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <p
      role="note"
      className={cx('flex items-start gap-1.5 text-[12px] leading-snug text-dim', className)}
    >
      <Icon name="info" size={12} className="mt-0.5 shrink-0" />
      <span>{children}</span>
    </p>
  );
}

export function StudyNotice({ className }: { className?: string }) {
  return <FooterNotice className={className}>{t('termos-aviso-estudo')}</FooterNotice>;
}

export function TaxNotice({
  source,
  date,
  country,
  className,
}: {
  source: string;
  date: string;
  country: string | null | undefined;
  className?: string;
}) {
  // As divergências conhecidas com os casos oficiais (docs/tax-validation-countries.md) ficam
  // junto do número, não só no doc (Termos 5.1).
  const limits =
    country === 'US'
      ? t('termos-aviso-imposto-limites-us')
      : country === 'PT'
        ? t('termos-aviso-imposto-limites-pt')
        : null;
  return (
    <FooterNotice className={className}>
      {t('termos-aviso-imposto', {
        fonte: source,
        data: date,
        pais: country && TAX_COUNTRIES.includes(country) ? country : 'outro',
      })}
      {limits ? ` ${limits}` : null}
    </FooterNotice>
  );
}
