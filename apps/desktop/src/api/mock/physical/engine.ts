/**
 * Motor de avaliação do patrimônio físico no mock, como `physical/valuation.rs`: âncora na
 * última avaliação até a data (ou no custo de aquisição) e a regra do bem — correção por
 * índice (pró-rata no mês), depreciação composta, linear ou sem estimativa — aplicada à
 * âncora e às benfeitorias de capital posteriores.
 */
import type { ValueBasis } from '@/api/generated/ValueBasis';

import { addDays, diffDays } from '../ledger/dates';
import { brl } from '../ledger/text';
import { addMonths, monthEnd, monthOf, monthStart, seriesPoints } from '../marketSim';

export type Rule =
  | { kind: 'flat' }
  | { kind: 'index'; name: string; series: string }
  | { kind: 'declining'; ratePct: number; fipe: boolean }
  | { kind: 'straight_line'; lifeYears: number; residualPct: number };

export interface Appraisal {
  date: string;
  value: number;
  source: string;
}

export interface Model {
  currency: string;
  purchaseDate: string;
  initialCost: number;
  /** Benfeitorias de capital (data, valor) em ordem. */
  capital: { date: string; amount: number }[];
  appraisals: Appraisal[];
  rule: Rule;
  sale: { date: string; net: number } | null;
}

export interface Estimate {
  value: number;
  basis: ValueBasis;
  estimated: boolean;
  formula: string;
  source: string;
  anchorDate: string;
  anchorValue: number;
}

interface Factor {
  factor: number;
  estimated: boolean;
  note: string | null;
}

export const brDate = (iso: string) => `${iso.slice(8, 10)}/${iso.slice(5, 7)}/${iso.slice(0, 4)}`;
const brMonth = (iso: string) => `${iso.slice(5, 7)}/${iso.slice(0, 4)}`;
const brNum = (n: number, dp: number) => String(Number(n.toFixed(dp))).replace('.', ',');
const years = (from: string, to: string) => Math.max(diffDays(to, from), 0) / 365.25;

/** Fator do índice mensal entre duas datas; meses não publicados ficam sem correção. */
export function indexFactor(series: string, from: string, to: string): Factor {
  if (to <= from) return { factor: 1, estimated: false, note: null };
  const pts = seriesPoints(series);
  if (pts.length === 0)
    return { factor: 1, estimated: true, note: 'sem dados do índice: sincronize os índices' };
  const start = pts[0]!.date;
  const last = pts[pts.length - 1]!.date;
  const end = monthStart(monthOf(addMonths(last, 1)));
  const f = from > start ? from : start;
  const t = to < end ? to : end;
  let note: string | null = null;
  if (t < to) note = `índice publicado até ${brMonth(last)}; o período seguinte fica sem correção`;
  else if (f > from) note = `índice disponível desde ${brMonth(start)}`;
  if (t <= f) return { factor: 1, estimated: true, note };
  const byMonth = new Map(pts.map((p) => [monthOf(p.date), p]));
  let factor = 1;
  let estimated = note != null;
  for (let m = monthOf(f); m <= monthOf(addDays(t, -1)); m = monthOf(addMonths(monthStart(m), 1))) {
    const p = byMonth.get(m);
    if (!p) return { factor: 1, estimated: true, note: 'série do índice com meses faltando' };
    const s = m === monthOf(f) ? f : monthStart(m);
    const next = addDays(monthEnd(m), 1);
    const e = t < next ? t : next;
    const dim = diffDays(next, monthStart(m));
    const frac = diffDays(e, s) / dim;
    if (frac > 0) factor *= (1 + p.value / 100) ** frac;
    estimated ||= p.estimated;
  }
  return { factor, estimated, note };
}

function ruleFactor(m: Model, from: string, to: string, appraisalFrom: boolean): Factor {
  const plain = (factor: number): Factor => ({ factor, estimated: to > from, note: null });
  const r = m.rule;
  switch (r.kind) {
    case 'flat':
      return { factor: 1, estimated: false, note: null };
    case 'index': {
      const f = indexFactor(r.series, from, to);
      return { ...f, estimated: f.estimated || to > from };
    }
    case 'declining':
      return plain(Math.max(1 - r.ratePct / 100, 0) ** years(from, to));
    case 'straight_line': {
      const span = appraisalFrom ? r.lifeYears - years(m.purchaseDate, from) : r.lifeYears;
      if (span <= 0)
        return { factor: 1, estimated: false, note: 'vida útil já encerrada na avaliação' };
      const used = Math.min(years(from, to) / span, 1);
      return plain(1 - Math.max(1 - r.residualPct / 100, 0) * used);
    }
  }
}

