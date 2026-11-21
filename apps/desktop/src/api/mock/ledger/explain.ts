/**
 * "Medido, nunca inventado" no mock: fórmula e fontes de cada medida, com os mesmos
 * textos de `reports/explain.rs`. A soma das fontes é igual ao valor (sem truncar).
 */
import type { Basis } from '@/api/generated/Basis';
import type { CategoryKind } from '@/api/generated/CategoryKind';
import type { Dashboard } from '@/api/generated/Dashboard';
import type { ExplainArgs } from '@/api/generated/ExplainArgs';
import type { Explanation } from '@/api/generated/Explanation';
import type { ValueSource } from '@/api/generated/ValueSource';
import { formatMoney } from '@/lib/format';
import { monthBounds, shiftMonth } from '@/lib/months';

import { budgetGet } from './budgetState';
import {
  aggregate,
  basisLabel,
  brDate,
  effDate,
  flowParts,
  Fx,
  KEY,
  monthLabel,
  ratio,
  total,
  type FlowQuery,
} from './engine';
import { ACCOUNTS, BASE, category, MEMBERS, withDescendants } from './fixture';
import { getLedger } from './ledger';
import { investmentsAt } from './portfolio';
import { physicalNetWorthItems } from '../physical';
import { monthFlow, netWorth, spending, upcoming } from './reports';
import { MockError } from './budgetState';

const text = (v: unknown, fallback = ''): string => (typeof v === 'string' ? v : fallback);
const money = (v: number, c = BASE) => formatMoney(v, c).replace('−', '-');
const plural = (n: number, one: string, many: string) => (n === 1 ? `1 ${one}` : `${n} ${many}`);
const percent = (r: number) => `${(r * 100).toFixed(1).replace('.', ',')}%`;

function fxSources(fx: Fx): ValueSource[] {
  return [...fx.used.values()].map((u) => ({
    kind: 'fx_rate',
    id: null,
    label: `câmbio: 1 ${u.currency} = ${u.rate.replace('.', ',')} ${BASE}${u.estimated ? ' (estimado)' : ''}`,
    date: u.date,
    amount: 0,
    currency: BASE,
    source: 'bcb_ptax_venda',
    estimated: u.estimated,
  }));
}

function txnSources(q: FlowQuery, fx: Fx, limit: number, negate: boolean) {
  const byTxn = new Map<string, { parts: number; amount: number; eff: string }>();
  for (const p of flowParts(q)) {
    const cur = byTxn.get(p.txn.id) ?? { parts: 0, amount: 0, eff: p.eff };
    cur.parts += p.split ? 1 : 0;
    cur.amount += p.amount;
    byTxn.set(p.txn.id, cur);
  }
  const txns = new Map(getLedger().txns.map((t) => [t.id, t]));
  const rows = [...byTxn.entries()].sort(
    (a, b) => b[1].eff.localeCompare(a[1].eff) || b[0].localeCompare(a[0]),
  );
  const out: ValueSource[] = [];
  for (const [id, r] of rows.slice(0, limit)) {
    const t = txns.get(id)!;
    const signed = negate ? -r.amount : r.amount;
    let label = t.description.trim() || '(sem descrição)';
    if (r.parts === 1) label += ' (parte da divisão)';
    else if (r.parts > 1) label += ` (${r.parts} partes da divisão)`;
    if (r.eff !== t.date)
      label += ` (compra em ${brDate(t.date)}, fatura com vencimento em ${brDate(r.eff)})`;
    const c = fx.convert(signed, t.currency, t.date);
    if (c && t.currency !== BASE)
      label += `: ${money(signed, t.currency)} × ${c.rate.replace('.', ',')} = ${money(c.amount)}`;
    if (!c) label += `: ${money(signed, t.currency)} sem câmbio, fora do total`;
    out.push({
      kind: 'txn',
      id,
      label,
      date: r.eff,
      amount: c?.amount ?? 0,
      currency: BASE,
      source: t.currency !== BASE ? (c?.source ?? null) : null,
      estimated: c?.estimated ?? false,
    });
  }
  return { sources: out, truncated: rows.length > limit };
}

const missingNote = (m: string[]) =>
  m.length ? `; sem câmbio para ${m.join(', ')} (fora do total)` : '';

