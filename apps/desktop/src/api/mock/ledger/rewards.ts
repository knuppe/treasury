/** Cashback e pontos do cartão no mock (V158), como `cards/rewards.rs`. */
import type { RewardProgram } from '@/api/generated/RewardProgram';
import type { RewardsView } from '@/api/generated/RewardsView';
import type { StatementReward } from '@/api/generated/StatementReward';

import { addDays, addMonths, nowUtc, parseYmd, today } from './dates';
import { account, categoryPaths, requireCard } from './domain';
import { cleanOpt, invalid, str } from './errors';
import { type RewardProgramRow, del, find, newId, patch, put, rows } from './store';
import { normalizeDescription } from './text';

function rateOf(v: unknown): number {
  const n = typeof v === 'string' ? Number(v.replace(',', '.')) : NaN;
  if (Number.isNaN(n) || n < 0 || n > 1000)
    invalid('a taxa é um número decimal (0,01 = 1% de cashback; pontos por unidade de moeda)');
  return n;
}

function active(accountId: string, date: string): RewardProgramRow | undefined {
  return rows('card_reward_program')
    .filter(
      (p) => p.account_id === accountId && p.starts_on <= date && (!p.ends_on || p.ends_on >= date),
    )
    .sort((a, b) => b.starts_on.localeCompare(a.starts_on))[0];
}

function programView(p: RewardProgramRow): RewardProgram {
  const paths = categoryPaths();
  return {
    id: p.id,
    account_id: p.account_id,
    kind: p.kind,
    base_rate: p.base_rate,
    rate_currency: p.rate_currency,
    cycle_cap: p.cycle_cap,
    destination: p.destination,
    destination_account_id: p.destination_account_id,
    payout_days: p.payout_days,
    starts_on: p.starts_on,
    ends_on: p.ends_on,
    notes: p.notes,
    loyalty_program_id: p.loyalty_program_id ?? null,
    multipliers: p.multipliers.map((m) => ({
      category_id: m.category_id,
      category_path: paths.get(m.category_id) ?? null,
      rate: m.rate,
    })),
  };
}

export function setRewardProgram(args: Record<string, unknown>): void {
  const accountId = str(args, 'account_id');
  requireCard(accountId);
  const kind = args.kind === 'points' ? 'points' : 'cashback';
  const base = rateOf(args.base_rate);
  if (kind === 'cashback' && base > 1)
    invalid('a taxa é um número decimal (0,01 = 1% de cashback; pontos por unidade de moeda)');
  const cap = typeof args.cycle_cap === 'number' ? args.cycle_cap : null;
  if (cap != null && cap < 0) invalid('o teto por fatura não pode ser negativo');
  const payout = typeof args.payout_days === 'number' ? args.payout_days : 40;
  if (payout < 0 || payout > 180) invalid('o prazo do crédito vai de 0 a 180 dias');
  const starts = cleanOpt(args.starts_on) ?? today();
  parseYmd(starts);
  const now = nowUtc();
  for (const p of rows('card_reward_program')) {
    if (p.account_id !== accountId) continue;
    if (p.starts_on === starts) del('card_reward_program', p.id);
    else if (p.starts_on < starts && (!p.ends_on || p.ends_on >= starts))
      patch('card_reward_program', p.id, { ends_on: addDays(starts, -1), updated_at: now });
  }
  const multipliers = Array.isArray(args.multipliers)
    ? (args.multipliers as { category_id: string; rate: string }[]).map((m) => ({
        category_id: String(m.category_id),
        rate: String(rateOf(m.rate)),
      }))
    : [];
  const dest = args.destination === 'account' || args.destination === 'investment';
  put('card_reward_program', {
    id: newId(),
    account_id: accountId,
    kind,
    base_rate: String(base),
    rate_currency: cleanOpt(args.rate_currency)?.toUpperCase() ?? null,
    cycle_cap: cap,
    destination: dest ? (args.destination as 'account' | 'investment') : 'statement',
    destination_account_id: cleanOpt(args.destination_account_id),
    payout_days: payout,
    starts_on: starts,
    ends_on: null,
    notes: cleanOpt(args.notes),
    multipliers,
    loyalty_program_id: kind === 'points' ? cleanOpt(args.loyalty_program_id) : null,
    created_at: now,
    updated_at: now,
  });
}

