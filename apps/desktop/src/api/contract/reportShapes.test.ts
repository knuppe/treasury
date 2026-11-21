/**
 * The mock answers with the same shapes as the core (the contract in `reportShapes.ts`, which
 * `scripts/contract-reports.mjs` also checks against `treasury-dev`).
 */
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import { mockCall, resetMock } from '@/api/mock';
import { setMockToday } from '@/api/mock/ledger/ledger';

import { checkShape, RESPONSES } from './reportShapes';

const TODAY = '2026-10-14';
const SEP = { from: '2026-09-01', to: '2026-09-30' };

const CALLS: [string, object][] = [
  ['reports.dashboard', {}],
  ['reports.dashboard', { month: '2026-09' }],
  ['reports.net_worth', { as_of: TODAY }],
  ['reports.net_worth_history', { months: 24 }],
  ['reports.month_flow', { from_month: '2025-11', to_month: '2026-10', basis: 'cash' }],
  ['reports.spending', { ...SEP, group_by: 'category' }],
  ['reports.spending', { ...SEP, group_by: 'parent_category', member_ids: ['mem-ana'] }],
  ['reports.spending', { ...SEP, group_by: 'member' }],
  ['reports.spending', { ...SEP, group_by: 'account', kind: 'income' }],
  ['reports.spending', { ...SEP, group_by: 'month' }],
  ['reports.sankey', SEP],
  ['reports.sankey', { ...SEP, depth: 1, compare: false }],
  ['reports.sankey_items', { ...SEP, node_ids: ['savings'] }],
  ['reports.sankey_items', { ...SEP, node_ids: ['expense:cat-moradia'] }],
  ['reports.sankey_items', { ...SEP, basis: 'cash', node_ids: ['investment', 'redemption'] }],
  ['reports.calendar', { from: '2025-11-01', to: TODAY }],
  ['reports.monthly_evolution', { months: 24, top_n: 99 }],
  ['reports.compare', { a: { from: '2026-08-01', to: '2026-08-31' }, b: SEP }],
  ['reports.by_member', SEP],
  ['reports.upcoming', { days: 30 }],
  ['reports.inflation', {}],
  ['reports.inflation', { months: 24, end_month: '2026-09', basis: 'cash' }],
  ['reports.spend_groups', { from: '2026-04', to: '2026-09' }],
  ['reports.spend_groups', { from: '2026-09', to: '2026-09', basis: 'cash', member_id: 'mem-ana' }],
  ['reports.explain', { metric: 'spending', params: { ...SEP, category_id: 'cat-moradia' } }],
  ['reports.explain', { metric: 'income', params: SEP }],
  ['reports.explain', { metric: 'savings_rate', params: { month: '2026-09' } }],
  ['reports.explain', { metric: 'net_worth', params: { as_of: TODAY } }],
  ['reports.explain', { metric: 'account_balance', params: { account_id: 'acc-itau' } }],
  ['budget.get', { month: '2026-10', basis: 'cash' }],
  ['budget.set_lines', { month: '2026-10', lines: [{ category_id: 'cat-pets', amount: 30_000 }] }],
  ['budget.copy', { from_month: '2026-10', to_month: '2026-11' }],
  ['budget.suggest', { month: '2026-11', months_back: 6 }],
  ['budget.list_months', {}],
  ['budget.group_targets', {}],
  ['budget.set_group_targets', { essential: 0.55, debt: null, future: 0.15, non_essential: 0.3 }],
  ['goals.list', { include_archived: true }],
  ['goals.get', { id: 'goal-japao' }],
  ['goals.create', { name: 'Contrato', kind: 'other', target_amount: 100_000 }],
  ['goals.update', { id: 'goal-carro', monthly_contribution: null, target_date: null }],
  ['goals.emergency_reserve', { target_months: 9 }],
  ['portfolio.summary', {}],
  ['recurrences.list', {}],
  ['cards.future_commitments', { account_id: 'acc-nucard', months: 12 }],
  ['categories.list', {}],
  ['household.list_members', {}],
  ['accounts.list', {}],
  ['health.score', {}],
  ['health.score', { as_of: '2026-07' }],
  ['health.get_settings', {}],
];

beforeAll(() => {
  setMockToday(TODAY);
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock contract', () => {
  it.each(CALLS)('%s %j', async (cmd, args) => {
    const shape = RESPONSES[cmd];
    expect(shape, `no shape for ${cmd}`).toBeDefined();
    const res = await mockCall(cmd, args);
    expect(checkShape(res, shape!)).toEqual([]);
  });

  it('the checker flags an extra key, a missing key and a wrong type', () => {
    expect(checkShape({ a: 1, b: 2 }, { a: 'number' })).toEqual(['$.b: chave a mais']);
    expect(checkShape({}, { a: 'number' })).toEqual(['$.a: faltando']);
    expect(checkShape({ a: 'x' }, { a: 'number|null' })).toEqual([
      '$.a: esperado number|null, veio string',
    ]);
    expect(checkShape({ a: 1, z: 1 }, { $partial: { a: 'number' } })).toEqual([]);
    expect(checkShape('cash', { $enum: ['accrual'] })[0]).toContain('esperado um de');
  });
});
