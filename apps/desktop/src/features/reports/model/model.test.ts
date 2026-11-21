import { describe, expect, it } from 'vitest';

import type { Category } from '@/api/generated/Category';
import type { MonthlyEvolution } from '@/api/generated/MonthlyEvolution';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { SankeyData } from '@/api/generated/SankeyData';
import type { SpendingSummary } from '@/api/generated/SpendingSummary';
import { categoricalPalette } from '@/charts/palette';
import { parseTxnLink, txnLink } from '@/lib/txnLink';

import { entityColors, OTHERS_KEY } from './colors';
import {
  cardMonthDrill,
  categoryDrill,
  dayDrill,
  memberDrill,
  monthDrill,
  recurrenceDrill,
  uncategorizedDrill,
  upcomingDrill,
} from './drill';
import { flowModel } from './flow';
import {
  calendarWindow,
  filtersPeriod,
  filtersSearch,
  parseFilters,
  periodLabel,
  presetPeriod,
  previousPeriod,
  singleMonth,
  trendWindow,
  yearBeforePeriod,
} from './period';
import {
  cardsModel,
  comparisonRows,
  compositionModel,
  membersModel,
  netWorthModel,
  recurrenceSummary,
  slopeRows,
  timelineRows,
  trendModel,
} from './series';

const TODAY = '2026-10-14';

/** Filtro que a tela de lançamentos lê do link gerado. */
const linkFilter = (f: Parameters<typeof txnLink>[0]) =>
  parseTxnLink(txnLink(f).split('?')[1] ?? '');

const cat = (id: string, name: string, parent: string | null = null, order = 0): Category => ({
  id,
  parent_id: parent,
  name,
  kind: 'expense',
  icon: null,
  color: null,
  is_essential: false,
  spend_group: 'non_essential',
  archived: false,
  sort_order: order,
  path: parent ? `${parent} › ${name}` : name,
});

describe('período e filtros', () => {
  it('atalhos relativos a hoje', () => {
    expect(presetPeriod('mes', TODAY)).toEqual({ from: '2026-10-01', to: '2026-10-31' });
    expect(presetPeriod('mes-anterior', TODAY)).toEqual({ from: '2026-09-01', to: '2026-09-30' });
    expect(presetPeriod('3m', TODAY)).toEqual({ from: '2026-08-01', to: '2026-10-31' });
    expect(presetPeriod('12m', TODAY).from).toBe('2025-11-01');
    expect(presetPeriod('ano-anterior', TODAY)).toEqual({ from: '2025-01-01', to: '2025-12-31' });
  });

  it('lê e grava os filtros na URL (ida e volta)', () => {
    const f = parseFilters(
      'v=composicao&p=personalizado&from=2026-01-10&to=2026-02-20&basis=cash&member=m1&account=a1',
      TODAY,
    );
    expect(f).toMatchObject({
      view: 'composicao',
      preset: 'personalizado',
      basis: 'cash',
      member: 'm1',
      account: 'a1',
    });
    expect(filtersPeriod(f, TODAY)).toEqual({ from: '2026-01-10', to: '2026-02-20' });
    expect(parseFilters(filtersSearch(f), TODAY)).toEqual(f);
    const d = parseFilters('', TODAY);
    expect(d).toMatchObject({ view: 'fluxo', preset: 'mes', basis: null, member: null });
    expect(parseFilters('v=xyz&basis=nada', TODAY)).toMatchObject({ view: 'fluxo', basis: null });
  });

  it('período anterior, um ano antes, rótulos e janelas', () => {
    expect(previousPeriod({ from: '2026-08-01', to: '2026-10-31' })).toEqual({
      from: '2026-05-01',
      to: '2026-07-31',
    });
    expect(previousPeriod({ from: '2026-10-05', to: '2026-10-14' })).toEqual({
      from: '2026-09-25',
      to: '2026-10-04',
    });
    expect(yearBeforePeriod({ from: '2026-10-01', to: '2026-10-31' })).toEqual({
      from: '2025-10-01',
      to: '2025-10-31',
    });
    expect(periodLabel({ from: '2026-10-01', to: '2026-10-31' })).toBe('outubro de 2026');
    expect(periodLabel({ from: '2026-01-01', to: '2026-03-31' })).toBe(
      'janeiro de 2026 a março de 2026',
    );
    expect(periodLabel({ from: '2026-01-05', to: '2026-01-20' })).toBe('05/01/2026 a 20/01/2026');
    expect(trendWindow({ from: '2026-10-01', to: '2026-10-31' })).toEqual({
      months: 12,
      endMonth: '2026-10',
    });
    expect(trendWindow({ from: '2024-01-01', to: '2026-10-31' }).months).toBe(24);
    expect(calendarWindow({ from: '2026-10-01', to: '2026-10-31' }, TODAY)).toEqual({
      from: '2025-11-01',
      to: TODAY,
    });
    expect(singleMonth({ from: '2026-10-01', to: '2026-10-31' })).toBe('2026-10');
    expect(singleMonth({ from: '2026-10-01', to: '2026-11-30' })).toBeNull();
  });
});

