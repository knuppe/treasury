import { useState, type ChangeEvent, type FocusEvent } from 'react';

import { formatDate, parseDate } from '@/lib/format';

import { Input, type InputProps } from './Input';

export interface DateInputProps extends Omit<
  InputProps,
  'value' | 'defaultValue' | 'onChange' | 'type'
> {
  /** Data ISO `YYYY-MM-DD`, ou `null` quando vazia/inválida. */
  value: string | null;
  onValueChange: (iso: string | null) => void;
}

function toText(iso: string | null): string {
  if (!iso) return '';
  try {
    return formatDate(iso);
  } catch {
    return '';
  }
}

const LIMITS = [2, 2, 4] as const;

/**
 * Máscara `dd/mm/aaaa` enquanto se digita: insere as barras sozinha, completa
 * dia/mês de um dígito quando a pessoa digita a barra (`1/` → `01/`) e aceita
 * colar uma data ISO (`2026-10-14`).
 */
export function maskDate(raw: string): string {
  const iso = /^\s*(\d{4})-(\d{2})-(\d{2})\s*$/.exec(raw);
  if (iso) return `${iso[3]}/${iso[2]}/${iso[1]}`;

  const done: string[] = [];
  let current = '';
  for (const ch of raw.replace(/[^\d/]/g, '')) {
    const segment = done.length;
    if (segment > 2) break;
    if (ch === '/') {
      if (segment < 2 && current.length > 0) {
        done.push(current.padStart(2, '0'));
        current = '';
      }
      continue;
    }
    if (current.length === LIMITS[segment]) {
      if (segment >= 2) break;
      done.push(current);
      current = '';
    }
    current += ch;
    if (done.length < 2 && current.length === LIMITS[done.length]) {
      done.push(current);
      current = '';
    }
  }
  const head = done.join('/');
  if (done.length === 0) return current;
  if (done.length >= 3) return head;
  return `${head}/${current}`;
}

/** Campo de data `dd/mm/aaaa` ↔ ISO `YYYY-MM-DD`. */
export function DateInput({
  value,
  onValueChange,
  invalid,
  onBlur,
  onFocus,
  placeholder = 'dd/mm/aaaa',
  ...rest
}: DateInputProps) {
  const [text, setText] = useState(() => toText(value));
  const [blurred, setBlurred] = useState(false);
  const [synced, setSynced] = useState<string | null>(value);
  if (value !== synced) {
    setSynced(value);
    if (parseDate(text) !== value) setText(toText(value));
  }

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    const raw = event.target.value;
    // Apagando: respeita o que a pessoa fez (senão a barra "volta" sozinha).
    const deleting = raw.length < text.length;
    const next = deleting ? raw.replace(/[^\d/]/g, '').slice(0, 10) : maskDate(raw);
    setText(next);
    const iso = parseDate(next);
    setSynced(iso);
    onValueChange(iso);
  };

  const handleBlur = (event: FocusEvent<HTMLInputElement>) => {
    const iso = parseDate(text);
    if (iso) setText(toText(iso));
    setBlurred(true);
    onBlur?.(event);
  };

  // Só acusa erro com a data completa ou depois de sair do campo.
  const textInvalid =
    text.trim() !== '' && parseDate(text) == null && (text.length === 10 || blurred);

  return (
    <Input
      type="text"
      inputMode="numeric"
      autoComplete="off"
      spellCheck={false}
      maxLength={10}
      placeholder={placeholder}
      value={text}
      onChange={handleChange}
      onBlur={handleBlur}
      onFocus={(event) => {
        setBlurred(false);
        onFocus?.(event);
      }}
      invalid={Boolean(invalid) || textInvalid}
      className="num"
      {...rest}
    />
  );
}
