/**
 * Decimais exatos em texto, como o núcleo grava quantidades e preços unitários
 * (`rust_decimal`): `"1234.5678"`, `"-0.00012"`. Nada aqui usa ponto flutuante:
 * o texto vira um inteiro `bigint` com uma escala (`1234.5678` = 12345678 × 10⁻⁴),
 * e as contas (produto, comparação, arredondamento para centavos) são exatas.
 *
 * - `parseDecimal` lê o texto canônico do núcleo (ponto decimal).
 * - `parseDecimalInput` lê o que a pessoa digita no formato do idioma (`1.234,5` em pt-BR,
 *   `1,234.5` em inglês) e devolve o texto canônico para mandar ao núcleo (que não aceita
 *   separador de milhar).
 * - `formatDecimal`, `formatQuantity`, `formatUnitPrice` mostram no formato do idioma.
 * - `decimalToMinor` e `productToMinor` convertem para a menor unidade da moeda
 *   (centavos), arredondando metade para longe do zero, igual ao `to_minor` do núcleo.
 */
import { baseCurrency, currencyPrefix, formatLocale, MINUS, minorUnits } from '@/lib/format';

/**
 * Separadores do idioma do app (os mesmos de `lib/format`): inglês com ponto decimal e
 * vírgula no milhar; pt-PT com espaço fino no milhar a partir de 5 dígitos; os demais como
 * o pt-BR.
 */
function separators(): { group: string; decimal: string; minGroup: number } {
  const code = formatLocale();
  if (code === 'en') return { group: ',', decimal: '.', minGroup: 4 };
  if (code === 'pt-PT') return { group: ' ', decimal: ',', minGroup: 5 };
  return { group: '.', decimal: ',', minGroup: 4 };
}

/** Separador decimal do idioma do app (`,` ou `.`). */
export function decimalSeparator(): string {
  return separators().decimal;
}

export interface Dec {
  /** Valor = units ÷ 10^scale (units já com sinal). */
  units: bigint;
  scale: number;
}

const CANONICAL = /^([+-]?)(\d*)(?:\.(\d*))?$/;

/** Texto canônico (`"-12.50"`, `"0.0001"`, `"3"`) → decimal exato; `null` se inválido. */
export function parseDecimal(text: string | null | undefined): Dec | null {
  if (text == null) return null;
  const s = text.trim().replace(MINUS, '-');
  const m = CANONICAL.exec(s);
  if (!m) return null;
  const [, sign, int = '', frac = ''] = m;
  if (int === '' && frac === '') return null;
  const units = BigInt((int || '0') + frac);
  return { units: sign === '-' ? -units : units, scale: frac.length };
}

function pow10(n: number): bigint {
  return 10n ** BigInt(n);
}

function abs(v: bigint): bigint {
  return v < 0n ? -v : v;
}

/** Mesma escala para os dois (para comparar e somar). */
function align(a: Dec, b: Dec): [bigint, bigint, number] {
  const scale = Math.max(a.scale, b.scale);
  return [a.units * pow10(scale - a.scale), b.units * pow10(scale - b.scale), scale];
}

/** Tira zeros à direita da parte fracionária (como `normalize()` do rust_decimal). */
export function normalizeDec(d: Dec): Dec {
  let { units, scale } = d;
  while (scale > 0 && units % 10n === 0n) {
    units /= 10n;
    scale -= 1;
  }
  return { units, scale };
}

/** Decimal → texto canônico normalizado (`"1234.5"`). */
export function decToString(d: Dec): string {
  const n = normalizeDec(d);
  const negative = n.units < 0n;
  const digits = abs(n.units)
    .toString()
    .padStart(n.scale + 1, '0');
  const int = digits.slice(0, digits.length - n.scale);
  const frac = n.scale > 0 ? '.' + digits.slice(digits.length - n.scale) : '';
  return (negative ? '-' : '') + int + frac;
}

/** Arredonda para `decimals` casas, metade para longe do zero. */
export function roundDec(d: Dec, decimals: number): Dec {
  if (d.scale <= decimals) return d;
  const drop = pow10(d.scale - decimals);
  const q = abs(d.units) / drop;
  const r = abs(d.units) % drop;
  const rounded = r * 2n >= drop ? q + 1n : q;
  return { units: d.units < 0n ? -rounded : rounded, scale: decimals };
}

export function compareDecimal(a: string, b: string): number {
  const x = parseDecimal(a);
  const y = parseDecimal(b);
  if (!x || !y) throw new RangeError(`Decimal inválido: ${!x ? a : b}`);
  const [ax, by] = align(x, y);
  return ax === by ? 0 : ax < by ? -1 : 1;
}

export function addDecimal(a: string, b: string): string {
  const x = parseDecimal(a);
  const y = parseDecimal(b);
  if (!x || !y) throw new RangeError(`Decimal inválido: ${!x ? a : b}`);
  const [ax, by, scale] = align(x, y);
  return decToString({ units: ax + by, scale });
}

