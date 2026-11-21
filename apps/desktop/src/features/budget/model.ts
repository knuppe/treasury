/**
 * Orçamento (funções puras): árvore pai › filhos a partir das linhas do núcleo, categorias
 * com gasto e sem orçamento, resumo do mês e diferença contra a sugestão pela média.
 */
import type { BudgetLineInput } from '@/api/generated/BudgetLineInput';
import type { BudgetLineStatus } from '@/api/generated/BudgetLineStatus';
import type { BudgetStatus } from '@/api/generated/BudgetStatus';
import type { Category } from '@/api/generated/Category';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import { bulletStatus, type BulletStatus } from '@/charts/bullet';

export interface BudgetRow {
  line: BudgetLineStatus;
  depth: number;
  children: BudgetRow[];
  status: BulletStatus;
  /** O previsto vem da soma das subcategorias (o pai não tem linha própria). */
  plannedFromChildren: boolean;
}

/** Linha vazia para uma categoria ainda fora do orçamento (adicionada pelo usuário). */
export function emptyLine(c: Category): BudgetLineStatus {
  return {
    category_id: c.id,
    category_name: c.name,
    category_path: c.path,
    parent_id: c.parent_id,
    kind: c.kind,
    planned: 0,
    actual: 0,
    remaining: 0,
    ratio: null,
    committed: 0,
    has_own_line: false,
    color: c.color,
    archived: c.archived,
  };
}

/**
 * Linhas → árvore, na ordem do núcleo (pais antes dos filhos). Linhas cujo pai não veio
 * (o pai não tem valores no mês) ficam no primeiro nível.
 */
export function budgetTree(
  status: Pick<BudgetStatus, 'lines' | 'month_elapsed'>,
  kind: CategoryKind,
  extra: readonly BudgetLineStatus[] = [],
): BudgetRow[] {
  const lines = [
    ...status.lines,
    ...extra.filter((e) => !status.lines.some((l) => l.category_id === e.category_id)),
  ].filter((l) => l.kind === kind);
  const rows = new Map<string, BudgetRow>();
  const roots: BudgetRow[] = [];
  for (const line of lines) {
    const parent = line.parent_id ? rows.get(line.parent_id) : undefined;
    const hasOwnChildren = lines.some((l) => l.parent_id === line.category_id && l.planned > 0);
    const row: BudgetRow = {
      line,
      depth: parent ? parent.depth + 1 : 0,
      children: [],
      status:
        kind === 'expense'
          ? bulletStatus({
              planned: line.planned,
              actual: line.actual,
              committed: line.committed,
              elapsed: status.month_elapsed,
            })
          : 'ok',
      plannedFromChildren: !line.has_own_line && line.planned > 0 && hasOwnChildren,
    };
    rows.set(line.category_id, row);
    if (parent) parent.children.push(row);
    else roots.push(row);
  }
  return roots;
}

/** Achata a árvore respeitando os pais recolhidos. */
export function visibleRows(
  tree: readonly BudgetRow[],
  expanded: ReadonlySet<string>,
): BudgetRow[] {
  const out: BudgetRow[] = [];
  const walk = (rows: readonly BudgetRow[]) => {
    for (const r of rows) {
      out.push(r);
      if (r.children.length && expanded.has(r.line.category_id)) walk(r.children);
    }
  };
  walk(tree);
  return out;
}

/**
 * Categorias de despesa com gasto no mês e sem previsto — nem nelas nem num ancestral
 * (uma linha no pai vale para todas as subcategorias).
 */
export function unbudgetedWithSpending(status: BudgetStatus): BudgetLineStatus[] {
  const byId = new Map(status.lines.map((l) => [l.category_id, l]));
  const coveredByAncestor = (l: BudgetLineStatus) => {
    let p = l.parent_id ? byId.get(l.parent_id) : undefined;
    while (p) {
      if (p.planned > 0) return true;
      p = p.parent_id ? byId.get(p.parent_id) : undefined;
    }
    return false;
  };
  const hits = status.lines.filter(
    (l) => l.kind === 'expense' && l.planned === 0 && l.actual > 0 && !coveredByAncestor(l),
  );
  // Basta o nível mais alto: um pai sem orçamento já cobre os filhos na lista.
  const ids = new Set(hits.map((l) => l.category_id));
  return hits.filter((l) => !l.parent_id || !ids.has(l.parent_id));
}

export interface BudgetSummary {
  planned: number;
  actual: number;
  remaining: number;
  committed: number;
  /** Realizado + comprometido. */
  projected: number;
  elapsed: number;
  status: BulletStatus;
  ratio: number | null;
  uncategorized: number;
  plannedIncome: number;
  actualIncome: number;
}

export function budgetSummary(s: BudgetStatus): BudgetSummary {
  const t = s.totals;
  return {
    planned: t.planned_expense,
    actual: t.actual_expense,
    remaining: t.planned_expense - t.actual_expense,
    committed: t.committed_expense,
    projected: t.actual_expense + t.committed_expense,
    elapsed: s.month_elapsed,
    status: bulletStatus({
      planned: t.planned_expense,
      actual: t.actual_expense,
      committed: t.committed_expense,
      elapsed: s.month_elapsed,
    }),
    ratio: t.planned_expense > 0 ? t.actual_expense / t.planned_expense : null,
    uncategorized: t.uncategorized_expense,
    plannedIncome: t.planned_income,
    actualIncome: t.actual_income,
  };
}

export interface SuggestionRow {
  category_id: string;
  name: string;
  /** Previsto atual da categoria (linha própria ou soma dos filhos). */
  current: number;
  suggested: number;
  delta: number;
}

/** Sugestão pela média × orçamento atual, só onde muda algo. */
export function suggestionRows(
  status: BudgetStatus,
  suggestion: readonly BudgetLineInput[],
  categories: readonly Category[],
): SuggestionRow[] {
  const lines = new Map(status.lines.map((l) => [l.category_id, l]));
  const names = new Map(categories.map((c) => [c.id, c.name]));
  return suggestion
    .map((s) => {
      const current = lines.get(s.category_id)?.planned ?? 0;
      return {
        category_id: s.category_id,
        name: lines.get(s.category_id)?.category_name ?? names.get(s.category_id) ?? s.category_id,
        current,
        suggested: s.amount,
        delta: s.amount - current,
      };
    })
    .filter((r) => r.delta !== 0);
}

/** Categorias que ainda podem entrar no orçamento (sem linha própria nem valores no mês). */
export function addableCategories(
  categories: readonly Category[],
  status: BudgetStatus,
  kind: CategoryKind,
): Category[] {
  const present = new Set(status.lines.map((l) => l.category_id));
  return categories.filter((c) => c.kind === kind && !c.archived && !present.has(c.id));
}
