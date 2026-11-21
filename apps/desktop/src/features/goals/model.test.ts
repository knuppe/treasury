import { describe, expect, it } from 'vitest';

import type { EmergencyReserve } from '@/api/generated/EmergencyReserve';
import type { GoalView } from '@/api/generated/GoalView';

import {
  emptyGoalForm,
  formFromGoal,
  goalInput,
  goalPatch,
  goalSituation,
  hasChanges,
  paceSource,
  reserveView,
  validateGoal,
} from './model';

const goal = (over: Partial<GoalView> = {}): GoalView => ({
  id: 'g1',
  name: 'Viagem',
  kind: 'travel',
  target_amount: 10_000,
  currency: 'BRL',
  target_date: '2027-06-01',
  initial_amount: 1_000,
  monthly_contribution: 500,
  account_ids: ['acc-b', 'acc-a'],
  archived: false,
  notes: 'Japão',
  real_return: null,
  current_amount: 4_000,
  progress: 0.4,
  remaining: 6_000,
  monthly_needed: 800,
  avg_monthly_contribution: 0,
  projected_date: '2027-10-01',
  on_track: false,
  missing_fx: [],
  estimated: false,
  sources: [],
  breakdown: [],
  stale_sources: [],
  ...over,
});

describe('metas', () => {
  it('situação: atingida, no caminho, fora do ritmo, sem aportes, sem data', () => {
    expect(goalSituation(goal({ remaining: 0 }))).toBe('done');
    expect(goalSituation(goal({ on_track: true }))).toBe('on_track');
    expect(goalSituation(goal())).toBe('off_track');
    expect(goalSituation(goal({ projected_date: null }))).toBe('no_pace');
    expect(goalSituation(goal({ target_date: null, on_track: null }))).toBe('no_date');
  });

  it('ritmo: real pelas contas, senão o planejado', () => {
    expect(paceSource(goal())).toBe('planned');
    expect(paceSource(goal({ avg_monthly_contribution: 300 }))).toBe('real');
    expect(paceSource(goal({ monthly_contribution: null }))).toBe('none');
  });

  it('valida o formulário', () => {
    const f = emptyGoalForm();
    expect(Object.keys(validateGoal(f, '2026-10-14')).sort()).toEqual(['name', 'target']);
    expect(
      validateGoal({ ...f, name: 'X', target: 100, targetDate: '2026-01-01' }, '2026-10-14'),
    ).toHaveProperty('targetDate');
    expect(validateGoal({ ...f, name: 'X', target: 100 }, '2026-10-14')).toEqual({});
  });

  it('entrada do núcleo só com o preenchido', () => {
    expect(
      goalInput({ ...emptyGoalForm('emergency'), target: 9_000, accountIds: ['b', 'a'] }),
    ).toEqual({
      name: 'Reserva de emergência',
      kind: 'emergency',
      target_amount: 9_000,
      account_ids: ['a', 'b'],
    });
  });

  it('rendimento real em % no formulário e fração no núcleo', () => {
    const f = { ...emptyGoalForm('other'), name: 'Milhão', target: 100, realReturn: 4 };
    expect(goalInput(f).real_return).toBe(0.04);
    expect(validateGoal({ ...f, realReturn: 20 }, '2026-10-14')).toHaveProperty('realReturn');
    const g = goal({ real_return: 0.035 });
    expect(formFromGoal(g).realReturn).toBe(3.5);
    expect(goalPatch(g, { ...formFromGoal(g), realReturn: null })).toEqual({
      id: 'g1',
      real_return: null,
    });
  });

  it('patch só com o que mudou; null limpa data, aporte e notas', () => {
    const g = goal();
    expect(hasChanges(goalPatch(g, formFromGoal(g)))).toBe(false);
    const p = goalPatch(g, {
      ...formFromGoal(g),
      targetDate: null,
      monthly: null,
      notes: '',
      accountIds: ['acc-a'],
    });
    expect(p).toEqual({
      id: 'g1',
      target_date: null,
      monthly_contribution: null,
      notes: null,
      account_ids: ['acc-a'],
    });
    expect(goalPatch(g, { ...formFromGoal(g), initial: null })).toEqual({
      id: 'g1',
      initial_amount: 0,
    });
  });

  it('reserva de emergência: cobertura, progresso e estado', () => {
    const r: EmergencyReserve = {
      avg_essential_monthly: 1_000,
      avg_essential_regular_monthly: 1_000,
      monthly_cost: 1_000,
      annual: { items: [], monthly: 0, essential_monthly: 0, months: [], complete_year: false },
      include_annual: false,
      avg_total_expense_monthly: 1_500,
      months_analyzed: 6,
      target_months: 6,
      target_amount: 6_000,
      current_amount: 2_000,
      coverage_months: 2,
      goal_id: null,
      currency: 'BRL',
      months: [],
      basis: 'accrual',
      missing_fx: [],
      estimated: false,
    };
    expect(reserveView(r)).toMatchObject({ missing: 4_000, status: 'low' });
    expect(reserveView({ ...r, current_amount: 7_000, coverage_months: 7 }).status).toBe('ok');
    expect(
      reserveView({ ...r, avg_essential_monthly: 0, target_amount: 0, coverage_months: null })
        .status,
    ).toBe('empty');
  });
});
