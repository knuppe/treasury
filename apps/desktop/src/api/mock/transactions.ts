/**
 * Mock de `transactions.*`: consulta com filtros e totais, criação, edição com
 * pernas de transferência coerentes, divisão, lote, tags e anexos — com as
 * mesmas validações e mensagens do núcleo (`transactions/write.rs`).
 */
import type { BulkResult } from '@/api/generated/BulkResult';
import type { TagSummary } from '@/api/generated/TagSummary';
import type { TransferInput } from '@/api/generated/TransferInput';
import type { TransferResult } from '@/api/generated/TransferResult';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnStatus } from '@/api/generated/TxnStatus';

import { registerMockHandlers } from '../mockRegistry';
import { linkCategoryPayments } from './debt';
import { linkProject } from './ledger/projectsState';
import { linkToAccount } from './transfers';
import { guarded } from './ledger/boot';
import { brDate, nowUtc, parseYmd } from './ledger/dates';
import {
  type SplitIn,
  account,
  categoryPaths,
  checkCategory,
  checkSplits,
  currencyAndKind,
  deleteRows,
  insertTxn,
  isCashFlow,
  kindLabel,
  otherLegs,
  ensureTag,
  queryTxnIds,
  queryTxns,
  checkOrig,
  reinheritOne,
  requirePlasticOn,
  sameDescription,
  replaceSplits,
  replaceTags,
  requireMember,
  statementFor,
  statementToPay,
  statusLabel,
  txnRow,
  txnView,
} from './ledger/domain';
import {
  cleanName,
  cleanOpt,
  conflict,
  has,
  invalid,
  notFound,
  num,
  str,
  text,
} from './ledger/errors';
import { type TxnRow, del, find, newId, patch, put, rows, write } from './ledger/store';
import { fingerprint, hashHex } from './ledger/text';

const KINDS: TxnKind[] = [
  'income',
  'expense',
  'transfer',
  'card_payment',
  'investment',
  'adjustment',
];
const STATUSES: TxnStatus[] = ['planned', 'cleared', 'reconciled'];

export function short(desc: string): string {
  const d = desc.trim();
  if ([...d].length > 60) return `${[...d].slice(0, 59).join('')}…`;
  return d || '(sem descrição)';
}

function kindArg(v: unknown): TxnKind | undefined {
  if (v === undefined || v === null) return undefined;
  if (!KINDS.includes(v as TxnKind))
    invalid(`argumentos inválidos: unknown variant \`${text(v)}\``);
  return v as TxnKind;
}

function statusArg(v: unknown): TxnStatus | undefined {
  if (v === undefined || v === null) return undefined;
  if (!STATUSES.includes(v as TxnStatus))
    invalid(`argumentos inválidos: unknown variant \`${text(v)}\``);
  return v as TxnStatus;
}

function splitsArg(v: unknown): SplitIn[] {
  if (!Array.isArray(v)) return [];
  return v.map((s: Record<string, unknown>) => ({
    category_id: cleanOpt(s.category_id),
    member_id: cleanOpt(s.member_id),
    amount:
      typeof s.amount === 'number'
        ? s.amount
        : invalid('argumentos inválidos: missing field `amount`'),
    notes: cleanOpt(s.notes),
  }));
}

const requireNonzero = (amount: number) => {
  if (amount === 0) invalid('o valor do lançamento não pode ser zero');
};

