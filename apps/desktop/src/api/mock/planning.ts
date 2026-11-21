/**
 * Mock do planejamento, com as regras do núcleo: previsão de saldo (`forecast/mod.rs`; aqui
 * só pelos lançamentos previstos, sem expandir recorrências nem faturas), quanto posso gastar
 * hoje, simulador de quitação (`debt/payoff.rs`), assinaturas detectadas
 * (`subscriptions/mod.rs`) e preferências de notificação (`notify/mod.rs`).
 */
import type { AccountForecast } from '../generated/AccountForecast';
import type { BalanceForecast } from '../generated/BalanceForecast';
import type { CustomDebt } from '../generated/CustomDebt';
import type { DailyAllowance } from '../generated/DailyAllowance';
import type { ForecastEvent } from '../generated/ForecastEvent';
import type { ForecastPoint } from '../generated/ForecastPoint';
import type { Goal } from '../generated/Goal';
import type { LoanView } from '../generated/LoanView';
import type { NotifyPrefs } from '../generated/NotifyPrefs';
import type { PayoffDebtResult } from '../generated/PayoffDebtResult';
import type { PayoffPlan } from '../generated/PayoffPlan';
import type { PayoffResult } from '../generated/PayoffResult';
import type { PayoffStrategy } from '../generated/PayoffStrategy';
import type { NetWorth } from '../generated/NetWorth';
import type { SavedScenario } from '../generated/SavedScenario';
import type { ScenarioEvent } from '../generated/ScenarioEvent';
import type { ScenarioPath } from '../generated/ScenarioPath';
import type { ScenarioResult } from '../generated/ScenarioResult';
import type { Subscription } from '../generated/Subscription';
import type { SubscriptionReport } from '../generated/SubscriptionReport';
import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import {
  addDays,
  addMonths,
  brDate,
  daysInMonth,
  diffDays,
  monthOf,
  parseYmd,
  today,
} from './ledger/dates';
import { balanceAt, baseCurrency } from './ledger/domain';
import { flowParts } from './ledger/engine';
import { invalid, notFound } from './ledger/errors';
import { invoke } from './ledger/investImport';
import { db } from './ledger/store';

const DAILY = new Set([
  'checking',
  'savings',
  'cash',
  'credit_card',
  'benefit',
  'other',
  'foreign',
]);

const DEFAULT_PREFS: NotifyPrefs = {
  system: true,
  show_amounts: false,
  bills: true,
  bills_days: 3,
  dividends: true,
  price_targets: true,
  negative_balance: true,
  subscriptions: true,
  budget: true,
  scores: true,
  agent: true,
  stale_accounts: true,
};
let prefs: NotifyPrefs = { ...DEFAULT_PREFS };
let dismissed: string[] = [];
let scenarios: SavedScenario[] = [];
onMockReset(() => {
  prefs = { ...DEFAULT_PREFS };
  dismissed = [];
  scenarios = [];
});

const brl = (c: number) =>
  (c / 100)
    .toLocaleString('pt-BR', { style: 'currency', currency: baseCurrency() })
    .replace(/\u00a0/g, ' ');

// ── Previsão de saldo ─────────────────────────────────────────────

