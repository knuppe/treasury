/**
 * Mock de `tax.annual_draft` e `tax.annual_compute`: a declaração de ajuste anual pelas
 * mesmas regras de `tax/annual.rs` (tabela anual, simplificado de 20% com teto, deduções do
 * completo, redução da Lei 15.270 a partir de 2026). A prévia do mock vem de um salário fixo
 * de demonstração; o núcleo lê holerites e categorias do cofre.
 */
import type { AnnualDraft } from '../generated/AnnualDraft';
import type { AnnualInput } from '../generated/AnnualInput';
import type { AnnualModel } from '../generated/AnnualModel';
import type { AnnualResult } from '../generated/AnnualResult';
import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { invalid } from './ledger/errors';

type Bracket = [upTo: number | null, permille: number, deduction: number];

function rules(year: number) {
  return year >= 2026
    ? {
        brackets: [
          [2_914_560, 0, 0],
          [3_391_980, 75, 218_592],
          [4_501_260, 150, 472_991],
          [5_597_616, 225, 810_585],
          [null, 275, 1_090_466],
        ] as Bracket[],
        cap: 1_764_000,
        reduction: true,
      }
    : {
        brackets: [
          [2_846_720, 0, 0],
          [3_391_980, 75, 213_504],
          [4_501_260, 150, 467_903],
          [5_597_616, 225, 805_497],
          [null, 275, 1_085_378],
        ] as Bracket[],
        cap: 1_675_434,
        reduction: false,
      };
}

const DEPENDENT = 227_508;
const EDUCATION_CAP = 356_150;

function tableTax(base: number, brackets: Bracket[]): number {
  if (base <= 0) return 0;
  const b = brackets.find(([u]) => u == null || base <= u) ?? brackets[brackets.length - 1]!;
  return Math.max(0, Math.trunc((base * b[1]) / 1000) - b[2]);
}

export function computeAnnual(input: AnnualInput): AnnualResult {
  const r = rules(input.year);
  const se = input.self_employed;
  const taxable = input.employment.reduce((s, e) => s + e.income, 0) + se.income;
  const withheld = input.employment.reduce((s, e) => s + e.withheld, 0);
  const basePaid = withheld + se.carne_leao + input.complementary_tax + input.irrf_lei_11033;
  const social = input.employment.reduce((s, e) => s + e.social_security, 0) + se.social_security;
  const education = input.education.reduce(
    (s, v) => s + Math.min(Math.max(v, 0), EDUCATION_CAP),
    0,
  );
  const pension = Math.min(Math.max(input.private_pension, 0), Math.trunc((taxable * 12) / 100));
  const full =
    social +
    DEPENDENT * input.dependents +
    education +
    Math.max(input.medical, 0) +
    Math.max(input.alimony, 0) +
    Math.max(se.alimony, 0) +
    pension +
    Math.max(se.livro_caixa, 0);
  const simplifiedOf = (income: number) => Math.min(Math.trunc((income * 20) / 100), r.cap);
  const reductionOf = (tax: number) => {
    if (!r.reduction) return 0;
    const cut =
      taxable <= 6_000_000
        ? 269_415
        : taxable <= 8_820_000
          ? Math.min(Math.max(842_973 - Math.trunc((taxable * 95_575) / 1_000_000), 0), 269_415)
          : 0;
    return Math.min(cut, tax);
  };
  const model = (deductions: number, withoutForeign: number, donationsAllowed: boolean) => {
    const base = Math.max(taxable - deductions, 0);
    const table = tableTax(base, r.brackets);
    const reduction = reductionOf(table);
    const after = table - reduction;
    const cap = Math.trunc((after * 3) / 100);
    const donations = donationsAllowed
      ? Math.min(Math.max(input.donation_children, 0), cap) +
        Math.min(Math.max(input.donation_elderly, 0), cap)
      : 0;
    const due = Math.max(after - donations, 0);
    const foreign =
      input.foreign_tax_paid > 0 && se.abroad > 0
        ? Math.min(
            input.foreign_tax_paid,
            Math.max(table - tableTax(withoutForeign, r.brackets), 0),
          )
        : 0;
    const paid = basePaid + foreign;
    return {
      deductions,
      base,
      table_tax: table,
      reduction,
      donations,
      tax_due: due,
      tax_paid: paid,
      to_pay: Math.max(due - paid, 0),
      refund: Math.max(paid - due, 0),
    } satisfies AnnualModel;
  };
  const fullModel = model(full, Math.max(taxable - se.abroad - full, 0), true);
  const simple = simplifiedOf(taxable);
  const simpleModel = model(
    simple,
    Math.max(taxable - se.abroad - simplifiedOf(taxable - se.abroad), 0),
    false,
  );
  const bestFull = fullModel.tax_due <= simpleModel.tax_due;
  const bestTax = bestFull ? fullModel.tax_due : simpleModel.tax_due;
  return {
    year: input.year,
    taxable_income: taxable,
    full: fullModel,
    simplified: simpleModel,
    best: bestFull ? 'completo' : 'simplificado',
    effective_rate_bp: taxable > 0 ? Math.trunc((bestTax * 10_000) / taxable) : 0,
  };
}

