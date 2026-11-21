/**
 * Imposto de renda e contribuições do trabalho, como `tax_burden::rules` do núcleo: tabela
 * progressiva com parcela a deduzir, INSS por fatia com teto, redutor de 2026 e o inverso
 * (bruto a partir do líquido). Centavos; alíquotas em %.
 */
import type { ContributionBand } from '@/api/generated/ContributionBand';
import type { IrBracket } from '@/api/generated/IrBracket';
import type { PayrollRules } from '@/api/generated/PayrollRules';

/** Arredonda como `f64::round` (metade para longe do zero). */
const round = (v: number) => Math.sign(v) * Math.round(Math.abs(v));

export function contribution(gross: number, bands: readonly ContributionBand[]): number {
  let prev = 0;
  let total = 0;
  for (const b of bands) {
    if (gross <= prev) break;
    total += ((Math.min(gross, b.up_to) - prev) * b.rate) / 100;
    prev = b.up_to;
  }
  return round(total);
}

export function irTable(base: number, brackets: readonly IrBracket[]): number {
  if (base <= 0) return 0;
  const b = brackets.find((x) => x.up_to == null || base <= x.up_to) ?? brackets.at(-1);
  return b ? Math.max(round((base * b.rate) / 100) - b.deduction, 0) : 0;
}

export function monthlyIrrf(
  gross: number,
  inss: number,
  dependents: number,
  r: PayrollRules,
): number {
  const legal = inss + r.dependent_deduction * dependents;
  const tax = irTable(gross - Math.max(legal, r.simplified_discount), r.ir_monthly);
  const red = r.reducer;
  const cut = !red
    ? 0
    : gross <= red.exempt_up_to
      ? red.max_reduction
      : gross < red.phase_out_up_to
        ? Math.max(red.a - round(red.b * gross), 0)
        : 0;
  return Math.max(tax - Math.min(cut, tax), 0);
}

/** INSS e IRRF do mês para um bruto. */
export function payroll(gross: number, dependents: number, r: PayrollRules): [number, number] {
  const inss = contribution(gross, r.inss_bands);
  return [inss, monthlyIrrf(gross, inss, dependents, r)];
}

/** Bruto mensal que, descontados INSS e IRRF, dá `net` (busca binária). */
export function grossFromNet(net: number, dependents: number, r: PayrollRules): number {
  if (net <= 0) return 0;
  const netOf = (g: number) => {
    const [inss, irrf] = payroll(g, dependents, r);
    return g - inss - irrf;
  };
  let lo = net;
  let hi = net * 2 + 1;
  while (netOf(hi) < net) hi *= 2;
  while (hi - lo > 1) {
    const mid = lo + Math.floor((hi - lo) / 2);
    if (netOf(mid) < net) lo = mid;
    else hi = mid;
  }
  return netOf(lo) >= net ? lo : hi;
}
