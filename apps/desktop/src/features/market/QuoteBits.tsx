/**
 * Peças da cotação intradiária (Yahoo, ~15 min de atraso): o selo de atraso com a situação
 * do pregão e a variação do dia. Preços e variações somem no modo privacidade.
 */
import type { MarketQuote } from '@/api/generated/MarketQuote';
import type { SessionState } from '@/api/generated/SessionState';
import { Private, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { localTime, SESSION_LABEL } from '@/lib/marketHours';

import { Pct } from '../portfolio/components/Pct';
import { formatUnitPrice } from '../portfolio/decimal';

const STATE_TONE: Record<SessionState, string> = {
  open: 'text-positive',
  lunch: 'text-warning',
  pre: 'text-dim',
  post: 'text-dim',
  closed: 'text-dim',
};

/** `≈15 min` com a situação do pregão; a dica traz o horário da cotação e a fonte. */
export function DelayBadge({
  delayMinutes = 15,
  state,
  quotedAt,
  timezone,
  compact = false,
  className,
}: {
  delayMinutes?: number;
  state?: SessionState | null;
  quotedAt?: string | null;
  timezone?: string | null;
  compact?: boolean;
  className?: string;
}) {
  const at = quotedAt ? new Date(quotedAt) : null;
  const text =
    delayMinutes > 0
      ? compact
        ? `≈${delayMinutes} min`
        : t('mercado-atraso', { min: delayMinutes })
      : t('mercado-agora');
  return (
    <Tooltip
      side="top"
      content={
        <div className="flex max-w-64 flex-col gap-1 py-0.5 text-[12px]">
          <span className="label-caps text-fg">{t('mercado-intradiaria')}</span>
          {state ? <span className={STATE_TONE[state]}>{SESSION_LABEL[state]}</span> : null}
          {at ? (
            <span className="text-dim">
              {timezone
                ? t('mercado-ultimo-negocio-bolsa', {
                    quando: formatDateTime(at),
                    hora: localTime(timezone, at),
                  })
                : t('mercado-ultimo-negocio', { quando: formatDateTime(at) })}
            </span>
          ) : null}
          <span className="text-dim">{t('mercado-fonte-yahoo', { min: delayMinutes || 0 })}</span>
        </div>
      }
    >
      <button
        type="button"
        data-testid="delay-badge"
        aria-label={
          state
            ? t('mercado-cotacao-com-estado', {
                atraso: text,
                estado: SESSION_LABEL[state].toLowerCase(),
              })
            : t('mercado-cotacao-com', { atraso: text })
        }
        className={cx(
          'inline-flex items-center gap-1 rounded-full border border-line px-1.5 py-px text-[10.5px] leading-tight whitespace-nowrap text-dim',
          className,
        )}
      >
        {state ? (
          <span
            aria-hidden
            className={cx(
              'size-1.5 rounded-full',
              state === 'open' ? 'bg-positive' : state === 'lunch' ? 'bg-warning' : 'bg-dim/60',
            )}
          />
        ) : null}
        {text}
      </button>
    </Tooltip>
  );
}

/** Último preço, variação do dia e o selo de atraso (tabelas de posições e watchlist). */
export function QuoteChange({
  quote,
  showPrice = true,
  className,
}: {
  quote: MarketQuote | null | undefined;
  showPrice?: boolean;
  className?: string;
}) {
  if (!quote) return <span className="text-dim">—</span>;
  return (
    <span className={cx('inline-flex items-center justify-end gap-1.5', className)}>
      {showPrice ? (
        <Private>
          <span className="num">
            {formatUnitPrice(quote.price, quote.currency, { maxDecimals: 4 })}
          </span>
        </Private>
      ) : null}
      <Pct value={quote.change_pct} tone="auto" decimals={2} />
      <DelayBadge
        compact
        delayMinutes={quote.delay_minutes}
        state={quote.market_state}
        quotedAt={quote.market_time}
        timezone={quote.timezone}
      />
    </span>
  );
}
