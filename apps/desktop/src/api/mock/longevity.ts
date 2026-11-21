/**
 * Mock de `longevity.*` (docs/life-expectancy.md): a linha da vida de cada membro a partir de
 * uma tábua sintética de Gompertz–Makeham que melhora com o ano (só para desenvolvimento e
 * testes; o núcleo usa as tábuas da ONU, WPP 2024). Mesmas formas do núcleo: expectativa por
 * geração, condicionada à idade de hoje, com a mediana, o "1 em 4" e o "1 em 10", a chance de
 * pelo menos um adulto do casal estar vivo e os marcos financeiros do titular.
 */
import type { FamilyOutlook } from '@/api/generated/FamilyOutlook';
import type { GoalView } from '@/api/generated/GoalView';
import type { LifeMilestone } from '@/api/generated/LifeMilestone';
import type { LifeStage } from '@/api/generated/LifeStage';
import type { LoanView } from '@/api/generated/LoanView';
import type { LongevitySettings } from '@/api/generated/LongevitySettings';
import type { LongevitySource } from '@/api/generated/LongevitySource';
import type { MemberOutlook } from '@/api/generated/MemberOutlook';
import type { MemberSex } from '@/api/generated/MemberSex';
import type { MissingLifeData } from '@/api/generated/MissingLifeData';
import type { RetirementSimulation } from '@/api/generated/RetirementSimulation';
import type { StageInfo } from '@/api/generated/StageInfo';

import { extraMockHandler, onMockReset, registerMockHandlers } from '../mockRegistry';
import { familyCountry } from './country';
import { guarded } from './ledger/boot';
import { today } from './ledger/dates';
import { invalid } from './ledger/errors';
import { type MemberRow, rows } from './ledger/store';

// ── Ajustes ─────────────────────────────────────────────────────

const DEFAULTS: LongevitySettings = {
  enabled: false,
  hidden_members: [],
  show_children_end: false,
  show_advanced_later_life: false,
};

let settings: LongevitySettings = { ...DEFAULTS };
onMockReset(() => {
  settings = { ...DEFAULTS, hidden_members: [] };
});

export function longevitySettings(): LongevitySettings {
  return settings;
}

/** O questionário da família liga a função ("Sim, quero ver"). */
export function enableLongevity(): void {
  settings = { ...settings, enabled: true };
}

// ── Fases ───────────────────────────────────────────────────────

const STAGES: StageInfo[] = [
  { stage: 'childhood', from: 0, to: 12 },
  { stage: 'adolescence', from: 12, to: 18 },
  { stage: 'youth', from: 18, to: 30 },
  { stage: 'adulthood', from: 30, to: 60 },
  { stage: 'later_life', from: 60, to: null },
];

function stageAt(age: number): LifeStage {
  return STAGES.find((s) => age >= s.from && (s.to == null || age < s.to))!.stage;
}

// ── Tábua sintética ─────────────────────────────────────────────

type Table = 'male' | 'female' | 'both';

/** Gompertz–Makeham: μ(x) = A + B·e^(c·x), ajustada para o Brasil de 2024 (IBGE: 73,3, 79,9 e
 * 76,6 anos ao nascer), com melhora de 1,1% ao ano depois de 2024. */
const GM: Record<Table, { a: number; b: number; c: number }> = {
  male: { a: 0.0011, b: 1.491e-5, c: 0.108 },
  female: { a: 0.0004, b: 6.663e-6, c: 0.112 },
  both: { a: 0.00075, b: 9.935e-6, c: 0.11 },
};
const IMPROVEMENT = 0.011;
const MAX_AGE = 110;

function qx(table: Table, age: number, year: number): number {
  const { a, b, c } = GM[table];
  const y = Math.min(year, 2100);
  const imp = Math.exp(-IMPROVEMENT * (y - 2024));
  const mu = a * imp + b * imp * Math.exp(c * age);
  const base = 1 - Math.exp(-mu);
  const infant = age === 0 ? 0.012 * Math.exp(-0.03 * (y - 2024)) : 0;
  return Math.min(1, base + infant);
}