function forecast(a: Record<string, unknown>): BalanceForecast {
  const months = Math.min(Math.max(Number(a.months ?? 3), 1), 24);
  const ids = a.account_ids as string[] | undefined;
  const from = today();
  const to = addMonths(from, months);
  const all = [...db.t.account.values()];
  if (ids) for (const id of ids) if (!all.some((x) => x.id === id)) notFound(`conta ${id}`);
  const chosen = all.filter((x) => (ids ? ids.includes(x.id) : !x.archived && DAILY.has(x.kind)));
  const accounts: AccountForecast[] = chosen.map((acc) => {
    const start = balanceAt(acc.id, from, false);
    const events: ForecastEvent[] = [...db.t.txn.values()]
      .filter(
        (t) => t.account_id === acc.id && t.status === 'planned' && t.date > from && t.date <= to,
      )
      .sort((x, y) => x.date.localeCompare(y.date))
      .map((t) => ({
        date: t.date,
        description: t.description,
        amount: t.amount,
        source: 'planned',
      }));
    const points: ForecastPoint[] = [{ date: from, balance: start }];
    let bal = start;
    let min = start;
    let minDate = from;
    let firstNegative: string | null = null;
    for (const e of events) {
      bal += e.amount;
      const last = points.at(-1)!;
      if (last.date === e.date) last.balance = bal;
      else points.push({ date: e.date, balance: bal });
      if (bal < min) {
        min = bal;
        minDate = e.date;
      }
    }
    if (acc.kind !== 'credit_card') firstNegative = points.find((p) => p.balance < 0)?.date ?? null;
    return {
      account_id: acc.id,
      name: acc.name,
      kind: acc.kind,
      currency: acc.currency,
      start,
      end: bal,
      min,
      min_date: minDate,
      first_negative: firstNegative,
      points,
      events: events.slice(0, 300),
    };
  });
  // Como no núcleo: o total soma as contas na moeda base da família.
  const base = baseCurrency();
  const baseAccounts = accounts.filter((x) => x.currency === base);
  const dates = [...new Set(baseAccounts.flatMap((x) => x.points.map((p) => p.date)))].sort();
  const at = (x: AccountForecast, d: string) =>
    x.points.filter((p) => p.date <= d).at(-1)?.balance ?? x.start;
  const totalPoints = dates.map((d) => ({
    date: d,
    balance: baseAccounts.reduce((s, x) => s + at(x, d), 0),
  }));
  const totalStart = totalPoints[0]?.balance ?? 0;
  const lowest = totalPoints.reduce(
    (m, p) => (p.balance < m.balance ? p : m),
    totalPoints[0] ?? { date: from, balance: 0 },
  );
  return {
    from,
    to,
    months,
    currency: base,
    accounts,
    total_start: totalStart,
    total_end: totalPoints.at(-1)?.balance ?? 0,
    total_min: lowest.balance,
    total_min_date: lowest.date,
    total_first_negative: totalPoints.find((p) => p.balance < 0)?.date ?? null,
    total_points: totalPoints,
  };
}

function dailyAllowance(): DailyAllowance {
  const t = today();
  const month = monthOf(t);
  const { y, m, d } = parseYmd(t);
  const first = `${month}-01`;
  const last = `${month}-${String(daysInMonth(y, m)).padStart(2, '0')}`;
  const daysLeft = Math.max(diffDays(last, t) + 1, 1);
  const sum = (parts: ReturnType<typeof flowParts>, kind: 'income' | 'expense') =>
    parts
      .filter((p) => (kind === 'income' ? p.amount > 0 : p.amount < 0))
      .reduce((s, p) => s + Math.abs(p.amount), 0);
  const realized = flowParts({ from: first, to: t, basis: 'accrual' });
  const todayParts = flowParts({ from: t, to: t, basis: 'accrual' });
  const tomorrow = addDays(t, 1);
  const planned =
    tomorrow <= last
      ? flowParts({ from: tomorrow, to: last, basis: 'accrual', status: 'planned' })
      : [];
  const goals = invoke<Goal[]>('goals.list').reduce((s, g) => s + (g.monthly_contribution ?? 0), 0);
  const income = sum(realized, 'income') + sum(planned, 'income');
  const spent = sum(realized, 'expense');
  const fixed = sum(planned, 'expense');
  // Já aportado no mês (saída de conta do dia a dia para investimento).
  const invested = [...db.t.txn.values()]
    .filter(
      (x) =>
        x.kind === 'investment' &&
        x.amount < 0 &&
        x.status !== 'planned' &&
        x.date >= first &&
        x.date <= t &&
        db.t.account.get(x.account_id)?.kind !== 'brokerage',
    )
    .reduce((s, x) => s - x.amount, 0);
  const savings = Math.max(goals - invested, 0);
  // Como no núcleo: o dia de hoje divide o que sobrava no começo dele (os gastos de hoje não
  // saem duas vezes).
  const spentToday = sum(todayParts, 'expense');
  const spentBefore = spent - spentToday;
  const availableStart = income - spentBefore - fixed - savings;
  const available = availableStart - spentToday;
  const perDay = Math.max(Math.trunc(availableStart / daysLeft), 0);
  const perDayFromTomorrow = daysLeft > 1 ? Math.max(Math.trunc(available / (daysLeft - 1)), 0) : 0;
  const dailyIncome = Math.trunc(income / Math.max(daysLeft + d - 1, 1));
  const status = available <= 0 ? 'over' : perDay * 10 < dailyIncome * 3 ? 'tight' : 'ok';
  return {
    month,
    days_left: daysLeft,
    income,
    spent,
    spent_today: spentToday,
    fixed_remaining: fixed,
    savings_remaining: savings,
    available,
    per_day: perDay,
    today_left: Math.max(perDay - spentToday, 0),
    per_day_from_tomorrow: perDayFromTomorrow,
    currency: baseCurrency(),
    status,
    explanation: `Receitas do mês ${brl(income)} − gastos até ontem ${brl(spentBefore)} − contas que ainda vencem ${brl(fixed)} − falta guardar ${brl(savings)} = ${brl(availableStart)} para ${daysLeft} dia(s): ${brl(perDay)} por dia.`,
  };
}