describe('cores por entidade', () => {
  const order = [
    'moradia',
    'alimentacao',
    'transporte',
    'saude',
    'educacao',
    'lazer',
    'compras',
    'cuidados',
    'pets',
    'filhos',
  ];

  it('ordem fixa da árvore; a 9ª em diante fica sem cor (vira "Outras")', () => {
    const c = entityColors(order, null, 'dark');
    expect(c.get('moradia')).toBe(categoricalPalette('dark')[0]);
    expect(c.get('cuidados')).toBe(categoricalPalette('dark')[7]);
    expect(c.get('pets')).toBeUndefined();
    expect(c.keys).toHaveLength(8);
  });

  it('só disputa cor quem teve gasto na janela fixa, sem depender do ranking', () => {
    const active = new Set(['transporte', 'moradia', 'pets']);
    const c = entityColors(order, active, 'dark');
    expect(c.get('moradia')).toBe(categoricalPalette('dark')[0]);
    expect(c.get('transporte')).toBe(categoricalPalette('dark')[1]);
    expect(c.get('pets')).toBe(categoricalPalette('dark')[2]);
    // Mesmas entradas, mesmas cores (o filtro da tela não repinta as séries).
    expect(entityColors(order, active, 'dark').get('pets')).toBe(c.get('pets'));
  });

  it('o Mono (um só matiz) tem só 4 cores', () => {
    expect(entityColors(order, null, 'mono').keys).toHaveLength(4);
  });
});

describe('drill-down', () => {
  const period = { from: '2026-10-01', to: '2026-10-31' };

  it('categoria, sem categoria e "Outras"', () => {
    expect(txnLink(categoryDrill({ period, basis: 'cash' }, ['cat-moradia']))).toBe(
      '/lancamentos?from=2026-10-01&to=2026-10-31&basis=cash&category=cat-moradia&kind=expense',
    );
    expect(linkFilter(categoryDrill({ period }, [null])).uncategorized).toBe(true);
    expect(linkFilter(categoryDrill({ period }, ['a', 'b'], 'income'))).toMatchObject({
      category_ids: ['a', 'b'],
      kinds: ['income'],
    });
    expect(
      linkFilter(categoryDrill({ period, memberIds: ['m1'], accountIds: ['acc'] }, ['a'])),
    ).toMatchObject({ member_ids: ['m1'], account_ids: ['acc'] });
  });

  it('dia, mês, titular, recorrência, cartão, sem categoria e vencimentos', () => {
    expect(txnLink(dayDrill('2026-10-08'))).toBe(
      '/lancamentos?from=2026-10-08&to=2026-10-08&kind=expense',
    );
    expect(linkFilter(monthDrill('2026-02', 'accrual', ['x']))).toMatchObject({
      date_from: '2026-02-01',
      date_to: '2026-02-28',
      category_ids: ['x'],
    });
    expect(linkFilter(memberDrill({ period }, 'mem-ana', 'cat-lazer'))).toMatchObject({
      member_ids: ['mem-ana'],
      category_ids: ['cat-lazer'],
    });
    expect(txnLink(recurrenceDrill('Netflix', 'acc-nucard'))).toBe(
      '/lancamentos?q=Netflix&account=acc-nucard',
    );
    expect(linkFilter(cardMonthDrill('acc-nucard', '2026-11'))).toMatchObject({
      basis: 'cash',
      account_ids: ['acc-nucard'],
      date_from: '2026-11-01',
    });
    expect(txnLink(uncategorizedDrill(period))).toBe(
      '/lancamentos?from=2026-10-01&to=2026-10-31&uncategorized=1',
    );
    expect(
      txnLink(
        upcomingDrill({ kind: 'card_due', ref_id: 'st-1', account_id: 'c', date: '2026-11-10' }),
      ),
    ).toBe('/lancamentos?statement=st-1');
    expect(
      linkFilter(
        upcomingDrill({ kind: 'planned_txn', ref_id: 't', account_id: 'acc', date: '2026-10-20' }),
      ),
    ).toMatchObject({ statuses: ['planned'], account_ids: ['acc'], date_from: '2026-10-20' });
  });
});

