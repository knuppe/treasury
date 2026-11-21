/**
 * Sankey do núcleo (`reports.sankey`) → entrada do gráfico: cores presas à categoria,
 * categorias sem cor reunidas em "Outras", a comparação com o período anterior em cada nó
 * e, para cada nó, o drill-down correspondente.
 */
import type { SankeyData } from '@/api/generated/SankeyData';
import type { SankeyDelta } from '@/api/generated/SankeyDelta';
import type { SankeyNodeKind } from '@/api/generated/SankeyNodeKind';
import type { TxnKind } from '@/api/generated/TxnKind';
import type { ColorRef } from '@/charts/colorMath';
import type { NodeCompare, SankeyInput, SankeyNodeInput } from '@/charts/sankey';
import { t } from '@/i18n';

import { OTHERS_KEY, othersLabel, type EntityColors } from './colors';

export interface NodeDrill {
  kind: TxnKind;
  /** Categorias (com subcategorias); `[null]` = sem categoria; vazio = todas do tipo. */
  categories: (string | null)[];
}

export interface FlowModel {
  input: SankeyInput;
  /** Nó → drill-down (nós sem drill: orçamento, sobra, déficit, aportes e resgates). */
  drill: Map<string, NodeDrill>;
  /** Categorias reunidas em "Outras". */
  folded: string[];
}

/** Coluna de cada tipo de nó: fontes, orçamento, destinos, subcategorias. */
const DEPTH: Record<SankeyNodeKind, number> = {
  income: 0,
  redemption: 0,
  deficit: 0,
  hub: 1,
  expense: 2,
  debt: 2,
  investment: 2,
  savings: 2,
  subcategory: 3,
};

/** Cor fixa dos nós que não são categoria (a mesma em todo período). */
const FIXED: Partial<Record<SankeyNodeKind, ColorRef>> = {
  income: '@positive',
  deficit: '@negative',
  savings: '@accent',
  investment: '@text',
  redemption: '@text',
  debt: '@warning',
  hub: '@gray',
};

export function flowModel(data: SankeyData, colors: EntityColors): FlowModel {
  const drill = new Map<string, NodeDrill>();
  const nodes: SankeyNodeInput[] = [];
  const folded: string[] = [];
  const foldedNodes = new Set<string>();
  for (const n of data.nodes) {
    if (n.kind === 'expense' && n.category_id && !colors.get(n.category_id)) {
      folded.push(n.category_id);
      foldedNodes.add(n.id);
    }
  }
  // Subcategorias das categorias reunidas também saem do diagrama.
  const foldedSubs = new Set(
    data.links.filter((l) => foldedNodes.has(l.source)).map((l) => l.target),
  );
  const compare = compareMap(data);
  const othersId = `expense:${OTHERS_KEY}`;
  const othersAdded = foldedNodes.size > 0;
  for (const n of data.nodes) {
    if (foldedNodes.has(n.id) || foldedSubs.has(n.id)) continue;
    const parent = data.links.find((l) => l.target === n.id)?.source;
    const color =
      FIXED[n.kind] ??
      (parent === 'debt'
        ? FIXED.debt!
        : n.category_id
          ? (colors.get(rootFor(n.id, n.category_id, data)) ?? '@gray')
          : '@dim');
    nodes.push({
      id: n.id,
      name: n.name,
      color,
      depth: DEPTH[n.kind],
      compare: compare.get(n.id),
    });
    if (n.kind === 'income') drill.set(n.id, { kind: 'income', categories: [n.category_id] });
    if (n.kind === 'expense' || n.kind === 'subcategory')
      drill.set(n.id, { kind: 'expense', categories: [n.category_id] });
    const debt = n.kind === 'debt' ? debtCategories(data) : [];
    if (debt.length) drill.set(n.id, { kind: 'expense', categories: debt });
  }
  if (othersAdded) {
    // "Outras" fecha a coluna das categorias (antes de dívidas, aportes e sobra).
    const lastExpense = nodes.reduce((last, n, i) => (n.id.startsWith('expense:') ? i : last), -1);
    nodes.splice(lastExpense + 1, 0, {
      id: othersId,
      name: othersLabel(),
      color: '@gray',
      depth: DEPTH.expense,
      compare: sumCompare(data.comparison.filter((r) => foldedNodes.has(r.id))),
    });
    drill.set(othersId, { kind: 'expense', categories: [...folded] });
  }
  // Subcategorias das categorias reunidas saem; o fluxo do orçamento para elas vai para "Outras".
  const links: SankeyInput['links'][number][] = [];
  let othersValue = 0;
  for (const l of data.links) {
    if (foldedNodes.has(l.target)) {
      othersValue += l.value;
      continue;
    }
    if (foldedNodes.has(l.source)) continue;
    links.push({ ...l });
  }
  if (othersAdded) links.push({ source: 'hub', target: othersId, value: othersValue });
  return {
    input: {
      nodes,
      links,
      // % da receita; sem receita, do que entrou no orçamento.
      total:
        data.total_income > 0 ? data.total_income : Math.max(data.total_income, data.total_expense),
      shareLabel: data.total_income > 0 ? t('relatorios-da-receita') : t('graficos-do-total'),
      currency: data.currency,
    },
    drill,
    folded,
  };
}