function create(args: Record<string, unknown>): string {
  const accountId = str(args, 'account_id');
  const { currency } = currencyAndKind(accountId);
  const date = str(args, 'date');
  parseYmd(date);
  const amount = num(args, 'amount');
  requireNonzero(amount);
  const kind = kindArg(args.kind) ?? (amount < 0 ? 'expense' : 'income');
  if (kind === 'card_payment') {
    invalid('pagamentos de fatura têm duas pernas; use transactions.create_transfer');
  }
  const categoryId = cleanOpt(args.category_id);
  checkCategory(categoryId, kind);
  const memberId = cleanOpt(args.member_id);
  if (memberId) requireMember(memberId);
  const splits = splitsArg(args.splits);
  checkSplits(amount, kind, splits);
  const plasticId = cleanOpt(args.card_plastic_id);
  if (plasticId) requirePlasticOn(plasticId, accountId);
  const row = insertTxn({
    account_id: accountId,
    date,
    amount,
    currency,
    kind,
    description: typeof args.description === 'string' ? args.description : '',
    status: statusArg(args.status) ?? 'cleared',
    payee: cleanOpt(args.payee),
    notes: cleanOpt(args.notes),
    category_id: categoryId,
    member_id: memberId,
    card_plastic_id: plasticId,
    ...checkOrig(amount, currency, args.orig_amount, args.orig_currency),
  });
  if (splits.length) replaceSplits(row.id, splits);
  if (Array.isArray(args.tags)) replaceTags(row.id, args.tags.map(String));
  const project = cleanOpt(args.project_id);
  if (project) linkProject([row.id], project);
  return row.id;
}

const RECONCILED =
  'este lançamento está conciliado; desconcilie (mude o status para efetivado) antes de alterar valor, data ou conta';

function legStatus(next: TxnStatus, other: TxnStatus): TxnStatus {
  if (next === 'planned' && other === 'reconciled') {
    conflict(
      'a outra perna da transferência está conciliada; desconcilie-a antes de voltar para previsto',
    );
  }
  if (next === 'planned') return 'planned';
  if (other === 'planned') return 'cleared';
  return other;
}

function statementForRow(r: TxnRow): string | null {
  return r.kind === 'card_payment' && r.amount > 0
    ? statementToPay(r.account_id, r.date)
    : statementFor(r.account_id, r.date);
}

function existingSplits(txnId: string): SplitIn[] {
  return rows('txn_split')
    .filter((s) => s.txn_id === txnId)
    .sort((a, b) => a.seq - b.seq)
    .map((s) => ({
      category_id: s.category_id,
      member_id: s.member_id,
      amount: s.amount,
      notes: s.notes,
    }));
}

function writeRow(cur: TxnRow, next: TxnRow): void {
  const changed = JSON.stringify(cur) !== JSON.stringify(next);
  if (!changed) return;
  const fp = fingerprint(next.date, next.amount, next.description);
  put('txn', { ...next, fingerprint: fp, updated_at: nowUtc() });
}

