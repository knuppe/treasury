import { describe, expect, it } from 'vitest';

import {
  addDecimal,
  compareDecimal,
  decimalToInput,
  decimalToMinor,
  divideDecimal,
  formatDecimal,
  formatQuantity,
  formatUnitPrice,
  isPositiveDecimal,
  minorToDecimal,
  multiplyDecimal,
  parseDecimal,
  parseDecimalInput,
  productToMinor,
  subtractDecimal,
} from './decimal';

describe('exact decimal', () => {
  it('reads the core canonical text', () => {
    expect(parseDecimal('1234.5678')).toEqual({ units: 12345678n, scale: 4 });
    expect(parseDecimal('-0.00012')).toEqual({ units: -12n, scale: 5 });
    expect(parseDecimal('3')).toEqual({ units: 3n, scale: 0 });
    expect(parseDecimal('1,5')).toBeNull();
    expect(parseDecimal('')).toBeNull();
    expect(parseDecimal(null)).toBeNull();
  });

  it('compares, adds and multiplies without floating point', () => {
    expect(compareDecimal('0.1', '0.10')).toBe(0);
    expect(compareDecimal('100', '99.99999999')).toBe(1);
    expect(compareDecimal('0.035', '0.04')).toBe(-1);
    expect(addDecimal('0.1', '0.2')).toBe('0.3');
    expect(subtractDecimal('300', '0.5')).toBe('299.5');
    expect(multiplyDecimal('0.1', '3')).toBe('0.3');
    expect(isPositiveDecimal('0.00000001')).toBe(true);
    expect(isPositiveDecimal('0')).toBe(false);
  });

  it('converts to cents with half away from zero (like to_minor)', () => {
    expect(decimalToMinor('12.345', 'BRL')).toBe(1235);
    expect(decimalToMinor('12.344', 'BRL')).toBe(1234);
    expect(decimalToMinor('-12.345', 'BRL')).toBe(-1235);
    expect(decimalToMinor('0.005', 'BRL')).toBe(1);
    expect(decimalToMinor('1500', 'JPY')).toBe(1500);
    expect(decimalToMinor('7', 'BRL')).toBe(700);
    expect(minorToDecimal(1235, 'BRL')).toBe('12.35');
    expect(minorToDecimal(-5, 'BRL')).toBe('-0.05');
  });

  it('divides to find the unit price that gives the total value', () => {
    expect(divideDecimal('5315.2', '5000', 12)).toBe('1.06304');
    expect(divideDecimal('100', '3', 4)).toBe('33.3333');
    expect(divideDecimal('2', '3', 4)).toBe('0.6667');
    expect(divideDecimal('-1', '8', 2)).toBe('-0.13');
    // Goes back to the total through the cents, even with a repeating decimal.
    const pu = divideDecimal('1000', '3', 12);
    expect(productToMinor(['3', pu], 'BRL')).toBe(100_000);
    expect(() => divideDecimal('1', '0', 2)).toThrow();
  });

  it('computes the gross of an operation (qty × price) rounding only once', () => {
    // 0.1 × 0.3 in float would give 0.030000000000000002.
    expect(productToMinor(['0.1', '0.3'], 'BRL')).toBe(3);
    expect(productToMinor(['300', '32.45'], 'BRL')).toBe(973_500);
    expect(productToMinor(['0.035', '42150.37'], 'USD')).toBe(147_526);
    // With an exchange rate: 10.5 × 190.13 × 5.4321 = 10,844.4543165 → R$ 10,844.45
    expect(productToMinor(['10.5', '190.13', '5.4321'], 'BRL')).toBe(1_084_445);
  });

  it('reads what the person types in pt-BR', () => {
    expect(parseDecimalInput('1.234,5678')).toBe('1234.5678');
    expect(parseDecimalInput('1234,5')).toBe('1234.5');
    expect(parseDecimalInput('0,035')).toBe('0.035');
    expect(parseDecimalInput('12.75')).toBe('12.75');
    expect(parseDecimalInput('0.500')).toBe('0.5');
    expect(parseDecimalInput('1.500')).toBe('1500');
    expect(parseDecimalInput('1.234.567')).toBe('1234567');
    expect(parseDecimalInput(',5')).toBe('0.5');
    expect(parseDecimalInput('  300 ')).toBe('300');
    expect(parseDecimalInput('1.23.4')).toBeNull();
    expect(parseDecimalInput('12,3,4')).toBeNull();
    expect(parseDecimalInput('abc')).toBeNull();
    expect(parseDecimalInput('')).toBeNull();
    expect(parseDecimalInput('-3')).toBeNull();
    expect(parseDecimalInput('−3', { allowNegative: true })).toBe('-3');
    expect(parseDecimalInput('1,123', { maxDecimals: 2 })).toBeNull();
  });

  it('formats quantities: integers without decimals, fractions without trailing zeros', () => {
    expect(formatQuantity('1200')).toBe('1.200');
    expect(formatQuantity('0.05231')).toBe('0,05231');
    expect(formatQuantity('12.50')).toBe('12,5');
    expect(formatQuantity('0.123456789')).toBe('0,12345679');
    expect(formatQuantity(null)).toBe('—');
  });

  it('formats unit prices with 2 to 8 decimals and the currency prefix', () => {
    expect(formatUnitPrice('32.4', 'BRL')).toBe('R$ 32,40');
    expect(formatUnitPrice('2647.95812', 'BRL')).toBe('R$ 2.647,95812');
    expect(formatUnitPrice('106400', 'USD')).toBe('USD 106.400,00');
    expect(formatUnitPrice('0.00001234', 'USD')).toBe('USD 0,00001234');
    expect(formatUnitPrice('-1.5', 'BRL')).toBe('−R$ 1,50');
    expect(formatUnitPrice('32.4', 'BRL', { hideCurrency: true })).toBe('32,40');
    // Average price: up to 4 decimals.
    expect(formatUnitPrice('35.250575', 'BRL', { maxDecimals: 4 })).toBe('R$ 35,2506');
    expect(formatUnitPrice('24.5', 'BRL', { maxDecimals: 4 })).toBe('R$ 24,50');
  });

  it('formats generic decimals with a typographic sign', () => {
    expect(formatDecimal('-1234.5')).toBe('−1.234,5');
    expect(formatDecimal('110', { minDecimals: 2 })).toBe('110,00');
    expect(formatDecimal('0.125', { maxDecimals: 2 })).toBe('0,13');
    expect(formatDecimal('3', { signed: true })).toBe('+3');
    expect(decimalToInput('1234.5')).toBe('1234,5');
  });
});
