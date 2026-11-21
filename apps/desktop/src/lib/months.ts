/**
 * Aritmética de datas ISO (`YYYY-MM-DD`) e meses (`YYYY-MM`) sem fuso, como o núcleo:
 * tudo em UTC por baixo para não escorregar um dia no horário de verão.
 */

const pad = (n: number) => String(n).padStart(2, '0');

export function isoDate(y: number, m: number, d: number): string {
  return `${y}-${pad(m)}-${pad(d)}`;
}

export function parseIso(iso: string): { y: number; m: number; d: number } {
  const [y, m, d] = iso.split('-').map(Number);
  return { y: y ?? 1970, m: m ?? 1, d: d ?? 1 };
}

export function daysInMonth(y: number, m: number): number {
  return new Date(Date.UTC(y, m, 0)).getUTCDate();
}

/** Dia limitado ao fim do mês (31 em fevereiro vira 28/29). */
export function clampDay(y: number, m: number, d: number): string {
  return isoDate(y, m, Math.min(d, daysInMonth(y, m)));
}

export function addDays(iso: string, n: number): string {
  const { y, m, d } = parseIso(iso);
  const t = new Date(Date.UTC(y, m - 1, d + n));
  return isoDate(t.getUTCFullYear(), t.getUTCMonth() + 1, t.getUTCDate());
}

/** Soma meses mantendo o dia (limitado ao fim do mês). */
export function addMonthsToDate(iso: string, n: number): string {
  const { y, m, d } = parseIso(iso);
  const idx = y * 12 + (m - 1) + n;
  const ny = Math.floor(idx / 12);
  const nm = (idx % 12) + 1;
  return clampDay(ny, nm, d);
}

export function monthOf(iso: string): string {
  return iso.slice(0, 7);
}

export function monthIndex(month: string): number {
  const { y, m } = parseIso(month);
  return y * 12 + (m - 1);
}

export function monthFromIndex(i: number): string {
  return `${Math.floor(i / 12)}-${pad((i % 12) + 1)}`;
}

export function shiftMonth(month: string, n: number): string {
  return monthFromIndex(monthIndex(month) + n);
}

/** Meses de `from` a `to`, inclusive. */
export function monthsBetween(from: string, to: string): string[] {
  const a = monthIndex(from);
  const b = monthIndex(to);
  const out: string[] = [];
  for (let i = a; i <= b; i++) out.push(monthFromIndex(i));
  return out;
}

/** Primeiro e último dia do mês. */
export function monthBounds(month: string): { from: string; to: string } {
  const { y, m } = parseIso(month);
  return { from: isoDate(y, m, 1), to: isoDate(y, m, daysInMonth(y, m)) };
}

/** Dias entre duas datas (b − a). */
export function daysBetween(a: string, b: string): number {
  const pa = parseIso(a);
  const pb = parseIso(b);
  return Math.round((Date.UTC(pb.y, pb.m - 1, pb.d) - Date.UTC(pa.y, pa.m - 1, pa.d)) / 86_400_000);
}

/** Dia da semana (0 = domingo). */
export function weekday(iso: string): number {
  const { y, m, d } = parseIso(iso);
  return new Date(Date.UTC(y, m - 1, d)).getUTCDay();
}

export function currentMonth(today: string): string {
  return monthOf(today);
}

/** Fração do mês já decorrida em `today` (meses passados = 1, futuros = 0), como o núcleo. */
export function monthElapsed(month: string, today: string): number {
  const cur = monthIndex(monthOf(today));
  const m = monthIndex(month);
  if (m < cur) return 1;
  if (m > cur) return 0;
  const { y, m: mm, d } = parseIso(today);
  return d / daysInMonth(y, mm);
}
