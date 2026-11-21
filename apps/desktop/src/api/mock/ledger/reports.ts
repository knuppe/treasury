/**
 * Relatórios do mock, espelhando `reports/mod.rs` (mesmas formas e regras de cálculo).
 */
import type { Basis } from '@/api/generated/Basis';
import { find as findRow, rows as allRows } from './store';
import type { CalendarData } from '@/api/generated/CalendarData';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { ComparisonRow } from '@/api/generated/ComparisonRow';
import type { EvolutionSeries } from '@/api/generated/EvolutionSeries';
import type { GroupAmount } from '@/api/generated/GroupAmount';
import type { GroupBy } from '@/api/generated/GroupBy';
import type { LoanView } from '@/api/generated/LoanView';
import type { MemberBreakdown } from '@/api/generated/MemberBreakdown';
import type { MonthFlow } from '@/api/generated/MonthFlow';
import type { MonthlyEvolution } from '@/api/generated/MonthlyEvolution';
import type { NetWorth } from '@/api/generated/NetWorth';
import type { NetWorthPoint } from '@/api/generated/NetWorthPoint';
import type { Period } from '@/api/generated/Period';
import type { PeriodComparison } from '@/api/generated/PeriodComparison';
import type { SankeyData } from '@/api/generated/SankeyData';
import type { SankeyDelta } from '@/api/generated/SankeyDelta';
import type { SankeyItem } from '@/api/generated/SankeyItem';
import type { SankeyItems } from '@/api/generated/SankeyItems';
import type { SankeyItemsGroup } from '@/api/generated/SankeyItemsGroup';
import type { SankeyItemsPeriod } from '@/api/generated/SankeyItemsPeriod';
import type { SankeyLink } from '@/api/generated/SankeyLink';
import type { SankeyNode } from '@/api/generated/SankeyNode';
import type { SankeyNodeKind } from '@/api/generated/SankeyNodeKind';
import type { SankeyTotals } from '@/api/generated/SankeyTotals';
import type { SpendingArgs } from '@/api/generated/SpendingArgs';
import type { SpendingSummary } from '@/api/generated/SpendingSummary';
import type { UpcomingItem } from '@/api/generated/UpcomingItem';
import { extraMockHandler } from '@/api/mockRegistry';
import { formatDate, formatMoney } from '@/lib/format';
import { addDays, daysBetween, monthBounds, monthsBetween, shiftMonth } from '@/lib/months';

import {
  aggregate,
  balancesAt,
  categoryLabel,
  cellAmount,
  cellCount,
  decodeKey,
  divRound,
  emptyCell,
  firstFlowMonth,
  flowKind,
  flowParts,
  Fx,
  KEY,
  monthLabel,
  ratio,
  total,
  type Cell,
} from './engine';
import { ACCOUNTS, BASE, category, MEMBERS, rootOf, withDescendants } from './fixture';
import { invalid } from './errors';
import { getLedger, type LedgerTxn } from './ledger';
import { investmentsAt } from './portfolio';
import { mockDebtsAt, physicalNetWorthItems } from '../physical';

const NO_CATEGORY = 'Sem categoria';
const NO_MEMBER = 'Sem membro';

const share = (amount: number, totalAmount: number) => (totalAmount > 0 ? amount / totalAmount : 0);

function sortedMissing(set: Set<string>): string[] {
  return [...set].sort();
}

// ── Patrimônio ───────────────────────────────────────────────────

export function netWorth(asOf: string): NetWorth {
  const fx = new Fx();
  const out: NetWorth = {
    as_of: asOf,
    base_currency: BASE,
    assets: 0,
    liabilities: 0,
    net: 0,
    investments: 0,
    physical: 0,
    debts: 0,
    accounts: [],
    missing_fx: [],
    estimated: false,
  };
  const start = getLedger().start;
  for (const b of balancesAt(asOf)) {
    const acc = ACCOUNTS.find((a) => a.id === b.id)!;
    if (!acc.include_in_net_worth || !(start <= asOf || b.balance !== 0)) continue;
    const c = fx.convert(b.balance, b.currency, asOf);
    if (c) {
      if (c.amount >= 0) out.assets += c.amount;
      else out.liabilities += -c.amount;
    }
    out.accounts.push({
      account_id: b.id,
      name: b.name,
      kind: acc.kind,
      currency: b.currency,
      balance: b.balance,
      balance_base: c ? c.amount : null,
      fx_estimated: c?.estimated ?? false,
    });
  }
  out.investments = investmentsAt(asOf);
  out.assets += out.investments;
  const physical = physicalNetWorthItems(asOf);
  out.physical = physical.reduce((s, i) => s + i.value, 0);
  out.assets += out.physical;
  // Contratos de 07 · Dívidas: o saldo devedor é passivo (os bens financiados entram pelo
  // valor bruto acima; a dívida conta uma vez só, aqui).
  out.debts = mockDebtsAt(asOf);
  out.liabilities += out.debts;
  out.net = out.assets - out.liabilities;
  out.missing_fx = sortedMissing(fx.missing);
  out.estimated = fx.estimated || physical.some((i) => i.estimated);
  return out;
}

export function netWorthHistory(months: number, endMonth: string): NetWorthPoint[] {
  const n = Math.min(Math.max(months, 1), 240);
  return monthsBetween(shiftMonth(endMonth, -(n - 1)), endMonth)
    .map((month): NetWorthPoint | null => {
      const last = monthBounds(month).to;
      if (last < getLedger().start) return null;
      const nw = netWorth(last);
      return {
        month,
        as_of: last,
        assets: nw.assets,
        liabilities: nw.liabilities,
        net: nw.net,
        investments: nw.investments,
        physical: nw.physical,
        debts: nw.debts,
        missing_fx: nw.missing_fx,
        estimated: nw.estimated,
      };
    })
    .filter((p): p is NetWorthPoint => p !== null);
}

// ── Fluxo do mês ─────────────────────────────────────────────────