registerMockHandlers(
  guarded({
    'tax.annual_draft': (a) => {
      const year = Number(a.year);
      if (!Number.isInteger(year) || year < 2000 || year > 2100) invalid('ano inválido');
      const input: AnnualInput = {
        year,
        employment: [{ income: 9_600_000, social_security: 1_000_000, withheld: 1_080_000 }],
        self_employed: {
          income: 0,
          abroad: 0,
          social_security: 0,
          alimony: 0,
          livro_caixa: 0,
          carne_leao: 0,
        },
        dependents: 1,
        medical: 600_000,
        education: [500_000],
        private_pension: 0,
        alimony: 0,
        donation_children: 0,
        donation_elderly: 0,
        complementary_tax: 0,
        foreign_tax_paid: 0,
        irrf_lei_11033: 0,
      };
      return {
        input,
        result: computeAnnual(input),
        irpfm: null,
        notes: ['Empresa de demonstração: holerites de 12 meses (mock).'],
      } satisfies AnnualDraft;
    },
    'tax.annual_compute': (a) => computeAnnual(a.input as AnnualInput),
    // Carnê-leão pela tabela mensal desde maio de 2025 e a redução de 2026 (como o núcleo).
    'tax.carne_leao': (a) => {
      const income = Number(a.income ?? 0);
      const legal =
        Number(a.social_security ?? 0) +
        Number(a.alimony ?? 0) +
        Number(a.livro_caixa ?? 0) +
        18_959 * Number(a.dependents ?? 0);
      const base = income - Math.max(legal, 60_720);
      const brackets: [number, number, number][] = [
        [242_880, 0, 0],
        [282_665, 7.5, 18_216],
        [375_105, 15, 39_416],
        [466_468, 22.5, 67_549],
        [Number.MAX_SAFE_INTEGER, 27.5, 90_873],
      ];
      const b = brackets.find(([u]) => base <= u)!;
      const table = base <= 0 ? 0 : Math.max(Math.round((base * b[1]) / 100) - b[2], 0);
      const month = String(a.month);
      const cut =
        month >= '2026-01'
          ? income <= 500_000
            ? 31_289
            : income < 735_000
              ? Math.max(97_862 - Math.round(0.133145 * income), 0)
              : 0
          : 0;
      const tax = Math.max(table - Math.min(cut, table), 0);
      const y = Number(month.slice(0, 4));
      const m = Number(month.slice(5, 7));
      const d = new Date(Date.UTC(y, m + 1, 0));
      while (d.getUTCDay() === 0 || d.getUTCDay() === 6) d.setUTCDate(d.getUTCDate() - 1);
      return {
        tax,
        revenue_code: '0190',
        due_date: d.toISOString().slice(0, 10),
        below_minimum: tax > 0 && tax < 1_000,
      };
    },
  }),
);
