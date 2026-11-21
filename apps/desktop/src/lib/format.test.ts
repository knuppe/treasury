import { afterEach, describe, expect, it } from 'vitest';

import {
  countLabel,
  formatDate,
  formatDateLong,
  formatMoney,
  formatMonthLong,
  formatMonthShort,
  formatNumber,
  formatPercent,
  MINUS,
  parseDate,
  parseMoney,
  pluralize,
  setFormatLocale,
} from './format';

describe('pluralize e countLabel', () => {
  it('inflects the noun and the word attached to it', () => {
    expect(pluralize('lançamento')).toBe('lançamentos');
    expect(pluralize('parte da divisão')).toBe('partes da divisão');
    expect(pluralize('divisão de lançamento')).toBe('divisões de lançamento');
    expect(pluralize('titular')).toBe('titulares');
    expect(pluralize('bem físico')).toBe('bens físicos');
    expect(pluralize('dados do cartão')).toBe('dados do cartão');
    expect(pluralize('tag')).toBe('tags');
  });

  it('uses the singular only for 1', () => {
    expect(countLabel(1, 'conta')).toBe('1 conta');
    expect(countLabel(0, 'conta')).toBe('0 contas');
    expect(countLabel(3, 'motor ligado', 'motores ligados')).toBe('3 motores ligados');
    expect(countLabel(1200, 'registro')).toBe('1.200 registros');
  });
});

describe('formatMoney', () => {
  it('formats BRL with R$, thousands with a dot and decimals with a comma', () => {
    expect(formatMoney(123456, 'BRL')).toBe('R$ 1.234,56');
    expect(formatMoney(0, 'BRL')).toBe('R$ 0,00');
    expect(formatMoney(5, 'BRL')).toBe('R$ 0,05');
    expect(formatMoney(100, 'BRL')).toBe('R$ 1,00');
    expect(formatMoney(123456789012, 'BRL')).toBe('R$ 1.234.567.890,12');
  });

  it('uses BRL as the default currency', () => {
    expect(formatMoney(99)).toBe('R$ 0,99');
  });

  it('shows the ISO code for other currencies, in the pt-BR style', () => {
    expect(formatMoney(123456, 'USD')).toBe('USD 1.234,56');
    expect(formatMoney(123456, 'eur')).toBe('EUR 1.234,56');
  });

  it('respects the decimal places of each currency', () => {
    expect(formatMoney(1234, 'JPY')).toBe('JPY 1.234');
    expect(formatMoney(1234567, 'KWD')).toBe('KWD 1.234,567');
  });

  it('uses the U+2212 minus sign, never the hyphen', () => {
    const text = formatMoney(-123456, 'BRL');
    expect(text).toBe(`${MINUS}R$ 1.234,56`);
    expect(text.codePointAt(0)).toBe(0x2212);
    expect(text).not.toContain('-');
  });

  it('shows + when signed and the value is positive', () => {
    expect(formatMoney(2500, 'BRL', { signed: true })).toBe('+R$ 25,00');
    expect(formatMoney(0, 'BRL', { signed: true })).toBe('R$ 0,00');
    expect(formatMoney(-2500, 'BRL', { signed: true })).toBe(`${MINUS}R$ 25,00`);
  });

  it('can omit the currency', () => {
    expect(formatMoney(-123456, 'BRL', { hideCurrency: true })).toBe(`${MINUS}1.234,56`);
  });

  it('rejects non-integer values (money is never floating point)', () => {
    expect(() => formatMoney(12.5, 'BRL')).toThrow(RangeError);
    expect(() => formatMoney(Number.NaN, 'BRL')).toThrow(RangeError);
    expect(() => formatMoney(2 ** 53, 'BRL')).toThrow(RangeError);
  });
});

