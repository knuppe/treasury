/** Mock de `accounts.*`: contas com saldos calculados, histórico e conciliação. */
import type { AccountKind } from '@/api/generated/AccountKind';
import type { OverdraftTerms } from '@/api/generated/OverdraftTerms';
import { bankDetailsOf, cleanBankRef, setBankDetails } from './ledger/bankDetails';
import { setStalenessLimits, stalenessSettings } from './ledger/freshness';
import { matchOfx } from './ledger/ofxMatch';
import { setOverdraft } from './ledger/overdraft';
import { recordManualLimit } from './ledger/cardOverview';
import { dropPlasticsOf, syncHolderLast4 } from './ledger/plastics';
import type { BalancePoint } from '@/api/generated/BalancePoint';
import type { CardDetails } from '@/api/generated/CardDetails';
import type { ReconcileResult } from '@/api/generated/ReconcileResult';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addDays, brDate, diffDays, nowUtc, parseYmd, today } from './ledger/dates';
import {
  account,
  accountView,
  accountViews,
  balanceAt,
  baseCurrency,
  datesForReferenceMonth,
  reinheritMembers,
  requireMember,
  statementDates,
} from './ledger/domain';
import {
  cleanName,
  cleanOpt,
  conflict,
  has,
  invalid,
  notFound,
  num,
  optBool,
  str,
  text,
} from './ledger/errors';
import { familyCountry } from './country';
import { retirementOf, setRetirementPlan } from './retirement';
import { type AccountRow, del, find, newId, patch, put, rows, write } from './ledger/store';

const KINDS: AccountKind[] = [
  'checking',
  'savings',
  'cash',
  'credit_card',
  'brokerage',
  'foreign',
  'benefit',
  'fgts',
  'retirement',
  'other',
];

function validateCurrency(c: unknown): string {
  const v = text(c).trim().toUpperCase();
  if (!/^[A-Z]{3}$/.test(v))
    invalid(`moeda inválida: ${text(c)} (use o código ISO, ex.: BRL, USD)`);
  return v;
}

function validateCard(card: CardDetails, selfId: string | null): CardDetails {
  if (!(card.closing_day >= 1 && card.closing_day <= 31))
    invalid('o dia de fechamento deve ficar entre 1 e 31');
  if (!(card.due_day >= 1 && card.due_day <= 31))
    invalid('o dia de vencimento deve ficar entre 1 e 31');
  if (card.credit_limit != null && card.credit_limit < 0)
    invalid('o limite do cartão não pode ser negativo');
  const payment = cleanOpt(card.payment_account_id);
  if (payment) {
    if (payment === selfId) invalid('o cartão não pode pagar a própria fatura');
    const p = find('account', payment);
    if (!p) notFound(`conta pagadora ${payment}`);
    if (p.kind === 'credit_card') invalid('a conta que paga a fatura não pode ser outro cartão');
  }
  const last4 = cleanOpt(card.last4);
  if (last4 && !/^\d{4}$/.test(last4))
    invalid('informe os 4 últimos dígitos do cartão (só números)');
  return {
    credit_limit: card.credit_limit ?? null,
    closing_day: card.closing_day,
    due_day: card.due_day,
    payment_account_id: payment,
    network: cleanOpt(card.network),
    last4,
  };
}

function validateMembers(ids: unknown): string[] {
  const list = Array.isArray(ids) ? [...new Set(ids.map(String))] : [];
  list.forEach(requireMember);
  return list;
}

function validateInstitution(id: unknown): string | null {
  const v = cleanOpt(id);
  if (v && !find('institution', v)) notFound(`instituição ${v}`);
  return v;
}

function replaceMembers(accountId: string, ids: string[]): void {
  for (const link of rows('account_member')) {
    if (link.account_id === accountId && !ids.includes(link.member_id))
      del('account_member', link.id);
  }
  for (const m of ids) {
    const id = `${accountId}|${m}`;
    if (!find('account_member', id))
      put('account_member', { id, account_id: accountId, member_id: m });
  }
}

function upsertCard(accountId: string, c: CardDetails): void {
  const now = nowUtc();
  const cur = find('credit_card', accountId);
  recordManualLimit(accountId, cur?.credit_limit, c.credit_limit);
  if (cur) patch('credit_card', accountId, { ...c, updated_at: now });
  else put('credit_card', { id: accountId, ...c, created_at: now, updated_at: now });
  syncHolderLast4(accountId, c.last4);
}

/** Fechamento/vencimento mudou: refaz as datas das faturas ainda não fechadas. */
function recomputeStatements(accountId: string, card: CardDetails): void {
  const T = today();
  for (const s of rows('card_statement')) {
    if (s.account_id !== accountId || s.closing_date < T || (s.dates_source ?? 'calc') !== 'calc')
      continue;
    const d = datesForReferenceMonth(s.reference_month, card.closing_day, card.due_day);
    patch('card_statement', s.id, { ...d, updated_at: nowUtc() });
  }
  for (const t of rows('txn')) {
    if (t.account_id !== accountId || t.kind === 'card_payment') continue;
    const st = find('card_statement', t.statement_id);
    if (st && st.closing_date < T) continue;
    const d = statementDates(t.date, card.closing_day, card.due_day);
    const target = rows('card_statement').find(
      (s) => s.account_id === accountId && s.reference_month === d.reference_month,
    );
    if (target && target.id !== t.statement_id)
      patch('txn', t.id, { statement_id: target.id, updated_at: nowUtc() });
  }
}

