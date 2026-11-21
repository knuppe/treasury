import { useId, useState } from 'react';

import { cx } from '../cx';
import { Icon } from './Icon';
import { CONTROL_FRAME } from './Input';

export interface ChipsInputProps {
  value: string[];
  onChange: (values: string[]) => void;
  /** Nome acessível do campo de texto. */
  label: string;
  /** Nome acessível do botão que tira um item. */
  removeLabel: (item: string) => string;
  /** Sugestões do campo (lista nativa do navegador). */
  suggestions?: readonly string[];
  placeholder?: string;
  /** Prefixo mostrado em cada item (`#` nas tags). */
  prefix?: string;
  /** Tira o prefixo digitado pela pessoa (`#viagem` → `viagem`). */
  stripPrefix?: boolean;
  className?: string;
}

/**
 * Lista de nomes em chips (tags, etiquetas): Enter ou vírgula adiciona, Backspace no campo vazio
 * tira o último. Repetidos (sem diferenciar maiúsculas) não entram.
 */
export function ChipsInput({
  value,
  onChange,
  label,
  removeLabel,
  suggestions = [],
  placeholder,
  prefix = '',
  stripPrefix = Boolean(prefix),
  className,
}: ChipsInputProps) {
  const [text, setText] = useState('');
  const listId = useId();
  const add = (raw: string) => {
    const names = raw
      .split(',')
      .map((name) => {
        const n = name.trim();
        return stripPrefix && prefix && n.startsWith(prefix) ? n.slice(prefix.length) : n;
      })
      .filter(Boolean);
    const next = [...value];
    for (const n of names) if (!next.some((x) => x.toLowerCase() === n.toLowerCase())) next.push(n);
    onChange(next);
    setText('');
  };
  return (
    <div
      className={cx(CONTROL_FRAME, 'h-auto min-h-9 flex-wrap border-line-strong py-1', className)}
    >
      {value.map((item) => (
        <span
          key={item}
          className="inline-flex h-6 items-center gap-1 rounded-control border border-line bg-surface-2 pr-0.5 pl-1.5 text-[12px] text-fg"
        >
          {`${prefix}${item}`}
          <button
            type="button"
            aria-label={removeLabel(item)}
            onClick={() => onChange(value.filter((x) => x !== item))}
            className="inline-flex size-5 items-center justify-center rounded-control text-dim hover:text-fg"
          >
            <Icon name="x" size={10} />
          </button>
        </span>
      ))}
      <input
        aria-label={label}
        list={listId}
        value={text}
        placeholder={value.length ? '' : placeholder}
        onChange={(e) => {
          if (e.target.value.endsWith(',')) add(e.target.value);
          else setText(e.target.value);
        }}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && text.trim()) {
            e.preventDefault();
            add(text);
          } else if (e.key === 'Backspace' && !text && value.length) {
            onChange(value.slice(0, -1));
          }
        }}
        onBlur={() => text.trim() && add(text)}
        className="h-7 min-w-24 flex-1 bg-transparent text-[13px] text-fg outline-none"
      />
      <datalist id={listId}>
        {suggestions.map((k) => (
          <option key={k} value={k} />
        ))}
      </datalist>
    </div>
  );
}
