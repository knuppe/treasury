/**
 * Linha do tempo e estatísticas do projeto (funções puras) a partir das entradas
 * (`projects.entries`): receitas e despesas por período, os acumulados e o resultado.
 * Projeto curto (até 62 dias) vai dia a dia; o resto, mês a mês.
 */
import type { ProjectEntry } from '@/api/generated/ProjectEntry';
import type { ProjectView } from '@/api/generated/ProjectView';
import { addDays, addMonthsToDate, daysBetween, monthOf, monthsBetween } from '@/lib/months';

export type Grain = 'day' | 'month';

export interface ProjectTimeline {
  grain: Grain;
  /** `YYYY-MM-DD` (dia) ou `YYYY-MM` (mês). */
  periods: string[];
  /** Receitas e despesas de cada período, positivas (estorno abate a despesa). */
  income: number[];
  expense: number[];
  cumIncome: number[];
  cumExpense: number[];
  /** Receitas − despesas acumuladas. */
  cumNet: number[];
}

/** Até quantos dias o projeto vai dia a dia. */
export const DAILY_SPAN = 62;

/** Valor da entrada do lado dela: receita positiva; despesa positiva (estorno negativo). */
export const sideValue = (e: ProjectEntry) => (e.flow === 'income' ? e.amount : -e.amount);

/**
 * Do primeiro dia (início do projeto ou primeira entrada) ao último (última entrada ou, com o
 * projeto aberto, hoje, sem passar do fim previsto). Sem entradas: `null`.
 */
export function timelineRange(
  p: Pick<ProjectView, 'start_date' | 'end_date' | 'status'>,
  entries: readonly ProjectEntry[],
  today: string,
): { from: string; to: string } | null {
  if (entries.length === 0) return null;
  const dates = entries.map((e) => e.date).sort();
  const first = dates[0]!;
  const last = dates[dates.length - 1]!;
  const from = p.start_date && p.start_date < first ? p.start_date : first;
  let to = last;
  if (p.status !== 'done') {
    const open = p.end_date && p.end_date < today ? p.end_date : today;
    if (open > to) to = open;
  }
  return { from, to };
}

export function buildTimeline(
  p: Pick<ProjectView, 'start_date' | 'end_date' | 'status'>,
  entries: readonly ProjectEntry[],
  today: string,
): ProjectTimeline | null {
  const range = timelineRange(p, entries, today);
  if (!range) return null;
  const grain: Grain = daysBetween(range.from, range.to) <= DAILY_SPAN ? 'day' : 'month';
  const periods: string[] = [];
  if (grain === 'day') {
    for (let d = range.from; d <= range.to; d = addDays(d, 1)) periods.push(d);
  } else {
    periods.push(...monthsBetween(monthOf(range.from), monthOf(range.to)));
  }
  const index = new Map(periods.map((k, i) => [k, i]));
  const income = periods.map(() => 0);
  const expense = periods.map(() => 0);
  for (const e of entries) {
    const i = index.get(grain === 'day' ? e.date : monthOf(e.date));
    if (i == null) continue;
    if (e.flow === 'income') income[i]! += e.amount;
    else expense[i]! -= e.amount;
  }
  const cum = (xs: number[]) => {
    let s = 0;
    return xs.map((v) => (s += v));
  };
  const cumIncome = cum(income);
  const cumExpense = cum(expense);
  return {
    grain,
    periods,
    income,
    expense,
    cumIncome,
    cumExpense,
    cumNet: cumIncome.map((v, i) => v - cumExpense[i]!),
  };
}

export interface ProjectStats {
  /** Meses com o projeto andando (do início à última data), no mínimo 1. */
  months: number;
  /** Gasto médio por mês nesse período. */
  monthlyPace: number;
  biggestExpense: ProjectEntry | null;
  biggestIncome: ProjectEntry | null;
  /** Período de maior gasto (dia ou mês, conforme a linha do tempo). */
  peak: { period: string; value: number } | null;
  /** Fração do orçamento já gasta (`null` sem orçamento). */
  budgetUsed: number | null;
  /** Com orçamento sobrando e o projeto andando: quando o orçamento acaba no ritmo atual. */
  budgetEnds: string | null;
}

export function projectStats(
  p: ProjectView,
  entries: readonly ProjectEntry[],
  timeline: ProjectTimeline | null,
  today: string,
): ProjectStats {
  const range = timelineRange(p, entries, today);
  const days = range ? daysBetween(range.from, range.to) + 1 : 0;
  const months = Math.max(1, days / 30.4375);
  const monthlyPace = range ? Math.round(p.spent / months) : 0;
  const pick = (flow: ProjectEntry['flow']) =>
    entries
      .filter((e) => e.flow === flow && sideValue(e) > 0)
      .reduce<ProjectEntry | null>((b, e) => (!b || sideValue(e) > sideValue(b) ? e : b), null);
  let peak: ProjectStats['peak'] = null;
  for (const [i, v] of (timeline?.expense ?? []).entries())
    if (v > 0 && (!peak || v > peak.value)) peak = { period: timeline!.periods[i]!, value: v };
  const budgetUsed = p.budget ? p.spent / p.budget : null;
  let budgetEnds: string | null = null;
  if (
    p.status !== 'done' &&
    p.remaining != null &&
    p.remaining > 0 &&
    monthlyPace > 0 &&
    range &&
    months >= 1
  ) {
    const ahead = Math.ceil(p.remaining / monthlyPace);
    budgetEnds = monthOf(addMonthsToDate(range.to, ahead));
  }
  return {
    months: Math.max(1, Math.round(months)),
    monthlyPace,
    biggestExpense: pick('expense'),
    biggestIncome: pick('income'),
    peak,
    budgetUsed,
    budgetEnds,
  };
}
