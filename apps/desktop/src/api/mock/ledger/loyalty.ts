/** Pontos e milhas no mock (V159), como `cards/loyalty.rs`. */
import type { LoyaltyEntry } from '@/api/generated/LoyaltyEntry';
import type { LoyaltyProgram } from '@/api/generated/LoyaltyProgram';

import { addDays, nowUtc, parseYmd, today } from './dates';
import { baseCurrency, requireMember } from './domain';
import { cleanOpt, invalid, notFound, str } from './errors';
import { type LoyaltyEntryRow, del, find, newId, patch, put, rows } from './store';

const OUT_KINDS = new Set(['transfer_out', 'redeem', 'expire']);

function programRow(id: string) {
  return find('loyalty_program', id) ?? notFound(`programa de pontos ${id}`);
}

export function loyaltyEntries(programId: string): LoyaltyEntry[] {
  programRow(programId);
  return rows('loyalty_entry')
    .filter((e) => e.program_id === programId)
    .sort((a, b) => b.date.localeCompare(a.date) || b.created_at.localeCompare(a.created_at))
    .map((e) => ({
      id: e.id,
      program_id: e.program_id,
      date: e.date,
      points: e.points,
      kind: e.kind,
      expires_on: e.expires_on,
      statement_id: e.statement_id,
      transfer_id: e.transfer_id,
      cost: e.cost,
      cost_currency: e.cost_currency,
      notes: e.notes,
    }));
}

/** Lotes abertos e os pontos vencidos sem uso, na ordem das datas (como
 * `cards::loyalty::open_lots`): cada saída usa os lotes vivos na data dela, os que vencem antes
 * primeiro; o registro de expiração usa primeiro os lotes já vencidos. */
function openLots(
  list: LoyaltyEntry[],
  T: string,
): { lots: [string | null, number][]; expired: number } {
  const events = list
    .filter((e) => e.points !== 0)
    .sort((a, b) => a.date.localeCompare(b.date) || Number(a.points < 0) - Number(b.points < 0));
  const lots: [string | null, number][] = [];
  for (const e of events) {
    if (e.points > 0) {
      lots.push([e.expires_on, e.points]);
      lots.sort((a, b) => (a[0] ?? '9999').localeCompare(b[0] ?? '9999'));
      continue;
    }
    let out = -e.points;
    const alive = (exp: string | null) => !exp || exp >= e.date;
    for (const wantAlive of e.kind === 'expire' ? [false, true] : [true]) {
      for (const lot of lots) {
        if (out === 0) break;
        if (lot[1] <= 0 || alive(lot[0]) !== wantAlive) continue;
        const used = Math.min(out, lot[1]);
        lot[1] -= used;
        out -= used;
      }
    }
  }
  const open: [string | null, number][] = [];
  let expired = 0;
  for (const [exp, left] of lots) {
    if (left <= 0) continue;
    if (!exp || exp >= T) open.push([exp, left]);
    else expired += left;
  }
  return { lots: open, expired };
}

function view(id: string, T = today()): LoyaltyProgram {
  const p = programRow(id);
  const list = loyaltyEntries(id);
  const { lots, expired } = openLots(list, T);
  const balance = list.reduce((s, e) => s + e.points, 0) - expired;
  const limit = addDays(T, 90);
  const next = lots.find(([d]) => d)?.[0] ?? null;
  const bought = list.filter((e) => e.kind === 'purchase' && e.cost != null);
  const boughtPoints = bought.reduce((s, e) => s + e.points, 0);
  const cards = rows('card_reward_program')
    .filter((r) => r.loyalty_program_id === id && (!r.ends_on || r.ends_on >= T))
    .map((r) => ({
      account_id: r.account_id,
      account_name: find('account', r.account_id)?.name ?? '',
    }));
  return {
    id: p.id,
    name: p.name,
    kind: p.kind,
    member_id: p.member_id,
    member_name: find('member', p.member_id)?.name ?? null,
    unit_label: p.unit_label,
    value_per_1000: p.value_per_1000,
    value_currency: p.value_currency,
    active: p.active,
    notes: p.notes,
    balance,
    expiring_90d: lots.filter(([d]) => d && d <= limit).reduce((s, [, n]) => s + n, 0),
    next_expiry: next,
    next_expiry_points: lots.filter(([d]) => d === next && next).reduce((s, [, n]) => s + n, 0),
    estimated_value:
      p.value_per_1000 == null
        ? null
        : Math.trunc((Math.max(0, balance) * p.value_per_1000) / 1000),
    purchase_cost_per_1000:
      boughtPoints > 0
        ? Math.trunc((bought.reduce((s, e) => s + (e.cost ?? 0), 0) * 1000) / boughtPoints)
        : null,
    cards: [...new Map(cards.map((c) => [c.account_id, c])).values()],
  };
}

export function loyaltyList(): LoyaltyProgram[] {
  return rows('loyalty_program')
    .sort(
      (a, b) =>
        Number(b.active) - Number(a.active) ||
        (find('member', a.member_id)?.name ?? '').localeCompare(
          find('member', b.member_id)?.name ?? '',
        ) ||
        a.name.localeCompare(b.name),
    )
    .map((p) => view(p.id));
}

export const loyaltyView = (id: string) => view(id);