function createAccount(args: Record<string, unknown>): string {
  const name = cleanName(args.name, 'da conta');
  const kind = args.kind as AccountKind;
  if (!KINDS.includes(kind))
    invalid(`argumentos inválidos: unknown variant \`${String(args.kind)}\``);
  // Tipos do Brasil (FGTS) só num cofre do Brasil, sempre em reais.
  if (kind === 'fgts' && familyCountry() !== 'BR')
    invalid(
      'Esse tipo de conta só existe no Brasil: o cofre precisa estar configurado para o Brasil.',
    );
  const currency =
    kind === 'fgts'
      ? 'BRL'
      : has(args, 'currency')
        ? validateCurrency(args.currency)
        : baseCurrency();
  const institution = validateInstitution(args.institution_id);
  const members = validateMembers(args.member_ids);
  const openingDate = has(args, 'opening_date') ? str(args, 'opening_date') : today();
  parseYmd(openingDate);
  let card: CardDetails | null = null;
  if (kind === 'credit_card') {
    if (!args.card) invalid('informe os dados do cartão (dia de fechamento e de vencimento)');
    card = validateCard(args.card as CardDetails, null);
  } else if (args.card) {
    invalid('dados de cartão só valem para contas do tipo cartão de crédito');
  }
  const now = nowUtc();
  const id = newId();
  put('account', {
    id,
    name,
    kind,
    currency,
    institution_id: institution,
    opening_balance: typeof args.opening_balance === 'number' ? args.opening_balance : 0,
    opening_date: openingDate,
    include_in_net_worth: args.include_in_net_worth !== false,
    archived: false,
    color: cleanOpt(args.color),
    notes: cleanOpt(args.notes),
    sort_order: Math.max(-1, ...rows('account').map((a) => a.sort_order)) + 1,
    created_at: now,
    updated_at: now,
  });
  replaceMembers(id, members);
  if (card) upsertCard(id, card);
  setBankDetails(
    id,
    cleanBankRef(args.branch, 'agência'),
    cleanBankRef(args.account_number, 'número da conta'),
  );
  if (kind === 'retirement') setRetirementPlan(id, args.retirement_plan, args.retirement_coverage);
  return id;
}

function updateAccount(args: Record<string, unknown>): void {
  const id = str(args, 'id');
  const cur = account(id);
  const changes: Partial<AccountRow> = {};
  if (has(args, 'name')) changes.name = cleanName(args.name, 'da conta');
  if (has(args, 'institution_id'))
    changes.institution_id = validateInstitution(args.institution_id);
  if (has(args, 'opening_balance')) changes.opening_balance = num(args, 'opening_balance');
  if (has(args, 'opening_date')) {
    parseYmd(str(args, 'opening_date'));
    changes.opening_date = str(args, 'opening_date');
  }
  if (has(args, 'include_in_net_worth'))
    changes.include_in_net_worth = Boolean(optBool(args, 'include_in_net_worth'));
  if (has(args, 'archived')) changes.archived = Boolean(optBool(args, 'archived'));
  if (has(args, 'color')) changes.color = cleanOpt(args.color);
  if (has(args, 'notes')) changes.notes = cleanOpt(args.notes);
  if (has(args, 'sort_order')) changes.sort_order = num(args, 'sort_order');
  const next = { ...cur, ...changes };
  if (JSON.stringify(next) !== JSON.stringify(cur))
    patch('account', id, { ...changes, updated_at: nowUtc() });
  if (has(args, 'member_ids')) {
    replaceMembers(id, validateMembers(args.member_ids));
    reinheritMembers(id);
  }
  if (has(args, 'branch') || has(args, 'account_number')) {
    const cur = bankDetailsOf(id);
    setBankDetails(
      id,
      has(args, 'branch') ? cleanBankRef(args.branch, 'agência') : cur.branch,
      has(args, 'account_number')
        ? cleanBankRef(args.account_number, 'número da conta')
        : cur.account_number,
    );
  }
  if (has(args, 'retirement_plan') || has(args, 'retirement_coverage')) {
    if (cur.kind !== 'retirement') invalid('só a conta de aposentadoria tem plano');
    const r = retirementOf(id);
    setRetirementPlan(
      id,
      args.retirement_plan ?? r.retirement_plan,
      args.retirement_coverage ?? r.retirement_coverage,
    );
  }
  if (args.card) {
    if (cur.kind !== 'credit_card')
      invalid('dados de cartão só valem para contas do tipo cartão de crédito');
    const c = validateCard(args.card as CardDetails, id);
    const old = find('credit_card', id);
    const cycleChanged = !old || old.closing_day !== c.closing_day || old.due_day !== c.due_day;
    upsertCard(id, c);
    if (cycleChanged) recomputeStatements(id, c);
  }
}

