import { formatDate } from '@/lib/format';
import { t } from '@/i18n';

import { cx } from '../cx';
import { Icon } from './Icon';
import { KeyValue } from './KeyValue';
import { Tooltip } from './Tooltip';

export type OriginStatus = 'oficial' | 'estimado';

export interface Origin {
  /** De onde veio o número: `BCB/SGS 433`, `B3 COTAHIST`, `Lançamento manual`. */
  source: string;
  /** Data de referência (ISO `YYYY-MM-DD`). */
  date?: string;
  /** Oficial (publicado pela fonte) ou estimado (projeção, último dado). */
  status?: OriginStatus;
  /** Detalhe curto (ex.: "Projeção Focus de 10/10/2026"). */
  note?: string;
}

export interface OriginBadgeProps extends Origin {
  className?: string;
}

function safeDate(iso: string | undefined): string | undefined {
  if (!iso) return undefined;
  try {
    return formatDate(iso);
  } catch {
    return iso;
  }
}

/**
 * Selo de origem: ícone discreto ao lado de um número. Passar o mouse ou focar
 * mostra fonte, data e se o valor é oficial ou estimado. "Medido, nunca inventado."
 */
export function OriginBadge({
  source,
  date,
  status = 'oficial',
  note,
  className,
}: OriginBadgeProps) {
  const estimated = status === 'estimado';
  const dateText = safeDate(date);
  const spoken = [
    t('componentes-origem-falada', { fonte: source }),
    dateText ? t('componentes-origem-data-falada', { data: dateText }) : null,
    estimated ? t('componentes-valor-estimado-falado') : t('componentes-valor-oficial-falado'),
  ]
    .filter(Boolean)
    .join(', ');

  return (
    <Tooltip
      side="top"
      content={
        <div className="flex min-w-48 flex-col gap-2 py-0.5">
          <span className="label-caps text-fg">{t('componentes-origem-titulo')}</span>
          <KeyValue
            layout="stacked"
            items={[
              { label: t('componentes-fonte'), value: source },
              ...(dateText ? [{ label: t('componentes-data'), value: dateText }] : []),
              {
                label: t('componentes-status'),
                value: estimated ? (
                  <span className="text-warning italic">{t('componentes-estimado')}</span>
                ) : (
                  <span className="text-positive">{t('componentes-oficial')}</span>
                ),
              },
            ]}
          />
          {note ? <p className="max-w-60 text-[12px] leading-snug text-dim">{note}</p> : null}
        </div>
      }
    >
      <button
        type="button"
        aria-label={spoken}
        data-origin-status={status}
        className={cx(
          'inline-flex size-4 shrink-0 items-center justify-center rounded-full align-middle transition-colors',
          estimated ? 'text-warning/80 hover:text-warning' : 'text-dim/70 hover:text-accent',
          className,
        )}
      >
        <Icon name={estimated ? 'reticleDashed' : 'reticle'} size={12} />
      </button>
    </Tooltip>
  );
}
