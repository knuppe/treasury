import type { ReactNode } from 'react';

import { cx } from '../cx';
import { t } from '@/i18n';

export interface EmptyStateProps {
  title: ReactNode;
  description?: ReactNode;
  /** Selo de fase do roadmap (`Fase 1 · Fluxo de caixa familiar`). */
  phase?: ReactNode;
  /** O que vai aparecer aqui (lista curta). */
  items?: readonly ReactNode[];
  /** Rótulo técnico acima do título (padrão "Sem dados"). */
  kicker?: ReactNode;
  actions?: ReactNode;
  className?: string;
}

/** Instrumento sem sinal: traço plano com marcas de escala. Decorativo. */
function FlatSignal() {
  return (
    <svg aria-hidden width="132" height="28" viewBox="0 0 132 28" className="text-line-strong">
      <path d="M0 27.5H132" stroke="currentColor" strokeOpacity=".6" />
      {Array.from({ length: 12 }, (_, i) => (
        <path
          key={i}
          d={`M${i * 12 + 0.5} 27.5V${i % 3 === 0 ? 21 : 24}`}
          stroke="currentColor"
          strokeOpacity=".6"
        />
      ))}
      <path
        d="M0 14H44l3-3 3 6 3-3H132"
        stroke="var(--accent)"
        strokeWidth="1.5"
        fill="none"
        strokeDasharray="3 3"
        strokeOpacity=".75"
      />
    </svg>
  );
}

/** Estado vazio: diz o que falta e quando chega, sem enfeite. */
export function EmptyState({
  title,
  description,
  phase,
  items,
  kicker = t('componentes-sem-dados'),
  actions,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cx(
        'flex flex-col items-start gap-4 rounded-card border border-dashed border-line-strong/60 bg-surface/40 px-6 py-7',
        className,
      )}
    >
      <div className="flex w-full flex-wrap items-center justify-between gap-3">
        <FlatSignal />
        {phase ? (
          <span className="label-caps rounded-control border border-accent/40 bg-accent-soft px-2 py-0.5 text-accent">
            {phase}
          </span>
        ) : null}
      </div>
      <div className="flex flex-col gap-1.5">
        <span className="label-caps">{kicker}</span>
        <h2 className="text-[17px] font-semibold tracking-[-0.005em] text-fg">{title}</h2>
        {description ? (
          <p className="max-w-[62ch] text-[13.5px] leading-relaxed text-dim">{description}</p>
        ) : null}
      </div>
      {items && items.length > 0 ? (
        <ul className="grid w-full gap-x-8 gap-y-1.5 text-[13px] text-fg sm:grid-cols-2">
          {items.map((item, i) => (
            <li key={i} className="flex items-baseline gap-2.5">
              <span aria-hidden className="num text-[11px] text-accent">
                {String(i + 1).padStart(2, '0')}
              </span>
              <span className="min-w-0">{item}</span>
            </li>
          ))}
        </ul>
      ) : null}
      {actions ? <div className="flex flex-wrap gap-2">{actions}</div> : null}
    </div>
  );
}
