/** Rules list: factory regular expressions read as plain words. */
import { describe, expect, it } from 'vitest';

import type { RuleInput } from '@/api/generated/RuleInput';

import { plainSummary, regexTerms } from './plainSummary';

const rule = (conditions: RuleInput['conditions']): RuleInput =>
  ({ conditions, match_all: true }) as RuleInput;

describe('regexTerms', () => {
  it('turns an alternation into words, dropping optional and grouped pieces', () => {
    expect(
      regexTerms(
        String.raw`\b(ifood|ifd\*|rappi|uber\s*\*?\s*eats|ze\s+delivery|aiqfome|james\s+delivery)`,
      ),
    ).toEqual(['ifood', 'ifd*', 'rappi', 'uber eats', 'ze delivery', 'aiqfome', 'james delivery']);
    expect(
      regexTerms(String.raw`\b(vivo|claro|tim\s|tim$|oi\s*movel|recarga\s+(vivo|claro|tim|oi))`),
    ).toEqual(['vivo', 'claro', 'tim', 'oi movel']);
    expect(regexTerms(String.raw`\b(uber|lyft)\b`)).toEqual(['uber', 'lyft']);
    expect(regexTerms(String.raw`(\brestaurante|\bpizzaria|grill\b|burger\b)`)).toEqual([
      'restaurante',
      'pizzaria',
      'grill',
      'burger',
    ]);
    expect(regexTerms(String.raw`\biof\b`)).toEqual(['iof']);
    expect(regexTerms(String.raw`max\.com`)).toEqual(['max.com']);
    expect(regexTerms(String.raw`^\d{4}.*$`)).toEqual([]);
  });
});

describe('plainSummary', () => {
  it('reads a factory rule as words and the sign of the amount as in or out', () => {
    expect(
      plainSummary(
        rule([
          {
            field: 'description',
            op: 'regex',
            value: String.raw`\b(ifood|ifd\*|rappi|uber\s*\*?\s*eats|ze\s+delivery|aiqfome)`,
          },
          { field: 'amount', op: 'lte', value: -1 },
        ] as RuleInput['conditions']),
      ),
    ).toBe('Descrição contém ifood, ifd*, rappi, uber eats e mais 2 e é uma saída');
    expect(
      plainSummary(rule([{ field: 'amount', op: 'gte', value: 1 }] as RuleInput['conditions'])),
    ).toBe('é uma entrada');
  });

  it('keeps unreadable patterns and other conditions honest', () => {
    expect(
      plainSummary(
        rule([
          { field: 'description', op: 'regex', value: String.raw`^\d{4}` },
        ] as RuleInput['conditions']),
      ),
    ).toBe('Descrição segue um padrão (veja na edição)');
    expect(
      plainSummary(
        rule([{ field: 'description', op: 'contains', value: 'uber' }] as RuleInput['conditions']),
      ),
    ).toBe('Descrição contém “uber”');
  });
});
