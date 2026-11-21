/**
 * Simulador de mercado do núcleo falso (sem comandos próprios): calendário, cotações
 * diárias determinísticas por ativo, câmbio USD/BRL e séries de índices (IPCA, CDI,
 * Selic, IGP-M, M2, US CPI, US M2, Fed Funds, Ibovespa, S&P 500).
 *
 * Tudo é gerado a partir de regras e da data de hoje, para que o mock continue
 * plausível em qualquer dia. Os números são fictícios (próximos da realidade de
 * 2024–2026), nunca dados de mercado de verdade.
 */
import { todayIso } from '@/lib/format';

import { countryDeflator } from './catalog';

// ── Datas ────────────────────────────────────────────────────────

export function mockToday(): string {
  return todayIso();
}

export function toDate(iso: string): Date {
  const [y, m, d] = iso.split('-').map(Number) as [number, number, number];
  return new Date(Date.UTC(y, m - 1, d));
}

export function iso(date: Date): string {
  return date.toISOString().slice(0, 10);
}

export function addDays(isoDate: string, days: number): string {
  const d = toDate(isoDate);
  d.setUTCDate(d.getUTCDate() + days);
  return iso(d);
}

export function addMonths(isoDate: string, months: number): string {
  const d = toDate(isoDate);
  const day = d.getUTCDate();
  d.setUTCDate(1);
  d.setUTCMonth(d.getUTCMonth() + months);
  const last = new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 0)).getUTCDate();
  d.setUTCDate(Math.min(day, last));
  return iso(d);
}

export function daysBetween(a: string, b: string): number {
  return Math.round((toDate(b).getTime() - toDate(a).getTime()) / 86_400_000);
}

export function monthOf(isoDate: string): string {
  return isoDate.slice(0, 7);
}

export function monthStart(month: string): string {
  return `${month}-01`;
}

export function monthEnd(month: string): string {
  const [y, m] = month.split('-').map(Number) as [number, number];
  return iso(new Date(Date.UTC(y, m, 0)));
}

export function isBusinessDay(isoDate: string): boolean {
  const wd = toDate(isoDate).getUTCDay();
  return wd !== 0 && wd !== 6;
}

/** Dias úteis (seg–sex) em [from, to]. */
export function businessDays(from: string, to: string): string[] {
  const out: string[] = [];
  for (let d = from; d <= to; d = addDays(d, 1)) if (isBusinessDay(d)) out.push(d);
  return out;
}

/** Último dia útil ≤ data. */
export function previousBusinessDay(isoDate: string): string {
  let d = isoDate;
  while (!isBusinessDay(d)) d = addDays(d, -1);
  return d;
}

// ── Aleatório determinístico ─────────────────────────────────────