/** Sobrevivência pela diagonal da geração, da idade `from` até 110. */
function cohortCurve(table: Table, birthYear: number, from: number): number[] {
  const out = [1];
  let s = 1;
  for (let x = from; x < MAX_AGE; x++) {
    s *= 1 - qx(table, x, birthYear + x);
    out.push(s);
  }
  return out;
}

/** Idade em que a curva cai até `p` (interpolada entre as idades inteiras). */
function crossing(curve: number[], from: number, p: number): number {
  for (let i = 1; i < curve.length; i++) {
    const s0 = curve[i - 1]!;
    const s1 = curve[i]!;
    if (s1 <= p) return from + i - 1 + (s0 - p) / (s0 - s1);
  }
  return from + curve.length - 1;
}

const round1 = (n: number) => Math.round(n * 10) / 10;

function periodRemaining(table: Table, age: number): number {
  let s = 1;
  let total = 0;
  for (let x = age; x < 130; x++) {
    s *= 1 - qx(table, x, 2024);
    total += s;
  }
  return total + 0.5;
}

/** Tábua do IBGE de 2024 (período), referência só no Brasil. */
const IBGE_E0: Record<Table, number> = { male: 73.3, female: 79.9, both: 76.6 };

function tableOf(sex: MemberSex | null): Table {
  return sex === 'male' || sex === 'female' ? sex : 'both';
}

// ── Membros ─────────────────────────────────────────────────────

const parseYmd = (iso: string) => iso.split('-').map(Number) as [number, number, number];

/** Idade completa hoje. */
export function ageOn(birth: string, on: string = today()): number {
  const [by, bm, bd] = parseYmd(birth);
  const [y, m, d] = parseYmd(on);
  return y - by - (m < bm || (m === bm && d < bd) ? 1 : 0);
}

function activeMembers(): MemberRow[] {
  return rows('member')
    .filter((m) => !m.archived)
    .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name, 'pt-BR'));
}

/** Curva e idades de um membro com data de nascimento (sem os marcos). */
export function survivalOf(m: MemberRow) {
  const birth = m.profile?.birth_date;
  if (!birth) return null;
  const sex = m.profile?.sex ?? null;
  const table = tableOf(sex);
  const age = Math.max(0, ageOn(birth));
  const birthYear = parseYmd(birth)[0];
  const curve = cohortCurve(table, birthYear, age);
  return { age, birthYear, sex, table, curve };
}

/** Chance de passar de uma idade, pela curva de quem tem `age` hoje. */
export function survivalAt(curve: number[], age: number, at: number): number {
  const i = Math.max(0, Math.min(curve.length - 1, Math.round(at - age)));
  return curve[i]!;
}

const handler = <T>(cmd: string, args: Record<string, unknown> = {}): T | null => {
  const h = extraMockHandler(cmd);
  if (!h) return null;
  try {
    return h(args) as T;
  } catch {
    return null;
  }
};

function milestones(m: MemberRow, age: number): LifeMilestone[] {
  const year = Number(today().slice(0, 4));
  const at = (a: number) => year + (a - age);
  const out: LifeMilestone[] = [];
  if (m.is_primary) {
    const sim = handler<RetirementSimulation>('goals.retirement', {});
    const p = sim?.plan;
    if (sim && p?.current_age != null && p.retirement_age != null) {
      out.push({
        kind: 'retirement',
        age: p.retirement_age,
        year: at(p.retirement_age),
        label: 'Aposentadoria',
        amount: null,
      });
      if (
        p.inss_age != null &&
        (p.inss_monthly || p.inss_calc || p.us_ssa_calc || p.pt_pension_calc)
      ) {
        out.push({
          kind: 'inss',
          age: p.inss_age,
          year: at(p.inss_age),
          label: 'Início do INSS',
          amount: p.inss_monthly ?? sim.inss?.monthly ?? null,
        });
      }
      const end = sim.depletion_age_p10 ?? p.life_expectancy ?? 95;
      out.push({
        kind: 'plan_end',
        age: end,
        year: at(end),
        label: sim.depletion_age_p10 != null ? 'O dinheiro acaba no cenário ruim' : 'Plano vai até',
        amount: null,
      });
      for (const e of p.events ?? []) {
        out.push({
          kind: 'life_event',
          age: e.age,
          year: at(e.age),
          label: e.label,
          amount: e.amount,
        });
      }
    }
    for (const g of handler<GoalView[]>('goals.list', {}) ?? []) {
      if (g.archived || !g.target_date || g.kind === 'retirement') continue;
      const a = age + (Number(g.target_date.slice(0, 4)) - year);
      out.push({ kind: 'goal', age: a, year: at(a), label: g.name, amount: g.target_amount });
    }
  }
  for (const l of handler<LoanView[]>('debt.list', {}) ?? []) {
    if (l.archived || l.member_id !== m.id || l.remaining_count <= 0) continue;
    const a = age + Math.round(l.remaining_count / 12);
    out.push({ kind: 'debt_end', age: a, year: at(a), label: l.name, amount: null });
  }
  return out.sort((a, b) => a.age - b.age);
}