function ruleText(m: Model, from: string, to: string, f: number): string {
  const r = m.rule;
  switch (r.kind) {
    case 'flat':
      return '';
    case 'index':
      return ` corrigido pelo ${r.name} de ${brDate(from)} a ${brDate(to)} (fator ${brNum(f, 4)})`;
    case 'declining':
      return ` depreciado a ${brNum(r.ratePct, 2)}% a.a. por ${brNum(years(from, to), 1)} anos (fator ${brNum(f, 4)})`;
    case 'straight_line':
      return ` depreciado linearmente em ${r.lifeYears} anos até ${brNum(r.residualPct, 2)}% (fator ${brNum(f, 4)})`;
  }
}

function ruleSource(m: Model): string {
  const r = m.rule;
  switch (r.kind) {
    case 'flat':
      return 'sem estimativa automática';
    case 'index':
      return `${r.name} (BCB SGS)`;
    case 'declining':
      return r.fipe
        ? `última consulta FIPE + depreciação de ${brNum(r.ratePct, 2)}% a.a.`
        : `depreciação de ${brNum(r.ratePct, 2)}% a.a.`;
    case 'straight_line':
      return `depreciação linear em ${r.lifeYears} anos`;
  }
}

export function costBasis(m: Model, at: string): number {
  return m.initialCost + m.capital.filter((c) => c.date <= at).reduce((s, c) => s + c.amount, 0);
}

const round = (v: number) => Math.max(Math.round(v), 0);

/** Valor estimado do bem na data `at`. */
export function valueAt(m: Model, at: string): Estimate {
  const zero = (basis: ValueBasis, formula: string, source: string): Estimate => ({
    value: 0,
    basis,
    estimated: false,
    formula,
    source,
    anchorDate: at,
    anchorValue: 0,
  });
  if (at < m.purchaseDate)
    return zero('not_owned', `comprado só em ${brDate(m.purchaseDate)}`, 'sem posse');
  if (m.sale && m.sale.date <= at)
    return zero(
      'sold',
      `vendido em ${brDate(m.sale.date)} por ${brl(m.sale.net, m.currency)} (líquido)`,
      'venda',
    );
  const anchor = [...m.appraisals].reverse().find((a) => a.date <= at) ?? null;
  const anchorDate = anchor?.date ?? m.purchaseDate;
  const anchorValue = anchor?.value ?? m.initialCost;
  const desc = anchor
    ? `avaliação de ${brDate(anchor.date)} (${anchor.source}): ${brl(anchor.value, m.currency)}`
    : `custo de aquisição de ${brl(m.initialCost, m.currency)} em ${brDate(m.purchaseDate)}`;
  const af = ruleFactor(m, anchorDate, at, anchor != null);
  let value = round(anchorValue * af.factor);
  let estimated = af.estimated;
  const notes = af.note ? [af.note] : [];
  const later = m.capital.filter((c) => c.date <= at && (anchor ? c.date > anchorDate : true));
  let laterCost = 0;
  let laterValue = 0;
  for (const c of later) {
    const from = c.date > m.purchaseDate ? c.date : m.purchaseDate;
    const f = ruleFactor(m, from, at, false);
    estimated ||= f.estimated;
    if (f.note && !notes.includes(f.note)) notes.push(f.note);
    laterCost += c.amount;
    laterValue += round(c.amount * f.factor);
  }
  value += laterValue;
  const flat = m.rule.kind === 'flat';
  const basis: ValueBasis =
    !anchor && flat
      ? 'cost'
      : anchor && (flat || at === anchorDate) && later.length === 0
        ? 'appraisal'
        : 'automatic';
  if (basis === 'cost' || (flat && later.length > 0)) estimated = true;
  let formula = desc + ruleText(m, anchorDate, at, af.factor);
  if (later.length > 0) {
    const what = later.length === 1 ? '1 benfeitoria' : `${later.length} benfeitorias`;
    formula +=
      flat || laterValue === laterCost
        ? ` + ${what} pelo custo (${brl(laterCost, m.currency)})`
        : ` + ${what} de ${brl(laterCost, m.currency)} pela mesma regra (${brl(laterValue, m.currency)})`;
  }
  if (basis !== 'appraisal' || later.length > 0 || !flat) formula += ` = ${brl(value, m.currency)}`;
  if (basis === 'cost')
    formula = `sem avaliação nem estimativa automática: ${formula}; registre uma avaliação para o valor de mercado`;
  for (const n of notes) formula += ` (${n})`;
  const source =
    basis === 'appraisal' && anchor
      ? `avaliação · ${anchor.source}`
      : basis === 'cost'
        ? 'custo de aquisição'
        : anchor
          ? `avaliação · ${anchor.source} + ${ruleSource(m)}`
          : ruleSource(m);
  return { value, basis, estimated, formula, source, anchorDate, anchorValue };
}