describe('parseMoney', () => {
  it('reads the Brazilian format and returns cents', () => {
    expect(parseMoney('1.234,56')).toBe(123456);
    expect(parseMoney('1234,56')).toBe(123456);
    expect(parseMoney('0,5')).toBe(50);
    expect(parseMoney(',5')).toBe(50);
    expect(parseMoney('10')).toBe(1000);
    expect(parseMoney('1.234')).toBe(123400);
    expect(parseMoney('1.234.567,89')).toBe(123456789);
  });

  it('accepts a currency prefix, spaces and sign', () => {
    expect(parseMoney('R$ 1.234,56')).toBe(123456);
    expect(parseMoney('  R$1.234,56 ')).toBe(123456);
    expect(parseMoney('USD 10,00', 'USD')).toBe(1000);
    expect(parseMoney('-1.234,56')).toBe(-123456);
    expect(parseMoney(`${MINUS}12,30`)).toBe(-1230);
    expect(parseMoney('-R$ 12,30')).toBe(-1230);
    expect(parseMoney('R$ -12,30')).toBe(-1230);
    expect(parseMoney('+5,00')).toBe(500);
  });

  it('accepts a decimal point when there is no comma and 1–2 digits follow', () => {
    expect(parseMoney('12.50')).toBe(1250);
    expect(parseMoney('12.5')).toBe(1250);
  });

  it('is the inverse of formatMoney', () => {
    for (const cents of [0, 1, 99, 100, 123456, -98765, 100000000]) {
      expect(parseMoney(formatMoney(cents, 'BRL'))).toBe(cents);
    }
  });

  it('returns null for invalid texts', () => {
    expect(parseMoney('')).toBeNull();
    expect(parseMoney('   ')).toBeNull();
    expect(parseMoney('abc')).toBeNull();
    expect(parseMoney('1,2,3')).toBeNull();
    expect(parseMoney('1,234')).toBeNull(); // more than 2 decimals
    expect(parseMoney('12.34.5')).toBeNull();
    expect(parseMoney('1.23,45')).toBeNull(); // malformed thousands
    expect(parseMoney('--5')).toBeNull();
    expect(parseMoney(',')).toBeNull();
  });

  it('respects the currency decimal places', () => {
    expect(parseMoney('1.234', 'JPY')).toBe(1234);
    expect(parseMoney('1,5', 'JPY')).toBeNull();
    expect(parseMoney('1,234', 'KWD')).toBe(1234);
  });
});

describe('formatPercent', () => {
  it('formats ratios as pt-BR percent', () => {
    expect(formatPercent(0.125)).toBe('12,5%');
    expect(formatPercent(1)).toBe('100,0%');
    expect(formatPercent(0.0456, { decimals: 2 })).toBe('4,56%');
    expect(formatPercent(12.3456, { decimals: 0 })).toBe('1.235%');
  });

  it('uses U+2212 for negatives and an optional +', () => {
    expect(formatPercent(-0.031)).toBe(`${MINUS}3,1%`);
    expect(formatPercent(0.031, { signed: true })).toBe('+3,1%');
    expect(formatPercent(-0.00001)).toBe('0,0%');
  });

  it('does not invent a number for non-finite values', () => {
    expect(formatPercent(Number.NaN)).toBe('—');
  });
});

describe('formatNumber', () => {
  it('formats with thousands and a typographic sign', () => {
    expect(formatNumber(1234567)).toBe('1.234.567');
    expect(formatNumber(-1234.5, 1)).toBe(`${MINUS}1.234,5`);
  });
});

describe('dates', () => {
  it('formatDate converts ISO to dd/mm/yyyy', () => {
    expect(formatDate('2026-10-14')).toBe('14/10/2026');
    expect(formatDate('2024-02-29')).toBe('29/02/2024');
  });

  it('formatDate rejects invalid dates', () => {
    expect(() => formatDate('14/10/2026')).toThrow(RangeError);
    expect(() => formatDate('2025-02-29')).toThrow(RangeError);
    expect(() => formatDate('2026-13-01')).toThrow(RangeError);
  });

  it('parseDate converts dd/mm/yyyy to ISO', () => {
    expect(parseDate('14/10/2026')).toBe('2026-10-14');
    expect(parseDate('1/2/2026')).toBe('2026-02-01');
    expect(parseDate(' 29/02/2024 ')).toBe('2024-02-29');
  });

  it('parseDate returns null for nonexistent or malformed dates', () => {
    expect(parseDate('31/02/2026')).toBeNull();
    expect(parseDate('29/02/2025')).toBeNull();
    expect(parseDate('00/10/2026')).toBeNull();
    expect(parseDate('14/13/2026')).toBeNull();
    expect(parseDate('2026-10-14')).toBeNull();
    expect(parseDate('14/10/26')).toBeNull();
    expect(parseDate('')).toBeNull();
  });

  it('parseDate is the inverse of formatDate', () => {
    for (const iso of ['2026-01-01', '2026-10-14', '2000-02-29', '1999-12-31']) {
      expect(parseDate(formatDate(iso))).toBe(iso);
    }
  });
});

