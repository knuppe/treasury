/**
 * Estado dos filtros da tela 03 ↔ URL. Os parâmetros do contrato de drill-down
 * (`lib/txnLink.ts`) vêm de `parseTxnLink`/`txnSearch`; esta tela acrescenta
 * `min`/`max` (valor, em centavos), `sort` e `page`.
 */
import type { TxnFilter } from '@/api/generated/TxnFilter';
import type { TxnSort } from '@/api/generated/TxnSort';
import { t } from '@/i18n';
import { formatDate, formatMonthShort, todayIso } from '@/lib/format';
import { parseTxnLink, txnSearch } from '@/lib/txnLink';

export const PAGE_SIZE = 100;

export interface TxnScreenState {
  filter: TxnFilter;
  page: number;
}

const SORTS: TxnSort[] = [
  'date',
  'amount',
  'description',
  'account',
  'category',
  'member',
  'status',
].flatMap((k) => [`${k}_desc`, `${k}_asc`]) as TxnSort[];

/** Sem nenhum parâmetro (entrada pela barra lateral), a tela abre no mês atual. */
const ALL_TIME = 'tudo';

function isBare(params: URLSearchParams): boolean {
  return [...params.keys()].every((k) => k === 'aba');
}

export function readState(params: URLSearchParams): TxnScreenState {
  const filter = parseTxnLink(params);
  if (isBare(params)) {
    const r = periodRange('this_month');
    filter.date_from = r.from;
    filter.date_to = r.to;
  }
  const min = Number(params.get('min'));
  const max = Number(params.get('max'));
  if (params.get('min') && Number.isInteger(min)) filter.amount_min = min;
  if (params.get('max') && Number.isInteger(max)) filter.amount_max = max;
  const sort = params.get('sort') as TxnSort | null;
  if (sort && SORTS.includes(sort) && sort !== 'date_desc') filter.sort = sort;
  const page = Math.max(1, Number(params.get('page')) || 1);
  return { filter, page };
}

/** Parâmetros da URL para um estado (mantém `aba` e outros parâmetros de fora). */
export function writeState(state: TxnScreenState, keep?: URLSearchParams): URLSearchParams {
  const p = new URLSearchParams(txnSearch(state.filter));
  if (state.filter.amount_min != null) p.set('min', String(state.filter.amount_min));
  if (state.filter.amount_max != null) p.set('max', String(state.filter.amount_max));
  if (state.filter.sort && state.filter.sort !== 'date_desc') p.set('sort', state.filter.sort);
  if (state.page > 1) p.set('page', String(state.page));
  // "Todo o período" sem outros filtros precisa de uma marca (vazio = mês atual).
  if (isBare(p)) p.set('periodo', ALL_TIME);
  const aba = keep?.get('aba');
  if (aba) p.set('aba', aba);
  return p;
}

/** Filtro enviado ao núcleo (com paginação). */
export function queryFilter(state: TxnScreenState): TxnFilter {
  return { ...state.filter, limit: PAGE_SIZE, offset: (state.page - 1) * PAGE_SIZE };
}

export function hasActiveFilters(f: TxnFilter): boolean {
  return Boolean(
    f.date_from ||
    f.date_to ||
    f.account_ids?.length ||
    f.category_ids?.length ||
    f.member_ids?.length ||
    f.kinds?.length ||
    f.statuses?.length ||
    f.tag_ids?.length ||
    f.project_ids?.length ||
    f.text ||
    f.amount_min != null ||
    f.amount_max != null ||
    f.uncategorized ||
    f.statement_id ||
    f.import_id,
  );
}

// ── Períodos ────────────────────────────────────────────────────

export type PeriodId =
  'all' | 'this_month' | 'last_month' | 'last_3' | 'last_12' | 'this_year' | 'custom';

const pad = (n: number) => String(n).padStart(2, '0');

function monthStart(y: number, m: number): string {
  const d = new Date(Date.UTC(y, m, 1));
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-01`;
}

function monthEnd(y: number, m: number): string {
  const d = new Date(Date.UTC(y, m + 1, 0));
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}

export function periodRange(id: PeriodId, today = todayIso()): { from?: string; to?: string } {
  const y = Number(today.slice(0, 4));
  const m = Number(today.slice(5, 7)) - 1;
  switch (id) {
    case 'this_month':
      return { from: monthStart(y, m), to: monthEnd(y, m) };
    case 'last_month':
      return { from: monthStart(y, m - 1), to: monthEnd(y, m - 1) };
    case 'last_3':
      return { from: monthStart(y, m - 2), to: monthEnd(y, m) };
    case 'last_12':
      return { from: monthStart(y, m - 11), to: monthEnd(y, m) };
    case 'this_year':
      return { from: `${y}-01-01`, to: `${y}-12-31` };
    default:
      return {};
  }
}

const PERIOD_KEYS = {
  all: 'lancamentos-periodo-tudo',
  this_month: 'lancamentos-periodo-este-mes',
  last_month: 'lancamentos-periodo-mes-passado',
  last_3: 'lancamentos-periodo-3-meses',
  last_12: 'lancamentos-periodo-12-meses',
  this_year: 'lancamentos-periodo-este-ano',
  custom: 'lancamentos-periodo-personalizado',
} as const satisfies Record<PeriodId, string>;

/** Atalhos de período, na ordem do painel (o rótulo sai do catálogo a cada leitura). */
export const PERIODS: { id: PeriodId; readonly label: string }[] = (
  Object.keys(PERIOD_KEYS) as PeriodId[]
).map((id) => ({
  id,
  get label() {
    return t(PERIOD_KEYS[id]);
  },
}));

export function detectPeriod(f: TxnFilter, today = todayIso()): PeriodId {
  if (!f.date_from && !f.date_to) return 'all';
  for (const p of PERIODS) {
    if (p.id === 'all' || p.id === 'custom') continue;
    const r = periodRange(p.id, today);
    if (r.from === f.date_from && r.to === f.date_to) return p.id;
  }
  return 'custom';
}

/** "out/26" para um mês inteiro, "01/10/2026 – 15/10/2026" para o resto (no idioma do app). */
export function periodLabel(f: TxnFilter): string {
  const id = detectPeriod(f);
  if (id !== 'custom') return PERIODS.find((p) => p.id === id)!.label;
  const from = f.date_from;
  const to = f.date_to;
  if (from && to && from.endsWith('-01') && to.slice(0, 7) === from.slice(0, 7)) {
    const y = Number(from.slice(0, 4));
    const m = Number(from.slice(5, 7)) - 1;
    if (monthEnd(y, m) === to) return formatMonthShort(from);
  }
  const fmt = (d?: string) => {
    if (!d) return '…';
    try {
      return formatDate(d);
    } catch {
      return d;
    }
  };
  return `${fmt(from)} – ${fmt(to)}`;
}
