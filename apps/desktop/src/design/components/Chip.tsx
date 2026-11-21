import type { ComponentPropsWithoutRef, ReactNode } from 'react';

import { cx } from '../cx';

/**
 * Chip de escolha (atalhos e filtros): botão arredondado com borda; marcado (`pressed`), ganha
 * a borda, o fundo e o texto do destaque (`.chip` em styles.css, tokens `--tab-*`).
 */
export function Chip({
  pressed,
  className,
  type = 'button',
  ...props
}: ComponentPropsWithoutRef<'button'> & { pressed: boolean }) {
  return <button type={type} aria-pressed={pressed} className={cx('chip', className)} {...props} />;
}

/** Grupo de chips com nome acessível. */
export function ChipGroup({
  label,
  className,
  children,
}: {
  label: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <div role="group" aria-label={label} className={cx('flex flex-wrap gap-1.5', className)}>
      {children}
    </div>
  );
}
