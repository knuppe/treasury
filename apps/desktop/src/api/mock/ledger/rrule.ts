/**
 * Recorrências no mock: o mesmo subconjunto de RRULE do núcleo
 * (`recurrence/rrule.rs`), com validação, normalização, ocorrências e a
 * descrição em português ("todo mês no dia 5").
 */
import type { RuleSpec } from '@/api/generated/RuleSpec';

import { addDays, addMonths, clampDay, parseYmd, weekdayMon0 } from './dates';
import { invalid } from './errors';

const CODES = ['MO', 'TU', 'WE', 'TH', 'FR', 'SA', 'SU'] as const;

function weekday(code: string): number {
  const i = CODES.indexOf(code.trim().toUpperCase() as (typeof CODES)[number]);
  if (i < 0) invalid(`dia da semana inválido: ${code}`);
  return i;
}

function daysOf(spec: RuleSpec): number[] {
  return [...new Set((spec.by_day ?? []).map(weekday))].sort((a, b) => a - b);
}

export function validate(spec: RuleSpec): void {
  const interval = spec.interval ?? 1;
  if (interval < 1 || interval > 999) invalid('o intervalo deve ficar entre 1 e 999');
  if (spec.count != null && (spec.count < 1 || spec.count > 10_000)) {
    invalid('o número de ocorrências deve ficar entre 1 e 10000');
  }
  if (spec.by_month_day != null) {
    if (spec.by_month_day < 1 || spec.by_month_day > 31)
      invalid('o dia do mês deve ficar entre 1 e 31');
    if (spec.freq !== 'MONTHLY' && spec.freq !== 'YEARLY') {
      invalid('o dia do mês só vale para recorrências mensais ou anuais');
    }
  }
  if (spec.by_month != null) {
    if (spec.by_month < 1 || spec.by_month > 12) invalid('o mês deve ficar entre 1 e 12');
    if (spec.freq !== 'YEARLY') invalid('o mês só vale para recorrências anuais');
  }
  if (spec.by_day != null) {
    if (spec.by_day.length === 0) invalid('escolha pelo menos um dia da semana');
    if (spec.freq !== 'WEEKLY' && spec.freq !== 'DAILY') {
      invalid('dias da semana só valem para recorrências diárias ou semanais');
    }
    daysOf(spec);
  }
}

export function normalize(spec: RuleSpec, start: string): RuleSpec {
  validate(spec);
  const s: RuleSpec = { ...spec, interval: spec.interval ?? 1 };
  const { m, d } = parseYmd(start);
  switch (s.freq) {
    case 'DAILY':
      if (s.by_day) s.by_day = daysOf(spec).map((i) => CODES[i]!);
      break;
    case 'WEEKLY':
      s.by_day = (s.by_day ? daysOf(spec) : [weekdayMon0(start)]).map((i) => CODES[i]!);
      break;
    case 'MONTHLY':
      s.by_month_day = s.by_month_day ?? d;
      break;
    case 'YEARLY':
      s.by_month = s.by_month ?? m;
      s.by_month_day = s.by_month_day ?? d;
      break;
  }
  return s;
}

export function toRrule(spec: RuleSpec): string {
  const parts = [`FREQ=${spec.freq}`, `INTERVAL=${spec.interval ?? 1}`];
  if (spec.by_month != null) parts.push(`BYMONTH=${spec.by_month}`);
  if (spec.by_month_day != null) parts.push(`BYMONTHDAY=${spec.by_month_day}`);
  if (spec.by_day?.length) parts.push(`BYDAY=${spec.by_day.join(',')}`);
  if (spec.count != null) parts.push(`COUNT=${spec.count}`);
  return parts.join(';');
}

export function parseRrule(text: string): RuleSpec {
  const spec: Partial<RuleSpec> & { freq?: RuleSpec['freq'] } = {};
  for (const part of text.split(';').filter(Boolean)) {
    const [k, v = ''] = part.split('=');
    switch (k) {
      case 'FREQ':
        spec.freq = v as RuleSpec['freq'];
        break;
      case 'INTERVAL':
        spec.interval = Number(v);
        break;
      case 'BYMONTH':
        spec.by_month = Number(v);
        break;
      case 'BYMONTHDAY':
        spec.by_month_day = Number(v);
        break;
      case 'BYDAY':
        spec.by_day = v.split(',');
        break;
      case 'COUNT':
        spec.count = Number(v);
        break;
    }
  }
  if (!spec.freq) invalid('a regra precisa de FREQ');
  return spec as RuleSpec;
}

