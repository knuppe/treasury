/** Máscaras dos campos de valor dos proventos (grade e confirmação). */
import { baseCurrency, formatMoney } from '@/lib/format';

/** Centavos → `1.234,50` (casas decimais da moeda base). */
export const maskAmount = (cents: number) =>
  formatMoney(cents, baseCurrency(), { hideCurrency: true });

/**
 * Máscara do valor enquanto se digita, como nos apps de banco: os dígitos entram pela direita
 * nos centavos (`1` → `0,01`, `1234` → `12,34`, `123456` → `1.234,56`).
 */
export function typeAmount(text: string): string {
  const digits = text.replace(/\D/g, '').replace(/^0+/, '').slice(0, 13);
  return digits ? maskAmount(Number(digits)) : '';
}