function update(args: Record<string, unknown>): void {
  const id = str(args, 'id');
  const cur = txnRow(id);
  const next: TxnRow = { ...cur };
  if (has(args, 'account_id')) next.account_id = str(args, 'account_id');
  if (has(args, 'date')) {
    parseYmd(str(args, 'date'));
    next.date = str(args, 'date');
  }
  if (has(args, 'amount')) {
    requireNonzero(num(args, 'amount'));
    next.amount = num(args, 'amount');
  }
  const k = kindArg(args.kind);
  if (k) next.kind = k;
  const s = statusArg(args.status);
  if (s) next.status = s;
  if (has(args, 'description')) next.description = String(args.description).trim();
  if (has(args, 'payee')) next.payee = cleanOpt(args.payee);
  if (has(args, 'notes')) next.notes = cleanOpt(args.notes);
  if (has(args, 'category_id')) next.category_id = cleanOpt(args.category_id);
  if (has(args, 'member_id')) {
    next.member_id = cleanOpt(args.member_id);
    if (next.member_id) requireMember(next.member_id);
  }
  const coreChanged =
    next.account_id !== cur.account_id || next.date !== cur.date || next.amount !== cur.amount;
  if (cur.status === 'reconciled' && next.status === 'reconciled' && coreChanged)
    conflict(RECONCILED);

  if (next.account_id !== cur.account_id) {
    const target = currencyAndKind(next.account_id);
    if (target.currency !== cur.currency) {
      invalid(
        `a conta escolhida usa ${target.currency} e o lançamento está em ${cur.currency}; crie um lançamento novo nessa conta`,
      );
    }
    if (cur.kind === 'card_payment' && cur.amount > 0 && target.kind !== 'credit_card') {
      invalid('o destino de um pagamento de fatura precisa ser uma conta de cartão');
    }
  }
  const legs = otherLegs(cur);
  const splitsGiven = Array.isArray(args.splits) ? splitsArg(args.splits) : null;
  if (cur.transfer_id) {
    if (next.kind !== cur.kind) {
      invalid(
        'não é possível mudar o tipo de uma perna de transferência; apague a transferência e crie outro lançamento',
      );
    }
    if (Math.sign(next.amount) !== Math.sign(cur.amount)) {
      invalid(
        'numa transferência a perna de origem é negativa e a de destino é positiva; mantenha o sinal',
      );
    }
    if (legs.some((l) => l.account_id === next.account_id)) {
      invalid('as duas pernas da transferência não podem ficar na mesma conta');
    }
    if (splitsGiven && splitsGiven.length > 0) invalid('transferências não podem ser divididas');
  } else if (next.kind === 'card_payment' && next.kind !== cur.kind) {
    invalid(
      'para transformar em pagamento de fatura, ligue-o à perna do cartão (transferências sugeridas)',
    );
  }
  if (has(args, 'transfer_account_id') && args.transfer_account_id && cur.transfer_id)
    invalid('este lançamento já está ligado a outra conta');
  if (next.category_id !== cur.category_id || next.kind !== cur.kind)
    checkCategory(next.category_id, next.kind);
  if (splitsGiven) {
    checkSplits(next.amount, next.kind, splitsGiven);
    replaceSplits(cur.id, splitsGiven);
  } else if (next.amount !== cur.amount || next.kind !== cur.kind) {
    const existing = existingSplits(cur.id);
    if (existing.length) {
      try {
        checkSplits(next.amount, next.kind, existing);
      } catch (e) {
        const msg = (e as { message?: string }).message ?? '';
        invalid(
          `${msg}; envie a divisão atualizada junto (ou uma lista vazia para remover a divisão)`,
        );
      }
    }
  }
  if (
    next.account_id !== cur.account_id ||
    (next.date !== cur.date && cur.kind !== 'card_payment')
  ) {
    next.statement_id = statementForRow(next);
  }
  // Valor de origem (compra no exterior), conferido contra o valor novo.
  if (has(args, 'orig_amount') || has(args, 'orig_currency') || next.amount !== cur.amount) {
    const clear = has(args, 'orig_amount') && args.orig_amount == null;
    const o = checkOrig(
      next.amount,
      cur.currency,
      clear ? null : has(args, 'orig_amount') ? args.orig_amount : cur.orig_amount,
      clear ? null : has(args, 'orig_currency') ? args.orig_currency : cur.orig_currency,
    );
    next.orig_amount = o.orig_amount;
    next.orig_currency = o.orig_currency;
  }
  // Plástico do cartão: o enviado (do cartão novo) ou, ao mudar de conta, nenhum.
  const plasticGiven = has(args, 'card_plastic_id');
  if (plasticGiven || next.account_id !== cur.account_id) {
    next.card_plastic_id = plasticGiven ? cleanOpt(args.card_plastic_id) : null;
    if (next.card_plastic_id) requirePlasticOn(next.card_plastic_id, next.account_id);
  }
  writeRow(cur, next);
  if (
    !has(args, 'member_id') &&
    (next.account_id !== cur.account_id || next.card_plastic_id !== (cur.card_plastic_id ?? null))
  ) {
    reinheritOne(txnRow(cur.id));
  }
  if (Array.isArray(args.tags)) replaceTags(cur.id, args.tags.map(String));
  if (has(args, 'project_id')) linkProject([cur.id], cleanOpt(args.project_id));

  for (const leg of legs) {
    const l: TxnRow = { ...leg };
    if (next.date !== cur.date) l.date = next.date;
    if (has(args, 'description')) l.description = next.description;
    if (has(args, 'member_id')) l.member_id = next.member_id;
    if (next.amount !== cur.amount && l.currency === cur.currency) l.amount = -next.amount;
    if (next.status !== cur.status) l.status = legStatus(next.status, l.status);
    const legCore = l.date !== leg.date || l.amount !== leg.amount;
    if (leg.status === 'reconciled' && l.status === 'reconciled' && legCore) {
      conflict(
        'a outra perna da transferência está conciliada; desconcilie-a antes de alterar valor ou data',
      );
    }
    if (l.date !== leg.date && l.kind !== 'card_payment') l.statement_id = statementForRow(l);
    writeRow(leg, l);
  }
  if (typeof args.transfer_account_id === 'string' && args.transfer_account_id) {
    linkToAccount(
      cur.id,
      args.transfer_account_id,
      typeof args.transfer_amount === 'number' ? args.transfer_amount : null,
      (row, amount) =>
        create({
          account_id: args.transfer_account_id,
          date: row.date,
          amount,
          kind: 'transfer',
          status: row.status,
          description: row.description,
          ...(row.member_id ? { member_id: row.member_id } : {}),
        }),
    );
  }
}