function flow(
  kind: CategoryKind,
  p: Record<string, unknown>,
  limit: number,
  defaultBasis: Basis,
): Explanation {
  const from = text(p.from);
  const to = text(p.to);
  if (!from || !to) throw new MockError('invalid', 'argumentos inválidos: missing field `from`');
  const basis = (p.basis as Basis | undefined) ?? defaultBasis;
  const catId = (p.category_id as string | undefined) ?? null;
  const memberId = (p.member_id as string | undefined) ?? null;
  const q: FlowQuery = {
    from,
    to,
    basis,
    kind,
    category_ids: catId ? withDescendants([catId]) : null,
    member_ids: memberId ? [memberId] : null,
  };
  const fx = new Fx();
  const t = total(aggregate(q, KEY.none, KEY.none, fx));
  const value = kind === 'income' ? t.income : t.expense;
  const count = kind === 'income' ? t.incomeIds.size : t.expenseIds.size;
  const { sources, truncated } = txnSources(q, fx, limit, kind === 'expense');
  sources.push(...fxSources(fx));
  let filters = '';
  if (catId) filters += ` em ${category(catId)?.path ?? catId} e subcategorias`;
  if (memberId) filters += ` de ${MEMBERS.find((m) => m.id === memberId)?.name ?? memberId}`;
  const what =
    kind === 'expense'
      ? plural(count, 'lançamento de despesa efetivado', 'lançamentos de despesa efetivados')
      : plural(count, 'lançamento de receita efetivado', 'lançamentos de receita efetivados');
  const rule =
    kind === 'expense'
      ? 'despesa = −soma dos valores (estornos reduzem o total)'
      : 'receita = soma dos valores';
  const converted = fx.used.size
    ? `; valores em outra moeda convertidos para ${BASE} pelo câmbio da data do lançamento`
    : '';
  const missing = [...t.missing].sort();
  return {
    metric: kind === 'expense' ? 'spending' : 'income',
    value,
    currency: BASE,
    formula: `${what} de ${brDate(from)} a ${brDate(to)} (${basisLabel(basis)})${filters}: ${rule} = ${money(value)}; transferências, pagamentos de fatura e aportes não entram; lançamentos divididos contam pelas partes${converted}${missingNote(missing)}`,
    sources,
    truncated,
    estimated: fx.estimated,
    ratio: null,
    count,
    missing_fx: missing,
  };
}

