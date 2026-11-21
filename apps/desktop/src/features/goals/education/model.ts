/**
 * Meta de faculdade (funções puras; docs/design/college-goal.md): o formulário do assistente
 * ↔ o plano do núcleo, a validação e os rótulos.
 */
import type { ContributionMode } from '@/api/generated/ContributionMode';
import type { EducationPlan } from '@/api/generated/EducationPlan';
import type { EducationPlanView } from '@/api/generated/EducationPlanView';
import type { InflationMode } from '@/api/generated/InflationMode';
import type { InstitutionType } from '@/api/generated/InstitutionType';
import type { Member } from '@/api/generated/Member';
import { t } from '@/i18n';

export interface EducationForm {
  /** Filho cadastrado ou que ainda vai nascer. */
  childMode: 'member' | 'unborn';
  memberId: string | null;
  expectedBirth: string | null;
  startAge: number;
  startMonth: number;
  institution: InstitutionType;
  course: string;
  /** Duração em anos (aceita meio ano). */
  durationYears: number;
  paymentsPerYear: number;
  fee: number | null;
  yearlyExtras: number | null;
  monthlyLiving: number | null;
  discountPct: number;
  inflationMode: InflationMode;
  /** Diferença fixa, em % ao ano. */
  fixedSpreadPct: number | null;
  /** Retorno real suposto, em % ao ano. */
  realReturnPct: number;
  contributionMode: ContributionMode;
  initial: number | null;
  name: string;
}

export type EducationFormErrors = Partial<Record<keyof EducationForm, string>>;

export const PAYMENTS_PER_YEAR = [12, 4, 2, 1] as const;

/** Filhos primeiro; os outros membros também podem ter uma meta. */
export function childCandidates(members: readonly Member[]): Member[] {
  return members
    .filter((m) => !m.archived)
    .sort(
      (a, b) =>
        Number(b.profile.relationship === 'filho') - Number(a.profile.relationship === 'filho'),
    );
}

export function emptyEducationForm(memberId: string | null = null): EducationForm {
  return {
    childMode: 'member',
    memberId,
    expectedBirth: null,
    startAge: 18,
    startMonth: 2,
    institution: 'private',
    course: '',
    durationYears: 4,
    paymentsPerYear: 12,
    fee: null,
    yearlyExtras: null,
    monthlyLiving: null,
    discountPct: 0,
    inflationMode: 'series_10y',
    fixedSpreadPct: null,
    realReturnPct: 3,
    contributionMode: 'indexed',
    initial: null,
    name: '',
  };
}

export function formFromPlan(v: EducationPlanView): EducationForm {
  const p = v.plan;
  return {
    childMode: p.member_id ? 'member' : 'unborn',
    memberId: p.member_id,
    expectedBirth: p.expected_birth,
    startAge: p.start_age,
    startMonth: p.start_month,
    institution: p.institution_type,
    course: p.course ?? '',
    durationYears: p.duration_months / 12,
    paymentsPerYear: p.payments_per_year,
    fee: p.fee,
    yearlyExtras: p.yearly_extras,
    monthlyLiving: p.monthly_living,
    discountPct: p.discount_pct,
    inflationMode: p.inflation_mode,
    fixedSpreadPct: p.fixed_spread == null ? null : round(p.fixed_spread * 100, 2),
    realReturnPct: round(p.real_return * 100, 2),
    contributionMode: p.contribution_mode,
    initial: v.goal.initial_amount,
    name: v.goal.name,
  };
}

const round = (x: number, d: number) => Math.round(x * 10 ** d) / 10 ** d;

/** O plano do núcleo a partir do formulário; `feeDate` é a data da cotação da mensalidade. */
export function planFromForm(
  f: EducationForm,
  feeDate: string,
  base?: EducationPlan,
): EducationPlan {
  return {
    member_id: f.childMode === 'member' ? f.memberId : null,
    expected_birth: f.childMode === 'unborn' ? f.expectedBirth : null,
    start_age: f.startAge,
    start_month: f.startMonth,
    start_date: base?.start_date ?? null,
    institution_type: f.institution,
    course: f.course.trim() || null,
    duration_months: Math.max(1, Math.round(f.durationYears * 12)),
    payments_per_year: f.paymentsPerYear,
    fee: f.fee ?? 0,
    fee_date: feeDate,
    yearly_extras: f.yearlyExtras ?? 0,
    monthly_living: f.monthlyLiving ?? 0,
    discount_pct: f.discountPct,
    inflation_mode: f.inflationMode,
    inflation_series: base?.inflation_series ?? null,
    fixed_spread:
      f.inflationMode === 'fixed_spread' && f.fixedSpreadPct != null
        ? f.fixedSpreadPct / 100
        : null,
    real_return: f.realReturnPct / 100,
    contribution_mode: f.contributionMode,
    pool_rule: base?.pool_rule ?? 'earliest_first',
  };
}

