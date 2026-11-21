/** Mock de `installments.*`: parcelas detectadas pela descrição, como `installments/mod.rs`. */
import type { DetectedInstallments } from '@/api/generated/DetectedInstallments';
import type { DetectedSeries } from '@/api/generated/DetectedSeries';
import type { ProjectedInstallment } from '@/api/generated/ProjectedInstallment';
import type { SeriesKind } from '@/api/generated/SeriesKind';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addMonths, monthOf, nowUtc, today } from './ledger/dates';
import { baseCurrency, datesForReferenceMonth, statementDates } from './ledger/domain';
import { notFound, str } from './ledger/errors';
import { type TxnRow, del, find, put, rows, write } from './ledger/store';
import { normalizeDescription } from './ledger/text';

const KEYWORD =
  /[\s\-–:]*\b(?:parcela|parc|prest(?:acao|ação)?|prest\.?\s*emprest|pmt)\b\.?\s*(\d{1,3})\s*(?:\/|de)\s*(\d{1,3})\b/i;
const BARE = /[\s\-–:]+(\d{1,3})\s*\/\s*(\d{1,3})\s*$/;

interface Parsed {
  number: number;
  total: number;
  keyword: boolean;
  loan: boolean;
  bill: boolean;
  base: string;
}

export function parseInstallment(desc: string): Parsed | null {
  let m = KEYWORD.exec(desc);
  const keyword = Boolean(m);
  if (!m) m = BARE.exec(desc);
  if (!m) return null;
  const number = Number(m[1]);
  const total = Number(m[2]);
  if (number === 0 || total < 2 || number > total || total > (keyword ? 120 : 72)) return null;
  const before = desc.slice(0, m.index).replace(/[\s\-–:*]+$/, '');
  const after = desc.slice(m.index + m[0].length).trim();
  let base = [before, after].filter(Boolean).join(' ').trim();
  if (!base)
    base = m[0].slice(0, m[0].indexOf(m[1] ?? '')).replace(/^[\s\-–:*.]+|[\s\-–:*.]+$/g, '');
  const folded = normalizeDescription(desc);
  const bill =
    folded.includes('fatura') || folded.includes('parc fat') || folded.includes('parcelamento fat');
  const loan = ['emprest', 'financ', 'prest', 'consig'].some((w) => folded.includes(w)) && !bill;
  return { number, total, keyword, loan, bill, base };
}

const seriesKey = (account: string, base: string, total: number) =>
  `${account}|${normalizeDescription(base)}|${total}`;

const close = (a: number, b: number, total: number) =>
  Math.abs(a - b) <= total || Math.abs(a - b) * 100 <= Math.max(a, b);

const days = (a: string, b: string) =>
  Math.round((Date.parse(`${b}T00:00:00Z`) - Date.parse(`${a}T00:00:00Z`)) / 86_400_000);

