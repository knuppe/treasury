import { useId, useMemo, useRef, useState } from 'react';

import type { Category } from '@/api/generated/Category';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import { useCategories } from '@/api/hooks/categories';
import { normalizeSearch } from '@/commands/registry';
import { Icon } from '@/design/components';
import { cx } from '@/design/cx';
import type { Point } from '@/design/pointer';
import { t } from '@/i18n';

import { Popover } from './Popover';

const NONE = '__none__';

/**
 * Escolha de categoria com busca (combobox): digite parte do caminho
 * ("merc" → Alimentação › Mercado), ↑/↓ para navegar e Enter para escolher.
 */
export function CategoryPicker({
  value,
  onChange,
  kind,
  label = t('controles-categoria'),
  allowNone = true,
  noneLabel = t('controles-sem-categoria'),
  compact,
  disabled,
  open,
  onOpenChange,
  className,
  anchor,
}: {
  value: string | null;
  onChange: (id: string | null) => void;
  /** Só categorias desse tipo (receita ou despesa). */
  kind?: CategoryKind | null;
  label?: string;
  allowNone?: boolean;
  noneLabel?: string;
  /** Visual de célula de tabela. */
  compact?: boolean;
  disabled?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  className?: string;
  /** Abre o painel neste ponto da tela (ex.: no mouse, pelo atalho `c`). */
  anchor?: Point | null;
}) {
  const { data: categories = [] } = useCategories(kind ? { kind } : {});
  const current = categories.find((c) => c.id === value);
  const [search, setSearch] = useState('');
  const [active, setActive] = useState(0);
  const [openInternal, setOpenInternal] = useState(false);
  const isOpen = open ?? openInternal;
  const input = useRef<HTMLInputElement>(null);
  const listId = useId();

  const setOpen = (next: boolean) => {
    if (open === undefined) setOpenInternal(next);
    onOpenChange?.(next);
    if (!next) setSearch('');
  };

  const items = useMemo(() => {
    const q = normalizeSearch(search.trim());
    const list: (Category | null)[] = categories.filter(
      (c) => !q || normalizeSearch(c.path).includes(q),
    );
    return allowNone && !q ? [null, ...list] : list;
  }, [categories, search, allowNone]);

  const choose = (c: Category | null) => {
    onChange(c ? c.id : null);
    setOpen(false);
  };

  const text = current ? current.path : value ? '…' : noneLabel;

  if (disabled) {
    return <span className={cx('text-[13px] text-dim', className)}>{text}</span>;
  }

  return (
    <Popover
      className={className}
      open={isOpen}
      onOpenChange={(o) => {
        setOpen(o);
        setActive(0);
      }}
      initialFocus={input}
      ariaLabel={`${label}: ${text}`}
      // Larga o bastante para "Resgate de investimentos › Previdência privada"; nomes ainda
      // maiores quebram a linha em vez de cortar.
      panelClassName="w-[min(28rem,calc(100vw-16px))]"
      variant={compact ? 'bare' : 'button'}
      anchor={anchor}
      label={
        compact ? (
          <span className={cx('min-w-0 truncate', !current && 'text-dim italic')}>{text}</span>
        ) : (
          <>
            <span className={cx('min-w-0 flex-1 truncate text-left', !current && 'text-dim')}>
              {text}
            </span>
            <Icon name="chevronDown" size={12} className="text-dim" />
          </>
        )
      }
    >
      <div className="flex flex-col gap-1.5">
        <input
          ref={input}
          role="combobox"
          aria-expanded
          aria-controls={listId}
          aria-activedescendant={items[active] !== undefined ? `${listId}-${active}` : undefined}
          aria-label={t('controles-buscar-rotulo', { rotulo: label.toLowerCase() })}
          placeholder={t('controles-buscar-categoria')}
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setActive(0);
          }}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') {
              e.preventDefault();
              setActive((i) => Math.min(items.length - 1, i + 1));
            } else if (e.key === 'ArrowUp') {
              e.preventDefault();
              setActive((i) => Math.max(0, i - 1));
            } else if (e.key === 'Enter') {
              e.preventDefault();
              const item = items[active];
              if (item !== undefined) choose(item);
            }
          }}
          className="h-8 rounded-control border border-line-strong bg-bg px-2 text-[13px] text-fg outline-none focus:border-accent"
        />
        <ul id={listId} role="listbox" aria-label={label} className="max-h-72 overflow-auto">
          {items.map((c, i) => (
            <li
              key={c?.id ?? NONE}
              id={`${listId}-${i}`}
              role="option"
              aria-selected={(c?.id ?? null) === value}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => choose(c)}
              onMouseEnter={() => setActive(i)}
              title={c?.path}
              className={cx(
                'flex min-h-8 cursor-default items-center gap-2 rounded-control px-2 py-1 text-[13px] leading-snug',
                i === active ? 'bg-accent-soft text-accent' : 'text-fg',
              )}
            >
              {c ? (
                <span className="min-w-0 break-words" style={{ paddingLeft: c.parent_id ? 12 : 0 }}>
                  {c.parent_id ? (
                    <span className="text-dim">{`${c.path.split(' › ')[0]} › `}</span>
                  ) : null}
                  {c.name}
                </span>
              ) : (
                <span className="text-dim italic">{noneLabel}</span>
              )}
              {(c?.id ?? null) === value ? (
                <Icon name="check" size={12} className="ml-auto" />
              ) : null}
            </li>
          ))}
          {items.length === 0 ? (
            <li className="px-2 py-2 text-[12.5px] text-dim">{t('controles-nenhuma-categoria')}</li>
          ) : null}
        </ul>
      </div>
    </Popover>
  );
}
