/**
 * Mock das cotas capital (`accounts.capital_share*`, espelho de `accounts::capital`): saldos
 * informados, aportes pelo texto ou pela categoria e rentabilidade = variação − aportes.
 */
import type { CapitalBalancePoint } from '@/api/generated/CapitalBalancePoint';
import type { CapitalContribution } from '@/api/generated/CapitalContribution';
import type { CapitalShareView } from '@/api/generated/CapitalShareView';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addDays, nowUtc, parseYmd } from './ledger/dates';
import { invalid, notFound, num, str } from './ledger/errors';
import { find, newId, patch, rows, write } from './ledger/store';
import { operationLinks } from './ledger/opLinks';
import { fold } from './ledger/text';

const DEFAULT_PATTERN = 'cota capital|cotas capital|integraliz|capital social';

interface Rec {
  account_id: string;
  started_on: string | null;
  contribution_pattern: string | null;
  contribution_category_id: string | null;
  include_in_net_worth: boolean;
  notes: string | null;
  balances: { id: string; date: string; balance: number; note: string | null }[];
}

const recs = new Map<string, Rec>();
onMockReset(() => recs.clear());

function contributions(r: Rec): CapitalContribution[] {
  const terms = (r.contribution_pattern ?? DEFAULT_PATTERN).split('|').map(fold).filter(Boolean);
  const links = operationLinks();
  return rows('txn')
    .filter((t) => {
      if (t.status === 'planned' || t.amount === 0) return false;
      if (r.started_on && t.date < r.started_on) return false;
      const link = links.get(t.id);
      // Vínculo explícito vence o texto e a categoria.
      if (link) return link.kind === 'capital_share' && link.target_id === r.account_id;
      return (
        t.account_id === r.account_id &&
        Boolean(
          (r.contribution_category_id && t.category_id === r.contribution_category_id) ||
          terms.some((x) => fold(t.description).includes(x)),
        )
      );
    })
    .sort((a, b) => a.date.localeCompare(b.date))
    .map((t) => ({ txn_id: t.id, date: t.date, amount: -t.amount, description: t.description }));
}

/** Contas com cotas capital e o texto dos aportes (para o vínculo de lançamentos). */
export function capitalShareAccounts(): { account_id: string; name: string; pattern: string }[] {
  return [...recs.values()].map((r) => ({
    account_id: r.account_id,
    name: find('account', r.account_id)?.name ?? '?',
    pattern: r.contribution_pattern ?? DEFAULT_PATTERN,
  }));
}

const pct = (n: number, d: number) => (d > 0 ? Math.round((n / d) * 10_000) / 100 : null);

function view(accountId: string): CapitalShareView | null {
  const r = recs.get(accountId);
  if (!r) return null;
  const acc = find('account', accountId)!;
  const cs = contributions(r);
  const sum = (from: string | null, to: string) =>
    cs.filter((c) => (!from || c.date > from) && c.date <= to).reduce((s, c) => s + c.amount, 0);
  const sorted = [...r.balances].sort((a, b) => a.date.localeCompare(b.date));
  const points: CapitalBalancePoint[] = [];
  let prev: { date: string; balance: number } | null = null;
  let total = 0;
  for (const b of sorted) {
    const c = sum(prev?.date ?? null, b.date);
    const y = prev ? b.balance - prev.balance - c : null;
    if (y != null) total += y;
    points.push({
      id: b.id,
      date: b.date,
      balance: b.balance,
      note: b.note,
      contributions: c,
      yield_amount: y,
      yield_pct: prev && y != null ? pct(y, prev.balance + c / 2) : null,
    });
    prev = b;
  }
  const last = points.at(-1);
  let y12: number | null = null;
  if (last) {
    const target = addDays(last.date, -365 + 31);
    const base = points.filter((p) => p.date <= target).at(-1);
    if (base && base.date !== last.date) {
      const c = sum(base.date, last.date);
      y12 = pct(last.balance - base.balance - c, base.balance + c / 2);
    }
  }
  return {
    account_id: accountId,
    currency: acc.currency,
    started_on: r.started_on,
    contribution_pattern: r.contribution_pattern ?? DEFAULT_PATTERN,
    contribution_category_id: r.contribution_category_id,
    include_in_net_worth: r.include_in_net_worth,
    notes: r.notes,
    balances: points,
    current_balance: last?.balance ?? null,
    current_date: last?.date ?? null,
    total_contributions: cs.reduce((s, c) => s + c.amount, 0),
    total_yield: total,
    yield_12m_pct: y12,
    contributions_since_last: cs
      .filter((c) => !last || c.date > last.date)
      .reduce((s, c) => s + c.amount, 0),
    contributions: cs,
  };
}

registerMockHandlers(
  guarded({
    'accounts.capital_share': (a) => view(str(a, 'account_id')),
    'accounts.capital_share_set': (a) => {
      const id = str(a, 'account_id');
      const acc = find('account', id) ?? notFound(`conta ${id}`);
      if (!['checking', 'savings', 'other'].includes(acc.kind))
        invalid('cotas capital ficam numa conta corrente (ou poupança) da cooperativa');
      write(`Cotas capital: ${acc.name}`, () => {
        const cur = recs.get(id);
        recs.set(id, {
          account_id: id,
          started_on: typeof a.started_on === 'string' ? a.started_on : null,
          contribution_pattern:
            typeof a.contribution_pattern === 'string' && a.contribution_pattern.trim()
              ? a.contribution_pattern.trim()
              : null,
          contribution_category_id:
            typeof a.contribution_category_id === 'string' ? a.contribution_category_id : null,
          include_in_net_worth: a.include_in_net_worth !== false,
          notes: typeof a.notes === 'string' ? a.notes : null,
          balances: cur?.balances ?? [],
        });
      });
      return view(id);
    },
    'accounts.capital_share_remove': (a) => {
      recs.delete(str(a, 'account_id'));
      return null;
    },
    'accounts.capital_share_add_balance': (a) => {
      const id = str(a, 'account_id');
      const r = recs.get(id) ?? invalid('ative as cotas capital nesta conta antes');
      const date = str(a, 'date');
      parseYmd(date);
      const balance = num(a, 'balance');
      if (balance < 0) invalid('o saldo das cotas não pode ser negativo');
      const note = typeof a.note === 'string' && a.note.trim() ? a.note.trim() : null;
      const same = r.balances.find((b) => b.date === date);
      if (same) Object.assign(same, { balance, note });
      else r.balances.push({ id: newId(), date, balance, note });
      return view(id);
    },
    'accounts.capital_share_delete_balance': (a) => {
      const id = str(a, 'id');
      for (const r of recs.values()) r.balances = r.balances.filter((b) => b.id !== id);
      return null;
    },
    'accounts.capital_share_mark_contributions': (a) => {
      const id = str(a, 'account_id');
      const v = view(id) ?? invalid('ative as cotas capital nesta conta antes');
      let n = 0;
      write('Aportes em cotas capital fora das despesas', () => {
        for (const c of v.contributions.filter((c) => c.amount > 0)) {
          const t = find('txn', c.txn_id);
          if (t && t.kind === 'expense' && !t.transfer_id) {
            patch('txn', t.id, { kind: 'investment', category_id: null, updated_at: nowUtc() });
            n++;
          }
        }
      });
      return { updated: n };
    },
  }),
);