/** A data da cotação muda só quando a mensalidade muda (é o preço de hoje). */
export function feeDateFor(f: EducationForm, prev: EducationPlan | null, today: string): string {
  if (prev && prev.fee === (f.fee ?? 0)) return prev.fee_date;
  return today;
}

/** Erros que impedem salvar (o núcleo confere de novo). */
export function validateEducation(
  f: EducationForm,
  members: readonly Member[],
): EducationFormErrors {
  const e: EducationFormErrors = {};
  if (f.childMode === 'member') {
    const m = members.find((x) => x.id === f.memberId);
    if (!m) e.memberId = t('educacao-erro-filho');
    else if (!m.profile.birth_date) e.memberId = t('educacao-erro-nascimento');
  } else if (!f.expectedBirth) {
    e.expectedBirth = t('educacao-erro-nascimento-previsto');
  }
  if (!(f.startAge >= 0 && f.startAge <= 60)) e.startAge = t('educacao-erro-idade');
  if (!(f.durationYears > 0 && f.durationYears <= 15)) e.durationYears = t('educacao-erro-duracao');
  if (f.institution !== 'public' && !(f.fee != null && f.fee > 0))
    e.fee = t('educacao-erro-mensalidade');
  if (!(f.discountPct >= 0 && f.discountPct <= 100)) e.discountPct = t('educacao-erro-desconto');
  if (!(f.realReturnPct >= -5 && f.realReturnPct <= 15))
    e.realReturnPct = t('educacao-erro-retorno');
  if (
    f.inflationMode === 'fixed_spread' &&
    !(f.fixedSpreadPct != null && f.fixedSpreadPct >= -10 && f.fixedSpreadPct <= 20)
  )
    e.fixedSpreadPct = t('educacao-erro-diferenca');
  return e;
}

/** O passo do assistente em que está cada campo (para voltar ao passo do primeiro erro). */
export const STEP_OF: Record<keyof EducationForm, 0 | 1 | 2> = {
  childMode: 0,
  memberId: 0,
  expectedBirth: 0,
  startAge: 0,
  startMonth: 0,
  name: 0,
  institution: 1,
  course: 1,
  durationYears: 1,
  paymentsPerYear: 1,
  fee: 1,
  yearlyExtras: 1,
  monthlyLiving: 1,
  discountPct: 1,
  inflationMode: 2,
  fixedSpreadPct: 2,
  realReturnPct: 2,
  contributionMode: 2,
  initial: 2,
};

export function firstErrorStep(e: EducationFormErrors): 0 | 1 | 2 | null {
  const steps = (Object.keys(e) as (keyof EducationForm)[]).map((k) => STEP_OF[k]);
  return steps.length ? (Math.min(...steps) as 0 | 1 | 2) : null;
}

export function institutionLabel(x: InstitutionType): string {
  return x === 'public'
    ? t('educacao-tipo-publica')
    : x === 'abroad'
      ? t('educacao-tipo-exterior')
      : t('educacao-tipo-particular');
}

/** Texto do número digitado (vírgula ou ponto) → número; vazio ou inválido → `null`. */
export function parseNumber(s: string): number | null {
  const n = Number(s.trim().replace(/\s/g, '').replace(',', '.'));
  return s.trim() === '' || !Number.isFinite(n) ? null : n;
}

/** Idade (anos completos) numa data. */
export function ageOn(birth: string, on: string): number {
  const [by, bm, bd] = birth.split('-').map(Number);
  const [y, m, d] = on.split('-').map(Number);
  let age = (y ?? 0) - (by ?? 0);
  if ((m ?? 0) < (bm ?? 0) || (m === bm && (d ?? 0) < (bd ?? 0))) age -= 1;
  return age;
}
