/** Peças do score de saúde por ativo: selo da faixa, barra de 0 a 100 e régua das faixas. */
import type { ReactNode } from 'react';

import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { BAND, BAND_ORDER, bandKey, bandOfScore, formatScore } from './format';

/**
 * Selo da faixa (sinal + texto, nunca só a cor). `partial`: o número existe, mas a faixa
 * ainda é "Dados insuficientes" (mais de 30% do peso imputado).
 */
export function BandChip({
  band,
  label,
  partial,
  size = 'sm',
  className,
}: {
  band: string;
  label?: string;
  partial?: boolean;
  size?: 'sm' | 'md';
  className?: string;
}) {
  const b = BAND[bandKey(band)];
  return (
    <span
      className={cx(
        'label-caps inline-flex shrink-0 items-center gap-1 rounded-[3px] border',
        size === 'md' ? 'px-2 py-0.5 text-[11px]' : 'px-1.5 py-px text-[10px]',
        b.chip,
        className,
      )}
    >
      <span aria-hidden>{b.mark}</span>
      {label ?? b.label}
      {partial ? <span className="normal-case">{t('score-chip-parcial')}</span> : null}
    </span>
  );
}

/** Barra de 0 a 100, na cor da faixa da própria nota (vazia sem nota). */
export function ScoreBar({
  score,
  label,
  muted,
}: {
  score: number | null;
  label: string;
  /** Nota imputada: barra tracejada/esmaecida. */
  muted?: boolean;
}) {
  const tone = BAND[bandOfScore(score)];
  return (
    <span
      role="img"
      aria-label={
        score == null
          ? t('score-barra-sem-nota', { rotulo: label })
          : t('score-barra-nota', { rotulo: label, nota: formatScore(score) })
      }
      className="block h-1.5 w-full overflow-hidden rounded-[2px] bg-line/50"
    >
      {score != null ? (
        <span
          className={cx('block h-full rounded-r-[3px]', tone.bar, muted && 'opacity-45')}
          style={{ width: `${Math.max(2, Math.min(100, score))}%` }}
        />
      ) : null}
    </span>
  );
}

/** Régua 0–100 com as quatro faixas e o marcador da nota. */
export function BandScale({ score, band }: { score: number | null; band: string }) {
  const current = bandKey(band);
  const bounds = BAND_ORDER.map((k, i) => {
    const min = BAND[k].min ?? 0;
    const max = i === 0 ? 100 : (BAND[BAND_ORDER[i - 1]!].min ?? 100) - 1;
    return { k, min, max };
  }).reverse();
  return (
    <div className="flex flex-col gap-1.5">
      <div className="relative pt-3">
        {score != null ? (
          <span
            aria-hidden
            className="absolute top-0 -translate-x-1/2 text-[10px] leading-none text-fg"
            style={{ left: `${Math.min(100, Math.max(0, score))}%` }}
          >
            {'▼'}
          </span>
        ) : null}
        <div
          role="img"
          aria-label={t('score-faixas-regua', {
            faixas: bounds.map((b) => `${BAND[b.k].label} ${b.min}–${b.max}`).join(', '),
          })}
          className="flex h-2 w-full gap-px overflow-hidden rounded-[2px]"
        >
          {bounds.map((b) => (
            <span
              key={b.k}
              title={`${BAND[b.k].label}: ${b.min}–${b.max}`}
              className={cx('h-full', BAND[b.k].bar)}
              style={{
                width: `${((b.max - b.min + 1) / 101) * 100}%`,
                // Sem faixa definitiva (parcial): todas a meia-luz.
                opacity: current === b.k ? 1 : current === 'dados_insuficientes' ? 0.55 : 0.28,
              }}
            />
          ))}
        </div>
      </div>
      <div aria-hidden className="num relative h-3 text-[10.5px] text-dim">
        {[0, 40, 60, 75, 100].map((tick) => (
          <span
            key={tick}
            className={cx(
              'absolute top-0',
              tick === 0 ? '' : tick === 100 ? '-translate-x-full' : '-translate-x-1/2',
            )}
            style={{ left: `${tick}%` }}
          >
            {tick}
          </span>
        ))}
      </div>
    </div>
  );
}

/** Selo pequeno e neutro (imputada, atratividade, crítica, defasado). */
export function Tag({
  children,
  tone = 'dim',
  title,
}: {
  children: ReactNode;
  tone?: 'dim' | 'warning' | 'accent' | 'negative';
  title?: string;
}) {
  return (
    <span
      title={title}
      className={cx(
        'label-caps inline-flex shrink-0 items-center rounded-[3px] border px-1.5 py-px text-[9.5px]',
        tone === 'warning' && 'border-warning/50 text-warning',
        tone === 'accent' && 'border-accent/50 text-accent',
        tone === 'negative' && 'border-negative/50 text-negative',
        tone === 'dim' && 'border-line-strong text-dim',
      )}
    >
      {children}
    </span>
  );
}
