/**
 * Montador de filtros do screener ↔ `ScreenQuery` do núcleo (`research::screen`).
 *
 * O rascunho guarda o que a pessoa digitou (texto pt-BR, percentuais em %); a conversão
 * para o núcleo transforma percentuais em fração (6 → 0,06), lê números pt-BR
 * (`1.234,5`), monta `between` como `[mín, máx]` e listas de texto (`a, b`) como arrays.
 * A volta (tela salva → rascunho) faz o caminho inverso.
 */
import type { CmpOp } from '@/api/generated/CmpOp';
import type { Condition } from '@/api/generated/Condition';
import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { Logic } from '@/api/generated/Logic';
import type { ScreenQuery } from '@/api/generated/ScreenQuery';
import type { UniverseFilter } from '@/api/generated/UniverseFilter';
import { t } from '@/i18n';

import { decimalToInput, parseDecimalInput } from '../portfolio/decimal';
import { BOOL_FIELDS, TEXT_FIELDS } from './labels';

export type FieldKind = 'numeric' | 'text' | 'bool';

export interface DraftCondition {
  id: string;
  field: string;
  op: CmpOp;
  /** Valor como digitado (percentuais em %; listas separadas por vírgula). */
  value: string;
  /** Máximo do `between`. */
  value2: string;
}

export interface DraftGroup {
  id: string;
  logic: Logic;
  conditions: DraftCondition[];
}

export interface DraftQuery {
  logic: Logic;
  conditions: DraftCondition[];
  groups: DraftGroup[];
  universe: UniverseFilter;
}

let seq = 0;
export function draftId(prefix = 'c'): string {
  seq += 1;
  return `${prefix}${seq}`;
}

const TEXT_KEYS = new Set<string>(TEXT_FIELDS.map((f) => f.key));
const BOOL_KEYS = new Set<string>(BOOL_FIELDS.map((f) => f.key));

export function fieldKind(field: string): FieldKind {
  if (BOOL_KEYS.has(field)) return 'bool';
  if (TEXT_KEYS.has(field)) return 'text';
  return 'numeric';
}

const OPS: Record<FieldKind, readonly CmpOp[]> = {
  numeric: ['gt', 'gte', 'lt', 'lte', 'between', 'eq', 'ne', 'exists', 'missing'],
  text: ['eq', 'ne', 'contains', 'in', 'not_in', 'exists', 'missing'],
  bool: ['eq'],
};

export function opsFor(kind: FieldKind): readonly CmpOp[] {
  return OPS[kind];
}

/** Quantos valores o operador pede. */
export function arity(op: CmpOp): 'none' | 'one' | 'two' | 'list' {
  if (op === 'exists' || op === 'missing') return 'none';
  if (op === 'between') return 'two';
  if (op === 'in' || op === 'not_in') return 'list';
  return 'one';
}

export function newCondition(field = 'pl', op?: CmpOp): DraftCondition {
  const kind = fieldKind(field);
  return {
    id: draftId('c'),
    field,
    op: op ?? (kind === 'numeric' ? 'lt' : kind === 'bool' ? 'eq' : 'contains'),
    value: kind === 'bool' ? 'true' : '',
    value2: '',
  };
}

export function newGroup(): DraftGroup {
  return { id: draftId('g'), logic: 'or', conditions: [newCondition('dy', 'gte')] };
}

export function emptyDraft(): DraftQuery {
  return { logic: 'and', conditions: [], groups: [], universe: {} };
}

/** Troca o campo e mantém o operador quando ele ainda vale para o novo campo. */
export function changeField(c: DraftCondition, field: string): DraftCondition {
  const kind = fieldKind(field);
  const op = opsFor(kind).includes(c.op) ? c.op : newCondition(field).op;
  const sameKind = fieldKind(c.field) === kind;
  return {
    ...c,
    field,
    op,
    value: sameKind ? c.value : kind === 'bool' ? 'true' : '',
    value2: sameKind ? c.value2 : '',
  };
}

function unitOf(field: string, catalog: readonly IndicatorInfo[]): IndicatorInfo['unit'] | null {
  return catalog.find((i) => i.key === field)?.unit ?? null;
}

/** Número pt-BR digitado → número do núcleo (percentual vira fração). */
export function parseNumberInput(text: string, percent: boolean): number | null {
  const d = parseDecimalInput(text, { allowNegative: true });
  if (d == null) return null;
  const n = Number(d);
  if (!Number.isFinite(n)) return null;
  return percent ? Number((n / 100).toPrecision(12)) : n;
}

/**
 * Número do núcleo → texto de edição (fração vira %; sem separador de milhar; separador
 * decimal do idioma do app, o mesmo que `parseNumberInput` lê).
 */
export function numberToInput(n: number, percent: boolean): string {
  const v = percent ? Number((n * 100).toPrecision(12)) : n;
  return decimalToInput(String(v));
}

function splitList(text: string): string[] {
  return text
    .split(/[;,]/)
    .map((s) => s.trim())
    .filter(Boolean);
}