/** Espelho de `transactions::duplicates`: o mesmo `external_id` em duas contas. */
function crossAccountPairs(): { keep: TxnRow; copy: TxnRow; leg: boolean }[] {
  const by = new Map<string, TxnRow[]>();
  for (const r of rows('txn')) {
    if (!r.external_id) continue;
    const list = by.get(r.external_id) ?? [];
    list.push(r);
    by.set(r.external_id, list);
  }
  const out: { keep: TxnRow; copy: TxnRow; leg: boolean }[] = [];
  for (const [ext, list] of by) {
    const one: TxnRow[] = [];
    for (const r of list) if (!one.some((x) => x.account_id === r.account_id)) one.push(r);
    for (let i = 0; i < one.length; i += 1)
      for (const b of one.slice(i + 1)) {
        const a = one[i]!;
        if (a.transfer_id && a.transfer_id === b.transfer_id) continue;
        const ia = find('account', a.account_id)?.institution_id ?? null;
        const ib = find('account', b.account_id)?.institution_id ?? null;
        if (ext.length < 32 && (ia && ib ? ia !== ib : ext.length < 16)) continue;
        const card = (r: TxnRow) => find('account', r.account_id)?.kind === 'credit_card';
        const [keep, copy] =
          card(a) && !card(b)
            ? [a, b]
            : card(b) && !card(a)
              ? [b, a]
              : a.id <= b.id
                ? [a, b]
                : [b, a];
        out.push({ keep, copy, leg: copy.transfer_id != null });
      }
  }
  return out;
}

function deleteTxns(ids: string[], refuseReconciled = true): number {
  const all = new Map<string, TxnRow>();
  for (const id of new Set(ids)) {
    const row = txnRow(id);
    for (const r of [row, ...otherLegs(row)]) all.set(r.id, r);
  }
  const reconciled = refuseReconciled
    ? [...all.values()].find((r) => r.status === 'reconciled')
    : undefined;
  if (reconciled) {
    conflict(
      `o lançamento "${reconciled.description}" de ${brDate(reconciled.date)} está conciliado; desconcilie antes de apagar`,
    );
  }
  return deleteRows([...all.values()]);
}