export function monthFlow(fromMonth: string, toMonth: string, basis: Basis): MonthFlow[] {
  const months = monthsBetween(fromMonth, toMonth);
  const from = monthBounds(months[0]!).from;
  const to = monthBounds(months[months.length - 1]!).to;
  const agg = aggregate({ from, to, basis }, KEY.month);
  return months.map((m) => {
    const c = agg.cells.get(`${m}|\u0000`) ?? emptyCell();
    const net = c.income - c.expense;
    return {
      month: m,
      income: c.income,
      expense: c.expense,
      net,
      savings_rate: ratio(net, c.income),
      missing_fx: sortedMissing(c.missing),
      estimated: c.estimated,
    };
  });
}

// ── Gastos agrupados ─────────────────────────────────────────────

export function spending(a: SpendingArgs, defaultBasis: Basis): SpendingSummary {
  const basis = a.basis ?? defaultBasis;
  const kind: CategoryKind = a.kind ?? 'expense';
  const groupBy: GroupBy = a.group_by ?? 'category';
  const cats = a.category_ids?.length ? withDescendants(a.category_ids) : null;
  const q = {
    from: a.from,
    to: a.to,
    basis,
    kind,
    member_ids: a.member_ids ?? null,
    account_ids: a.account_ids ?? null,
    category_ids: cats,
  };
  const keyFn = {
    category: KEY.category,
    parent_category: KEY.root,
    member: KEY.member,
    account: KEY.account,
    month: KEY.month,
  }[groupBy];
  const agg = aggregate(q, keyFn);
  const cells = new Map<string | null, { amount: number; count: number }>();
  for (const [k, c] of agg.cells) {
    const [key] = decodeKey(k);
    const amount = cellAmount(c, kind);
    const count = cellCount(c, kind);
    if (amount !== 0 || count !== 0) cells.set(key, { amount, count });
  }
  if (groupBy === 'month') {
    for (const m of monthsBetween(a.from.slice(0, 7), a.to.slice(0, 7))) {
      if (!cells.has(m)) cells.set(m, { amount: 0, count: 0 });
    }
  }
  const sum = [...cells.values()].reduce((s, c) => s + c.amount, 0);
  const count = cellCount(total(aggregate(q)), kind);
  let groups: GroupAmount[] = [...cells.entries()].map(([key, v]) => {
    let label: string;
    let path: string | null = null;
    let parent_key: string | null = null;
    if (groupBy === 'category' || groupBy === 'parent_category') {
      const l = categoryLabel(key);
      label = key ? l.label : NO_CATEGORY;
      path = l.path;
      parent_key = l.parent;
    } else if (groupBy === 'member') {
      label = MEMBERS.find((m) => m.id === key)?.name ?? key ?? NO_MEMBER;
    } else if (groupBy === 'account') {
      label = ACCOUNTS.find((x) => x.id === key)?.name ?? key ?? '';
    } else {
      label = key ? monthLabel(key) : '';
    }
    return {
      key,
      label,
      parent_key,
      path,
      amount: v.amount,
      share: share(v.amount, sum),
      count: v.count,
      color: null,
    };
  });
  if (groupBy === 'month') {
    groups.sort((x, y) => (x.key ?? '').localeCompare(y.key ?? ''));
  } else {
    groups = groups.sort((x, y) => y.amount - x.amount || x.label.localeCompare(y.label));
  }
  return {
    from: a.from,
    to: a.to,
    basis,
    kind,
    currency: BASE,
    total: sum,
    count,
    groups,
    missing_fx: sortedMissing(agg.missing),
    estimated: agg.estimated,
  };
}

// ── Sankey ───────────────────────────────────────────────────────

/** Subcategoria de nível 1 (filha direta da raiz) de uma categoria. */
function level1(id: string): string | null {
  let c = category(id);
  while (c?.parent_id) {
    const parent = category(c.parent_id);
    if (!parent?.parent_id) return c.id;
    c = parent;
  }
  return null;
}

/** Período anterior de mesma duração (meses inteiros recuam em meses), como `previous_period`. */
export function sankeyPreviousPeriod(from: string, to: string): Period {
  const fromM = from.slice(0, 7);
  const toM = to.slice(0, 7);
  if (from === monthBounds(fromM).from && to === monthBounds(toM).to) {
    const n = monthsBetween(fromM, toM).length;
    return {
      from: monthBounds(shiftMonth(fromM, -n)).from,
      to: monthBounds(shiftMonth(toM, -n)).to,
    };
  }
  const days = daysBetween(from, to) + 1;
  return { from: addDays(from, -days), to: addDays(from, -1) };
}

/** Categorias ligadas a contratos (`loan.category_id`) → nome do contrato. */
function debtCategories(): Map<string, string> {
  const out = new Map<string, string>();
  let loans: LoanView[];
  try {
    loans = (extraMockHandler('debt.list')?.({ include_archived: true }) ?? []) as LoanView[];
  } catch {
    loans = [];
  }
  const sorted = [...loans].sort(
    (a, b) => Number(a.archived) - Number(b.archived) || a.name.localeCompare(b.name),
  );
  for (const l of sorted)
    if (l.category_id && !out.has(l.category_id)) out.set(l.category_id, l.name);
  return out;
}

export interface InvestmentLeg {
  txn: LedgerTxn;
  /** Saída (aporte); senão, entrada (resgate). */
  out: boolean;
  /** Conta da outra perna (a corretora), quando houver. */
  other: string | null;
}

/** Aportes e resgates: perna `investment` numa conta que não é de corretora. */
export function investmentLegs(from: string, to: string): InvestmentLeg[] {
  const ledger = getLedger();
  const kindOf = (id: string) => ACCOUNTS.find((a) => a.id === id)?.kind;
  const out: InvestmentLeg[] = [];
  for (const t of ledger.txns) {
    if (t.kind !== 'investment' || t.status === 'planned' || t.date < from || t.date > to) continue;
    if (kindOf(t.account_id) === 'brokerage') continue;
    const other = t.transfer_id
      ? ledger.txns.find((o) => o.transfer_id === t.transfer_id && o.id !== t.id)
      : undefined;
    if (t.amount > 0 && other && kindOf(other.account_id) !== 'brokerage') continue;
    out.push({ txn: t, out: t.amount < 0, other: other?.account_id ?? null });
  }
  return out.sort(
    (a, b) => a.txn.date.localeCompare(b.txn.date) || a.txn.id.localeCompare(b.txn.id),
  );
}

