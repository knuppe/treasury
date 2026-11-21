/**
 * O banco do mock (`store.ts`) no formato `Ledger` que os relatórios, o orçamento,
 * as metas e a carteira do mock leem (`ledger.ts`). Recalculado só quando o banco
 * muda (`db.version`).
 */
import { ensureSeeded } from './boot';
import { today } from './dates';
import type { Ledger, LedgerSplit, LedgerStatement, LedgerTxn, RecurrenceDef } from './ledger';
import { normalize, parseRrule } from './rrule';
import { categoryTree } from './domain';
import { setFixtureBase, syncLive, type FixtureAccount } from './fixture';
import { db, EMPTY_PROFILE, onStoreChange } from './store';

/** Contas, categorias e titulares do banco nas listas vivas de `fixture.ts`. */
function mirror(): void {
  const institution = (id: string | null) => (id ? (db.t.institution.get(id)?.name ?? null) : null);
  const accounts: FixtureAccount[] = [...db.t.account.values()]
    .sort((a, b) => a.sort_order - b.sort_order)
    .map((a) => {
      const card = db.t.credit_card.get(a.id);
      return {
        id: a.id,
        name: a.name,
        kind: a.kind,
        currency: a.currency,
        opening_balance: a.opening_balance,
        member_ids: [...db.t.account_member.values()]
          .filter((m) => m.account_id === a.id)
          .map((m) => m.member_id),
        institution: institution(a.institution_id),
        include_in_net_worth: a.include_in_net_worth,
        ...(card
          ? {
              card: {
                closing_day: card.closing_day,
                due_day: card.due_day,
                credit_limit: card.credit_limit ?? 0,
                payment: card.payment_account_id ?? '',
              },
            }
          : {}),
      };
    });
  const flat = (nodes: ReturnType<typeof categoryTree>): ReturnType<typeof categoryTree> =>
    nodes.flatMap((n) => [n, ...flat(n.children)]);
  const categories = flat(categoryTree(undefined, true)).map(({ children: _c, ...c }) => c);
  const members = [...db.t.member.values()]
    .sort((a, b) => a.sort_order - b.sort_order)
    .map(({ created_at: _a, updated_at: _b, ...m }) => ({
      ...m,
      profile: m.profile ?? EMPTY_PROFILE,
    }));
  syncLive({ accounts, categories, members });
}

const MIRRORED = new Set([
  'account',
  'credit_card',
  'account_member',
  'institution',
  'category',
  'member',
]);
onStoreChange((tables) => {
  if (tables === 'all' || [...tables].some((t) => MIRRORED.has(t))) mirror();
  // A moeda base dos relatórios do mock segue a da família (troca de país com a moeda).
  if (tables === 'all' || tables.has('household'))
    setFixtureBase([...db.t.household.values()][0]?.base_currency ?? 'BRL');
});

let cache: { version: number; today: string; ledger: Ledger } | null = null;

export function ledgerView(): Ledger {
  ensureSeeded();
  const t = today();
  if (cache && cache.version === db.version && cache.today === t) return cache.ledger;
  const splits = new Map<string, LedgerSplit[]>();
  for (const s of [...db.t.txn_split.values()].sort((a, b) => a.seq - b.seq)) {
    const list = splits.get(s.txn_id) ?? [];
    list.push({ category_id: s.category_id, member_id: s.member_id, amount: s.amount });
    splits.set(s.txn_id, list);
  }
  const txns: LedgerTxn[] = [...db.t.txn.values()]
    .map((r) => {
      const plan = r.installment_plan_id
        ? db.t.installment_plan.get(r.installment_plan_id)
        : undefined;
      const parts = splits.get(r.id);
      return {
        id: r.id,
        date: r.date,
        account_id: r.account_id,
        amount: r.amount,
        currency: r.currency,
        kind: r.kind,
        status: r.status,
        description: r.description,
        category_id: r.category_id,
        member_id: r.member_id,
        statement_id: r.statement_id,
        ...(parts ? { splits: parts } : {}),
        ...(r.transfer_id ? { transfer_id: r.transfer_id } : {}),
        ...(r.recurrence_id ? { recurrence_id: r.recurrence_id } : {}),
        ...(plan && r.installment_number
          ? { installment: { plan: plan.id, n: r.installment_number, of: plan.installments } }
          : {}),
      };
    })
    .sort((a, b) => (a.date === b.date ? a.id.localeCompare(b.id) : a.date.localeCompare(b.date)));
  const statements = new Map<string, LedgerStatement>(
    [...db.t.card_statement.values()].map((s) => [
      s.id,
      {
        id: s.id,
        account_id: s.account_id,
        reference_month: s.reference_month,
        period_start: s.period_start,
        closing_date: s.closing_date,
        due_date: s.due_date,
      },
    ]),
  );
  const recurrences: RecurrenceDef[] = [...db.t.recurrence.values()]
    .filter((r) => r.active)
    .map((r) => {
      const spec = normalize(parseRrule(r.rrule), r.start_date);
      return {
        id: r.id,
        description: r.description,
        account_id: r.account_id,
        kind: r.kind,
        category_id: r.category_id,
        member_id: r.member_id,
        ...(r.to_account_id ? { to_account_id: r.to_account_id } : {}),
        day: spec.by_month_day ?? Number(r.start_date.slice(8, 10)),
        amountAt: () => r.amount,
        spec,
      };
    });
  const start = db.meta.start ?? t;
  const ledger: Ledger = {
    today: t,
    start,
    firstMonth: db.meta.firstMonth ?? start.slice(0, 7),
    txns,
    statements,
    recurrences,
  };
  cache = { version: db.version, today: t, ledger };
  return ledger;
}
