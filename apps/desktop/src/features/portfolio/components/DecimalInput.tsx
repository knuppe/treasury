import { useState, type ChangeEvent, type FocusEvent } from 'react';

import { Input, type InputProps } from '@/design/components';

import { decimalToInput, formatDecimal, parseDecimalInput } from '../decimal';

export interface DecimalInputProps extends Omit<
  InputProps,
  'value' | 'defaultValue' | 'onChange' | 'type' | 'numeric'
> {
  /** Decimal canônico (`"1234.5"`) ou `null` quando vazio/inválido. */
  value: string | null;
  onValueChange: (value: string | null) => void;
  /** Casas máximas aceitas (ex.: 8 para quantidades). */
  maxDecimals?: number;
}

/**
 * Campo de decimal em pt-BR (quantidade, preço unitário, câmbio, taxa): aceita
 * `1.234,5678`, `0,035` ou `12.75` colado, e devolve o texto canônico que o núcleo
 * aceita. Ao sair do campo, mostra o número formatado.
 */
export function DecimalInput({
  value,
  onValueChange,
  maxDecimals = 8,
  invalid,
  onBlur,
  ...rest
}: DecimalInputProps) {
  const [text, setText] = useState(() => (value ? decimalToInput(value) : ''));
  const [synced, setSynced] = useState<string | null>(value);
  if (value !== synced) {
    setSynced(value);
    if (parseDecimalInput(text, { maxDecimals }) !== value)
      setText(value ? decimalToInput(value) : '');
  }
  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    const raw = event.target.value.replace(/[^\d.,]/g, '');
    setText(raw);
    const parsed = parseDecimalInput(raw, { maxDecimals });
    setSynced(parsed);
    onValueChange(parsed);
  };
  const handleBlur = (event: FocusEvent<HTMLInputElement>) => {
    const parsed = parseDecimalInput(text, { maxDecimals });
    if (parsed) setText(formatDecimal(parsed));
    onBlur?.(event);
  };
  const textInvalid = text.trim() !== '' && parseDecimalInput(text, { maxDecimals }) == null;
  return (
    <Input
      type="text"
      inputMode="decimal"
      autoComplete="off"
      spellCheck={false}
      numeric
      value={text}
      onChange={handleChange}
      onBlur={handleBlur}
      invalid={Boolean(invalid) || textInvalid}
      {...rest}
    />
  );
}
