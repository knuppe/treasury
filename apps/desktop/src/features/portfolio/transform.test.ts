import { describe, expect, it } from 'vitest';

import type { ClassSummary } from '@/api/generated/ClassSummary';
import type { CorporateAction } from '@/api/generated/CorporateAction';
import type { DividendCalendarItem } from '@/api/generated/DividendCalendarItem';
import type { Performance } from '@/api/generated/Performance';
import type { PositionView } from '@/api/generated/PositionView';
import type { Trade } from '@/api/generated/Trade';
import { categoricalPalette } from '@/charts';

import {
  agendaByMonth,
  allocationSlices,
  bridgeSteps,
  capitalGap,
  classColors,
  groupPositions,
  indexChartValues,
  meterScale,
  monthlyReturns,
  passiveIncomeBars,
  performanceLines,
  priceChartData,
} from './transform';

function cls(
  c: ClassSummary['class'],
  value: number,
  extra: Partial<ClassSummary> = {},
): ClassSummary {
  return {
    class: c,
    label: c,
    market_value_base: value,
    cost_base: value,
    weight: 0,
    target_pct: null,
    deviation_pp: null,
    out_of_band: false,
    xirr: null,
    ...extra,
  };
}

function trade(t: Partial<Trade> & Pick<Trade, 'date' | 'side' | 'quantity' | 'price'>): Trade {
  return {
    id: `t-${t.date}-${t.side}`,
    asset_id: 'a',
    ticker: 'BBAS3',
    asset_name: 'Banco do Brasil ON',
    by_value: false,
    account_id: 'xp',
    account_name: 'XP',
    fees: 0,
    taxes: 0,
    currency: 'BRL',
    fx_rate: null,
    day_trade: false,
    gross: 0,
    cash_amount: 0,
    txn_id: null,
    note_number: null,
    notes: null,
    import_id: null,
    ...t,
  };
}

describe('colors by class', () => {
  it('follow the core fixed order, not the size, and gather the excess into "Outras"', () => {
    const pal = categoricalPalette('dark');
    const { colors, grouped } = classColors(['crypto', 'stock_br', 'fii'], 'dark');
    expect(colors.get('stock_br')).toBe(pal[0]);
    expect(colors.get('fii')).toBe(pal[1]);
    expect(colors.get('crypto')).toBe(pal[2]);
    expect(grouped.size).toBe(0);

    const mono = classColors(
      ['stock_br', 'fii', 'etf_br', 'bdr', 'fixed_income', 'crypto'],
      'mono',
    );
    expect([...mono.grouped]).toEqual(['fixed_income', 'crypto']);
  });
});

describe('allocation slices', () => {
  it('sorts by value, ignores zeroed classes and puts "Outras" last', () => {
    const byClass = [
      cls('stock_br', 300, { weight: 0.3, target_pct: 25, deviation_pp: 5, out_of_band: false }),
      cls('fii', 100, { weight: 0.1 }),
      cls('fixed_income', 500, { weight: 0.5 }),
      cls('crypto', 100, { weight: 0.1, target_pct: 5 }),
      cls('etf_us', 0, { target_pct: 5 }),
    ];
    const slices = allocationSlices(byClass, 'mono', '#999');
    // Mono has 4 colors; the classes with value (stock_br, fii, fixed_income,
    // crypto) all fit.
    expect(slices.map((s) => s.key)).toEqual(['fixed_income', 'stock_br', 'fii', 'crypto']);
    expect(slices.find((s) => s.key === 'stock_br')?.deviationPp).toBe(5);

    const many = allocationSlices(
      [...byClass, cls('etf_br', 50, { weight: 0.05, target_pct: 5 })],
      'mono',
      '#999',
    );
    const others = many.at(-1)!;
    expect(others.key).toBe('outras');
    expect(others.members).toEqual(['crypto']);
    expect(others.value).toBe(100);
    expect(others.color).toBe('#999');
    expect(others.deviationPp).toBeCloseTo(5);
  });
});

