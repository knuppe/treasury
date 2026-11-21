/**
 * Projetos do mock (`project`, V105; `project_item`, V106) no banco do livro-caixa:
 * cadastro, lançamentos ligados (`txn.project_id`), itens lançados à mão, gasto e receitas,
 * como `crates/treasury-core/src/projects`.
 */
import type { ProjectCategorySpend } from '@/api/generated/ProjectCategorySpend';
import type { ProjectEntry } from '@/api/generated/ProjectEntry';
import type { ProjectFlow } from '@/api/generated/ProjectFlow';
import type { ProjectInput } from '@/api/generated/ProjectInput';
import type { ProjectItemInput } from '@/api/generated/ProjectItemInput';
import type { ProjectItemPatch } from '@/api/generated/ProjectItemPatch';
import type { ProjectPatch } from '@/api/generated/ProjectPatch';
import type { ProjectStatus } from '@/api/generated/ProjectStatus';
import type { ProjectView } from '@/api/generated/ProjectView';

import { nowUtc, parseYmd } from './dates';
import { baseCurrency, categoryPaths } from './domain';
import { invalid, notFound } from './errors';
import { type ProjectItemRow, type ProjectRow, del, find, newId, patch, put, rows } from './store';

const STATUSES: readonly ProjectStatus[] = ['planned', 'active', 'done'];

const clean = (v: unknown): string | null => (typeof v === 'string' && v.trim() ? v.trim() : null);

export function projectRow(id: string): ProjectRow {
  return find('project', id) ?? notFound(`projeto ${id}`);
}

interface Flow {
  date: string;
  amount: number;
  category_id: string | null;
  income: boolean;
  fromTxn: boolean;
}

/** Lançamentos ligados e itens do projeto, cada um com o valor (com sinal) e se é receita. */
function flows(id: string): Flow[] {
  return [
    ...rows('txn')
      .filter((t) => t.project_id === id)
      .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id))
      .map((t) => ({
        date: t.date,
        amount: t.amount,
        category_id: t.category_id,
        income: t.kind === 'income',
        fromTxn: true,
      })),
    ...rows('project_item')
      .filter((i) => i.project_id === id)
      .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id))
      .map((i) => ({
        date: i.date,
        amount: i.amount,
        category_id: i.category_id,
        income: i.amount > 0,
        fromTxn: false,
      })),
  ].sort((a, b) => a.date.localeCompare(b.date));
}

/** Despesas do projeto (data, valor gasto), em ordem de data: o valor das benfeitorias. */
export function projectOutflows(id: string): { date: string; amount: number }[] {
  return flows(id)
    .filter((f) => !f.income)
    .map((f) => ({ date: f.date, amount: -f.amount }));
}

/** Gasto do projeto: despesas menos estornos. */
export function projectSpent(id: string): number {
  return projectOutflows(id).reduce((s, f) => s + f.amount, 0);
}

export interface PhysicalLookup {
  name: (assetId: string) => string | null;
  improvementIds: (projectId: string) => string[];
  exists: (assetId: string) => boolean;
  /** Tira o projeto das benfeitorias (projeto apagado). */
  unlink: (projectId: string) => void;
}

let physical: PhysicalLookup = {
  name: () => null,
  improvementIds: () => [],
  exists: () => false,
  unlink: () => undefined,
};

/** O mock de bens informa nomes e benfeitorias (evita import circular). */
export function setPhysicalLookup(l: PhysicalLookup): void {
  physical = l;
}

function byCategory(list: readonly Flow[], income: boolean): ProjectCategorySpend[] {
  const paths = categoryPaths();
  const sums = new Map<string | null, number>();
  for (const f of list)
    if (f.income === income)
      sums.set(f.category_id, (sums.get(f.category_id) ?? 0) + (income ? f.amount : -f.amount));
  return [...sums.entries()]
    .filter(([, v]) => v !== 0)
    .sort((a, b) => b[1] - a[1])
    .map(([category_id, spent]) => ({
      category_id,
      category_path: category_id ? (paths.get(category_id) ?? null) : null,
      spent,
    }));
}

