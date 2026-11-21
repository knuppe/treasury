/** Mock de `fx.*`: taxas de câmbio (manuais) e conversão. */
import type { FxRate } from '@/api/generated/FxRate';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { nowUtc, parseYmd } from './ledger/dates';
import { convert } from './ledger/domain';
import { invalid, notFound, num, str, text } from './ledger/errors';
import { type FxRow, del, find, newId, patch, put, rows, write } from './ledger/store';

function toRate(r: FxRow): FxRate {
  return {
    id: r.id,
    base: r.base,
    quote: r.quote,
    date: r.date,
    rate: r.rate,
    source: r.source,
    fetched_at: r.fetched_at,
  };
}

function currency(v: unknown): string {
  const c = text(v).trim().toUpperCase();
  if (!/^[A-Z]{3}$/.test(c))
    invalid(`moeda inválida: ${text(v)} (use o código ISO, ex.: BRL, USD)`);
  return c;
}

function parseRate(s: string): string {
  const v = s.trim().replace(',', '.');
  if (!/^\d+(\.\d+)?$/.test(v)) invalid(`taxa inválida: ${s}`);
  if (Number(v) <= 0) invalid('a taxa de câmbio deve ser positiva');
  return String(Number(v));
}

registerMockHandlers(
  guarded({
    'fx.list': (args) =>
      rows('fx_rate')
        .filter((r) => !args.base || r.base === args.base)
        .filter((r) => !args.quote || r.quote === args.quote)
        .sort((a, b) => b.date.localeCompare(a.date))
        .slice(0, Number(args.limit ?? 200))
        .map(toRate),
    'fx.convert': (args) => {
      parseYmd(str(args, 'date'));
      return convert(
        num(args, 'amount'),
        currency(args.from),
        currency(args.to),
        str(args, 'date'),
      );
    },
    'fx.set': (args) =>
      write('Câmbio informado manualmente', () => {
        const base = currency(args.base);
        const quote = currency(args.quote);
        if (base === quote) invalid('as moedas devem ser diferentes');
        const date = str(args, 'date');
        parseYmd(date);
        const rate = parseRate(str(args, 'rate'));
        const now = nowUtc();
        const cur = rows('fx_rate').find(
          (r) => r.base === base && r.quote === quote && r.date === date && r.source === 'manual',
        );
        const row = cur
          ? patch('fx_rate', cur.id, { rate, updated_at: now })
          : put('fx_rate', {
              id: newId(),
              base,
              quote,
              date,
              rate,
              source: 'manual',
              fetched_at: now,
              created_at: now,
              updated_at: now,
            });
        return toRate(row);
      }).value,
    'fx.delete': (args) => {
      const id = str(args, 'id');
      write('Câmbio removido', () => {
        if (!find('fx_rate', id)) notFound('taxa de câmbio');
        del('fx_rate', id);
      });
      return null;
    },
  }),
);
