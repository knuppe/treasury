/**
 * Mock de `transactions.transfer_candidates`, `transfer_match`, `link_transfers`,
 * `unlink_transfer` e `dismiss_transfer` (espelho de `transactions::transfers`): pares saída/entrada de mesmo
 * valor e moeda em contas diferentes, até 4 dias de distância, ainda soltos.
 */
import type { TransferCandidate } from '@/api/generated/TransferCandidate';
import type { TransferMatch } from '@/api/generated/TransferMatch';
import type { TxnKind } from '@/api/generated/TxnKind';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addDays, diffDays, nowUtc, today } from './ledger/dates';
import { categoryPaths, statementToPay } from './ledger/domain';
import { conflict, invalid, notFound, str } from './ledger/errors';
import { find, newId, patch, rows, write, type TxnRow } from './ledger/store';
import { fold } from './ledger/text';

const MAX_DAYS = 4;
const KEYWORDS = [
  'pix',
  'ted',
  'doc',
  'tef',
  'transf',
  'transferencia',
  'envio',
  'enviado',
  'recebido',
  'resgate',
  'aplicacao',
  'aplic',
  'deposito',
  'saque',
];

const dismissed = new Set<string>();
const auto = new Set<string>();
onMockReset(() => {
  dismissed.clear();
  auto.clear();
});

const accKey = (a: string, b: string) => (a <= b ? `${a}|${b}` : `${b}|${a}`);

function hasKeyword(text: string): boolean {
  return fold(text)
    .split(/[^a-z0-9]+/)
    .some((w) => KEYWORDS.some((k) => w === k || (k.length > 4 && w.startsWith(k))));
}

function kindFor(from: string, to: string): TxnKind {
  const kf = find('account', from)?.kind;
  const kt = find('account', to)?.kind;
  if (kt === 'credit_card') return 'card_payment';
  if (kf === 'brokerage' || kt === 'brokerage') return 'investment';
  return 'transfer';
}

function loose(since: string): TxnRow[] {
  const split = new Set(rows('txn_split').map((s) => s.txn_id));
  return rows('txn').filter(
    (t) =>
      t.date >= since &&
      !t.transfer_id &&
      (t.kind === 'income' || t.kind === 'expense' || t.kind === 'transfer') &&
      t.status !== 'planned' &&
      t.amount !== 0 &&
      !t.installment_plan_id &&
      !split.has(t.id) &&
      !find('account', t.account_id)?.archived,
  );
}

function leg(t: TxnRow, paths: Map<string, string>) {
  return {
    txn_id: t.id,
    account_id: t.account_id,
    account_name: find('account', t.account_id)?.name ?? '?',
    date: t.date,
    description: t.description,
    category_path: t.category_id ? (paths.get(t.category_id) ?? null) : null,
  };
}

export function candidates(args: Record<string, unknown>): TransferCandidate[] {
  const days = typeof args.days === 'number' ? args.days : 180;
  const only = Array.isArray(args.txn_ids) ? new Set(args.txn_ids as string[]) : null;
  const all = loose(addDays(today(), -days));
  const paths = categoryPaths();
  const outs = all.filter((t) => t.amount < 0);
  const ins = all.filter((t) => t.amount > 0);
  const found: TransferCandidate[] = [];
  for (const o of outs)
    for (const i of ins) {
      if (i.account_id === o.account_id || i.currency !== o.currency || i.amount !== -o.amount)
        continue;
      if (only && !only.has(o.id) && !only.has(i.id)) continue;
      const gap = Math.abs(diffDays(o.date, i.date));
      if (gap > MAX_DAYS || dismissed.has(`${o.id}|${i.id}`)) continue;
      const words = hasKeyword(o.description) || hasKeyword(i.description);
      const on = find('account', o.account_id)?.name ?? '';
      const inn = find('account', i.account_id)?.name ?? '';
      const named =
        fold(o.description).includes(fold(inn)) || fold(i.description).includes(fold(on));
      const score = Math.max(5, Math.min(100, 70 - gap * 12 + (words ? 20 : 0) + (named ? 15 : 0)));
      const why = [
        gap === 0 ? 'mesmo valor no mesmo dia' : `mesmo valor com ${gap} dia(s) de diferença`,
      ];
      if (words) why.push('texto de transferência (PIX, TED…)');
      if (named) why.push('o nome da outra conta aparece na descrição');
      found.push({
        from: leg(o, paths),
        to: leg(i, paths),
        amount: -o.amount,
        currency: o.currency,
        kind: kindFor(o.account_id, i.account_id),
        days_apart: gap,
        score,
        reason: why.join('; '),
        auto: auto.has(accKey(o.account_id, i.account_id)),
      });
    }
  found.sort(
    (a, b) =>
      b.score - a.score || a.days_apart - b.days_apart || b.from.date.localeCompare(a.from.date),
  );
  const used = new Set<string>();
  const out: TransferCandidate[] = [];
  for (const c of found) {
    if (used.has(c.from.txn_id) || used.has(c.to.txn_id)) continue;
    used.add(c.from.txn_id);
    used.add(c.to.txn_id);
    out.push(c);
  }
  return out;
}

