import { useRef, type KeyboardEvent, type ReactNode } from 'react';

import { Tooltip } from '@/design/components';
import { cx } from '@/design/cx';

export interface SegmentedOption<T extends string> {
  value: T;
  label: ReactNode;
  /** Dica ao passar o mouse/focar. */
  hint?: string;
}

export interface SegmentedProps<T extends string> {
  value: T;
  onChange: (value: T) => void;
  options: readonly SegmentedOption<T>[];
  /** Nome acessível do grupo. */
  label: string;
  size?: 'sm' | 'md';
  className?: string;
}

/**
 * Controle segmentado (grupo de rádio): ←/→ trocam a opção, só a marcada entra
 * na ordem do Tab. Visual de chave de instrumento.
 */
export function Segmented<T extends string>({
  value,
  onChange,
  options,
  label,
  size = 'md',
  className,
}: SegmentedProps<T>) {
  const refs = useRef(new Map<T, HTMLButtonElement>());
  const move = (event: KeyboardEvent, index: number) => {
    const delta =
      event.key === 'ArrowRight' || event.key === 'ArrowDown'
        ? 1
        : event.key === 'ArrowLeft' || event.key === 'ArrowUp'
          ? -1
          : 0;
    if (!delta && event.key !== 'Home' && event.key !== 'End') return;
    event.preventDefault();
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? options.length - 1
          : (index + delta + options.length) % options.length;
    const opt = options[next];
    if (!opt) return;
    onChange(opt.value);
    refs.current.get(opt.value)?.focus();
  };
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cx(
        'inline-flex items-center rounded-control border border-[var(--tab-border)] bg-[var(--tab-track)] p-0.5',
        className,
      )}
    >
      {options.map((o, i) => {
        const checked = o.value === value;
        const button = (
          <button
            key={o.value}
            ref={(el) => {
              if (el) refs.current.set(o.value, el);
              else refs.current.delete(o.value);
            }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={checked ? 0 : -1}
            onClick={() => onChange(o.value)}
            onKeyDown={(e) => move(e, i)}
            className={cx(
              'num rounded-[1px] px-2.5 font-medium tracking-[0.06em] whitespace-nowrap uppercase transition-colors duration-150',
              size === 'sm' ? 'h-6 text-[10.5px]' : 'h-7 text-[11px]',
              checked
                ? 'bg-[var(--tab-active)] text-accent shadow-[inset_0_-2px_0_var(--accent)]'
                : 'text-[var(--tab-text)] hover:bg-[var(--tab-hover)] hover:text-fg',
            )}
          >
            {o.label}
          </button>
        );
        return o.hint ? (
          <Tooltip key={o.value} content={o.hint} side="bottom">
            {button}
          </Tooltip>
        ) : (
          button
        );
      })}
    </div>
  );
}
