import { cx } from '../cx';
import { t } from '@/i18n';

export interface LiveIndicatorProps {
  /** Texto do indicador (padrão "Ao vivo"). */
  label?: string;
  /** `false` mostra o ponto parado, sem pulsar. */
  active?: boolean;
  tone?: 'positive' | 'accent' | 'warning' | 'negative' | 'dim';
  className?: string;
}

const TONES = {
  positive: 'text-positive',
  accent: 'text-accent',
  warning: 'text-warning',
  negative: 'text-negative',
  dim: 'text-dim',
} as const;

/**
 * `● AO VIVO`: indica dados atualizando. O pulso some com movimento reduzido. No tema
 * Scanner, o ponto ativo vira um mini scanner (scanner.css).
 */
export function LiveIndicator({
  label = t('componentes-ao-vivo'),
  active = true,
  tone = 'positive',
  className,
}: LiveIndicatorProps) {
  return (
    <span className={cx('label-caps inline-flex items-center gap-1.5', TONES[tone], className)}>
      <span
        aria-hidden
        className={cx(
          'inline-block size-1.5 rounded-full bg-current',
          active ? 'live-dot animate-pulse-dot' : 'opacity-60',
        )}
      />
      {active ? <span aria-hidden className="scanner-mini" /> : null}
      {label}
    </span>
  );
}
