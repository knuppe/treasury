/**
 * Formatação do Treasury, por idioma (docs/i18n-design.md §Números, datas e moeda). O padrão é
 * pt-BR; o `I18nProvider` troca com `setFormatLocale` ao mudar o idioma da instalação.
 *
 * Dinheiro é sempre um inteiro na menor unidade da moeda (centavos para BRL),
 * nunca ponto flutuante: `number` é aceito porque inteiros cabem com folga em
 * 2^53. Toda a aritmética aqui é feita sobre inteiros e strings, sem Intl para
 * valores monetários, para o resultado ser idêntico em qualquer WebView.
 *
 * Convenções (docs/design.md):
 * - BRL: `R$ 1.234,56`; outras moedas: código ISO, `USD 1.234,56`.
 * - Sinal negativo é o caractere MENOS (U+2212), não o hífen.
 * - Datas: `14/10/2026`; meses: `outubro de 2026` e `out/26`.
 */

/** Sinal de menos tipográfico (U+2212). */
export const MINUS = '−';

export type FormatLocale = 'pt-BR' | 'en' | 'es' | 'pt-PT' | 'xx';

interface Table {
  code: FormatLocale;
  thousands: string;
  decimal: string;
  /** pt-PT: separador de milhar só a partir de 5 dígitos. */
  minGroup: number;
  months: readonly string[];
  monthsShort: readonly string[];
}

const PT_MONTHS = [
  'janeiro',
  'fevereiro',
  'março',
  'abril',
  'maio',
  'junho',
  'julho',
  'agosto',
  'setembro',
  'outubro',
  'novembro',
  'dezembro',
] as const;

const PT_MONTHS_SHORT = [
  'jan',
  'fev',
  'mar',
  'abr',
  'mai',
  'jun',
  'jul',
  'ago',
  'set',
  'out',
  'nov',
  'dez',
] as const;

const PT_BR: Table = {
  code: 'pt-BR',
  thousands: '.',
  decimal: ',',
  minGroup: 4,
  months: PT_MONTHS,
  monthsShort: PT_MONTHS_SHORT,
};

let active: Table = PT_BR;

/** Idioma ativo da formatação. */
export function formatLocale(): FormatLocale {
  return active.code;
}

/**
 * Troca o idioma da formatação. Os nomes dos meses vêm do catálogo (`comum-meses`,
 * `comum-meses-curtos`); sem eles, os do pt-BR.
 */
export function setFormatLocale(
  code: FormatLocale,
  months?: readonly string[],
  monthsShort?: readonly string[],
): void {
  const base =
    code === 'en'
      ? { thousands: ',', decimal: '.', minGroup: 4 }
      : code === 'pt-PT'
        ? { thousands: '\u202F', decimal: ',', minGroup: 5 }
        : { thousands: '.', decimal: ',', minGroup: 4 };
  active = {
    code,
    ...base,
    months: months?.length === 12 ? months : PT_MONTHS,
    monthsShort: monthsShort?.length === 12 ? monthsShort : PT_MONTHS_SHORT,
  };
}

/** Casas decimais por moeda (ISO 4217). Ausente = 2. */
const MINOR_UNITS: Readonly<Record<string, number>> = {
  BIF: 0,
  CLP: 0,
  DJF: 0,
  GNF: 0,
  ISK: 0,
  JPY: 0,
  KMF: 0,
  KRW: 0,
  PYG: 0,
  RWF: 0,
  UGX: 0,
  VND: 0,
  VUV: 0,
  XAF: 0,
  XOF: 0,
  XPF: 0,
  BHD: 3,
  IQD: 3,
  JOD: 3,
  KWD: 3,
  LYD: 3,
  OMR: 3,
  TND: 3,
};

/** Quantas casas decimais a moeda usa (2 para BRL e USD, 0 para JPY…). */
export function minorUnits(currency: string): number {
  return MINOR_UNITS[currency.toUpperCase()] ?? 2;
}

function groupThousands(digits: string): string {
  if (digits.length < active.minGroup) return digits;
  return digits.replace(/\B(?=(\d{3})+(?!\d))/g, active.thousands);
}

function assertMinorInteger(value: number, what: string): void {
  if (!Number.isSafeInteger(value)) {
    throw new RangeError(
      `${what} precisa ser um inteiro na menor unidade da moeda (recebido: ${String(value)}).`,
    );
  }
}

export interface FormatMoneyOptions {
  /** Mostra `+` em valores positivos (para variações, ganhos e perdas). */
  signed?: boolean;
  /** Omite o símbolo/código da moeda (útil em colunas de tabela de uma só moeda). */
  hideCurrency?: boolean;
}

