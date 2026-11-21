import { formatShortcut } from '@/lib/platform';

import { cx } from '../cx';

export interface KbdProps {
  /** Teclas, ex.: `['mod', 'K']` → `⌘K` (macOS) ou `Ctrl+K` (Linux). */
  keys: readonly string[];
  /** `onAccent`: dentro de um botão primário (tinta do destaque, não o cinza dos rótulos). */
  tone?: 'default' | 'onAccent';
  className?: string;
}

/** Atalho de teclado no estilo técnico. */
export function Kbd({ keys, tone = 'default', className }: KbdProps) {
  return (
    <kbd
      className={cx(
        'num inline-flex h-5 min-w-5 items-center justify-center rounded-control border px-1 text-[10px] leading-none tracking-normal',
        tone === 'onAccent' ? 'border-accent-ink/40 text-accent-ink' : 'border-line text-dim',
        className,
      )}
    >
      {formatShortcut(keys)}
    </kbd>
  );
}
