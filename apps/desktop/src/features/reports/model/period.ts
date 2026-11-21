/**
 * Filtros da tela Relatórios (período, base, membro, conta e visualização), lidos e
 * gravados na URL (`#/relatorios?v=fluxo&p=mes`), para a barra ⌘K e o "voltar" funcionarem.
 */
import type { Basis } from '@/api/generated/Basis';
import { t } from '@/i18n';
import { formatDate, formatMonthLong } from '@/lib/format';
import {
  addDays,
  daysBetween,
  monthBounds,
  monthOf,
  monthsBetween,
  parseIso,
  shiftMonth,
} from '@/lib/months';

const VIEW_IDS = [
  'fluxo',
  'composicao',
  'ritmo',
  'tendencia',
  'orcamento',
  'comparacao',
  'inflacao',
  'titulares',
  'recorrentes',
  'cartoes',
  'patrimonio',
  'grupos',
] as const;

export type ViewId = (typeof VIEW_IDS)[number];

/** Visualizações na ordem das abas (rótulo e pergunta no idioma atual). */
export const VIEWS: readonly { id: ViewId; label: string; question: string }[] = VIEW_IDS.map(
  (id) => ({
    id,
    get label() {
      return t(`relatorios-vista-${id}`);
    },
    get question() {
      return t(`relatorios-vista-${id}-pergunta`);
    },
  }),
);

const PRESET_IDS = [
  'mes',
  'mes-anterior',
  'trimestre',
  'trimestre-anterior',
  '3m',
  '6m',
  'ano',
  '12m',
  'ano-anterior',
  'personalizado',
] as const;

export type PresetId = (typeof PRESET_IDS)[number];

export const PRESETS: readonly { id: PresetId; label: string }[] = PRESET_IDS.map((id) => ({
  id,
  get label() {
    return t(`relatorios-periodo-${id}`);
  },
}));

export interface Period {
  from: string;
  to: string;
}

export interface ReportFilters {
  view: ViewId;
  preset: PresetId;
  /** Usado só no período personalizado. */
  from: string;
  to: string;
  /** `null` = o padrão do cofre (ajuste `budget_basis`). */
  basis: Basis | null;
  member: string | null;
  account: string | null;
}

const isView = (v: string | null): v is ViewId => VIEWS.some((x) => x.id === v);
const isPreset = (v: string | null): v is PresetId => PRESETS.some((x) => x.id === v);
const ISO = /^\d{4}-\d{2}-\d{2}$/;

/** Período de um atalho, relativo a `today`. */
export function presetPeriod(preset: PresetId, today: string, custom?: Period): Period {
  const month = monthOf(today);
  const { y } = parseIso(today);
  switch (preset) {
    case 'mes':
      return monthBounds(month);
    case 'mes-anterior':
      return monthBounds(shiftMonth(month, -1));
    case 'trimestre':
    case 'trimestre-anterior': {
      const start = shiftMonth(quarterStart(month), preset === 'trimestre' ? 0 : -3);
      return { from: monthBounds(start).from, to: monthBounds(shiftMonth(start, 2)).to };
    }
    case '3m':
      return { from: monthBounds(shiftMonth(month, -2)).from, to: monthBounds(month).to };
    case '6m':
      return { from: monthBounds(shiftMonth(month, -5)).from, to: monthBounds(month).to };
    case 'ano':
      return { from: `${y}-01-01`, to: `${y}-12-31` };
    case '12m':
      return { from: monthBounds(shiftMonth(month, -11)).from, to: monthBounds(month).to };
    case 'ano-anterior':
      return { from: `${y - 1}-01-01`, to: `${y - 1}-12-31` };
    case 'personalizado':
      return custom && custom.from <= custom.to ? custom : monthBounds(month);
  }
}

/** Primeiro mês do trimestre de `month` (`2026-08` → `2026-07`). */
function quarterStart(month: string): string {
  const { m } = parseIso(`${month}-01`);
  return shiftMonth(month, -((m - 1) % 3));
}

export function parseFilters(search: string | URLSearchParams, today: string): ReportFilters {
  const p = typeof search === 'string' ? new URLSearchParams(search.replace(/^\?/, '')) : search;
  const view = p.get('v');
  const preset = p.get('p');
  const from = p.get('from');
  const to = p.get('to');
  const basis = p.get('basis');
  const fallback = monthBounds(monthOf(today));
  const custom = from && to && ISO.test(from) && ISO.test(to) && from <= to;
  return {
    view: isView(view) ? view : 'fluxo',
    preset: isPreset(preset) ? preset : custom ? 'personalizado' : 'mes',
    from: custom ? from : fallback.from,
    to: custom ? to : fallback.to,
    basis: basis === 'cash' || basis === 'accrual' ? basis : null,
    member: p.get('member') || null,
    account: p.get('account') || null,
  };
}

