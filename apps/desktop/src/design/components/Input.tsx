import type { InputHTMLAttributes, ReactNode, Ref } from 'react';

import { cx } from '../cx';
import { useFieldControl } from './Field';

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'prefix'> {
  /** Conteúdo fixo à esquerda (ex.: `R$`). */
  prefix?: ReactNode;
  /** Conteúdo à direita (ex.: botão de mostrar senha). */
  suffix?: ReactNode;
  invalid?: boolean;
  /** Texto mono com algarismos tabulares, alinhado à direita. */
  numeric?: boolean;
  /** Classes do contêiner (borda); `className` vai para o <input>. */
  wrapperClassName?: string;
  ref?: Ref<HTMLInputElement>;
}

export const CONTROL_FRAME =
  'flex h-9 w-full items-center gap-2 rounded-control border bg-bg px-2.5 transition-[border-color,box-shadow] duration-150 focus-within:border-accent focus-within:shadow-[0_0_0_3px_var(--focus-ring)]';

export function Input({
  prefix,
  suffix,
  invalid,
  numeric,
  wrapperClassName,
  className,
  id,
  disabled,
  'aria-describedby': ariaDescribedBy,
  'aria-invalid': ariaInvalid,
  ...rest
}: InputProps) {
  const control = useFieldControl({
    id,
    invalid,
    'aria-describedby': ariaDescribedBy,
    'aria-invalid': ariaInvalid,
  });
  return (
    <div
      className={cx(
        CONTROL_FRAME,
        control.invalid ? 'border-negative' : 'border-line-strong',
        disabled && 'opacity-50',
        wrapperClassName,
      )}
    >
      {prefix ? (
        <span className="num shrink-0 select-none text-[12px] text-dim">{prefix}</span>
      ) : null}
      <input
        id={control.id}
        aria-describedby={control['aria-describedby']}
        aria-invalid={control['aria-invalid']}
        disabled={disabled}
        className={cx(
          'h-full min-w-0 flex-1 bg-transparent text-[13.5px] text-fg outline-none disabled:cursor-not-allowed',
          numeric && 'num text-right',
          className,
        )}
        {...rest}
      />
      {suffix ? <span className="-mr-1.5 flex shrink-0 items-center">{suffix}</span> : null}
    </div>
  );
}
