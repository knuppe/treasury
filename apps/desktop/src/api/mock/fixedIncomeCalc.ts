/**
 * Mock das calculadoras de renda fixa (`portfolio/compare.rs`): comparar ofertas e vender
 * um prefixado ou IPCA+ antes do vencimento. Mesmas regras do núcleo, simplificadas: dias
 * úteis ≈ dias corridos × 252/365 (sem feriados), IR regressivo e IOF por dias corridos,
 * custódia do Tesouro, regra da poupança só nos aniversários e equivalência em % do CDI por
 * bisseção. As ofertas saem na ordem em que vieram (sem "melhor").
 */
import type { CompareFixedIncomeArgs } from '../generated/CompareFixedIncomeArgs';
import type { FixedIncomeComparison } from '../generated/FixedIncomeComparison';
import type { FixedIncomeKind } from '../generated/FixedIncomeKind';
import type { FixedIncomeOffer } from '../generated/FixedIncomeOffer';
import type { FixedIncomeSale } from '../generated/FixedIncomeSale';
import type { FixedIncomeSaleArgs } from '../generated/FixedIncomeSaleArgs';
import type { OfferIndex } from '../generated/OfferIndex';
import type { OfferResult } from '../generated/OfferResult';
import type { RateAssumption } from '../generated/RateAssumption';
import type { RateAssumptions } from '../generated/RateAssumptions';
import type { SaleLeg } from '../generated/SaleLeg';
import { mockFail } from '../mockRegistry';
import { addMonths, daysBetween, mockToday, monthStart, previousBusinessDay } from './marketSim';

const MAX_OFFERS = 12;
/** Custódia da B3 no Tesouro Direto (fração a.a.) e a isenção do Tesouro Selic (centavos). */
const B3_FEE = 0.002;
const SELIC_EXEMPT = 1_000_000;

const EXEMPT: readonly FixedIncomeKind[] = [
  'lci',
  'lca',
  'cri',
  'cra',
  'debenture_incentivada',
  'poupanca',
];
const FGC_YES: readonly FixedIncomeKind[] = ['cdb', 'rdb', 'lc', 'lci', 'lca', 'poupanca'];
const FGC_NO: readonly FixedIncomeKind[] = [
  'cri',
  'cra',
  'debenture',
  'debenture_incentivada',
  'tesouro_selic',
  'tesouro_prefixado',
  'tesouro_ipca',
  'tesouro_renda',
];
const TREASURY: readonly FixedIncomeKind[] = [
  'tesouro_selic',
  'tesouro_prefixado',
  'tesouro_ipca',
  'tesouro_renda',
];

const KIND_LABEL: Record<FixedIncomeKind, string> = {
  cdb: 'CDB',
  rdb: 'RDB',
  lc: 'LC',
  lci: 'LCI',
  lca: 'LCA',
  cri: 'CRI',
  cra: 'CRA',
  debenture: 'Debênture',
  debenture_incentivada: 'Debênture incentivada',
  tesouro_selic: 'Tesouro Selic',
  tesouro_prefixado: 'Tesouro Prefixado',
  tesouro_ipca: 'Tesouro IPCA+',
  tesouro_renda: 'Tesouro Renda+',
  poupanca: 'Poupança',
  other: 'Renda fixa',
};

const IOF_TABLE = [
  96, 93, 90, 86, 83, 80, 76, 73, 70, 66, 63, 60, 56, 53, 50, 46, 43, 40, 36, 33, 30, 26, 23, 20,
  16, 13, 10, 6, 3, 0,
];

