/**
 * Descrição com autocompletar: sugere lançamentos parecidos da história (sem acento, por
 * início de palavra) e, ao escolher, preenche categoria, favorecido e valor que ainda
 * estiverem vazios. Teclado: ↑/↓ escolhe, Enter aplica, Esc fecha.
 */
import { useEffect, useId, useRef, useState } from 'react';

import { call } from '@/api/client';
import type { TxnSuggestion } from '@/api/generated/TxnSuggestion';
import { Input, Money } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

export function DescriptionInput({
  value,
  onChange,
  onPick,
  accountId,
  kind,
  placeholder,
  autoFocus,
  enabled = true,
}: {
  value: string;
  onChange: (v: string) => void;
  onPick: (s: TxnSuggestion) => void;
  accountId?: string;
  kind?: 'expense' | 'income';
  placeholder?: string;
  autoFocus?: boolean;
  /** Desligado ao editar um lançamento existente. */
  enabled?: boolean;
}) {
  const [items, setItems] = useState<TxnSuggestion[]>([]);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const typed = useRef(false);
  const listId = useId();

  useEffect(() => {
    if (!enabled || !typed.current || value.trim().length < 2) return;
    let alive = true;
    const timer = setTimeout(() => {
      void call<TxnSuggestion[]>('transactions.suggest', {
        text: value,
        ...(accountId ? { account_id: accountId } : {}),
        ...(kind ? { kind } : {}),
        limit: 6,
      })
        .then((r) => {
          if (!alive) return;
          setItems(r);
          setActive(0);
          setOpen(r.length > 0);
        })
        .catch(() => alive && setOpen(false));
    }, 150);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [value, accountId, kind, enabled]);

  const pick = (s: TxnSuggestion) => {
    typed.current = false;
    setOpen(false);
    onPick(s);
  };

  return (
    <div className="relative">
      <Input
        value={value}
        placeholder={placeholder}
        autoFocus={autoFocus}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={open}
        aria-controls={listId}
        aria-activedescendant={open ? `${listId}-${active}` : undefined}
        onChange={(e) => {
          typed.current = true;
          if (e.target.value.trim().length < 2) setOpen(false);
          onChange(e.target.value);
        }}
        onBlur={() => setTimeout(() => setOpen(false), 120)}
        onKeyDown={(e) => {
          if (!open) return;
          if (e.key === 'ArrowDown') {
            e.preventDefault();
            setActive((a) => Math.min(a + 1, items.length - 1));
          } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setActive((a) => Math.max(a - 1, 0));
          } else if (e.key === 'Enter' && items[active]) {
            e.preventDefault();
            pick(items[active]);
          } else if (e.key === 'Escape') {
            e.stopPropagation();
            setOpen(false);
          }
        }}
      />
      {open ? (
        <ul
          id={listId}
          role="listbox"
          aria-label={t('controles-sugestoes')}
          className="absolute top-full right-0 left-0 z-50 mt-1 max-h-64 overflow-auto rounded-control border border-line bg-surface-2 py-1 shadow-popover"
        >
          {items.map((s, i) => (
            <li
              key={`${s.description}-${i}`}
              id={`${listId}-${i}`}
              role="option"
              aria-selected={i === active}
              onMouseDown={(e) => {
                e.preventDefault();
                pick(s);
              }}
              onMouseEnter={() => setActive(i)}
              className={cx(
                'flex cursor-default items-center justify-between gap-3 px-3 py-1.5 text-[13px]',
                i === active ? 'bg-accent-soft text-accent' : 'text-fg',
              )}
            >
              <span className="min-w-0">
                <span className="block truncate">{s.description}</span>
                <span className="block truncate text-[11.5px] text-dim">
                  {s.category_path ?? t('controles-sem-categoria')}
                  {` · ${s.count}×`}
                </span>
              </span>
              <span className="num shrink-0 text-[12px] text-dim">
                <Money cents={s.last_amount} />
              </span>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