/** Prefixo usado para a moeda: `R$` para BRL, o código ISO para as demais. */
export function currencyPrefix(currency: string): string {
  const code = currency.toUpperCase();
  if (code === 'BRL') return 'R$';
  if (code === 'USD' && active.code === 'en') return '$';
  if (code === 'USD' && active.code === 'es') return 'US$';
  if (code === 'EUR' && active.code === 'en') return '€';
  return code;
}

/** Moedas escritas com o símbolo depois do número (`1234,56 €`). */
export function currencySuffix(currency: string): string | null {
  const code = currency.toUpperCase();
  return code === 'EUR' && (active.code === 'pt-PT' || active.code === 'es') ? '€' : null;
}

/** Símbolo colado ao número (`$1,234.56`), sem espaço, no inglês. */
function glued(prefix: string): boolean {
  return active.code === 'en' && (prefix === '$' || prefix === '€' || prefix === 'R$');
}

/** Moeda base da família (a do país do cofre): o padrão de `formatMoney`, `parseMoney` e `<Money>`. */
let base = 'BRL';

export function baseCurrency(): string {
  return base;
}

/** Passa a usar a moeda base do cofre aberto (ver `state/currency.ts`, que avisa os componentes). */
export function setFormatBaseCurrency(code: string): void {
  base = code.trim().toUpperCase() || 'BRL';
}

/**
 * Formata um valor em centavos (menor unidade) no idioma do app; sem moeda, na moeda base.
 *
 * ```ts
 * formatMoney(123456, 'BRL')   // 'R$ 1.234,56'
 * formatMoney(-500, 'BRL')     // '−R$ 5,00'
 * formatMoney(123456, 'USD')   // 'USD 1.234,56'
 * ```
 */
export function formatMoney(
  cents: number,
  currency: string = base,
  options: FormatMoneyOptions = {},
): string {
  assertMinorInteger(cents, 'O valor');
  const decimals = minorUnits(currency);
  const negative = cents < 0;
  const abs = Math.abs(cents);
  const factor = 10 ** decimals;
  const whole = Math.floor(abs / factor);
  const fraction = abs - whole * factor;

  let number = groupThousands(String(whole));
  if (decimals > 0) number += active.decimal + String(fraction).padStart(decimals, '0');

  const sign = negative ? MINUS : options.signed && cents > 0 ? '+' : '';
  const suffix = currencySuffix(currency);
  const prefix = currencyPrefix(currency);
  const body = options.hideCurrency
    ? number
    : suffix
      ? `${number} ${suffix}`
      : glued(prefix)
        ? `${prefix}${number}`
        : `${prefix} ${number}`;
  return sign + body;
}

/**
 * Lê um valor digitado em pt-BR e devolve inteiro na menor unidade da moeda,
 * ou `null` se o texto não for um valor válido.
 *
 * Aceita `1.234,56`, `1234,56`, `1234`, `R$ 1.234,56`, `−12,5`, `,50` e, sem
 * vírgula, um ponto seguido de 1–2 dígitos como decimal (`12.50` colado de
 * fonte americana). Mais casas que a moeda permite → `null`.
 */
export function parseMoney(text: string, currency: string = base): number | null {
  const decimals = minorUnits(currency);
  let s = text.trim().replace(/[\s\u202F€]+/g, '');
  if (s === '') return null;
  // Inglês: vírgula é milhar e ponto é decimal (`1,234.56`, `1,234` = mil duzentos e
  // trinta e quatro). Troca para o formato pt-BR e segue a mesma leitura.
  if (active.code === 'en') {
    s = s.replace(/^US\$|^\$/i, '');
    if (/,/.test(s) && !/^-?[\d]{1,3}(,\d{3})+(\.\d*)?$/.test(s.replace(/^[-−+]/, ''))) {
      return null;
    }
    s = s.replaceAll(',', '').replace('.', ',');
  } else {
    s = s.replace(/^US\$/i, '');
  }

  // Prefixo de moeda opcional: "R$" ou um código ISO em maiúsculas ("USD").
  s = s.replace(/^R\$/i, '').replace(/^[A-Z]{3}(?=[\d.,+\-\u2212])/, '');

  let negative = false;
  if (/^[-−]/.test(s)) {
    negative = true;
    s = s.slice(1);
  } else if (s.startsWith('+')) {
    s = s.slice(1);
  }
  // Também aceita o sinal depois do prefixo, ex.: "R$ -10,00" ou "-R$ 10,00".
  s = s.replace(/^R\$/i, '');
  if (/^[-−]/.test(s)) {
    if (negative) return null;
    negative = true;
    s = s.slice(1);
  }

  let integerPart: string;
  let fractionPart = '';

  if (s.includes(',')) {
    const parts = s.split(',');
    if (parts.length !== 2) return null;
    integerPart = parts[0] ?? '';
    fractionPart = parts[1] ?? '';
    if (!/^\d*$/.test(fractionPart)) return null;
    if (!isValidIntegerPart(integerPart)) return null;
    integerPart = integerPart.replaceAll('.', '');
  } else {
    const lastDot = s.lastIndexOf('.');
    const afterDot = lastDot >= 0 ? s.slice(lastDot + 1) : '';
    const singleDot = lastDot >= 0 && s.indexOf('.') === lastDot;
    if (singleDot && /^\d{1,2}$/.test(afterDot)) {
      // "12.5" / "12.50": ponto decimal.
      integerPart = s.slice(0, lastDot);
      fractionPart = afterDot;
      if (!/^\d*$/.test(integerPart)) return null;
    } else {
      if (!isValidIntegerPart(s)) return null;
      integerPart = s.replaceAll('.', '');
    }
  }

  if (integerPart === '' && fractionPart === '') return null;
  if (fractionPart.length > decimals) return null;

  const whole = integerPart === '' ? 0 : Number(integerPart);
  const fraction = fractionPart === '' ? 0 : Number(fractionPart.padEnd(decimals, '0'));
  const value = whole * 10 ** decimals + fraction;
  if (!Number.isSafeInteger(value)) return null;
  return negative && value !== 0 ? -value : value;
}

