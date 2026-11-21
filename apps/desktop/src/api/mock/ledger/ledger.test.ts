/**
 * O mock de relatórios, orçamento e metas é coerente entre si (os números batem de um
 * comando para o outro, como no núcleo) e segue as regras de fluxo de caixa.
 */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { CalendarData } from '@/api/generated/CalendarData';
import type { Dashboard } from '@/api/generated/Dashboard';
import type { EmergencyReserve } from '@/api/generated/EmergencyReserve';
import type { Explanation } from '@/api/generated/Explanation';
import type { GoalView } from '@/api/generated/GoalView';
import type { MemberBreakdown } from '@/api/generated/MemberBreakdown';
import type { MonthFlow } from '@/api/generated/MonthFlow';
import type { MonthlyEvolution } from '@/api/generated/MonthlyEvolution';
import type { NetWorth } from '@/api/generated/NetWorth';
import type { SankeyData } from '@/api/generated/SankeyData';
import type { SankeyItems } from '@/api/generated/SankeyItems';
import type { SpendingSummary } from '@/api/generated/SpendingSummary';
import { mockCall, resetMock } from '@/api/mock';

import { setMockToday } from './ledger';

const TODAY = '2026-10-14';
const MONTH = { from: '2026-09-01', to: '2026-09-30' };

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday(TODAY);
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock de relatórios: coerência', () => {
  it('é determinístico para o mesmo dia', async () => {
    const a = await call<SpendingSummary>('reports.spending', MONTH);
    resetMock('unlocked');
    const b = await call<SpendingSummary>('reports.spending', MONTH);
    expect(b.total).toBe(a.total);
    expect(a.total).toBeGreaterThan(1_000_000);
  });

  it('painel, fluxo do mês, gastos e orçamento contam o mesmo', async () => {
    const d = await call<Dashboard>('reports.dashboard', { month: '2026-09' });
    const flows = await call<MonthFlow[]>('reports.month_flow', {
      from_month: '2026-09',
      to_month: '2026-09',
    });
    const spending = await call<SpendingSummary>('reports.spending', {
      ...MONTH,
      group_by: 'parent_category',
    });
    const budget = await call<BudgetStatus>('budget.get', { month: '2026-09' });
    expect(d.flow).toEqual(flows[0]);
    expect(spending.total).toBe(d.flow.expense);
    expect(budget.totals.actual_expense).toBe(d.flow.expense);
    expect(d.budget_actual).toBe(d.flow.expense);
    expect(spending.groups.reduce((s, g) => s + g.amount, 0)).toBe(spending.total);
    expect(d.flow_history).toHaveLength(12);
    expect(d.top_categories.length).toBeLessThanOrEqual(5);
  });

  it('Sankey: o que entra no orçamento sai dele; totais iguais aos do fluxo', async () => {
    const s = await call<SankeyData>('reports.sankey', { ...MONTH, basis: 'accrual' });
    const flows = await call<MonthFlow[]>('reports.month_flow', {
      from_month: '2026-09',
      to_month: '2026-09',
    });
    const into = s.links.filter((l) => l.target === 'hub').reduce((a, l) => a + l.value, 0);
    const out = s.links.filter((l) => l.source === 'hub').reduce((a, l) => a + l.value, 0);
    expect(into).toBe(out);
    expect(s.total_income).toBe(flows[0]!.income);
    expect(s.total_expense).toBe(flows[0]!.expense);
    for (const n of s.nodes.filter((x) => x.kind === 'expense')) {
      const inN = s.links.filter((l) => l.target === n.id).reduce((a, l) => a + l.value, 0);
      const outN = s.links.filter((l) => l.source === n.id).reduce((a, l) => a + l.value, 0);
      if (outN) expect(outN).toBe(inN);
    }
  });

  it('Sankey: aportes à parte e o mês anterior ao lado', async () => {
    const s = await call<SankeyData>('reports.sankey', { ...MONTH, basis: 'accrual' });
    const aug = await call<MonthFlow[]>('reports.month_flow', {
      from_month: '2026-08',
      to_month: '2026-08',
    });
    // O aporte mensal na XP sai da conta do Bruno; a compra de ativos na corretora não conta.
    expect(s.totals.investments).toBe(200_000);
    expect(s.links.find((l) => l.target === 'investment')?.value).toBe(200_000);
    expect(s.totals.leftover).toBe(
      s.totals.income + s.totals.redemptions - s.totals.expense - s.totals.investments,
    );
    expect(s.previous).toMatchObject({ from: '2026-08-01', to: '2026-08-31' });
    expect(s.previous!.income).toBe(aug[0]!.income);
    expect(s.previous!.expense).toBe(aug[0]!.expense);
    const inv = s.comparison.find((r) => r.id === 'investment')!;
    expect([inv.current, inv.previous, inv.delta, inv.delta_pct]).toEqual([200_000, 200_000, 0, 0]);
    expect(s.comparison.some((r) => r.kind === 'hub')).toBe(false);

    const flat = await call<SankeyData>('reports.sankey', {
      ...MONTH,
      basis: 'accrual',
      depth: 1,
      compare: false,
    });
    expect(flat.depth).toBe(1);
    expect(flat.nodes.some((n) => n.kind === 'subcategory')).toBe(false);
    expect([flat.previous, flat.comparison]).toEqual([null, []]);
  });

  it('o analítico de cada linha do comparativo soma as duas células', async () => {
    for (const basis of ['accrual', 'cash'] as const) {
      for (const period of [MONTH, { from: '2026-07-01', to: '2026-09-30' }]) {
        const s = await call<SankeyData>('reports.sankey', { ...period, basis });
        expect(s.comparison.length).toBeGreaterThan(5);
        for (const row of s.comparison) {
          const it = await call<SankeyItems>('reports.sankey_items', {
            ...period,
            basis,
            node_ids: [row.id],
          });
          const label = `${row.name} ${basis} ${period.from}`;
          expect([it.current.value, it.previous.value], label).toEqual([row.current, row.previous]);
          expect([it.current.items_total, it.previous.items_total], label).toEqual([
            row.current,
            row.previous,
          ]);
          expect(
            it.groups.reduce((a, g) => a + g.delta, 0),
            label,
          ).toBe(row.delta);
          expect(it.name, label).toBe(row.name);
        }
      }
    }
    await expect(
      mockCall('reports.sankey_items', { ...MONTH, node_ids: ['hub'] }),
    ).rejects.toMatchObject({ code: 'invalid' });
  });

  it('calendário, titulares e evolução somam o mesmo gasto', async () => {
    const cal = await call<CalendarData>('reports.calendar', MONTH);
    const members = await call<MemberBreakdown>('reports.by_member', MONTH);
    const evo = await call<MonthlyEvolution>('reports.monthly_evolution', {
      months: 12,
      end_month: '2026-09',
    });
    const spending = await call<SpendingSummary>('reports.spending', MONTH);
    expect(cal.days).toHaveLength(30);
    expect(cal.days.reduce((a, d) => a + d.expense, 0)).toBe(spending.total);
    expect(members.total).toBe(spending.total);
    expect(members.members.map((m) => m.label)).toEqual(['Ana', 'Bruno', 'Sem membro']);
    expect(evo.totals[11]).toBe(spending.total);
    expect(evo.series.reduce((a, s) => a + (s.values[11] ?? 0), 0)).toBe(spending.total);
    expect(evo.moving_avg_3).toHaveLength(12);
  });

  it('competência × caixa: o cartão conta no vencimento da fatura', async () => {
    const accrual = await call<SpendingSummary>('reports.spending', {
      ...MONTH,
      basis: 'accrual',
      account_ids: ['acc-nucard'],
    });
    const cash = await call<SpendingSummary>('reports.spending', {
      ...MONTH,
      basis: 'cash',
      account_ids: ['acc-nucard'],
    });
    expect(accrual.total).toBeGreaterThan(0);
    expect(cash.total).not.toBe(accrual.total);
  });

  it('a explicação soma as fontes até o valor', async () => {
    const e = await call<Explanation>('reports.explain', {
      metric: 'spending',
      params: MONTH,
      limit: 10_000,
    });
    const spending = await call<SpendingSummary>('reports.spending', MONTH);
    expect(e.value).toBe(spending.total);
    expect(e.truncated).toBe(false);
    expect(e.sources.reduce((a, s) => a + s.amount, 0)).toBe(e.value);
    expect(e.formula).toContain('competência');
    const nw = await call<Explanation>('reports.explain', {
      metric: 'net_worth',
      params: { as_of: TODAY },
    });
    expect(nw.sources.reduce((a, s) => a + s.amount, 0)).toBe(nw.value);
    const rate = await call<Explanation>('reports.explain', {
      metric: 'savings_rate',
      params: { month: '2026-09' },
    });
    expect(rate.ratio).not.toBeNull();
  });

  it('patrimônio: ativos − passivos, carteira inclusa, EUR sem câmbio sinalizado', async () => {
    const nw = await call<NetWorth>('reports.net_worth', { as_of: TODAY });
    expect(nw.net).toBe(nw.assets - nw.liabilities);
    expect(nw.investments).toBeGreaterThan(0);
    expect(nw.missing_fx).toEqual(['EUR']);
    expect(nw.estimated).toBe(true);
    expect(nw.accounts.find((a) => a.currency === 'EUR')!.balance_base).toBeNull();
  });

  it('erros no formato do núcleo', async () => {
    // Sem base, o diagrama de fluxo usa o ajuste próprio (padrão caixa).
    expect((await call<SankeyData>('reports.sankey', MONTH)).basis).toBe('cash');
    await expect(mockCall('reports.sankey', {})).rejects.toMatchObject({ code: 'invalid' });
    resetMock('locked');
    await expect(mockCall('reports.dashboard', {})).rejects.toMatchObject({ code: 'vault_locked' });
  });
});

