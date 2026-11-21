import type { ReactNode } from 'react';

import { cx } from '../cx';
import { t } from '@/i18n';

export interface FigureProps {
  /** Número da figura: `4` vira `FIG. 04`. */
  number: number | string;
  title: ReactNode;
  children: ReactNode;
  /** Legenda das séries, à direita da legenda. */
  legend?: ReactNode;
  /** Nota de origem abaixo do gráfico (`FONTE · IBGE/SIDRA 1737`). */
  source?: ReactNode;
  className?: string;
}

/** Figura com legenda numerada: `FIG. 04 · PATRIMÔNIO LÍQUIDO, 24 MESES`. */
export function Figure({ number, title, children, legend, source, className }: FigureProps) {
  const label = typeof number === 'number' ? String(number).padStart(2, '0') : number;
  return (
    <figure className={cx('flex min-w-0 flex-col gap-3', className)}>
      <figcaption className="flex flex-wrap items-baseline gap-x-2 gap-y-1 border-b border-line pb-2">
        <span className="label-caps text-accent">{t('componentes-figura', { numero: label })}</span>
        <span aria-hidden className="label-caps">
          ·
        </span>
        <span className="label-caps text-fg">{title}</span>
        {legend ? <span className="ml-auto flex items-center gap-3">{legend}</span> : null}
      </figcaption>
      <div className="min-w-0">{children}</div>
      {source ? <p className="label-caps">{source}</p> : null}
    </figure>
  );
}

/** Item de legenda de série: quadradinho na cor da série + nome. */
export function LegendItem({
  color = 'var(--accent)',
  dashed,
  children,
}: {
  color?: string;
  dashed?: boolean;
  children: ReactNode;
}) {
  return (
    <span className="label-caps inline-flex items-center gap-1.5">
      <span
        aria-hidden
        className="inline-block h-[2px] w-3"
        style={
          dashed
            ? {
                backgroundImage: `linear-gradient(90deg, ${color} 60%, transparent 0)`,
                backgroundSize: '4px 2px',
              }
            : { background: color }
        }
      />
      {children}
    </span>
  );
}
