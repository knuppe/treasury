/** Mock de `goals.*` sobre o livro-caixa fictício (`./ledger/goalsState.ts`). */
import type { GoalInput } from '@/api/generated/GoalInput';
import type { GoalPatch } from '@/api/generated/GoalPatch';
import type { GoalView } from '@/api/generated/GoalView';
import type { InssEstimate } from '@/api/generated/InssEstimate';
import type { InssInput } from '@/api/generated/InssInput';
import type { InssRule } from '@/api/generated/InssRule';
import type { PtPensionInput } from '@/api/generated/PtPensionInput';
import type { RetirementPlan } from '@/api/generated/RetirementPlan';
import type { RetirementScenario } from '@/api/generated/RetirementScenario';
import type { RetirementScenarioView } from '@/api/generated/RetirementScenarioView';
import type { RetirementSimulation } from '@/api/generated/RetirementSimulation';
import type { UsSsaInput } from '@/api/generated/UsSsaInput';

import { mockFail, onMockReset } from '../mockRegistry';

import { withEducation } from './education';
import { optNum, register, str } from './ledger/common';
import { memberAge, retirementHorizon } from './longevity';
import {
  emergencyReserve,
  goalCreate,
  goalDelete,
  goalGet,
  goalsList,
  goalUpdate,
} from './ledger/goalsState';

let savedPlan: RetirementPlan = {};
let scenarios: RetirementScenario[] = [];
let linkedGoal: string | null = null;
let scenarioSeq = 0;
onMockReset(() => {
  savedPlan = {};
  scenarios = [];
  linkedGoal = null;
});

const MIN_WAGE = 162_100;
const CEILING = 847_555;

/** INSS simplificado (o núcleo aplica todas as regras da reforma): transição por idade. */
function inssEstimate(i: InssInput, age: number, retire: number): InssEstimate {
  const male = i.sex === 'male';
  const minAge = male ? 65 : 62;
  const start = Math.max(retire, minAge);
  const until = i.contribute_until_age ?? retire;
  const contrib = i.contribution_years + Math.max(0, Math.min(start, until) - age);
  const coef = 0.6 + 0.02 * Math.max(0, Math.floor(contrib - (male ? 20 : 15)));
  const avg = Math.min(i.average_salary, CEILING);
  const monthly = Math.min(CEILING, Math.max(MIN_WAGE, Math.round(avg * coef)));
  const ok = contrib >= 15;
  return {
    system: 'br_inss',
    payments_per_year: 13,
    rules: [
      {
        id: 'age',
        name: 'Transição por idade',
        requirement: `${minAge} anos de idade e 15 de contribuição`,
        eligible_date: ok ? `${new Date().getFullYear() + (minAge - age)}-01-01` : null,
        eligible_age: ok ? minAge : null,
        monthly_at_eligibility: ok ? monthly : null,
        start_age: ok ? start : null,
        monthly_at_start: ok ? monthly : null,
        coefficient_at_start: ok ? Math.round(coef * 1000) / 1000 : null,
      },
    ],
    chosen: ok ? 'age' : null,
    monthly: ok ? monthly : null,
    start_age: ok ? start : null,
    average_used: avg,
    table_year: 2026,
    min_wage: MIN_WAGE,
    ceiling: CEILING,
    source: 'Portaria Interministerial MPS/MF nº 13, de 09/01/2026',
    notes: ['Estimativa de exemplo.'],
  };
}

const yearOf = (birth: string | undefined, age: number) =>
  birth ? Number(birth.slice(0, 4)) : new Date().getFullYear() - age;

/** Uma regra de pedido numa idade (meses), como em `pension_us.rs`. */
function rule(
  id: string,
  name: string,
  requirement: string,
  months: number,
  birthYear: number,
  monthly: number,
): InssRule {
  const startAge = Math.ceil(months / 12);
  return {
    id,
    name,
    requirement,
    eligible_date: `${birthYear + Math.floor(months / 12)}-06-15`,
    eligible_age: Math.round((months / 12) * 10) / 10,
    monthly_at_eligibility: monthly,
    start_age: startAge,
    monthly_at_start: monthly,
    coefficient_at_start: null,
  };
}

