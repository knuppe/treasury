import * as RSelect from '@radix-ui/react-select';
import type { ReactNode } from 'react';

import { cx } from '../cx';
import { useFieldControl } from './Field';
import { Icon } from './Icon';
import { CONTROL_FRAME } from './Input';

export interface SelectOption {
  value: string;
  label: ReactNode;
  /** Texto curto à direita da opção (ex.: código da moeda). */
  hint?: ReactNode;
  disabled?: boolean;
}

export interface SelectProps {
  value: string | undefined;
  onValueChange: (value: string) => void;
  options: readonly SelectOption[];
  placeholder?: string;
  disabled?: boolean;
  invalid?: boolean;
  id?: string;
  'aria-label'?: string;
  className?: string;
}

/** Seleção (Radix Select): teclado completo, digitar para buscar, leitores de tela. */
export function Select({
  value,
  onValueChange,
  options,
  placeholder = 'Selecione…',
  disabled,
  invalid,
  id,
  className,
  'aria-label': ariaLabel,
}: SelectProps) {
  const control = useFieldControl({ id, invalid });
  return (
    // Sempre controlado: sem valor ainda, '' mostra o placeholder. Passar `undefined` e depois
    // um valor fazia o Radix avisar "Select is changing from uncontrolled to controlled".
    <RSelect.Root value={value ?? ''} onValueChange={onValueChange} disabled={disabled}>
      <RSelect.Trigger
        id={control.id}
        aria-label={ariaLabel}
        aria-describedby={control['aria-describedby']}
        aria-invalid={control['aria-invalid']}
        className={cx(
          CONTROL_FRAME,
          'justify-between text-left text-[13.5px] text-fg outline-none focus-visible:border-accent focus-visible:shadow-[0_0_0_1px_var(--accent)] data-[placeholder]:text-dim disabled:cursor-not-allowed disabled:opacity-50',
          control.invalid ? 'border-negative' : 'border-line-strong',
          className,
        )}
      >
        <span className="min-w-0 truncate">
          <RSelect.Value placeholder={placeholder} />
        </span>
        <RSelect.Icon className="text-dim">
          <Icon name="chevronDown" size={14} />
        </RSelect.Icon>
      </RSelect.Trigger>
      <RSelect.Portal>
        <RSelect.Content
          position="popper"
          sideOffset={4}
          collisionPadding={8}
          className="z-[60] max-h-[min(320px,var(--radix-select-content-available-height))] min-w-[var(--radix-select-trigger-width)] animate-pop-in overflow-hidden rounded-card border border-panel pop-surface"
        >
          <RSelect.Viewport className="p-1">
            {options.map((option) => (
              <RSelect.Item
                key={option.value}
                value={option.value}
                disabled={option.disabled}
                className="relative flex h-8 cursor-default select-none items-center gap-3 rounded-control pr-2 pl-7 text-[13px] text-fg outline-none data-[disabled]:opacity-40 data-[highlighted]:bg-accent-soft data-[highlighted]:text-accent"
              >
                <RSelect.ItemIndicator className="absolute left-2 inline-flex">
                  <Icon name="check" size={13} />
                </RSelect.ItemIndicator>
                <RSelect.ItemText>{option.label}</RSelect.ItemText>
                {option.hint ? (
                  <span className="num ml-auto text-[11px] text-dim">{option.hint}</span>
                ) : null}
              </RSelect.Item>
            ))}
          </RSelect.Viewport>
        </RSelect.Content>
      </RSelect.Portal>
    </RSelect.Root>
  );
}