export function endRewardProgram(accountId: string): void {
  const T = today();
  for (const p of rows('card_reward_program')) {
    if (p.account_id !== accountId) continue;
    if (p.starts_on > T) del('card_reward_program', p.id);
    else if (!p.ends_on || p.ends_on > T)
      patch('card_reward_program', p.id, { ends_on: T, updated_at: nowUtc() });
  }
}

const skip = (desc: string) => {
  const d = normalizeDescription(desc);
  return (
    d.includes('anuidade') ||
    d.includes('cashback') ||
    d
      .split(/\s+/)
      .some((w) => ['juros', 'encargo', 'encargos', 'rotativo', 'multa', 'mora', 'iof'].includes(w))
  );
};

export function rewardsView(accountId: string, T = today()): RewardsView {
  requireCard(accountId);
  const currency = account(accountId).currency;
  const p = active(accountId, T);
  const out: StatementReward[] = [];
  let expected12 = 0;
  let received12 = 0;
  if (p) {
    const from = addMonths(T, -12);
    const base = Number(p.base_rate);
    const rates = new Map(p.multipliers.map((m) => [m.category_id, Number(m.rate)]));
    const statements = rows('card_statement')
      .filter((s) => s.account_id === accountId && s.period_start <= T)
      .sort((a, b) => a.closing_date.localeCompare(b.closing_date));
    const closed = statements.filter((s) => s.closing_date < T);
    const cbCat = rows('category').find((c) => c.name === 'Cashback e recompensas')?.id;
    const received = new Map<string, number>();
    if (p.kind === 'points' && p.loyalty_program_id) {
      for (const e of rows('loyalty_entry')) {
        if (e.program_id !== p.loyalty_program_id || !e.statement_id) continue;
        if (e.kind !== 'earn' && e.kind !== 'bonus') continue;
        received.set(e.statement_id, (received.get(e.statement_id) ?? 0) + e.points);
      }
    }
    const tracks = p.kind === 'cashback' || Boolean(p.loyalty_program_id);
    if (p.kind === 'cashback') {
      for (const t of rows('txn')) {
        if (t.account_id !== accountId && t.account_id !== p.destination_account_id) continue;
        if (t.amount <= 0 || t.date < from) continue;
        const isCb =
          (cbCat && t.category_id === cbCat) ||
          normalizeDescription(t.description).includes('cashback');
        if (!isCb) continue;
        const s = [...closed].reverse().find((x) => x.closing_date < t.date);
        if (s) received.set(s.id, (received.get(s.id) ?? 0) + t.amount);
      }
    }
    for (const s of [...statements].reverse()) {
      if (s.closing_date < from || s.closing_date < p.starts_on) continue;
      let spend = 0;
      let reward = 0;
      for (const t of rows('txn')) {
        if (t.statement_id !== s.id || t.kind !== 'expense' || skip(t.description)) continue;
        spend -= t.amount;
        const parent = find('category', t.category_id)?.parent_id ?? null;
        const r =
          (t.category_id && rates.get(t.category_id)) || (parent && rates.get(parent)) || base;
        reward += -t.amount * r;
      }
      let expected = p.kind === 'cashback' ? Math.round(reward) : Math.floor(reward / 100);
      if (p.cycle_cap != null) expected = Math.min(expected, p.cycle_cap);
      expected = Math.max(0, expected);
      const dueBy = addDays(s.closing_date, p.payout_days);
      const got = tracks ? (received.get(s.id) ?? 0) : null;
      expected12 += expected;
      received12 += got ?? 0;
      out.push({
        statement_id: s.id,
        reference_month: s.reference_month,
        closing_date: s.closing_date,
        eligible_spend: spend,
        expected,
        received: got,
        due_by: dueBy,
        missing: tracks && dueBy < T && expected > 0 && (got ?? 0) * 2 < expected,
      });
    }
  }
  return {
    account_id: accountId,
    currency,
    program: p ? programView(p) : null,
    statements: out,
    expected_12m: expected12,
    received_12m: received12,
  };
}