/** Social Security simplificada: PIA (ou AIME pelos bend points de 2027) e o fator do pedido. */
function usSsaEstimate(i: UsSsaInput, age: number, retire: number): InssEstimate {
  const by = yearOf(i.birth_date, age);
  const fra = by >= 1960 ? 804 : by >= 1955 ? 792 + 2 * (by - 1954) : 792;
  const a = i.aime ?? 0;
  const pia =
    i.pia ??
    Math.floor(
      (0.9 * Math.min(a, 134_000) +
        0.32 * Math.max(0, Math.min(a, 807_700) - 134_000) +
        0.15 * Math.max(0, a - 807_700)) /
        10,
    ) * 10;
  const factor = (m: number) => {
    const d = m - fra;
    return d >= 0
      ? 1 + (d * 2) / 300
      : 1 - (Math.min(-d, 36) * 5) / 900 - (Math.max(-d - 36, 0) * 5) / 1200;
  };
  const claim = Math.min(70, Math.max(62, retire)) * 12;
  const rules = [
    rule(
      'early',
      'Pedido antecipado (62 anos)',
      'a partir dos 62 anos',
      744,
      by,
      Math.floor(pia * factor(744)),
    ),
    rule('full', 'Idade plena (FRA)', 'idade plena', fra, by, pia),
    rule('delayed', 'Pedido adiado (70 anos)', '70 anos', 840, by, Math.floor(pia * factor(840))),
  ];
  if (![744, fra, 840].includes(claim))
    rules.push(
      rule(
        'plan_age',
        `Pedido aos ${claim / 12} anos`,
        'idade do plano',
        claim,
        by,
        Math.floor(pia * factor(claim)),
      ),
    );
  const chosen = rules.find((r) => r.start_age === claim / 12) ?? rules[0]!;
  return {
    system: 'us_ssa',
    payments_per_year: 12,
    rules,
    chosen: chosen.id,
    monthly: chosen.monthly_at_start,
    start_age: chosen.start_age,
    average_used: a,
    table_year: 2027,
    min_wage: 0,
    ceiling: 0,
    source: 'SSA, PIA bend points 2027 (14/10/2026)',
    notes: ['Estimativa de exemplo.'],
  };
}

/** Pensão de velhice simplificada: RR × taxa global na idade normal de 2027 (66a11m). */
function ptPensionEstimate(i: PtPensionInput, age: number, retire: number): InssEstimate {
  const by = yearOf(i.birth_date, age);
  const start = Math.max(retire, 67);
  const until = i.contribute_until_age ?? retire;
  const years = Math.min(
    40,
    Math.floor(i.contribution_years + Math.max(0, Math.min(start, until) - age)),
  );
  const ias = 53_713;
  const rr = i.reference_salary;
  let base = rr * 0.02 * years;
  if (years > 20) {
    base = 0;
    let low = 0;
    for (const [up, rate] of [
      [1.1, 0.023],
      [2, 0.0225],
      [4, 0.022],
      [8, 0.021],
    ] as const) {
      base += Math.max(0, Math.min(rr, up * ias) - low) * rate * years;
      low = up * ias;
    }
    base += Math.max(0, rr - low) * 0.02 * years;
  }
  const min = years >= 31 ? 49_352 : years >= 21 ? 39_482 : 35_780;
  const ok = years >= 15;
  const monthly = Math.max(Math.round(base), min);
  const r = rule('pt_normal', 'Idade pessoal de reforma', '66 anos e 11 meses', 803, by, monthly);
  return {
    system: 'pt_ss',
    payments_per_year: 14,
    rules: ok ? [{ ...r, start_age: start, monthly_at_start: monthly }] : [],
    chosen: ok ? 'pt_normal' : null,
    monthly: ok ? monthly : null,
    start_age: ok ? start : null,
    average_used: rr,
    table_year: 2026,
    min_wage: 35_780,
    ceiling: 0,
    source: 'ISS, Guia Prático Pensão de Velhice (2026); Portaria 476/2025/1',
    notes: ['Estimativa de exemplo.'],
  };
}

const clean = (a: Record<string, unknown>): RetirementPlan =>
  Object.fromEntries(Object.entries(a).filter(([, v]) => v != null));