function compareMap(data: SankeyData): Map<string, NodeCompare> {
  return new Map(
    data.comparison.map((r) => [
      r.id,
      { previous: r.previous, delta: r.delta, deltaPct: r.delta_pct },
    ]),
  );
}

function sumCompare(rows: readonly SankeyDelta[]): NodeCompare | undefined {
  if (!rows.length) return undefined;
  const previous = rows.reduce((s, r) => s + r.previous, 0);
  const delta = rows.reduce((s, r) => s + r.delta, 0);
  return { previous, delta, deltaPct: previous > 0 ? delta / previous : null };
}

/** Categorias dos contratos (para abrir os lançamentos do nó de dívidas). */
function debtCategories(data: SankeyData): string[] {
  const debt = data.nodes.find((n) => n.id === 'debt');
  const children = data.links
    .filter((l) => l.source === 'debt')
    .map((l) => data.nodes.find((n) => n.id === l.target)?.category_id ?? null)
    .filter((c): c is string => c !== null);
  if (children.length) return children;
  return debt?.category_id ? [debt.category_id] : [];
}

/** Categoria de primeiro nível de um nó de despesa/subcategoria (pelo link de origem). */
function rootFor(nodeId: string, categoryId: string, data: SankeyData): string {
  if (nodeId.startsWith('expense:')) return categoryId;
  const parent = data.links.find((l) => l.target === nodeId)?.source;
  const parentNode = data.nodes.find((n) => n.id === parent);
  return parentNode?.category_id ?? categoryId;
}

/** Grupo da linha no comparativo: o que entra, o que sai e o resultado. */
export type FlowGroup = 'in' | 'out' | 'result';

export interface FlowCompareRow {
  id: string;
  name: string;
  kind: SankeyNodeKind;
  group: FlowGroup;
  /** 0 = ligado ao orçamento; 1 = subcategoria ou contrato. */
  level: 0 | 1;
  categoryId: string | null;
  parentId: string | null;
  current: number;
  previous: number;
  delta: number;
  deltaPct: number | null;
  share: number | null;
  /** Subir é bom (receitas, aportes, sobra) ou ruim (despesas, dívidas, déficit)? */
  goodWhenUp: boolean;
}

const groupOf = (k: SankeyNodeKind): FlowGroup =>
  k === 'income' || k === 'redemption'
    ? 'in'
    : k === 'savings' || k === 'deficit'
      ? 'result'
      : 'out';

const GOOD_WHEN_UP: ReadonlySet<SankeyNodeKind> = new Set([
  'income',
  'redemption',
  'investment',
  'savings',
]);

/**
 * Linhas do comparativo na ordem de leitura: entradas, saídas (cada categoria seguida das
 * subcategorias) e o resultado. Inclui o que só existiu no período anterior.
 */
export function comparisonRows(data: SankeyData): FlowCompareRow[] {
  const toRow = (r: SankeyDelta): FlowCompareRow => ({
    id: r.id,
    name: r.name,
    kind: r.kind,
    group: groupOf(r.kind),
    level: r.parent_id ? 1 : 0,
    categoryId: r.category_id,
    parentId: r.parent_id,
    current: r.current,
    previous: r.previous,
    delta: r.delta,
    deltaPct: r.delta_pct,
    share: r.share_of_income,
    goodWhenUp: GOOD_WHEN_UP.has(r.kind),
  });
  const children = new Map<string, FlowCompareRow[]>();
  const roots: FlowCompareRow[] = [];
  for (const r of data.comparison.map(toRow)) {
    if (r.parentId) children.set(r.parentId, [...(children.get(r.parentId) ?? []), r]);
    else roots.push(r);
  }
  const out: FlowCompareRow[] = [];
  for (const g of ['in', 'out', 'result'] as const) {
    for (const root of roots.filter((r) => r.group === g)) {
      out.push(root);
      // Os filhos herdam o sentido do pai (subcategoria de despesa, contrato de dívida).
      for (const c of children.get(root.id) ?? [])
        out.push({ ...c, group: g, goodWhenUp: root.goodWhenUp });
    }
  }
  return out;
}
