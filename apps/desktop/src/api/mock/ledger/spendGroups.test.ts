/**
 * Grupos de gasto no mock: o relatório bate com o fluxo do mês, Futuro conta aportes e
 * transferências para metas, o grupo das categorias acompanha `is_essential` e as metas validam.
 */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { Category } from '@/api/generated/Category';
import type { ChangeSetSummary } from '@/api/generated/ChangeSetSummary';
import type { GroupTargets } from '@/api/generated/GroupTargets';
import type { MonthFlow } from '@/api/generated/MonthFlow';
import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import { mockCall, resetMock } from '@/api/mock';
import { groupStatus } from '@/features/reports/model/groups';

import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;
const get = (id: string) => call<Category>('categories.get', { id });

beforeEach(() => {
  setMockToday('2026-10-14');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock de grupos de gasto', () => {
  it('relatório: despesas por grupo batem com o fluxo; Futuro soma aportes e metas', async () => {
    const r = await call<SpendGroupsReport>('reports.spend_groups', {
      from: '2026-07',
      to: '2026-09',
    });
    const flow = await call<MonthFlow[]>('reports.month_flow', {
      from_month: '2026-07',
      to_month: '2026-09',
    });
    expect(r.months.map((m) => m.month)).toEqual(['2026-07', '2026-08', '2026-09']);
    r.months.forEach((m, i) => {
      expect(m.groups.map((g) => g.group)).toEqual([
        'essential',
        'debt',
        'future',
        'non_essential',
      ]);
      expect(m.income).toBe(flow[i]!.income);
      const byGroup = Object.fromEntries(m.groups.map((g) => [g.group, g]));
      // Despesas com categoria (e sem) ficam nos outros três grupos; Futuro vem de fora delas.
      expect(byGroup.essential!.amount + byGroup.debt!.amount + byGroup.non_essential!.amount).toBe(
        flow[i]!.expense,
      );
      expect(m.spending).toBe(m.groups.reduce((s, g) => s + g.amount, 0));
      const future = byGroup.future!;
      expect(future.top.map((c) => c.label).sort()).toEqual([
        'Aportes',
        'Transferências para metas',
      ]);
      expect(future.amount).toBe(200_000 + 120_000);
      expect(future.share_of_income).toBeCloseTo(future.amount / m.income);
      expect(byGroup.essential!.top.length).toBeLessThanOrEqual(3);
    });
    expect(r.total.month).toBe('total');
    expect(r.total.income).toBe(r.months.reduce((s, m) => s + m.income, 0));
    expect(r.total.spending).toBe(r.months.reduce((s, m) => s + m.spending, 0));
    expect(r.targets).toEqual({ essential: 0.5, debt: null, future: 0.2, non_essential: 0.3 });
  });

  it('categorias: grupo padrão, set_group com is_essential junto, desfazer e receitas sem grupo', async () => {
    expect((await get('cat-moradia-aluguel')).spend_group).toBe('essential');
    expect((await get('cat-alimentacao-restaurantes')).spend_group).toBe('non_essential');
    expect((await get('cat-impostos-juros')).spend_group).toBe('debt');
    expect((await get('cat-salario')).spend_group).toBeNull();

    const res = await call<{ updated: number }>('categories.set_group', {
      ids: ['cat-lazer', 'cat-lazer-viagens', 'cat-moradia'],
      group: 'future',
    });
    expect(res.updated).toBe(3);
    const lazer = await get('cat-lazer');
    expect(lazer.spend_group).toBe('future');
    expect((await get('cat-moradia')).is_essential).toBe(false);
    const [last] = await call<ChangeSetSummary[]>('changes.list', { limit: 1 });
    await call('changes.undo', { id: last!.id });
    expect((await get('cat-moradia')).spend_group).toBe('essential');
    expect((await get('cat-moradia')).is_essential).toBe(true);

    // Receitas da lista ficam como estão (não têm grupo).
    expect(
      await call<{ updated: number }>('categories.set_group', {
        ids: ['cat-salario'],
        group: 'future',
      }),
    ).toEqual({ updated: 0 });
    expect((await get('cat-salario')).spend_group).toBeNull();
    await expect(
      call('categories.set_group', { ids: ['cat-lazer'], group: 'luxo' }),
    ).rejects.toMatchObject({ code: 'invalid' });

    // `is_essential` antigo continua valendo e leva o grupo junto.
    await call('categories.update', { id: 'cat-pets', is_essential: true });
    expect((await get('cat-pets')).spend_group).toBe('essential');
    await call('categories.update', { id: 'cat-pets', is_essential: false });
    expect((await get('cat-pets')).spend_group).toBe('non_essential');
    await call('categories.update', { id: 'cat-pets', spend_group: 'debt' });
    expect(await get('cat-pets')).toMatchObject({ spend_group: 'debt', is_essential: true });

    // Categoria nova: herda do pai; sem pai, não essencial.
    const sub = await call<Category>('categories.create', {
      name: 'Faxina',
      kind: 'expense',
      parent_id: 'cat-moradia',
    });
    expect(sub.spend_group).toBe('essential');
    const root = await call<Category>('categories.create', { name: 'Clube', kind: 'expense' });
    expect(root.spend_group).toBe('non_essential');
  });

  it('metas: grava frações de 0 a 1 (ou sem meta) e recusa o resto', async () => {
    const next = await call<GroupTargets>('budget.set_group_targets', {
      essential: 0.6,
      debt: 0.1,
      future: null,
      non_essential: 0.3,
    });
    expect(next).toEqual({ essential: 0.6, debt: 0.1, future: null, non_essential: 0.3 });
    expect(await call<GroupTargets>('budget.group_targets')).toEqual(next);
    await expect(
      call('budget.set_group_targets', { ...next, essential: 1.5 }),
    ).rejects.toMatchObject({ code: 'invalid' });
    const r = await call<SpendGroupsReport>('reports.spend_groups', {
      from: '2026-09',
      to: '2026-09',
    });
    expect(r.targets).toEqual(next);
  });

  it('situação contra a meta: Futuro é piso, os outros são teto', () => {
    expect(groupStatus('essential', 0.55, 0.5)).toBe('over');
    expect(groupStatus('essential', 0.5, 0.5)).toBe('within');
    expect(groupStatus('future', 0.1, 0.2)).toBe('below');
    expect(groupStatus('future', 0.2004, 0.2)).toBe('reached');
    expect(groupStatus('debt', 0.1, null)).toBe('none');
    expect(groupStatus('non_essential', null, 0.3)).toBe('none');
  });
});