export function filtersSearch(f: ReportFilters): string {
  const p = new URLSearchParams();
  p.set('v', f.view);
  if (f.preset !== 'mes') p.set('p', f.preset);
  if (f.preset === 'personalizado') {
    p.set('from', f.from);
    p.set('to', f.to);
  }
  if (f.basis) p.set('basis', f.basis);
  if (f.member) p.set('member', f.member);
  if (f.account) p.set('account', f.account);
  return p.toString();
}

/** Período efetivo dos filtros. */
export function filtersPeriod(f: ReportFilters, today: string): Period {
  return presetPeriod(f.preset, today, { from: f.from, to: f.to });
}

/** Período anterior de mesma duração (meses inteiros quando o período é de meses inteiros). */
export function previousPeriod(p: Period): Period {
  const fromM = monthOf(p.from);
  const toM = monthOf(p.to);
  if (p.from === monthBounds(fromM).from && p.to === monthBounds(toM).to) {
    const n = monthsBetween(fromM, toM).length;
    return {
      from: monthBounds(shiftMonth(fromM, -n)).from,
      to: monthBounds(shiftMonth(toM, -n)).to,
    };
  }
  const days = daysBetween(p.from, p.to) + 1;
  return { from: addDays(p.from, -days), to: addDays(p.from, -1) };
}

/** Período seguinte de mesma duração (o inverso de `previousPeriod`). */
export function nextPeriod(p: Period): Period {
  const fromM = monthOf(p.from);
  const toM = monthOf(p.to);
  if (p.from === monthBounds(fromM).from && p.to === monthBounds(toM).to) {
    const n = monthsBetween(fromM, toM).length;
    return { from: monthBounds(shiftMonth(fromM, n)).from, to: monthBounds(shiftMonth(toM, n)).to };
  }
  const days = daysBetween(p.from, p.to) + 1;
  return { from: addDays(p.to, 1), to: addDays(p.to, days) };
}

/**
 * Filtros com o período deslocado para trás (−1) ou para a frente (+1): volta ao atalho
 * quando o novo período é um deles (este mês, mês passado…), senão vira personalizado.
 */
export function stepFilters(f: ReportFilters, today: string, dir: -1 | 1): ReportFilters {
  const cur = filtersPeriod(f, today);
  const next = dir < 0 ? previousPeriod(cur) : nextPeriod(cur);
  const preset = PRESETS.find((x) => {
    if (x.id === 'personalizado') return false;
    const pp = presetPeriod(x.id, today);
    return pp.from === next.from && pp.to === next.to;
  });
  return preset
    ? { ...f, preset: preset.id }
    : { ...f, preset: 'personalizado', from: next.from, to: next.to };
}

/** O mesmo período um ano antes. */
export function yearBeforePeriod(p: Period): Period {
  const fromM = monthOf(p.from);
  const toM = monthOf(p.to);
  if (p.from === monthBounds(fromM).from && p.to === monthBounds(toM).to) {
    return {
      from: monthBounds(shiftMonth(fromM, -12)).from,
      to: monthBounds(shiftMonth(toM, -12)).to,
    };
  }
  const shift = (iso: string) => `${parseIso(iso).y - 1}${iso.slice(4)}`;
  return { from: shift(p.from), to: shift(p.to) };
}

/** Rótulo curto do período: `outubro de 2026`, `jan–mar/2026`, `01/01/2026 a 14/10/2026`. */
export function periodLabel(p: Period): string {
  const fromM = monthOf(p.from);
  const toM = monthOf(p.to);
  const wholeMonths = p.from === monthBounds(fromM).from && p.to === monthBounds(toM).to;
  if (wholeMonths && fromM === toM) return formatMonthLong(fromM);
  if (wholeMonths) {
    return t('relatorios-periodo-intervalo', {
      de: formatMonthLong(fromM),
      ate: formatMonthLong(toM),
    });
  }
  return t('relatorios-periodo-intervalo', { de: formatDate(p.from), ate: formatDate(p.to) });
}

/** Meses (12 a 24) que terminam no último mês do período, para a Tendência. */
export function trendWindow(p: Period): { months: number; endMonth: string } {
  const n = monthsBetween(monthOf(p.from), monthOf(p.to)).length;
  return { months: Math.min(24, Math.max(12, n)), endMonth: monthOf(p.to) };
}

/** Um ano de calendário que termina no fim do período (sem passar de hoje). */
export function calendarWindow(p: Period, today?: string): Period {
  const end = monthOf(p.to);
  const to = monthBounds(end).to;
  return {
    from: monthBounds(shiftMonth(end, -11)).from,
    to: today && today < to && today >= monthBounds(end).from ? today : to,
  };
}

/** O período é um mês inteiro? Devolve o mês (`YYYY-MM`) ou `null`. */
export function singleMonth(p: Period): string | null {
  const m = monthOf(p.from);
  const b = monthBounds(m);
  return p.from === b.from && p.to === b.to ? m : null;
}
