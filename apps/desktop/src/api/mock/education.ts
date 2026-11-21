/**
 * Mock da meta de faculdade (`goals.education_*`, `goals.holdings_*`): a mesma conta do núcleo
 * (`goals::education`), com a inflação da educação fixa em 6% contra 4% de inflação geral.
 */
import type { EducationBrief } from '@/api/generated/EducationBrief';
import type { EducationInflation } from '@/api/generated/EducationInflation';
import type { EducationOverview } from '@/api/generated/EducationOverview';
import type { EducationPlan } from '@/api/generated/EducationPlan';
import type { EducationPlanView } from '@/api/generated/EducationPlanView';
import type { EducationResult } from '@/api/generated/EducationResult';
import type { EducationReview } from '@/api/generated/EducationReview';
import type { EducationSaveArgs } from '@/api/generated/EducationSaveArgs';
import type { EducationYear } from '@/api/generated/EducationYear';
import type { GoalView } from '@/api/generated/GoalView';
import type { HoldingInput } from '@/api/generated/HoldingInput';
import type { HoldingView } from '@/api/generated/HoldingView';
import type { ObjectiveAllocation } from '@/api/generated/ObjectiveAllocation';

import { mockFail, onMockReset } from '../mockRegistry';

import { register, str } from './ledger/common';
import { today } from './ledger/dates';
import { extraSources, goalCreate, goalGet, goalsList, goalUpdate } from './ledger/goalsState';
import { portfolioMarketValue } from './portfolio';
import { rows } from './ledger/store';

const plans = new Map<string, EducationPlan>();
const reviews = new Map<string, EducationReview[]>();
const holdings = new Map<string, (HoldingInput & { id: string })[]>();
let seq = 0;
onMockReset(() => {
  plans.clear();
  reviews.clear();
  holdings.clear();
});

const INFLATION: EducationInflation = {
  country: 'BR',
  mode: 'series_10y',
  series_id: 'ipca_g8',
  series_name: 'IPCA · Educação',
  headline_id: 'ipca',
  headline_name: 'IPCA',
  spread: 1.06 / 1.04 - 1,
  education_annual: 0.06,
  headline_annual: 0.04,
  from: '2016-01',
  to: '2025-12',
  years: 10,
  last_data: '2026-09',
  missing: false,
};

export function defaultPlan(): EducationPlan {
  return {
    member_id: null,
    expected_birth: null,
    start_age: 18,
    start_month: 2,
    start_date: null,
    institution_type: 'private',
    course: null,
    duration_months: 48,
    payments_per_year: 12,
    fee: 0,
    fee_date: today(),
    yearly_extras: 0,
    monthly_living: 0,
    discount_pct: 0,
    inflation_mode: 'series_10y',
    inflation_series: null,
    fixed_spread: null,
    real_return: 0.03,
    contribution_mode: 'indexed',
    pool_rule: 'earliest_first',
  };
}

// ── Datas ────────────────────────────────────────────────────────

const ym = (iso: string) => ({ y: Number(iso.slice(0, 4)), m: Number(iso.slice(5, 7)) });
const iso = (y: number, m: number) => `${y}-${String(m).padStart(2, '0')}-01`;
function addMonths(isoDate: string, n: number): string {
  const { y, m } = ym(isoDate);
  const total = y * 12 + (m - 1) + n;
  return iso(Math.floor(total / 12), (total % 12) + 1);
}
const monthsBetween = (a: string, b: string) => {
  const x = ym(a);
  const z = ym(b);
  return z.y * 12 + z.m - (x.y * 12 + x.m);
};
/** Reajustes (no mês de início do ano letivo) entre a cotação e a data. */
function steps(feeDate: string, date: string, month: number): number {
  const fy = Number(feeDate.slice(0, 4));
  const dy = Number(date.slice(0, 4));
  const first = iso(fy, month) > feeDate ? fy : fy + 1;
  const last = iso(dy, month) <= date ? dy : dy - 1;
  return Math.max(0, last - first + 1);
}

function birthOf(p: EducationPlan): { name: string | null; birth: string | null } {
  if (p.member_id) {
    const m = rows('member').find((x) => x.id === p.member_id);
    return { name: m?.name ?? null, birth: m?.profile?.birth_date ?? null };
  }
  return { name: null, birth: p.expected_birth };
}

// ── A conta (igual à do núcleo) ──────────────────────────────────

