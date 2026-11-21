import { useState, type ChangeEvent, type FocusEvent } from 'react';

import { currencyPrefix, formatMoney, parseMoney } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';

import { Input, type InputProps } from './Input';

export interface MoneyInputProps extends Omit<
  InputProps,
  'value' | 'defaultValue' | 'onChange' | 'prefix' | 'type' | 'numeric'
> {
  /** Valor em centavos (menor unidade da moeda) ou `null` quando vazio/inválido. */
  value: number | null;
  /** Chamado a cada digitação com o valor em centavos (`null` se vazio ou inválido). */
  onValueChange: (cents: number | null) => void;
  /** Código ISO 4217 (padrão: a moeda base da família). */
  currency?: string;
  /** Permite valores negativos (padrão: não). */
  allowNegative?: boolean;
}

function toText(cents: number | null, currency: string): string {
  return cents == null ? '' : formatMoney(cents, currency, { hideCurrency: true });
}

/**
 * Campo de valor monetário no idioma do app: aceita `1.234,56` (ou `1234,56`, `12.50`
 * colado; em inglês, `1,234.56`) e devolve centavos inteiros. Ao sair do campo, normaliza o
 * texto (`1234,5` → `1.234,50`). Texto inválido marca o campo com `aria-invalid`.
 */
export function MoneyInput({
  value,
  onValueChange,
  currency: wanted,
  allowNegative = false,
  invalid,
  onBlur,
  placeholder,
  ...rest
}: MoneyInputProps) {
  const base = useBaseCurrency();
  const currency = wanted ?? base;
  const [text, setText] = useState(() => toText(value, currency));
  // Último valor reconciliado com o texto: quando o pai muda `value` por conta
  // própria (não por digitação aqui), o texto acompanha.
  const [synced, setSynced] = useState<number | null>(value);
  if (!Object.is(value, synced)) {
    setSynced(value);
    if (!Object.is(parseMoney(text, currency), value)) setText(toText(value, currency));
  }

  const parse = (raw: string): number | null => {
    const cents = parseMoney(raw, currency);
    if (cents != null && cents < 0 && !allowNegative) return null;
    return cents;
  };

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    const allowed = allowNegative ? /[^\d.,\-−]/g : /[^\d.,]/g;
    const raw = event.target.value.replace(allowed, '');
    setText(raw);
    const cents = parse(raw);
    setSynced(cents);
    onValueChange(cents);
  };

  const handleBlur = (event: FocusEvent<HTMLInputElement>) => {
    const cents = parse(text);
    if (cents != null) setText(toText(cents, currency));
    onBlur?.(event);
  };

  const textInvalid = text.trim() !== '' && parse(text) == null;

  return (
    <Input
      type="text"
      inputMode="decimal"
      autoComplete="off"
      spellCheck={false}
      numeric
      prefix={currencyPrefix(currency)}
      placeholder={placeholder ?? formatMoney(0, currency, { hideCurrency: true })}
      value={text}
      onChange={handleChange}
      onBlur={handleBlur}
      invalid={Boolean(invalid) || textInvalid}
      {...rest}
    />
  );
}