function outlook(m: MemberRow): MemberOutlook {
  const s = survivalOf(m);
  if (!s) invalid(`${m.name} não tem data de nascimento`);
  const { age, birthYear, sex, table, curve } = s;
  const child = age < 18;
  const mean = age + curve.slice(1).reduce((a, b) => a + b, 0) + 0.5;
  const e0 =
    birthYear >= 1950
      ? cohortCurve(table, birthYear, 0)
          .slice(1)
          .reduce((a, b) => a + b, 0)
      : null;
  return {
    member_id: m.id,
    name: m.name,
    age,
    birth_year: birthYear,
    sex,
    stage: stageAt(age),
    median_age: round1(crossing(curve, age, 0.5)),
    p25_age: round1(crossing(curve, age, 0.25)),
    p10_age: round1(crossing(curve, age, 0.1)),
    mean_age: round1(mean),
    remaining_mean: round1(mean - age),
    cohort_e0: e0 == null ? null : round1(e0 + 0.5),
    period_ref:
      familyCountry() === 'BR'
        ? {
            source: 'IBGE 2024',
            e0: IBGE_E0[table],
            remaining_at_age: round1(periodRemaining(table, age)),
          }
        : null,
    curve: curve.map((v, i) => ({ age: age + i, survival: Math.round(v * 10000) / 10000 })),
    milestones: milestones(m, age),
    child,
    show_end: !child || settings.show_children_end,
  };
}

function source(country: string): LongevitySource {
  const br = country === 'BR';
  return {
    name: br ? 'ONU, WPP 2024 e IBGE 2024' : 'ONU, WPP 2024',
    attribution:
      'United Nations, Department of Economic and Social Affairs, Population Division (2024). ' +
      'World Population Prospects 2024, Online Edition. Dados recortados e transformados pelo ' +
      'Treasury; a ONU não apoia este app.' +
      (br ? ' IBGE, Tábuas Completas de Mortalidade 2024.' : ''),
    synthetic: false,
  };
}

/** Pelo menos um dos dois adultos (titular e cônjuge) vivo, por ano civil. */
function couple(members: MemberRow[]) {
  const adults = members
    .filter((m) => m.is_primary || m.profile?.relationship === 'conjuge')
    .map(survivalOf)
    .filter((s): s is NonNullable<ReturnType<typeof survivalOf>> => s != null && s.age >= 18);
  if (adults.length < 2) return { points: null, p25: null, p10: null };
  const year = Number(today().slice(0, 4));
  const points: { year: number; probability: number }[] = [];
  let p25: number | null = null;
  let p10: number | null = null;
  for (let y = year; y <= year + 80; y++) {
    const none = adults.reduce((acc, a) => {
      const i = y - year;
      return acc * (1 - (a.curve[i] ?? 0));
    }, 1);
    const p = 1 - none;
    points.push({ year: y, probability: Math.round(p * 10000) / 10000 });
    if (p25 == null && p <= 0.25) p25 = y;
    if (p10 == null && p <= 0.1) p10 = y;
    if (p < 0.005) break;
  }
  return { points, p25, p10 };
}

