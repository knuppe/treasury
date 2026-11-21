import { createContext, useContext, useId, type ReactNode } from 'react';

import { cx } from '../cx';
import { AutoHelp } from '../help';

interface FieldContextValue {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
}

const FieldContext = createContext<FieldContextValue | null>(null);

export interface FieldProps {
  label: ReactNode;
  children: ReactNode;
  /** Texto de ajuda abaixo do campo. */
  hint?: ReactNode;
  /** Mensagem de erro: marca o controle com `aria-invalid` e o descreve. */
  error?: ReactNode;
  /** Conteúdo à direita do rótulo (contador, medidor, link). */
  aside?: ReactNode;
  className?: string;
}

/**
 * Rótulo + controle + ajuda/erro, já ligados por `id`, `aria-describedby` e
 * `aria-invalid`. Os controles do design system (Input, MoneyInput, DateInput,
 * Select) leem este contexto sozinhos.
 */
export function Field({ label, children, hint, error, aside, className }: FieldProps) {
  const id = useId();
  const hintId = `${id}-hint`;
  const errorId = `${id}-error`;
  const describedBy = [hint ? hintId : null, error ? errorId : null].filter(Boolean).join(' ');
  return (
    <div className={cx('flex flex-col gap-1.5', className)}>
      <div className="flex min-h-4 min-w-0 items-baseline justify-between gap-3">
        {/* Uma linha só: um rótulo que quebra empurra o controle para baixo e desalinha a
            fileira de campos. O texto inteiro fica na dica. */}
        <span className="flex min-w-0 items-center gap-1.5">
          <label
            htmlFor={id}
            className="label-caps min-w-0 truncate"
            title={typeof label === 'string' ? label : undefined}
          >
            {label}
          </label>
          <AutoHelp label={label} />
        </span>
        {aside}
      </div>
      <FieldContext.Provider
        value={{ id, describedBy: describedBy || undefined, invalid: !!error }}
      >
        {children}
      </FieldContext.Provider>
      {hint ? (
        <p id={hintId} className="text-[12px] leading-snug text-dim">
          {hint}
        </p>
      ) : null}
      {error ? (
        <p id={errorId} className="flex items-start gap-1.5 text-[12px] leading-snug text-negative">
          <span aria-hidden className="num">
            ×
          </span>
          <span>{error}</span>
        </p>
      ) : null}
    </div>
  );
}

/** Props de acessibilidade vindas do `Field` em volta (se houver). */
export function useFieldControl(props: {
  id?: string;
  'aria-describedby'?: string;
  'aria-invalid'?: boolean | 'true' | 'false' | 'grammar' | 'spelling';
  invalid?: boolean;
}) {
  const field = useContext(FieldContext);
  const invalid = Boolean(props.invalid) || props['aria-invalid'] === true || !!field?.invalid;
  const describedBy =
    [field?.describedBy, props['aria-describedby']].filter(Boolean).join(' ') || undefined;
  return {
    id: props.id ?? field?.id,
    'aria-describedby': describedBy,
    'aria-invalid': invalid || undefined,
    invalid,
  };
}