export function projectView(p: ProjectRow): ProjectView {
  const list = flows(p.id);
  const spent = list.filter((f) => !f.income).reduce((s, f) => s - f.amount, 0);
  const income = list.filter((f) => f.income).reduce((s, f) => s + f.amount, 0);
  const dates = list.map((f) => f.date).sort();
  return {
    id: p.id,
    name: p.name,
    physical_asset_id: p.physical_asset_id,
    physical_asset_name: p.physical_asset_id ? physical.name(p.physical_asset_id) : null,
    status: p.status,
    budget: p.budget,
    start_date: p.start_date,
    end_date: p.end_date,
    notes: p.notes,
    spent,
    income,
    net: income - spent,
    txn_count: list.filter((f) => f.fromTxn).length,
    item_count: list.filter((f) => !f.fromTxn).length,
    first_date: dates[0] ?? null,
    last_date: dates[dates.length - 1] ?? null,
    remaining: p.budget != null ? p.budget - spent : null,
    by_category: byCategory(list, false),
    income_by_category: byCategory(list, true),
    improvement_ids: physical.improvementIds(p.id),
    created_at: p.created_at,
    updated_at: p.updated_at,
  };
}

export function listProjects(includeDone = true, assetId?: string): ProjectView[] {
  const key = (p: ProjectRow) => p.start_date ?? p.created_at.slice(0, 10);
  return rows('project')
    .filter((p) => includeDone || p.status !== 'done')
    .filter((p) => !assetId || p.physical_asset_id === assetId)
    .sort(
      (a, b) =>
        Number(a.status === 'done') - Number(b.status === 'done') ||
        key(b).localeCompare(key(a)) ||
        a.name.localeCompare(b.name, 'pt-BR'),
    )
    .map(projectView);
}

function validate(p: ProjectRow): void {
  if (!p.name.trim()) invalid('dê um nome ao projeto');
  if (p.budget != null && p.budget < 0) invalid('o orçamento do projeto não pode ser negativo');
  if (!STATUSES.includes(p.status))
    invalid(`argumentos inválidos: unknown variant \`${String(p.status)}\``);
  for (const d of [p.start_date, p.end_date]) if (d) parseYmd(d);
  if (p.start_date && p.end_date && p.end_date < p.start_date)
    invalid('o fim do projeto é antes do início');
  if (p.physical_asset_id && !physical.exists(p.physical_asset_id))
    notFound(`bem ${p.physical_asset_id}`);
}

export function createProject(i: ProjectInput): string {
  const now = nowUtc();
  const p: ProjectRow = {
    id: newId(),
    name: (i.name ?? '').trim(),
    physical_asset_id: clean(i.physical_asset_id),
    status: i.status ?? 'active',
    budget: typeof i.budget === 'number' ? i.budget : null,
    start_date: clean(i.start_date),
    end_date: clean(i.end_date),
    notes: clean(i.notes),
    created_at: now,
    updated_at: now,
  };
  validate(p);
  put('project', p);
  return p.id;
}

export function updateProject(a: ProjectPatch): void {
  const cur = projectRow(a.id);
  const next: ProjectRow = { ...cur };
  if (typeof a.name === 'string') next.name = a.name.trim();
  if ('physical_asset_id' in a) next.physical_asset_id = clean(a.physical_asset_id);
  if (a.status) next.status = a.status;
  if ('budget' in a) next.budget = typeof a.budget === 'number' ? a.budget : null;
  if ('start_date' in a) next.start_date = clean(a.start_date);
  if ('end_date' in a) next.end_date = clean(a.end_date);
  if ('notes' in a) next.notes = clean(a.notes);
  validate(next);
  if (JSON.stringify(next) !== JSON.stringify(cur))
    put('project', { ...next, updated_at: nowUtc() });
}

/** Apaga o projeto e os itens dele; lançamentos e benfeitorias ficam sem ele. */
export function deleteProject(id: string): void {
  projectRow(id);
  for (const i of rows('project_item')) if (i.project_id === id) del('project_item', i.id);
  for (const t of rows('txn'))
    if (t.project_id === id) patch('txn', t.id, { project_id: null, updated_at: nowUtc() });
  del('project', id);
  physical.unlink(id);
}

/** Liga (ou desliga, com `null`) lançamentos ao projeto; devolve quantos mudaram. */
export function linkProject(ids: readonly string[], projectId: string | null): number {
  if (projectId) projectRow(projectId);
  let n = 0;
  for (const id of new Set(ids)) {
    const t = find('txn', id) ?? notFound(`lançamento ${id}`);
    if ((t.project_id ?? null) !== projectId) {
      patch('txn', id, { project_id: projectId, updated_at: nowUtc() });
      n += 1;
    }
  }
  return n;
}

