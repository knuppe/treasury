/**
 * Autocompletar de tickers (`market.search`): ativos cadastrados, o universo da análise de
 * ativos e a busca do Yahoo, com espera de digitação. Cada linha mostra ticker, nome,
 * bolsa, tipo e moeda. Teclado: ↑/↓ escolhem, Enter confirma, Esc fecha.
 */
import { useId, useState, type KeyboardEvent } from 'react';

import type { TickerSuggestion } from '@/api/generated/TickerSuggestion';
import { useTickerSearch } from '@/api/hooks/market';
import { Icon, Input } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

const ORIGIN_KEY = {
  asset: 'mercado-origem-cadastrado',
  research: 'mercado-origem-analise',
  yahoo: 'mercado-origem-yahoo',
} as const satisfies Record<TickerSuggestion['origin'], string>;

const originLabel = (o: TickerSuggestion['origin']) => t(ORIGIN_KEY[o]);

export function suggestionKey(s: TickerSuggestion): string {
  return `${s.origin}:${s.asset_id ?? ''}:${s.symbol ?? s.ticker}:${s.exchange ?? ''}`;
}

export function TickerSearch({
  onPick,
  placeholder = t('mercado-busca-placeholder'),
  autoFocus,
  label = t('mercado-busca-rotulo'),
}: {
  onPick: (s: TickerSuggestion) => void;
  placeholder?: string;
  autoFocus?: boolean;
  label?: string;
}) {
  const [text, setText] = useState('');
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const listId = useId();
  const search = useTickerSearch(text);
  const results = text.trim() ? (search.data?.results ?? []) : [];
  const show = open && text.trim().length > 0;

  const pick = (s: TickerSuggestion) => {
    onPick(s);
    setText('');
    setOpen(false);
    setActive(0);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setOpen(true);
      setActive((i) => Math.min(i + 1, Math.max(results.length - 1, 0)));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActive((i) => Math.max(i - 1, 0));
    } else if (e.key === 'Enter') {
      const s = results[active];
      if (show && s) {
        e.preventDefault();
        pick(s);
      }
    } else if (e.key === 'Escape' && show) {
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
    }
  };

  return (
    <div className="relative">
      <Input
        value={text}
        aria-label={label}
        role="combobox"
        aria-expanded={show}
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={show && results[active] ? `${listId}-${active}` : undefined}
        autoComplete="off"
        spellCheck={false}
        autoFocus={autoFocus}
        placeholder={placeholder}
        prefix={<Icon name="search" size={12} />}
        onChange={(e) => {
          setText(e.target.value);
          setOpen(true);
          setActive(0);
        }}
        onFocus={() => setOpen(true)}
        onBlur={() => setTimeout(() => setOpen(false), 150)}
        onKeyDown={onKeyDown}
      />
      {show ? (
        <ul
          id={listId}
          role="listbox"
          aria-label={t('mercado-sugestoes')}
          className="absolute z-[60] mt-1 max-h-80 w-full overflow-y-auto rounded-control border border-panel pop-surface py-1 shadow-lg"
        >
          {results.map((s, i) => (
            <li
              key={suggestionKey(s)}
              id={`${listId}-${i}`}
              role="option"
              aria-selected={i === active}
              aria-label={`${s.ticker} — ${s.name} · ${[
                s.exchange ?? s.exchange_name ?? t('mercado-bolsa-fora'),
                s.type_label,
                s.currency,
              ]
                .filter(Boolean)
                .join(' · ')} (${originLabel(s.origin)})`}
              onMouseDown={(e) => {
                e.preventDefault();
                pick(s);
              }}
              onMouseEnter={() => setActive(i)}
              className={cx(
                'flex cursor-pointer items-center gap-3 px-2.5 py-1.5 text-[12.5px]',
                i === active && 'bg-surface-2',
              )}
            >
              <span className="num w-24 shrink-0 truncate font-medium text-fg">{s.ticker}</span>
              <span className="min-w-0 flex-1 truncate text-dim">{s.name}</span>
              <span className="shrink-0 text-[11px] text-dim">
                {[
                  s.exchange ?? s.exchange_name ?? t('mercado-bolsa-fora'),
                  s.type_label,
                  s.currency,
                ]
                  .filter(Boolean)
                  .join(' · ')}
              </span>
              <span
                className={cx(
                  'shrink-0 rounded-full border px-1.5 text-[10px]',
                  s.origin === 'asset' ? 'border-accent text-accent' : 'border-line text-dim',
                )}
              >
                {originLabel(s.origin)}
              </span>
            </li>
          ))}
          {results.length === 0 ? (
            <li className="px-2.5 py-2 text-[12px] text-dim" role="presentation">
              {search.pending ? t('mercado-buscando') : t('controles-nada-encontrado')}
            </li>
          ) : null}
          {search.data?.error ? (
            <li className="px-2.5 py-1.5 text-[11.5px] text-warning" role="presentation">
              {t('mercado-so-locais', { erro: search.data.error })}
            </li>
          ) : null}
        </ul>
      ) : null}
    </div>
  );
}