export function subtractDecimal(a: string, b: string): string {
  const y = parseDecimal(b);
  if (!y) throw new RangeError(`Decimal inválido: ${b}`);
  return addDecimal(a, decToString({ units: -y.units, scale: y.scale }));
}

/** Produto exato de vários decimais em texto. */
export function multiplyDecimal(...factors: string[]): string {
  return decToString(multiplyDec(factors));
}

function multiplyDec(factors: readonly string[]): Dec {
  let acc: Dec = { units: 1n, scale: 0 };
  for (const f of factors) {
    const d = parseDecimal(f);
    if (!d) throw new RangeError(`Decimal inválido: ${f}`);
    acc = { units: acc.units * d.units, scale: acc.scale + d.scale };
  }
  return acc;
}

/**
 * Divisão exata arredondada a `decimals` casas (metade para longe do zero): o preço unitário
 * que, vezes a quantidade, dá o valor total (`divideDecimal("5315.2", "5000", 12)` →
 * `"1.06304"`).
 */
export function divideDecimal(numer: string, denom: string, decimals: number): string {
  const n = parseDecimal(numer);
  const d = parseDecimal(denom);
  if (!n || !d) throw new RangeError(`Decimal inválido: ${!n ? numer : denom}`);
  if (d.units === 0n) throw new RangeError('Divisão por zero.');
  // n/d = (n.units × 10^(decimals + d.scale − n.scale + 1)) / d.units × 10^-(decimals + 1)
  const shift = decimals + 1 + d.scale - n.scale;
  const num = shift >= 0 ? n.units * pow10(shift) : n.units / pow10(-shift);
  const q = num / d.units;
  return decToString(roundDec({ units: q, scale: decimals + 1 }, decimals));
}

export function isZeroDecimal(text: string): boolean {
  const d = parseDecimal(text);
  return d != null && d.units === 0n;
}

export function isPositiveDecimal(text: string): boolean {
  const d = parseDecimal(text);
  return d != null && d.units > 0n;
}

function toSafeNumber(v: bigint): number {
  const n = Number(v);
  if (!Number.isSafeInteger(n)) throw new RangeError('Valor grande demais para centavos.');
  return n;
}

/**
 * Decimal na unidade maior da moeda → inteiro na menor unidade (centavos).
 * `decimalToMinor("12.345", "BRL")` → `1235` (metade para longe do zero).
 */
export function decimalToMinor(text: string, currency: string = baseCurrency()): number {
  const d = parseDecimal(text);
  if (!d) throw new RangeError(`Decimal inválido: ${text}`);
  const decimals = minorUnits(currency);
  const r = roundDec(d, decimals);
  return toSafeNumber(r.units * pow10(decimals - r.scale));
}

/**
 * Produto exato (quantidade × preço [× câmbio]) já na menor unidade da moeda,
 * arredondado uma única vez no fim, como o núcleo calcula o valor bruto de uma
 * operação (`to_minor(qtd × preço)`).
 */
export function productToMinor(
  factors: readonly string[],
  currency: string = baseCurrency(),
): number {
  const d = multiplyDec(factors);
  const decimals = minorUnits(currency);
  const r = roundDec(d, decimals);
  return toSafeNumber(r.units * pow10(decimals - r.scale));
}

/** Centavos → decimal canônico na unidade maior (`1235, 'BRL'` → `"12.35"`). */
export function minorToDecimal(cents: number, currency: string = baseCurrency()): string {
  if (!Number.isSafeInteger(cents)) throw new RangeError('Centavos precisam ser inteiros.');
  return decToString({ units: BigInt(cents), scale: minorUnits(currency) });
}

// ── Entrada no formato do idioma ─────────────────────────────────

function validGroups(s: string): boolean {
  return /^\d{1,3}(\.\d{3})+$/.test(s);
}

/**
 * Lê um decimal digitado no formato do idioma e devolve o texto canônico (`"1234.5"`), ou
 * `null` se não for um número válido (ou for negativo sem `allowNegative`). Em pt-BR:
 *
 * - Com vírgula: a vírgula é o decimal e os pontos são milhar (`1.234,5678`).
 * - Sem vírgula e com um só ponto: é decimal (`0.5`, `12.75`, colado de fonte
 *   americana), exceto quando parece milhar (`1.500`, `12.000`: 1–3 dígitos, ponto,
 *   exatamente 3 dígitos, sem zero à esquerda).
 * - Vários pontos sem vírgula: milhar (`1.234.567`).
 *
 * Em inglês valem as mesmas regras com os papéis trocados (`1,234.5`; `0,5` colado de fonte
 * brasileira é decimal). Espaços (o milhar do pt-PT) são ignorados.
 */