const perf: Performance = {
  months: ['2026-01', '2026-02', '2026-03'],
  monthly: [0.01, null, -0.02],
  cumulative: [0.01, 0.01, -0.0102],
  cumulative_real: [0.005, null, -0.02],
  cdi_cumulative: [0.011, 0.022, null],
  benchmark_label: 'CDI',
  ibov_cumulative: [null, null, null],
  sp500_brl_cumulative: [null, null, null],
  usd_cumulative: [null, null, null],
  inflation_cumulative: [0.003, 0.008, 0.012],
  deflator: 'ipca',
  value_base: [100, 200, 300],
  net_contributions: [100, 200, 300],
  estimated: false,
  method: 'monthly',
  risk: null,
};

describe('profitability', () => {
  it('builds portfolio, CDI, inflation and real in % on the same axis', () => {
    const { labels, series } = performanceLines(perf);
    expect(labels).toEqual(['jan/26', 'fev/26', 'mar/26']);
    expect(series.map((s) => s.key)).toEqual(['carteira', 'cdi', 'inflacao', 'real']);
    expect(series[0]!.values).toEqual([1, 1, -1.02]);
    expect(series[1]!.values[2]).toBeNull();
    expect(series[2]!.name).toBe('IPCA');
    expect(series[3]!.dashed).toBe(true);
  });

  it('without a deflator, it shows neither inflation nor real', () => {
    const { series } = performanceLines({ ...perf, deflator: 'none' });
    expect(series.map((s) => s.key)).toEqual(['carteira', 'cdi']);
  });

  it('includes Ibovespa and S&P 500 in reais when the core has the data', () => {
    const { series } = performanceLines({
      ...perf,
      ibov_cumulative: [0.1, -0.01, null],
      sp500_brl_cumulative: [0.02, 0.03, 0.04],
    });
    expect(series.map((s) => s.key).slice(-2)).toEqual(['ibov', 'sp500']);
    expect(series.at(-2)!.values).toEqual([10, -1, null]);
    expect(series.at(-1)!.name).toBe('S&P 500 (em R$)');
  });

  it('monthly returns in %', () => {
    expect(monthlyReturns(perf).values).toEqual([1, null, -2]);
  });
});

describe('passive income', () => {
  it('bars per month and how much is left to the goal', () => {
    const bars = passiveIncomeBars({
      last_12m: 300,
      monthly_avg: 25,
      by_month: [
        { month: '2026-09', amount: 100 },
        { month: '2026-10', amount: 200 },
      ],
      by_asset: [],
      target_monthly: null,
      capital_needed: null,
      portfolio_yield_12m: null,
    });
    expect(bars.labels).toEqual(['set/26', 'out/26']);
    expect(bars.values).toEqual([100, 200]);
    expect(capitalGap(1_000_000, 400_000)).toBe(600_000);
    expect(capitalGap(1_000_000, 2_000_000)).toBe(0);
    expect(capitalGap(null, 1)).toBeNull();
  });
});