/** Simulação determinística simples (o núcleo faz o Monte Carlo): faixas a ±1,28 σ. */
function retirement(plan: RetirementPlan): RetirementSimulation {
  if (plan.current_age == null) mockFail('invalid', 'informe a idade de hoje');
  const age = plan.current_age;
  const retire = plan.retirement_age ?? Math.max(65, age);
  const inssCalc =
    plan.inss_monthly != null
      ? null
      : plan.inss_calc
        ? inssEstimate(plan.inss_calc, age, retire)
        : plan.us_ssa_calc
          ? usSsaEstimate(plan.us_ssa_calc, age, retire)
          : plan.pt_pension_calc
            ? ptPensionEstimate(plan.pt_pension_calc, age, retire)
            : null;
  const p: RetirementPlan = inssCalc
    ? {
        ...plan,
        // Todos os pagamentos do ano (13º do INSS, 14 em Portugal) como renda mensal.
        inss_monthly: Math.floor(((inssCalc.monthly ?? 0) * inssCalc.payments_per_year) / 12),
        inss_age: plan.inss_age ?? inssCalc.start_age ?? undefined,
      }
    : plan;
  // Como no núcleo, o horizonte padrão segue 95; a sugestão "1 em 10" (seção 7) vem de
  // `goals.retirement_defaults`, e a interface a manda como `life_expectancy`.
  const horizon = retirementHorizon();
  const end = p.life_expectancy ?? 95;
  const ret = p.real_return ?? 0.04;
  const vol = p.volatility ?? 0.1;
  const wealth0 = p.current_wealth ?? 27_413_498;
  const contrib = (p.monthly_contribution ?? 0) * 12;
  const spend = (p.monthly_spending ?? 0) * 12;
  const inss = (p.inss_monthly ?? 0) * 12;
  const inssAge = p.inss_age ?? retire;
  const path = (r: number, spending: number) => {
    let w = wealth0;
    const out = [w];
    for (let a = age; a < end; a++) {
      let flow = a < retire ? contrib : -spending;
      if (a >= inssAge) flow += inss;
      for (const e of p.events ?? []) if (e.age === a) flow += e.amount;
      w = w > 0 ? w * (1 + r) + flow : w + flow;
      out.push(w);
    }
    return out;
  };
  const mid = path(ret, spend);
  const low = path(ret - 1.28 * vol * 0.4, spend);
  const high = path(ret + 1.28 * vol * 0.4, spend);
  const okLow = low.every((w) => w >= 0);
  const okMid = mid.every((w) => w >= 0);
  const success = okLow ? 0.93 : okMid ? 0.7 : 0.3;
  let lo = 0;
  let hi = wealth0 + contrib * 60 + inss * 60;
  for (let i = 0; i < 30; i++) {
    const m = (lo + hi) / 2;
    if (path(ret - 1.28 * vol * 0.4, m).every((w) => w >= 0)) lo = m;
    else hi = m;
  }
  const depletion = low.findIndex((w) => w < 0);
  const yearsAfter = end - retire;
  const required = Math.max(0, (spend - inss) * yearsAfter * 0.8);
  return {
    plan: {
      ...p,
      retirement_age: retire,
      life_expectancy: end,
      inss_age: inssAge,
      ...(inssCalc ? { inss_monthly: undefined, inss_age: plan.inss_age } : {}),
    },
    required_wealth: Math.round(required),
    inss: inssCalc,
    fgts_at_retirement: null,
    success,
    simulations: 2000,
    start_wealth: wealth0,
    wealth_at_retirement: Math.round(Math.max(0, mid[retire - age] ?? 0)),
    sustainable_spending: Math.floor(lo / 12),
    depletion_age_p10: depletion >= 0 ? age + depletion : null,
    bands: mid.map((w, i) => ({
      age: age + i,
      p10: Math.round(Math.max(0, low[i]!)),
      p50: Math.round(Math.max(0, w)),
      p90: Math.round(Math.max(0, high[i]!)),
    })),
    notes: ['Valores reais de hoje (simulação de exemplo).'],
    longevity: horizon
      ? {
          plan_age: end,
          survival: horizon.survivalPast(end),
          suggested_age: horizon.suggested,
        }
      : null,
  };
}

const withLink = (g: GoalView): GoalView =>
  g.id === linkedGoal && !g.archived ? { ...g, linked_to_plan: true } : g;