export function createTransfer(input: TransferInput, forcedStatement: string | null = null) {
  if (input.from_account_id === input.to_account_id) {
    invalid('a conta de origem e a de destino devem ser diferentes');
  }
  const from = currencyAndKind(input.from_account_id);
  const to = currencyAndKind(input.to_account_id);
  parseYmd(input.date);
  if (!(input.amount > 0)) invalid('informe o valor da transferência como um número positivo');
  let toAmount = input.amount;
  if (from.currency === to.currency) {
    if (input.to_amount != null && input.to_amount !== input.amount) {
      invalid('contas na mesma moeda: o valor que entra deve ser igual ao que sai');
    }
  } else {
    if (!(input.to_amount != null && input.to_amount > 0)) {
      invalid(
        `a origem está em ${from.currency} e o destino em ${to.currency}; informe o valor que entra no destino (to_amount)`,
      );
    }
    toAmount = input.to_amount;
  }
  const kind = input.kind ?? 'transfer';
  if (kind === 'card_payment') {
    if (to.kind !== 'credit_card')
      invalid('pagamento de fatura só pode ter uma conta de cartão como destino');
    if (from.kind === 'credit_card')
      invalid('a fatura deve ser paga a partir de uma conta que não seja cartão');
  } else if (kind !== 'transfer' && kind !== 'investment') {
    invalid(`transferências não podem ser do tipo ${kindLabel(kind)}`);
  }
  const memberId = cleanOpt(input.member_id);
  if (memberId) requireMember(memberId);
  const fromName = account(input.from_account_id).name;
  const toName = account(input.to_account_id).name;
  const custom = cleanOpt(input.description);
  const [fromDesc, toDesc] = custom
    ? [custom, custom]
    : kind === 'card_payment'
      ? [`Pagamento da fatura ${toName}`, `Pagamento recebido de ${fromName}`]
      : kind === 'investment'
        ? [`Aporte em ${toName}`, `Aporte vindo de ${fromName}`]
        : [`Transferência para ${toName}`, `Transferência de ${fromName}`];
  const status = input.status ?? 'cleared';
  const transferId = newId();
  const a = insertTxn({
    account_id: input.from_account_id,
    date: input.date,
    amount: -input.amount,
    currency: from.currency,
    kind,
    description: fromDesc,
    status,
    member_id: memberId,
    transfer_id: transferId,
  });
  const b = insertTxn({
    account_id: input.to_account_id,
    date: input.date,
    amount: toAmount,
    currency: to.currency,
    kind,
    description: toDesc,
    status,
    member_id: memberId,
    transfer_id: transferId,
    statement_id:
      kind === 'card_payment'
        ? (forcedStatement ?? statementToPay(input.to_account_id, input.date))
        : undefined,
  });
  return { transferId, fromId: a.id, toId: b.id };
}

/** Como `transactions::set_category`: aporte, pagamento de fatura e ajuste ficam de fora
 * (`skipped`); só eles, erro. */
function setCategory(ids: string[], categoryId: string | null): { n: number; skipped: number } {
  let list = [...new Set(ids)].map(txnRow);
  let skipped = 0;
  if (categoryId) {
    const before = list.length;
    list = list.filter((r) => isCashFlow(r.kind) || r.kind === 'transfer');
    skipped = before - list.length;
    if (!list.length) {
      invalid(
        `aporte, pagamento de fatura e ajuste não usam categoria: são movimentação do patrimônio e ficam fora do fluxo de caixa e do orçamento (${skipped} marcado(s))`,
      );
    }
    for (const k of new Set(list.map((r) => r.kind))) checkCategory(categoryId, k);
  }
  let n = 0;
  for (const r of list) {
    const hadSplits = rows('txn_split').some((s) => s.txn_id === r.id);
    if (hadSplits) replaceSplits(r.id, []);
    if (r.category_id !== categoryId) {
      patch('txn', r.id, { category_id: categoryId, updated_at: nowUtc() });
      n++;
    } else if (hadSplits) n++;
  }
  return { n, skipped };
}

function setStatus(ids: string[], status: TxnStatus): number {
  let n = 0;
  for (const id of new Set(ids)) {
    const cur = txnRow(id);
    if (cur.status === status) continue;
    patch('txn', id, { status, updated_at: nowUtc() });
    n++;
    for (const leg of otherLegs(cur)) {
      patch('txn', leg.id, { status: legStatus(status, leg.status), updated_at: nowUtc() });
    }
  }
  return n;
}