describe('Sankey → gráfico', () => {
  const data: SankeyData = {
    from: '2026-10-01',
    to: '2026-10-31',
    basis: 'accrual',
    currency: 'BRL',
    missing_fx: [],
    estimated: false,
    card_bills_paid: 0,
    total_income: 1000,
    total_expense: 800,
    nodes: [
      { id: 'income:sal', name: 'Salário', kind: 'income', category_id: 'sal' },
      { id: 'hub', name: 'Orçamento', kind: 'hub', category_id: null },
      { id: 'expense:mor', name: 'Moradia', kind: 'expense', category_id: 'mor' },
      { id: 'expense:pets', name: 'Pets', kind: 'expense', category_id: 'pets' },
      { id: 'expense:none', name: 'Sem categoria', kind: 'expense', category_id: null },
      { id: 'savings', name: 'Poupança', kind: 'savings', category_id: null },
      { id: 'sub:alug', name: 'Aluguel', kind: 'subcategory', category_id: 'alug' },
      { id: 'sub:racao', name: 'Ração', kind: 'subcategory', category_id: 'racao' },
    ],
    links: [
      { source: 'income:sal', target: 'hub', value: 1000 },
      { source: 'hub', target: 'expense:mor', value: 500 },
      { source: 'hub', target: 'expense:pets', value: 200 },
      { source: 'hub', target: 'expense:none', value: 100 },
      { source: 'hub', target: 'savings', value: 200 },
      { source: 'expense:mor', target: 'sub:alug', value: 500 },
      { source: 'expense:pets', target: 'sub:racao', value: 200 },
    ],
    depth: 2,
    totals: {
      from: '2026-10-01',
      to: '2026-10-31',
      income: 1000,
      expense: 800,
      debt_payments: 0,
      investments: 0,
      redemptions: 0,
      leftover: 200,
      savings_rate: 0.2,
    },
    previous: null,
    comparison: [],
  };

  it('categorias sem cor viram "Outras" (e as subcategorias delas saem)', () => {
    const colors = entityColors(['mor'], null, 'dark');
    const m = flowModel(data, colors);
    const ids = m.input.nodes.map((n) => n.id);
    expect(ids).toContain(`expense:${OTHERS_KEY}`);
    expect(ids).not.toContain('expense:pets');
    expect(ids).not.toContain('sub:racao');
    expect(m.folded).toEqual(['pets']);
    expect(m.input.links.find((l) => l.target === `expense:${OTHERS_KEY}`)?.value).toBe(200);
    // Conservação no orçamento: o que entra = o que sai.
    const into = m.input.links.filter((l) => l.target === 'hub').reduce((s, l) => s + l.value, 0);
    const out = m.input.links.filter((l) => l.source === 'hub').reduce((s, l) => s + l.value, 0);
    expect(into).toBe(out);
  });

  it('cores e drill-down de cada nó', () => {
    const m = flowModel(data, entityColors(['mor'], null, 'dark'));
    const byId = new Map(m.input.nodes.map((n) => [n.id, n]));
    expect(byId.get('income:sal')!.color).toBe('@positive');
    expect(byId.get('savings')!.color).toBe('@accent');
    expect(byId.get('sub:alug')!.color).toBe(categoricalPalette('dark')[0]);
    expect(m.drill.get('income:sal')).toEqual({ kind: 'income', categories: ['sal'] });
    expect(m.drill.get('expense:none')).toEqual({ kind: 'expense', categories: [null] });
    expect(m.drill.get(`expense:${OTHERS_KEY}`)).toEqual({ kind: 'expense', categories: ['pets'] });
    expect(m.drill.has('hub')).toBe(false);
  });

  it('dívidas, aportes e a comparação com o período anterior em cada nó', () => {
    const delta = (id: string, current: number, previous: number) => ({
      id,
      name: id,
      kind: 'expense' as const,
      category_id: null,
      parent_id: null,
      current,
      previous,
      delta: current - previous,
      delta_pct: previous > 0 ? (current - previous) / previous : null,
      share_of_income: current / 1000,
    });
    const m = flowModel(
      {
        ...data,
        nodes: [
          ...data.nodes,
          { id: 'debt', name: 'Pagamento de dívidas', kind: 'debt', category_id: null },
          { id: 'debt:fin', name: 'Apartamento', kind: 'subcategory', category_id: 'fin' },
          { id: 'debt:cons', name: 'Consignado', kind: 'subcategory', category_id: 'cons' },
          { id: 'investment', name: 'Investimentos', kind: 'investment', category_id: null },
        ],
        links: [
          ...data.links,
          { source: 'debt', target: 'debt:fin', value: 60 },
          { source: 'debt', target: 'debt:cons', value: 40 },
        ],
        comparison: [delta('expense:mor', 500, 400), delta('expense:pets', 200, 0)],
      },
      entityColors(['mor'], null, 'dark'),
    );
    const byId = new Map(m.input.nodes.map((n) => [n.id, n]));
    expect(byId.get('debt')!.color).toBe('@warning');
    expect(byId.get('debt:fin')!.color).toBe('@warning');
    expect(byId.get('debt')!.depth).toBe(2);
    expect(byId.get('debt:fin')!.depth).toBe(3);
    expect(byId.get('investment')!.depth).toBe(2);
    expect(m.drill.get('debt')).toEqual({ kind: 'expense', categories: ['fin', 'cons'] });
    expect(m.drill.has('investment')).toBe(false);
    expect(byId.get('expense:mor')!.compare).toEqual({ previous: 400, delta: 100, deltaPct: 0.25 });
    // "Outras" soma a comparação das categorias reunidas.
    expect(byId.get(`expense:${OTHERS_KEY}`)!.compare).toEqual({
      previous: 0,
      delta: 200,
      deltaPct: null,
    });
    expect(m.input.shareLabel).toBe('Da receita');
    expect(m.input.total).toBe(1000);
  });
});