function goalFields(sim: RetirementSimulation) {
  const age = sim.plan.current_age ?? 0;
  const retire = sim.plan.retirement_age ?? age;
  const d = new Date();
  d.setFullYear(d.getFullYear() + (retire - age));
  return {
    target_amount: Math.max(1, sim.required_wealth),
    target_date: d.toISOString().slice(0, 10),
    monthly_contribution: sim.plan.monthly_contribution ?? null,
  };
}

function scenarioView(s: RetirementScenario): RetirementScenarioView {
  try {
    const r = retirement(s.plan);
    return {
      ...s,
      success: r.success,
      sustainable_spending: r.sustainable_spending,
      required_wealth: r.required_wealth,
      wealth_at_retirement: r.wealth_at_retirement,
      error: null,
    };
  } catch (e) {
    return {
      ...s,
      success: null,
      sustainable_spending: null,
      required_wealth: null,
      wealth_at_retirement: null,
      error: String(e),
    };
  }
}

register({
  'goals.list': (a) =>
    goalsList(a.include_archived === true)
      .map(withLink)
      .map(withEducation),
  'goals.get': (a) => withEducation(withLink(goalGet(str(a, 'id')))),
  'goals.inss_estimate': (a) => {
    const age = (a.current_age as number | undefined) ?? savedPlan.current_age ?? 40;
    const retire = (a.retirement_age as number | undefined) ?? savedPlan.retirement_age ?? 65;
    return inssEstimate(a.input as InssInput, age, retire);
  },
  'goals.retirement_scenarios': () => scenarios.map(scenarioView),
  'goals.retirement_scenario_save': (a) => {
    const name = str(a, 'name').trim();
    if (!name) mockFail('invalid', 'dê um nome ao cenário');
    const plan = clean(a.plan as Record<string, unknown>);
    retirement(plan);
    const id = a.id as string | undefined;
    if (scenarios.some((s) => s.name.toLowerCase() === name.toLowerCase() && s.id !== id))
      mockFail('conflict', `já existe um cenário “${name}”`);
    const now = new Date().toISOString();
    if (id) {
      const s = scenarios.find((x) => x.id === id) ?? mockFail('not_found', `cenário ${id}`);
      Object.assign(s, { name, plan, updated_at: now });
      return s;
    }
    const s = { id: `scn-${++scenarioSeq}`, name, plan, updated_at: now };
    scenarios.push(s);
    return s;
  },
  'goals.retirement_scenario_delete': (a) => {
    scenarios = scenarios.filter((s) => s.id !== str(a, 'id'));
    return null;
  },
  'goals.retirement_link_goal': (a) => {
    if (savedPlan.current_age == null)
      mockFail('invalid', 'salve o plano de aposentadoria antes de ligar uma meta');
    const f = goalFields(retirement(savedPlan));
    const id =
      (a.goal_id as string | undefined) ??
      goalCreate({
        name: 'Aposentadoria',
        kind: 'retirement',
        ...f,
        monthly_contribution: f.monthly_contribution ?? undefined,
      }).id;
    goalUpdate({ id, ...f });
    linkedGoal = id;
    return withLink(goalGet(id));
  },
  'goals.retirement_unlink_goal': () => {
    linkedGoal = null;
    return null;
  },
  'goals.create': (a) => goalCreate(a as unknown as GoalInput),
  'goals.update': (a) => goalUpdate({ ...(a as unknown as GoalPatch), id: str(a, 'id') }),
  'goals.delete': (a) => {
    goalDelete(str(a, 'id'));
    return null;
  },
  'goals.retirement': (a) => retirement({ ...savedPlan, ...clean(a) }),
  'goals.retirement_save': (a) => {
    const plan = clean(a);
    const sim = retirement(plan);
    savedPlan = plan;
    if (linkedGoal) goalUpdate({ id: linkedGoal, ...goalFields(sim) });
    return sim;
  },
  'goals.retirement_defaults': (a) => {
    const member = typeof a.member_id === 'string' && a.member_id ? a.member_id : null;
    const horizon = retirementHorizon(member);
    return {
      current_age: memberAge(member),
      life_expectancy: horizon?.suggested ?? 95,
      from_longevity: horizon != null,
    };
  },
  'goals.emergency_reserve': (a) =>
    emergencyReserve(
      optNum(a, 'target_months') ?? 6,
      optNum(a, 'months_back') ?? 6,
      typeof a.include_annual === 'boolean' ? a.include_annual : null,
    ),
});