function setMember(ids: string[], memberId: string | null): number {
  if (memberId) requireMember(memberId);
  let n = 0;
  for (const id of new Set(ids)) {
    const cur = txnRow(id);
    if (cur.member_id === memberId) continue;
    patch('txn', id, { member_id: memberId, member_origin: 'manual', updated_at: nowUtc() });
    n++;
    for (const leg of otherLegs(cur))
      patch('txn', leg.id, { member_id: memberId, member_origin: 'manual', updated_at: nowUtc() });
  }
  return n;
}

function editTags(ids: string[], add: string[], remove: string[]): number {
  const clean = (v: string[]) => v.map((s) => s.trim()).filter(Boolean);
  const [plus, minus] = [clean(add), clean(remove)];
  if (plus.length === 0 && minus.length === 0) invalid('informe as tags a pôr ou a tirar');
  const plusIds = plus.map(ensureTag);
  const minusIds = new Set(
    rows('tag')
      .filter((g) => minus.some((m) => m.toLowerCase() === g.name.toLowerCase()))
      .map((g) => g.id),
  );
  let n = 0;
  for (const id of new Set(ids)) {
    txnRow(id);
    let changed = false;
    for (const tagId of plusIds) {
      const linkId = `${id}|${tagId}`;
      if (!find('txn_tag', linkId)) {
        put('txn_tag', { id: linkId, txn_id: id, tag_id: tagId });
        changed = true;
      }
    }
    for (const link of rows('txn_tag')) {
      if (link.txn_id === id && minusIds.has(link.tag_id)) {
        del('txn_tag', link.id);
        changed = true;
      }
    }
    if (changed) {
      patch('txn', id, { updated_at: nowUtc() });
      n++;
    }
  }
  return n;
}

function idsArg(args: Record<string, unknown>): string[] {
  if (!Array.isArray(args.ids)) invalid('argumentos inválidos: missing field `ids`');
  return args.ids.map(String);
}

function tagSummaries(): TagSummary[] {
  const counts = new Map<string, number>();
  for (const l of rows('txn_tag')) counts.set(l.tag_id, (counts.get(l.tag_id) ?? 0) + 1);
  return rows('tag')
    .sort((a, b) => a.name.localeCompare(b.name, 'pt-BR', { sensitivity: 'base' }))
    .map((t) => ({ id: t.id, name: t.name, color: t.color, txn_count: counts.get(t.id) ?? 0 }));
}

function attachmentMeta(a: {
  id: string;
  txn_id: string | null;
  filename: string;
  mime: string;
  size_bytes: number;
  sha256: string;
  created_at: string;
}) {
  return {
    id: a.id,
    txn_id: a.txn_id,
    filename: a.filename,
    mime: a.mime,
    size_bytes: a.size_bytes,
    sha256: a.sha256,
    created_at: a.created_at,
  };
}

function base64Size(b64: string): number {
  const clean = b64.replace(/\s/g, '');
  if (!/^[A-Za-z0-9+/]*={0,2}$/.test(clean)) invalid('conteúdo do anexo inválido (base64)');
  return (
    Math.floor((clean.length * 3) / 4) - (clean.endsWith('==') ? 2 : clean.endsWith('=') ? 1 : 0)
  );
}