describe('price chart', () => {
  const prices = [
    { date: '2024-01-15', close: '55', source: 'yahoo' },
    { date: '2024-04-15', close: '58', source: 'yahoo' },
    { date: '2024-04-16', close: '29', source: 'yahoo' },
    { date: '2025-03-14', close: '27', source: 'yahoo' },
    { date: '2025-03-17', close: '27.5', source: 'yahoo' },
    { date: '2026-01-20', close: '24', source: 'yahoo' },
  ];
  const trades = [
    trade({ date: '2024-01-15', side: 'buy', quantity: '100', price: '55', fees: 1000 }),
    trade({ date: '2025-03-15', side: 'buy', quantity: '100', price: '27' }),
    trade({ date: '2026-01-20', side: 'sell', quantity: '50', price: '24' }),
  ];
  const split: CorporateAction = {
    id: 's',
    asset_id: 'a',
    date: '2024-04-16',
    kind: 'split',
    factor: '2',
    unit_cost: null,
    amount: null,
    new_asset_id: null,
    notes: null,
    source: 'yahoo',
  };

  it('marks buys and sells on the trading day (or the next one with a quote)', () => {
    const d = priceChartData(prices, trades, [split]);
    expect(d.hasPrices).toBe(true);
    expect(d.line).toHaveLength(6);
    expect(d.markers.map((m) => [m.time, m.side, m.text])).toEqual([
      ['2024-01-15', 'buy', 'C 100'],
      ['2025-03-17', 'buy', 'C 100'],
      ['2026-01-20', 'sell', 'V 50'],
    ]);
  });

  it('reproduces the average price with costs, split and sale', () => {
    const d = priceChartData(prices, trades, [split]);
    // Buy: (100 × 55 + 10) / 100 = 55.10; 1:2 split → 27.55; buy of
    // 100 at 27 → (5,510 + 2,700) / 300 = 27.3667; the sale does not change the average.
    expect(d.avg.map((p) => p.time)).toEqual([
      '2024-01-15',
      '2024-04-16',
      '2025-03-15',
      '2026-01-20',
    ]);
    expect(d.avg[0]!.value).toBeCloseTo(55.1);
    expect(d.avg[1]!.value).toBeCloseTo(27.55);
    expect(d.avg[2]!.value).toBeCloseTo(27.3667, 3);
    expect(d.avg[3]!.value).toBeCloseTo(27.3667, 3);
  });

  it('clips markers and average price to the quotes window', () => {
    const d = priceChartData(prices.slice(3), trades, [split]);
    expect(d.markers.map((m) => m.time)).toEqual(['2025-03-17', '2026-01-20']);
    expect(d.avg[0]!.time).toBe('2025-03-14');
    expect(d.avg[0]!.value).toBeCloseTo(27.55);
    expect(d.avg[1]!.time).toBe('2025-03-15');
  });

  it('without quotes, draws the operation prices', () => {
    const d = priceChartData([], trades, [split]);
    expect(d.hasPrices).toBe(false);
    expect(d.line.map((p) => p.value)).toEqual([55, 27, 24]);
    expect(d.markers).toHaveLength(3);
  });
});

function pos(
  id: string,
  cls2: PositionView['asset']['class'],
  value: number,
  extra: Partial<PositionView> = {},
): PositionView {
  return {
    asset: { id, ticker: id, name: id, class: cls2, class_label: cls2, currency: 'BRL' },
    account_id: null,
    quantity: '1',
    avg_price: '1',
    cost: value,
    cost_base: value,
    price: null,
    market_value: value,
    market_value_base: value,
    unrealized_base: 0,
    realized_base: 0,
    income_base: 0,
    total_result_base: 10,
    real_result_base: 5,
    inflation_effect_base: 5,
    xirr: 0.1,
    xirr_real: 0.05,
    holding_days: 400,
    period_return: 0.1,
    period_return_real: 0.05,
    return_basis: 'annualized',
    weight: 0.25,
    valuation: 'market',
    net_redemption_base: null,
    first_date: null,
    estimated: false,
    warnings: [],
    ...extra,
  };
}

describe('positions', () => {
  it('groups by class in the fixed order and sums value, cost and result', () => {
    const groups = groupPositions(
      [
        pos('HGLG11', 'fii', 100),
        pos('PETR4', 'stock_br', 50),
        pos('ITUB4', 'stock_br', 70, { total_result_base: null }),
      ],
      'class',
    );
    expect(groups.map((g) => g.key)).toEqual(['stock_br', 'fii']);
    expect(groups[0]!.value).toBe(120);
    expect(groups[0]!.result).toBeNull();
    expect(groups[1]!.result).toBe(10);
  });

  it('groups by account with the account name', () => {
    const groups = groupPositions(
      [pos('A', 'fii', 10, { account_id: 'xp' }), pos('B', 'fii', 30, { account_id: 'inter' })],
      'account',
      [
        { id: 'xp', name: 'XP' },
        { id: 'inter', name: 'Inter' },
      ],
    );
    expect(groups.map((g) => g.label)).toEqual(['Inter', 'XP']);
  });

  it('gauge scale nominal × real', () => {
    expect(meterScale([0.1, -0.3, null, 0.2])).toBe(0.3);
    expect(meterScale([null])).toBe(0.1);
  });
});

