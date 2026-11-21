import type { ButtonHTMLAttributes, ReactNode, Ref } from 'react';

import { cx } from '../cx';
import { Kbd } from './Kbd';
import { Tooltip } from './Tooltip';

export interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  /** Nome acessível e texto da dica. Obrigatório: o botão só tem ícone. */
  label: string;
  icon: ReactNode;
  /** Atalho mostrado na dica, ex.: `['mod', 'shift', 'H']`. */
  shortcut?: readonly string[];
  /** Botão de alternância: define `aria-pressed`. */
  pressed?: boolean;
  variant?: 'ghost' | 'secondary';
  size?: 'sm' | 'md';
  /** `false` desliga a dica (o nome acessível continua). */
  tooltip?: boolean;
  tooltipSide?: 'top' | 'right' | 'bottom' | 'left';
  ref?: Ref<HTMLButtonElement>;
}

export function IconButton({
  label,
  icon,
  shortcut,
  pressed,
  variant = 'ghost',
  size = 'md',
  tooltip = true,
  tooltipSide = 'bottom',
  className,
  type = 'button',
  ...rest
}: IconButtonProps) {
  const button = (
    <button
      type={type}
      aria-label={label}
      aria-pressed={pressed}
      className={cx(
        'inline-flex shrink-0 items-center justify-center rounded-control transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-45',
        size === 'sm' ? 'size-7' : 'size-8',
        variant === 'secondary'
          ? 'border border-line-strong bg-surface-2 text-fg hover:border-accent hover:text-accent'
          : 'text-dim hover:bg-surface-2 hover:text-fg',
        pressed && 'bg-accent-soft text-accent hover:bg-accent-soft hover:text-accent',
        className,
      )}
      {...rest}
    >
      {icon}
    </button>
  );
  if (!tooltip) return button;
  return (
    <Tooltip
      side={tooltipSide}
      content={
        <span className="flex items-center gap-2">
          {label}
          {shortcut ? <Kbd keys={shortcut} /> : null}
        </span>
      }
    >
      {button}
    </Tooltip>
  );
}