export function cleanUniverse(u: UniverseFilter): UniverseFilter | undefined {
  const out: UniverseFilter = {};
  if (u.classes?.length) out.classes = [...u.classes];
  const sectors = u.sectors?.map((s) => s.trim()).filter(Boolean);
  if (sectors?.length) out.sectors = sectors;
  const segments = u.segments?.map((s) => s.trim()).filter(Boolean);
  if (segments?.length) out.segments = segments;
  if (u.min_liquidity != null && Number.isFinite(u.min_liquidity) && u.min_liquidity > 0)
    out.min_liquidity = u.min_liquidity;
  if (u.only_mine) out.only_mine = true;
  return Object.keys(out).length ? out : undefined;
}

function toCondition(
  c: DraftCondition,
  catalog: readonly IndicatorInfo[],
): { condition: Condition } | { error: string } {
  const kind = fieldKind(c.field);
  if (!opsFor(kind).includes(c.op)) return { error: t('pesquisa-erro-operador') };
  const a = arity(c.op);
  if (a === 'none') return { condition: { field: c.field, op: c.op } };
  if (kind === 'bool') {
    return { condition: { field: c.field, op: 'eq', value: c.value !== 'false' } };
  }
  if (kind === 'text') {
    if (a === 'list') {
      const list = splitList(c.value);
      if (!list.length) return { error: t('pesquisa-erro-lista') };
      return { condition: { field: c.field, op: c.op, value: list } };
    }
    if (!c.value.trim()) return { error: t('pesquisa-erro-texto') };
    return { condition: { field: c.field, op: c.op, value: c.value.trim() } };
  }
  const percent = unitOf(c.field, catalog) === 'percent';
  const lo = parseNumberInput(c.value, percent);
  if (lo == null) return { error: t('pesquisa-erro-numero') };
  if (a === 'two') {
    const hi = parseNumberInput(c.value2, percent);
    if (hi == null) return { error: t('pesquisa-erro-maximo') };
    if (hi < lo) return { error: t('pesquisa-erro-maximo-menor') };
    return { condition: { field: c.field, op: 'between', value: [lo, hi] } };
  }
  return { condition: { field: c.field, op: c.op, value: lo } };
}

export interface BuildResult {
  /** `null` quando há erros. */
  query: ScreenQuery | null;
  /** Mensagem por id de condição ou de grupo. */
  errors: Record<string, string>;
}

export function toScreenQuery(d: DraftQuery, catalog: readonly IndicatorInfo[]): BuildResult {
  const errors: Record<string, string> = {};
  const convert = (list: DraftCondition[]) =>
    list.flatMap((c) => {
      const r = toCondition(c, catalog);
      if ('error' in r) {
        errors[c.id] = r.error;
        return [];
      }
      return [r.condition];
    });
  const conditions = convert(d.conditions);
  const groups = d.groups.map((g) => {
    if (g.conditions.length === 0) errors[g.id] = t('pesquisa-erro-grupo-vazio');
    return { logic: g.logic, conditions: convert(g.conditions) };
  });
  const universe = cleanUniverse(d.universe);
  const query: ScreenQuery = {
    logic: d.logic,
    conditions,
    groups,
    ...(universe ? { universe } : {}),
  };
  return { query: Object.keys(errors).length ? null : query, errors };
}

function fromCondition(c: Condition, catalog: readonly IndicatorInfo[]): DraftCondition {
  const kind = fieldKind(c.field);
  const base: DraftCondition = {
    id: draftId('c'),
    field: c.field,
    op: c.op,
    value: '',
    value2: '',
  };
  const v = c.value;
  if (kind === 'bool') return { ...base, op: 'eq', value: v === false ? 'false' : 'true' };
  if (arity(c.op) === 'none') return base;
  if (kind === 'text') {
    if (Array.isArray(v)) return { ...base, value: v.map(String).join(', ') };
    return { ...base, value: v == null ? '' : typeof v === 'string' ? v : JSON.stringify(v) };
  }
  const percent = unitOf(c.field, catalog) === 'percent';
  const num = (x: unknown) => (typeof x === 'number' ? numberToInput(x, percent) : '');
  if (c.op === 'between' && Array.isArray(v)) {
    return { ...base, value: num(v[0]), value2: num(v[1]) };
  }
  return { ...base, value: num(v) };
}

export function fromScreenQuery(q: ScreenQuery, catalog: readonly IndicatorInfo[]): DraftQuery {
  return {
    logic: q.logic,
    conditions: q.conditions.map((c) => fromCondition(c, catalog)),
    groups: q.groups.map((g) => ({
      id: draftId('g'),
      logic: g.logic,
      conditions: g.conditions.map((c) => fromCondition(c, catalog)),
    })),
    universe: { ...(q.universe ?? {}) },
  };
}

/** Indicadores usados nos filtros (viram colunas do resultado). */
export function filterFields(q: ScreenQuery): string[] {
  const out: string[] = [];
  for (const c of [...q.conditions, ...q.groups.flatMap((g) => g.conditions)]) {
    if (fieldKind(c.field) === 'numeric' && !out.includes(c.field)) out.push(c.field);
  }
  return out;
}
