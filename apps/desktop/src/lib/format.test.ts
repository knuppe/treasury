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
  it('flexiona o substantivo e a palavra colada nele', () => {
    expect(pluralize('lançamento')).toBe('lançamentos');
    expect(pluralize('parte da divisão')).toBe('partes da divisão');
    expect(pluralize('divisão de lançamento')).toBe('divisões de lançamento');
    expect(pluralize('titular')).toBe('titulares');
    expect(pluralize('bem físico')).toBe('bens físicos');
    expect(pluralize('dados do cartão')).toBe('dados do cartão');
    expect(pluralize('tag')).toBe('tags');
  });

  it('usa o singular só para 1', () => {
    expect(countLabel(1, 'conta')).toBe('1 conta');
    expect(countLabel(0, 'conta')).toBe('0 contas');
    expect(countLabel(3, 'motor ligado', 'motores ligados')).toBe('3 motores ligados');
    expect(countLabel(1200, 'registro')).toBe('1.200 registros');
  });
});

describe('formatMoney', () => {
  it('formata BRL com R$, milhar com ponto e decimais com vírgula', () => {
    expect(formatMoney(123456, 'BRL')).toBe('R$ 1.234,56');
    expect(formatMoney(0, 'BRL')).toBe('R$ 0,00');
    expect(formatMoney(5, 'BRL')).toBe('R$ 0,05');
    expect(formatMoney(100, 'BRL')).toBe('R$ 1,00');
    expect(formatMoney(123456789012, 'BRL')).toBe('R$ 1.234.567.890,12');
  });

  it('usa BRL como moeda padrão', () => {
    expect(formatMoney(99)).toBe('R$ 0,99');
  });

  it('mostra o código ISO para outras moedas, no padrão pt-BR', () => {
    expect(formatMoney(123456, 'USD')).toBe('USD 1.234,56');
    expect(formatMoney(123456, 'eur')).toBe('EUR 1.234,56');
  });

  it('respeita as casas decimais de cada moeda', () => {
    expect(formatMoney(1234, 'JPY')).toBe('JPY 1.234');
    expect(formatMoney(1234567, 'KWD')).toBe('KWD 1.234,567');
  });

  it('usa o sinal de menos U+2212, nunca o hífen', () => {
    const text = formatMoney(-123456, 'BRL');
    expect(text).toBe(`${MINUS}R$ 1.234,56`);
    expect(text.codePointAt(0)).toBe(0x2212);
    expect(text).not.toContain('-');
  });

  it('mostra + quando signed e o valor é positivo', () => {
    expect(formatMoney(2500, 'BRL', { signed: true })).toBe('+R$ 25,00');
    expect(formatMoney(0, 'BRL', { signed: true })).toBe('R$ 0,00');
    expect(formatMoney(-2500, 'BRL', { signed: true })).toBe(`${MINUS}R$ 25,00`);
  });

  it('pode omitir a moeda', () => {
    expect(formatMoney(-123456, 'BRL', { hideCurrency: true })).toBe(`${MINUS}1.234,56`);
  });

  it('recusa valores que não são inteiros (dinheiro nunca em ponto flutuante)', () => {
    expect(() => formatMoney(12.5, 'BRL')).toThrow(RangeError);
    expect(() => formatMoney(Number.NaN, 'BRL')).toThrow(RangeError);
    expect(() => formatMoney(2 ** 53, 'BRL')).toThrow(RangeError);
  });
});

describe('parseMoney', () => {
  it('lê o formato brasileiro e devolve centavos', () => {
    expect(parseMoney('1.234,56')).toBe(123456);
    expect(parseMoney('1234,56')).toBe(123456);
    expect(parseMoney('0,5')).toBe(50);
    expect(parseMoney(',5')).toBe(50);
    expect(parseMoney('10')).toBe(1000);
    expect(parseMoney('1.234')).toBe(123400);
    expect(parseMoney('1.234.567,89')).toBe(123456789);
  });

  it('aceita prefixo de moeda, espaços e sinal', () => {
    expect(parseMoney('R$ 1.234,56')).toBe(123456);
    expect(parseMoney('  R$1.234,56 ')).toBe(123456);
    expect(parseMoney('USD 10,00', 'USD')).toBe(1000);
    expect(parseMoney('-1.234,56')).toBe(-123456);
    expect(parseMoney(`${MINUS}12,30`)).toBe(-1230);
    expect(parseMoney('-R$ 12,30')).toBe(-1230);
    expect(parseMoney('R$ -12,30')).toBe(-1230);
    expect(parseMoney('+5,00')).toBe(500);
  });

  it('aceita ponto decimal quando não há vírgula e há 1–2 dígitos depois', () => {
    expect(parseMoney('12.50')).toBe(1250);
    expect(parseMoney('12.5')).toBe(1250);
  });

  it('é o inverso de formatMoney', () => {
    for (const cents of [0, 1, 99, 100, 123456, -98765, 100000000]) {
      expect(parseMoney(formatMoney(cents, 'BRL'))).toBe(cents);
    }
  });

  it('devolve null para textos inválidos', () => {
    expect(parseMoney('')).toBeNull();
    expect(parseMoney('   ')).toBeNull();
    expect(parseMoney('abc')).toBeNull();
    expect(parseMoney('1,2,3')).toBeNull();
    expect(parseMoney('1,234')).toBeNull(); // mais de 2 casas
    expect(parseMoney('12.34.5')).toBeNull();
    expect(parseMoney('1.23,45')).toBeNull(); // milhar malformado
    expect(parseMoney('--5')).toBeNull();
    expect(parseMoney(',')).toBeNull();
  });

  it('respeita as casas decimais da moeda', () => {
    expect(parseMoney('1.234', 'JPY')).toBe(1234);
    expect(parseMoney('1,5', 'JPY')).toBeNull();
    expect(parseMoney('1,234', 'KWD')).toBe(1234);
  });
});