describe('schedule and indices', () => {
  it('groups the schedule by payment month and sums the forecast by currency', () => {
    const base = {
      asset: {
        id: 'a',
        ticker: 'A',
        name: 'A',
        class: 'fii' as const,
        class_label: 'FIIs',
        currency: 'BRL',
      },
      kind: 'fii_income' as const,
      ex_date: null,
      amount_per_share: '1',
      quantity: '10',
      estimated: false,
      source: 'yahoo',
    };
    const items: DividendCalendarItem[] = [
      { ...base, pay_date: '2026-11-14', currency: 'BRL', expected_net: 1000 },
      { ...base, pay_date: '2026-10-14', currency: 'BRL', expected_net: 500 },
      { ...base, pay_date: '2026-10-20', currency: 'USD', expected_net: 300 },
      { ...base, pay_date: '2026-10-21', currency: 'BRL', expected_net: null },
    ];
    const months = agendaByMonth(items);
    expect(months.map((m) => m.month)).toEqual(['2026-10', '2026-11']);
    expect(months[0]!.totals.get('BRL')).toBe(500);
    expect(months[0]!.totals.get('USD')).toBe(300);
  });

  it('annualizes daily rates (CDI) and keeps levels', () => {
    const daily = indexChartValues('daily_rate', [
      { date: '2026-10-01', value: '0.050788', estimated: false },
    ]);
    expect(daily.values[0]).toBeCloseTo(13.65, 1);
    expect(daily.unit).toMatch(/a\.a\./);
    const level = indexChartValues('level', [
      { date: '2026-10-01', value: '146800.5', estimated: true },
    ]);
    expect(level.values).toEqual([146800.5]);
    expect(level.estimated).toEqual([true]);
  });
});

describe('result bridge', () => {
  it('the waterfall closes at the end value, with the outflows going down', () => {
    const steps = bridgeSteps({
      start_value: 100_000,
      price_gain: -30_000,
      fx_gain: -5_000,
      income: 2_000,
      fees: 500,
      taxes: 300,
      contributions: 50_000,
      withdrawals: 10_000,
      end_value: 111_200,
      delta: -28_800,
    });
    expect(steps.map((s) => s.key)).toEqual([
      'inicio',
      'preco',
      'proventos',
      'taxas',
      'impostos',
      'aportes',
      'retiradas',
      'fim',
    ]);
    const moves = steps.filter((s) => !s.total).reduce((a, s) => a + s.amount, 0);
    expect(steps[0]!.amount + moves).toBe(steps.at(-1)!.amount);
    expect(steps[1]).toMatchObject({ amount: -30_000, low: 70_000, high: 100_000 });
    expect(steps[3]).toMatchObject({ amount: -500 });
    expect(steps.at(-2)).toMatchObject({ low: 111_200, high: 121_200 });
  });

  it('from zero (since the start), the price drop crosses zero', () => {
    const steps = bridgeSteps({
      start_value: 0,
      price_gain: -1_000,
      fx_gain: 0,
      income: 0,
      fees: 0,
      taxes: 0,
      contributions: 5_000,
      withdrawals: 0,
      end_value: 4_000,
      delta: -1_000,
    });
    expect(steps[1]).toMatchObject({ low: -1_000, high: 0 });
    expect(steps[5]).toMatchObject({ low: -1_000, high: 4_000 });
  });
});
