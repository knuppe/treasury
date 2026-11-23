/** Mock de `recurrences.*`: regras RRULE, prévia em português e lançamentos previstos. */
import type { Recurrence } from '@/api/generated/Recurrence';
import type { RuleSpec } from '@/api/generated/RuleSpec';
import type { TxnKind } from '@/api/generated/TxnKind';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addDays, brDate, nowUtc, parseYmd, today } from './ledger/dates';
import {
  account,
  checkCategory,
  currencyAndKind,
  deleteRows,
  insertTxn,
  isCashFlow,
  requireMember,
  statementToPay,
} from './ledger/domain';
import { cleanOpt, has, invalid, notFound, str, text } from './ledger/errors';
import {
  human,
  monthlyEquivalent,
  nextN,
  normalize,
  occurrencesBetween,
  parseRrule,
  toRrule,
} from './ledger/rrule';
import { type RecurrenceRow, del, find, newId, patch, put, rows, write } from './ledger/store';
import { short } from './transactions';

const HORIZON = 62;

function load(id: string): RecurrenceRow {
  return find('recurrence', id) ?? notFound(`recorrência ${id}`);
}

function toView(r: RecurrenceRow, T = today()): Recurrence {
  const spec = normalize(parseRrule(r.rrule), r.start_date);
  let text = human(spec);
  if (r.end_date) text += `, até ${brDate(r.end_date)}`;
  return {
    id: r.id,
    description: r.description,
    account_id: r.account_id,
    account_name: account(r.account_id).name,
    amount: r.amount,
    currency: r.currency,
    kind: r.kind,
    category_id: r.category_id,
    member_id: r.member_id,
    to_account_id: r.to_account_id,
    rrule: r.rrule,
    // Como o núcleo: campos ausentes da regra vêm como `null` (o tipo TS diz opcional).
    spec: {
      by_month_day: null,
      by_day: null,
      by_month: null,
      count: null,
      ...spec,
    } as unknown as RuleSpec,
    human: text,
    start_date: r.start_date,
    end_date: r.end_date,
    active: r.active,
    next_dates: r.active ? nextN(spec, r.start_date, r.end_date, T, 3) : [],
    monthly_equivalent: monthlyEquivalent(spec, r.amount),
    remind: !quietRecurrences.has(r.id),
  };
}

function checkFields(
  accountId: string,
  kind: TxnKind,
  amount: number,
  to: string | null,
  categoryId: string | null,
): number {
  if (amount === 0) invalid('o valor da recorrência não pode ser zero');
  const { currency } = currencyAndKind(accountId);
  if (kind === 'income' || kind === 'expense') {
    if (to) invalid('conta de destino só vale para transferências recorrentes');
    checkCategory(categoryId, kind);
    return kind === 'income' ? Math.abs(amount) : -Math.abs(amount);
  }
  if (kind === 'adjustment') invalid('ajustes não podem ser recorrentes');
  if (!to) invalid('informe a conta de destino da transferência recorrente');
  if (to === accountId) invalid('a conta de origem e a de destino devem ser diferentes');
  const target = currencyAndKind(to);
  if (target.currency !== currency) {
    invalid(
      `transferências recorrentes entre moedas diferentes (${currency} → ${target.currency}) não são suportadas`,
    );
  }
  if (kind === 'card_payment' && target.kind !== 'credit_card') {
    invalid('pagamento de fatura recorrente precisa de um cartão como destino');
  }
  if (categoryId) invalid('transferências não usam categoria');
  return -Math.abs(amount);
}

function plannedRows(id: string, from: string | null) {
  return rows('txn').filter(
    (t) => t.recurrence_id === id && t.status === 'planned' && (!from || t.date >= from),
  );
}