// ── Quitação ──────────────────────────────────────────────────────

interface Debt {
  loan_id: string | null;
  name: string;
  balance: number;
  rate: number;
  annual: string;
  min: number;
}

function simulate(debts: Debt[], strategy: PayoffStrategy, extra: number): PayoffResult {
  const order = debts.map((_, i) => i);
  if (strategy === 'avalanche')
    order.sort((a, b) => debts[b]!.rate - debts[a]!.rate || debts[a]!.balance - debts[b]!.balance);
  else
    order.sort((a, b) => debts[a]!.balance - debts[b]!.balance || debts[b]!.rate - debts[a]!.rate);
  const bal = debts.map((x) => x.balance);
  const interest = debts.map(() => 0);
  const done: (number | null)[] = debts.map(() => null);
  const budget = debts.reduce((s, x) => s + x.min, 0) + extra;
  const runaway = debts.reduce((s, x) => s + x.balance, 0) * 100;
  const label = (n: number) => monthOf(addMonths(today(), n));
  const series = [];
  let paid = 0;
  let month = 0;
  while (month < 600 && bal.some((b) => b > 0) && bal.reduce((s, b) => s + b, 0) < runaway) {
    month += 1;
    debts.forEach((x, i) => {
      if (bal[i]! > 0) {
        const j = Math.round(bal[i]! * x.rate);
        bal[i]! += j;
        interest[i]! += j;
      }
    });
    let left = strategy === 'minimum' ? 0 : budget;
    debts.forEach((x, i) => {
      if (bal[i]! > 0) {
        const pay = Math.min(x.min, bal[i]!);
        bal[i]! -= pay;
        paid += pay;
        left -= pay;
      }
    });
    for (const i of order) {
      if (left <= 0) break;
      if (bal[i]! > 0) {
        const pay = Math.min(left, bal[i]!);
        bal[i]! -= pay;
        paid += pay;
        left -= pay;
      }
    }
    bal.forEach((b, i) => {
      if (b === 0 && done[i] === null) done[i] = month;
    });
    series.push({ month: label(month), balance: bal.reduce((s, b) => s + b, 0) });
  }
  const feasible = bal.every((b) => b <= 0);
  const ranked = order
    .map((i, pos) => ({ i, pos }))
    .sort((a, b) => (done[a.i] ?? Infinity) - (done[b.i] ?? Infinity) || a.pos - b.pos);
  const results: PayoffDebtResult[] = ranked.map(({ i }, n) => ({
    name: debts[i]!.name,
    loan_id: debts[i]!.loan_id,
    order: n + 1,
    paid_off: done[i] !== null ? label(done[i]!) : null,
    months: done[i] ?? null,
    interest: interest[i]!,
  }));
  return {
    strategy,
    feasible,
    months: month,
    debt_free: feasible ? label(month) : null,
    total_interest: interest.reduce((s, v) => s + v, 0),
    total_paid: paid,
    interest_saved: 0,
    months_saved: 0,
    debts: results,
    series,
  };
}

