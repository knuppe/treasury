/**
 * Mock de `reports.*`, calculado sobre o livro-caixa fictício (`./ledger/`), e reservas
 * para os comandos de outros módulos que as telas de relatórios leem (categorias, contas,
 * titulares, recorrências, faturas futuras e resumo da carteira). As reservas só entram
 * se o mock do módulo dono não estiver registrado.
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { AppSettings } from '@/api/generated/AppSettings';
import type { Basis } from '@/api/generated/Basis';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { CategoryNode } from '@/api/generated/CategoryNode';
import type { ExplainArgs } from '@/api/generated/ExplainArgs';
import type { MonthAmount } from '@/api/generated/MonthAmount';
import type { Period } from '@/api/generated/Period';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { SpendingArgs } from '@/api/generated/SpendingArgs';
import type { StatementView } from '@/api/generated/StatementView';
import { onMockReset } from '@/api/mockRegistry';
import { formatMoney } from '@/lib/format';
import { clampDay, monthBounds, monthsBetween, parseIso, shiftMonth } from '@/lib/months';

import { resetBudgets } from './ledger/budgetState';
import {
  basisArg,
  defaultBasis,
  monthArg,
  optNum,
  optStr,
  register,
  registerFallbacks,
  str,
} from './ledger/common';
import { baseCurrency } from './ledger/domain';
import { balancesAt, convert } from './ledger/engine';
import { optBool } from './ledger/errors';
import { dashboard, explain } from './ledger/explain';
import { inflation } from './ledger/inflation';
import { invoke } from './ledger/investImport';
import { ACCOUNTS, CATEGORIES, MEMBERS, toAccount } from './ledger/fixture';
import { resetGoals } from './ledger/goalsState';
import { getLedger, resetLedger } from './ledger/ledger';
import { portfolioSummary } from './ledger/portfolio';
import { spendGroups } from './ledger/spendGroups';
import {
  byMember,
  calendar,
  compare,
  monthFlow,
  monthlyEvolution,
  netWorth,
  netWorthHistory,
  sankey,
  sankeyItems,
  spending,
  statementTotals,
  upcoming,
} from './ledger/reports';

/** Base do diagrama de fluxo: a pedida ou o ajuste `flow_basis` (padrão caixa), como no núcleo. */
function flowBasisArg(a: Record<string, unknown>): Basis {
  if (a.basis === 'cash' || a.basis === 'accrual') return a.basis;
  return invoke<AppSettings>('settings.get').flow_basis ?? 'cash';
}

onMockReset(() => {
  resetLedger();
  resetBudgets();
  resetGoals();
});

const today = () => getLedger().today;
const currentMonth = () => today().slice(0, 7);

function period(v: unknown, name: string): Period {
  const p = v as Partial<Period> | undefined;
  if (!p?.from || !p.to) return str({}, `${name}.from`) as never;
  return { from: p.from, to: p.to };
}

register({
  'reports.dashboard': (a) => {
    const month = monthArg(a, 'month', currentMonth());
    return dashboard(month, defaultBasis());
  },
  'reports.net_worth': (a) => netWorth(optStr(a, 'as_of') ?? today()),
  'reports.net_worth_history': (a) =>
    netWorthHistory(optNum(a, 'months') ?? 12, monthArg(a, 'end_month', currentMonth())),
  'reports.month_flow': (a) =>
    monthFlow(monthArg(a, 'from_month'), monthArg(a, 'to_month'), basisArg(a)),
  'reports.spending': (a) =>
    spending(
      { ...(a as unknown as SpendingArgs), from: str(a, 'from'), to: str(a, 'to') },
      defaultBasis(),
    ),
  'reports.sankey': (a) =>
    sankey(
      str(a, 'from'),
      str(a, 'to'),
      flowBasisArg(a),
      optNum(a, 'depth') ?? 2,
      optBool(a, 'compare') ?? true,
    ),
  'reports.sankey_items': (a) =>
    sankeyItems(
      str(a, 'from'),
      str(a, 'to'),
      flowBasisArg(a),
      optNum(a, 'depth') ?? 2,
      Array.isArray(a.node_ids) ? (a.node_ids as string[]) : [],
    ),
  'reports.calendar': (a) => calendar(str(a, 'from'), str(a, 'to'), basisArg(a)),
  'reports.monthly_evolution': (a) =>
    monthlyEvolution(
      optNum(a, 'months') ?? 12,
      monthArg(a, 'end_month', currentMonth()),
      optNum(a, 'top_n') ?? 6,
      basisArg(a),
    ),
  'reports.compare': (a) =>
    compare(
      period(a.a, 'a'),
      period(a.b, 'b'),
      basisArg(a),
      (a.kind as CategoryKind | undefined) ?? 'expense',
    ),
  'reports.by_member': (a) => byMember(str(a, 'from'), str(a, 'to'), basisArg(a)),
  'reports.upcoming': (a) => upcoming(optStr(a, 'from') ?? today(), optNum(a, 'days') ?? 30),
  'reports.explain': (a) => explain(a as unknown as ExplainArgs, defaultBasis()),
  'reports.insights': () => mockInsights(),
  'reports.inflation': (a) =>
    inflation(
      optNum(a, 'months') ?? 12,
      monthArg(a, 'end_month', currentMonth()),
      today(),
      basisArg(a),
    ),
  'reports.pdf': (a) => ({ path: str(a, 'path'), pages: 2, bytes: 48_213 }),
  'reports.spend_groups': (a) =>
    spendGroups(
      monthArg(a, 'from'),
      monthArg(a, 'to'),
      basisArg(a),
      optStr(a, 'member_id') ?? null,
    ),
});