describe('formatPercent', () => {
  it('formata razões como percentual pt-BR', () => {
    expect(formatPercent(0.125)).toBe('12,5%');
    expect(formatPercent(1)).toBe('100,0%');
    expect(formatPercent(0.0456, { decimals: 2 })).toBe('4,56%');
    expect(formatPercent(12.3456, { decimals: 0 })).toBe('1.235%');
  });

  it('usa U+2212 para negativos e + opcional', () => {
    expect(formatPercent(-0.031)).toBe(`${MINUS}3,1%`);
    expect(formatPercent(0.031, { signed: true })).toBe('+3,1%');
    expect(formatPercent(-0.00001)).toBe('0,0%');
  });

  it('não inventa número para valores não finitos', () => {
    expect(formatPercent(Number.NaN)).toBe('—');
  });
});

describe('formatNumber', () => {
  it('formata com milhar e sinal tipográfico', () => {
    expect(formatNumber(1234567)).toBe('1.234.567');
    expect(formatNumber(-1234.5, 1)).toBe(`${MINUS}1.234,5`);
  });
});

describe('datas', () => {
  it('formatDate converte ISO para dd/mm/aaaa', () => {
    expect(formatDate('2026-10-14')).toBe('14/10/2026');
    expect(formatDate('2024-02-29')).toBe('29/02/2024');
  });

  it('formatDate recusa datas inválidas', () => {
    expect(() => formatDate('14/10/2026')).toThrow(RangeError);
    expect(() => formatDate('2025-02-29')).toThrow(RangeError);
    expect(() => formatDate('2026-13-01')).toThrow(RangeError);
  });

  it('parseDate converte dd/mm/aaaa para ISO', () => {
    expect(parseDate('14/10/2026')).toBe('2026-10-14');
    expect(parseDate('1/2/2026')).toBe('2026-02-01');
    expect(parseDate(' 29/02/2024 ')).toBe('2024-02-29');
  });

  it('parseDate devolve null para datas inexistentes ou malformadas', () => {
    expect(parseDate('31/02/2026')).toBeNull();
    expect(parseDate('29/02/2025')).toBeNull();
    expect(parseDate('00/10/2026')).toBeNull();
    expect(parseDate('14/13/2026')).toBeNull();
    expect(parseDate('2026-10-14')).toBeNull();
    expect(parseDate('14/10/26')).toBeNull();
    expect(parseDate('')).toBeNull();
  });

  it('parseDate é o inverso de formatDate', () => {
    for (const iso of ['2026-01-01', '2026-10-14', '2000-02-29', '1999-12-31']) {
      expect(parseDate(formatDate(iso))).toBe(iso);
    }
  });
});

describe('meses', () => {
  it('por extenso', () => {
    expect(formatMonthLong('2026-10')).toBe('outubro de 2026');
    expect(formatMonthLong('2026-03-15')).toBe('março de 2026');
  });

  it('abreviado', () => {
    expect(formatMonthShort('2026-10')).toBe('out/26');
    expect(formatMonthShort('2009-02-01')).toBe('fev/09');
  });

  it('recusa meses inválidos', () => {
    expect(() => formatMonthLong('2026-13')).toThrow(RangeError);
    expect(() => formatMonthShort('outubro')).toThrow(RangeError);
  });
});

describe('formatação por idioma (docs/i18n-design.md)', () => {
  afterEach(() => setFormatLocale('pt-BR'));

  it('inglês: vírgula de milhar, ponto decimal, mês antes do dia', () => {
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

  it('espanhol e pt-PT', () => {
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

  it('pt-BR continua igual', () => {
    expect(formatMoney(123456, 'BRL')).toBe('R$ 1.234,56');
    expect(formatMoney(123456, 'USD')).toBe('USD 1.234,56');
    expect(formatMonthLong('2026-10')).toBe('outubro de 2026');
  });
});