function investmentFlows(from: string, to: string, fx: Fx): { out: number; back: number } {
  let out = 0;
  let back = 0;
  for (const leg of investmentLegs(from, to)) {
    const c = fx.convert(leg.txn.amount, leg.txn.currency, leg.txn.date);
    if (!c) continue;
    if (leg.out) out -= c.amount;
    else back += c.amount;
  }
  return { out, back };
}

/** Onde um valor de receita ou despesa entra no diagrama (como `Place` no núcleo). */
type Place =
  | { kind: 'income'; root: string | null }
  | { kind: 'expense'; root: string | null; sub: string | null }
  | { kind: 'debt'; cat: string };

function debtOf(id: string, debts: Map<string, string>): string | null {
  for (let c = category(id); c; c = c.parent_id ? category(c.parent_id) : undefined) {
    if (debts.has(c.id)) return c.id;
  }
  return null;
}

function placeOf(cat: string | null, income: boolean, debts: Map<string, string>): Place {
  const info = category(cat);
  if (income) return { kind: 'income', root: info ? rootOf(info.id) : null };
  const d = info ? debtOf(info.id, debts) : null;
  if (d) return { kind: 'debt', cat: d };
  return {
    kind: 'expense',
    root: info ? rootOf(info.id) : null,
    sub: info?.parent_id ? level1(info.id) : null,
  };
}

const placeKey = (p: Place) =>
  p.kind === 'debt'
    ? `debt|${p.cat}`
    : `${p.kind}|${p.root ?? ''}|${p.kind === 'expense' ? (p.sub ?? '') : ''}`;

interface SankeyGraph {
  nodes: SankeyNode[];
  links: SankeyLink[];
  /** Valor de cada nó e das subcategorias que o diagrama não desenha (como no núcleo). */
  values: Map<string, number>;
  totals: SankeyTotals;
  missing: Set<string>;
  estimated: boolean;
}

function sankeyGraph(
  from: string,
  to: string,
  basis: Basis,
  depth: number,
  debts: Map<string, string>,
): SankeyGraph {
  const fx = new Fx();
  const agg = aggregate({ from, to, basis }, KEY.category, KEY.none, fx);
  const flows = investmentFlows(from, to, fx);
  const income = new Map<string | null, number>();
  const expense = new Map<string | null, Map<string | null, number>>();
  const debt = new Map<string, number>();
  for (const [k, cell] of agg.cells) {
    const [cat] = decodeKey(k);
    if (cell.income !== 0) {
      const p = placeOf(cat, true, debts);
      if (p.kind === 'income') income.set(p.root, (income.get(p.root) ?? 0) + cell.income);
    }
    if (cell.expense === 0) continue;
    const p = placeOf(cat, false, debts);
    if (p.kind === 'debt') {
      debt.set(p.cat, (debt.get(p.cat) ?? 0) + cell.expense);
    } else if (p.kind === 'expense') {
      const subs = expense.get(p.root) ?? new Map<string | null, number>();
      subs.set(p.sub, (subs.get(p.sub) ?? 0) + cell.expense);
      expense.set(p.root, subs);
    }
  }
  const nameOf = (id: string | null) => category(id)?.name ?? NO_CATEGORY;
  const suffix = (id: string | null) => id ?? 'none';
  const nodes: SankeyNode[] = [];
  const links: SankeyLink[] = [];
  const seen = new Set<string>();
  const addNode = (n: SankeyNode) => {
    if (!seen.has(n.id)) {
      seen.add(n.id);
      nodes.push(n);
    }
  };
  const special = (id: string, name: string, kind: SankeyNodeKind): SankeyNode => ({
    id,
    name,
    kind,
    category_id: null,
  });
  const hub = special('hub', 'Orçamento', 'hub');
  const incomeLinks = [...income.entries()]
    .filter(([, v]) => v > 0)
    .sort((x, y) => y[1] - x[1] || nameOf(x[0]).localeCompare(nameOf(y[0])));
  const totalIn = incomeLinks.reduce((s, [, v]) => s + v, 0);
  const roots = [...expense.entries()]
    .map(([root, subsMap]) => {
      const subs = [...subsMap.entries()]
        .filter(([, v]) => v > 0)
        .sort((x, y) => y[1] - x[1] || nameOf(x[0]).localeCompare(nameOf(y[0])));
      return { root, subs, value: subs.reduce((s, [, v]) => s + v, 0) };
    })
    .filter((r) => r.value > 0)
    .sort((x, y) => y.value - x.value || nameOf(x.root).localeCompare(nameOf(y.root)));
  const debtLinks = [...debt.entries()]
    .filter(([, v]) => v > 0)
    .sort((x, y) => y[1] - x[1] || (debts.get(x[0]) ?? '').localeCompare(debts.get(y[0]) ?? ''));
  const totalDebt = debtLinks.reduce((s, [, v]) => s + v, 0);
  const totalRoots = roots.reduce((s, r) => s + r.value, 0);
  const sources = totalIn + flows.back;
  const uses = totalRoots + totalDebt + flows.out;
  const link = (source: string, target: string, value: number) =>
    links.push({ source, target, value });

  for (const [root, v] of incomeLinks) {
    const id = `income:${suffix(root)}`;
    addNode({
      id,
      name: root ? nameOf(root) : 'Receitas sem categoria',
      kind: 'income',
      category_id: root,
    });
    addNode(hub);
    link(id, 'hub', v);
  }
  if (flows.back > 0) {
    addNode(special('redemption', 'Resgates', 'redemption'));
    addNode(hub);
    link('redemption', 'hub', flows.back);
  }
  if (uses > sources) {
    addNode(special('deficit', 'Déficit', 'deficit'));
    addNode(hub);
    link('deficit', 'hub', uses - sources);
  }
  for (const r of roots) {
    const id = `expense:${suffix(r.root)}`;
    addNode(hub);
    addNode({ id, name: nameOf(r.root), kind: 'expense', category_id: r.root });
    link('hub', id, r.value);
  }
  if (totalDebt > 0) {
    addNode(hub);
    addNode({
      ...special('debt', 'Pagamento de dívidas', 'debt'),
      category_id: debtLinks.length === 1 ? debtLinks[0]![0] : null,
    });
    link('hub', 'debt', totalDebt);
  }
  if (flows.out > 0) {
    addNode(hub);
    addNode(special('investment', 'Investimentos', 'investment'));
    link('hub', 'investment', flows.out);
  }
  if (sources > uses) {
    addNode(hub);
    addNode(special('savings', 'Sobra', 'savings'));
    link('hub', 'savings', sources - uses);
  }
  if (depth >= 2) {
    for (const r of roots) {
      if (r.root === null || !r.subs.some(([s]) => s !== null)) continue;
      const parent = `expense:${suffix(r.root)}`;
      for (const [sub, v] of r.subs) {
        const node: SankeyNode = sub
          ? { id: `sub:${sub}`, name: nameOf(sub), kind: 'subcategory', category_id: sub }
          : {
              id: `sub:${suffix(r.root)}`,
              name: `${nameOf(r.root)} (sem subcategoria)`,
              kind: 'subcategory',
              category_id: r.root,
            };
        addNode(node);
        link(parent, node.id, v);
      }
    }
    if (debtLinks.length > 1) {
      for (const [cat, v] of debtLinks) {
        const id = `debt:${cat}`;
        addNode({ id, name: debts.get(cat) ?? '', kind: 'subcategory', category_id: cat });
        link('debt', id, v);
      }
    }
  }
  const values = sankeyNodeValues(links);
  if (depth >= 2) {
    for (const r of roots) {
      const only = r.subs.length === 1 ? r.subs[0]! : null;
      if (r.root && only && only[0] === null && !values.has(`sub:${r.root}`))
        values.set(`sub:${r.root}`, only[1]);
    }
    if (debtLinks.length === 1 && !values.has(`debt:${debtLinks[0]![0]}`))
      values.set(`debt:${debtLinks[0]![0]}`, debtLinks[0]![1]);
  }
  const t = total(agg);
  return {
    nodes,
    links,
    values,
    totals: {
      from,
      to,
      income: t.income,
      expense: t.expense,
      debt_payments: totalDebt,
      investments: flows.out,
      redemptions: flows.back,
      leftover: t.income + flows.back - t.expense - flows.out,
      savings_rate: ratio(t.income - t.expense, t.income),
    },
    missing: fx.missing,
    estimated: fx.estimated,
  };
}