function family(): FamilyOutlook {
  const country = familyCountry();
  if (!settings.enabled) {
    return {
      enabled: false,
      members: [],
      missing: [],
      any_adult_alive: null,
      any_alive_p25_year: null,
      any_alive_p10_year: null,
      country,
      source: source(country),
    };
  }
  const all = activeMembers();
  // Como no núcleo, quem escondeu a barra fica fora da lista e do pedido do que falta.
  const missing: MissingLifeData[] = all
    .filter((m) => !settings.hidden_members.includes(m.id))
    .map((m) => ({
      member_id: m.id,
      name: m.name,
      missing: [
        ...(m.profile?.birth_date ? [] : (['birth_date'] as const)),
        ...(m.profile?.sex ? [] : (['sex'] as const)),
      ],
    }))
    .filter((x) => x.missing.length > 0);
  const c = couple(all);
  return {
    enabled: true,
    members: all
      .filter((m) => m.profile?.birth_date && !settings.hidden_members.includes(m.id))
      .map(outlook),
    missing,
    any_adult_alive: c.points,
    any_alive_p25_year: c.p25,
    any_alive_p10_year: c.p10,
    country,
    source: source(country),
  };
}

registerMockHandlers(
  guarded({
    'longevity.settings': () => settings,
    'longevity.set_settings': (args) => {
      const next = { ...settings, ...(args as Partial<LongevitySettings>) };
      if (
        !Array.isArray(next.hidden_members) ||
        next.hidden_members.some((x) => typeof x !== 'string')
      )
        invalid('hidden_members: lista de ids de membro');
      settings = {
        enabled: Boolean(next.enabled),
        hidden_members: [...new Set(next.hidden_members)],
        show_children_end: Boolean(next.show_children_end),
        show_advanced_later_life: Boolean(next.show_advanced_later_life),
      };
      return settings;
    },
    'longevity.stages': () => STAGES,
    'longevity.family': () => family(),
    'longevity.outlook': (args) => {
      const id = typeof args.member_id === 'string' ? args.member_id : '';
      const m = rows('member').find((x) => x.id === id);
      if (!m) invalid(`membro ${id} não encontrado`);
      return outlook(m);
    },
  }),
);

// ── Aposentadoria ───────────────────────────────────────────────

/**
 * Horizonte do plano de aposentadoria pela linha da vida (seção 7): a idade "1 em 10" do
 * titular ou, com cônjuge, a idade do titular no ano em que a chance de pelo menos um dos
 * dois estar vivo cai a 10%. `null` com a função desligada ou sem data de nascimento.
 */
export function retirementHorizon(memberId?: string | null) {
  if (!settings.enabled) return null;
  const all = activeMembers();
  const m = (memberId ? all.find((x) => x.id === memberId) : null) ?? all.find((x) => x.is_primary);
  const own = m ? survivalOf(m) : null;
  if (!m || !own) return null;
  const spouse = m.is_primary
    ? all.find((x) => x.id !== m.id && x.profile?.relationship === 'conjuge')
    : null;
  const other = spouse ? survivalOf(spouse) : null;
  // Chance de pelo menos um vivo k anos depois de hoje.
  const alive = (k: number) => {
    const a = own.curve[k] ?? 0;
    if (!other || other.age < 18) return a;
    return 1 - (1 - a) * (1 - (other.curve[k] ?? 0));
  };
  let k = 0;
  while (k < own.curve.length - 1 && alive(k + 1) > 0.1) k++;
  // Interpolação entre k e k + 1.
  const s0 = alive(k);
  const s1 = alive(k + 1);
  const p10 = own.age + k + (s0 > s1 ? (s0 - 0.1) / (s0 - s1) : 0);
  return {
    age: own.age,
    suggested: Math.ceil(p10),
    /** Chance de passar da idade `planAge` (do titular). */
    survivalPast: (planAge: number) =>
      Math.round(alive(Math.max(0, Math.round(planAge - own.age))) * 1000) / 1000,
  };
}

/** Idade de hoje do membro (ou do titular) pela data de nascimento. */
export function memberAge(memberId?: string | null): number | null {
  const all = activeMembers();
  const m = (memberId ? all.find((x) => x.id === memberId) : null) ?? all.find((x) => x.is_primary);
  return m?.profile?.birth_date ? ageOn(m.profile.birth_date) : null;
}