function deleteAccount(id: string): void {
  const a = account(id);
  const txns = rows('txn').filter((t) => t.account_id === id).length;
  if (txns > 0)
    conflict(`a conta ${a.name} tem ${txns} lançamento(s); apague-os antes ou arquive a conta`);
  const recs = rows('recurrence').filter(
    (r) => r.account_id === id || r.to_account_id === id,
  ).length;
  if (recs > 0) conflict(`a conta ${a.name} é usada por ${recs} recorrência(s); apague-as antes`);
  const plans = rows('installment_plan').filter((p) => p.account_id === id).length;
  if (plans > 0) conflict(`a conta ${a.name} tem ${plans} parcelamento(s); apague-os antes`);
  for (const c of rows('credit_card')) {
    if (c.payment_account_id === id)
      patch('credit_card', c.id, { payment_account_id: null, updated_at: nowUtc() });
  }
  for (const b of rows('import_batch')) if (b.account_id === id) del('import_batch', b.id);
  for (const s of rows('card_statement')) if (s.account_id === id) del('card_statement', s.id);
  del('credit_card', id);
  dropPlasticsOf(id);
  for (const l of rows('account_member')) if (l.account_id === id) del('account_member', l.id);
  del('account', id);
}

function balanceHistory(args: Record<string, unknown>): BalancePoint[] {
  const id = str(args, 'id');
  const from = str(args, 'from');
  const to = str(args, 'to');
  parseYmd(from);
  parseYmd(to);
  account(id);
  if (from > to) invalid('a data inicial é posterior à data final');
  const interval = args.interval === 'month' ? 'month' : 'day';
  if (interval === 'day' && diffDays(to, from) + 1 > 3700) {
    invalid('período longo demais para pontos diários (máximo 3700 dias); use interval = month');
  }
  const planned = Boolean(args.include_planned);
  let running = balanceAt(id, addDays(from, -1), planned);
  const deltas = new Map<string, number>();
  for (const t of rows('txn')) {
    if (t.account_id !== id || t.date < from || t.date > to) continue;
    if (t.status === 'planned' && !planned) continue;
    deltas.set(t.date, (deltas.get(t.date) ?? 0) + t.amount);
  }
  const out: BalancePoint[] = [];
  for (let d = from; d <= to; d = addDays(d, 1)) {
    running += deltas.get(d) ?? 0;
    const point = interval === 'day' || d === to || addDays(d, 1).endsWith('-01');
    if (point) out.push({ date: d, balance: running });
  }
  return out;
}

function reconcile(args: Record<string, unknown>): ReconcileResult {
  const accountId = str(args, 'account_id');
  const date = str(args, 'statement_date');
  parseYmd(date);
  const statementBalance = num(args, 'statement_balance');
  const computed = balanceAt(accountId, date, false);
  const result: ReconcileResult = {
    computed_balance: computed,
    statement_balance: statementBalance,
    difference: statementBalance - computed,
    marked_reconciled: 0,
    change_set_id: null,
  };
  if (args.apply === true && result.difference === 0) {
    const rec = write(`Conciliação da conta ${account(accountId).name} até ${brDate(date)}`, () => {
      let n = 0;
      for (const t of rows('txn')) {
        if (t.account_id === accountId && t.status === 'cleared' && t.date <= date) {
          patch('txn', t.id, { status: 'reconciled', updated_at: nowUtc() });
          n++;
        }
      }
      return n;
    });
    result.marked_reconciled = rec.value;
    result.change_set_id = rec.changeSetId;
  }
  return result;
}

registerMockHandlers(
  guarded({
    'accounts.list': (args) => accountViews(Boolean(args.include_archived)),
    'accounts.get': (args) => accountView(account(str(args, 'id'))),
    'accounts.create': (args) => {
      const name = typeof args.name === 'string' ? args.name.trim() : '';
      const id = write(`Conta criada: ${name}`, () => createAccount(args)).value;
      if (args.overdraft) setOverdraft(id, args.overdraft as OverdraftTerms);
      return accountView(account(id));
    },
    'accounts.update': (args) => {
      const id = str(args, 'id');
      write(`Conta alterada: ${account(id).name}`, () => updateAccount(args));
      if ('overdraft' in args) setOverdraft(id, (args.overdraft ?? null) as OverdraftTerms | null);
      return accountView(account(id));
    },
    'accounts.delete': (args) => {
      const id = str(args, 'id');
      write(`Conta apagada: ${account(id).name}`, () => deleteAccount(id));
      return null;
    },
    'accounts.balance_history': balanceHistory,
    'accounts.reconcile': reconcile,
    'accounts.match_ofx': matchOfx,
    'accounts.staleness_limits': () => stalenessSettings(),
    'accounts.set_staleness_limits': (args) => setStalenessLimits(args),
  }),
);
