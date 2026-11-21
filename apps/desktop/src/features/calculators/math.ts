/**
 * Contas das calculadoras livres (docs/calculators.md). Puras, sem cofre: valores em unidades
 * de moeda (não centavos), taxas em decimal (0,1 = 10%). As taxas anuais viram mensais pela
 * taxa equivalente, (1 + a)^(1/12) − 1, nunca dividindo por 12.
 */

export const monthly = (annual: number): number => (1 + annual) ** (1 / 12) - 1;
export const annual = (monthlyRate: number): number => (1 + monthlyRate) ** 12 - 1;

// --- Juros compostos -----------------------------------------------------------------------

export interface CompoundInput {
  initial: number;
  /** Aporte no fim de cada mês. */
  contribution: number;
  /** Rendimento ao ano, antes da taxa de administração. */
  rate: number;
  months: number;
  /** Inflação ao ano, para trazer o resultado a dinheiro de hoje. */
  inflation: number;
  /** Taxa de administração ao ano, cobrada sobre o saldo. */
  fee: number;
}

export interface CompoundYear {
  year: number;
  invested: number;
  balance: number;
  /** Saldo em dinheiro de hoje (descontada a inflação). */
  real: number;
}

export interface CompoundResult {
  balance: number;
  invested: number;
  interest: number;
  /** Saldo final em dinheiro de hoje. */
  real: number;
  /** Quanto a taxa de administração levou (saldo sem taxa − saldo com taxa). */
  feeCost: number;
  years: CompoundYear[];
}

function grow(i: CompoundInput, net: number): { balance: number; years: CompoundYear[] } {
  const r = monthly(net);
  const inf = monthly(i.inflation);
  let balance = i.initial;
  const years: CompoundYear[] = [];
  for (let m = 1; m <= i.months; m++) {
    balance = balance * (1 + r) + i.contribution;
    if (m % 12 === 0 || m === i.months) {
      years.push({
        year: Math.ceil(m / 12),
        invested: i.initial + i.contribution * m,
        balance,
        real: balance / (1 + inf) ** m,
      });
    }
  }
  return { balance, years };
}

export function compound(i: CompoundInput): CompoundResult {
  const net = (1 + i.rate) * (1 - i.fee) - 1;
  const withFee = grow(i, net);
  const noFee = i.fee > 0 ? grow(i, i.rate).balance : withFee.balance;
  const invested = i.initial + i.contribution * i.months;
  return {
    balance: withFee.balance,
    invested,
    interest: withFee.balance - invested,
    real: withFee.balance / (1 + monthly(i.inflation)) ** i.months,
    feeCost: noFee - withFee.balance,
    years: withFee.years,
  };
}

// --- Viver de renda --------------------------------------------------------------------------

export interface IncomeInput {
  /** Gasto mensal desejado, em dinheiro de hoje. */
  spending: number;
  /** Rendimento real ao ano (acima da inflação). */
  realRate: number;
  /** Patrimônio que já existe. */
  wealth: number;
}

export interface IncomeResult {
  /** Patrimônio que, só com o rendimento real, paga o gasto para sempre. */
  needed: number;
  /** Quanto do gasto o patrimônio atual já paga só com o rendimento (0–1+). */
  coverage: number;
  /** Meses que o patrimônio atual dura sacando o gasto (rendendo o real); `null` = para sempre. */
  lastsMonths: number | null;
}

export function liveOnIncome(i: IncomeInput): IncomeResult {
  const r = monthly(i.realRate);
  const needed = r > 0 ? i.spending / r : Infinity;
  const coverage = i.spending > 0 ? (i.wealth * r) / i.spending : 0;
  let lastsMonths: number | null;
  if (i.spending <= 0 || i.wealth * r >= i.spending) lastsMonths = null;
  else if (r <= 0) lastsMonths = i.wealth / i.spending;
  else {
    // Saldo zera quando W(1+r)^n = S((1+r)^n − 1)/r → n = ln(S / (S − W r)) / ln(1 + r).
    lastsMonths = Math.log(i.spending / (i.spending - i.wealth * r)) / Math.log(1 + r);
  }
  return { needed, coverage, lastsMonths };
}

// --- À vista × parcelado ---------------------------------------------------------------------

export interface InstallmentsInput {
  /** Preço à vista (já com o desconto). */
  cash: number;
  /** Valor de cada parcela. */
  installment: number;
  count: number;
  /** A primeira parcela é paga hoje (como entrada). */
  firstToday: boolean;
  /** Quanto o dinheiro renderia ao ano enquanto não é usado nas parcelas. */
  opportunity: number;
}

export interface InstallmentsResult {
  total: number;
  /** Juros escondidos na parcela, ao mês e ao ano; `null` se o parcelado sai por menos. */
  hiddenMonthly: number | null;
  hiddenAnnual: number | null;
  /** Valor de hoje das parcelas, descontadas pelo rendimento. */
  presentValue: number;
  /** Valor de hoje das parcelas − preço à vista: positivo = à vista custa menos hoje. */
  advantageCash: number;
}

function pv(installment: number, count: number, r: number, firstToday: boolean): number {
  let s = 0;
  for (let k = 0; k < count; k++) s += installment / (1 + r) ** (firstToday ? k : k + 1);
  return s;
}