registerMockHandlers(
  guarded({
    'transactions.query': (args) => queryTxns(args),
    'transactions.get': (args) => txnView(str(args, 'id')),
    'transactions.cross_account_duplicates': () =>
      crossAccountPairs()
        .map((p) => ({
          keep: txnView(p.keep.id),
          copy: txnView(p.copy.id),
          copy_is_transfer_leg: p.leg,
        }))
        .sort((x, y) => y.copy.date.localeCompare(x.copy.date)),
    'transactions.remove_copies': (args) => {
      const ok = new Map(crossAccountPairs().map((p) => [p.copy.id, p.leg]));
      const ids = idsArg(args);
      const del2 = ids.filter((id) => ok.has(id));
      const skipped = ids.filter((id) => !ok.has(id));
      if (!del2.length)
        return { removed: 0, balance_syncs_adjusted: 0, skipped, change_set_id: null };
      const rec = write(`${del2.length} cópias entre contas apagadas`, () => {
        // Cópia ligada como transferência: desfaz a ligação; o outro lado volta a ser
        // despesa ou receita pelo sinal.
        for (const id of del2) {
          const row = find('txn', id);
          if (!row?.transfer_id) continue;
          for (const other of rows('txn').filter(
            (r) => r.transfer_id === row.transfer_id && r.id !== id,
          ))
            patch('txn', other.id, {
              transfer_id: null,
              kind: other.amount < 0 ? 'expense' : 'income',
            });
          patch('txn', id, { transfer_id: null });
        }
        return deleteTxns(del2, false);
      });
      return {
        removed: rec.value,
        balance_syncs_adjusted: 0,
        skipped,
        change_set_id: rec.changeSetId,
      };
    },
    'transactions.create': (args) => {
      const desc = typeof args.description === 'string' ? args.description : '';
      const id = write(`Lançamento criado: ${short(desc)}`, () => {
        const id = create(args);
        linkCategoryPayments([id]);
        return id;
      }).value;
      return txnView(id);
    },
    'transactions.update': (args) => {
      const id = str(args, 'id');
      const cur = txnRow(id);
      write(`Lançamento alterado: ${short(cur.description)}`, () => {
        update(args);
        linkCategoryPayments([id]);
      });
      return txnView(id);
    },
    'transactions.delete': (args): BulkResult => {
      const ids = idsArg(args);
      if (ids.length === 0) invalid('nenhum lançamento selecionado');
      const reason =
        ids.length === 1
          ? `Lançamento apagado: ${short(txnRow(ids[0]!).description)}`
          : `${ids.length} lançamentos apagados`;
      const rec = write(reason, () => deleteTxns(ids));
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'transactions.create_transfer': (args): TransferResult => {
      const input = args as unknown as TransferInput;
      const label =
        input.kind === 'card_payment'
          ? 'Pagamento de fatura'
          : input.kind === 'investment'
            ? 'Aporte'
            : 'Transferência';
      const name = (id: string) => find('account', id)?.name ?? '?';
      const reason = `${label}: ${name(input.from_account_id)} → ${name(input.to_account_id)}`;
      const ids = write(reason, () => createTransfer(input)).value;
      return { transfer_id: ids.transferId, from: txnView(ids.fromId), to: txnView(ids.toId) };
    },
    'transactions.set_category': (args): BulkResult => {
      const ids = idsArg(args);
      const categoryId = cleanOpt(args.category_id);
      const reason = categoryId
        ? `Categoria ${categoryPaths().get(categoryId) ?? notFound(`categoria ${categoryId}`)} definida em ${ids.length} lançamento(s)`
        : `Categoria removida de ${ids.length} lançamento(s)`;
      const rec = write(reason, () => {
        const r = setCategory(ids, categoryId);
        linkCategoryPayments(ids);
        return r;
      });
      return {
        updated: rec.value.n,
        change_set_id: rec.changeSetId,
        ...(rec.value.skipped ? { skipped: rec.value.skipped } : {}),
      };
    },
    'transactions.set_status': (args): BulkResult => {
      const ids = idsArg(args);
      const status =
        statusArg(args.status) ?? invalid('argumentos inválidos: missing field `status`');
      const reason = `Status alterado para ${statusLabel(status)} em ${ids.length} lançamento(s)`;
      const rec = write(reason, () => setStatus(ids, status));
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'transactions.set_member': (args): BulkResult => {
      const ids = idsArg(args);
      const memberId = cleanOpt(args.member_id);
      const reason = memberId
        ? `Membro ${find('member', memberId)?.name ?? notFound(`membro ${memberId}`)} definido em ${ids.length} lançamento(s)`
        : `Membro removido de ${ids.length} lançamento(s)`;
      const rec = write(reason, () => setMember(ids, memberId));
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'transactions.edit_tags': (args): BulkResult => {
      const ids = idsArg(args);
      const list = (v: unknown) => (Array.isArray(v) ? v.map(String) : []);
      const rec = write(`Tags alteradas em ${ids.length} lançamento(s)`, () =>
        editTags(ids, list(args.add), list(args.remove)),
      );
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'transactions.same_description': (args) => sameDescription(idsArg(args), args.filter ?? {}),
    'transactions.query_ids': (args) => queryTxnIds(args),
    'transactions.list_tags': () => tagSummaries(),
    'transactions.create_tag': (args) => {
      const name = cleanName(args.name, 'da tag');
      return write(`Tag criada: ${name}`, () => {
        if (rows('tag').some((t) => t.name.toLowerCase() === name.toLowerCase())) {
          conflict(`já existe uma tag chamada ${name}`);
        }
        const now = nowUtc();
        const t = put('tag', {
          id: newId(),
          name,
          color: cleanOpt(args.color),
          created_at: now,
          updated_at: now,
        });
        return { id: t.id, name: t.name, color: t.color };
      }).value;
    },
    'transactions.update_tag': (args) => {
      const id = str(args, 'id');
      return write('Tag alterada', () => {
        const cur = find('tag', id) ?? notFound(`tag ${id}`);
        const name = has(args, 'name') ? cleanName(args.name, 'da tag') : cur.name;
        if (rows('tag').some((t) => t.id !== id && t.name.toLowerCase() === name.toLowerCase())) {
          conflict(`já existe uma tag chamada ${name}`);
        }
        const t = patch('tag', id, {
          name,
          color: has(args, 'color') ? cleanOpt(args.color) : cur.color,
          updated_at: nowUtc(),
        });
        return { id: t.id, name: t.name, color: t.color };
      }).value;
    },
    'transactions.delete_tag': (args) => {
      const id = str(args, 'id');
      const tag = find('tag', id) ?? notFound(`tag ${id}`);
      write(`Tag apagada: ${tag.name}`, () => {
        for (const l of rows('txn_tag')) if (l.tag_id === id) del('txn_tag', l.id);
        del('tag', id);
      });
      return null;
    },
    'transactions.list_attachments': (args) => {
      const txnId = str(args, 'txn_id');
      txnRow(txnId);
      return rows('attachment')
        .filter((a) => a.txn_id === txnId)
        .sort((a, b) => a.created_at.localeCompare(b.created_at))
        .map(attachmentMeta);
    },
    'transactions.add_attachment': (args) => {
      const txnId = str(args, 'txn_id');
      const filename = str(args, 'filename').trim();
      const mime = str(args, 'mime').trim();
      const content = str(args, 'content_base64');
      return write(`Anexo adicionado: ${short(filename)}`, () => {
        txnRow(txnId);
        if (!filename) invalid('informe o nome do arquivo');
        const size = base64Size(content);
        if (size === 0) invalid('o anexo está vazio');
        if (size > 20 * 1024 * 1024) invalid('o anexo passa de 20 MB');
        const now = nowUtc();
        const row = put('attachment', {
          id: newId(),
          txn_id: txnId,
          filename,
          mime: mime || 'application/octet-stream',
          size_bytes: size,
          sha256: hashHex(content),
          data: content,
          created_at: now,
          updated_at: now,
        });
        return attachmentMeta(row);
      }).value;
    },
    'transactions.get_attachment': (args) => {
      const a = find('attachment', str(args, 'id')) ?? notFound(`anexo ${String(args.id)}`);
      return { ...attachmentMeta(a), content_base64: a.data };
    },
    'transactions.save_attachment': (args) => {
      const a = find('attachment', str(args, 'id')) ?? notFound(`anexo ${String(args.id)}`);
      return { path: str(args, 'path'), bytes: a.size_bytes };
    },
    'transactions.delete_attachment': (args) => {
      const a = find('attachment', str(args, 'id')) ?? notFound(`anexo ${String(args.id)}`);
      write(`Anexo apagado: ${short(a.filename)}`, () => del('attachment', a.id));
      return null;
    },
  }),
);
