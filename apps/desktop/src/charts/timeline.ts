/**
 * Linha do tempo de recorrências (funções puras): expande a regra (RuleSpec) em datas
 * dentro de uma janela e calcula a posição de cada ocorrência na trilha.
 */
import type { RuleSpec } from '@/api/generated/RuleSpec';
import {
  addDays,
  clampDay,
  daysBetween,
  monthOf,
  monthsBetween,
  parseIso,
  shiftMonth,
  weekday,
} from '@/lib/months';

const DAY_CODES = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'];

/**
 * Datas da regra a partir de `first` (a próxima ocorrência conhecida) até `until`.
 * Cobre as formas que a interface cria (diária, semanal com dias, mensal no dia, anual).
 */
export function expandOccurrences(
  spec: RuleSpec,
  first: string,
  until: string,
  limit = 400,
): string[] {
  const out: string[] = [];
  const interval = Math.max(1, spec.interval ?? 1);
  const push = (d: string) => {
    if (d >= first && d <= until && !out.includes(d)) out.push(d);
  };
  switch (spec.freq) {
    case 'DAILY':
      for (let d = first; d <= until && out.length < limit; d = addDays(d, interval)) push(d);
      break;
    case 'WEEKLY': {
      const days = spec.by_day?.length ? spec.by_day : [DAY_CODES[weekday(first)]!];
      for (
        let start = addDays(first, -weekday(first));
        start <= until && out.length < limit;
        start = addDays(start, 7 * interval)
      ) {
        for (let i = 0; i < 7; i++) {
          const d = addDays(start, i);
          if (days.includes(DAY_CODES[weekday(d)]!)) push(d);
        }
      }
      break;
    }
    case 'MONTHLY': {
      const day = spec.by_month_day ?? parseIso(first).d;
      for (const m of monthsBetween(monthOf(first), monthOf(until))) {
        if (out.length >= limit) break;
        const offset = monthsBetween(monthOf(first), m).length - 1;
        if (offset % interval !== 0) continue;
        const { y, m: mm } = parseIso(m);
        push(clampDay(y, mm, day));
      }
      break;
    }
    case 'YEARLY': {
      const { m: fm, d: fd } = parseIso(first);
      const month = spec.by_month ?? fm;
      const day = spec.by_month_day ?? fd;
      for (
        let y = parseIso(first).y;
        `${y}` <= until.slice(0, 4) && out.length < limit;
        y += interval
      ) {
        push(clampDay(y, month, day));
      }
      break;
    }
  }
  return out.sort();
}

/** Posição (0–100%) de uma data na janela. */
export function positionIn(date: string, from: string, to: string): number {
  const span = Math.max(1, daysBetween(from, to));
  return Math.min(100, Math.max(0, (daysBetween(from, date) / span) * 100));
}

/** Primeiros dias de cada mês dentro da janela (marcas da régua). */
export function monthTicks(from: string, to: string): string[] {
  const out: string[] = [];
  for (let m = shiftMonth(monthOf(from), 1); `${m}-01` <= to; m = shiftMonth(m, 1))
    out.push(`${m}-01`);
  return out;
}