// ── Itens lançados à mão e a lista de entradas ──────────────────

export function itemRow(id: string): ProjectItemRow {
  return find('project_item', id) ?? notFound(`item de projeto ${id}`);
}

/** Lançamentos e itens do projeto, em ordem de data (lançamentos antes dos itens no dia). */
export function projectEntries(projectId: string): ProjectEntry[] {
  projectRow(projectId);
  const paths = categoryPaths();
  const path = (c: string | null) => (c ? (paths.get(c) ?? null) : null);
  const txns: ProjectEntry[] = rows('txn')
    .filter((t) => t.project_id === projectId)
    .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id))
    .map((t) => ({
      source: 'txn',
      id: t.id,
      project_id: projectId,
      date: t.date,
      amount: t.amount,
      flow: t.kind === 'income' ? 'income' : 'expense',
      description: t.description,
      category_id: t.category_id,
      category_path: path(t.category_id),
      account_id: t.account_id,
      account_name: find('account', t.account_id)?.name ?? null,
      currency: t.currency,
      notes: t.notes,
      planned: t.status === 'planned',
    }));
  const currency = baseCurrency();
  const items: ProjectEntry[] = rows('project_item')
    .filter((i) => i.project_id === projectId)
    .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id))
    .map((i) => ({
      source: 'item',
      id: i.id,
      project_id: projectId,
      date: i.date,
      amount: i.amount,
      flow: i.amount > 0 ? 'income' : 'expense',
      description: i.description,
      category_id: i.category_id,
      category_path: path(i.category_id),
      account_id: null,
      account_name: null,
      currency,
      notes: i.notes,
      planned: false,
    }));
  return [...txns, ...items].sort((a, b) => a.date.localeCompare(b.date));
}

export function itemEntry(id: string): ProjectEntry {
  const i = itemRow(id);
  return projectEntries(i.project_id).find((e) => e.source === 'item' && e.id === id)!;
}

const signed = (flow: ProjectFlow, amount: number) => (flow === 'income' ? amount : -amount);

function positive(amount: unknown): number {
  if (typeof amount !== 'number' || !(amount > 0))
    invalid('o valor do item tem de ser maior que zero (receita ou despesa vem do tipo)');
  return amount;
}

function validateItem(i: ProjectItemRow): void {
  projectRow(i.project_id);
  if (!i.description.trim()) invalid('descreva o item do projeto');
  parseYmd(i.date);
  if (i.category_id && !find('category', i.category_id)) notFound(`categoria ${i.category_id}`);
}

export function addItem(a: ProjectItemInput): string {
  const now = nowUtc();
  if (a.flow !== 'income' && a.flow !== 'expense')
    invalid(`argumentos inválidos: unknown variant \`${String(a.flow)}\``);
  const i: ProjectItemRow = {
    id: newId(),
    project_id: a.project_id,
    date: (a.date ?? '').trim(),
    amount: signed(a.flow, positive(a.amount)),
    description: (a.description ?? '').trim(),
    category_id: clean(a.category_id),
    notes: clean(a.notes),
    created_at: now,
    updated_at: now,
  };
  validateItem(i);
  put('project_item', i);
  return i.id;
}

export function updateItem(a: ProjectItemPatch): void {
  const cur = itemRow(a.id);
  const next: ProjectItemRow = { ...cur };
  if (a.project_id) next.project_id = a.project_id;
  if (typeof a.date === 'string') next.date = a.date.trim();
  const flow: ProjectFlow = a.flow ?? (cur.amount > 0 ? 'income' : 'expense');
  next.amount = signed(flow, a.amount !== undefined ? positive(a.amount) : Math.abs(cur.amount));
  if (typeof a.description === 'string') next.description = a.description.trim();
  if ('category_id' in a) next.category_id = clean(a.category_id);
  if ('notes' in a) next.notes = clean(a.notes);
  validateItem(next);
  if (JSON.stringify(next) !== JSON.stringify(cur))
    put('project_item', { ...next, updated_at: nowUtc() });
}

/** Tira do projeto: lançamentos ficam sem projeto, itens são apagados; devolve quantos. */
export function removeEntries(txnIds: readonly string[], itemIds: readonly string[]): number {
  let n = linkProject(txnIds, null);
  for (const id of new Set(itemIds)) {
    itemRow(id);
    del('project_item', id);
    n += 1;
  }
  return n;
}