describe('mock de orçamento', () => {
  it('grava, remove (0), copia e sugere', async () => {
    const after = await call<BudgetStatus>('budget.set_lines', {
      month: '2026-10',
      lines: [{ category_id: 'cat-pets', amount: 40_000 }],
    });
    expect(after.lines.find((l) => l.category_id === 'cat-pets')).toMatchObject({
      planned: 40_000,
      has_own_line: true,
    });
    const removed = await call<BudgetStatus>('budget.set_lines', {
      month: '2026-10',
      lines: [{ category_id: 'cat-pets', amount: 0 }],
    });
    expect(removed.lines.find((l) => l.category_id === 'cat-pets')?.has_own_line).toBe(false);

    await expect(
      mockCall('budget.copy', { from_month: '2026-12', to_month: '2027-01' }),
    ).rejects.toMatchObject({
      code: 'not_found',
    });
    const copied = await call<BudgetStatus>('budget.copy', {
      from_month: '2026-10',
      to_month: '2026-11',
    });
    expect(copied.totals.planned_expense).toBeGreaterThan(0);
    expect(await call<string[]>('budget.list_months')).toContain('2026-11');

    const suggestion = await call<{ category_id: string; amount: number }[]>('budget.suggest', {
      month: '2026-11',
    });
    expect(suggestion.length).toBeGreaterThan(5);
    expect(suggestion.every((s) => s.amount % 1_000 === 0)).toBe(true);
  });

  it('pais sem linha própria somam os filhos; o realizado inclui as subcategorias', async () => {
    const b = await call<BudgetStatus>('budget.get', { month: '2026-09' });
    const alim = b.lines.find((l) => l.category_id === 'cat-alimentacao')!;
    const kids = b.lines.filter((l) => l.parent_id === 'cat-alimentacao');
    expect(alim.has_own_line).toBe(false);
    expect(alim.planned).toBe(kids.reduce((s, l) => s + l.planned, 0));
    expect(alim.actual).toBeGreaterThanOrEqual(kids.reduce((s, l) => s + l.actual, 0));
    expect(b.month_elapsed).toBe(1);
  });
});