export function explain(a: ExplainArgs, defaultBasis: Basis): Explanation {
  const limit = Math.min(a.limit ?? 200, 10_000);
  const p: Record<string, unknown> = a.params ?? {};
  const today = getLedger().today;
  switch (a.metric) {
    case 'spending':
      return flow('expense', p, limit, defaultBasis);
    case 'income':
      return flow('income', p, limit, defaultBasis);
    case 'savings_rate': {
      const month = text(p.month, today.slice(0, 7)).slice(0, 7);
      const basis = (p.basis as Basis | undefined) ?? defaultBasis;
      const { from, to } = monthBounds(month);
      const q: FlowQuery = { from, to, basis };
      const fx = new Fx();
      const t = total(aggregate(q, KEY.none, KEY.none, fx));
      const value = t.income - t.expense;
      const rate = ratio(value, t.income);
      const count = t.incomeIds.size + t.expenseIds.size;
      const { sources, truncated } = txnSources(q, fx, limit, false);
      sources.push(...fxSources(fx));
      const head = `taxa de poupança de ${monthLabel(month)} (${basisLabel(basis)}) = (receitas de ${money(t.income)} − despesas de ${money(t.expense)}) ÷ receitas de ${money(t.income)}`;
      return {
        metric: 'savings_rate',
        value,
        currency: BASE,
        formula:
          rate != null
            ? `${head} = ${percent(rate)}; valor = receitas − despesas = ${money(value)}, de ${plural(count, 'lançamento efetivado', 'lançamentos efetivados')}`
            : `sem receitas em ${monthLabel(month)}, a taxa de poupança não se aplica; receitas − despesas = ${money(value)}, de ${plural(count, 'lançamento efetivado', 'lançamentos efetivados')}`,
        sources,
        truncated,
        estimated: fx.estimated,
        ratio: rate,
        count,
        missing_fx: [],
      };
    }
    case 'net_worth': {
      const asOf = text(p.as_of, today);
      const nw = netWorth(asOf);
      const fx = new Fx();
      const sources: ValueSource[] = nw.accounts.map((acc) => {
        let label = acc.name;
        if (acc.currency !== BASE) {
          label += `: ${money(acc.balance, acc.currency)}`;
          const c = fx.convert(acc.balance, acc.currency, asOf);
          label += c
            ? ` × ${c.rate.replace('.', ',')} = ${money(c.amount)}`
            : ' (sem câmbio, fora do total)';
        }
        return {
          kind: 'account',
          id: acc.account_id,
          label,
          date: asOf,
          amount: acc.balance_base ?? 0,
          currency: BASE,
          source: acc.currency !== BASE && acc.balance_base != null ? 'bcb_ptax_venda' : null,
          estimated: acc.fx_estimated,
        };
      });
      const inv = investmentsAt(asOf);
      if (inv) {
        sources.push({
          kind: 'account',
          id: 'acc-xp',
          label: 'carteira de investimentos: valor de mercado das posições',
          date: asOf,
          amount: inv,
          currency: BASE,
          source: 'portfolio',
          estimated: false,
        });
      }
      for (const item of physicalNetWorthItems(asOf))
        sources.push({
          kind: 'physical',
          id: item.id,
          label: `${item.name}: ${item.formula}`,
          date: asOf,
          amount: item.value,
          currency: BASE,
          source: item.estimated ? 'estimativa' : 'avaliação',
          estimated: item.estimated,
        });
      if (nw.debts)
        sources.push({
          kind: 'loan',
          id: null,
          label: 'Empréstimos e financiamentos (saldo devedor pelo cronograma)',
          date: asOf,
          amount: -nw.debts,
          currency: BASE,
          source: null,
          estimated: false,
        });
      sources.push(...fxSources(fx));
      return {
        metric: 'net_worth',
        value: nw.net,
        currency: BASE,
        formula: `ativos de ${money(nw.assets)} − passivos de ${money(nw.liabilities)} = ${money(nw.net)}, somando o saldo de ${plural(nw.accounts.length, 'conta', 'contas')} em ${brDate(asOf)} (saldo inicial + lançamentos efetivados até a data; saldos negativos, como a dívida dos cartões, são passivos)${inv ? ` e o valor de mercado da carteira (${money(inv)})` : ''}${nw.physical ? `, mais os bens físicos (${money(nw.physical)}, avaliação ou estimativa)` : ''}${nw.debts ? `, menos o saldo devedor das dívidas (${money(nw.debts)})` : ''}${missingNote(nw.missing_fx)}`,
        sources,
        truncated: false,
        estimated: nw.estimated,
        ratio: null,
        count: nw.accounts.length,
        missing_fx: nw.missing_fx,
      };
    }
    case 'account_balance': {
      const id = text(p.account_id);
      const acc = ACCOUNTS.find((x) => x.id === id);
      if (!acc) throw new MockError('not_found', `não encontrado: conta ${id}`);
      const asOf = text(p.as_of, today);
      const { start, txns } = getLedger();
      const list = txns.filter(
        (t) => t.account_id === id && t.status !== 'planned' && t.date <= asOf,
      );
      const sum = list.reduce((s, t) => s + t.amount, 0);
      const openingCounts = start <= asOf;
      const value = sum + (openingCounts ? acc.opening_balance : 0);
      const sources: ValueSource[] = [];
      if (openingCounts) {
        sources.push({
          kind: 'opening',
          id,
          label: `saldo inicial de ${acc.name}`,
          date: start,
          amount: acc.opening_balance,
          currency: acc.currency,
          source: null,
          estimated: false,
        });
      }
      for (const t of [...list].reverse().slice(0, limit)) {
        sources.push({
          kind: 'txn',
          id: t.id,
          label: t.description,
          date: t.date,
          amount: t.amount,
          currency: acc.currency,
          source: null,
          estimated: false,
        });
      }
      return {
        metric: 'account_balance',
        value,
        currency: acc.currency,
        formula: `saldo inicial de ${money(acc.opening_balance, acc.currency)} + ${plural(list.length, 'lançamento efetivado', 'lançamentos efetivados')} até ${brDate(asOf)} (${money(sum, acc.currency)}) = ${money(value, acc.currency)}`,
        sources,
        truncated: list.length > limit,
        estimated: false,
        ratio: null,
        count: list.length,
        missing_fx: [],
      };
    }
    default:
      throw new MockError(
        'invalid',
        `argumentos inválidos: métrica desconhecida ${String(a.metric)}`,
      );
  }
}

// ── Painel ───────────────────────────────────────────────────────

export function dashboard(month: string, basis: Basis): Dashboard {
  const today = getLedger().today;
  const { from, to } = monthBounds(month);
  const asOf = to < today ? to : today;
  const history = monthFlow(shiftMonth(month, -11), month, basis);
  const flowNow = history[history.length - 1]!;
  const inc = history.reduce((s, f) => s + f.income, 0);
  const exp = history.reduce((s, f) => s + f.expense, 0);
  const top = spending({ from, to, basis, kind: 'expense', group_by: 'parent_category' }, basis);
  const budget = budgetGet(month, basis);
  const hasBudget = budget.lines.some((l) => l.has_own_line);
  const cats = aggregate({ from, to, basis }, KEY.category);
  const unc = cats.cells.get('\u0000|\u0000');
  return {
    month,
    basis,
    net_worth: netWorth(asOf),
    flow: flowNow,
    flow_history: history,
    savings_rate_12m: ratio(inc - exp, inc),
    top_categories: top.groups.filter((g) => g.amount > 0).slice(0, 5),
    upcoming: upcoming(today, 30),
    budget_planned: hasBudget ? budget.totals.planned_expense : null,
    budget_actual: budget.totals.actual_expense,
    uncategorized_count: unc ? unc.incomeIds.size + unc.expenseIds.size : 0,
    currency: BASE,
  };
}

export { effDate };