/** Parte inteira com ou sem separador de milhar (`1234` ou `1.234.567`). */
function isValidIntegerPart(s: string): boolean {
  if (s === '') return true;
  if (/^\d+$/.test(s)) return true;
  return /^\d{1,3}(\.\d{3})+$/.test(s);
}

/** Código para o `Intl` (o pseudo-idioma usa o pt-BR). */
function intlCode(): string {
  return active.code === 'xx' ? 'pt-BR' : active.code;
}

// ── Percentuais ──────────────────────────────────────────────────

export interface FormatPercentOptions {
  /** Casas decimais (padrão 1). */
  decimals?: number;
  /** Mostra `+` em valores positivos. */
  signed?: boolean;
}

/**
 * Formata uma razão como percentual: `formatPercent(0.125)` → `12,5%`.
 * Percentuais são razões (não dinheiro), então ponto flutuante é aceitável.
 */
export function formatPercent(ratio: number, options: FormatPercentOptions = {}): string {
  const decimals = options.decimals ?? 1;
  if (!Number.isFinite(ratio)) return '—';
  const pct = ratio * 100;
  const rounded = Number(pct.toFixed(decimals));
  const isZero = rounded === 0;
  const formatted = new Intl.NumberFormat(intlCode(), {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
    useGrouping: true,
  }).format(Math.abs(rounded));
  const sign = isZero ? '' : rounded < 0 ? MINUS : options.signed ? '+' : '';
  return `${sign}${formatted}%`;
}

