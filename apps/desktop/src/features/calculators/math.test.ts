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

describe('calculadoras: contas', () => {
  it('taxa equivalente, não divisão por 12', () => {
    expect(monthly(0.12)).toBeCloseTo(0.009489, 6);
    expect(annual(monthly(0.12))).toBeCloseTo(0.12, 12);
  });

  it('juros compostos: aporte no fim do mês, inflação e taxa de administração', () => {
    // Sem aporte: 1.000 a 10% a.a. por 2 anos = 1.210.
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
    // Só aportes, taxa zero: soma simples.
    const b = compound({
      initial: 0,
      contribution: 100,
      rate: 0,
      months: 12,
      inflation: 0,
      fee: 0,
    });
    expect(b.balance).toBeCloseTo(1200, 9);
    // Anuidade: 100 por mês a 1% a.m. por 12 meses = 100 × ((1,01^12 − 1)/0,01).
    const c = compound({
      initial: 0,
      contribution: 100,
      rate: annual(0.01),
      months: 12,
      inflation: 0,
      fee: 0,
    });
    expect(c.balance).toBeCloseTo(100 * ((1.01 ** 12 - 1) / 0.01), 6);
    // Inflação de 10% a.a. em 2 anos: 1.210 vale 1.000 hoje.
    const d = compound({
      initial: 1000,
      contribution: 0,
      rate: 0.1,
      months: 24,
      inflation: 0.1,
      fee: 0,
    });
    expect(d.real).toBeCloseTo(1000, 6);
    // Taxa de administração reduz o saldo e o custo aparece.
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

  it('viver de renda: patrimônio perpétuo, cobertura e quanto tempo dura', () => {
    const r = liveOnIncome({ spending: 5000, realRate: annual(0.005), wealth: 0 });
    // 0,5% real ao mês: 5.000 / 0,005 = 1 milhão.
    expect(r.needed).toBeCloseTo(1_000_000, 3);
    expect(r.coverage).toBe(0);
    expect(r.lastsMonths).toBe(0);
    const half = liveOnIncome({ spending: 5000, realRate: annual(0.005), wealth: 500_000 });
    expect(half.coverage).toBeCloseTo(0.5, 9);
    // 500 mil sacando 5 mil a 0,5% a.m.: n = ln(5000/2500)/ln(1,005) ≈ 139 meses.
    expect(half.lastsMonths).toBeCloseTo(Math.log(2) / Math.log(1.005), 6);
    expect(
      liveOnIncome({ spending: 5000, realRate: annual(0.005), wealth: 2_000_000 }).lastsMonths,
    ).toBeNull();
    // Sem rendimento: dura o patrimônio ÷ gasto.
    expect(liveOnIncome({ spending: 1000, realRate: 0, wealth: 12000 }).lastsMonths).toBe(12);
  });

  it('à vista × parcelado: acha os juros escondidos na parcela', () => {
    // 10 × 110 contra 1.000 à vista, primeira em 30 dias: taxa que zera o fluxo.
    const i = impliedRate(1000, 110, 10, false)!;
    let pv = 0;
    for (let k = 1; k <= 10; k++) pv += 110 / (1 + i) ** k;
    expect(pv).toBeCloseTo(1000, 6);
    expect(i).toBeGreaterThan(0.017);
    // "Sem juros" de verdade: parcelas somam o preço à vista.
    expect(impliedRate(1000, 100, 10, false)).toBeNull();
    // Primeira parcela hoje: taxa maior para o mesmo valor.
    expect(impliedRate(1000, 110, 10, true)!).toBeGreaterThan(i);
    const r = installments({
      cash: 900,
      installment: 100,
      count: 10,
      firstToday: false,
      opportunity: 0,
    });
    // Sem rendimento, as parcelas valem 1.000 hoje: à vista sai 100 mais barato.
    expect(r.total).toBe(1000);
    expect(r.advantageCash).toBeCloseTo(100, 9);
    expect(r.hiddenAnnual).toBeCloseTo(annual(r.hiddenMonthly!), 12);
    // Rendimento alto o bastante faz o parcelado valer menos que o à vista hoje.
    const high = installments({
      cash: 990,
      installment: 100,
      count: 10,
      firstToday: false,
      opportunity: 0.3,
    });
    expect(high.advantageCash).toBeLessThan(0);
  });

  it('alugar × comprar: Price e SAC, mesmo gasto nos dois caminhos', () => {
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
    // Sem rendimento nem valorização: quem compra fica com o imóvel; quem aluga, com a
    // entrada, os custos e as diferenças mês a mês.
    expect(price.buyWealth).toBeCloseTo(500_000, 3);
    expect(price.rentWealth).toBeCloseTo(
      100_000 + 20_000 + 360 * price.firstInstallment - 360 * 2500,
      0,
    );
    const sac = rentVsBuy({ ...base, system: 'sac' });
    expect(sac.firstInstallment).toBeGreaterThan(price.firstInstallment);
    expect(sac.interestPaid).toBeLessThan(price.interestPaid);
    // Horizonte menor que o financiamento: sobra saldo devedor.
    const short = rentVsBuy({ ...base, months: 60 });
    expect(short.loanBalance).toBeGreaterThan(0);
    expect(short.buyWealth).toBeCloseTo(500_000 - short.loanBalance, 3);
    // Valores de hoje com inflação.
    const inf = rentVsBuy({ ...base, inflation: 0.05 });
    expect(inf.buyWealthReal).toBeCloseTo(inf.buyWealth / 1.05 ** 30, 3);
  });
});