export function parseDecimalInput(
  text: string,
  options: { allowNegative?: boolean; maxDecimals?: number } = {},
): string | null {
  let s = text.trim().replace(/\s+/g, '');
  if (s === '') return null;
  if (separators().decimal === '.') s = s.replace(/[.,]/g, (c) => (c === '.' ? ',' : '.'));
  let negative = false;
  if (/^[-−]/.test(s)) {
    negative = true;
    s = s.slice(1);
  } else if (s.startsWith('+')) {
    s = s.slice(1);
  }
  let int: string;
  let frac = '';
  if (s.includes(',')) {
    const parts = s.split(',');
    if (parts.length !== 2) return null;
    int = parts[0] ?? '';
    frac = parts[1] ?? '';
    if (!/^\d*$/.test(frac)) return null;
    if (int.includes('.')) {
      if (!validGroups(int)) return null;
      int = int.replaceAll('.', '');
    }
    if (!/^\d*$/.test(int)) return null;
  } else {
    const dots = s.split('.').length - 1;
    if (dots === 0) {
      if (!/^\d+$/.test(s)) return null;
      int = s;
    } else if (dots === 1) {
      const [a = '', b = ''] = s.split('.');
      if (!/^\d*$/.test(a) || !/^\d*$/.test(b)) return null;
      const looksLikeThousands = /^[1-9]\d{0,2}$/.test(a) && b.length === 3;
      if (looksLikeThousands) {
        int = a + b;
      } else {
        int = a;
        frac = b;
      }
    } else {
      if (!validGroups(s)) return null;
      int = s.replaceAll('.', '');
    }
  }
  if (int === '' && frac === '') return null;
  if (options.maxDecimals != null && frac.length > options.maxDecimals) return null;
  if (negative && !options.allowNegative) return null;
  const d = parseDecimal(`${negative ? '-' : ''}${int || '0'}${frac ? '.' + frac : ''}`);
  return d ? decToString(d) : null;
}

// ── Exibição no formato do idioma ────────────────────────────────

export interface FormatDecimalOptions {
  /** Casas mínimas (completa com zeros). Padrão 0. */
  minDecimals?: number;
  /** Casas máximas (arredonda metade para longe do zero). Padrão: todas. */
  maxDecimals?: number;
  /** `+` em positivos. */
  signed?: boolean;
}

/** `"1234.5"` → `"1.234,5"` (`"1,234.5"` em inglês); sinal negativo com U+2212. Texto inválido → `—`. */
export function formatDecimal(
  text: string | null | undefined,
  options: FormatDecimalOptions = {},
): string {
  const d0 = parseDecimal(text);
  if (!d0) return '—';
  const min = options.minDecimals ?? 0;
  let d = options.maxDecimals != null ? roundDec(d0, options.maxDecimals) : d0;
  d = normalizeDec(d);
  if (d.scale < min) d = { units: d.units * pow10(min - d.scale), scale: min };
  const negative = d.units < 0n;
  const digits = abs(d.units)
    .toString()
    .padStart(d.scale + 1, '0');
  const sep = separators();
  const rawInt = digits.slice(0, digits.length - d.scale);
  const int =
    rawInt.length >= sep.minGroup ? rawInt.replace(/\B(?=(\d{3})+(?!\d))/g, sep.group) : rawInt;
  const frac = d.scale > 0 ? sep.decimal + digits.slice(digits.length - d.scale) : '';
  const sign = negative ? MINUS : options.signed && d.units > 0n ? '+' : '';
  return sign + int + frac;
}

/**
 * Quantidade de cotas/ações: inteiros sem casas (`1.200`), frações com até 8
 * casas e sem zeros sobrando (`0,05231`, `12,5`).
 */
export function formatQuantity(text: string | null | undefined): string {
  return formatDecimal(text, { maxDecimals: 8 });
}

/**
 * Preço unitário na moeda do ativo: pelo menos 2 casas, até 8 quando o preço
 * tem mais precisão (PU do Tesouro, cripto, frações de centavo).
 * `formatUnitPrice("32.4", "BRL")` → `R$ 32,40`; `("0.00001234", "USD")` → `USD 0,00001234`.
 */
export function formatUnitPrice(
  text: string | null | undefined,
  currency: string = baseCurrency(),
  options: { hideCurrency?: boolean; maxDecimals?: number } = {},
): string {
  const min = minorUnits(currency);
  const body = formatDecimal(text, {
    minDecimals: min,
    maxDecimals: Math.max(min, options.maxDecimals ?? 8),
  });
  if (body === '—' || options.hideCurrency) return body;
  const negative = body.startsWith(MINUS);
  return `${negative ? MINUS : ''}${currencyPrefix(currency)} ${negative ? body.slice(1) : body}`;
}

/**
 * Texto canônico → valor no formato do idioma para preencher um campo de edição
 * (`"1234.5"` → `"1234,5"`, sem separador de milhar para não confundir a releitura).
 */
export function decimalToInput(text: string | null | undefined): string {
  const d = parseDecimal(text);
  if (!d) return '';
  return decToString(d).replace('.', separators().decimal);
}

/**
 * Só para desenho (gráficos): decimal → `number`. Nunca use o resultado em
 * contas de dinheiro exibidas.
 */
export function decimalToNumber(text: string | null | undefined): number | null {
  const d = parseDecimal(text);
  if (!d) return null;
  return Number(decToString(d));
}
