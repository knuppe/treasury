import { describe, expect, it } from 'vitest';

import { cetRate, roundPct, usApr } from '../debt/cet';
import { addMonths } from './dates';

// Made-up loan of docs/design/loan-cet.md: 12 installments of R$ 1.019,68 from 10/04/2026.
const flows = Array.from({ length: 12 }, (_, k) => ({
  date: addMonths('2026-04-10', k),
  amount: 101_968,
}));

describe('loan CET in the mock (mirror of debt/cet.rs)', () => {
  it('takes the upfront costs out of the amount released', () => {
    expect(roundPct(cetRate(1_085_000, '2026-03-10', flows)!)).toBe('25.19');
    expect(roundPct(cetRate(1_000_000, '2026-03-10', flows)!)).toBe('46.49');
  });

  it('computes the US APR as the monthly rate times twelve', () => {
    expect(usApr(1_000_000, '2026-03-10', flows)!).toBeCloseTo(0.39, 3);
  });

  it('rounds to two decimals', () => {
    expect(roundPct(0.1)).toBe('10.00');
    expect(roundPct(0.123_451)).toBe('12.35');
  });
});