describe('composição', () => {
  const categories = [
    cat('mor', 'Moradia', null, 0),
    cat('alug', 'Aluguel', 'mor'),
    cat('pets', 'Pets', null, 1),
    cat('racao', 'Ração', 'pets'),
  ];
  const summary: SpendingSummary = {
    from: '2026-10-01',
    to: '2026-10-31',
    basis: 'accrual',
    kind: 'expense',
    currency: 'BRL',
    total: 1000,
    count: 4,
    missing_fx: [],
    estimated: false,
    groups: [
      {
        key: 'alug',
        label: 'Aluguel',
        parent_key: 'mor',
        path: null,
        amount: 600,
        share: 0.6,
        count: 1,
        color: null,
      },
      {
        key: 'mor',
        label: 'Moradia',
        parent_key: null,
        path: null,
        amount: 100,
        share: 0.1,
        count: 1,
        color: null,
      },
      {
        key: 'racao',
        label: 'Ração',
        parent_key: 'pets',
        path: null,
        amount: 200,
        share: 0.2,
        count: 1,
        color: null,
      },
      {
        key: null,
        label: 'Sem categoria',
        parent_key: null,
        path: null,
        amount: 100,
        share: 0.1,
        count: 1,
        color: null,
      },
    ],
  };

  it('árvore com "(direto)", "Outras" e "Sem categoria"', () => {
    const m = compositionModel(summary, categories, entityColors(['mor'], null, 'dark'));
    expect(m.nodes.map((n) => n.name)).toEqual(['Moradia', 'Outras', 'Sem categoria']);
    expect(m.nodes[0]!.value).toBe(700);
    expect(m.nodes[0]!.children!.map((c) => c.name)).toEqual(['Aluguel', 'Moradia (direto)']);
    expect(m.nodes[1]!.children!.map((c) => c.name)).toEqual(['Pets']);
    expect(m.drill.get('mor:direto')).toEqual(['mor']);
    expect(m.drill.get(OTHERS_KEY)).toEqual(['pets']);
    expect(m.drill.get('__sem_categoria__')).toEqual([null]);
  });
});

