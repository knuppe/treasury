/**
 * Taxa de um empréstimo a partir da parcela, no mesmo modelo do cronograma do núcleo
 * (`debt::solve_rate`): períodos mensais, Price com parcela constante depois da carência
 * e SAC pela primeira parcela com amortização. Seguros e taxas mensais saem da parcela.
 */
export interface LoanRateInput {
  principal: number;
  /** Parcela paga por mês, em centavos (com seguros e taxas). */
  installment: number;
  term: number;
  grace: number;
  system: 'sac' | 'price' | 'bullet';
  /** Seguros + taxas por mês (centavos). */
  charges: number;
}

export interface LoanRate {
  /** % a.m. */
  monthlyPct: number;
  /** % a.a. efetiva. */
  annualPct: number;
}

export function solveLoanRate(p: LoanRateInput): LoanRate | null {
  const n = p.term - p.grace;
  const pmt = p.installment - p.charges;
  if (!(p.principal > 0) || !(n > 0) || !(pmt > 0)) return null;
  let i: number;
  if (p.system === 'bullet') {
    // Só juros: a parcela é o saldo × taxa.
    i = pmt / p.principal;
  } else if (p.system === 'sac') {
    i = (pmt - p.principal / n) / p.principal;
  } else {
    if (pmt * n < p.principal - 0.5) return null;
    const f = (r: number) => (r === 0 ? p.principal / n : (p.principal * r) / (1 - (1 + r) ** -n));
    let lo = 0;
    let hi = 0.5;
    if (f(hi) < pmt) return null;
    for (let k = 0; k < 200; k++) {
      const mid = (lo + hi) / 2;
      if (f(mid) < pmt) lo = mid;
      else hi = mid;
    }
    i = (lo + hi) / 2;
  }
  if (!(i >= -1e-9 && i <= 0.5)) return null;
  i = Math.max(i, 0);
  return { monthlyPct: i * 100, annualPct: ((1 + i) ** 12 - 1) * 100 };
}

/** Taxa em texto decimal com até 6 casas (o formato que o núcleo recebe). */
export function rateString(pct: number): string {
  return String(Number(pct.toFixed(6)));
}