export function createLoyalty(args: Record<string, unknown>): string {
  const name = cleanOpt(args.name);
  if (!name) invalid('dê um nome ao programa (ex.: Livelo, Smiles)');
  const member = cleanOpt(args.member_id);
  if (member) requireMember(member);
  const value = typeof args.value_per_1000 === 'number' ? args.value_per_1000 : null;
  if (value != null && value < 0) invalid('valores não podem ser negativos');
  const now = nowUtc();
  const kind = (['bank', 'airline', 'retail', 'other'] as const).find((k) => k === args.kind);
  return put('loyalty_program', {
    id: newId(),
    name,
    kind: kind ?? 'bank',
    member_id: member,
    unit_label: cleanOpt(args.unit_label),
    value_per_1000: value,
    value_currency: value != null ? (cleanOpt(args.value_currency) ?? baseCurrency()) : null,
    active: true,
    notes: cleanOpt(args.notes),
    created_at: now,
    updated_at: now,
  }).id;
}

export function updateLoyalty(args: Record<string, unknown>): void {
  const p = programRow(str(args, 'id'));
  const has = (k: string) => Object.prototype.hasOwnProperty.call(args, k);
  const next = { ...p };
  if (has('name')) {
    const n = cleanOpt(args.name);
    if (!n) invalid('dê um nome ao programa (ex.: Livelo, Smiles)');
    next.name = n;
  }
  if (has('kind') && typeof args.kind === 'string') next.kind = args.kind as typeof p.kind;
  if (has('member_id')) {
    next.member_id = cleanOpt(args.member_id);
    if (next.member_id) requireMember(next.member_id);
  }
  if (has('unit_label')) next.unit_label = cleanOpt(args.unit_label);
  if (has('value_per_1000')) {
    next.value_per_1000 = typeof args.value_per_1000 === 'number' ? args.value_per_1000 : null;
    if (next.value_per_1000 != null && next.value_per_1000 < 0)
      invalid('valores não podem ser negativos');
    next.value_currency = next.value_per_1000 == null ? null : (p.value_currency ?? baseCurrency());
  }
  if (typeof args.active === 'boolean') next.active = args.active;
  if (has('notes')) next.notes = cleanOpt(args.notes);
  patch('loyalty_program', p.id, { ...next, updated_at: nowUtc() });
}

export function deleteLoyalty(id: string): void {
  programRow(id);
  for (const r of rows('card_reward_program'))
    if (r.loyalty_program_id === id)
      patch('card_reward_program', r.id, { loyalty_program_id: null });
  for (const e of rows('loyalty_entry')) if (e.program_id === id) del('loyalty_entry', e.id);
  del('loyalty_program', id);
}

function insertEntry(args: Record<string, unknown>, transferId: string | null): void {
  const programId = str(args, 'program_id');
  programRow(programId);
  const date = str(args, 'date');
  parseYmd(date);
  const kind = String(args.kind) as LoyaltyEntryRow['kind'];
  const raw = typeof args.points === 'number' ? args.points : 0;
  if (raw === 0) invalid('informe uma quantidade de pontos diferente de zero');
  const points = kind === 'adjust' ? raw : OUT_KINDS.has(kind) ? -Math.abs(raw) : Math.abs(raw);
  const expires = cleanOpt(args.expires_on);
  if (expires) parseYmd(expires);
  const cost = typeof args.cost === 'number' ? args.cost : null;
  if (cost != null && cost < 0) invalid('valores não podem ser negativos');
  const now = nowUtc();
  put('loyalty_entry', {
    id: newId(),
    program_id: programId,
    date,
    points,
    kind,
    expires_on: points > 0 ? expires : null,
    statement_id: cleanOpt(args.statement_id),
    transfer_id: transferId,
    cost,
    cost_currency: cost != null ? (cleanOpt(args.cost_currency) ?? baseCurrency()) : null,
    notes: cleanOpt(args.notes),
    created_at: now,
    updated_at: now,
  });
}

export function addLoyaltyEntry(args: Record<string, unknown>): void {
  if (args.kind === 'transfer_in' || args.kind === 'transfer_out')
    invalid('transferência entre programas é registrada pelos dois lados de uma vez');
  insertEntry(args, null);
}

export function transferLoyalty(args: Record<string, unknown>): void {
  const from = str(args, 'from_program_id');
  const to = str(args, 'to_program_id');
  if (from === to) invalid('a transferência precisa de dois programas diferentes');
  const out = typeof args.points_out === 'number' ? args.points_out : 0;
  const inn = typeof args.points_in === 'number' ? args.points_in : 0;
  if (out <= 0 || inn <= 0) invalid('informe uma quantidade de pontos diferente de zero');
  const tid = newId();
  insertEntry({ ...args, program_id: from, points: out, kind: 'transfer_out' }, tid);
  insertEntry({ ...args, program_id: to, points: inn, kind: 'transfer_in' }, tid);
}

export function deleteLoyaltyEntry(id: string): number {
  const e = find('loyalty_entry', id) ?? notFound('movimento de pontos');
  const ids = e.transfer_id
    ? rows('loyalty_entry')
        .filter((x) => x.transfer_id === e.transfer_id)
        .map((x) => x.id)
    : [id];
  for (const i of ids) del('loyalty_entry', i);
  return ids.length;
}