/** Valor de cada nó: o maior entre o que entra e o que sai. */
function sankeyNodeValues(links: readonly SankeyLink[]): Map<string, number> {
  const inflow = new Map<string, number>();
  const outflow = new Map<string, number>();
  for (const l of links) {
    outflow.set(l.source, (outflow.get(l.source) ?? 0) + l.value);
    inflow.set(l.target, (inflow.get(l.target) ?? 0) + l.value);
  }
  const out = new Map(inflow);
  for (const [k, v] of outflow) out.set(k, Math.max(out.get(k) ?? 0, v));
  return out;
}

function sankeyParents(links: readonly SankeyLink[]): Map<string, string> {
  return new Map(
    links.filter((l) => l.source !== 'hub' && l.target !== 'hub').map((l) => [l.target, l.source]),
  );
}

function compareSankey(cur: SankeyGraph, prev: SankeyGraph): SankeyDelta[] {
  const cv = cur.values;
  const pv = prev.values;
  const cp = sankeyParents(cur.links);
  const pp = sankeyParents(prev.links);
  const row = (n: SankeyNode, parent: string | undefined): SankeyDelta => {
    const current = cv.get(n.id) ?? 0;
    const previous = pv.get(n.id) ?? 0;
    const delta = current - previous;
    return {
      id: n.id,
      name: n.name,
      kind: n.kind,
      category_id: n.category_id,
      parent_id: parent ?? null,
      current,
      previous,
      delta,
      delta_pct: previous > 0 ? delta / previous : null,
      share_of_income: ratio(current, cur.totals.income),
    };
  };
  const seen = new Set(cur.nodes.map((n) => n.id));
  return [
    ...cur.nodes.filter((n) => n.kind !== 'hub').map((n) => row(n, cp.get(n.id))),
    ...prev.nodes
      .filter((n) => n.kind !== 'hub' && !seen.has(n.id))
      .map((n) => row(n, pp.get(n.id))),
  ];
}

export function sankey(
  from: string,
  to: string,
  basis: Basis,
  depth = 2,
  withPrevious = true,
): SankeyData {
  if (from > to)
    invalid(`a data inicial (${formatDate(from)}) é posterior à final (${formatDate(to)})`);
  const d = Math.min(2, Math.max(1, Math.trunc(depth)));
  const debts = debtCategories();
  const cur = sankeyGraph(from, to, basis, d, debts);
  let previous: SankeyTotals | null = null;
  let comparison: SankeyDelta[] = [];
  if (withPrevious) {
    const p = sankeyPreviousPeriod(from, to);
    const prev = sankeyGraph(p.from, p.to, basis, d, debts);
    previous = prev.totals;
    comparison = compareSankey(cur, prev);
  }
  // Faturas pagas no período (perna que entra no cartão), como no núcleo.
  const cardBillsPaid = allRows('txn')
    .filter(
      (t) =>
        t.kind === 'card_payment' &&
        t.amount > 0 &&
        t.status !== 'planned' &&
        t.date >= from &&
        t.date <= to &&
        findRow('account', t.account_id)?.kind === 'credit_card',
    )
    .reduce((s, t) => s + t.amount, 0);
  return {
    from,
    to,
    basis,
    card_bills_paid: cardBillsPaid,
    nodes: cur.nodes,
    links: cur.links,
    total_income: cur.totals.income,
    total_expense: cur.totals.expense,
    currency: BASE,
    missing_fx: sortedMissing(cur.missing),
    estimated: cur.estimated,
    depth: d,
    totals: cur.totals,
    previous,
    comparison,
  };
}

