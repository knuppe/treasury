import type { ReactNode } from 'react';

import { formatMoney } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { usePrivacy } from '@/state/ui';
import { t } from '@/i18n';

import { cx } from '../cx';
import { Decode } from './Decode';

export const PRIVACY_MASK = '•••••';

export interface MoneyProps {
  /** Valor inteiro na menor unidade da moeda (centavos). */
  cents: number;
  /** Código ISO 4217 (padrão: a moeda base da família). */
  currency?: string;
  /**
   * Cor por sinal: `auto` pinta ganho em --positive e perda em --negative
   * (e mostra o sinal `+`); `neutral` (padrão) usa a cor do texto.
   */
  tone?: 'auto' | 'neutral' | 'positive' | 'negative';
  /** Mostra `+` em positivos. Padrão: ligado com `tone="auto"` ou `trend`. */
  signed?: boolean;
  /** Mostra ▲/▼ antes do valor. */
  trend?: boolean;
  /** Valor estimado: itálico com `≈`. */
  estimated?: boolean;
  hideCurrency?: boolean;
  /** Ignora o modo privacidade (use só quando o próprio usuário pediu para ver). */
  reveal?: boolean;
  className?: string;
}

/**
 * Valor monetário formatado em pt-BR. Ganho e perda nunca dependem só da cor:
 * há sempre sinal (`+`/`−`) e, opcionalmente, ▲/▼. No modo privacidade,
 * mostra `•••••`.
 */
export function Money({
  cents,
  currency,
  tone = 'neutral',
  signed,
  trend = false,
  estimated = false,
  hideCurrency = false,
  reveal = false,
  className,
}: MoneyProps) {
  const privacy = usePrivacy();
  const base = useBaseCurrency();

  if (privacy && !reveal) {
    return (
      <span data-money="hidden" className={cx('num whitespace-nowrap', className)}>
        <span aria-hidden>{PRIVACY_MASK}</span>
        <span className="sr-only">{t('componentes-valor-oculto')}</span>
      </span>
    );
  }

  const showSign = signed ?? (tone === 'auto' || trend);
  const text = formatMoney(cents, currency ?? base, { signed: showSign, hideCurrency });
  const direction = cents > 0 ? 'up' : cents < 0 ? 'down' : 'flat';
  const color =
    tone === 'positive'
      ? 'text-positive'
      : tone === 'negative'
        ? 'text-negative'
        : tone === 'auto'
          ? direction === 'up'
            ? 'text-positive'
            : direction === 'down'
              ? 'text-negative'
              : undefined
          : undefined;

  return (
    <span
      data-money=""
      className={cx('num whitespace-nowrap', color, estimated && 'italic', className)}
      title={estimated ? t('componentes-valor-estimado') : undefined}
    >
      {trend && direction !== 'flat' ? (
        <span aria-hidden className="mr-[0.35em] inline-block align-[0.08em] text-[0.72em]">
          {direction === 'up' ? '▲' : '▼'}
        </span>
      ) : null}
      {estimated ? (
        <>
          <span aria-hidden>≈&#8239;</span>
          <span className="sr-only">{`${t('componentes-aproximadamente')} `}</span>
        </>
      ) : null}
      <Decode text={text} />
    </span>
  );
}

/**
 * Esconde no modo privacidade qualquer valor que não seja dinheiro (quantidade
 * de cotas, percentual do patrimônio, saldo em texto livre).
 */
export function Private({ children, className }: { children: ReactNode; className?: string }) {
  const privacy = usePrivacy();
  if (!privacy) return <>{children}</>;
  return (
    <span data-private="" className={cx('num whitespace-nowrap', className)}>
      <span aria-hidden>{PRIVACY_MASK}</span>
      <span className="sr-only">{t('componentes-valor-oculto')}</span>
    </span>
  );
}
