/** Plásticos do cartão no mock (V155), como `cards/plastics.rs`: titular, adicionais e virtuais. */
import type { CardPlastic } from '@/api/generated/CardPlastic';
import type { PlasticSubtotal } from '@/api/generated/PlasticSubtotal';

import { nowUtc, today } from './dates';
import { reinheritMembers, requireCard, requireMember } from './domain';
import { cleanOpt, conflict, invalid, notFound, str } from './errors';
import { type CardPlasticRow, del, find, newId, patch, put, rows } from './store';

const KIND_ORDER: Record<CardPlasticRow['kind'], number> = {
  holder: 0,
  additional: 1,
  virtual: 2,
  temporary: 3,
};
const KINDS = Object.keys(KIND_ORDER) as CardPlasticRow['kind'][];

/** Aceita `1234`, `final 1234` ou `**** 1234`; recusa mais de 4 dígitos. */
export function normalizeLast4(raw: string): string {
  const digits = raw.replace(/\D/g, '');
  if (digits.length > 4)
    invalid('guarde só os 4 últimos dígitos do cartão, nunca o número inteiro');
  if (digits.length !== 4) invalid('informe os 4 últimos dígitos do cartão (só números)');
  return digits;
}

function cleanLast4(v: unknown): string | null {
  const s = cleanOpt(v);
  return s ? normalizeLast4(s) : null;
}

function kindArg(v: unknown): CardPlasticRow['kind'] {
  if (typeof v === 'string' && (KINDS as string[]).includes(v)) return v as CardPlasticRow['kind'];
  return invalid('tipo de cartão desconhecido (titular, adicional, virtual ou temporário)');
}

function checkUnique(accountId: string, last4: string, selfId?: string): void {
  const other = rows('card_plastic').find(
    (p) => p.account_id === accountId && p.last4 === last4 && p.id !== selfId,
  );
  if (other) conflict(`este cartão já tem um plástico com final ${last4}`);
}

function checkLimit(v: number | null): void {
  if (v != null && v < 0) invalid('o sublimite não pode ser negativo');
}

export function plasticRow(id: string): CardPlasticRow {
  return find('card_plastic', id) ?? notFound(`cartão ${id}`);
}

export function findByLast4(accountId: string, last4: string): string | null {
  return (
    rows('card_plastic').find((p) => p.account_id === accountId && p.last4 === last4)?.id ?? null
  );
}

function view(p: CardPlasticRow, T = today()): CardPlastic {
  let cycleSpent = 0;
  let count = 0;
  for (const t of rows('txn')) {
    if (t.card_plastic_id !== p.id) continue;
    count++;
    const s = find('card_statement', t.statement_id);
    if (t.kind !== 'card_payment' && s && s.period_start <= T && s.closing_date >= T)
      cycleSpent -= t.amount;
  }
  return {
    id: p.id,
    account_id: p.account_id,
    kind: p.kind,
    last4: p.last4,
    member_id: p.member_id,
    member_name: find('member', p.member_id)?.name ?? null,
    label: p.label,
    spend_limit: p.spend_limit,
    active: p.active,
    cycle_spent: cycleSpent,
    txn_count: count,
  };
}

export function listPlastics(accountId?: string | null): CardPlastic[] {
  return rows('card_plastic')
    .filter((p) => !accountId || p.account_id === accountId)
    .sort(
      (a, b) =>
        a.account_id.localeCompare(b.account_id) ||
        KIND_ORDER[a.kind] - KIND_ORDER[b.kind] ||
        Number(b.active) - Number(a.active) ||
        (a.last4 ?? '').localeCompare(b.last4 ?? '') ||
        a.created_at.localeCompare(b.created_at),
    )
    .map((p) => view(p));
}

export function plasticView(id: string): CardPlastic {
  return view(plasticRow(id));
}

export function createPlastic(args: Record<string, unknown>): string {
  const accountId = str(args, 'account_id');
  requireCard(accountId);
  const kind = kindArg(args.kind);
  const last4 = cleanLast4(args.last4);
  if (last4) checkUnique(accountId, last4);
  const memberId = cleanOpt(args.member_id);
  if (memberId) requireMember(memberId);
  const limit = typeof args.spend_limit === 'number' ? args.spend_limit : null;
  checkLimit(limit);
  const now = nowUtc();
  const row = put('card_plastic', {
    id: newId(),
    account_id: accountId,
    kind,
    last4,
    member_id: memberId,
    label: cleanOpt(args.label),
    spend_limit: limit,
    active: true,
    created_at: now,
    updated_at: now,
  });
  return row.id;
}