// ── Analítico do fluxo ───────────────────────────────────────────

const DERIVED = new Set(['savings', 'deficit']);
const money = (v: number) => formatMoney(v, BASE).replace('−', '-');

function validNode(id: string): boolean {
  if (['savings', 'deficit', 'investment', 'redemption', 'debt'].includes(id)) return true;
  return ['income:', 'expense:', 'sub:', 'debt:'].some(
    (p) => id.startsWith(p) && id.length > p.length,
  );
}

/** Nós de um valor: o de 1º nível (ou de dívidas) e a subcategoria (ou o contrato). */
function nodesOf(p: Place): [string, string | null] {
  if (p.kind === 'income') return [`income:${p.root ?? 'none'}`, null];
  if (p.kind === 'debt') return ['debt', `debt:${p.cat}`];
  return [`expense:${p.root ?? 'none'}`, p.root ? `sub:${p.sub ?? p.root}` : null];
}

function unitLabel(p: Place, debts: Map<string, string>): string {
  const name = (id: string | null) => category(id)?.name ?? NO_CATEGORY;
  if (p.kind === 'income')
    return p.root ? `A receita de ${name(p.root)}` : 'Receitas sem categoria';
  if (p.kind === 'debt') return `O pagamento de ${debts.get(p.cat) ?? ''}`;
  if (!p.root) return NO_CATEGORY;
  return p.sub ? name(p.sub) : `${name(p.root)} (sem subcategoria)`;
}

function describeNode(id: string, debts: Map<string, string>): [string, SankeyNodeKind] {
  const name = (c: string) => (c === 'none' ? NO_CATEGORY : (category(c)?.name ?? NO_CATEGORY));
  if (id.startsWith('income:')) {
    const c = id.slice(7);
    return [c === 'none' ? 'Receitas sem categoria' : name(c), 'income'];
  }
  if (id.startsWith('expense:')) return [name(id.slice(8)), 'expense'];
  if (id.startsWith('sub:')) {
    const c = category(id.slice(4));
    return [c && !c.parent_id ? `${c.name} (sem subcategoria)` : name(id.slice(4)), 'subcategory'];
  }
  if (id.startsWith('debt:')) return [debts.get(id.slice(5)) ?? '', 'subcategory'];
  const fixed: Record<string, [string, SankeyNodeKind]> = {
    debt: ['Pagamento de dívidas', 'debt'],
    investment: ['Investimentos', 'investment'],
    redemption: ['Resgates', 'redemption'],
    deficit: ['Déficit', 'deficit'],
  };
  return fixed[id] ?? ['Sobra', 'savings'];
}

const accountName = (id: string) => ACCOUNTS.find((a) => a.id === id)?.name ?? id;

function flowItems(
  from: string,
  to: string,
  basis: Basis,
  debts: Map<string, string>,
  wants: (id: string) => boolean,
  fx: Fx,
): SankeyItem[] {
  // Um item por lançamento e categoria (partes de uma divisão na mesma categoria juntas).
  interface Row {
    txn: LedgerTxn;
    eff: string;
    category: string | null;
    income: boolean;
    amount: number;
    converted: number | null;
    split: boolean;
    estimated: boolean;
  }
  const rows = new Map<string, Row>();
  for (const part of flowParts({ from, to, basis })) {
    const key = `${part.txn.id}|${part.category_id ?? ''}`;
    let row = rows.get(key);
    if (!row) {
      row = {
        txn: part.txn,
        eff: part.eff,
        category: part.category_id,
        income: flowKind(part.txn) === 'income',
        amount: 0,
        converted: 0,
        split: part.split,
        estimated: false,
      };
      rows.set(key, row);
    }
    row.amount += part.amount;
    const c = fx.convert(part.amount, part.txn.currency, part.txn.date);
    if (!c) row.converted = null;
    else if (row.converted !== null) row.converted += c.amount;
    if (c?.estimated) row.estimated = true;
  }
  const sorted = [...rows.values()].sort(
    (a, b) =>
      a.eff.localeCompare(b.eff) ||
      a.txn.date.localeCompare(b.txn.date) ||
      a.txn.id.localeCompare(b.txn.id) ||
      (a.category ?? '').localeCompare(b.category ?? ''),
  );
  const units = new Map<string, number>();
  const placed = sorted.map((row) => {
    const place = placeOf(row.category, row.income, debts);
    const sign = row.income ? 1 : -1;
    const amount = row.converted === null ? null : sign * row.converted;
    if (amount !== null) units.set(placeKey(place), (units.get(placeKey(place)) ?? 0) + amount);
    return { row, place, sign, amount };
  });
  const out: SankeyItem[] = [];
  for (const { row, place, sign, amount } of placed) {
    const matches = nodesOf(place).filter((n): n is string => n !== null && wants(n));
    if (!matches.length) continue;
    if (matches.length > 1)
      invalid('os nós pedidos se sobrepõem (uma categoria e uma subcategoria dela)');
    const unit = units.get(placeKey(place)) ?? 0;
    const info = category(row.category);
    const foreign = row.txn.currency !== BASE;
    out.push({
      txn_id: row.txn.id,
      node_id: matches[0]!,
      component: null,
      date: row.txn.date,
      due_date: row.eff !== row.txn.date ? row.eff : null,
      label: row.txn.description.trim() || '(sem descrição)',
      account: accountName(row.txn.account_id),
      category_id: info?.id ?? null,
      group: info ? (categoryLabel(info.id).path ?? info.name) : NO_CATEGORY,
      amount: amount ?? 0,
      original_amount: foreign ? sign * row.amount : null,
      original_currency: foreign ? row.txn.currency : null,
      split: row.split,
      counted: amount !== null && unit > 0,
      note:
        amount === null
          ? `Sem câmbio de ${row.txn.currency} para ${BASE} na data: fica fora do total.`
          : unit <= 0
            ? `${unitLabel(place, debts)} somou ${money(unit)} no período (estornos maiores que os lançamentos); o diagrama deixa de fora grupos sem saldo positivo.`
            : null,
      estimated: row.estimated,
    });
  }
  return out;
}

