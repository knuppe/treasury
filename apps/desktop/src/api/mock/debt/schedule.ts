/**
 * Cronograma de amortização do mock, espelho de `debt/schedule.rs`: SAC e Price, carência
 * (só juros), correção do saldo pelo indexador, seguros e taxas fixos e amortizações
 * extraordinárias (reduzir prazo × parcela). Centavos com arredondamento metade para longe
 * do zero; a taxa mensal é a equivalente à efetiva anual.
 */
import { addMonths } from '../ledger/dates';

export type System = 'sac' | 'price' | 'bullet';

/** Período de um pós-fixado: vencimento, fator de juros e dias úteis (ver `postfixed.rs`). */
export interface Period {
  due: string;
  factor: number;
  businessDays: number;
  estimated: boolean;
}
export type ExtraMode = 'reduce_term' | 'reduce_installment';

export interface Extra {
  /** Aplicada logo depois da parcela `afterNumber` (0 = antes da primeira). */
  afterNumber: number;
  amount: number;
  mode: ExtraMode;
}

export interface Params {
  principal: number;
  annualRatePct: number;
  system: System;
  term: number;
  grace: number;
  firstDue: string;
  insurance: number;
  fees: number;
  /** Seguro MIP: fração ao mês sobre o saldo devedor. */
  mipRate?: number;
  /** MIP em valor fixo por mês (centavos; 0 = usa a fração sobre o saldo). */
  mipFixed?: number;
  /** Seguro DFI do mês, em centavos (fração sobre o valor do imóvel). */
  dfi?: number;
  /** Fator de correção por parcela (índice 0 = parcela 1) e se é estimado. */
  indexFactors: readonly [number, boolean][];
  extras: readonly Extra[];
  /** Pós-fixado de taxa diária: um período por parcela (vazio = taxa mensal fixa). */
  periods?: readonly Period[];
}

export interface Row {
  number: number;
  due_date: string;
  amortization: number;
  interest: number;
  adjustment: number;
  insurance: number;
  insurance_mip: number;
  insurance_dfi: number;
  fees: number;
  total: number;
  balance_after: number;
  estimated: boolean;
  extra_payment: number;
  index_factor: string | null;
  business_days: number | null;
}

/** Arredonda para centavos inteiros, metade para longe do zero. */
export function round(v: number): number {
  return Math.sign(v) * Math.round(Math.abs(v) + 1e-9);
}

export function monthlyRate(annualPct: number): number {
  return (1 + annualPct / 100) ** (1 / 12) - 1;
}

/** PMT da tabela Price: B × i ÷ (1 − (1+i)^−n). */
export function pricePayment(balance: number, i: number, n: number): number {
  if (i === 0) return balance / Math.max(n, 1);
  const f = (1 + i) ** n;
  return (balance * i * f) / (f - 1);
}

/** Parcelas para quitar `balance` mantendo a amortização (SAC) ou a parcela (Price). */
function monthsLeft(system: System, balance: number, kept: number, i: number): number {
  let n: number;
  if (system === 'sac' || i === 0) n = balance / kept;
  else {
    const q = 1 - (balance * i) / kept;
    if (q <= 0) return 600;
    n = -Math.log(q) / Math.log(1 + i);
  }
  return Number.isFinite(n) && n > 0 ? Math.min(Math.ceil(n - 1e-9), 600) : 1;
}