describe('tendência', () => {
  const evo: MonthlyEvolution = {
    months: ['2026-09', '2026-10'],
    basis: 'accrual',
    currency: 'BRL',
    missing_fx: [],
    estimated: false,
    series: [
      // O núcleo ordena pelo gasto; as barras seguem a ordem fixa das cores.
      { key: 'b', label: 'B', color: null, values: [50, 60] },
      { key: 'a', label: 'A', color: null, values: [10, 20] },
      { key: 'z', label: 'Z', color: null, values: [5, 5] },
      { key: null, label: 'Sem categoria', color: null, values: [1, 1] },
    ],
    totals: [66, 86],
    moving_avg_3: [60, 76],
    income: [100, 100],
  };

  it('barras na ordem fixa das entidades, o resto em "Outras"', () => {
    const m = trendModel(evo, entityColors(['a', 'b'], null, 'dark'));
    expect(m.input.bars.map((b) => b.key)).toEqual(['a', 'b', OTHERS_KEY]);
    expect(m.input.bars[2]!.values).toEqual([6, 6]);
    expect(m.drill.get(OTHERS_KEY)).toEqual(['z']);
    expect(m.input.lines.map((l) => l.key)).toEqual(['avg', 'income']);
    expect(m.trend).toBeCloseTo(76 / 60 - 1);
  });
});