// ── Reservas de outros módulos ───────────────────────────────────

function accountViews(): AccountView[] {
  const { start, txns } = getLedger();
  const t = today();
  const endOfMonth = monthBounds(currentMonth()).to;
  const balances = new Map(balancesAt(t).map((b) => [b.id, b.balance]));
  return ACCOUNTS.map((a, i) => {
    const balance = balances.get(a.id) ?? 0;
    const projected =
      balance +
      txns
        .filter((x) => x.account_id === a.id && x.status === 'planned' && x.date <= endOfMonth)
        .reduce((s, x) => s + x.amount, 0);
    const conv = convert(balance, a.currency, t);
    const last = [...txns].reverse().find((x) => x.account_id === a.id && x.status !== 'planned');
    return {
      ...toAccount(a, start, i),
      balance,
      projected_balance: projected,
      reconciled_balance: 0,
      balance_base: conv?.amount ?? null,
      fx_estimated: conv?.estimated ?? false,
      last_txn_date: last?.date ?? null,
      last_import_at: null,
      staleness_days: null,
      balance_status: 'fresh',
      missed_month: null,
      available_credit: a.card ? a.card.credit_limit + balance : null,
      overdraft_status: null,
    };
  });
}

function tree(kind?: CategoryKind): CategoryNode[] {
  const nodes = CATEGORIES.filter((c) => !kind || c.kind === kind).map((c): CategoryNode => ({
    ...c,
    children: [],
  }));
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const roots: CategoryNode[] = [];
  for (const n of nodes) {
    const parent = n.parent_id ? byId.get(n.parent_id) : undefined;
    if (parent) parent.children.push(n);
    else roots.push(n);
  }
  return roots;
}

function recurrences(): Recurrence[] {
  const t = today();
  const { start } = getLedger();
  const months = monthsBetween(currentMonth(), shiftMonth(currentMonth(), 14));
  return getLedger().recurrences.map((r) => {
    const next = months
      .filter((m) => !r.months || r.months.includes(parseIso(m).m))
      .map((m) => clampDay(parseIso(m).y, parseIso(m).m, r.day))
      .filter((d) => d >= t)
      .slice(0, 3);
    const amount = r.amountAt(currentMonth(), 99);
    const acc = ACCOUNTS.find((a) => a.id === r.account_id)!;
    const rrule = r.months
      ? `FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=${r.day};BYMONTH=${r.months.join(',')}`
      : `FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=${r.day}`;
    return {
      id: r.id,
      description: r.description,
      account_id: r.account_id,
      account_name: acc.name,
      amount,
      currency: acc.currency,
      kind: r.kind,
      category_id: r.category_id,
      member_id: r.member_id,
      to_account_id: r.to_account_id ?? null,
      rrule,
      spec: r.spec,
      human: r.months
        ? `todo mês no dia ${r.day}, de fevereiro a novembro`
        : `todo mês no dia ${r.day}`,
      start_date: start,
      end_date: null,
      active: true,
      next_dates: next,
      monthly_equivalent: r.months ? Math.round((amount * r.months.length) / 12) : amount,
    };
  });
}

function statementViews(accountId?: string, fromMonth?: string, toMonth?: string): StatementView[] {
  const t = today();
  return [...getLedger().statements.values()]
    .filter((s) => !accountId || s.account_id === accountId)
    .filter(
      (s) =>
        (!fromMonth || s.reference_month >= fromMonth) &&
        (!toMonth || s.reference_month <= toMonth),
    )
    .sort((a, b) => a.due_date.localeCompare(b.due_date))
    .map((s) => {
      const v = statementTotals(s.id, t);
      const acc = ACCOUNTS.find((a) => a.id === s.account_id)!;
      return {
        id: s.id,
        account_id: s.account_id,
        account_name: acc.name,
        reference_month: s.reference_month,
        period_start: s.period_start,
        closing_date: s.closing_date,
        due_date: s.due_date,
        status: v.status,
        total: v.total,
        planned: v.planned,
        paid: v.paid,
        remaining: v.remaining,
        informed_total: null,
        txn_count: v.count,
        currency: acc.currency,
        dates_source: 'calc',
        informed_minimum: null,
        informed_limit: null,
        informed_previous_balance: null,
      };
    });
}