function investmentItems(
  from: string,
  to: string,
  wants: (id: string) => boolean,
  fx: Fx,
): SankeyItem[] {
  const out: SankeyItem[] = [];
  for (const leg of investmentLegs(from, to)) {
    const node = leg.out ? 'investment' : 'redemption';
    if (!wants(node)) continue;
    const t = leg.txn;
    const sign = leg.out ? -1 : 1;
    const c = fx.convert(t.amount, t.currency, t.date);
    const foreign = t.currency !== BASE;
    out.push({
      txn_id: t.id,
      node_id: node,
      component: null,
      date: t.date,
      due_date: null,
      label: t.description.trim() || '(sem descrição)',
      account: accountName(t.account_id),
      category_id: null,
      group: accountName(leg.other ?? t.account_id),
      amount: c ? sign * c.amount : 0,
      original_amount: foreign ? sign * t.amount : null,
      original_currency: foreign ? t.currency : null,
      split: false,
      counted: c !== null,
      note: c ? null : `Sem câmbio de ${t.currency} para ${BASE} na data: fica fora do total.`,
      estimated: c?.estimated ?? false,
    });
  }
  return out;
}

function derivedItems(graph: SankeyGraph, id: string): SankeyItem[] | null {
  if (!graph.values.has(id)) return null;
  const sign = id === 'savings' ? 1 : -1;
  const out: SankeyItem[] = [];
  for (const l of graph.links) {
    let component: string;
    let amount: number;
    let group: string;
    if (l.target === 'hub' && l.source !== id) {
      [component, amount, group] = [l.source, sign * l.value, 'Entradas'];
    } else if (l.source === 'hub' && l.target !== id) {
      [component, amount, group] = [l.target, -sign * l.value, 'Saídas'];
    } else continue;
    const n = graph.nodes.find((x) => x.id === component);
    out.push({
      txn_id: null,
      node_id: id,
      component,
      date: null,
      due_date: null,
      label: n?.name ?? component,
      account: null,
      category_id: n?.category_id ?? null,
      group,
      amount,
      original_amount: null,
      original_currency: null,
      split: false,
      counted: true,
      note: null,
      estimated: false,
    });
  }
  return out;
}

function absentNote(graph: SankeyGraph, id: string): string {
  const other = graph.values.get(id === 'savings' ? 'deficit' : 'savings');
  if (id === 'savings')
    return other !== undefined
      ? `Não houve sobra no período: o que saiu passou do que entrou em ${money(other)}.`
      : 'Não houve sobra no período.';
  return other !== undefined
    ? `Não houve déficit no período: sobraram ${money(other)}.`
    : 'Não houve déficit no período.';
}

function itemsPeriod(
  from: string,
  to: string,
  basis: Basis,
  depth: number,
  debts: Map<string, string>,
  nodeIds: readonly string[],
): [SankeyItemsPeriod, SankeyGraph] {
  const graph = sankeyGraph(from, to, basis, depth, debts);
  const wants = (id: string) => nodeIds.includes(id);
  const fx = new Fx();
  const items: SankeyItem[] = [];
  const notes: string[] = [];
  for (const id of nodeIds.filter((n) => DERIVED.has(n))) {
    const d = derivedItems(graph, id);
    if (d) items.push(...d);
    else notes.push(absentNote(graph, id));
  }
  if (nodeIds.some((n) => !DERIVED.has(n) && n !== 'investment' && n !== 'redemption'))
    items.push(...flowItems(from, to, basis, debts, wants, fx));
  if (wants('investment') || wants('redemption'))
    items.push(...investmentItems(from, to, wants, fx));
  return [
    {
      from,
      to,
      value: nodeIds.reduce((s, id) => s + (graph.values.get(id) ?? 0), 0),
      items_total: items.filter((i) => i.counted).reduce((s, i) => s + i.amount, 0),
      items,
      missing_fx: sortedMissing(fx.missing),
      estimated: fx.estimated || items.some((i) => i.estimated),
      note: notes.length ? notes.join(' ') : null,
    },
    graph,
  ];
}

function itemGroups(current: SankeyItem[], previous: SankeyItem[]): SankeyItemsGroup[] {
  const by = new Map<string, SankeyItemsGroup>();
  const add = (items: SankeyItem[], cur: boolean) => {
    for (const i of items.filter((x) => x.counted)) {
      const key = `${i.group}|${i.category_id ?? ''}`;
      let g = by.get(key);
      if (!g) {
        g = {
          group: i.group,
          category_id: i.category_id,
          current: 0,
          previous: 0,
          delta: 0,
          current_count: 0,
          previous_count: 0,
        };
        by.set(key, g);
      }
      if (cur) {
        g.current += i.amount;
        g.current_count += 1;
      } else {
        g.previous += i.amount;
        g.previous_count += 1;
      }
    }
  };
  add(current, true);
  add(previous, false);
  return [...by.values()]
    .map((g) => ({ ...g, delta: g.current - g.previous }))
    .sort(
      (a, b) =>
        Math.abs(b.delta) - Math.abs(a.delta) ||
        (a.group < b.group ? -1 : a.group > b.group ? 1 : 0),
    );
}

