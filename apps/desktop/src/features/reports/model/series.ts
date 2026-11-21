/**
 * Transformações puras das respostas do núcleo para as entradas dos gráficos (composição,
 * tendência, comparação, titulares, recorrências, cartões e patrimônio). Cores sempre pela
 * entidade; o que não tem cor é reunido em "Outras".
 */
import type { Category } from '@/api/generated/Category';
import type { Member } from '@/api/generated/Member';
import type { MemberBreakdown } from '@/api/generated/MemberBreakdown';
import type { MonthAmount } from '@/api/generated/MonthAmount';
import type { MonthlyEvolution } from '@/api/generated/MonthlyEvolution';
import type { NetWorthPoint } from '@/api/generated/NetWorthPoint';
import type { PeriodComparison } from '@/api/generated/PeriodComparison';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { SpendingSummary } from '@/api/generated/SpendingSummary';
import type { ColorRef } from '@/charts/colorMath';
import type { ComparisonInput } from '@/charts/comparison';
import type { CompositionNode } from '@/charts/composition';
import type { GroupedBarsInput } from '@/charts/groupedBarsOption';
import type { StackedAreaInput } from '@/charts/stackedAreaOption';
import type { StackedBarsInput } from '@/charts/stackedBarsOption';
import { expandOccurrences } from '@/charts/timeline';
import type { TimelineRow } from '@/charts/RecurrenceTimeline';
import { t } from '@/i18n';

import { OTHERS_KEY, othersLabel, type EntityColors } from './colors';

export const NO_CATEGORY_KEY = '__sem_categoria__';
const GRAY: ColorRef = '@gray';

// ── Composição ───────────────────────────────────────────────────

export interface CompositionModel {
  nodes: CompositionNode[];
  /** Nó → categorias do drill-down (`null` = sem categoria). */
  drill: Map<string, (string | null)[]>;
  total: number;
}

/**
 * `reports.spending` por categoria-folha → árvore de dois níveis (categoria de primeiro
 * nível › subcategoria). Gasto lançado direto no pai vira o filho "(direto)".
 */
export function compositionModel(
  summary: SpendingSummary,
  categories: readonly Category[],
  colors: EntityColors,
): CompositionModel {
  const byId = new Map(categories.map((c) => [c.id, c]));
  const rootOf = (id: string): Category | undefined => {
    let c = byId.get(id);
    while (c?.parent_id && byId.has(c.parent_id)) c = byId.get(c.parent_id);
    return c;
  };
  const level1Of = (id: string): Category | undefined => {
    let c = byId.get(id);
    while (c?.parent_id) {
      const p = byId.get(c.parent_id);
      if (!p?.parent_id) return c;
      c = p;
    }
    return undefined;
  };
  interface Root {
    id: string;
    name: string;
    order: number;
    value: number;
    subs: Map<string, { name: string; value: number }>;
  }
  const roots = new Map<string, Root>();
  let uncategorized = 0;
  for (const g of summary.groups) {
    if (g.amount <= 0) continue;
    if (!g.key) {
      uncategorized += g.amount;
      continue;
    }
    const root = rootOf(g.key);
    const rootId = root?.id ?? g.key;
    const r: Root = roots.get(rootId) ?? {
      id: rootId,
      name: root?.name ?? g.label,
      order: root?.sort_order ?? 999,
      value: 0,
      subs: new Map(),
    };
    r.value += g.amount;
    const sub = level1Of(g.key);
    const subId = sub?.id ?? `${rootId}:direto`;
    const s = r.subs.get(subId) ?? {
      name: sub?.name ?? t('relatorios-direto', { categoria: r.name }),
      value: 0,
    };
    s.value += g.amount;
    r.subs.set(subId, s);
    roots.set(rootId, r);
  }
  const drill = new Map<string, (string | null)[]>();
  const colored: CompositionNode[] = [];
  const others: CompositionNode[] = [];
  const sorted = [...roots.values()].sort((a, b) => b.value - a.value);
  for (const r of sorted) {
    const color = colors.get(r.id);
    const children = [...r.subs.entries()]
      .sort((a, b) => b[1].value - a[1].value)
      .map(([id, s]) => {
        drill.set(id, [id.endsWith(':direto') ? r.id : id]);
        return { id, name: s.name, value: s.value, color: color ?? GRAY };
      });
    drill.set(r.id, [r.id]);
    const node: CompositionNode = {
      id: r.id,
      name: r.name,
      value: r.value,
      color: color ?? GRAY,
      children: children.length > 1 || !children[0]?.id.endsWith(':direto') ? children : undefined,
    };
    if (color) colored.push(node);
    else others.push({ ...node, children: undefined });
  }
  const nodes = [...colored];
  if (others.length > 0) {
    nodes.push({
      id: OTHERS_KEY,
      name: othersLabel(),
      value: others.reduce((s, n) => s + n.value, 0),
      color: GRAY,
      children: others,
    });
    drill.set(
      OTHERS_KEY,
      others.map((n) => n.id),
    );
  }
  if (uncategorized > 0) {
    nodes.push({
      id: NO_CATEGORY_KEY,
      name: t('relatorios-sem-categoria'),
      value: uncategorized,
      color: '@dim',
    });
    drill.set(NO_CATEGORY_KEY, [null]);
  }
  return { nodes, drill, total: summary.total };
}