describe('mock de metas', () => {
  it('lista, cria, edita, arquiva e remove', async () => {
    const list = await call<GoalView[]>('goals.list');
    expect(list.map((g) => g.name)).toEqual([
      'Reserva de emergência',
      'Viagem ao Japão',
      'Troca do carro',
    ]);
    const reserva = list[0]!;
    expect(reserva.current_amount).toBeGreaterThan(0);
    expect(reserva.avg_monthly_contribution).toBeGreaterThan(0);

    const created = await call<GoalView>('goals.create', {
      name: '  Intercâmbio ',
      kind: 'education',
      target_amount: 2_000_000,
      target_date: '2028-01-01',
      account_ids: ['acc-poupanca'],
    });
    expect(created.name).toBe('Intercâmbio');
    expect(typeof created.monthly_needed).toBe('number');
    await expect(
      mockCall('goals.create', { name: '', kind: 'other', target_amount: 0 }),
    ).rejects.toMatchObject({
      code: 'invalid',
    });

    const edited = await call<GoalView>('goals.update', {
      id: created.id,
      target_date: null,
      archived: true,
    });
    expect(edited).toMatchObject({
      target_date: null,
      archived: true,
      on_track: null,
      monthly_needed: null,
    });
    expect((await call<GoalView[]>('goals.list')).some((g) => g.id === created.id)).toBe(false);
    expect(
      (await call<GoalView[]>('goals.list', { include_archived: true })).some(
        (g) => g.id === created.id,
      ),
    ).toBe(true);

    await call('goals.delete', { id: created.id });
    await expect(mockCall('goals.get', { id: created.id })).rejects.toMatchObject({
      code: 'not_found',
    });
  });

  it('reserva de emergência: média essencial × meses-alvo', async () => {
    const r = await call<EmergencyReserve>('goals.emergency_reserve', {
      target_months: 3,
      months_back: 6,
    });
    expect(r.months).toEqual(['2026-04', '2026-05', '2026-06', '2026-07', '2026-08', '2026-09']);
    // Gastos anuais do ano entram divididos por 12, no lugar dos picos.
    expect(r.annual.complete_year).toBe(true);
    expect(r.include_annual).toBe(r.annual.essential_monthly > 0);
    expect(r.monthly_cost).toBe(
      r.include_annual
        ? r.avg_essential_regular_monthly + r.annual.essential_monthly
        : r.avg_essential_monthly,
    );
    expect(r.target_amount).toBe(r.monthly_cost * 3);
    expect(r.avg_essential_monthly).toBeLessThan(r.avg_total_expense_monthly);
    expect(r.goal_id).toBe('goal-reserva');
    const off = await call<EmergencyReserve>('goals.emergency_reserve', {
      target_months: 3,
      months_back: 6,
      include_annual: false,
    });
    expect(off.include_annual).toBe(false);
    expect(off.target_amount).toBe(off.avg_essential_monthly * 3);
  });

  it('meta com rendimento: anuidade, aportes e rendimento', async () => {
    const g = await call<GoalView>('goals.create', {
      name: 'Primeiro milhão',
      kind: 'other',
      target_amount: 100_000_000,
      target_date: '2046-10-01',
      real_return: 0.04,
    });
    expect(g.growth?.months).toBeGreaterThan(200);
    expect(g.monthly_needed!).toBeLessThan(g.growth!.linear_monthly_needed!);
    expect(g.growth!.from_contributions! + g.growth!.from_returns!).toBe(100_000_000);
    const linear = await call<GoalView>('goals.update', { id: g.id, real_return: null });
    expect(linear.growth).toBeUndefined();
    expect(linear.monthly_needed).toBe(g.growth!.linear_monthly_needed);
  });
});