export function detectInstallments(T = today()): DetectedInstallments {
  const base = baseCurrency();
  const ignored = new Set(rows('detected_installment_ignore').map((r) => r.id));
  const from = addMonths(T, -13);
  const groups = new Map<string, { t: TxnRow; p: Parsed }[]>();
  for (const t of rows('txn')) {
    if (t.date < from || t.date > T || t.amount >= 0 || t.status === 'planned') continue;
    if (t.installment_plan_id || t.transfer_id) continue;
    if (t.kind !== 'expense' && t.kind !== 'adjustment') continue;
    const p = parseInstallment(t.description);
    if (!p) continue;
    const key = seriesKey(t.account_id, p.base, p.total);
    if (ignored.has(key)) continue;
    const g = groups.get(key) ?? [];
    g.push({ t, p });
    groups.set(key, g);
  }
  const series: DetectedSeries[] = [];
  const sm = (x: { t: TxnRow }) =>
    find('card_statement', x.t.statement_id)?.reference_month ?? null;
  // Mês da 1ª parcela: o da fatura (ou da data) menos as parcelas já vistas.
  const origin = (x: { t: TxnRow; p: Parsed }) =>
    monthOf(addMonths(`${sm(x) ?? x.t.date.slice(0, 7)}-01`, -(x.p.number - 1)));
  const monthsBetween = (a: string, b: string) =>
    (Number(b.slice(0, 4)) - Number(a.slice(0, 4))) * 12 +
    Number(b.slice(5, 7)) -
    Number(a.slice(5, 7));
  // Separa as compras de um grupo como `installments::split_purchases`.
  const purchases = (list: { t: TxnRow; p: Parsed }[]) => {
    const sorted = [...list].sort(
      (a, b) =>
        origin(a).localeCompare(origin(b)) ||
        a.t.date.localeCompare(b.t.date) ||
        a.p.number - b.p.number,
    );
    const chains: { origin: string; rows: typeof list }[] = [];
    for (const x of sorted) {
      const o = origin(x);
      const fit = chains.find(
        (c) =>
          Math.abs(monthsBetween(c.origin, o)) <= 1 &&
          c.rows.every((y) => y.p.number !== x.p.number) &&
          c.rows.some((y) => close(-y.t.amount, -x.t.amount, x.p.total)),
      );
      if (fit) fit.rows.push(x);
      else chains.push({ origin: o, rows: [x] });
    }
    const seen = new Map<string, number>();
    return chains.map((c) => {
      const n = (seen.get(c.origin) ?? 0) + 1;
      seen.set(c.origin, n);
      c.rows.sort((a, b) => a.t.date.localeCompare(b.t.date) || a.p.number - b.p.number);
      return { suffix: n === 1 ? c.origin : `${c.origin}#${n}`, chain: c.rows };
    });
  };
  for (const [group, list] of groups)
    for (const { suffix, chain } of purchases(list)) {
      const key = `${group}|${suffix}`;
      if (ignored.has(key)) continue;
      const confirmed = chain.some((x, i) => {
        const y = chain[i + 1];
        if (!y || y.p.number !== x.p.number + 1) return false;
        const a = sm(x);
        const b = sm(y);
        if (a && b) return monthOf(addMonths(`${a}-01`, 1)) === b;
        const d = days(x.t.date, y.t.date);
        return d >= 20 && d <= 45;
      });
      const tail = chain.at(-1)!;
      if (!tail.p.keyword && !confirmed) continue;
      const k = tail.p.number;
      const n = tail.p.total;
      if (k >= n) continue;
      const acc = find('account', tail.t.account_id);
      const isCard = acc?.kind === 'credit_card';
      const card = isCard ? find('credit_card', tail.t.account_id) : undefined;
      const kind: SeriesKind = isCard
        ? tail.p.bill
          ? 'bill'
          : 'card'
        : tail.p.loan
          ? 'loan'
          : 'account';
      const amount = -tail.t.amount;
      const month = sm(tail);
      const projected: ProjectedInstallment[] = [];
      for (let i = 1; i <= n - k; i++) {
        if (card && month) {
          const m = monthOf(addMonths(`${month}-01`, i));
          const d = datesForReferenceMonth(m, card.closing_day, card.due_day);
          projected.push({ number: k + i, date: d.closing_date, due_date: d.due_date, amount });
        } else if (card) {
          const d = addMonths(tail.t.date, i);
          projected.push({
            number: k + i,
            date: d,
            due_date: statementDates(d, card.closing_day, card.due_day).due_date,
            amount,
          });
        } else {
          projected.push({
            number: k + i,
            date: addMonths(tail.t.date, i),
            due_date: null,
            amount,
          });
        }
      }
      const late = days(projected[0]!.date, T);
      if (late > 45) continue;
      const missing = late > 15;
      const lastP = projected.at(-1)!;
      const endMonth = (lastP.due_date ?? lastP.date).slice(0, 7);
      series.push({
        key,
        account_id: tail.t.account_id,
        account_name: acc?.name ?? '',
        kind,
        description: tail.p.base,
        member_id: tail.t.member_id,
        member_name: find('member', tail.t.member_id)?.name ?? null,
        number: k,
        total: n,
        amount,
        currency: tail.t.currency,
        last_date: tail.t.date,
        remaining_amount: amount * (n - k),
        remaining: n - k,
        end_month: endMonth,
        missing,
        confirmed_by_sequence: confirmed,
        // No cartão vale o vencimento: a fatura fechada e não importada continua a pagar.
        projected: projected.filter((p) => (p.due_date ?? p.date) > T || missing),
        txn_ids: chain.map((x) => x.t.id),
      });
    }
  series.sort(
    (a, b) => b.remaining_amount - a.remaining_amount || a.description.localeCompare(b.description),
  );
  const first = `${T.slice(0, 7)}-01`;
  const months = Array.from({ length: 12 }, (_, i) => monthOf(addMonths(first, i)));
  const byMonth = new Map(months.map((m) => [m, 0]));
  let total = 0;
  for (const s of series) {
    if (s.currency !== base) continue;
    total += s.remaining_amount;
    for (const p of s.projected) {
      const m = (p.due_date ?? p.date).slice(0, 7);
      if (byMonth.has(m)) byMonth.set(m, byMonth.get(m)! + p.amount);
    }
  }
  return {
    currency: base,
    series,
    remaining_total: total,
    by_month: [...byMonth].map(([month, amount]) => ({ month, amount })),
  };
}

registerMockHandlers(
  guarded({
    'installments.detected': () => detectInstallments(),
    'installments.ignored': () =>
      rows('detected_installment_ignore').map((r) => ({
        key: r.id,
        account_id: r.account_id,
        description: r.description,
      })),
    'installments.ignore': (args) => {
      const key = str(args, 'key');
      const s =
        detectInstallments().series.find((x) => x.key === key) ?? notFound('série de parcelas');
      write(`Não é parcela: ${s.description}`, () => {
        const now = nowUtc();
        put('detected_installment_ignore', {
          id: key,
          account_id: s.account_id,
          description: s.description,
          created_at: now,
          updated_at: now,
        });
      });
      return detectInstallments();
    },
    'installments.unignore': (args) => {
      const key = str(args, 'key');
      write('Série de parcelas de volta', () => {
        if (find('detected_installment_ignore', key)) del('detected_installment_ignore', key);
      });
      return detectInstallments();
    },
  }),
);