export function compute(p: EducationPlan, saved: number, birthOver?: string): EducationResult {
  const birth = birthOver ?? birthOf(p).birth;
  let start: string;
  if (p.start_date) start = `${p.start_date.slice(0, 7)}-01`;
  else if (birth) start = iso(Number(birth.slice(0, 4)) + p.start_age, p.start_month);
  else return mockFail('invalid', 'cadastre a data de nascimento do filho (Família)');
  const end = addMonths(start, p.duration_months - 1);
  const now = `${today().slice(0, 7)}-01`;
  const n = monthsBetween(now, start);
  const spread =
    p.inflation_mode === 'headline'
      ? 0
      : p.inflation_mode === 'fixed_spread'
        ? (p.fixed_spread ?? 0)
        : INFLATION.spread;
  const pi = INFLATION.headline_annual ?? 0.04;
  const rm = (1 + p.real_return) ** (1 / 12) - 1;
  const pim = (1 + pi) ** (1 / 12) - 1;
  const month = Number(start.slice(5, 7));
  const every = 12 / p.payments_per_year;
  const net = p.fee * (1 - p.discount_pct / 100);
  const pays = (s: number) =>
    Array.from({ length: p.duration_months }, (_, k) => {
      const inYear = k % 12;
      let e = inYear % every === 0 ? net : 0;
      if (inYear === 0) e += p.yearly_extras;
      return (
        Math.round(e * (1 + s) ** steps(p.fee_date, addMonths(start, k), month)) + p.monthly_living
      );
    });
  const pv = (xs: number[]) => xs.reduce((a, x, k) => a + x / (1 + rm) ** k, 0);
  const payments = pays(spread);
  const target = pv(payments);
  const phase = now > end ? 'finished' : n <= 0 ? 'paying' : 'saving';
  const annuity = (need: number) =>
    need <= 0 ? 0 : rm === 0 ? need / n : (need * rm) / ((1 + rm) ** n - 1);
  const feeReal = (d: string) => Math.round(net * (1 + spread) ** steps(p.fee_date, d, month));
  const yearsFromNow = (d: string) => Math.max(0, monthsBetween(now, d)) / 12;
  const r: EducationResult = {
    birth_date: birth,
    child_age: birth ? Number(today().slice(0, 4)) - Number(birth.slice(0, 4)) : null,
    start_date: start,
    end_date: end,
    months_to_start: n,
    phase,
    inflation: { ...INFLATION, mode: p.inflation_mode, spread },
    headline_assumed: pi,
    headline_default: false,
    fee_at_start: feeReal(start),
    fee_at_start_nominal: Math.round(feeReal(start) * (1 + pi) ** yearsFromNow(start)),
    total_payments: payments.reduce((a, x) => a + x, 0),
    target_at_start: Math.round(target),
    saved,
    monthly_needed: 0,
    monthly_indexed: 0,
    monthly_flat: 0,
    monthly_headline_only: 0,
    monthly_next_year: 0,
    uncovered_year: null,
    shortfall: 0,
    years: [],
    notes: [],
  };
  if (phase === 'saving') {
    const need = target - saved * (1 + rm) ** n;
    const indexed = annuity(need);
    let factor = 0;
    for (let j = 1; j <= n; j++) factor += (1 + rm) ** (n - j) / (1 + pim) ** j;
    r.monthly_indexed = Math.round(indexed);
    r.monthly_flat = Math.round(need > 0 ? need / factor : 0);
    r.monthly_headline_only = Math.round(annuity(pv(pays(0)) - saved * (1 + rm) ** n));
    r.monthly_next_year = Math.round(indexed * (1 + pi));
    r.shortfall = Math.round(Math.max(0, need));
    r.monthly_needed = p.contribution_mode === 'flat' ? r.monthly_flat : r.monthly_indexed;
  } else if (phase === 'paying') {
    const left = payments.slice(-n);
    let bal = saved;
    for (let i = 0; i < left.length; i++) {
      bal -= left[i] ?? 0;
      if (bal < -0.5) {
        r.uncovered_year = Math.floor((-n + i) / 12) + 1;
        break;
      }
      bal *= 1 + rm;
    }
    r.shortfall = Math.round(Math.max(0, pv(left) - saved));
  }
  if (phase !== 'finished') {
    const years = new Map<number, EducationYear>();
    let bal = saved;
    for (let d = now; d <= end; d = addMonths(d, 1)) {
      const y = Number(d.slice(0, 4));
      const row = years.get(y) ?? {
        year: y,
        in_course: false,
        fee_real: 0,
        fee_nominal: 0,
        payments_real: 0,
        planned_balance: Math.round(bal),
      };
      years.set(y, row);
      const k = monthsBetween(start, d);
      if (k >= 0) {
        const pay = payments[k] ?? 0;
        row.in_course = true;
        row.payments_real += pay;
        bal -= pay;
      } else bal += r.monthly_indexed;
      bal *= 1 + rm;
    }
    for (const row of years.values()) {
      const on = iso(row.year, month) > now ? iso(row.year, month) : now;
      row.fee_real = feeReal(on);
      row.fee_nominal = Math.round(row.fee_real * (1 + pi) ** yearsFromNow(on));
    }
    r.years = [...years.values()];
  }
  if (p.institution_type === 'public' && p.monthly_living === 0)
    r.notes.push(
      'Em faculdade pública, o custo costuma ser o de vida (moradia, transporte, alimentação em outra cidade). Vale preencher o custo de vida por mês.',
    );
  return r;
}