/** Formata um número comum em pt-BR (`1.234,5`), com sinal U+2212. */
export function formatNumber(value: number, decimals = 0): string {
  if (!Number.isFinite(value)) return '—';
  const formatted = new Intl.NumberFormat(intlCode(), {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(Math.abs(value));
  const isZero = Number(value.toFixed(decimals)) === 0;
  return (value < 0 && !isZero ? MINUS : '') + formatted;
}

// ── Datas ────────────────────────────────────────────────────────

const ISO_DATE = /^(\d{4})-(\d{2})-(\d{2})$/;
const ISO_MONTH = /^(\d{4})-(\d{2})(?:-\d{2})?$/;

function isValidDate(year: number, month: number, day: number): boolean {
  if (month < 1 || month > 12 || day < 1 || year < 1) return false;
  const daysInMonth = new Date(Date.UTC(year, month, 0)).getUTCDate();
  return day <= daysInMonth;
}

/** `'2026-10-14'` → `'14/10/2026'`. Lança `RangeError` se a data for inválida. */
export function formatDate(iso: string): string {
  const match = ISO_DATE.exec(iso);
  if (!match) throw new RangeError(`Data ISO inválida: ${iso}`);
  const [, y, m, d] = match as unknown as [string, string, string, string];
  if (!isValidDate(Number(y), Number(m), Number(d))) {
    throw new RangeError(`Data inexistente: ${iso}`);
  }
  return active.code === 'en' ? `${m}/${d}/${y}` : `${d}/${m}/${y}`;
}

/** `'2026-10-14'` → `'14 de outubro de 2026'` / `'October 14, 2026'` (texto corrido). */
export function formatDateLong(iso: string): string {
  formatDate(iso);
  const [y, m, d] = iso.split('-').map(Number) as [number, number, number];
  const month = active.months[m - 1] ?? '';
  return active.code === 'en' ? `${month} ${d}, ${y}` : `${d} de ${month} de ${y}`;
}

/**
 * `'14/10/2026'` → `'2026-10-14'`; `null` se o texto não for uma data real.
 * Aceita dia e mês com um dígito (`1/2/2026`).
 */
export function parseDate(text: string): string | null {
  const match = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(text.trim());
  if (!match) return null;
  const [, a, b, y] = match as unknown as [string, string, string, string];
  // Inglês: mês antes do dia (`10/14/2026`).
  const [d, m] = active.code === 'en' ? [b, a] : [a, b];
  const day = Number(d);
  const month = Number(m);
  const year = Number(y);
  if (!isValidDate(year, month, day)) return null;
  return `${y}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

/** Instante ISO (UTC) → `14/10/2026 13:05` no fuso local. */
export function formatDateTime(instant: string | Date): string {
  const date = typeof instant === 'string' ? new Date(instant) : instant;
  if (Number.isNaN(date.getTime())) throw new RangeError(`Instante inválido: ${String(instant)}`);
  const pad = (n: number) => String(n).padStart(2, '0');
  const [first, second] =
    active.code === 'en'
      ? [date.getMonth() + 1, date.getDate()]
      : [date.getDate(), date.getMonth() + 1];
  return (
    `${pad(first)}/${pad(second)}/${date.getFullYear()} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}`
  );
}

/** Data ISO (`YYYY-MM-DD`) de hoje no fuso local. */
export function todayIso(now: Date = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** Meses em pt-BR (constantes antigas; o texto da tela usa `monthName`). */
export const MONTHS_LONG = PT_MONTHS;
export const MONTHS_SHORT = PT_MONTHS_SHORT;

/** Nome do mês (1–12) no idioma ativo. */
export function monthName(month: number, short = false): string {
  return (short ? active.monthsShort : active.months)[month - 1] ?? '';
}

function parseMonth(isoMonth: string): { year: number; month: number } {
  const match = ISO_MONTH.exec(isoMonth);
  const year = Number(match?.[1]);
  const month = Number(match?.[2]);
  if (!match || month < 1 || month > 12) throw new RangeError(`Mês ISO inválido: ${isoMonth}`);
  return { year, month };
}

/** `'2026-10'` (ou `'2026-10-14'`) → `'outubro de 2026'`. */
export function formatMonthLong(isoMonth: string): string {
  const { year, month } = parseMonth(isoMonth);
  const name = active.months[month - 1] ?? '';
  return active.code === 'en' ? `${name} ${year}` : `${name} de ${year}`;
}

/** `'2026-10'` (ou `'2026-10-14'`) → `'out/26'`. */
export function formatMonthShort(isoMonth: string): string {
  const { year, month } = parseMonth(isoMonth);
  const yy = String(year % 100).padStart(2, '0');
  const name = active.monthsShort[month - 1] ?? '';
  return active.code === 'en' ? `${name} ${yy}` : `${name}/${yy}`;
}

// ── Plural ────────────────────────────────────────────────────────

const PREPOSITIONS = new Set(['de', 'do', 'da', 'dos', 'das', 'em', 'no', 'na', 'entre', 'por']);

function pluralWord(word: string): string {
  if (/[sx]$/i.test(word)) return word;
  if (/ão$/i.test(word)) return word.replace(/ão$/i, 'ões');
  if (/m$/i.test(word)) return word.replace(/m$/i, 'ns');
  if (/[rz]$/i.test(word)) return `${word}es`;
  if (/al$/i.test(word)) return word.replace(/al$/i, 'ais');
  return `${word}s`;
}

/**
 * Plural de um rótulo curto: `lançamento` → `lançamentos`, `parte da divisão` →
 * `partes da divisão`, `bem físico` → `bens físicos`. Flexiona o substantivo e
 * a palavra logo depois dele, se não for preposição.
 */
export function pluralize(label: string): string {
  const words = label.split(' ');
  const second = words[1];
  words[0] = pluralWord(words[0] ?? '');
  if (second && !PREPOSITIONS.has(second.toLowerCase())) words[1] = pluralWord(second);
  return words.join(' ');
}

/** `1 conta`, `3 contas`: singular só para 1; o plural vem de `pluralize` ou do 3º argumento. */
export function countLabel(n: number, singular: string, plural = pluralize(singular)): string {
  return `${n.toLocaleString(intlCode())} ${n === 1 ? singular : plural}`;
}
