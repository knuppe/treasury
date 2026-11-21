import { formatLocale } from '@/lib/format';

/**
 * Número decimal que o núcleo manda como texto (`"5.4321"`, taxas e percentuais) com o separador
 * decimal do idioma do app: vírgula em pt-BR, pt-PT e es; ponto em inglês. Sem arredondar.
 */
export function localDecimal(value: string): string {
  return formatLocale() === 'en' ? value : value.replace('.', ',');
}