// ── Tendência ────────────────────────────────────────────────────

export interface TrendModel {
  input: StackedBarsInput;
  /** Série → categorias do drill-down. */
  drill: Map<string, (string | null)[]>;
  /** Variação da média móvel do último mês contra a de 12 meses antes (null sem base). */
  trend: number | null;
}

/**
 * `reports.monthly_evolution` (pedida com `top_n` alto) → barras na ordem fixa das
 * categorias com cor; o resto (e "sem categoria") vira "Outras".
 */
export function trendModel(evo: MonthlyEvolution, colors: EntityColors): TrendModel {
  const n = evo.months.length;
  const drill = new Map<string, (string | null)[]>();
  const byKey = new Map(evo.series.map((s) => [s.key, s]));
  const bars: StackedBarsInput['bars'][number][] = [];
  for (const key of colors.keys) {
    const s = byKey.get(key);
    if (!s || !s.values.some((v) => v !== 0)) continue;
    bars.push({ key, name: s.label, color: colors.get(key)!, values: s.values });
    drill.set(key, [key]);
  }
  const rest = evo.series.filter((s) => !s.key || !colors.get(s.key) || s.key === OTHERS_KEY);
  if (rest.length > 0) {
    const values = Array.from({ length: n }, (_, i) =>
      rest.reduce((sum, s) => sum + (s.values[i] ?? 0), 0),
    );
    if (values.some((v) => v !== 0)) {
      bars.push({ key: OTHERS_KEY, name: othersLabel(), color: GRAY, values });
      drill.set(
        OTHERS_KEY,
        rest.filter((s) => s.key && s.key !== OTHERS_KEY).map((s) => s.key!),
      );
    }
  }
  const avg = evo.moving_avg_3;
  const last = avg[n - 1];
  const yearAgo = n > 12 ? avg[n - 13] : avg[0];
  return {
    input: {
      months: evo.months,
      bars,
      lines: [
        { key: 'avg', name: t('relatorios-media-movel-3m'), color: '@text', values: avg },
        {
          key: 'income',
          name: t('relatorios-receitas'),
          color: '@positive',
          values: evo.income,
          dashed: true,
        },
      ],
      currency: evo.currency,
    },
    drill,
    trend: last != null && yearAgo ? (last - yearAgo) / yearAgo : null,
  };
}

// ── Comparação ───────────────────────────────────────────────────

export function comparisonRows(cmp: PeriodComparison, colors: EntityColors): ComparisonInput[] {
  return cmp.rows.map((r) => ({
    key: r.key ?? NO_CATEGORY_KEY,
    label: r.label,
    a: r.a,
    b: r.b,
    color: (r.key && colors.get(r.key)) || GRAY,
  }));
}

/** Para o slope: categorias com cor e "Outras" somando o resto. */
export function slopeRows(
  rows: readonly ComparisonInput[],
  colors: EntityColors,
): ComparisonInput[] {
  const kept = rows.filter((r) => colors.get(r.key));
  const rest = rows.filter((r) => !colors.get(r.key));
  if (rest.length === 0) return kept;
  return [
    ...kept,
    {
      key: OTHERS_KEY,
      label: othersLabel(),
      a: rest.reduce((s, r) => s + r.a, 0),
      b: rest.reduce((s, r) => s + r.b, 0),
      color: GRAY,
    },
  ];
}

// ── Titulares ────────────────────────────────────────────────────

/** Linhas com nome próprio; as demais categorias somam em "Outras". */
export const MEMBER_ROWS = 10;

export function membersModel(
  data: MemberBreakdown,
  colors: EntityColors,
  noMemberKey: string,
): { input: GroupedBarsInput; folded: string[] } {
  const groups = data.members.map((m, gi) => ({
    key: m.key ?? noMemberKey,
    name: m.label,
    color: colors.get(m.key ?? noMemberKey) ?? GRAY,
    gi,
  }));
  const top = data.categories.slice(0, MEMBER_ROWS);
  const rest = data.categories.slice(MEMBER_ROWS);
  const rows = top.map((c) => ({ key: c.key ?? NO_CATEGORY_KEY, label: c.label }));
  const values = groups.map((g) => top.map((c) => c.values[g.gi] ?? 0));
  if (rest.length > 0) {
    rows.push({ key: OTHERS_KEY, label: othersLabel() });
    groups.forEach((g, i) => values[i]!.push(rest.reduce((s, c) => s + (c.values[g.gi] ?? 0), 0)));
  }
  return {
    input: {
      rows,
      groups: groups.map((g, i) => ({
        key: g.key,
        name: g.name,
        color: g.color,
        values: values[i]!,
      })),
      currency: data.currency,
    },
    folded: rest.map((c) => c.key ?? NO_CATEGORY_KEY),
  };
}

