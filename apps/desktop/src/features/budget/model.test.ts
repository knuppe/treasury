import { describe, expect, it } from 'vitest';

import type { BudgetLineStatus } from '@/api/generated/BudgetLineStatus';
import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { Category } from '@/api/generated/Category';

import {
  addableCategories,
  budgetSummary,
  budgetTree,
  emptyLine,
  suggestionRows,
  unbudgetedWithSpending,
  visibleRows,
} from './model';

const line = (
  id: string,
  parent: string | null,
  planned: number,
  actual: number,
  own = planned > 0,
  committed = 0,
): BudgetLineStatus => ({
  category_id: id,
  category_name: id,
  category_path: parent ? `${parent} › ${id}` : id,
  parent_id: parent,
  kind: 'expense',
  planned,
  actual,
  remaining: planned - actual,
  ratio: planned ? actual / planned : null,
  committed,
  has_own_line: own,
  color: null,
  archived: false,
});

const status = (lines: BudgetLineStatus[]): BudgetStatus => ({
  month: '2026-10',
  basis: 'accrual',
  lines,
  month_elapsed: 0.5,
  currency: 'BRL',
  missing_fx: [],
  estimated: false,
  totals: {
    planned_expense: 1000,
    actual_expense: 700,
    planned_income: 0,
    actual_income: 0,
    committed_expense: 400,
    committed_income: 0,
    uncategorized_expense: 50,
  },
});

const category = (id: string, parent: string | null = null): Category => ({
  id,
  parent_id: parent,
  name: id,
  kind: 'expense',
  icon: null,
  color: null,
  is_essential: false,
  spend_group: 'non_essential',
  archived: false,
  sort_order: 0,
  path: id,
});

describe('budget', () => {
  const s = status([
    line('alim', null, 300, 250, false),
    line('mercado', 'alim', 200, 180),
    line('rest', 'alim', 100, 70),
    line('pets', null, 0, 80),
    line('racao', 'pets', 0, 80),
    line('lazer', null, 100, 130),
  ]);

  it('tree parent › children; parent planned by the sum of the children', () => {
    const tree = budgetTree(s, 'expense');
    expect(tree.map((r) => r.line.category_id)).toEqual(['alim', 'pets', 'lazer']);
    expect(tree[0]!.children.map((r) => r.line.category_id)).toEqual(['mercado', 'rest']);
    expect(tree[0]!.plannedFromChildren).toBe(true);
    expect(tree[2]!.status).toBe('over');
    expect(tree[1]!.status).toBe('none');
  });

  it('expands only the open parents', () => {
    const tree = budgetTree(s, 'expense');
    expect(visibleRows(tree, new Set()).length).toBe(3);
    expect(visibleRows(tree, new Set(['alim'])).map((r) => r.line.category_id)).toEqual([
      'alim',
      'mercado',
      'rest',
      'pets',
      'lazer',
    ]);
  });

  it('a user-added category comes in as an empty row', () => {
    const tree = budgetTree(s, 'expense', [emptyLine(category('casa'))]);
    expect(tree.map((r) => r.line.category_id)).toContain('casa');
  });

  it('spending without a budget: only the highest uncovered level', () => {
    expect(unbudgetedWithSpending(s).map((l) => l.category_id)).toEqual(['pets']);
    const covered = status([line('alim', null, 300, 100), line('padaria', 'alim', 0, 50, false)]);
    expect(unbudgetedWithSpending(covered)).toEqual([]);
  });

  it('summary with projection and state', () => {
    expect(budgetSummary(s)).toMatchObject({
      planned: 1000,
      actual: 700,
      remaining: 300,
      projected: 1100,
      status: 'risk',
      uncategorized: 50,
    });
  });

  it('suggestion: only where it changes, with name and difference', () => {
    const rows = suggestionRows(
      s,
      [
        { category_id: 'lazer', amount: 150 },
        { category_id: 'alim', amount: 300 },
        { category_id: 'casa', amount: 50 },
      ],
      [category('casa')],
    );
    expect(rows).toEqual([
      { category_id: 'lazer', name: 'lazer', current: 100, suggested: 150, delta: 50 },
      { category_id: 'casa', name: 'casa', current: 0, suggested: 50, delta: 50 },
    ]);
  });

  it('categories that can still enter the budget', () => {
    expect(
      addableCategories([category('lazer'), category('casa')], s, 'expense').map((c) => c.id),
    ).toEqual(['casa']);
  });
});