// ── Metas ────────────────────────────────────────────────────────

function holdingViews(goalId: string, currency: string): HoldingView[] {
  const accounts = rows('account');
  return (holdings.get(goalId) ?? []).map((h) => {
    const acc = accounts.find((a) => a.id === h.account_id);
    const full = 1_000_000;
    return {
      id: h.id,
      account_id: h.account_id,
      account_name: acc?.name ?? h.account_id,
      asset_id: h.asset_id ?? null,
      asset_name: null,
      ticker: null,
      share: h.share ?? null,
      pool_share: h.share == null ? 1 : null,
      full_value: full,
      value: Math.round(full * (h.share ?? 1)),
      currency,
      maturity_date: null,
      missing_price: false,
      value_mode: h.value_mode ?? 'net_redemption',
      value_used: 'market',
    };
  });
}

function brief(p: EducationPlan, r: EducationResult): EducationBrief {
  return {
    member_id: p.member_id,
    child_name: birthOf(p).name,
    start_date: r.start_date,
    end_date: r.end_date,
    months_to_start: r.months_to_start,
    phase: r.phase,
    institution_type: p.institution_type,
    course: p.course,
    spread: r.inflation.spread,
    target_at_start: r.target_at_start,
    monthly_needed: r.monthly_needed,
  };
}

/** A meta com o plano por cima (alvo, data e aporte vêm da conta). */
export function withEducation(g: GoalView): GoalView {
  const hv = holdingViews(g.id, g.currency).reduce((a, h) => a + h.value, 0);
  const current = g.current_amount + hv;
  const p = plans.get(g.id);
  const base = hv ? { ...g, current_amount: current, holdings_value: hv } : g;
  if (!p) return base;
  const r = compute(p, current);
  return {
    ...base,
    target_amount: Math.max(1, r.target_at_start),
    target_date: r.start_date,
    monthly_needed: r.monthly_needed,
    remaining: Math.max(0, r.target_at_start - current),
    progress: current / Math.max(1, r.target_at_start),
    education: brief(p, r),
  };
}

function planView(goalId: string): EducationPlanView {
  const p = plans.get(goalId) ?? mockFail('not_found', `plano de educação da meta ${goalId}`);
  const goal = withEducation(goalGet(goalId));
  return {
    goal,
    plan: p,
    child_name: birthOf(p).name,
    result: compute(p, goal.current_amount),
    holdings: holdingViews(goalId, goal.currency),
    reviews: reviews.get(goalId) ?? [],
  };
}

function record(
  goalId: string,
  reason: EducationReview['reason'],
  r: EducationResult,
  fee: number,
) {
  const list = reviews.get(goalId) ?? [];
  list.unshift({
    id: `rev-${++seq}`,
    reviewed_at: new Date().toISOString(),
    reason,
    spread: r.inflation.spread,
    spread_from: r.inflation.from,
    spread_to: r.inflation.to,
    fee,
    target_at_start: r.target_at_start,
    saved: r.saved,
    monthly_needed: r.monthly_needed,
  });
  reviews.set(goalId, list);
}