/** `reports.sankey_items`: o que compõe uma linha do fluxo nos dois períodos. */
export function sankeyItems(
  from: string,
  to: string,
  basis: Basis,
  depth: number,
  nodeIds: readonly string[],
): SankeyItems {
  if (from > to)
    invalid(`a data inicial (${formatDate(from)}) é posterior à final (${formatDate(to)})`);
  if (!nodeIds.length) invalid('informe ao menos um nó do fluxo');
  for (const id of nodeIds) if (!validNode(id)) invalid(`nó do fluxo desconhecido: ${id}`);
  const d = Math.min(2, Math.max(1, Math.trunc(depth)));
  const debts = debtCategories();
  const p = sankeyPreviousPeriod(from, to);
  const [current, cur] = itemsPeriod(from, to, basis, d, debts, nodeIds);
  const [previous, prev] = itemsPeriod(p.from, p.to, basis, d, debts, nodeIds);
  const first = nodeIds[0]!;
  const node = [...cur.nodes, ...prev.nodes].find((n) => n.id === first);
  const [name, kind] = node ? [node.name, node.kind] : describeNode(first, debts);
  return {
    node_ids: [...nodeIds],
    name,
    kind,
    basis,
    currency: BASE,
    current,
    previous,
    groups: itemGroups(current.items, previous.items),
  };
}

// ── Calendário ───────────────────────────────────────────────────

export function calendar(from: string, to: string, basis: Basis): CalendarData {
  const agg = aggregate({ from, to, basis }, KEY.day);
  const days: CalendarData['days'] = [];
  for (let d = from; d <= to; d = addDays(d, 1)) {
    const c = agg.cells.get(`${d}|\u0000`) ?? emptyCell();
    days.push({
      date: d,
      expense: c.expense,
      income: c.income,
      count: c.incomeIds.size + c.expenseIds.size,
    });
  }
  const t = total(agg);
  return {
    from,
    to,
    basis,
    days,
    max_expense: Math.max(0, ...days.map((d) => d.expense)),
    total_expense: t.expense,
    total_income: t.income,
    currency: BASE,
    missing_fx: sortedMissing(agg.missing),
    estimated: agg.estimated,
  };
}

// ── Evolução mensal ──────────────────────────────────────────────

export function monthlyEvolution(
  months: number,
  endMonth: string,
  topN: number,
  basis: Basis,
): MonthlyEvolution {
  const n = Math.min(Math.max(months, 1), 240);
  const list = monthsBetween(shiftMonth(endMonth, -(n - 1)), endMonth);
  const ext = monthsBetween(shiftMonth(list[0]!, -2), endMonth);
  const agg = aggregate(
    { from: monthBounds(ext[0]!).from, to: monthBounds(endMonth).to, basis },
    KEY.root,
    KEY.month,
  );
  const pos = new Map(list.map((m, i) => [m, i]));
  const extPos = new Map(ext.map((m, i) => [m, i]));
  const extTotals = ext.map(() => 0);
  const income = list.map(() => 0);
  const byRoot = new Map<string | null, number[]>();
  for (const [k, c] of agg.cells) {
    const [root, month] = decodeKey(k);
    if (!month) continue;
    const ei = extPos.get(month);
    if (ei !== undefined) extTotals[ei]! += c.expense;
    const i = pos.get(month);
    if (i !== undefined) {
      income[i]! += c.income;
      if (c.expense !== 0) {
        const v = byRoot.get(root) ?? list.map(() => 0);
        v[i]! += c.expense;
        byRoot.set(root, v);
      }
    }
  }
  const ranked = [...byRoot.entries()]
    .map(([key, values]) => ({ key, values, sum: values.reduce((s, v) => s + v, 0) }))
    .filter((r) => r.values.some((v) => v !== 0))
    .sort((a, b) => b.sum - a.sum || String(a.key).localeCompare(String(b.key)));
  const series: EvolutionSeries[] = [];
  const others = list.map(() => 0);
  let hasOthers = false;
  ranked.forEach((r, i) => {
    if (i < topN) {
      series.push({
        key: r.key,
        label: r.key ? categoryLabel(r.key).label : NO_CATEGORY,
        color: null,
        values: r.values,
      });
    } else {
      hasOthers = true;
      r.values.forEach((v, j) => (others[j]! += v));
    }
  });
  if (hasOthers) series.push({ key: 'others', label: 'Outras', color: null, values: others });
  const first = firstFlowMonth();
  const firstIdx = first ? monthsBetween(ext[0]!, first).length - 1 : Number.MAX_SAFE_INTEGER;
  const moving: number[] = [];
  for (let i = 2; i < ext.length; i++) {
    const lo = Math.min(Math.max(i - 2, firstIdx), i);
    const slice = extTotals.slice(lo, i + 1);
    moving.push(
      divRound(
        slice.reduce((s, v) => s + v, 0),
        slice.length,
      ),
    );
  }
  return {
    months: list,
    basis,
    series,
    totals: extTotals.slice(2),
    moving_avg_3: moving,
    income,
    currency: BASE,
    missing_fx: sortedMissing(agg.missing),
    estimated: agg.estimated,
  };
}

// ── Comparação ───────────────────────────────────────────────────

export function compare(a: Period, b: Period, basis: Basis, kind: CategoryKind): PeriodComparison {
  const fx = new Fx();
  const aggA = aggregate({ from: a.from, to: a.to, basis, kind }, KEY.root, KEY.none, fx);
  const aggB = aggregate({ from: b.from, to: b.to, basis, kind }, KEY.root, KEY.none, fx);
  const vals = new Map<string | null, [number, number]>();
  for (const [k, c] of aggA.cells) {
    const [key] = decodeKey(k);
    const v = vals.get(key) ?? [0, 0];
    v[0] += cellAmount(c, kind);
    vals.set(key, v);
  }
  for (const [k, c] of aggB.cells) {
    const [key] = decodeKey(k);
    const v = vals.get(key) ?? [0, 0];
    v[1] += cellAmount(c, kind);
    vals.set(key, v);
  }
  const rows: ComparisonRow[] = [...vals.entries()]
    .filter(([, [x, y]]) => x !== 0 || y !== 0)
    .map(([key, [x, y]]) => ({
      key,
      label: key ? categoryLabel(key).label : NO_CATEGORY,
      a: x,
      b: y,
      delta: y - x,
      delta_ratio: ratio(y - x, x),
      color: null,
    }))
    .sort((p, q) => Math.abs(q.delta) - Math.abs(p.delta) || p.label.localeCompare(q.label));
  return {
    a,
    b,
    basis,
    kind,
    rows,
    total_a: cellAmount(total(aggA), kind),
    total_b: cellAmount(total(aggB), kind),
    currency: BASE,
    missing_fx: sortedMissing(fx.missing),
    estimated: fx.estimated,
  };
}