export function hashSeed(text: string): number {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

export function prng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function gaussian(rand: () => number): number {
  const u = Math.max(rand(), 1e-12);
  const v = rand();
  return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
}

/** Início do histórico simulado e a data em que os níveis-alvo são atingidos. */
export const SIM_START = '2023-12-01';
export const SIM_ANCHOR = '2026-10-14';

export interface WalkSpec {
  /** Nível no início (SIM_START). */
  start: number;
  /** Nível em SIM_ANCHOR (ponte browniana até lá; depois, passeio sem tendência). */
  anchor: number;
  /** Volatilidade anual. */
  vol: number;
}

/**
 * Passeio aleatório diário (dias úteis) de SIM_START até hoje, preso aos níveis
 * `start` e `anchor` por uma ponte browniana.
 */
export function walk(key: string, spec: WalkSpec, to = mockToday()): Map<string, number> {
  const rand = prng(hashSeed(key));
  const end = to > SIM_ANCHOR ? to : SIM_ANCHOR;
  const days = businessDays(SIM_START, end);
  const anchorIndex = days.findIndex((d) => d >= SIM_ANCHOR);
  const n = anchorIndex < 0 ? days.length - 1 : anchorIndex;
  const sigma = spec.vol / Math.sqrt(252);
  const w: number[] = [0];
  for (let i = 1; i < days.length; i++) w.push(w[i - 1]! + gaussian(rand) * sigma);
  const l0 = Math.log(spec.start);
  const l1 = Math.log(spec.anchor);
  const out = new Map<string, number>();
  days.forEach((d, i) => {
    if (d > to) return;
    let level: number;
    if (i <= n) {
      const t = n === 0 ? 1 : i / n;
      level = l0 + (l1 - l0) * t + (w[i]! - w[n]! * t);
    } else {
      level = l1 + (w[i]! - w[n]!);
    }
    out.set(d, Math.exp(level));
  });
  return out;
}

/** Valor da série na data ou no último dia anterior com valor. */
export function valueOn(series: Map<string, number>, date: string): [string, number] | null {
  let d = date;
  for (let i = 0; i < 15; i++) {
    const v = series.get(d);
    if (v !== undefined) return [d, v];
    d = addDays(d, -1);
  }
  // Fora da janela de busca rápida: procura o último anterior.
  let best: [string, number] | null = null;
  for (const [k, v] of series) if (k <= date && (!best || k > best[0])) best = [k, v];
  return best;
}

// ── Números ──────────────────────────────────────────────────────

/** Argumento de texto (strings e números; o resto vira vazio). */
export function txt(v: unknown): string {
  return typeof v === 'string' ? v : typeof v === 'number' ? String(v) : '';
}

/** Número → decimal em texto (até `dp` casas, sem zeros sobrando). */
export function dec(x: number, dp = 8): string {
  if (!Number.isFinite(x)) return '0';
  const s = x.toFixed(dp);
  const t = s.includes('.') ? s.replace(/0+$/, '').replace(/\.$/, '') : s;
  return t === '-0' ? '0' : t;
}

export function cents(x: number): number {
  const v = Math.round(Math.abs(x) * 100) * Math.sign(x);
  return v === 0 ? 0 : v;
}

// ── Câmbio ───────────────────────────────────────────────────────

let usdCache: { to: string; series: Map<string, number> } | null = null;

function usdBrl(): Map<string, number> {
  const to = mockToday();
  if (!usdCache || usdCache.to !== to) {
    usdCache = { to, series: walk('fx:USDBRL', { start: 4.92, anchor: 5.38, vol: 0.13 }, to) };
  }
  return usdCache.series;
}

/** 1 `ccy` = x BRL na data (PTAX venda simulada), com a data usada. */
export function fxToBrl(
  ccy: string,
  date: string,
): { rate: number; date: string; estimated: boolean } | null {
  if (ccy === 'BRL') return { rate: 1, date, estimated: false };
  if (ccy !== 'USD') return null;
  const hit = valueOn(usdBrl(), date);
  if (!hit) return null;
  return {
    rate: Number(hit[1].toFixed(4)),
    date: hit[0],
    estimated: daysBetween(hit[0], date) > 7,
  };
}

export function usdBrlPoints(): [string, number][] {
  return [...usdBrl().entries()].map(([d, v]) => [d, Number(v.toFixed(4))]);
}

// ── Índices ──────────────────────────────────────────────────────

export interface SeriesDef {
  id: string;
  name: string;
  kind: 'monthly_rate' | 'daily_rate' | 'level' | 'annual_rate';
  unit: string;
  frequency: 'daily' | 'monthly';
  country: 'BR' | 'US';
  source: string;
}

export const SERIES: readonly SeriesDef[] = [
  {
    id: 'cdi',
    name: 'CDI',
    kind: 'daily_rate',
    unit: '% a.d.',
    frequency: 'daily',
    country: 'BR',
    source: 'bcb_sgs:12',
  },
  {
    id: 'ibov',
    name: 'Ibovespa',
    kind: 'level',
    unit: 'pontos',
    frequency: 'daily',
    country: 'BR',
    source: 'yahoo:^BVSP',
  },
  {
    id: 'igpm',
    name: 'IGP-M',
    kind: 'monthly_rate',
    unit: '% a.m.',
    frequency: 'monthly',
    country: 'BR',
    source: 'bcb_sgs:189',
  },
  {
    id: 'incc',
    name: 'INCC',
    kind: 'monthly_rate',
    unit: '% a.m.',
    frequency: 'monthly',
    country: 'BR',
    source: 'bcb_sgs:192',
  },
  {
    id: 'ipca',
    name: 'IPCA',
    kind: 'monthly_rate',
    unit: '% a.m.',
    frequency: 'monthly',
    country: 'BR',
    source: 'bcb_sgs:433',
  },
  {
    id: 'm2_br',
    name: 'M2 (meios de pagamento amplos)',
    kind: 'level',
    unit: 'R$ mil',
    frequency: 'monthly',
    country: 'BR',
    source: 'bcb_sgs:27810',
  },
  {
    id: 'selic',
    name: 'Selic diária',
    kind: 'daily_rate',
    unit: '% a.d.',
    frequency: 'daily',
    country: 'BR',
    source: 'bcb_sgs:11',
  },
  {
    id: 'selic_meta',
    name: 'Selic meta',
    kind: 'annual_rate',
    unit: '% a.a.',
    frequency: 'daily',
    country: 'BR',
    source: 'bcb_sgs:432',
  },
  {
    id: 'tr',
    name: 'TR',
    kind: 'monthly_rate',
    unit: '% a.m.',
    frequency: 'monthly',
    country: 'BR',
    source: 'bcb_sgs:226',
  },
  {
    id: 'fed_funds',
    name: 'Fed Funds',
    kind: 'annual_rate',
    unit: '% a.a.',
    frequency: 'monthly',
    country: 'US',
    source: 'fred:FEDFUNDS',
  },
  {
    id: 'sp500',
    name: 'S&P 500',
    kind: 'level',
    unit: 'pontos',
    frequency: 'daily',
    country: 'US',
    source: 'yahoo:^GSPC',
  },
  {
    id: 'us_cpi',
    name: 'US CPI-U',
    kind: 'level',
    unit: 'índice',
    frequency: 'monthly',
    country: 'US',
    source: 'fred:CPIAUCSL',
  },
  {
    id: 'us_m2',
    name: 'US M2',
    kind: 'level',
    unit: 'US$ bi',
    frequency: 'monthly',
    country: 'US',
    source: 'fred:M2SL',
  },
];

export interface SimPoint {
  date: string;
  value: number;
  estimated: boolean;
}

/** IPCA mensal (% a.m.): 2023–2025 próximos dos oficiais; depois, fictícios. */
const IPCA: Record<string, number> = {
  '2023-01': 0.53,
  '2023-02': 0.84,
  '2023-03': 0.71,
  '2023-04': 0.61,
  '2023-05': 0.23,
  '2023-06': -0.08,
  '2023-07': 0.12,
  '2023-08': 0.23,
  '2023-09': 0.26,
  '2023-10': 0.24,
  '2023-11': 0.28,
  '2023-12': 0.56,
  '2024-01': 0.42,
  '2024-02': 0.83,
  '2024-03': 0.16,
  '2024-04': 0.38,
  '2024-05': 0.46,
  '2024-06': 0.21,
  '2024-07': 0.38,
  '2024-08': -0.02,
  '2024-09': 0.44,
  '2024-10': 0.56,
  '2024-11': 0.39,
  '2024-12': 0.52,
  '2025-01': 0.16,
  '2025-02': 1.31,
  '2025-03': 0.56,
  '2025-04': 0.43,
  '2025-05': 0.26,
  '2025-06': 0.24,
  '2025-07': 0.26,
  '2025-08': -0.11,
  '2025-09': 0.48,
  '2025-10': 0.09,
  '2025-11': 0.18,
  '2025-12': 0.33,
  '2026-01': 0.3,
  '2026-02': 0.55,
  '2026-03': 0.4,
  '2026-04': 0.35,
  '2026-05': 0.28,
  '2026-06': 0.22,
  '2026-07': 0.2,
  '2026-08': 0.1,
  '2026-09': 0.38,
};

/** CDI anual (% a.a.) por mês: segue os movimentos da Selic. */
function cdiAnnual(month: string): number {
  if (month < '2024-02') return 11.65;
  if (month < '2024-04') return 11.15;
  if (month < '2024-06') return 10.4;
  if (month < '2024-10') return 10.65;
  if (month < '2024-12') return 11.15;
  if (month < '2025-02') return 12.15;
  if (month < '2025-04') return 13.15;
  if (month < '2025-06') return 14.15;
  if (month < '2026-03') return 14.9;
  if (month < '2026-06') return 14.4;
  if (month < '2026-09') return 13.9;
  return 13.4;
}

/** Último mês de IPCA oficial: o anterior a partir do dia 10, senão o retrasado. */
export function ipcaLastOfficial(today = mockToday()): string {
  const day = Number(today.slice(8, 10));
  return monthOf(addMonths(monthStart(monthOf(today)), day >= 10 ? -1 : -2));
}

function months(from: string, to: string): string[] {
  const out: string[] = [];
  for (let m = from; m <= to; m = monthOf(addMonths(monthStart(m), 1))) out.push(m);
  return out;
}

const seriesCache = new Map<string, { to: string; points: SimPoint[] }>();

function buildSeries(id: string, today: string): SimPoint[] {
  const lastMonth = monthOf(today);
  switch (id) {
    case 'ipca': {
      const official = ipcaLastOfficial(today);
      // Projeção Focus: o mês corrente e o seguinte.
      const until = monthOf(addMonths(monthStart(lastMonth), 1));
      return months('2023-01', until).map((m) => ({
        date: monthStart(m),
        value: IPCA[m] ?? 0.35,
        estimated: m > official,
      }));
    }
    case 'igpm':
      return months('2023-01', monthOf(addMonths(today, -1))).map((m, i) => ({
        date: monthStart(m),
        value: Number((0.35 + Math.sin(i / 2.3) * 0.6).toFixed(2)),
        estimated: false,
      }));
    case 'incc':
      return months('2023-01', monthOf(addMonths(today, -1))).map((m, i) => ({
        date: monthStart(m),
        value: Number((0.45 + Math.cos(i / 3.1) * 0.25).toFixed(2)),
        estimated: false,
      }));
    case 'tr':
      return months('2023-01', monthOf(addMonths(today, -1))).map((m, i) => ({
        date: monthStart(m),
        value: Number((0.07 + (i % 5) * 0.004).toFixed(4)),
        estimated: false,
      }));
    case 'cdi':
    case 'selic':
      return businessDays(SIM_START, addDays(today, -1)).map((d) => {
        const annual = cdiAnnual(monthOf(d)) + (id === 'selic' ? 0.1 : 0);
        return {
          date: d,
          value: Number((((1 + annual / 100) ** (1 / 252) - 1) * 100).toFixed(6)),
          estimated: false,
        };
      });
    case 'selic_meta':
      return businessDays(SIM_START, addDays(today, -1)).map((d) => ({
        date: d,
        value: cdiAnnual(monthOf(d)) + 0.1,
        estimated: false,
      }));
    case 'm2_br':
      return months('2023-01', monthOf(addMonths(today, -2))).map((m, i) => ({
        date: monthStart(m),
        value: Math.round(5_400_000_000 * 1.0062 ** i),
        estimated: false,
      }));
    case 'us_cpi':
      return months('2023-01', monthOf(addMonths(today, -1))).map((m, i) => ({
        date: monthStart(m),
        value: Number((299.2 * 1.0024 ** i).toFixed(3)),
        estimated: false,
      }));
    case 'us_m2':
      return months('2023-01', monthOf(addMonths(today, -2))).map((m, i) => ({
        date: monthStart(m),
        value: Number((20_800 * 1.0031 ** i).toFixed(1)),
        estimated: false,
      }));
    case 'fed_funds':
      return months('2023-01', monthOf(addMonths(today, -1))).map((m) => ({
        date: monthStart(m),
        value:
          m < '2024-09'
            ? 5.33
            : m < '2024-12'
              ? 4.83
              : m < '2025-09'
                ? 4.33
                : m < '2026-03'
                  ? 3.88
                  : 3.63,
        estimated: false,
      }));
    case 'ibov':
    case 'sp500': {
      const spec =
        id === 'ibov'
          ? { start: 132_500, anchor: 146_800, vol: 0.18 }
          : { start: 4_770, anchor: 6_690, vol: 0.15 };
      return [...walk(`idx:${id}`, spec, addDays(today, -1)).entries()].map(([d, v]) => ({
        date: d,
        value: Math.round(v * 100) / 100,
        estimated: false,
      }));
    }
    default:
      return [];
  }
}

export function seriesPoints(id: string): SimPoint[] {
  const today = mockToday();
  const hit = seriesCache.get(id);
  if (hit && hit.to === today) return hit.points;
  const points = buildSeries(id, today);
  seriesCache.set(id, { to: today, points });
  return points;
}

/** Taxas diárias do CDI (% a.d.) em (from, to]. */
export function cdiRates(from: string, to: string): SimPoint[] {
  return seriesPoints('cdi').filter((p) => p.date > from && p.date <= to);
}

/**
 * Fator de deflação (1 + inflação) entre duas datas, como `indices::deflator_factor`:
 * IPCA por variações mensais pró-rata (geométrico dentro do mês); CPI e M2 por níveis
 * mensais com interpolação geométrica. `null` sem dados que cubram o período.
 */
export function deflatorFactor(
  deflator: string,
  from: string,
  to: string,
): { factor: number; estimated: boolean } | null {
  if (deflator === 'none' || to <= from) return { factor: 1, estimated: false };
  if (deflator === 'country') {
    // "Inflação do país": no mock só Brasil (IPCA) e EUA (US CPI) têm série simulada.
    const effective = countryDeflator();
    return effective ? deflatorFactor(effective, from, to) : null;
  }
  if (deflator === 'ipca' || deflator === 'igpm') {
    const pts = new Map(seriesPoints(deflator).map((p) => [monthOf(p.date), p]));
    let factor = 1;
    let estimated = false;
    for (let m = monthOf(from); m <= monthOf(to); m = monthOf(addMonths(monthStart(m), 1))) {
      const p = pts.get(m);
      if (!p) return null;
      const start = m === monthOf(from) ? from : monthStart(m);
      const endExclusive = m === monthOf(to) ? to : addDays(monthEnd(m), 1);
      const dim = daysBetween(monthStart(m), addDays(monthEnd(m), 1));
      const frac = daysBetween(start, endExclusive) / dim;
      if (frac <= 0) continue;
      factor *= (1 + p.value / 100) ** frac;
      estimated ||= p.estimated;
    }
    return { factor, estimated };
  }
  const id = deflator === 'us_cpi' ? 'us_cpi' : deflator === 'm2_br' ? 'm2_br' : 'us_m2';
  const pts = seriesPoints(id);
  // Como `inflation::level_factor`: precisa de um nível publicado na data ou depois
  // dela (não extrapola além do último mês divulgado).
  const level = (d: string): number | null => {
    const after = pts.findIndex((p) => p.date >= d);
    if (after < 0) return null;
    const b = pts[after]!;
    if (b.date === d) return b.value;
    if (after === 0) return null;
    const a = pts[after - 1]!;
    const t = daysBetween(a.date, d) / daysBetween(a.date, b.date);
    return a.value * (b.value / a.value) ** t;
  };
  const l0 = level(from);
  const l1 = level(to);
  if (l0 == null || l1 == null) return null;
  return { factor: l1 / l0, estimated: false };
}
