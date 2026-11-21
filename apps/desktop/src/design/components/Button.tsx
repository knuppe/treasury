import type { ButtonHTMLAttributes, ReactNode, Ref } from 'react';

import { cx } from '../cx';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'demo';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Mostra o indicador de trabalho e desabilita o botão. */
  loading?: boolean;
  /** Texto alternativo enquanto `loading` (ex.: "Derivando chave…"). */
  loadingText?: ReactNode;
  icon?: ReactNode;
  iconRight?: ReactNode;
  fullWidth?: boolean;
  ref?: Ref<HTMLButtonElement>;
}

const BASE =
  'relative inline-flex select-none items-center justify-center gap-2 whitespace-nowrap rounded-control font-mono font-medium uppercase tracking-[0.08em] transition-[background-color,border-color,color,box-shadow,transform] duration-150 ease-instrument active:translate-y-px disabled:cursor-not-allowed disabled:opacity-45';

const VARIANTS: Record<ButtonVariant, string> = {
  primary:
    'bg-accent text-accent-ink shadow-[inset_0_1px_0_rgb(255_255_255/0.22),0_1px_2px_rgb(0_0_0/0.18),0_4px_14px_-6px_var(--accent)] hover:bg-[color-mix(in_srgb,var(--accent)_86%,var(--text))] active:bg-[color-mix(in_srgb,var(--accent)_78%,var(--text))] disabled:hover:bg-accent',
  secondary:
    'border border-line-strong/70 bg-surface-2 text-fg shadow-[var(--highlight)] hover:border-accent hover:text-accent disabled:hover:border-line-strong/70 disabled:hover:text-fg',
  ghost: 'text-dim hover:bg-surface-2 hover:text-fg disabled:hover:bg-transparent',
  danger:
    'border border-negative/70 text-negative hover:bg-negative/12 disabled:hover:bg-transparent',
  /** Cofre de demonstração: cheio, na cor própria de cada tema (--demo), para se destacar. */
  demo: 'bg-demo text-demo-ink shadow-[inset_0_1px_0_rgb(255_255_255/0.22),0_1px_2px_rgb(0_0_0/0.18),0_4px_14px_-6px_var(--demo)] hover:bg-[color-mix(in_srgb,var(--demo)_86%,var(--text))] active:bg-[color-mix(in_srgb,var(--demo)_78%,var(--text))] disabled:hover:bg-demo',
};

const SIZES: Record<ButtonSize, string> = {
  sm: 'h-7 px-2.5 text-[10.5px]',
  md: 'h-9 px-3.5 text-[11.5px]',
  lg: 'h-11 px-5 text-[12.5px]',
};

/** Botão do Treasury: mono, caixa alta (via CSS), cinco variantes. */
export function Button({
  variant = 'secondary',
  size = 'md',
  loading = false,
  loadingText,
  icon,
  iconRight,
  fullWidth,
  className,
  children,
  disabled,
  type = 'button',
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={Boolean(disabled) || loading}
      aria-busy={loading || undefined}
      className={cx(BASE, VARIANTS[variant], SIZES[size], fullWidth && 'w-full', className)}
      {...rest}
    >
      {loading ? (
        <span aria-hidden className="inline-flex gap-[3px]">
          <span className="size-1 animate-pulse-dot rounded-full bg-current" />
          <span className="size-1 animate-pulse-dot rounded-full bg-current [animation-delay:160ms]" />
          <span className="size-1 animate-pulse-dot rounded-full bg-current [animation-delay:320ms]" />
        </span>
      ) : (
        icon
      )}
      <span>{loading && loadingText ? loadingText : children}</span>
      {!loading && iconRight}
    </button>
  );
}