/** Percorre as ocorrências a partir de `start`; `f` devolve `false` para parar. */
function walk(spec: RuleSpec, start: string, end: string | null, f: (d: string) => boolean): void {
  const s = normalize(spec, start);
  const interval = s.interval ?? 1;
  let emitted = 0;
  const emit = (d: string): boolean => {
    if (end && d > end) return false;
    if (s.count != null && emitted >= s.count) return false;
    emitted++;
    return f(d);
  };
  const days = daysOf(s);
  const LIMIT = 20_000;
  switch (s.freq) {
    case 'DAILY':
      for (let k = 0; k < LIMIT; k++) {
        const d = addDays(start, k * interval);
        if (days.length > 0 && !days.includes(weekdayMon0(d))) {
          if (end && d > end) break;
          continue;
        }
        if (!emit(d)) break;
      }
      break;
    case 'WEEKLY': {
      const week0 = addDays(start, -weekdayMon0(start));
      outer: for (let k = 0; k < LIMIT; k++) {
        const monday = addDays(week0, k * 7 * interval);
        for (const wd of days) {
          const d = addDays(monday, wd);
          if (d < start) continue;
          if (!emit(d)) break outer;
        }
      }
      break;
    }
    case 'MONTHLY': {
      const day = s.by_month_day ?? parseYmd(start).d;
      const first = `${start.slice(0, 7)}-01`;
      for (let k = 0; k < LIMIT; k++) {
        const m = parseYmd(addMonths(first, k * interval));
        const d = clampDay(m.y, m.m, day);
        if (d < start) continue;
        if (!emit(d)) break;
      }
      break;
    }
    case 'YEARLY': {
      const { y } = parseYmd(start);
      const month = s.by_month ?? parseYmd(start).m;
      const day = s.by_month_day ?? parseYmd(start).d;
      for (let k = 0; k < LIMIT; k++) {
        const d = clampDay(y + k * interval, month, day);
        if (d < start) continue;
        if (!emit(d)) break;
      }
      break;
    }
  }
}

export function occurrencesBetween(
  spec: RuleSpec,
  start: string,
  end: string | null,
  from: string,
  until: string,
): string[] {
  const out: string[] = [];
  walk(spec, start, end, (d) => {
    if (d > until) return false;
    if (d >= from) out.push(d);
    return true;
  });
  return out;
}

export function nextN(
  spec: RuleSpec,
  start: string,
  end: string | null,
  from: string,
  n: number,
): string[] {
  const out: string[] = [];
  if (n <= 0) return out;
  walk(spec, start, end, (d) => {
    if (d >= from) out.push(d);
    return out.length < n;
  });
  return out;
}

const MONTHS = [
  'janeiro', 'fevereiro', 'março', 'abril', 'maio', 'junho',
  'julho', 'agosto', 'setembro', 'outubro', 'novembro', 'dezembro',
]; // prettier-ignore

const PLURAL: [string, boolean][] = [
  ['segundas', true],
  ['terças', true],
  ['quartas', true],
  ['quintas', true],
  ['sextas', true],
  ['sábados', false],
  ['domingos', false],
];

function joinPt(items: string[]): string {
  if (items.length <= 1) return items[0] ?? '';
  return `${items.slice(0, -1).join(', ')} e ${items[items.length - 1]}`;
}

function daysPhrase(days: number[]): string {
  const names = days.map((d) => PLURAL[d]!);
  if (names.every(([, f]) => f)) return `às ${joinPt(names.map(([n]) => n))}`;
  if (names.every(([, f]) => !f)) return `aos ${joinPt(names.map(([n]) => n))}`;
  return joinPt(names.map(([n, f]) => `${f ? 'às' : 'aos'} ${n}`));
}

const every = (interval: number, one: string, many: string) =>
  interval <= 1 ? one : `a cada ${interval} ${many}`;

export function human(spec: RuleSpec): string {
  const interval = spec.interval ?? 1;
  let days: number[];
  try {
    days = daysOf(spec);
  } catch {
    days = [];
  }
  let s: string;
  switch (spec.freq) {
    case 'DAILY': {
      const base = every(interval, 'todo dia', 'dias');
      if (days.length === 0 || days.length === 7) s = base;
      else if (interval === 1 && days.join() === '0,1,2,3,4') s = 'todo dia útil (segunda a sexta)';
      else s = `${base}, só ${daysPhrase(days)}`;
      break;
    }
    case 'WEEKLY': {
      const base = every(interval, 'toda semana', 'semanas');
      s = days.length === 0 ? base : `${base} ${daysPhrase(days)}`;
      break;
    }
    case 'MONTHLY': {
      const base = every(interval, 'todo mês', 'meses');
      s =
        spec.by_month_day === 31
          ? `${base} no último dia`
          : spec.by_month_day != null
            ? `${base} no dia ${spec.by_month_day}`
            : base;
      break;
    }
    case 'YEARLY': {
      const base = every(interval, 'todo ano', 'anos');
      const month = spec.by_month != null ? MONTHS[spec.by_month - 1] : undefined;
      const d = spec.by_month_day;
      if (d === 1 && month) s = `${base} em 1º de ${month}`;
      else if (d != null && month) s = `${base} em ${d} de ${month}`;
      else if (month) s = `${base} em ${month}`;
      else if (d === 31) s = `${base} no último dia do mês`;
      else if (d != null) s = `${base} no dia ${d}`;
      else s = base;
      break;
    }
  }
  if (spec.count === 1) s += ', uma vez';
  else if (spec.count != null) s += `, ${spec.count} vezes`;
  return s;
}

function divRound(a: number, b: number): number {
  const q = a / b;
  return Math.sign(q) * Math.round(Math.abs(q) + 1e-9);
}

export function monthlyEquivalent(spec: RuleSpec, amount: number): number {
  const interval = Math.max(spec.interval ?? 1, 1);
  let n: number;
  try {
    n = daysOf(spec).length;
  } catch {
    n = 0;
  }
  switch (spec.freq) {
    case 'DAILY':
      return n > 0 && n < 7
        ? divRound(amount * n * 52, 12 * interval)
        : divRound(amount * 365, 12 * interval);
    case 'WEEKLY':
      return divRound(amount * Math.max(n, 1) * 52, 12 * interval);
    case 'MONTHLY':
      return divRound(amount, interval);
    case 'YEARLY':
      return divRound(amount, 12 * interval);
  }
}
