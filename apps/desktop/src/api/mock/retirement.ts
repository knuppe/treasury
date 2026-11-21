/**
 * Mock de `retirement.*` (docs/international.md), simplificado: os planos do país com o limite
 * de 2026 (sem idade nem grupo) e o que entrou no ano na conta.
 */
import type { RetirementAccountPlan } from '../generated/RetirementAccountPlan';
import type { RetirementAccountStatus } from '../generated/RetirementAccountStatus';
import { mockFail, onMockReset, registerMockHandlers } from '../mockRegistry';
import { familyCountry } from './country';
import { guarded } from './ledger/boot';
import { rows } from './ledger/store';

const PLANS: [string, string, string, string, number | null, boolean][] = [
  ['us_401k', 'US', '401(k)', 'USD', 2_450_000, false],
  ['us_roth_401k', 'US', 'Roth 401(k)', 'USD', 2_450_000, false],
  ['us_ira', 'US', 'IRA tradicional', 'USD', 750_000, false],
  ['us_roth_ira', 'US', 'Roth IRA', 'USD', 750_000, false],
  ['us_hsa', 'US', 'HSA (conta de saúde)', 'USD', 440_000, true],
  ['us_529', 'US', '529 (educação dos filhos)', 'USD', null, false],
  ['pt_ppr', 'PT', 'PPR', 'EUR', 150_000, false],
  ['es_plan_pensiones', 'ES', 'Plan de pensiones individual', 'EUR', 150_000, false],
  ['es_plan_empleo', 'ES', 'Plan de pensiones de empleo', 'EUR', 850_000, false],
  ['gb_isa', 'GB', 'ISA (Stocks & Shares ou Cash)', 'GBP', 2_000_000, false],
  ['gb_lisa', 'GB', 'Lifetime ISA', 'GBP', 400_000, false],
  ['gb_pension', 'GB', 'Pensão privada (SIPP ou do trabalho)', 'GBP', 6_000_000, false],
  ['ca_rrsp', 'CA', 'RRSP', 'CAD', 3_381_000, false],
  ['ca_tfsa', 'CA', 'TFSA', 'CAD', 700_000, false],
  ['ca_fhsa', 'CA', 'FHSA (primeira casa)', 'CAD', 800_000, false],
  ['fr_per', 'FR', 'PER (plano de aposentadoria)', 'EUR', null, false],
  ['fr_pea', 'FR', 'PEA (ações)', 'EUR', null, false],
  ['de_riester', 'DE', 'Riester-Rente', 'EUR', 210_000, false],
  ['de_ruerup', 'DE', 'Rürup-Rente (Basisrente)', 'EUR', null, false],
  ['cl_apv', 'CL', 'APV (poupança previdenciária voluntária)', 'CLP', 24_690_000, false],
];

const plan = ([id, country, name, currency, limit, coverage]: (typeof PLANS)[number]) =>
  ({
    id,
    country,
    name,
    currency,
    limit,
    coverage,
    kind: ['pt_ppr', 'fr_per', 'de_riester', 'de_ruerup', 'cl_apv'].includes(id)
      ? 'tax_benefit'
      : 'contribution',
  }) satisfies RetirementAccountPlan;

const byAccount = new Map<string, { plan: string; coverage: string | null }>();
onMockReset(() => byAccount.clear());

/** Plano e cobertura da conta (para `AccountView`). */
export function retirementOf(id: string): {
  retirement_plan: string | null;
  retirement_coverage: string | null;
} {
  const r = byAccount.get(id);
  return { retirement_plan: r?.plan ?? null, retirement_coverage: r?.coverage ?? null };
}

export function setRetirementPlan(id: string, planId: unknown, coverage: unknown): void {
  const country = familyCountry();
  const p =
    PLANS.find((x) => x[0] === planId) ??
    (planId == null ? PLANS.find((x) => x[1] === country) : undefined) ??
    mockFail('invalid', 'Plano de aposentadoria desconhecido para este país.');
  byAccount.set(id, {
    plan: p[0],
    coverage: p[5] ? (coverage === 'family' ? 'family' : 'self') : null,
  });
}

registerMockHandlers(
  guarded({
    'retirement.plans': () => PLANS.filter((p) => p[1] === familyCountry()).map(plan),
    'retirement.status': (a): RetirementAccountStatus => {
      const id = String(a.account_id);
      const r = byAccount.get(id) ?? mockFail('not_found', 'Esta conta não é de aposentadoria.');
      const p = plan(PLANS.find((x) => x[0] === r.plan)!);
      const year = new Date().getFullYear();
      const contributed = rows('txn')
        .filter(
          (t) =>
            t.account_id === id &&
            t.amount > 0 &&
            t.status !== 'planned' &&
            t.date.startsWith(String(year)),
        )
        .reduce((s, t) => s + t.amount, 0);
      const limit = r.coverage === 'family' ? 875_000 : p.limit;
      return {
        account_id: id,
        plan: p,
        coverage: r.coverage,
        year,
        contributed,
        limit,
        remaining: limit == null ? null : Math.max(0, limit - contributed),
        shared_with: [],
        age: null,
        source: null,
        limit_year: null,
      };
    },
  }),
);