registerFallbacks({
  'categories.list': (a) =>
    CATEGORIES.filter((c) => !a.kind || c.kind === a.kind).filter(
      (c) => a.include_archived || !c.archived,
    ),
  'categories.tree': (a) => tree(a.kind as CategoryKind | undefined),
  'accounts.list': () => accountViews(),
  'household.list_members': () => MEMBERS,
  'recurrences.list': (a) => {
    const list = recurrences();
    return list
      .filter((r) => !a.account_id || r.account_id === a.account_id)
      .filter((r) => typeof a.active !== 'boolean' || r.active === a.active);
  },
  'cards.list_statements': (a) =>
    statementViews(optStr(a, 'account_id'), optStr(a, 'from_month'), optStr(a, 'to_month')),
  'cards.future_commitments': (a): MonthAmount[] => {
    const n = Math.min(Math.max(optNum(a, 'months') ?? 12, 1), 120);
    const keys = monthsBetween(currentMonth(), shiftMonth(currentMonth(), n - 1));
    const out = new Map(keys.map((k) => [k, 0]));
    for (const v of statementViews(optStr(a, 'account_id'), keys[0], keys[keys.length - 1])) {
      if (v.remaining <= 0) continue;
      out.set(v.reference_month, (out.get(v.reference_month) ?? 0) + v.remaining);
    }
    return keys.map((month) => ({ month, amount: out.get(month) ?? 0 }));
  },
  'portfolio.summary': (a) => portfolioSummary(optStr(a, 'as_of') ?? today()),
});

/** Percepções de exemplo (o núcleo calcula com os dados do cofre). */
function mockInsights() {
  const groups = [
    {
      id: 'ipca_g1',
      name: 'Alimentação e bebidas',
      weight: 0.42,
      change: 0.071,
      contribution: 0.0298,
      spent: 1_200_000,
    },
    {
      id: 'ipca_g2',
      name: 'Habitação',
      weight: 0.38,
      change: 0.052,
      contribution: 0.0198,
      spent: 1_080_000,
    },
    {
      id: 'ipca_g5',
      name: 'Transportes',
      weight: 0.2,
      change: 0.031,
      contribution: 0.0062,
      spent: 570_000,
    },
  ];
  const inflation = {
    from_month: '2025-10',
    to_month: '2026-09',
    personal: 0.0558,
    ipca: 0.0462,
    groups,
    coverage: 0.86,
  };
  // Valores na moeda base da família, como o núcleo escreve as percepções.
  const currency = baseCurrency();
  const m = (cents: number) => formatMoney(cents, currency);
  return {
    currency,
    personal_inflation: inflation,
    items: [
      {
        id: 'inflation:2026-09',
        kind: 'personal_inflation',
        tone: 'warning',
        title: 'Sua inflação em 12 meses: 5,6% (IPCA: 4,6%)',
        body: 'Com o peso que cada grupo tem nos seus gastos, os preços do que você compra subiram 5,6% de out/25 a set/26. O que mais pesou: Alimentação e bebidas (42,0% dos gastos, subiu 7,1%).',
        action:
          'Reajustes de renda e de orçamento menores que essa inflação significam perder poder de compra.',
        screen: 'orcamento',
        amount: null,
        data: inflation,
      },
      {
        id: 'fees:2026-09',
        kind: 'fees_and_interest',
        tone: 'warning',
        title: `${m(120_000)} em tarifas e juros em 12 meses`,
        body: `De out/25 a set/26, ${m(48_000)} saíram em tarifas bancárias (12 cobranças) e ${m(72_000)} em juros, IOF e encargos (12 cobranças).`,
        action:
          'Vale saber: os bancos são obrigados a oferecer um pacote de serviços essenciais sem tarifa (Resolução CMN 3.919).',
        screen: 'lancamentos',
        amount: 120_000,
        data: {},
      },
      {
        id: 'drop:2026-09:Lazer',
        kind: 'category_drop',
        tone: 'good',
        title: 'Lazer abaixo do normal em set/26',
        body: `Em set/26, Lazer somou ${m(15_000)}, ${m(25_000)} a menos que o normal (${m(40_000)} por mês).`,
        action: 'Se a economia continuar, dá para guardar a diferença numa meta.',
        screen: 'orcamento',
        amount: 25_000,
        data: {},
      },
    ],
  };
}