/** Gera os previstos até hoje + horizonte (idempotente, como `materialize_one`). */
function materializeOne(r: RecurrenceRow, T: string, horizon: number): number {
  if (!r.active) return 0;
  let until = addDays(T, Math.min(Math.max(horizon, 0), 731));
  if (r.end_date && r.end_date < until) until = r.end_date;
  const from = [r.generated_until ? addDays(r.generated_until, 1) : T, r.start_date].sort().at(-1)!;
  if (from > until) return 0;
  const spec = parseRrule(r.rrule);
  const dates = occurrencesBetween(spec, r.start_date, r.end_date, from, until);
  const existing = new Set(
    rows('txn')
      .filter((t) => t.recurrence_id === r.id)
      .map((t) => t.date),
  );
  let created = 0;
  for (const date of dates.filter((d) => !existing.has(d))) {
    const base = {
      date,
      currency: r.currency,
      kind: r.kind,
      description: r.description,
      status: 'planned' as const,
      member_id: r.member_id,
      recurrence_id: r.id,
    };
    if (r.to_account_id) {
      const transfer = newId();
      insertTxn({ ...base, account_id: r.account_id, amount: r.amount, transfer_id: transfer });
      insertTxn({
        ...base,
        account_id: r.to_account_id,
        amount: -r.amount,
        transfer_id: transfer,
        statement_id: r.kind === 'card_payment' ? statementToPay(r.to_account_id, date) : undefined,
      });
      created += 2;
    } else {
      insertTxn({
        ...base,
        account_id: r.account_id,
        amount: r.amount,
        category_id: isCashFlow(r.kind) ? r.category_id : null,
      });
      created += 1;
    }
  }
  patch('recurrence', r.id, { generated_until: until, updated_at: nowUtc() });
  return created;
}

function create(args: Record<string, unknown>): string {
  const description = text(args.description).trim();
  if (!description) invalid('informe a descrição da recorrência');
  const start = str(args, 'start_date');
  parseYmd(start);
  const end = cleanOpt(args.end_date);
  if (end) {
    parseYmd(end);
    if (end < start) invalid('a data final é anterior à data inicial');
  }
  const accountId = str(args, 'account_id');
  const kind = args.kind as TxnKind;
  const categoryId = cleanOpt(args.category_id);
  const to = cleanOpt(args.to_account_id);
  const amount = checkFields(accountId, kind, Number(args.amount), to, categoryId);
  const memberId = cleanOpt(args.member_id);
  if (memberId) requireMember(memberId);
  const spec = normalize(args.spec as RuleSpec, start);
  const now = nowUtc();
  const row = put('recurrence', {
    id: newId(),
    description,
    account_id: accountId,
    amount,
    currency: currencyAndKind(accountId).currency,
    kind,
    category_id: categoryId,
    member_id: memberId,
    to_account_id: to,
    rrule: toRrule(spec),
    start_date: start,
    end_date: end,
    generated_until: null,
    active: true,
    created_at: now,
    updated_at: now,
  });
  materializeOne(row, today(), HORIZON);
  return row.id;
}

function update(args: Record<string, unknown>): void {
  const T = today();
  const cur = load(str(args, 'id'));
  const description = has(args, 'description') ? String(args.description).trim() : cur.description;
  if (!description) invalid('informe a descrição da recorrência');
  const categoryId = has(args, 'category_id') ? cleanOpt(args.category_id) : cur.category_id;
  const memberId = has(args, 'member_id') ? cleanOpt(args.member_id) : cur.member_id;
  if (memberId) requireMember(memberId);
  const amount = checkFields(
    cur.account_id,
    cur.kind,
    has(args, 'amount') ? Number(args.amount) : cur.amount,
    cur.to_account_id,
    categoryId !== cur.category_id ? categoryId : null,
  );
  const rrule = args.spec ? toRrule(normalize(args.spec as RuleSpec, cur.start_date)) : cur.rrule;
  const end = has(args, 'end_date') ? cleanOpt(args.end_date) : cur.end_date;
  if (end && end < cur.start_date) invalid('a data final é anterior à data inicial');
  const active = typeof args.active === 'boolean' ? args.active : cur.active;
  const datesChanged = rrule !== cur.rrule || end !== cur.end_date;
  const valuesChanged =
    amount !== cur.amount ||
    description !== cur.description ||
    categoryId !== cur.category_id ||
    memberId !== cur.member_id;
  const yesterday = addDays(T, -1);
  let generated = cur.generated_until;
  const updatePlanned = args.update_planned !== false;
  if (updatePlanned && (datesChanged || (cur.active && !active))) {
    deleteRows(plannedRows(cur.id, T));
    if (generated && generated > yesterday) generated = yesterday;
  } else if (updatePlanned && valuesChanged) {
    for (const t of plannedRows(cur.id, T)) {
      const legAmount = t.account_id === cur.account_id ? amount : -amount;
      patch('txn', t.id, {
        amount: legAmount,
        description,
        category_id: isCashFlow(t.kind) ? categoryId : null,
        member_id: memberId,
        updated_at: nowUtc(),
      });
    }
  }
  if (!cur.active && active && (!generated || generated < yesterday)) generated = yesterday;
  const next = patch('recurrence', cur.id, {
    description,
    amount,
    category_id: categoryId,
    member_id: memberId,
    rrule,
    end_date: end,
    active,
    generated_until: generated,
    updated_at: nowUtc(),
  });
  if (next.active) materializeOne(next, T, HORIZON);
}

