/**
 * Aviso de imposto: fonte, data e órgão do país, e as divergências conhecidas dos EUA e de
 * Portugal junto do número (docs/tax-validation-countries.md, Termos 5.1).
 */
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { TaxNotice } from './Disclaimer';

const note = () => screen.getByRole('note').textContent ?? '';

describe('TaxNotice', () => {
  it('nos EUA, diz o que a estimativa ainda não considera', () => {
    render(<TaxNotice source="IRS" date="2026" country="US" />);
    expect(note()).toContain('do IRS');
    expect(note()).toContain('head of household');
    expect(note()).toContain('EITC');
  });

  it('em Portugal, diz o que falta e que conta com as despesas gerais familiares', () => {
    render(<TaxNotice source="AT" date="2026" country="PT" />);
    expect(note()).toContain('IRS Jovem');
    expect(note()).toContain('despesas gerais familiares');
    expect(note()).toContain('mínimo de existência');
  });

  it('no Brasil e nos outros países, só o aviso da fonte', () => {
    render(<TaxNotice source="Receita" date="2026" country="BR" />);
    expect(note()).toContain('da Receita Federal');
    expect(note()).not.toContain('considera');
  });
});
