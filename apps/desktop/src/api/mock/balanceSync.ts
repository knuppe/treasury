/**
 * Mock de `accounts.sync_balance`, `list_syncs`, `sync_hints` e `resolve_sync` (espelho de
 * `accounts::sync`): marco com o saldo do banco, diferença lançada como ajuste e causas
 * prováveis (duplicados, mesmo valor, previsto, sinal trocado).
 */
import type { BalanceSync } from '@/api/generated/BalanceSync';
import type { BalanceSyncHint } from '@/api/generated/BalanceSyncHint';
import type { BalanceSyncHints } from '@/api/generated/BalanceSyncHints';
import type { BalanceSyncStatus } from '@/api/generated/BalanceSyncStatus';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addDays, brDate, nowUtc, parseYmd, today } from './ledger/dates';
import { balanceAt, insertTxn } from './ledger/domain';
import { invalid, notFound, num, str } from './ledger/errors';
import { find, newId, patch, rows, write } from './ledger/store';
import { fold } from './ledger/text';

interface SyncRec extends Omit<BalanceSync, 'account_name' | 'currency'> {
  updated_at: string;
}

let syncs: SyncRec[] = [];
onMockReset(() => {
  syncs = [];
});

const brl = (c: number) =>
  (c / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });

function view(s: SyncRec): BalanceSync {
  const a = find('account', s.account_id);
  const { updated_at: _u, ...rest } = s;
  return { ...rest, account_name: a?.name ?? '?', currency: a?.currency ?? 'BRL' };
}

function get(id: string): SyncRec {
  return syncs.find((s) => s.id === id) ?? notFound(`sincronia de saldo ${id}`);
}

function hints(id: string): BalanceSyncHints {
  const s = get(id);
  const diff = s.difference;
  const prev = syncs
    .filter((x) => x.account_id === s.account_id && x.date < s.date)
    .map((x) => x.date)
    .sort()
    .at(-1);
  const start = prev ?? addDays(s.date, -90);
  const period = rows('txn').filter(
    (t) =>
      t.account_id === s.account_id &&
      t.date > start &&
      t.date <= s.date &&
      t.id !== s.adjustment_txn_id,
  );
  const out: BalanceSyncHint[] = [];
  if (diff !== 0) {
    const same = period.filter(
      (t) => t.status !== 'planned' && Math.abs(t.amount) === Math.abs(diff),
    );
    if (same.length)
      out.push({
        kind: 'same_amount',
        message: `${same.length} lançamento(s) com o valor exato da diferença (${brl(Math.abs(diff))}): pode ser um lançado a mais (ou de menos) no Treasury.`,
        txn_ids: same.map((t) => t.id),
      });
    const planned = period.filter((t) => t.status === 'planned' && t.amount === diff);
    if (planned.length)
      out.push({
        kind: 'planned_not_cleared',
        message:
          'Há previsto com o valor da diferença ainda não efetivado: talvez já tenha acontecido no banco.',
        txn_ids: planned.map((t) => t.id),
      });
    if (diff % 2 === 0) {
      const half = period.filter((t) => t.status !== 'planned' && t.amount === -diff / 2);
      if (half.length)
        out.push({
          kind: 'sign_flipped',
          message: `Lançamento de ${brl(Math.abs(diff / 2))} com o sinal trocado explicaria a diferença.`,
          txn_ids: half.map((t) => t.id),
        });
    }
    const dups = new Set<string>();
    period.forEach((a, i) =>
      period.slice(i + 1).forEach((b) => {
        if (
          a.date === b.date &&
          a.amount === b.amount &&
          a.status !== 'planned' &&
          b.status !== 'planned' &&
          fold(a.description) === fold(b.description)
        ) {
          dups.add(a.id);
          dups.add(b.id);
        }
      }),
    );
    if (dups.size)
      out.push({
        kind: 'duplicate',
        message: `${dups.size} lançamento(s) parecem duplicados (mesma data, valor e descrição).`,
        txn_ids: [...dups],
      });
    const open = period.filter((t) => t.status === 'cleared');
    if (!out.length && open.length)
      out.push({
        kind: 'unreconciled',
        message: `Nenhuma causa óbvia: confira os ${period.length} lançamento(s) do período com o extrato (tarifas, juros e estornos costumam faltar).`,
        txn_ids: open.map((t) => t.id),
      });
  }
  return { sync: view(s), since: prev ?? null, txns_in_period: period.length, hints: out };
}

registerMockHandlers(
  guarded({
    'accounts.list_syncs': (args) =>
      syncs
        .filter(
          (s) =>
            (!args.account_id || s.account_id === args.account_id) &&
            (args.open_only !== true || s.status === 'open'),
        )
        .sort((a, b) => b.date.localeCompare(a.date) || b.created_at.localeCompare(a.created_at))
        .map(view),
    'accounts.sync_hints': (args) => hints(str(args, 'id')),
    'accounts.sync_balance': (args) => {
      const accountId = str(args, 'account_id');
      const acc = find('account', accountId) ?? notFound(`conta ${accountId}`);
      const date = str(args, 'date');
      parseYmd(date);
      if (date > today()) invalid('a data do saldo não pode estar no futuro');
      const balance = num(args, 'balance');
      const note = typeof args.note === 'string' && args.note.trim() ? args.note.trim() : null;
      const rec = write(`Sincronia de saldo: ${acc.name} em ${brDate(date)}`, () => {
        const computed = balanceAt(accountId, date, false);
        const diff = balance - computed;
        const adj =
          diff !== 0
            ? insertTxn({
                account_id: accountId,
                date,
                amount: diff,
                currency: acc.currency,
                kind: 'adjustment',
                status: 'reconciled',
                description: 'Sincronia de saldo',
                notes: `Saldo no banco em ${brDate(date)}: ${brl(balance)}; o Treasury calculava ${brl(computed)}.${note ? ` ${note}` : ''}`,
              }).id
            : null;
        if (args.reconcile !== false)
          for (const t of rows('txn'))
            if (t.account_id === accountId && t.status === 'cleared' && t.date <= date)
              patch('txn', t.id, { status: 'reconciled', updated_at: nowUtc() });
        const now = nowUtc();
        const s: SyncRec = {
          id: newId(),
          account_id: accountId,
          date,
          statement_balance: balance,
          computed_balance: computed,
          difference: diff,
          adjustment_txn_id: adj,
          status: diff === 0 ? 'explained' : 'open',
          note,
          explanation: null,
          created_at: now,
          updated_at: now,
        };
        syncs.push(s);
        return s;
      });
      return view(rec.value);
    },
    'accounts.resolve_sync': (args) => {
      const s = get(str(args, 'id'));
      const status = str(args, 'status') as BalanceSyncStatus;
      const explanation =
        typeof args.explanation === 'string' && args.explanation.trim()
          ? args.explanation.trim()
          : null;
      if (status === 'explained' && !explanation) invalid('escreva o motivo da diferença');
      s.status = status;
      if (explanation) s.explanation = explanation;
      s.updated_at = nowUtc();
      return view(s);
    },
  }),
);