describe('months', () => {
  it('in full', () => {
    expect(formatMonthLong('2026-10')).toBe('outubro de 2026');
    expect(formatMonthLong('2026-03-15')).toBe('março de 2026');
  });

  it('abbreviated', () => {
    expect(formatMonthShort('2026-10')).toBe('out/26');
    expect(formatMonthShort('2009-02-01')).toBe('fev/09');
  });

  it('rejects invalid months', () => {
    expect(() => formatMonthLong('2026-13')).toThrow(RangeError);
    expect(() => formatMonthShort('outubro')).toThrow(RangeError);
  });
});

describe('formatting by language (docs/i18n-design.md)', () => {
  afterEach(() => setFormatLocale('pt-BR'));

  it('English: comma for thousands, decimal point, month before day', () => {
    setFormatLocale(
      'en',
      [
        'January',
        'February',
        'March',
        'April',
        'May',
        'June',
        'July',
        'August',
        'September',
        'October',
        'November',
        'December',
      ],
      ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'],
    );
    expect(formatMoney(123456, 'USD')).toBe('$1,234.56');
    expect(formatMoney(-500, 'USD')).toBe('−$5.00');
    expect(formatMoney(123456, 'BRL')).toBe('R$1,234.56');
    expect(formatMoney(123456, 'CAD')).toBe('CAD 1,234.56');
    expect(parseMoney('1,234.56', 'USD')).toBe(123456);
    expect(parseMoney('1,234', 'USD')).toBe(123400);
    expect(parseMoney('$12.5', 'USD')).toBe(1250);
    expect(parseMoney('1,23', 'USD')).toBeNull();
    expect(formatDate('2026-10-14')).toBe('10/14/2026');
    expect(parseDate('10/14/2026')).toBe('2026-10-14');
    expect(formatDateLong('2026-10-14')).toBe('October 14, 2026');
    expect(formatMonthLong('2026-10')).toBe('October 2026');
    expect(formatMonthShort('2026-10')).toBe('Oct 26');
    expect(formatPercent(0.125)).toBe('12.5%');
  });

  it('Spanish and pt-PT', () => {
    setFormatLocale('es', [
      'enero',
      'febrero',
      'marzo',
      'abril',
      'mayo',
      'junio',
      'julio',
      'agosto',
      'septiembre',
      'octubre',
      'noviembre',
      'diciembre',
    ]);
    expect(formatMoney(123456, 'USD')).toBe('US$ 1.234,56');
    expect(formatMoney(123456, 'EUR')).toBe('1.234,56 €');
    expect(formatMonthLong('2026-10')).toBe('octubre de 2026');
    expect(formatDate('2026-10-14')).toBe('14/10/2026');
    setFormatLocale('pt-PT');
    expect(formatMoney(123456, 'EUR')).toBe('1234,56 €');
    expect(formatMoney(1234567, 'EUR')).toBe('12 345,67 €');
    expect(parseMoney('12 345,67 €', 'EUR')).toBe(1234567);
  });

  it('pt-BR stays the same', () => {
    expect(formatMoney(123456, 'BRL')).toBe('R$ 1.234,56');
    expect(formatMoney(123456, 'USD')).toBe('USD 1.234,56');
    expect(formatMonthLong('2026-10')).toBe('outubro de 2026');
  });
});