describe('comparação, titulares, recorrências, cartões e patrimônio', () => {
  it('comparação: cores por entidade e slope com "Outras"', () => {
    const colors = entityColors(['a'], null, 'dark');
    const rows = comparisonRows(
      {
        a: { from: '', to: '' },
        b: { from: '', to: '' },
        basis: 'accrual',
        kind: 'expense',
        currency: 'BRL',
        missing_fx: [],
        estimated: false,
        total_a: 0,
        total_b: 0,
        rows: [
          { key: 'a', label: 'A', a: 10, b: 20, delta: 10, delta_ratio: 1, color: null },
          { key: 'x', label: 'X', a: 5, b: 1, delta: -4, delta_ratio: -0.8, color: null },
          { key: null, label: 'Sem categoria', a: 1, b: 2, delta: 1, delta_ratio: 1, color: null },
        ],
      },
      colors,
    );
    expect(rows[0]!.color).toBe(categoricalPalette('dark')[0]);
    expect(rows[1]!.color).toBe('@gray');
    expect(rows[2]!.key).toBe('__sem_categoria__');
    const slope = slopeRows(rows, colors);
    expect(slope.map((r) => r.key)).toEqual(['a', OTHERS_KEY]);
    expect(slope[1]).toMatchObject({ a: 6, b: 3 });
  });

  it('titulares: até 10 categorias e o resto em "Outras"', () => {
    const categories = Array.from({ length: 12 }, (_, i) => ({
      key: `c${i}`,
      label: `C${i}`,
      color: null,
      values: [10, 1],
    }));
    const { input, folded } = membersModel(
      {
        from: '',
        to: '',
        basis: 'accrual',
        currency: 'BRL',
        missing_fx: [],
        estimated: false,
        total: 132,
        members: [
          {
            key: 'ana',
            label: 'Ana',
            parent_key: null,
            path: null,
            amount: 120,
            share: 0.9,
            count: 1,
            color: null,
          },
          {
            key: null,
            label: 'Sem membro',
            parent_key: null,
            path: null,
            amount: 12,
            share: 0.1,
            count: 1,
            color: null,
          },
        ],
        categories,
      },
      entityColors(['ana', '__sem__'], null, 'dark'),
      '__sem__',
    );
    expect(input.rows).toHaveLength(11);
    expect(input.rows[10]!.key).toBe(OTHERS_KEY);
    expect(input.groups[0]!.values[10]).toBe(20);
    expect(input.groups[1]!.key).toBe('__sem__');
    expect(folded).toEqual(['c10', 'c11']);
  });

  it('recorrências: comprometido por mês e linha do tempo', () => {
    const rec = (over: Partial<Recurrence>): Recurrence => ({
      id: 'r',
      description: 'Netflix',
      account_id: 'acc',
      account_name: 'Cartão',
      amount: -5590,
      currency: 'BRL',
      kind: 'expense',
      category_id: 'stream',
      member_id: null,
      to_account_id: null,
      rrule: 'FREQ=MONTHLY;BYMONTHDAY=17',
      spec: { freq: 'MONTHLY', by_month_day: 17 },
      human: 'todo mês no dia 17',
      start_date: '2025-01-17',
      end_date: null,
      active: true,
      next_dates: ['2026-10-17', '2026-11-17', '2026-12-17'],
      monthly_equivalent: -5590,
      ...over,
    });
    const list = [
      rec({}),
      rec({ id: 's', kind: 'income', amount: 100_000, monthly_equivalent: 100_000 }),
      rec({ id: 'p', active: false, monthly_equivalent: -99_999 }),
    ];
    const s = recurrenceSummary(list);
    expect(s).toMatchObject({ expenseMonthly: 5590, incomeMonthly: 100_000, count: 1 });
    expect(s.committedShare).toBeCloseTo(0.0559);
    const rows = timelineRows(
      list,
      [cat('assin', 'Assinaturas'), cat('stream', 'Streaming', 'assin')],
      entityColors(['assin'], null, 'dark'),
      { from: TODAY, to: '2027-01-14' },
    );
    expect(rows).toHaveLength(1);
    expect(rows[0]!.dates).toEqual(['2026-10-17', '2026-11-17', '2026-12-17']);
    expect(rows[0]!.color).toBe(categoricalPalette('dark')[0]);
    expect(rows[0]!.detail).toContain('Assinaturas');
  });

  it('cartões: meses unidos, empilhados por cartão, com total', () => {
    const m = cardsModel(
      [
        {
          id: 'c1',
          name: 'Nubank',
          data: [
            { month: '2026-10', amount: 0 },
            { month: '2026-11', amount: 500 },
          ],
        },
        {
          id: 'c2',
          name: 'Itaú',
          data: [
            { month: '2026-11', amount: 300 },
            { month: '2026-12', amount: 100 },
          ],
        },
      ],
      entityColors(['c1', 'c2'], null, 'dark'),
      'BRL',
    );
    expect(m.months).toEqual(['2026-10', '2026-11', '2026-12']);
    expect(m.totals).toEqual([0, 800, 100]);
    expect(m.total).toBe(900);
    expect(m.series.every((s) => s.stack === 'faturas')).toBe(true);
  });

  it('patrimônio: contas = ativos − investimentos − bens; dívidas abaixo do zero', () => {
    const m = netWorthModel(
      [
        {
          month: '2026-10',
          as_of: '2026-10-31',
          assets: 1000,
          liabilities: 200,
          net: 800,
          investments: 300,
          physical: 150,
          debts: 0,
          missing_fx: [],
          estimated: false,
        },
      ],
      'BRL',
    );
    expect(m.series.map((s) => [s.key, s.values[0]])).toEqual([
      ['accounts', 550],
      ['investments', 300],
      ['physical', 150],
      ['liabilities', -200],
      ['net', 800],
    ]);
  });
});
