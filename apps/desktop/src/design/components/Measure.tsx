import type { ReactNode } from 'react';

import { cx } from '../cx';
import { AutoHelp } from '../help';
import { OriginBadge, type Origin } from './OriginBadge';
import { useFitWidth } from './useFitWidth';

export interface MeasureProps {
  /** Rótulo pequeno acima da medida (`PATRIMÔNIO LÍQUIDO`). */
  label: ReactNode;
  /** A medida: normalmente um <Money/> ou um percentual formatado. */
  value: ReactNode;
  /** 32px, 40px ou 48px (padrão 40px). */
  size?: 'md' | 'lg' | 'xl';
  /** Variação abaixo da medida (ex.: `<Money tone="auto" trend …/>`). */
  delta?: ReactNode;
  /** Texto de apoio ao lado da variação (`em 12 meses`). */
  hint?: ReactNode;
  /** Selo de origem ao lado do rótulo. */
  origin?: Origin;
  /**
   * Quando muda, o número "chega" com uma transição curta (desligada com
   * movimento reduzido). Passe o valor bruto, ex.: os centavos.
   */
  changeKey?: string | number;
  align?: 'left' | 'right';
  className?: string;
}

const SIZES = {
  md: 'text-[length:calc(var(--measure-md)*var(--fit,1))]',
  lg: 'text-[length:calc(var(--measure-lg)*var(--fit,1))]',
  xl: 'text-[length:calc(var(--measure-xl)*var(--fit,1))]',
};

/** Medida principal de painel: rótulo técnico + número grande mono. */
export function Measure({
  label,
  value,
  size = 'lg',
  delta,
  hint,
  origin,
  changeKey,
  align = 'left',
  className,
}: MeasureProps) {
  const { box, content } = useFitWidth(changeKey);
  return (
    <dl className={cx('flex min-w-0 flex-col gap-2', align === 'right' && 'items-end', className)}>
      <dt className="label-caps flex items-center gap-1.5">
        {label}
        <AutoHelp label={label} />
        {origin ? <OriginBadge {...origin} /> : null}
      </dt>
      <dd
        ref={box}
        className={cx(
          'num max-w-full leading-none font-medium tracking-[-0.02em] whitespace-nowrap text-fg',
          SIZES[size],
        )}
      >
        <span key={changeKey} ref={content} className="inline-block animate-arrive">
          {value}
        </span>
      </dd>
      {delta || hint ? (
        <dd className="flex items-baseline gap-2 text-[12.5px]">
          {delta}
          {hint ? <span className="text-dim">{hint}</span> : null}
        </dd>
      ) : null}
    </dl>
  );
}
