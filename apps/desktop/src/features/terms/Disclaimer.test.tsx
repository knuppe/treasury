/**
 * Tax notice: source, date and agency of the country, and the known discrepancies for the US and
 * Portugal next to the number (docs/tax-validation-countries.md, Termos 5.1).
 */
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { TaxNotice } from './Disclaimer';

const note = () => screen.getByRole('note').textContent ?? '';

describe('TaxNotice', () => {
  it('in the US, says what the estimate does not yet consider', () => {
    render(<TaxNotice source="IRS" date="2026" country="US" />);
    expect(note()).toContain('do IRS');
    expect(note()).toContain('head of household');
    expect(note()).toContain('EITC');
  });

  it('in Portugal, says what is missing and that it counts the general family expenses', () => {
    render(<TaxNotice source="AT" date="2026" country="PT" />);
    expect(note()).toContain('IRS Jovem');
    expect(note()).toContain('despesas gerais familiares');
    expect(note()).toContain('mínimo de existência');
  });

  it('in Brazil and in the other countries, only the source notice', () => {
    render(<TaxNotice source="Receita" date="2026" country="BR" />);
    expect(note()).toContain('da Receita Federal');
    expect(note()).not.toContain('considera');
  });
});