const NOTES = {
  iof: 'Resgate antes de 30 dias: paga IOF sobre o rendimento.',
  aniversario: 'A poupança só rende no aniversário: o resgate antes dele perde o mês corrente.',
  custodia:
    'Inclui a taxa de custódia da B3 (0,20% a.a.; no Tesouro Selic, só acima de R$ 10 mil).',
  venda:
    'Vender antes do vencimento recebe o preço de mercado do dia, que pode ficar acima ou abaixo da curva.',
  vendaIpca:
    'O IPCA até a venda e até o vencimento é a premissa informada, não o índice publicado.',
  aviso:
    'Estimativa com as premissas mostradas, não promessa de rendimento nem recomendação de investimento.',
  legal: [
    'IR regressivo (22,5% a 15%) e isenção de LCI, LCA, CRI, CRA e poupança: Lei 11.033/2004; debêntures incentivadas: Lei 12.431/2011.',
    'IOF regressivo sobre o rendimento até o 29º dia: Decreto 6.306/2007.',
    'Custódia do Tesouro Direto: 0,20% a.a. da B3; Tesouro Selic sem taxa até R$ 10 mil.',
    'Poupança: 0,5% a.m. + TR com a Selic acima de 8,5% a.a.; senão, 70% da Selic + TR (Lei 8.177/1991).',
  ],
};

const round = (v: number) => Math.round(v);
const num = (v: unknown) => (typeof v === 'number' && Number.isFinite(v) ? v : undefined);
const str = (v: unknown) => (typeof v === 'string' && v ? v : undefined);
const isoDate = (v: string) => /^\d{4}-\d{2}-\d{2}$/.test(v);

/** Dias úteis aproximados (base 252, sem calendário de feriados). */
const businessDaysApprox = (days: number) => Math.round((days * 252) / 365);

export function irRate(days: number): number {
  return days <= 180 ? 0.225 : days <= 360 ? 0.2 : days <= 720 ? 0.175 : 0.15;
}

function iofRate(days: number): number {
  if (days <= 0) return 1;
  return (IOF_TABLE[days - 1] ?? 0) / 100;
}

/** (IOF, IR) de um resgate, em centavos. */
function taxes(invested: number, gross: number, days: number, exempt: boolean) {
  if (exempt) return { iof: 0, ir: 0 };
  const income = Math.max(gross - invested, 0);
  const iof = round(income * iofRate(days));
  return { iof, ir: round((income - iof) * irRate(days)) };
}

function assumption(user: number | undefined, vault: RateAssumption): RateAssumption {
  return user == null ? vault : { value: user, source: 'user', date: null };
}

/** Premissas do mock: as "do cofre" fixas, com as digitadas por cima. */
export function assumptions(a: Record<string, unknown>): RateAssumptions {
  const today = mockToday();
  const lastMonth = monthStart(addMonths(today, -1).slice(0, 7));
  return {
    cdi: assumption(num(a.cdi), {
      value: 0.149,
      source: 'vault',
      date: previousBusinessDay(today),
    }),
    selic: assumption(num(a.selic), { value: 0.15, source: 'vault', date: null }),
    ipca: assumption(num(a.ipca), { value: 0.045, source: 'vault', date: lastMonth }),
    tr: assumption(num(a.tr), { value: 0.0017, source: 'vault', date: lastMonth }),
  };
}

export function savingsMonthly(selic: number, tr: number): number {
  const base = selic > 0.085 ? 0.005 : 0.7 * ((1 + selic) ** (1 / 12) - 1);
  return base + tr;
}

function anniversaries(start: string, end: string): number {
  let n = 0;
  while (addMonths(start, n + 1) <= end) n += 1;
  return n;
}

function grossFactor(
  index: OfferIndex,
  rate: number,
  du: number,
  months: number,
  a: RateAssumptions,
): number {
  const years = du / 252;
  switch (index) {
    case 'cdi_pct': {
      const day = (1 + a.cdi.value) ** (1 / 252) - 1;
      return (1 + (day * rate) / 100) ** du;
    }
    case 'cdi_plus':
    case 'selic_plus':
      return ((1 + a.cdi.value) * (1 + rate / 100)) ** years;
    case 'prefixed':
      return (1 + rate / 100) ** years;
    case 'ipca_plus':
      return ((1 + a.ipca.value) * (1 + rate / 100)) ** years;
    case 'savings':
      return (1 + savingsMonthly(a.selic.value, a.tr.value)) ** months;
  }
}

function netAtPctCdi(
  pct: number,
  exempt: boolean,
  amount: number,
  du: number,
  days: number,
  a: RateAssumptions,
) {
  const gross = round(amount * grossFactor('cdi_pct', pct, du, 0, a));
  const { iof, ir } = taxes(amount, gross, days, exempt);
  return gross - iof - ir;
}

