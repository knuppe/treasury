import { describe, expect, it } from 'vitest';

import {
  annual,
  compound,
  impliedRate,
  installments,
  liveOnIncome,
  monthly,
  rentVsBuy,
  type RentBuyInput,
} from './math';

describe('calculators: math', () => {
  it('equivalent rate, not division by 12', () => {
    expect(monthly(0.12)).toBeCloseTo(0.009489, 6);
    expect(annual(monthly(0.12))).toBeCloseTo(0.12, 12);
  });

  it('compound interest: contribution at month end, inflation and management fee', () => {
    // Without contributions: 1,000 at 10% p.a. for 2 years = 1,210.
    const a = compound({
      initial: 1000,
      contribution: 0,
      rate: 0.1,
      months: 24,
      inflation: 0,
      fee: 0,
    });
    expect(a.balance).toBeCloseTo(1210, 6);
    expect(a.interest).toBeCloseTo(210, 6);
    expect(a.years.map((y) => y.year)).toEqual([1, 2]);
    // Contributions only, zero rate: simple sum.
    const b = compound({
      initial: 0,
      contribution: 100,
      rate: 0,
      months: 12,
      inflation: 0,
      fee: 0,
    });
    expect(b.balance).toBeCloseTo(1200, 9);
    // Annuity: 100 per month at 1% p.m. for 12 months = 100 × ((1.01^12 − 1)/0.01).
    const c = compound({
      initial: 0,
      contribution: 100,
      rate: annual(0.01),
      months: 12,
      inflation: 0,
      fee: 0,
    });
    expect(c.balance).toBeCloseTo(100 * ((1.01 ** 12 - 1) / 0.01), 6);
    // Inflation of 10% p.a. over 2 years: 1,210 is worth 1,000 today.
    const d = compound({
      initial: 1000,
      contribution: 0,
      rate: 0.1,
      months: 24,
      inflation: 0.1,
      fee: 0,
    });
    expect(d.real).toBeCloseTo(1000, 6);
    // Management fee reduces the balance and the cost shows up.
    const e = compound({
      initial: 10000,
      contribution: 0,
      rate: 0.1,
      months: 120,
      inflation: 0,
      fee: 0.02,
    });
    expect(e.balance).toBeLessThan(10000 * 1.1 ** 10);
    expect(e.feeCost).toBeCloseTo(10000 * 1.1 ** 10 - e.balance, 6);
    expect(e.feeCost).toBeGreaterThan(0);
  });

  it('living off income: perpetual wealth, coverage and how long it lasts', () => {
    const r = liveOnIncome({ spending: 5000, realRate: annual(0.005), wealth: 0 });
    // 0.5% real per month: 5,000 / 0.005 = 1 million.
    expect(r.needed).toBeCloseTo(1_000_000, 3);
    expect(r.coverage).toBe(0);
    expect(r.lastsMonths).toBe(0);
    const half = liveOnIncome({ spending: 5000, realRate: annual(0.005), wealth: 500_000 });
    expect(half.coverage).toBeCloseTo(0.5, 9);
    // 500 thousand withdrawing 5 thousand at 0.5% p.m.: n = ln(5000/2500)/ln(1.005) ≈ 139 months.
    expect(half.lastsMonths).toBeCloseTo(Math.log(2) / Math.log(1.005), 6);
    expect(
      liveOnIncome({ spending: 5000, realRate: annual(0.005), wealth: 2_000_000 }).lastsMonths,
    ).toBeNull();
    // No yield: lasts wealth ÷ spending.
    expect(liveOnIncome({ spending: 1000, realRate: 0, wealth: 12000 }).lastsMonths).toBe(12);
  });

  it('cash × installments: finds the interest hidden in the installment', () => {
    // 10 × 110 against 1,000 in cash, first one in 30 days: the rate that zeroes the flow.
    const i = impliedRate(1000, 110, 10, false)!;
    let pv = 0;
    for (let k = 1; k <= 10; k++) pv += 110 / (1 + i) ** k;
    expect(pv).toBeCloseTo(1000, 6);
    expect(i).toBeGreaterThan(0.017);
    // A truly "interest-free" plan: installments add up to the cash price.
    expect(impliedRate(1000, 100, 10, false)).toBeNull();
    // First installment today: higher rate for the same amount.
    expect(impliedRate(1000, 110, 10, true)!).toBeGreaterThan(i);
    const r = installments({
      cash: 900,
      installment: 100,
      count: 10,
      firstToday: false,
      opportunity: 0,
    });
    // With no yield, the installments are worth 1,000 today: cash comes out 100 cheaper.
    expect(r.total).toBe(1000);
    expect(r.advantageCash).toBeCloseTo(100, 9);
    expect(r.hiddenAnnual).toBeCloseTo(annual(r.hiddenMonthly!), 12);
    // A high enough yield makes installments worth less than cash today.
    const high = installments({
      cash: 990,
      installment: 100,
      count: 10,
      firstToday: false,
      opportunity: 0.3,
    });
    expect(high.advantageCash).toBeLessThan(0);
  });

  it('rent × buy: Price and SAC, same spending on both paths', () => {
    const base: RentBuyInput = {
      price: 500_000,
      downPayment: 100_000,
      buyCosts: 0.04,
      loanRate: 0.1,
      loanMonths: 360,
      system: 'price',
      rent: 2500,
      rentGrowth: 0,
      appreciation: 0,
      ownerCosts: 0,
      investReturn: 0,
      inflation: 0,
      months: 360,
    };
    const price = rentVsBuy(base);
    expect(price.loanBalance).toBeCloseTo(0, 3);
    // No yield or appreciation: whoever buys keeps the property; whoever rents keeps the
    // down payment, the costs and the month-by-month differences.
    expect(price.buyWealth).toBeCloseTo(500_000, 3);
    expect(price.rentWealth).toBeCloseTo(
      100_000 + 20_000 + 360 * price.firstInstallment - 360 * 2500,
      0,
    );
    const sac = rentVsBuy({ ...base, system: 'sac' });
    expect(sac.firstInstallment).toBeGreaterThan(price.firstInstallment);
    expect(sac.interestPaid).toBeLessThan(price.interestPaid);
    // Horizon shorter than the financing: an outstanding balance is left.
    const short = rentVsBuy({ ...base, months: 60 });
    expect(short.loanBalance).toBeGreaterThan(0);
    expect(short.buyWealth).toBeCloseTo(500_000 - short.loanBalance, 3);
    // Today's values with inflation.
    const inf = rentVsBuy({ ...base, inflation: 0.05 });
    expect(inf.buyWealthReal).toBeCloseTo(inf.buyWealth / 1.05 ** 30, 3);
  });
});