export function updatePlastic(args: Record<string, unknown>): void {
  const cur = plasticRow(str(args, 'id'));
  const has = (k: string) => Object.prototype.hasOwnProperty.call(args, k);
  const next: CardPlasticRow = { ...cur };
  if (has('kind')) next.kind = kindArg(args.kind);
  if (has('last4')) next.last4 = cleanLast4(args.last4);
  if (next.last4) checkUnique(cur.account_id, next.last4, cur.id);
  if (has('member_id')) {
    next.member_id = cleanOpt(args.member_id);
    if (next.member_id) requireMember(next.member_id);
  }
  if (has('label')) next.label = cleanOpt(args.label);
  if (has('spend_limit'))
    next.spend_limit = typeof args.spend_limit === 'number' ? args.spend_limit : null;
  checkLimit(next.spend_limit);
  if (typeof args.active === 'boolean') next.active = args.active;
  patch('card_plastic', cur.id, { ...next, updated_at: nowUtc() });
  if (next.member_id !== cur.member_id) reinheritMembers(cur.account_id);
}

export function deletePlastic(id: string): number {
  const cur = plasticRow(id);
  let n = 0;
  for (const t of rows('txn')) {
    if (t.card_plastic_id !== id) continue;
    patch('txn', t.id, { card_plastic_id: null, updated_at: nowUtc() });
    n++;
  }
  del('card_plastic', id);
  reinheritMembers(cur.account_id);
  return n;
}

/** O final do cadastro do cartão mudou: o plástico do titular acompanha (ou nasce). */
export function syncHolderLast4(accountId: string, last4: string | null | undefined): void {
  if (!last4 || findByLast4(accountId, last4)) return;
  const holder = rows('card_plastic')
    .filter((p) => p.account_id === accountId && p.kind === 'holder')
    .sort((a, b) => a.created_at.localeCompare(b.created_at))[0];
  const now = nowUtc();
  if (holder) patch('card_plastic', holder.id, { last4, updated_at: now });
  else
    put('card_plastic', {
      id: newId(),
      account_id: accountId,
      kind: 'holder',
      last4,
      member_id: null,
      label: null,
      spend_limit: null,
      active: true,
      created_at: now,
      updated_at: now,
    });
}

/** Subtotais da fatura por plástico; vazio se nenhuma compra tem plástico. */
export function plasticSubtotals(statementId: string): PlasticSubtotal[] {
  const groups = new Map<string, PlasticSubtotal>();
  for (const t of rows('txn')) {
    if (t.statement_id !== statementId || t.kind === 'card_payment') continue;
    const key = `${t.card_plastic_id ?? ''}|${t.member_id ?? ''}`;
    let g = groups.get(key);
    if (!g) {
      const p = find('card_plastic', t.card_plastic_id ?? null);
      g = {
        card_plastic_id: p?.id ?? null,
        kind: p?.kind ?? null,
        last4: p?.last4 ?? null,
        label: p?.label ?? null,
        member_id: t.member_id,
        member_name: find('member', t.member_id)?.name ?? null,
        total: 0,
        txn_count: 0,
      };
      groups.set(key, g);
    }
    g.total -= t.amount;
    g.txn_count++;
  }
  const out = [...groups.values()];
  if (out.every((s) => !s.card_plastic_id)) return [];
  const rank = (s: PlasticSubtotal) => (s.kind ? KIND_ORDER[s.kind] : 9);
  return out.sort(
    (a, b) =>
      Number(!a.card_plastic_id) - Number(!b.card_plastic_id) ||
      rank(a) - rank(b) ||
      (a.last4 ?? '').localeCompare(b.last4 ?? '') ||
      (a.member_name ?? '').localeCompare(b.member_name ?? ''),
  );
}

/** Ao apagar a conta do cartão. */
export function dropPlasticsOf(accountId: string): void {
  for (const p of rows('card_plastic')) if (p.account_id === accountId) del('card_plastic', p.id);
}