// ── Recorrências ─────────────────────────────────────────────────

export interface RecurrenceSummary {
  /** Despesas recorrentes ativas por mês (positivo). */
  expenseMonthly: number;
  /** Receitas recorrentes ativas por mês. */
  incomeMonthly: number;
  /** Fração das receitas recorrentes já comprometida. */
  committedShare: number | null;
  count: number;
}

export function recurrenceSummary(list: readonly Recurrence[]): RecurrenceSummary {
  const active = list.filter((r) => r.active);
  const expense = -active
    .filter((r) => r.kind === 'expense')
    .reduce((s, r) => s + r.monthly_equivalent, 0);
  const income = active
    .filter((r) => r.kind === 'income')
    .reduce((s, r) => s + r.monthly_equivalent, 0);
  return {
    expenseMonthly: expense,
    incomeMonthly: income,
    committedShare: income > 0 ? expense / income : null,
    count: active.filter((r) => r.kind === 'expense').length,
  };
}

export function timelineRows(
  list: readonly Recurrence[],
  categories: readonly Category[],
  colors: EntityColors,
  window: { from: string; to: string },
): TimelineRow[] {
  const byId = new Map(categories.map((c) => [c.id, c]));
  const rootOf = (id: string | null) => {
    let c = id ? byId.get(id) : undefined;
    while (c?.parent_id && byId.has(c.parent_id)) c = byId.get(c.parent_id);
    return c;
  };
  return list
    .filter((r) => r.active && r.kind === 'expense')
    .map((r) => {
      const first = r.next_dates[0];
      const dates = first
        ? [...new Set([...r.next_dates, ...expandOccurrences(r.spec, first, window.to)])]
            .filter((d) => d >= window.from && d <= window.to)
            .sort()
        : [];
      const root = rootOf(r.category_id);
      return {
        id: r.id,
        label: r.description,
        detail: [r.human, root?.name, r.account_name].filter(Boolean).join(' · '),
        amount: r.amount,
        monthly: r.monthly_equivalent,
        currency: r.currency,
        color: (root && colors.get(root.id)) || 'var(--line-strong)',
        dates,
      };
    })
    .sort((a, b) => a.monthly - b.monthly || a.label.localeCompare(b.label, 'pt-BR'));
}

// ── Cartões ──────────────────────────────────────────────────────

export function cardsModel(
  cards: readonly { id: string; name: string; data: readonly MonthAmount[] | undefined }[],
  colors: EntityColors,
  currency: string,
): StackedAreaInput & { total: number; totals: number[] } {
  const months = [...new Set(cards.flatMap((c) => (c.data ?? []).map((d) => d.month)))].sort();
  const series = cards.map((c) => {
    const map = new Map((c.data ?? []).map((d) => [d.month, d.amount]));
    return {
      key: c.id,
      name: c.name,
      color: colors.get(c.id) ?? GRAY,
      values: months.map((m) => map.get(m) ?? 0),
      stack: 'faturas',
    };
  });
  const totals = months.map((_, i) => series.reduce((s, x) => s + (x.values[i] ?? 0), 0));
  return {
    months,
    series,
    currency,
    totalLabel: t('relatorios-total-faturas'),
    totals,
    total: totals.reduce((s, v) => s + v, 0),
  };
}

// ── Patrimônio ───────────────────────────────────────────────────

export function netWorthModel(
  points: readonly NetWorthPoint[],
  currency: string,
): StackedAreaInput {
  return {
    months: points.map((p) => p.month),
    currency,
    series: [
      {
        key: 'accounts',
        name: t('relatorios-contas'),
        color: '@s0',
        values: points.map((p) => p.assets - p.investments - p.physical),
        stack: 'ativos',
      },
      {
        key: 'investments',
        name: t('relatorios-investimentos'),
        color: '@s2',
        values: points.map((p) => p.investments),
        stack: 'ativos',
      },
      {
        key: 'physical',
        name: t('relatorios-bens'),
        color: '@s3',
        values: points.map((p) => p.physical),
        stack: 'ativos',
      },
      {
        key: 'liabilities',
        name: t('relatorios-dividas'),
        color: '@s1',
        values: points.map((p) => -p.liabilities),
        stack: 'passivos',
      },
      {
        key: 'net',
        name: t('relatorios-patrimonio-liquido'),
        color: '@text',
        values: points.map((p) => p.net),
      },
    ],
  };
}

/** Titulares na ordem de cadastro (para cores e filtros). */
export function sortedMembers(members: readonly Member[]): Member[] {
  return [...members].sort(
    (a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name, 'pt-BR'),
  );
}
