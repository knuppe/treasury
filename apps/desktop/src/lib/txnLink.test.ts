import { describe, expect, it } from 'vitest';

import { parseTxnLink, txnLink } from './txnLink';

describe('link de drill-down para lançamentos', () => {
  it('vai e volta sem perder filtros', () => {
    const f = {
      date_from: '2026-10-01',
      date_to: '2026-10-31',
      basis: 'cash' as const,
      category_ids: ['c1', 'c2'],
      member_ids: ['m1'],
      uncategorized: true,
    };
    const link = txnLink(f);
    expect(link.startsWith('/lancamentos?')).toBe(true);
    expect(parseTxnLink(link.split('?')[1]!)).toEqual(f);
  });

  it('sem filtros abre a tela limpa', () => {
    expect(txnLink({})).toBe('/lancamentos');
    expect(parseTxnLink('')).toEqual({});
  });
});