/** Recorrências sem aviso de vencimento (como `notify::quiet` no núcleo). */
export const quietRecurrences = new Set<string>();
onMockReset(() => quietRecurrences.clear());

registerMockHandlers(
  guarded({
    'notify.set_recurrence_reminder': (args) => {
      const id = str(args, 'recurrence_id');
      load(id);
      if (args.remind === false) quietRecurrences.add(id);
      else quietRecurrences.delete(id);
      return { recurrence_id: id, remind: args.remind !== false, resolved: 0 };
    },
    'recurrences.list': (args) =>
      rows('recurrence')
        .filter(
          (r) =>
            !args.account_id ||
            r.account_id === args.account_id ||
            r.to_account_id === args.account_id,
        )
        .filter((r) => typeof args.active !== 'boolean' || r.active === args.active)
        .sort(
          (a, b) =>
            Number(b.active) - Number(a.active) ||
            a.description.localeCompare(b.description, 'pt-BR', { sensitivity: 'base' }) ||
            a.id.localeCompare(b.id),
        )
        .map((r) => toView(r)),
    'recurrences.get': (args) => toView(load(str(args, 'id'))),
    'recurrences.preview': (args) => {
      const start = str(args, 'start_date');
      parseYmd(start);
      const end = cleanOpt(args.end_date);
      if (end) parseYmd(end);
      const spec = normalize(args.spec as RuleSpec, start);
      const n = Math.min(Math.max(Number(args.count ?? 12), 1), 366);
      return { rrule: toRrule(spec), human: human(spec), dates: nextN(spec, start, end, start, n) };
    },
    'recurrences.create': (args) => {
      const id = write(`Recorrência criada: ${short(text(args.description))}`, () =>
        create(args),
      ).value;
      return toView(load(id));
    },
    'recurrences.update': (args) => {
      const cur = load(str(args, 'id'));
      write(`Recorrência alterada: ${short(cur.description)}`, () => update(args));
      return toView(load(cur.id));
    },
    'recurrences.delete': (args) => {
      const cur = load(str(args, 'id'));
      const rec = write(`Recorrência apagada: ${short(cur.description)}`, () => {
        const removed = args.delete_planned === false ? 0 : deleteRows(plannedRows(cur.id, null));
        for (const t of rows('txn'))
          if (t.recurrence_id === cur.id)
            patch('txn', t.id, { recurrence_id: null, updated_at: nowUtc() });
        del('recurrence', cur.id);
        return removed;
      });
      return { deleted_planned: rec.value, change_set_id: rec.changeSetId };
    },
    'recurrences.materialize': (args) => {
      const horizon = typeof args.horizon_days === 'number' ? args.horizon_days : HORIZON;
      if (horizon < 0 || horizon > 731) invalid('o horizonte deve ficar entre 0 e 731 dias');
      const rec = write('Lançamentos previstos de recorrências', () =>
        rows('recurrence').reduce((n, r) => n + materializeOne(r, today(), horizon), 0),
      );
      return { created: rec.value, change_set_id: rec.changeSetId };
    },
  }),
);