// ── Titulares ────────────────────────────────────────────────────

export function byMember(from: string, to: string, basis: Basis): MemberBreakdown {
  const fx = new Fx();
  const q = { from, to, basis, kind: 'expense' as const };
  const perMember = aggregate(q, KEY.member, KEY.none, fx);
  const grid = aggregate(q, KEY.member, KEY.root, fx);
  const totals = new Map<string | null, [number, number]>();
  for (const [k, c] of perMember.cells) totals.set(decodeKey(k)[0], [c.expense, c.expenseIds.size]);
  const sum = [...totals.values()].reduce((s, [v]) => s + v, 0);
  const keys: (string | null)[] = MEMBERS.filter((m) => !m.archived)
    .sort((x, y) => x.sort_order - y.sort_order)
    .map((m) => m.id);
  const none = totals.get(null);
  if (none && (none[0] !== 0 || none[1] !== 0)) keys.push(null);
  const members: GroupAmount[] = keys.map((k) => {
    const [amount, count] = totals.get(k) ?? [0, 0];
    return {
      key: k,
      label: MEMBERS.find((m) => m.id === k)?.name ?? k ?? NO_MEMBER,
      parent_key: null,
      path: null,
      amount,
      share: share(amount, sum),
      count,
      color: null,
    };
  });
  const index = new Map(keys.map((k, i) => [k, i]));
  const rows = new Map<string | null, number[]>();
  for (const [k, c] of grid.cells) {
    if (c.expense === 0) continue;
    const [m, root] = decodeKey(k);
    const i = index.get(m);
    if (i === undefined) continue;
    const v = rows.get(root) ?? keys.map(() => 0);
    v[i]! += c.expense;
    rows.set(root, v);
  }
  const categories = [...rows.entries()]
    .map(([key, values]) => ({
      sum: values.reduce((s, v) => s + v, 0),
      s: { key, label: key ? categoryLabel(key).label : NO_CATEGORY, color: null, values },
    }))
    .sort((x, y) => y.sum - x.sum || x.s.label.localeCompare(y.s.label))
    .map((x) => x.s);
  return {
    from,
    to,
    members,
    categories,
    basis,
    total: sum,
    currency: BASE,
    missing_fx: sortedMissing(fx.missing),
    estimated: fx.estimated,
  };
}

// ── Próximos vencimentos ─────────────────────────────────────────

export function upcoming(from: string, days: number): UpcomingItem[] {
  const to = addDays(from, Math.min(days, 3660));
  const { txns, statements, today } = getLedger();
  const out: UpcomingItem[] = [];
  for (const t of txns) {
    if (t.status !== 'planned' || t.date < from || t.date > to || t.statement_id) continue;
    if (t.kind === 'card_payment') continue;
    if ((t.kind === 'transfer' || t.kind === 'investment') && t.amount > 0) continue;
    const acc = ACCOUNTS.find((a) => a.id === t.account_id)!;
    out.push({
      ref_id: t.id,
      date: t.date,
      kind: 'planned_txn',
      description: t.description.trim() || (category(t.category_id)?.name ?? '(sem descrição)'),
      amount: t.amount,
      currency: t.currency,
      account_id: t.account_id,
      account_name: acc.name,
    });
  }
  for (const st of statements.values()) {
    if (st.due_date < from || st.due_date > to) continue;
    const view = statementTotals(st.id, today);
    if (view.status === 'paid') continue;
    const remaining = view.total - view.paid;
    if (remaining <= 0) continue;
    const acc = ACCOUNTS.find((a) => a.id === st.account_id)!;
    out.push({
      ref_id: st.id,
      account_id: st.account_id,
      description: `Fatura ${acc.name} (${monthLabel(st.reference_month)})`,
      account_name: acc.name,
      currency: acc.currency,
      date: st.due_date,
      kind: 'card_due',
      amount: -remaining,
    });
  }
  out.sort(
    (a, b) =>
      a.date.localeCompare(b.date) ||
      Number(a.kind === 'planned_txn') - Number(b.kind === 'planned_txn') ||
      a.description.localeCompare(b.description),
  );
  return out;
}

/** Total, previsto e pago de uma fatura (como `cards::statement_view`). */
export function statementTotals(id: string, today: string) {
  const { txns, statements } = getLedger();
  const st = statements.get(id)!;
  let totalAmount = 0;
  let planned = 0;
  let paid = 0;
  let count = 0;
  for (const t of txns) {
    if (t.statement_id !== id) continue;
    if (t.kind === 'card_payment') {
      if (t.status !== 'planned') paid += t.amount;
      continue;
    }
    totalAmount -= t.amount;
    count += 1;
    if (t.status === 'planned') planned -= t.amount;
  }
  // Como `cards::compute_status` no núcleo: fechada com total ≤ 0 já está paga.
  const closed = st.closing_date < today;
  const status =
    paid > 0 || (closed && totalAmount <= 0)
      ? totalAmount - paid <= 0
        ? 'paid'
        : 'partial'
      : closed
        ? 'closed'
        : 'open';
  return {
    total: totalAmount,
    planned,
    paid,
    remaining: totalAmount - paid,
    count,
    status,
  } as const;
}

export type { Cell };
