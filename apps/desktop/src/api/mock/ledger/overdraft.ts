/**
 * Cheque especial no mock (espelho de `accounts::overdraft`): limite e condições por conta e
 * o uso/custo estimado a partir do saldo.
 */
import type { IndexReading } from '@/api/generated/IndexReading';
import type { LoanIndexer } from '@/api/generated/LoanIndexer';
import type { OverdraftStatus } from '@/api/generated/OverdraftStatus';
import type { OverdraftTerms } from '@/api/generated/OverdraftTerms';

import { mockFail, onMockReset } from '@/api/mockRegistry';

import { mockToday, seriesPoints } from '../marketSim';

const terms = new Map<string, OverdraftTerms>();
onMockReset(() => terms.clear());

export function overdraftOf(id: string): OverdraftTerms | null {
  return terms.get(id) ?? null;
}

function num(v: string | null | undefined): number | null {
  if (v == null || v.trim() === '') return null;
  const raw = v.replace('%', '').trim();
  const n = Number(raw.includes(',') ? raw.replace(/\./g, '').replace(',', '.') : raw);
  return Number.isFinite(n) ? n : null;
}

function clean(v: string | null | undefined): string | null {
  const n = num(v);
  return n == null ? null : String(n);
}

const INDEXERS: Record<string, LoanIndexer> = {
  tr: 'tr',
  ipca: 'ipca',
  igpm: 'igpm',
  inpc: 'inpc',
  incc: 'incc',
  cdi: 'cdi',
  di: 'cdi',
  selic: 'selic',
  poupanca: 'poupanca',
};

/** Como `debt::split_rate_index`: "4,75 + TR" → ["4,75", "tr"]. */
export function splitRateIndex(v: string | null | undefined): [string, LoanIndexer] | null {
  const parts = (v ?? '').split('+').map((x) => x.trim());
  if (parts.length !== 2) return null;
  const [a, b] = /^\d/.test(parts[0]!) ? [parts[0]!, parts[1]!] : [parts[1]!, parts[0]!];
  const key = b.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().replace(/[-. ]/g, '');
  const idx = INDEXERS[key];
  const num = /^[\d.,]+/.exec(a)?.[0];
  return idx && num ? [num, idx] : null;
}

export function setOverdraft(id: string, input: OverdraftTerms | null): void {
  if (!input) {
    terms.delete(id);
    return;
  }
  const t = { ...input };
  let indexer: LoanIndexer | null = t.indexer && t.indexer !== 'none' ? t.indexer : null;
  for (const key of ['annual_rate', 'monthly_rate'] as const) {
    const split = splitRateIndex(t[key]);
    if (split) {
      if (indexer && indexer !== split[1])
        mockFail('invalid', 'a taxa ao ano e a taxa ao mês trazem indexadores diferentes');
      indexer = split[1];
      t[key] = split[0];
    }
  }
  terms.set(id, {
    ...(indexer ? { indexer } : {}),
    limit: Math.max(0, Math.round(t.limit)),
    monthly_rate: clean(t.monthly_rate),
    discount_pct: clean(t.discount_pct),
    annual_rate: clean(t.annual_rate),
    iof_daily_rate: clean(t.iof_daily_rate),
    iof_additional_rate: clean(t.iof_additional_rate),
    cet_month: clean(t.cet_month),
    cet_year: clean(t.cet_year),
  });
}

const SERIES: Partial<Record<LoanIndexer, string>> = {
  tr: 'tr',
  ipca: 'ipca',
  igpm: 'igpm',
  cdi: 'cdi',
  selic: 'selic',
  poupanca: 'poupanca',
};

/** Último valor do índice no mock (% a.m.; CDI e Selic diários compostos em 21 dias). */
function indexReading(i: LoanIndexer | undefined): IndexReading | null {
  const series = i ? SERIES[i] : undefined;
  if (!i || !series) return null;
  const today = mockToday();
  const last = seriesPoints(series)
    .filter((p) => p.date <= today)
    .at(-1);
  if (!last) return null;
  const monthly =
    i === 'cdi' || i === 'selic' ? ((1 + last.value / 100) ** 21 - 1) * 100 : last.value;
  return {
    indexer: i,
    monthly_pct: String(Math.round(monthly * 1e4) / 1e4),
    date: last.date,
    estimated: last.estimated,
  };
}

export function overdraftStatus(t: OverdraftTerms, balance: number): OverdraftStatus {
  const used = Math.max(0, -balance);
  const index = t.indexer && t.indexer !== 'none' ? indexReading(t.indexer) : null;
  const spreadMonthly =
    num(t.monthly_rate) ??
    (num(t.annual_rate) == null ? null : ((1 + num(t.annual_rate)! / 100) ** (1 / 12) - 1) * 100);
  const rate =
    t.indexer && t.indexer !== 'none'
      ? index
        ? ((1 + (spreadMonthly ?? 0) / 100) * (1 + Number(index.monthly_pct) / 100) - 1) * 100
        : null
      : num(t.monthly_rate);
  const discount = num(t.discount_pct) ?? 0;
  const effective = rate == null ? null : rate * (1 - discount / 100);
  const daily = num(t.iof_daily_rate);
  const add = num(t.iof_additional_rate);
  const iof =
    daily == null && add == null
      ? null
      : Math.round((used * ((daily ?? 0) * 30 + (add ?? 0))) / 100);
  return {
    limit: t.limit,
    used,
    available: t.limit - used,
    spendable: balance + t.limit,
    effective_monthly_rate: effective == null ? null : String(Math.round(effective * 1e4) / 1e4),
    index,
    estimated_interest: effective == null ? null : Math.round((used * effective) / 100),
    uncertain: false,
    estimated_iof: iof,
  };
}
