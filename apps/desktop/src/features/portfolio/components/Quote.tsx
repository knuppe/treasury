import type { PositionView } from '@/api/generated/PositionView';
import type { PriceOn } from '@/api/generated/PriceOn';
import { OriginBadge, Tooltip } from '@/design/components';
import { t } from '@/i18n';
import { formatDateTime, todayIso } from '@/lib/format';
import { SESSION_LABEL } from '@/lib/marketHours';

import { formatUnitPrice } from '../decimal';
import { sourceName } from '../labels';

/** Dias corridos entre duas datas ISO. */
function daysBetween(a: string, b: string): number {
  return Math.round((Date.parse(`${b}T00:00:00Z`) - Date.parse(`${a}T00:00:00Z`)) / 86_400_000);
}

/**
 * Cotação de outro dia é normal (o pregão de hoje ainda não fechou, fim de semana);
 * só vira "≈ estimada" quando tem mais de 5 dias de defasagem.
 */
export function isOldQuote(price: PriceOn, asOf?: string): boolean {
  if (!price.stale) return false;
  const ref = asOf ?? todayIso();
  return daysBetween(price.date, ref) > 5;
}

/** Cotação com selo de origem (fonte, data e se é de outro dia). */
export function PriceWithOrigin({
  price,
  currency,
  asOf,
}: {
  price: PriceOn;
  currency: string;
  asOf?: string;
}) {
  const stale = isOldQuote(price, asOf);
  const live = price.intraday;
  if (live) {
    const running = live.market_state === 'open' || live.market_state === 'lunch';
    return (
      <span className="inline-flex items-center justify-end gap-1">
        <span className={running ? 'italic' : undefined}>
          {formatUnitPrice(price.close, currency)}
        </span>
        <OriginBadge
          source={sourceName(price.source)}
          date={price.date}
          status={running ? 'estimado' : 'oficial'}
          note={t('carteira-cotacao-intradia', {
            quando: formatDateTime(new Date(live.quoted_at)),
            min: live.delay_minutes,
            sessao: SESSION_LABEL[live.market_state].toLowerCase(),
          })}
        />
      </span>
    );
  }
  return (
    <span className="inline-flex items-center justify-end gap-1">
      <span className={stale ? 'italic' : undefined}>
        {stale ? <span aria-hidden>≈&#8239;</span> : null}
        {formatUnitPrice(price.close, currency)}
      </span>
      <OriginBadge
        source={sourceName(price.source)}
        date={price.date}
        status={stale ? 'estimado' : 'oficial'}
        note={
          stale
            ? t('carteira-cotacao-defasada')
            : price.stale
              ? t('carteira-cotacao-ultimo-pregao')
              : t('carteira-cotacao-fechamento')
        }
      />
    </span>
  );
}

/** Célula "Cotação" da tabela de posições: mercado, curva ou sem cotação. */
export function QuoteCell({ position }: { position: PositionView }) {
  if (position.price)
    return <PriceWithOrigin price={position.price} currency={position.asset.currency} />;
  if (position.valuation === 'curve') {
    return (
      <span className="inline-flex items-center justify-end gap-1 text-dim">
        {t('carteira-na-curva')}
        <OriginBadge
          source={t('carteira-curva-fonte')}
          status={position.estimated ? 'estimado' : 'oficial'}
          note={t('carteira-curva-nota')}
        />
      </span>
    );
  }
  if (position.valuation === 'missing') {
    return (
      <Tooltip content={t('carteira-sem-cotacao-dica')}>
        <button type="button" className="inline-flex items-center gap-1 text-warning">
          {t('carteira-sem-cotacao')}
        </button>
      </Tooltip>
    );
  }
  return <span className="text-dim">—</span>;
}
