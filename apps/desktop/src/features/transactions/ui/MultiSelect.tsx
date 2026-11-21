import { useMemo, useState, type ReactNode } from 'react';

import { normalizeSearch } from '@/commands/registry';
import { Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { Popover } from './Popover';

export interface MultiOption {
  value: string;
  label: string;
  /** Recuo (subcategorias). */
  depth?: number;
  hint?: ReactNode;
}

/**
 * Seleção múltipla em painel: busca, marcar/desmarcar com Espaço e "limpar".
 * Mostra no botão quantos estão marcados.
 */
export function MultiSelect({
  label,
  options,
  value,
  onChange,
  searchable = options.length > 8,
  emptyLabel = t('controles-todos'),
  className,
}: {
  label: string;
  options: readonly MultiOption[];
  value: readonly string[];
  onChange: (value: string[]) => void;
  searchable?: boolean;
  emptyLabel?: string;
  className?: string;
}) {
  const [search, setSearch] = useState('');
  const selected = new Set(value);
  const visible = useMemo(() => {
    const q = normalizeSearch(search.trim());
    return q ? options.filter((o) => normalizeSearch(o.label).includes(q)) : options;
  }, [options, search]);
  const summary =
    value.length === 0
      ? emptyLabel
      : value.length === 1
        ? (options.find((o) => o.value === value[0])?.label ?? '1')
        : t('controles-n-selecionados', { n: value.length });

  const toggle = (v: string) => {
    const next = new Set(selected);
    if (next.has(v)) next.delete(v);
    else next.add(v);
    onChange(options.filter((o) => next.has(o.value)).map((o) => o.value));
  };

  return (
    <Popover
      className={className}
      active={value.length > 0}
      ariaLabel={`${label}: ${summary}`}
      label={
        <>
          <span className="label-caps">{label}</span>
          <span className="max-w-40 truncate">{summary}</span>
          <Icon name="chevronDown" size={12} className="text-dim" />
        </>
      }
      panelClassName="w-72"
    >
      <div className="flex flex-col gap-1.5">
        {searchable ? (
          <input
            type="search"
            aria-label={t('controles-buscar-em', { rotulo: label.toLowerCase() })}
            placeholder={t('controles-buscar')}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="h-8 rounded-control border border-line-strong bg-bg px-2 text-[13px] text-fg outline-none focus:border-accent"
          />
        ) : null}
        <ul
          role="listbox"
          aria-multiselectable
          aria-label={label}
          className="max-h-72 overflow-auto"
        >
          {visible.map((o) => {
            const on = selected.has(o.value);
            return (
              <li key={o.value} role="option" aria-selected={on}>
                <button
                  type="button"
                  onClick={() => toggle(o.value)}
                  className={cx(
                    'flex h-8 w-full items-center gap-2 rounded-control pr-2 text-left text-[13px] text-fg hover:bg-accent-soft focus-visible:bg-accent-soft',
                  )}
                  style={{ paddingLeft: 8 + (o.depth ?? 0) * 14 }}
                >
                  <span
                    aria-hidden
                    className={cx(
                      'inline-flex size-3.5 shrink-0 items-center justify-center rounded-[2px] border',
                      on ? 'border-accent bg-accent text-accent-ink' : 'border-line-strong',
                    )}
                  >
                    {on ? <Icon name="check" size={10} /> : null}
                  </span>
                  <span className="min-w-0 flex-1 truncate">{o.label}</span>
                  {o.hint ? <span className="num text-[11px] text-dim">{o.hint}</span> : null}
                </button>
              </li>
            );
          })}
          {visible.length === 0 ? (
            <li className="px-2 py-2 text-[12.5px] text-dim">{t('controles-nada-encontrado')}</li>
          ) : null}
        </ul>
        {value.length > 0 ? (
          <button
            type="button"
            onClick={() => onChange([])}
            className="label-caps self-start rounded-control px-2 py-1 hover:text-fg"
          >
            {t('controles-limpar-selecao')}
          </button>
        ) : null}
      </div>
    </Popover>
  );
}