function payoff(a: Record<string, unknown>): PayoffPlan {
  const extra = Number(a.extra_monthly ?? 0);
  if (extra < 0) invalid('o extra por mês não pode ser negativo');
  const ids = a.loan_ids as string[] | undefined;
  const loans = invoke<LoanView[]>('debt.list');
  for (const id of ids ?? []) if (!loans.some((l) => l.id === id)) notFound(`contrato ${id}`);
  const debts: Debt[] = [];
  const notes: string[] = [];
  for (const l of loans) {
    if (l.balance <= 0 || (ids && !ids.includes(l.id))) continue;
    const rate = Number(l.monthly_rate) / 100;
    const n = l.next_installment;
    const parts = n ? n.amortization + n.interest + n.adjustment : 0;
    const min =
      parts > 0
        ? parts
        : Math.round(
            rate === 0
              ? l.balance / Math.max(l.remaining_count, 1)
              : (l.balance * rate) / (1 - (1 + rate) ** -Math.max(l.remaining_count, 1)),
          );
    if (l.indexer !== 'none')
      notes.push(`${l.name}: só a taxa contratada, sem a correção do indexador.`);
    if (l.system === 'sac')
      notes.push(
        `${l.name}: no SAC a parcela cai com o tempo; aqui ela fica fixa (quita um pouco antes).`,
      );
    debts.push({
      loan_id: l.id,
      name: l.name,
      balance: l.balance,
      rate,
      annual: l.annual_rate,
      min,
    });
  }
  for (const c of (a.custom as CustomDebt[] | undefined) ?? []) {
    if (!c.name.trim() || c.balance <= 0 || c.min_payment <= 0)
      invalid('cada dívida informada precisa de nome, saldo e pagamento mínimo positivos');
    const annual = Number(c.annual_rate.replace(',', '.'));
    if (!Number.isFinite(annual)) invalid(`taxa inválida: ${c.annual_rate}`);
    if (annual < 0) invalid('taxa negativa');
    debts.push({
      loan_id: null,
      name: c.name.trim(),
      balance: c.balance,
      rate: (1 + annual / 100) ** (1 / 12) - 1,
      annual: String(annual),
      min: c.min_payment,
    });
  }
  if (!debts.length)
    invalid('nenhuma dívida com saldo: cadastre um empréstimo ou informe uma dívida');
  const minimum = simulate(debts, 'minimum', 0);
  const snowball = simulate(debts, 'snowball', extra);
  const avalanche = simulate(debts, 'avalanche', extra);
  if (minimum.feasible)
    for (const r of [snowball, avalanche]) {
      r.interest_saved = minimum.total_interest - r.total_interest;
      r.months_saved = minimum.months - r.months;
    }
  const recommended: PayoffStrategy =
    avalanche.total_interest < snowball.total_interest ? 'avalanche' : 'snowball';
  const when = (r: PayoffResult) =>
    r.debt_free ? `${r.debt_free.slice(5)}/${r.debt_free.slice(0, 4)}` : 'mais de 50 anos';
  const best = recommended === 'avalanche' ? avalanche : snowball;
  const other = recommended === 'avalanche' ? snowball : avalanche;
  const name = recommended === 'avalanche' ? 'avalanche' : 'bola de neve';
  let explanation = minimum.feasible
    ? `Só com o mínimo: livre das dívidas em ${when(minimum)} com ${brl(minimum.total_interest)} de juros. `
    : 'Só com o mínimo, alguma dívida nunca acaba: a parcela não cobre os juros. ';
  explanation += `Bola de neve: ${when(snowball)} (${brl(snowball.total_interest)} de juros). Avalanche: ${when(avalanche)} (${brl(avalanche.total_interest)} de juros). `;
  explanation +=
    snowball.total_interest === avalanche.total_interest
      ? 'As duas dão o mesmo resultado aqui; a bola de neve quita a primeira dívida mais cedo.'
      : minimum.feasible
        ? `A ${name} economiza ${brl(best.interest_saved)} de juros em relação ao mínimo.`
        : `A ${name} paga ${brl(other.total_interest - best.total_interest)} a menos de juros.`;
  if (!minimum.feasible)
    notes.push('Só com o mínimo alguma dívida não acaba em 50 anos: a parcela não cobre os juros.');
  return {
    start: monthOf(addMonths(today(), 1)),
    extra_monthly: extra,
    monthly_budget: debts.reduce((s, x) => s + x.min, 0) + extra,
    debts: debts.map((x) => ({
      loan_id: x.loan_id,
      name: x.name,
      balance: x.balance,
      annual_rate: x.annual,
      monthly_rate: String(Math.round(x.rate * 1e6) / 1e4),
      min_payment: x.min,
    })),
    minimum,
    snowball,
    avalanche,
    recommended,
    explanation,
    notes: [...new Set(notes)],
  };
}

// ── Assinaturas ───────────────────────────────────────────────────

function merchantKey(payee: string | null, desc: string): string {
  const base = payee?.trim() ? payee : desc;
  return base
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .split(' ')
    .filter((w) => w && !/^\d+$/.test(w))
    .join(' ');
}