/** % do CDI que dá `target` de líquido no prazo (bisseção em [0, 1000]). */
function equivalentPct(
  target: number,
  exempt: boolean,
  amount: number,
  du: number,
  days: number,
  a: RateAssumptions,
): number | null {
  if (du <= 0 || a.cdi.value <= 0) return null;
  let lo = 0;
  let hi = 1000;
  if (
    netAtPctCdi(hi, exempt, amount, du, days, a) < target ||
    netAtPctCdi(lo, exempt, amount, du, days, a) > target
  )
    return null;
  for (let i = 0; i < 60; i++) {
    const mid = (lo + hi) / 2;
    if (netAtPctCdi(mid, exempt, amount, du, days, a) < target) lo = mid;
    else hi = mid;
  }
  return Math.round(hi * 10) / 10;
}

const pctText = (v: number) => String(v).replace('.', ',');

function defaultLabel(o: FixedIncomeOffer): string {
  const kind = KIND_LABEL[o.kind];
  const r = pctText(o.rate);
  switch (o.index) {
    case 'cdi_pct':
      return `${kind} ${r}% do CDI`;
    case 'cdi_plus':
      return `${kind} CDI + ${r}%`;
    case 'selic_plus':
      return `${kind} Selic + ${r}%`;
    case 'prefixed':
      return `${kind} ${r}% a.a.`;
    case 'ipca_plus':
      return `${kind} IPCA + ${r}%`;
    case 'savings':
      return kind;
  }
}

function evaluate(o: FixedIncomeOffer, amount: number, start: string, a: RateAssumptions) {
  const end = o.end ? o.end : o.months && o.months > 0 ? addMonths(start, o.months) : null;
  if (!end || !isoDate(end) || end <= start)
    mockFail('invalid', 'informe um prazo em meses ou uma data de resgate depois da aplicação');
  if (
    !Number.isFinite(o.rate) ||
    (o.index !== 'savings' && o.index !== 'selic_plus' && o.rate <= -100)
  )
    mockFail('invalid', 'taxa inválida');
  const days = daysBetween(start, end);
  const du = businessDaysApprox(days);
  const months = anniversaries(start, end);
  const exempt = o.ir_exempt ?? EXEMPT.includes(o.kind);
  const gross = round(amount * grossFactor(o.index, o.rate, du, months, a));
  const { iof, ir } = taxes(amount, gross, days, exempt);
  const fee = o.custody ?? (TREASURY.includes(o.kind) ? B3_FEE : 0);
  let custody = 0;
  if (fee > 0 && days > 0) {
    let avg = (amount + gross) / 2;
    if (o.kind === 'tesouro_selic') avg = Math.max(avg - SELIC_EXEMPT, 0);
    custody = round((avg * fee * days) / 365);
  }
  const net = gross - iof - ir - custody;
  const netAnnual = amount > 0 && du > 0 ? (net / amount) ** (252 / du) - 1 : 0;
  const notes: string[] = [];
  if (!exempt && days < 30) notes.push(NOTES.iof);
  if (o.index === 'savings' && addMonths(start, months) < end) notes.push(NOTES.aniversario);
  if (custody > 0) notes.push(NOTES.custodia);
  const result: OfferResult = {
    label: o.label?.trim() || defaultLabel(o),
    kind: o.kind,
    index: o.index,
    rate: o.rate,
    start,
    end,
    calendar_days: days,
    business_days: du,
    ir_exempt: exempt,
    ir_rate: exempt ? 0 : irRate(days),
    fgc: FGC_YES.includes(o.kind) ? true : FGC_NO.includes(o.kind) ? false : null,
    invested: amount,
    gross,
    iof,
    ir,
    custody,
    net,
    net_income: net - amount,
    net_annual: netAnnual,
    real_annual: (1 + netAnnual) / (1 + a.ipca.value) - 1,
    taxable_equivalent_pct_cdi: equivalentPct(net, false, amount, du, days, a),
    exempt_equivalent_pct_cdi: equivalentPct(net, true, amount, du, days, a),
    notes,
  };
  return result;
}

