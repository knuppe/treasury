/**
 * Mock of `accounts.close_preview`, `accounts.close` and `accounts.reopen` (accounts/closing.rs
 * in the core): the same blockers, except positions at the broker (the mock has no portfolio by
 * account) and loans debited from the account.
 */
import type { AccountCloseBlocker } from '@/api/generated/AccountCloseBlocker';
import type { AccountClosePreview } from '@/api/generated/AccountClosePreview';
import type { AccountCloseResult } from '@/api/generated/AccountCloseResult';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { brDate, nowUtc, parseYmd, today } from './ledger/dates';
import { account, accountView, balanceAt } from './ledger/domain';
import { conflict, str } from './ledger/errors';
import { del, find, patch, rows, write } from './ledger/store';

function money(cents: number, currency: string): string {
  const v = (Math.abs(cents) / 100).toLocaleString('pt-BR', { minimumFractionDigits: 2 });
  const sign = cents < 0 ? '−' : '';
  return currency === 'BRL' ? `${sign}R$ ${v}` : `${sign}${currency} ${v}`;
}

/** Planned transactions the closing removes (both legs of planned transfers). */
function plannedIds(id: string): string[] {
  const legs = new Set<string>();
  for (const t of rows('txn'))
    if (t.account_id === id && t.status === 'planned' && !t.installment_plan_id && t.transfer_id)
      legs.add(t.transfer_id);
  return rows('txn')
    .filter(
      (t) =>
        t.status === 'planned' &&
        !t.installment_plan_id &&
        (t.account_id === id || (t.transfer_id != null && legs.has(t.transfer_id))),
    )
    .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id))
    .map((t) => t.id);
}

export function closePreview(args: Record<string, unknown>): AccountClosePreview {
  const id = str(args, 'id');
  const a = account(id);
  const date = typeof args.date === 'string' && args.date ? args.date : today();
  parseYmd(date);
  const blockers: AccountCloseBlocker[] = [];
  if (a.closed_on) blockers.push({ kind: 'already_closed', closed_on: a.closed_on });
  if (date > today()) blockers.push({ kind: 'future_date' });
  if (date < a.opening_date)
    blockers.push({ kind: 'before_opening', opening_date: a.opening_date });
  const balance = balanceAt(id, date, false);
  if (balance !== 0) blockers.push({ kind: 'non_zero_balance', balance, currency: a.currency });
  const after = rows('txn').filter(
    (t) => t.account_id === id && t.status !== 'planned' && t.date > date,
  );
  if (after.length)
    blockers.push({
      kind: 'activity_after',
      count: after.length,
      last_date: after
        .map((t) => t.date)
        .sort()
        .at(-1)!,
    });
  const installments = rows('txn').filter(
    (t) => t.account_id === id && t.status === 'planned' && t.installment_plan_id,
  ).length;
  if (installments) blockers.push({ kind: 'pending_installments', count: installments });
  return {
    account_id: id,
    date,
    balance,
    currency: a.currency,
    blockers,
    planned: plannedIds(id).map((tid) => {
      const t = find('txn', tid)!;
      return {
        id: t.id,
        date: t.date,
        description: t.description,
        amount: t.amount,
        currency: t.currency,
      };
    }),
    recurrences: rows('recurrence')
      .filter((r) => r.active && (r.account_id === id || r.to_account_id === id))
      .map((r) => ({ id: r.id, name: r.description })),
    loans: [],
    cards_paid: rows('credit_card')
      .filter((c) => c.payment_account_id === id && !find('account', c.id)?.archived)
      .map((c) => ({ id: c.id, name: find('account', c.id)?.name ?? c.id })),
  };
}

function blockerText(b: AccountCloseBlocker): string {
  switch (b.kind) {
    case 'future_date':
      return 'A data de encerramento não pode ser depois de hoje.';
    case 'before_opening':
      return `A data de encerramento é antes da abertura da conta (${brDate(b.opening_date)}).`;
    case 'already_closed':
      return `A conta já foi encerrada em ${brDate(b.closed_on)}.`;
    case 'non_zero_balance':
      return `A conta ainda tem saldo de ${money(b.balance, b.currency)} na data. Transfira o saldo ou lance o ajuste antes de encerrar.`;
    case 'activity_after':
      return `Há ${b.count} lançamento(s) depois da data (o último em ${brDate(b.last_date)}).`;
    case 'open_positions':
      return `A conta ainda tem posição em ${b.assets.join(', ')}.`;
    case 'pending_installments':
      return `O cartão ainda tem ${b.count} parcela(s) a vencer.`;
  }
}

/** Closes inside the caller's write (used by `accounts.update` with `closed_on`). */
export function closeAccount(args: Record<string, unknown>): AccountCloseResult {
  const p = closePreview(args);
  const first = p.blockers[0];
  if (first) conflict(blockerText(first));
  for (const r of p.recurrences) {
    const rec = find('recurrence', r.id)!;
    const end = rec.start_date <= p.date && (!rec.end_date || rec.end_date > p.date);
    patch('recurrence', r.id, {
      active: false,
      ...(end ? { end_date: p.date } : {}),
      updated_at: nowUtc(),
    });
  }
  for (const t of p.planned) del('txn', t.id);
  patch('account', p.account_id, { closed_on: p.date, archived: true, updated_at: nowUtc() });
  return { planned_removed: p.planned.length, recurrences_paused: p.recurrences.length };
}

export function reopenAccount(id: string, archived = false): void {
  account(id);
  patch('account', id, { closed_on: null, archived, updated_at: nowUtc() });
}

registerMockHandlers(
  guarded({
    'accounts.close_preview': closePreview,
    'accounts.close': (args) =>
      write(`Conta encerrada: ${account(str(args, 'id')).name}`, () => closeAccount(args)).value,
    'accounts.reopen': (args) => {
      const id = str(args, 'id');
      write(`Conta reaberta: ${account(id).name}`, () => reopenAccount(id));
      return accountView(account(id));
    },
  }),
);
