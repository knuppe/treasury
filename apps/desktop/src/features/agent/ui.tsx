/** Peças pequenas das telas do agente: selo, interruptor e bloco de valor bruto. */
import type { ReactNode } from 'react';

import { cx } from '@/design/cx';

export type PillTone = 'accent' | 'positive' | 'negative' | 'warning' | 'dim';

const PILL: Record<PillTone, string> = {
  accent: 'border-accent/50 text-accent',
  positive: 'border-positive/50 text-positive',
  negative: 'border-negative/50 text-negative',
  warning: 'border-warning/50 text-warning',
  dim: 'border-line-strong text-dim',
};

/** Selo em caixa alta (nível, estado, tipo de motor). */
export function Pill({
  tone = 'dim',
  children,
  className,
  title,
}: {
  tone?: PillTone;
  children: ReactNode;
  className?: string;
  title?: string;
}) {
  return (
    <span
      title={title}
      className={cx(
        'inline-flex h-5 shrink-0 items-center gap-1 rounded-sm border px-1.5 font-mono text-[10px] font-medium tracking-[0.08em] whitespace-nowrap uppercase',
        PILL[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}

/** Caixa de seleção com rótulo e ajuda (ajustes liga/desliga). */
export function Toggle({
  label,
  hint,
  checked,
  onChange,
  disabled,
  className,
}: {
  label: ReactNode;
  hint?: ReactNode;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <label
      className={cx(
        'flex cursor-pointer items-start gap-2.5 text-[13px] text-fg',
        disabled && 'cursor-not-allowed opacity-60',
        className,
      )}
    >
      <input
        type="checkbox"
        className="mt-0.5 size-3.5 accent-[var(--accent)]"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className="flex flex-col gap-0.5">
        <span>{label}</span>
        {hint ? <span className="text-[12px] leading-snug text-dim">{hint}</span> : null}
      </span>
    </label>
  );
}

/** Texto técnico (parâmetros, templates) em mono, quebrando linha. */
export function Mono({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <code
      className={cx('font-mono text-[11.5px] break-all whitespace-pre-wrap text-fg', className)}
    >
      {children}
    </code>
  );
}

/** Rótulo pequeno em caixa alta seguido de conteúdo, numa linha. */
export function Spec({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <span className="inline-flex items-baseline gap-1.5">
      <span className="label-caps text-[10px]">{label}</span>
      <span className="num text-[11.5px] text-fg">{children}</span>
    </span>
  );
}