export function linkPair(fromId: string, toId: string): void {
  const f = find('txn', fromId) ?? notFound(`lançamento ${fromId}`);
  const t = find('txn', toId) ?? notFound(`lançamento ${toId}`);
  if (f.transfer_id || t.transfer_id)
    conflict('um dos lançamentos já faz parte de uma transferência');
  const cash = (k: TxnKind) => k === 'income' || k === 'expense' || k === 'transfer';
  if (!cash(f.kind) || !cash(t.kind))
    invalid('só receitas, despesas e transferências soltas viram transferência');
  if (f.account_id === t.account_id) invalid('as duas pernas precisam estar em contas diferentes');
  if (f.amount >= 0 || t.amount <= 0)
    invalid('a perna de origem precisa ser uma saída e a de destino uma entrada');
  if (f.currency === t.currency && -f.amount !== t.amount)
    invalid('contas na mesma moeda: o valor que sai precisa ser igual ao que entra');
  const kind = kindFor(f.account_id, t.account_id);
  const tid = newId();
  for (const id of [fromId, toId]) {
    const keep = kind === 'transfer' ? (find('txn', id)?.category_id ?? null) : null;
    patch('txn', id, { kind, transfer_id: tid, category_id: keep, updated_at: nowUtc() });
  }
}

/** Valor da outra perna: o oposto na mesma moeda; em outra moeda, o informado (ou `null`). */
function otherLegAmount(row: TxnRow, currency: string, informed: number | null): number | null {
  if (currency === row.currency) return -row.amount;
  return informed ? Math.abs(informed) * -Math.sign(row.amount) : null;
}

/** Lançamento solto da outra conta que forma o par (o de data mais próxima, até 4 dias). */
function counterpart(accountId: string, amount: number, row: TxnRow): TxnRow | undefined {
  const split = new Set(rows('txn_split').map((s) => s.txn_id));
  return rows('txn')
    .filter(
      (t) =>
        t.id !== row.id &&
        t.account_id === accountId &&
        t.amount === amount &&
        !t.transfer_id &&
        (t.kind === 'income' || t.kind === 'expense' || t.kind === 'transfer') &&
        !t.installment_plan_id &&
        !split.has(t.id) &&
        Math.abs(diffDays(t.date, row.date)) <= MAX_DAYS,
    )
    .sort(
      (a, b) =>
        Math.abs(diffDays(a.date, row.date)) - Math.abs(diffDays(b.date, row.date)) ||
        a.id.localeCompare(b.id),
    )[0];
}

