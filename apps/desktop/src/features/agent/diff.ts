/**
 * Diff exato de uma proposta: cada `change_item` (antes/depois de uma linha do banco) vira
 * a lista de campos com o valor antigo e o novo. Nada é omitido nem arredondado: o usuário
 * aprova exatamente o que será gravado. Campos técnicos (`created_at`, `updated_at`) ficam
 * marcados para a interface poder recolhê-los.
 */
import type { ChangeItem } from '@/api/generated/ChangeItem';
import type { ChangeOp } from '@/api/generated/ChangeOp';

import { labels, tableLabel } from './format';

export type FieldChange = 'added' | 'removed' | 'changed' | 'same';

export interface DiffField {
  field: string;
  before: unknown;
  after: unknown;
  change: FieldChange;
  /** Campo de controle (datas de criação/alteração): recolhido por padrão. */
  technical: boolean;
}

export interface DiffRow {
  key: string;
  table: string;
  tableLabel: string;
  rowId: string;
  op: ChangeOp;
  opLabel: string;
  fields: DiffField[];
  /** Campos que mudam de fato (o que importa revisar). */
  changedCount: number;
}

const TECHNICAL = new Set(['created_at', 'updated_at']);

export const OP_LABEL: Record<ChangeOp, string> = labels({
  insert: 'agente-op-inclusao',
  update: 'agente-op-alteracao',
  delete: 'agente-op-exclusao',
});

/** Igualdade estrutural (valores vêm de JSON). */
export function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a == null || b == null) return a == null && b == null;
  if (typeof a === 'object' && typeof b === 'object')
    return JSON.stringify(a) === JSON.stringify(b);
  return false;
}

function fieldOrder(before: Record<string, unknown>, after: Record<string, unknown>): string[] {
  const keys = [...Object.keys(before)];
  for (const k of Object.keys(after)) if (!keys.includes(k)) keys.push(k);
  // `id` primeiro; técnicos por último; o resto na ordem das colunas.
  const rank = (k: string) => (k === 'id' ? 0 : TECHNICAL.has(k) ? 2 : 1);
  return keys
    .map((k, i) => ({ k, i }))
    .sort((a, b) => rank(a.k) - rank(b.k) || a.i - b.i)
    .map((e) => e.k);
}

export function diffItem(item: ChangeItem): DiffRow {
  const before = item.before ?? {};
  const after = item.after ?? {};
  const fields: DiffField[] = fieldOrder(before, after).map((field) => {
    const inBefore = item.before != null && field in before;
    const inAfter = item.after != null && field in after;
    const b = inBefore ? before[field] : undefined;
    const a = inAfter ? after[field] : undefined;
    let change: FieldChange;
    if (item.op === 'insert') change = 'added';
    else if (item.op === 'delete') change = 'removed';
    else if (!inBefore) change = 'added';
    else if (!inAfter) change = 'removed';
    else change = sameValue(b, a) ? 'same' : 'changed';
    return { field, before: b, after: a, change, technical: TECHNICAL.has(field) };
  });
  return {
    key: `${item.seq}:${item.table_name}:${item.row_id}`,
    table: item.table_name,
    tableLabel: tableLabel(item.table_name),
    rowId: item.row_id,
    op: item.op,
    opLabel: OP_LABEL[item.op],
    fields,
    changedCount: fields.filter((f) => f.change !== 'same' && !f.technical).length,
  };
}

export function diffItems(items: readonly ChangeItem[]): DiffRow[] {
  return [...items].sort((a, b) => a.seq - b.seq).map(diffItem);
}

/**
 * Campos a mostrar: numa alteração, só os que mudam (e o `id`); numa inclusão ou
 * exclusão, todos. `showAll` inclui os iguais e os técnicos.
 */
export function visibleFields(row: DiffRow, showAll = false): DiffField[] {
  if (showAll) return row.fields;
  return row.fields.filter(
    (f) => !f.technical && (row.op !== 'update' || f.change !== 'same' || f.field === 'id'),
  );
}

/** Valor bruto como texto, sem perder precisão: `null` → `∅`, textos entre aspas. */
export function formatDiffValue(value: unknown): string {
  if (value === undefined) return '—';
  if (value === null) return '∅';
  if (typeof value === 'string') return `"${value}"`;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  return JSON.stringify(value);
}

/** Campos de dinheiro (inteiros na menor unidade) ganham a leitura formatada ao lado. */
export function isMoneyField(field: string, value: unknown): value is number {
  return (
    typeof value === 'number' &&
    Number.isInteger(value) &&
    (field === 'amount' || field.endsWith('_amount') || field.endsWith('_cents'))
  );
}