register({
  'goals.education_inflation': () => INFLATION,
  'goals.education_simulate': (a) => {
    const id = a.goal_id as string | undefined;
    const base = (id && plans.get(id)) || defaultPlan();
    const plan = { ...base, ...((a.plan as Partial<EducationPlan> | undefined) ?? {}) };
    const saved =
      (a.saved as number | undefined) ?? (id ? withEducation(goalGet(id)).current_amount : 0);
    return compute(plan, saved, a.birth_date as string | undefined);
  },
  'goals.education_get': (a) => planView(str(a, 'goal_id')),
  'goals.education_save': (a) => {
    const args = a as unknown as EducationSaveArgs;
    const p = args.plan;
    if (!p.member_id && !p.expected_birth)
      mockFail('invalid', 'escolha o filho ou informe a data prevista do nascimento');
    const prev = args.goal_id ? plans.get(args.goal_id) : undefined;
    const id =
      args.goal_id ??
      goalCreate({
        name: args.name?.trim() || `Faculdade de ${birthOf(p).name ?? 'filho'}`,
        kind: 'education',
        target_amount: 1,
        initial_amount: args.initial_amount ?? 0,
        currency: args.currency,
      }).id;
    if (args.goal_id)
      goalUpdate({ id, name: args.name, initial_amount: args.initial_amount, kind: 'education' });
    plans.set(id, p);
    const goal = withEducation(goalGet(id));
    const r = compute(p, goal.current_amount);
    goalUpdate({
      id,
      target_amount: Math.max(1, r.target_at_start),
      target_date: r.start_date,
      monthly_contribution: r.monthly_needed,
    });
    if (!prev) record(id, 'created', r, p.fee);
    else if (JSON.stringify(prev) !== JSON.stringify(p))
      record(id, prev.fee !== p.fee ? 'fee_update' : 'edit', r, p.fee);
    return planView(id);
  },
  'goals.education_overview': (): EducationOverview => {
    const items = [...plans.keys()]
      .map((id) => withEducation(goalGet(id)))
      .filter((g) => !g.archived && g.education)
      .map((g) => ({
        goal_id: g.id,
        name: g.name,
        currency: g.currency,
        saved: g.current_amount,
        brief: g.education as EducationBrief,
      }));
    const count = new Map<number, number>();
    for (const i of items)
      for (
        let y = Number(i.brief.start_date.slice(0, 4));
        y <= Number(i.brief.end_date.slice(0, 4));
        y++
      )
        count.set(y, (count.get(y) ?? 0) + 1);
    return {
      plans: items,
      monthly_total: items.reduce((a, i) => a + i.brief.monthly_needed, 0),
      saved_total: items.reduce((a, i) => a + i.saved, 0),
      target_total: items.reduce((a, i) => a + i.brief.target_at_start, 0),
      currency: 'BRL',
      overlap_years: [...count].filter(([, c]) => c >= 2).map(([y]) => y),
    };
  },
  'goals.objectives': (): ObjectiveAllocation => {
    const slices = goalsList(false)
      .filter((g) => (holdings.get(g.id) ?? []).length > 0)
      .map((g) => {
        const views = holdingViews(g.id, g.currency);
        return {
          goal_id: g.id,
          goal_name: g.name,
          kind: g.kind,
          value: views.reduce((a, h) => a + h.value, 0),
          share: 0,
          target_amount: g.target_amount,
          positions: views.length,
        };
      });
    const total = portfolioMarketValue();
    const assigned = slices.reduce((a, s) => a + s.value, 0);
    const unassigned = Math.max(total - assigned, 0);
    const whole = Math.max(assigned + unassigned, 1);
    for (const s of slices) s.share = s.value / whole;
    slices.sort((a, b) => b.value - a.value || a.goal_name.localeCompare(b.goal_name));
    return {
      currency: 'BRL',
      total,
      slices,
      unassigned,
      unassigned_share: unassigned / whole,
    };
  },
  'goals.holdings_get': (a) => {
    const g = goalGet(str(a, 'goal_id'));
    return holdingViews(g.id, g.currency);
  },
  'goals.holdings_set': (a) => {
    const g = goalGet(str(a, 'goal_id'));
    const list = (a.holdings as HoldingInput[]).map((h) => ({ ...h, id: `hold-${++seq}` }));
    holdings.set(g.id, list);
    return holdingViews(g.id, g.currency);
  },
  // Fontes da meta (docs/goal-sources.md): contas na própria meta, ativos nas posições ligadas
  // e etiquetas à parte.
  'goals.link_source': (a) => {
    const g = goalGet(str(a, 'goal_id'));
    const ref = str(a, 'ref_id');
    if (a.kind === 'account') {
      if (!g.account_ids.includes(ref))
        goalUpdate({ id: g.id, account_ids: [...g.account_ids, ref] });
    } else if (a.kind === 'asset') {
      const account = typeof a.account_id === 'string' ? a.account_id : '';
      if (!account) mockFail('invalid', 'o ativo está em mais de uma conta; informe qual');
      const list = (holdings.get(g.id) ?? []).filter(
        (h) => !(h.account_id === account && h.asset_id === ref),
      );
      list.push({ account_id: account, asset_id: ref, id: `hold-${++seq}` });
      holdings.set(g.id, list);
    } else {
      const list = (extraSources.get(g.id) ?? []).filter((s) => s.ref_id !== ref);
      list.push({
        kind: 'asset_tag',
        ref_id: ref,
        account_id: null,
        value_mode: 'net_redemption',
        share: null,
      });
      extraSources.set(g.id, list);
    }
    return withEducation(goalGet(g.id));
  },
  'goals.unlink_source': (a) => {
    const g = goalGet(str(a, 'goal_id'));
    const ref = str(a, 'ref_id');
    if (a.kind === 'account') {
      goalUpdate({ id: g.id, account_ids: g.account_ids.filter((x) => x !== ref) });
    } else if (a.kind === 'asset') {
      holdings.set(
        g.id,
        (holdings.get(g.id) ?? []).filter((h) => h.asset_id !== ref),
      );
    } else {
      extraSources.set(
        g.id,
        (extraSources.get(g.id) ?? []).filter((s) => s.ref_id !== ref),
      );
    }
    return withEducation(goalGet(g.id));
  },
  'goals.liquidity_candidates': () => [],
});
