import type { ComponentPropsWithoutRef, ElementType } from 'react';

import { cx } from '../cx';

export interface LabelProps extends ComponentPropsWithoutRef<'span'> {
  /** Elemento HTML a renderizar (padrão `span`). */
  as?: ElementType;
  /** Cor: `dim` (padrão, rótulo), `fg` (texto principal) ou `accent`. */
  tone?: 'dim' | 'fg' | 'accent';
  htmlFor?: string;
}

/**
 * Rótulo técnico: 11px, mono, caixa alta, `letter-spacing: 0.12em`, `--text-dim`.
 * Escreva o texto em caixa normal: a caixa alta vem do CSS, para leitores de
 * tela não soletrarem as palavras.
 */
export function Label({ as, tone = 'dim', className, ...rest }: LabelProps) {
  const Comp: ElementType = as ?? 'span';
  return (
    <Comp
      className={cx(
        'label-caps',
        tone === 'fg' && 'text-fg',
        tone === 'accent' && 'text-accent',
        className,
      )}
      {...rest}
    />
  );
}