export function build(p: Params): Row[] {
  const monthly = monthlyRate(p.annualRatePct);
  const periods = p.periods ?? [];
  const rateOf = (k: number) => {
    const per = periods[k - 1] ?? periods[periods.length - 1];
    return per ? per.factor - 1 : monthly;
  };
  let i = rateOf(1);
  let balance = p.principal;
  const rows: Row[] = [];
  let fixed: number | null = null;
  let termLeft = p.term;
  for (const e of p.extras.filter((x) => x.afterNumber === 0)) {
    // Antes da 1ª parcela, "reduzir prazo" mantém a parcela/amortização original.
    if (e.mode === 'reduce_term' && fixed == null) {
      const n = Math.max(p.term - p.grace, 1);
      fixed =
        p.system === 'sac'
          ? round(balance / n)
          : p.system === 'price'
            ? round(pricePayment(balance, i, n))
            : null;
    }
    balance = Math.max(balance - e.amount, 0);
  }
  let k = 0;
  while (balance > 0 && k < p.term + 600) {
    k += 1;
    const [factor, est] = p.indexFactors[k - 1] ?? [1, false];
    const before = balance;
    balance = round(balance * factor);
    const adjustment = balance - before;
    // Parcela/amortização mantida (reduzir prazo) também é corrigida pelo índice.
    if (fixed != null) fixed = round(fixed * factor);
    i = rateOf(k);
    const period = periods[k - 1];
    const estimated = est || (period?.estimated ?? false);
    const interest = round(balance * i);
    const remaining = Math.max(termLeft - (k - 1), 1);
    const left = Math.max(remaining - Math.max(p.grace - (k - 1), 0), 1);
    let amort: number;
    if (k <= p.grace || p.system === 'bullet') amort = 0;
    else if (p.system === 'sac') amort = fixed ?? round(balance / left);
    else
      amort =
        fixed != null ? round(fixed - interest) : round(pricePayment(balance, i, left) - interest);
    if (amort > balance || (k >= termLeft && fixed == null)) amort = balance;
    amort = Math.max(amort, 0);
    balance -= amort;
    let extraTotal = 0;
    for (const e of p.extras.filter((x) => x.afterNumber === k)) {
      const x = Math.min(e.amount, balance);
      balance -= x;
      extraTotal += x;
      if (balance > 0) {
        if (e.mode === 'reduce_term')
          fixed = p.system === 'sac' ? amort : p.system === 'price' ? amort + interest : null;
        else {
          // Depois de um "reduzir prazo", mantém o prazo que restava (não o original + folga).
          if (fixed != null) termLeft = k + monthsLeft(p.system, balance + x, fixed, i);
          fixed = null;
        }
      }
    }
    if (fixed != null) termLeft = p.term + 600;
    // MIP sobre o saldo do mês (corrigido, antes de amortizar); DFI fixo.
    const open = balance + amort > 0;
    const mip = !open ? 0 : p.mipFixed ? p.mipFixed : round((balance + amort) * (p.mipRate ?? 0));
    const dfi = open ? (p.dfi ?? 0) : 0;
    const ins = open ? p.insurance + mip + dfi : 0;
    rows.push({
      number: k,
      due_date: period?.due ?? addMonths(p.firstDue, k - 1),
      amortization: amort,
      interest,
      adjustment,
      insurance: ins,
      insurance_mip: mip,
      insurance_dfi: dfi,
      fees: p.fees,
      total: amort + interest + ins + p.fees,
      balance_after: balance,
      estimated,
      extra_payment: extraTotal,
      index_factor: period ? String(Number(period.factor.toFixed(9))) : null,
      business_days: period?.businessDays ?? null,
    });
  }
  return rows;
}

/** XIRR (taxa anual efetiva) de fluxos datados; `null` sem convergência. */
export function xirr(flows: readonly { date: string; amount: number }[]): number | null {
  if (flows.length < 2 || !flows.some((f) => f.amount < 0) || !flows.some((f) => f.amount > 0))
    return null;
  const t0 = flows.reduce((m, f) => (f.date < m ? f.date : m), flows[0]!.date);
  const years = (d: string) => (Date.parse(d) - Date.parse(t0)) / 86_400_000 / 365;
  const npv = (r: number) => flows.reduce((s, f) => s + f.amount / (1 + r) ** years(f.date), 0);
  const dnpv = (r: number) =>
    flows.reduce((s, f) => s - (years(f.date) * f.amount) / (1 + r) ** (years(f.date) + 1), 0);
  for (const guess of [0.1, 0, -0.5, 1, 5]) {
    let r = guess;
    for (let n = 0; n < 100; n++) {
      const v = npv(r);
      const d = dnpv(r);
      if (!Number.isFinite(v) || !Number.isFinite(d) || d === 0) break;
      const next = r - v / d;
      if (next <= -1) break;
      if (Math.abs(next - r) < 1e-10) return next;
      r = next;
    }
  }
  return null;
}
