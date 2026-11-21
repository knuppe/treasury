/**
 * Mock de `fx.reference_rate`, `fx.exchange_quote`, `fx.exchange` e `fx.operations`
 * (espelho de `fx::exchange`): câmbio de referência na moeda da casa (a moeda base da família;
 * PTAX no Brasil), taxa aplicada ou referência + spread, IOF sobre o valor na moeda da casa,
 * tarifa e VET.
 */
import type { ExchangeQuote } from '@/api/generated/ExchangeQuote';
import type { FxOperation } from '@/api/generated/FxOperation';
import type { ReferenceRate } from '@/api/generated/ReferenceRate';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { today } from './ledger/dates';
import { baseCurrency, insertTxn } from './ledger/domain';
import { invalid, notFound, str } from './ledger/errors';
import { find, newId, rows, write } from './ledger/store';
import { createTransfer } from './transactions';

/** Reais por unidade (PTAX fictícia); fora do Brasil, o câmbio cruzado por ela. */
const PTAX: Record<string, number> = { BRL: 1, USD: 5.4321, EUR: 5.9012, GBP: 6.8811 };

let ops: FxOperation[] = [];
onMockReset(() => {
  ops = [];
});

const dec = (v: unknown): number | null => {
  if (typeof v !== 'string' || !v.trim()) return null;
  const n = Number(v.trim().replace(',', '.'));
  return Number.isFinite(n) ? n : invalid(`número inválido: ${v}`);
};
const round = (v: number) => Math.sign(v) * Math.round(Math.abs(v) + 1e-9);
const fmt = (n: number, dp: number) => String(Number(n.toFixed(dp)));

function quote(a: Record<string, unknown>): ExchangeQuote {
  const from = find('account', str(a, 'from_account_id')) ?? notFound('conta de origem');
  const to = find('account', str(a, 'to_account_id')) ?? notFound('conta de destino');
  if (from.currency === to.currency)
    invalid('câmbio é entre moedas diferentes; para a mesma moeda use transferência');
  const ref = dec(a.reference_rate);
  const spread = dec(a.spread_pct);
  const applied = dec(a.rate);
  const rate = applied ?? (ref != null ? ref * (1 + (spread ?? 0) / 100) : null);
  if (rate == null) invalid('informe a taxa aplicada ou a de referência (PTAX)');
  const home = baseCurrency();
  const brlFrom = from.currency === home;
  const brlTo = to.currency === home;
  let amountFrom: number;
  let amountTo: number;
  if (typeof a.amount_from === 'number' && a.amount_from > 0) {
    amountFrom = a.amount_from;
    amountTo = round(brlFrom ? amountFrom / rate : amountFrom * rate);
  } else if (typeof a.amount_to === 'number' && a.amount_to > 0) {
    amountTo = a.amount_to;
    amountFrom = round(brlFrom ? amountTo * rate : amountTo / rate);
  } else invalid('informe o valor que sai ou o valor que entra (positivo)');
  const iofPct = dec(a.iof_pct) ?? 0;
  const base = brlFrom ? amountFrom : brlTo ? amountTo : 0;
  const iof = iofPct > 0 ? round((base * iofPct) / 100) : 0;
  const fee = typeof a.fee === 'number' ? Math.max(0, a.fee) : 0;
  const total = amountFrom + fee + (brlFrom ? iof : 0);
  return {
    from_currency: from.currency,
    to_currency: to.currency,
    amount_from: amountFrom,
    amount_to: amountTo,
    rate: fmt(rate, 8),
    spread_pct:
      applied != null && ref
        ? fmt((applied / ref - 1) * 100, 4)
        : spread != null
          ? String(spread)
          : null,
    iof_amount: iof,
    fee_amount: fee,
    total_from: total,
    vet: brlFrom
      ? fmt(total / amountTo, 6)
      : brlTo
        ? fmt((amountTo - iof) / (amountFrom + fee), 6)
        : null,
  };
}

registerMockHandlers(
  guarded({
    'fx.reference_rate': (a): ReferenceRate => {
      const ccy = str(a, 'currency').toUpperCase();
      const home = baseCurrency();
      if (ccy === home) invalid(`a referência é de outra moeda para ${home}`);
      const unit = PTAX[ccy];
      const per = PTAX[home];
      const r =
        unit != null && per != null
          ? Number((unit / per).toFixed(4))
          : notFound(`sem cotação de referência de ${ccy}: informe a taxa à mão`);
      return {
        currency: ccy,
        sell: String(r),
        buy: String(Number((r - 0.0006).toFixed(4))),
        date: typeof a.date === 'string' ? a.date : today(),
        bulletin: home === 'BRL' ? 'Intermediário' : 'coletada',
        source: home === 'BRL' ? 'bcb_ptax' : 'ecb',
        stale: false,
      };
    },
    'fx.exchange_quote': quote,
    'fx.operations': (a) =>
      ops.filter(
        (o) =>
          !a.account_id || o.from_account_id === a.account_id || o.to_account_id === a.account_id,
      ),
    'fx.exchange': (a) => {
      const q = quote(a);
      const date = str(a, 'date');
      const rec = write(`Câmbio ${q.from_currency} → ${q.to_currency}`, () => {
        const ids = createTransfer({
          from_account_id: str(a, 'from_account_id'),
          to_account_id: str(a, 'to_account_id'),
          date,
          amount: q.amount_from,
          to_amount: q.amount_to,
          description: `Câmbio ${q.from_currency} → ${q.to_currency}`,
        });
        const cat = (name: string) =>
          rows('category').find((c) => c.kind === 'expense' && c.name === name)?.id ?? null;
        if (q.iof_amount > 0)
          insertTxn({
            account_id:
              q.from_currency === baseCurrency()
                ? str(a, 'from_account_id')
                : str(a, 'to_account_id'),
            date,
            amount: -q.iof_amount,
            currency: baseCurrency(),
            kind: 'expense',
            description: 'IOF do câmbio',
            category_id: cat('IOF'),
          });
        if (q.fee_amount > 0)
          insertTxn({
            account_id: str(a, 'from_account_id'),
            date,
            amount: -q.fee_amount,
            currency: q.from_currency,
            kind: 'expense',
            description: 'Tarifa do câmbio',
            category_id: cat('Tarifas bancárias'),
          });
        const op: FxOperation = {
          id: newId(),
          transfer_id: ids.transferId,
          date,
          from_account_id: str(a, 'from_account_id'),
          to_account_id: str(a, 'to_account_id'),
          from_currency: q.from_currency,
          to_currency: q.to_currency,
          amount_from: q.amount_from,
          amount_to: q.amount_to,
          rate: q.rate,
          reference_rate: typeof a.reference_rate === 'string' ? a.reference_rate : null,
          reference_source: typeof a.reference_source === 'string' ? a.reference_source : null,
          spread_pct: q.spread_pct,
          iof_pct: typeof a.iof_pct === 'string' ? a.iof_pct : null,
          iof_amount: q.iof_amount,
          fee_amount: q.fee_amount,
          vet: q.vet,
          notes: typeof a.notes === 'string' ? a.notes : null,
        };
        ops.unshift(op);
        return op;
      });
      return rec.value;
    },
  }),
);