/** Liga um lançamento solto à outra conta da família (perna existente ou nova), como o núcleo. */
export function linkToAccount(
  txnId: string,
  accountId: string,
  otherAmount: number | null,
  createLeg: (row: TxnRow, amount: number) => string,
): void {
  const row = find('txn', txnId) ?? notFound(`lançamento ${txnId}`);
  if (row.transfer_id) invalid('este lançamento já está ligado a outra conta');
  if (row.account_id === accountId) invalid('escolha uma conta diferente da do lançamento');
  const acc = find('account', accountId) ?? notFound(`conta ${accountId}`);
  const amount = otherLegAmount(row, acc.currency, otherAmount);
  if (amount == null)
    invalid(`a outra conta usa ${acc.currency}: informe quanto entrou (ou saiu) nela`);
  const other = counterpart(accountId, amount, row)?.id ?? createLeg(row, amount);
  if (row.amount < 0) linkPair(txnId, other);
  else linkPair(other, txnId);
}

/** Prévia de `linkToAccount`: a perna que já existe (ou nenhuma), o tipo e a fatura. */
function matchAccount(txnId: string, accountId: string, otherAmount: number | null): TransferMatch {
  const row = find('txn', txnId) ?? notFound(`lançamento ${txnId}`);
  if (row.transfer_id) invalid('este lançamento já está ligado a outra conta');
  if (row.account_id === accountId) invalid('escolha uma conta diferente da do lançamento');
  const acc = find('account', accountId) ?? notFound(`conta ${accountId}`);
  const amount = otherLegAmount(row, acc.currency, otherAmount);
  const found = amount == null ? undefined : counterpart(accountId, amount, row);
  const kind =
    row.amount < 0 ? kindFor(row.account_id, accountId) : kindFor(accountId, row.account_id);
  let due: string | null = null;
  if (kind === 'card_payment') {
    const [card, date, stmt] =
      row.amount < 0
        ? [accountId, found?.date ?? row.date, found?.statement_id ?? null]
        : [row.account_id, row.date, row.statement_id];
    const id = stmt ?? statementToPay(card, date);
    due = id ? (find('card_statement', id)?.due_date ?? null) : null;
  }
  return {
    existing: found ? leg(found, categoryPaths()) : null,
    amount,
    currency: acc.currency,
    kind,
    statement_due_date: due,
  };
}

registerMockHandlers(
  guarded({
    'transactions.transfer_candidates': (args) => candidates(args),
    'transactions.transfer_match': (args) =>
      matchAccount(
        str(args, 'txn_id'),
        str(args, 'account_id'),
        typeof args.other_amount === 'number' ? args.other_amount : null,
      ),
    'transactions.link_transfers': (args) => {
      const pairs = (args.pairs ?? []) as { from_id: string; to_id: string }[];
      if (!pairs.length) invalid('nenhum par selecionado');
      const reason =
        pairs.length === 1
          ? 'Lançamentos ligados como transferência entre contas'
          : `${pairs.length} pares ligados como transferências entre contas`;
      const rec = write(reason, () => {
        for (const p of pairs) linkPair(p.from_id, p.to_id);
        return pairs.length;
      });
      if (args.remember === true)
        for (const p of pairs) {
          const a = find('txn', p.from_id)?.account_id;
          const b = find('txn', p.to_id)?.account_id;
          if (a && b) auto.add(accKey(a, b));
        }
      return { linked: rec.value, change_set_id: rec.changeSetId };
    },
    'transactions.unlink_transfer': (args) => {
      const row = find('txn', str(args, 'txn_id')) ?? notFound('lançamento');
      if (!row.transfer_id) invalid('este lançamento não está ligado a outra conta');
      const tid = row.transfer_id;
      const legs = rows('txn')
        .filter((t) => t.transfer_id === tid)
        .sort((a, b) => a.amount - b.amount);
      const rec = write('Transferência desligada: as duas pernas ficaram soltas', () => {
        for (const l of legs)
          patch('txn', l.id, { kind: 'transfer', transfer_id: null, updated_at: nowUtc() });
        return legs.map((l) => l.id);
      });
      if (args.dismiss !== false && legs.length === 2)
        dismissed.add(`${legs[0]!.id}|${legs[1]!.id}`);
      return { txn_ids: rec.value, change_set_id: rec.changeSetId };
    },
    'transactions.dismiss_transfer': (args) => {
      dismissed.add(`${str(args, 'from_id')}|${str(args, 'to_id')}`);
      return null;
    },
  }),
);