function displayName(desc: string): string {
  const words = desc.trim().split(/\s+/);
  let keep = words.length;
  while (keep > 0 && /^[\d/#*-]+$/.test(words[keep - 1]!)) keep -= 1;
  return words.slice(0, keep).join(' ') || desc.trim();
}

function subscriptions(): SubscriptionReport {
  const t = today();
  const from = addDays(t, -760);
  const groups = new Map<string, typeof rows>();
  const rows = [...db.t.txn.values()]
    .filter(
      (x) =>
        x.kind === 'expense' &&
        x.status !== 'planned' &&
        x.amount < 0 &&
        x.date >= from &&
        x.date <= t &&
        !x.installment_plan_id &&
        !x.transfer_id,
    )
    .sort((a, b) => a.date.localeCompare(b.date) || a.id.localeCompare(b.id));
  for (const r of rows) {
    const k = merchantKey(r.payee, r.description);
    if (k.length < 3) continue;
    const key = `${k}|${r.account_id}`;
    groups.set(key, [...(groups.get(key) ?? []), r]);
  }
  const recurrences = new Map(
    [...db.t.recurrence.values()]
      .filter((r) => r.active && r.kind === 'expense')
      .map((r) => [`${merchantKey(null, r.description)}|${r.account_id}`, r.id]),
  );
  const items: Subscription[] = [];
  for (const [key, list0] of groups) {
    if (dismissed.includes(key)) continue;
    const list = list0.filter(
      (r, i) => i === 0 || !(r.date === list0[i - 1]!.date && r.amount === list0[i - 1]!.amount),
    );
    if (list.length < 2) continue;
    const gaps = list.slice(1).map((r, i) => diffDays(r.date, list[i]!.date));
    const gap = [...gaps].sort((a, b) => a - b)[Math.floor(gaps.length / 2)]!;
    const cadence = gap >= 26 && gap <= 35 ? 'monthly' : gap >= 350 && gap <= 380 ? 'yearly' : null;
    if (!cadence) continue;
    const [lo, hi] = cadence === 'monthly' ? [20, 40] : [330, 400];
    if (gaps.filter((g) => g < lo || g > hi).length > 1) continue;
    const amounts = list.map((r) => -r.amount);
    const min = Math.min(...amounts);
    const max = Math.max(...amounts);
    if (max > min * 1.5) continue;
    const first = list[0]!;
    const last = list.at(-1)!;
    const isNew = diffDays(t, first.date) <= 70;
    if (cadence === 'monthly' && list.length < 3 && !(isNew && (max - min) * 20 <= max)) continue;
    const next = addMonths(last.date, cadence === 'monthly' ? 1 : 12);
    const grace = cadence === 'monthly' ? 10 : 30;
    const late = t > addDays(next, grace);
    if (late && t > addDays(next, grace + 60)) continue;
    const amount = -last.amount;
    const previous = list.length >= 2 ? -list.at(-2)!.amount : null;
    const d = previous !== null ? amount - previous : 0;
    const change =
      previous && Math.abs(d) >= 100 && Math.abs(d) / previous >= 0.01
        ? amount / previous - 1
        : null;
    const acc = db.t.account.get(last.account_id);
    const cat = last.category_id ? db.t.category.get(last.category_id) : undefined;
    const recurrenceId =
      [...list].reverse().find((r) => r.recurrence_id)?.recurrence_id ??
      recurrences.get(key) ??
      null;
    items.push({
      key,
      name: displayName(last.description),
      account_id: last.account_id,
      account_name: acc?.name ?? '',
      currency: last.currency,
      category_id: last.category_id,
      category: cat?.name ?? null,
      cadence,
      status: late ? 'late' : isNew ? 'new' : 'active',
      // Como o núcleo: categoria essencial ou palavra de conta da casa → conta fixa.
      kind:
        /\b(escola|colegio|faculdade|universidade|mensalidade|condominio|boleto|aluguel|iptu)\b/.test(
          key,
        ) ||
        (cat != null && (cat as { spend_group?: string | null }).spend_group === 'essential')
          ? 'fixed_bill'
          : 'subscription',
      amount,
      previous_amount: previous,
      change_pct: change,
      monthly_cost: cadence === 'monthly' ? amount : Math.round(amount / 12),
      annual_cost: cadence === 'monthly' ? amount * 12 : amount,
      first_date: first.date,
      last_date: last.date,
      next_expected: next,
      recurrence_id: recurrenceId,
      suggested_recurrence: recurrenceId
        ? null
        : {
            description: last.description,
            account_id: last.account_id,
            amount: last.amount,
            kind: 'expense',
            ...(last.category_id ? { category_id: last.category_id } : {}),
            spec: {
              freq: cadence === 'monthly' ? 'MONTHLY' : 'YEARLY',
              by_month_day: parseYmd(last.date).d,
              ...(cadence === 'yearly' ? { by_month: parseYmd(last.date).m } : {}),
            },
            start_date: next > t ? next : t,
          },
      charges: list.map((r) => ({ txn_id: r.id, date: r.date, amount: -r.amount })),
    });
  }
  items.sort((a, b) => b.monthly_cost - a.monthly_cost || a.name.localeCompare(b.name));
  const counted = items.filter((s) => s.status !== 'late' && s.currency === baseCurrency());
  const subs = counted.filter((s) => s.kind === 'subscription');
  const fixed = counted.filter((s) => s.kind === 'fixed_bill');
  return {
    as_of: t,
    items,
    monthly_total: subs.reduce((s, x) => s + x.monthly_cost, 0),
    annual_total: subs.reduce((s, x) => s + x.annual_cost, 0),
    fixed_monthly_total: fixed.reduce((s, x) => s + x.monthly_cost, 0),
    fixed_annual_total: fixed.reduce((s, x) => s + x.annual_cost, 0),
    currency: baseCurrency(),
    increases: items.filter((s) => (s.change_pct ?? 0) > 0).length,
    new_count: items.filter((s) => s.status === 'new').length,
    late_count: items.filter((s) => s.status === 'late').length,
    dismissed: [...dismissed].sort(),
  };
}

// ── Cenários "e se" ───────────────────────────────────────────────

const eventStart = (e: ScenarioEvent) => ('date' in e ? e.date : e.start);

function monthsFrom(t: string, date: string): number {
  const a = parseYmd(t);
  const b = parseYmd(date);
  return (b.y - a.y) * 12 + b.m - a.m;
}

function pricePayment(p: number, i: number, n: number): number {
  return Math.abs(i) < 1e-12 ? p / Math.max(n, 1) : (p * i) / (1 - (1 + i) ** -Math.max(n, 1));
}

interface ScenarioStart {
  liquid: number;
  others: number;
  income: number;
  expense: number;
}

function project(
  s: ScenarioStart,
  events: ScenarioEvent[],
  t: string,
  months: number,
  rate: number,
): { path: ScenarioPath; flows: number[] } {
  const mr = (1 + rate) ** (1 / 12) - 1;
  const starts = events.map((e) => Math.max(monthsFrom(t, eventStart(e)), 1));
  let liquid = s.liquid;
  let others = s.others;
  const assets: [number, number][] = [];
  const loans: [number, number, number, number][] = [];
  const points = [];
  const flows: number[] = [];
  let firstShort: string | null = liquid < 0 ? monthOf(t) : null;
  const on = (st: number, n: number | undefined, m: number) => m >= st && (n == null || m < st + n);
  for (let m = 1; m <= months; m += 1) {
    let factor = 1;
    let cash = 0;
    let change = 0;
    events.forEach((e, k) => {
      const st = starts[k]!;
      if (e.kind === 'one_off' && m === st) cash += e.amount;
      if (e.kind === 'monthly' && on(st, e.months, m)) cash += e.amount;
      if (e.kind === 'income_pct' && on(st, e.months, m)) factor *= e.pct / 100;
      if (e.kind === 'asset' && m === st) {
        assets.push([e.value, (1 + (e.real_growth_pct ?? 0) / 100) ** (1 / 12) - 1]);
        change += e.value;
      }
      if (e.kind === 'loan' && m === st) {
        const i = (1 + e.annual_rate_pct / 100) ** (1 / 12) - 1;
        cash += e.principal;
        loans.push([e.principal, i, pricePayment(e.principal, i, e.months), m]);
        change -= e.principal;
      }
    });
    let paid = 0;
    let amortized = 0;
    for (const l of loans) {
      if (l[0] <= 0.5 || l[3] >= m) continue;
      const interest = l[0] * l[1];
      const pay = Math.min(l[2], l[0] + interest);
      l[0] = l[0] + interest - pay;
      paid += pay;
      amortized += pay - interest;
    }
    let grown = 0;
    for (const a of assets) {
      const g = a[0] * a[1];
      a[0] += g;
      grown += g;
    }
    const income = s.income * factor;
    const flow = income - s.expense + cash - paid;
    if (liquid > 0) liquid *= 1 + mr;
    liquid += flow;
    others += change + amortized + grown;
    flows.push(flow);
    const month = monthOf(addMonths(t, m));
    if (firstShort === null && liquid < 0) firstShort = month;
    points.push({
      month,
      liquid: Math.round(liquid),
      net_worth: Math.round(liquid + others),
      income: Math.round(income),
      expense: Math.round(s.expense - Math.min(cash, 0) + paid),
    });
  }
  const last = points.at(-1);
  return {
    path: {
      points,
      first_short: firstShort,
      end_liquid: last?.liquid ?? s.liquid,
      end_net_worth: last?.net_worth ?? s.liquid + s.others,
    },
    flows,
  };
}

function scenarioSimulate(a: Record<string, unknown>): ScenarioResult {
  const t = today();
  let events = (a.events as ScenarioEvent[] | undefined) ?? [];
  const name = typeof a.name === 'string' ? a.name : null;
  if (!events.length && name) events = scenarios.find((x) => x.name === name)?.events ?? [];
  for (const e of events) {
    if (!e.label.trim()) invalid('cada evento precisa de um nome');
    if (e.kind === 'loan' && (e.principal <= 0 || e.months <= 0 || e.months > 600))
      invalid(`${e.label}: financiamento com valor, prazo (1 a 600 meses) e taxa válidos`);
  }
  const years = Math.min(Math.max(Number(a.years ?? 10), 1), 40);
  const rate = typeof a.real_return === 'number' ? a.real_return : 0.04;
  const nw = invoke<NetWorth>('reports.net_worth', { as_of: t });
  const first = addMonths(`${monthOf(t)}-01`, -12);
  const last = addDays(`${monthOf(t)}-01`, -1);
  const parts = flowParts({ from: first, to: last, basis: 'accrual' });
  const months = Math.max(new Set(parts.map((p) => p.txn.date.slice(0, 7))).size, 1);
  const s: ScenarioStart = {
    liquid: nw.net - nw.physical + nw.debts,
    others: nw.physical - nw.debts,
    income: Math.trunc(
      parts.filter((p) => p.amount > 0).reduce((x, p) => x + p.amount, 0) / months,
    ),
    expense: Math.trunc(
      -parts.filter((p) => p.amount < 0).reduce((x, p) => x + p.amount, 0) / months,
    ),
  };
  if (typeof a.monthly_income === 'number') s.income = a.monthly_income;
  if (typeof a.monthly_expense === 'number') s.expense = a.monthly_expense;
  const base = project(s, [], t, years * 12, rate);
  const scen = project(s, events, t, years * 12, rate);
  // Previsão de 12 meses: o total previsto + o efeito em caixa dos eventos.
  const f = forecast({ months: 12 });
  const effects: [string, number][] = [];
  for (const e of events) {
    const d0 = eventStart(e) > t ? eventStart(e) : t;
    if (e.kind === 'one_off' && d0 <= f.to) effects.push([d0, e.amount]);
    const repeat = (amount: number, n: number | undefined, offset = 0) => {
      for (let k = offset; k < Math.min(n ?? 13, 13) + offset; k += 1) {
        const d = addMonths(d0, k);
        if (d > f.to) break;
        effects.push([d, amount]);
      }
    };
    if (e.kind === 'monthly') repeat(e.amount, e.months ?? undefined);
    if (e.kind === 'income_pct')
      repeat(Math.round(s.income * (e.pct / 100 - 1)), e.months ?? undefined);
    if (e.kind === 'loan') {
      const i = (1 + e.annual_rate_pct / 100) ** (1 / 12) - 1;
      if (d0 <= f.to) effects.push([d0, e.principal]);
      repeat(-Math.round(pricePayment(e.principal, i, e.months)), e.months, 1);
    }
  }
  const dates = [
    ...new Set([...f.total_points.map((p) => p.date), ...effects.map(([d]) => d)]),
  ].sort();
  const points = dates.map((d) => {
    const b = f.total_points.filter((p) => p.date <= d).at(-1)?.balance ?? f.total_start;
    const delta = effects.filter(([e]) => e <= d).reduce((x, [, v]) => x + v, 0);
    return { date: d, base: b, scenario: b + delta };
  });
  const nwDelta = scen.path.end_net_worth - base.path.end_net_worth;
  const liqDelta = scen.path.end_liquid - base.path.end_liquid;
  const scenNeg = points.find((p) => p.scenario < 0)?.date ?? null;
  const baseNeg = points.find((p) => p.base < 0)?.date ?? null;
  let explanation = `Em ${years} ano(s), o patrimônio fica ${brl(scen.path.end_net_worth)} no cenário contra ${brl(base.path.end_net_worth)} na base (${brl(nwDelta)} de diferença; ${brl(liqDelta)} em contas e investimentos).`;
  if (scen.path.first_short === monthOf(today()))
    explanation += ' O dinheiro disponível já está negativo hoje.';
  else if (scen.path.first_short)
    explanation += ` O dinheiro disponível acaba em ${scen.path.first_short.slice(5)}/${scen.path.first_short.slice(0, 4)}.`;
  if (scenNeg && scenNeg !== baseNeg)
    explanation += ` Nos próximos 12 meses, o saldo das contas fica negativo em ${brDate(scenNeg)}.`;
  return {
    name: name ?? 'Cenário',
    start: monthOf(t),
    years,
    real_return: rate,
    monthly_income: s.income,
    monthly_expense: s.expense,
    base: base.path,
    scenario: scen.path,
    net_worth_delta: nwDelta,
    liquid_delta: liqDelta,
    forecast: { points, base_first_negative: baseNeg, scenario_first_negative: scenNeg },
    retirement: null,
    explanation,
    notes: [
      `Valores reais de hoje; renda de ${brl(s.income)} e gasto de ${brl(s.expense)} por mês (média dos últimos 12 meses), retorno real de ${(rate * 100).toFixed(1).replace('.', ',')}% a.a.`,
      'Bens e dívidas que já existem ficam no valor de hoje; as parcelas atuais já estão no gasto médio.',
      'Sem idade no plano de aposentadoria (Orçamento › Aposentadoria): o efeito na aposentadoria fica de fora.',
    ],
  };
}

/** Taxas do mercado de exemplo (o núcleo baixa do BCB). */
function marketRates() {
  const mk = (modality: string, label: string, rates: number[]) => {
    const s = [...rates].sort((a, b) => a - b);
    const q = (p: number) => s[Math.round((s.length - 1) * p)]!;
    return {
      modality,
      label,
      period_start: '2026-09-25',
      period_end: '2026-10-01',
      count: s.length,
      min: s[0]!,
      p25: q(0.25),
      median: q(0.5),
      max: s[s.length - 1]!,
      median_year: Math.round(((1 + q(0.5) / 100) ** 12 - 1) * 10000) / 100,
      best: s.slice(0, 5).map((r, i) => ({
        institution: `Banco ${String.fromCharCode(65 + i)}`,
        rate_month: r,
        rate_year: Math.round(((1 + r / 100) ** 12 - 1) * 10000) / 100,
      })),
    };
  };
  return {
    fetched_at: new Date().toISOString(),
    modalities: [
      mk('Cheque especial - Prefixado', 'Cheque especial', [6.1, 7.4, 7.8, 7.9, 8.0]),
      mk(
        'Cartão de crédito - rotativo total - Prefixado',
        'Cartão de crédito: rotativo',
        [5.5, 9.8, 12.1, 14.3, 15.2],
      ),
      mk(
        'Crédito pessoal não consignado - Prefixado',
        'Crédito pessoal',
        [1.9, 3.2, 4.1, 6.5, 9.8],
      ),
      mk(
        'Crédito pessoal consignado INSS - Prefixado',
        'Consignado INSS',
        [1.5, 1.6, 1.7, 1.8, 1.85],
      ),
      mk(
        'Aquisição de veículos - Prefixado',
        'Financiamento de veículo',
        [1.2, 1.5, 1.8, 2.1, 2.6],
      ),
    ],
    loans: [],
    warning: null,
    source: 'Banco Central do Brasil (exemplo).',
  };
}

registerMockHandlers(
  guarded({
    'forecast.balance': (a) => forecast(a),
    'forecast.daily_allowance': () => dailyAllowance(),
    'debt.payoff_plan': (a) => payoff(a),
    'debt.market_rates': () => marketRates(),
    'subscriptions.list': () => subscriptions(),
    'subscriptions.dismiss': (a) => {
      const key = typeof a.key === 'string' ? a.key : '';
      dismissed = [...dismissed.filter((k) => k !== key), key];
      return subscriptions();
    },
    'subscriptions.restore': (a) => {
      dismissed = dismissed.filter((k) => k !== a.key);
      return subscriptions();
    },
    'notify.get_prefs': () => prefs,
    'notify.set_prefs': (a) => {
      prefs = { ...DEFAULT_PREFS, ...(a as Partial<NotifyPrefs>) };
      prefs.bills_days = Math.min(prefs.bills_days, 15);
      return prefs;
    },
    'notify.run_now': () => ({ created: 0 }),
    'scenarios.simulate': (a) => scenarioSimulate(a),
    'scenarios.list': () => scenarios,
    'scenarios.save': (a) => {
      const sc = a as unknown as SavedScenario;
      const name = sc.name.trim();
      if (!name) invalid('dê um nome ao cenário');
      scenarios = [...scenarios.filter((x) => x.name !== name), { name, events: sc.events }].sort(
        (x, y) => x.name.localeCompare(y.name),
      );
      return scenarios;
    },
    'scenarios.delete': (a) => {
      scenarios = scenarios.filter((x) => x.name !== a.name);
      return scenarios;
    },
  }),
);
