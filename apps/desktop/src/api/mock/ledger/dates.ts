/** Datas `YYYY-MM-DD` sem fuso, com a mesma aritmética do núcleo (`crate::time`). */
import { mockToday } from './clock';
import { fail } from './errors';

export interface Ymd {
  y: number;
  m: number;
  d: number;
}

export function parseYmd(iso: string): Ymd {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  const y = Number(match?.[1]);
  const m = Number(match?.[2]);
  const d = Number(match?.[3]);
  if (!match || m < 1 || m > 12 || d < 1 || d > daysInMonth(y, m)) {
    fail('invalid', `data inválida: ${iso} (use AAAA-MM-DD)`);
  }
  return { y, m, d };
}

export function isIsoDate(iso: string): boolean {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  if (!match) return false;
  const m = Number(match[2]);
  const d = Number(match[3]);
  return m >= 1 && m <= 12 && d >= 1 && d <= daysInMonth(Number(match[1]), m);
}

export function daysInMonth(y: number, m: number): number {
  return new Date(Date.UTC(y, m, 0)).getUTCDate();
}

const pad = (n: number, w = 2) => String(n).padStart(w, '0');

export function fmt({ y, m, d }: Ymd): string {
  return `${pad(y, 4)}-${pad(m)}-${pad(d)}`;
}

export function clampDay(y: number, m: number, day: number): string {
  return fmt({ y, m, d: Math.min(day, daysInMonth(y, m)) });
}

/** Soma meses mantendo o dia (limitado ao fim do mês). */
export function addMonths(iso: string, months: number): string {
  const { y, m, d } = parseYmd(iso);
  const total = y * 12 + (m - 1) + months;
  const ny = Math.floor(total / 12);
  const nm = (total % 12) + 1;
  return clampDay(ny, nm, d);
}

export function addDays(iso: string, days: number): string {
  const { y, m, d } = parseYmd(iso);
  const t = new Date(Date.UTC(y, m - 1, d + days));
  return fmt({ y: t.getUTCFullYear(), m: t.getUTCMonth() + 1, d: t.getUTCDate() });
}

export function diffDays(a: string, b: string): number {
  const pa = parseYmd(a);
  const pb = parseYmd(b);
  return Math.round((Date.UTC(pa.y, pa.m - 1, pa.d) - Date.UTC(pb.y, pb.m - 1, pb.d)) / 86_400_000);
}

/** Dia da semana: 0 = segunda … 6 = domingo. */
export function weekdayMon0(iso: string): number {
  const { y, m, d } = parseYmd(iso);
  return (new Date(Date.UTC(y, m - 1, d)).getUTCDay() + 6) % 7;
}

export function monthOf(iso: string): string {
  return iso.slice(0, 7);
}

export function firstOfMonth(iso: string): string {
  return `${iso.slice(0, 7)}-01`;
}

export function monthEnd(iso: string): string {
  const { y, m } = parseYmd(firstOfMonth(iso));
  return clampDay(y, m, 31);
}

export function parseMonth(month: string): { y: number; m: number } {
  const match = /^(\d{4})-(\d{2})$/.exec(month);
  const y = Number(match?.[1]);
  const m = Number(match?.[2]);
  if (!match || m < 1 || m > 12) fail('invalid', `mês inválido: ${month} (use AAAA-MM)`);
  return { y, m };
}

/** "Hoje" do mock: a data local real ou a fixada nos testes (os dados são relativos a ela). */
export function today(): string {
  return mockToday();
}

/** Instante técnico UTC (RFC 3339 com milissegundos), estritamente crescente. */
let lastNow = 0;
export function nowUtc(): string {
  const t = Math.max(Date.now(), lastNow + 1);
  lastNow = t;
  return new Date(t).toISOString();
}

/** `2026-10-14` → `14/10/2026` (mensagens e motivos, como `br_date`). */
export function brDate(iso: string): string {
  return isIsoDate(iso) ? `${iso.slice(8, 10)}/${iso.slice(5, 7)}/${iso.slice(0, 4)}` : iso;
}