/** Taxa mensal que iguala o valor de hoje das parcelas ao preço à vista (bisseção). */
export function impliedRate(
  cash: number,
  installment: number,
  count: number,
  firstToday: boolean,
): number | null {
  if (cash <= 0 || installment <= 0 || count <= 0) return null;
  if (installment * count <= cash) return null;
  if (firstToday && count === 1) return null;
  let lo = 0;
  let hi = 1;
  while (pv(installment, count, hi, firstToday) > cash && hi < 1e6) hi *= 2;
  for (let n = 0; n < 200; n++) {
    const mid = (lo + hi) / 2;
    if (pv(installment, count, mid, firstToday) > cash) lo = mid;
    else hi = mid;
  }
  return (lo + hi) / 2;
}

export function installments(i: InstallmentsInput): InstallmentsResult {
  const hidden = impliedRate(i.cash, i.installment, i.count, i.firstToday);
  const present = pv(i.installment, i.count, monthly(i.opportunity), i.firstToday);
  return {
    total: i.installment * i.count,
    hiddenMonthly: hidden,
    hiddenAnnual: hidden == null ? null : annual(hidden),
    presentValue: present,
    advantageCash: present - i.cash,
  };
}

// --- Alugar × comprar ------------------------------------------------------------------------

export type AmortizationSystem = 'price' | 'sac';

export interface RentBuyInput {
  price: number;
  downPayment: number;
  /** Custos da compra (impostos, cartório, avaliação), em fração do preço. */
  buyCosts: number;
  /** Juros do financiamento ao ano. */
  loanRate: number;
  loanMonths: number;
  system: AmortizationSystem;
  /** Aluguel do mesmo imóvel hoje, ao mês. */
  rent: number;
  /** Reajuste do aluguel ao ano. */
  rentGrowth: number;
  /** Valorização do imóvel ao ano. */
  appreciation: number;
  /** Manutenção, condomínio extra e imposto do imóvel ao ano, em fração do valor. */
  ownerCosts: number;
  /** Rendimento ao ano do dinheiro investido por quem aluga. */
  investReturn: number;
  /** Inflação ao ano, para os valores de hoje. */
  inflation: number;
  /** Horizonte da comparação, em meses. */
  months: number;
}

export interface RentBuyResult {
  /** Patrimônio de quem compra no fim: imóvel − saldo devedor + o que investiu. */
  buyWealth: number;
  /** Patrimônio de quem aluga no fim: o que investiu. */
  rentWealth: number;
  /** Os dois em dinheiro de hoje. */
  buyWealthReal: number;
  rentWealthReal: number;
  /** Primeira parcela do financiamento. */
  firstInstallment: number;
  /** Total pago de juros do financiamento no horizonte. */
  interestPaid: number;
  /** Valor do imóvel no fim. */
  propertyValue: number;
  /** Saldo devedor no fim. */
  loanBalance: number;
}

/**
 * Os dois caminhos gastam o mesmo dinheiro a cada mês. Quem compra paga a parcela e os
 * custos do imóvel; quem aluga paga o aluguel e investe a entrada, os custos da compra e, a
 * cada mês, a diferença para o que o comprador gastou. Se o aluguel passar do custo de quem
 * compra, é o comprador que investe a diferença.
 */
export function rentVsBuy(i: RentBuyInput): RentBuyResult {
  const loan = Math.max(0, i.price - i.downPayment);
  const rLoan = monthly(i.loanRate);
  const rInv = monthly(i.investReturn);
  const n = Math.max(1, i.loanMonths);
  const pricePmt = rLoan > 0 ? (loan * rLoan) / (1 - (1 + rLoan) ** -n) : loan / n;
  const sacAmort = loan / n;
  let balance = loan;
  let rent = i.rent;
  let value = i.price;
  let renterInv = i.downPayment + i.price * i.buyCosts;
  let buyerInv = 0;
  let interestPaid = 0;
  let firstInstallment = 0;
  for (let m = 1; m <= i.months; m++) {
    let installment = 0;
    if (balance > 1e-9 && m <= n) {
      const interest = balance * rLoan;
      const amort = i.system === 'price' ? pricePmt - interest : sacAmort;
      installment = interest + Math.min(amort, balance);
      balance = Math.max(0, balance - amort);
      interestPaid += interest;
      if (m === 1) firstInstallment = installment;
    }
    const ownerCosts = (value * i.ownerCosts) / 12;
    const buyerSpend = installment + ownerCosts;
    renterInv *= 1 + rInv;
    buyerInv *= 1 + rInv;
    if (buyerSpend >= rent) renterInv += buyerSpend - rent;
    else buyerInv += rent - buyerSpend;
    value *= 1 + monthly(i.appreciation);
    if (m % 12 === 0) rent *= 1 + i.rentGrowth;
  }
  const deflate = (1 + monthly(i.inflation)) ** i.months;
  const buyWealth = value - balance + buyerInv;
  return {
    buyWealth,
    rentWealth: renterInv,
    buyWealthReal: buyWealth / deflate,
    rentWealthReal: renterInv / deflate,
    firstInstallment,
    interestPaid,
    propertyValue: value,
    loanBalance: balance,
  };
}