export function compareFixedIncome(raw: Record<string, unknown>): FixedIncomeComparison {
  const args = raw as unknown as CompareFixedIncomeArgs;
  const amount = num(args.amount) ?? 0;
  if (amount <= 0) mockFail('invalid', 'informe o valor aplicado');
  const offers = Array.isArray(args.offers) ? args.offers : [];
  if (!offers.length || offers.length > MAX_OFFERS)
    mockFail('invalid', `informe de 1 a ${MAX_OFFERS} ofertas`);
  const start = str(args.start) ?? mockToday();
  const a = assumptions(raw);
  return {
    assumptions: a,
    offers: offers.map((o) => evaluate(o, amount, start, a)),
    notes: [...NOTES.legal, NOTES.aviso],
  };
}

function leg(invested: number, gross: number, days: number, exempt: boolean): SaleLeg {
  const { iof, ir } = taxes(invested, gross, days, exempt);
  return { gross, iof, ir, net: gross - iof - ir, calendar_days: days };
}

export function simulateFixedIncomeSale(raw: Record<string, unknown>): FixedIncomeSale {
  const args = raw as unknown as FixedIncomeSaleArgs;
  const amount = num(args.amount) ?? 0;
  if (amount <= 0) mockFail('invalid', 'informe o valor aplicado');
  const start = str(args.start) ?? '';
  const maturity = str(args.maturity) ?? '';
  const sale = str(args.sale_date) ?? mockToday();
  if (!(isoDate(start) && isoDate(maturity) && start < sale && sale < maturity))
    mockFail('invalid', 'a venda precisa ficar entre a aplicação e o vencimento');
  for (const r of [args.contracted_rate, args.market_rate])
    if (!Number.isFinite(r) || r <= -100) mockFail('invalid', 'taxa inválida');
  const ipca: RateAssumption =
    args.index === 'prefixed'
      ? { value: 0, source: 'user', date: null }
      : assumptions({ ipca: args.ipca }).ipca;
  const exempt = args.ir_exempt ?? false;
  const duTotal = businessDaysApprox(daysBetween(start, maturity));
  const duDone = businessDaysApprox(daysBetween(start, sale));
  const duRest = duTotal - duDone;
  const rc = 1 + args.contracted_rate / 100;
  const rm = 1 + args.market_rate / 100;
  const infl = 1 + ipca.value;
  // Valor nominal atualizado pelo IPCA suposto até a venda (prefixado: o próprio aplicado).
  const vnaSale = amount * infl ** (duDone / 252);
  const curve = round(vnaSale * rc ** (duDone / 252));
  const market = round((vnaSale * rc ** (duTotal / 252)) / rm ** (duRest / 252));
  const atMaturity = round(amount * infl ** (duTotal / 252) * rc ** (duTotal / 252));
  const saleLeg = leg(amount, market, daysBetween(start, sale), exempt);
  const holdLeg = leg(amount, atMaturity, daysBetween(start, maturity), exempt);
  // Reaplicar o líquido da venda até o vencimento, com o IR contando do zero.
  const restDays = daysBetween(sale, maturity);
  const reinvest = (rate: number) =>
    leg(
      saleLeg.net,
      round(saleLeg.net * infl ** (duRest / 252) * (1 + rate / 100) ** (duRest / 252)),
      restDays,
      exempt,
    ).net;
  let breakeven: number | null = null;
  if (duRest > 0 && saleLeg.net > 0) {
    let lo = -50;
    let hi = 200;
    if (reinvest(lo) <= holdLeg.net && reinvest(hi) >= holdLeg.net) {
      for (let i = 0; i < 80; i++) {
        const mid = (lo + hi) / 2;
        if (reinvest(mid) < holdLeg.net) lo = mid;
        else hi = mid;
      }
      breakeven = Math.round(hi * 100) / 100;
    }
  }
  const notes = [NOTES.venda, NOTES.aviso];
  if (args.index === 'ipca_plus') notes.unshift(NOTES.vendaIpca);
  return {
    ipca,
    business_days_elapsed: duDone,
    business_days_remaining: duRest,
    curve_value: curve,
    market_value: market,
    mark_to_market: market - curve,
    sale: saleLeg,
    hold: holdLeg,
    breakeven_rate: breakeven,
    notes,
  };
}
