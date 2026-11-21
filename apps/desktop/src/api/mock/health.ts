/**
 * Mock de `health.*` sobre o livro-caixa fictício: as mesmas leituras, curvas, faixas e
 * textos de `crates/treasury-core/src/health/` (facts.rs + models.rs), para a interface ver
 * a nota que o núcleo daria.
 */
import type { Basis } from '@/api/generated/Basis';
import type { HealthAmount } from '@/api/generated/HealthAmount';
import type { HealthBand } from '@/api/generated/HealthBand';
import type { HealthDimension } from '@/api/generated/HealthDimension';
import type { HealthMetric } from '@/api/generated/HealthMetric';
import type { HealthModel } from '@/api/generated/HealthModel';
import type { HealthModelInfo } from '@/api/generated/HealthModelInfo';
import type { HealthReport } from '@/api/generated/HealthReport';
import type { HealthSettingsView } from '@/api/generated/HealthSettingsView';
import type { HealthStatus } from '@/api/generated/HealthStatus';
import type { HealthUnit } from '@/api/generated/HealthUnit';
import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import type { MonthFlow } from '@/api/generated/MonthFlow';
import type { NetWorthPoint } from '@/api/generated/NetWorthPoint';
import { extraMockHandler, onMockReset } from '@/api/mockRegistry';
import { addDays, monthBounds, monthsBetween, shiftMonth } from '@/lib/months';

import { budgetGet, budgetMonths } from './ledger/budgetState';
import { defaultBasis, monthArg, optStr, register } from './ledger/common';
import { invalid } from './ledger/errors';
import {
  aggregate,
  balancesAt,
  divRound,
  firstFlowMonth,
  Fx,
  KEY,
  monthLabel,
} from './ledger/engine';
import { ACCOUNTS, BASE, CATEGORIES, category } from './ledger/fixture';
import { emergencyReserve } from './ledger/goalsState';
import { getLedger } from './ledger/ledger';
import { monthFlow, netWorthHistory } from './ledger/reports';

const SOURCE_URL = 'https://meubolsoemdia.com.br/Materias/indice-saude-financeira-2024';
const NATIONAL_AVERAGE = 56.7;
const MIN_MONTHS = 3;
const WINDOW = 12;
const ESSENTIAL_MONTHS = 6;
const GRACE_DAYS = 5;
const BILL_TOLERANCE = 100;
const HOUSING_NAMES = ['moradia', 'habitação', 'habitacao', 'casa'];

let model: HealthModel = 'isf';
onMockReset(() => {
  model = 'isf';
});

const MODELS: HealthModelInfo[] = [
  {
    id: 'isf',
    label: 'I-SFB (Febraban e Banco Central)',
    description:
      'Inspirado no Índice de Saúde Financeira do Brasileiro: nota de 0 a 100 em quatro dimensões (comportamento, habilidade, liberdade e segurança), comparada com a média nacional de 56,7 em 2024. Estimativa a partir dos seus dados, não o questionário oficial.',
    source_url: SOURCE_URL,
  },
  {
    id: 'ratios',
    label: 'Indicadores clássicos',
    description:
      'As razões da educação financeira, cada uma com meta: poupar 20% da renda, reserva de 6 meses de gastos essenciais, parcelas até 15% da renda, moradia até 30%, nenhum rotativo e patrimônio crescendo.',
    source_url: null,
  },
  {
    id: 'off',
    label: 'Desativado',
    description:
      'Sem análise de saúde financeira: o card some do painel e a seção dos relatórios fica vazia.',
    source_url: null,
  },
];

const settingsView = (): HealthSettingsView => ({ model, models: MODELS });

// ── Fatos ────────────────────────────────────────────────────────

interface Facts {
  as_of: string;
  basis: Basis;
  months: string[];
  flows: MonthFlow[];
  income: number;
  expense: number;
  essentialAvg: number;
  essentialMonths: number;
  reserve: number;
  reserveFromGoal: boolean;
  debtService: number;
  loans: number;
  instOnTime: number;
  instLate: number;
  billsOnTime: number;
  billsLate: number;
  cardsTracked: number;
  revolving: Set<string>;
  checkingAccounts: number;
  overdraft: Set<string>;
  budgetMonths: number;
  budgetWithin: number;
  housing: number | null;
  netWorth: NetWorthPoint[];
  missing: Set<string>;
  estimated: boolean;
}

const n = (f: Facts) => f.months.length;
const enough = (f: Facts) => n(f) >= MIN_MONTHS;

function periodLabel(f: Facts): string {
  const a = f.months[0];
  const b = f.months[f.months.length - 1];
  if (!a || !b) return 'sem meses';
  return a === b ? monthLabel(a) : `${monthLabel(a)}–${monthLabel(b)}`;
}

function loadFacts(asOf: string): Facts {
  const { today, txns, statements } = getLedger();
  const current = today.slice(0, 7);
  const isCurrent = asOf === current;
  const end = isCurrent ? shiftMonth(asOf, -1) : asOf;
  const ref = isCurrent ? today : monthBounds(asOf).to;
  const basis = defaultBasis();
  const f: Facts = {
    as_of: asOf,
    basis,
    months: [],
    flows: [],
    income: 0,
    expense: 0,
    essentialAvg: 0,
    essentialMonths: 0,
    reserve: 0,
    reserveFromGoal: false,
    debtService: 0,
    loans: 0,
    instOnTime: 0,
    instLate: 0,
    billsOnTime: 0,
    billsLate: 0,
    cardsTracked: 0,
    revolving: new Set(),
    checkingAccounts: 0,
    overdraft: new Set(),
    budgetMonths: 0,
    budgetWithin: 0,
    housing: null,
    netWorth: [],
    missing: new Set(),
    estimated: false,
  };
  const first = firstFlowMonth();
  if (!first || first > end) return f;
  const start = shiftMonth(end, -(WINDOW - 1));
  f.months = monthsBetween(start > first ? start : first, end);
  const from = monthBounds(f.months[0]!).from;
  const to = monthBounds(f.months[f.months.length - 1]!).to;
  const fx = new Fx();

  f.flows = monthFlow(f.months[0]!, f.months[f.months.length - 1]!, basis);
  for (const m of f.flows) {
    f.income += m.income;
    f.expense += m.expense;
    m.missing_fx.forEach((c) => f.missing.add(c));
    f.estimated ||= m.estimated;
  }

  const ess = f.months.slice(-ESSENTIAL_MONTHS);
  if (isCurrent) {
    const er = emergencyReserve(6, ESSENTIAL_MONTHS);
    f.essentialAvg = er.avg_essential_monthly;
    f.essentialMonths = er.months_analyzed;
    f.reserve = er.current_amount;
    f.reserveFromGoal = er.goal_id != null;
  } else {
    const agg = aggregate(
      {
        from: monthBounds(ess[0]!).from,
        to: monthBounds(ess[ess.length - 1]!).to,
        basis,
        kind: 'expense',
      },
      KEY.category,
      KEY.none,
      fx,
    );
    let essential = 0;
    for (const [k, cell] of agg.cells) {
      if (category(k.split('|')[0])?.is_essential) essential += cell.expense;
    }
    f.essentialAvg = divRound(essential, ess.length);
    f.essentialMonths = ess.length;
    for (const b of balancesAt(ref)) {
      const acc = ACCOUNTS.find((a) => a.id === b.id);
      if (!acc?.include_in_net_worth || !['checking', 'savings', 'cash'].includes(acc.kind))
        continue;
      const c = fx.convert(b.balance, b.currency, ref);
      if (c) f.reserve += c.amount;
    }
  }

  const housingRoots = new Set(
    CATEGORIES.filter(
      (c) =>
        !c.parent_id && c.kind === 'expense' && HOUSING_NAMES.includes(c.name.trim().toLowerCase()),
    ).map((c) => c.id),
  );
  const inHousing = (id: string | null) => {
    let c = category(id);
    while (c?.parent_id) c = category(c.parent_id);
    return !!c && housingRoots.has(c.id);
  };

  // Parcelas (pelo mock de dívidas, se houver).
  const list = extraMockHandler('debt.list');
  const schedule = extraMockHandler('debt.schedule');
  let mortgageOutside = 0;
  let hasMortgage = false;
  if (list && schedule) {
    for (const loan of list({ include_archived: false }) as LoanView[]) {
      let any = false;
      for (const i of schedule({ id: loan.id }) as Installment[]) {
        if (i.due_date < from || i.due_date > to) continue;
        const c = fx.convert(i.total, loan.currency, i.due_date);
        if (!c) continue;
        any = true;
        f.debtService += c.amount;
        if (loan.kind === 'mortgage') {
          hasMortgage = true;
          if (!inHousing(loan.category_id)) mortgageOutside += c.amount;
        }
        const limit = addDays(i.due_date, GRACE_DAYS);
        if (i.paid) {
          if (i.paid_date && i.paid_date > limit) f.instLate += 1;
          else f.instOnTime += 1;
        } else if (limit < ref) {
          f.instLate += 1;
        }
      }
      if (any) f.loans += 1;
    }
  }

  // Faturas dos cartões com pagamento registrado.
  const payers = new Set(
    txns
      .filter((t) => t.kind === 'card_payment' && t.status !== 'planned')
      .map((t) => t.account_id),
  );
  const cards = new Set<string>();
  for (const s of [...statements.values()].sort((a, b) => a.due_date.localeCompare(b.due_date))) {
    if (s.due_date < from || s.due_date > to || !payers.has(s.account_id)) continue;
    cards.add(s.account_id);
    const limit = addDays(s.due_date, GRACE_DAYS);
    let total = 0;
    let paid = 0;
    for (const t of txns) {
      if (t.statement_id !== s.id || t.status === 'planned') continue;
      if (t.kind !== 'card_payment') total -= t.amount;
      else if (t.date <= limit) paid += t.amount;
    }
    if (total <= 0 || limit >= ref) continue;
    if (paid + BILL_TOLERANCE >= total) f.billsOnTime += 1;
    else {
      f.billsLate += 1;
      f.revolving.add(s.due_date.slice(0, 7));
    }
  }
  f.cardsTracked = cards.size;

  // Cheque especial: saldo do dia negativo numa conta corrente.
  const { start: opening } = getLedger();
  const checking = ACCOUNTS.filter((a) => a.kind === 'checking' && opening <= to);
  f.checkingAccounts = checking.length;
  for (const a of checking) {
    const events = new Map<string, number>([[opening, a.opening_balance]]);
    for (const t of txns) {
      if (t.account_id !== a.id || t.status === 'planned' || t.date > to) continue;
      events.set(t.date, (events.get(t.date) ?? 0) + t.amount);
    }
    const days = [...events.entries()].sort((x, y) => x[0].localeCompare(y[0]));
    let balance = days.filter(([d]) => d < from).reduce((s, [, v]) => s + v, 0);
    const inside = days.filter(([d]) => d >= from);
    let k = 0;
    for (const m of f.months) {
      if (balance < 0) f.overdraft.add(m);
      while (k < inside.length && inside[k]![0].slice(0, 7) === m) {
        balance += inside[k]![1];
        if (balance < 0) f.overdraft.add(m);
        k += 1;
      }
    }
  }

  const budgeted = new Set(budgetMonths());
  for (const m of f.months.filter((x) => budgeted.has(x))) {
    const t = budgetGet(m, basis).totals;
    if (t.planned_expense > 0) {
      f.budgetMonths += 1;
      if (t.actual_expense <= t.planned_expense) f.budgetWithin += 1;
    }
  }

  if (housingRoots.size > 0) {
    const agg = aggregate({ from, to, basis, kind: 'expense' }, KEY.root, KEY.none, fx);
    let spent = 0;
    for (const [k, c] of agg.cells) if (housingRoots.has(k.split('|')[0]!)) spent += c.expense;
    f.housing = spent + mortgageOutside;
  } else if (hasMortgage) {
    f.housing = mortgageOutside;
  }

  f.netWorth = netWorthHistory(WINDOW + 1, asOf);
  for (const p of f.netWorth) p.missing_fx.forEach((c) => f.missing.add(c));
  fx.missing.forEach((c) => f.missing.add(c));
  f.estimated ||= fx.estimated;
  return f;
}

// ── Utilitários ──────────────────────────────────────────────────

function curve(x: number, pts: readonly (readonly [number, number])[]): number {
  if (x <= pts[0]![0]) return pts[0]![1];
  for (let i = 1; i < pts.length; i++) {
    const [a, ya] = pts[i - 1]!;
    const [b, yb] = pts[i]!;
    if (x <= b) return ya + ((yb - ya) * (x - a)) / (b - a);
  }
  return pts[pts.length - 1]![1];
}

const round1 = (x: number) => Math.round(x * 10) / 10;
const scoreBr = (x: number) => x.toFixed(1).replace('.', ',');
const percentBr = (r: number) => `${(r * 100).toFixed(1).replace('.', ',')}%`.replace('-', '−');
function monthsBr(x: number): string {
  const s = x.toFixed(1).replace('.', ',');
  return Math.abs(x - 1) < 0.05 ? `${s} mês` : `${s} meses`;
}
const of = (a: number, total: number, what: string) => `${a} de ${total} ${what}`;
const pluralMonths = (k: number) => (k === 1 ? '1 mês' : `${k} meses`);
const basisLabel = (f: Facts) => (f.basis === 'cash' ? 'caixa' : 'competência');

type Cmp = { atLeast: number } | { atMost: number };
const passes = (c: Cmp, v: number) =>
  'atLeast' in c ? v >= c.atLeast - 1e-9 : v <= c.atMost + 1e-9;

interface Scale {
  points: readonly (readonly [number, number])[];
  good: Cmp;
  ok: Cmp;
}

interface Reading {
  value: number;
  graded?: number;
  text: string;
  computation: string;
  amounts: HealthAmount[];
}
type Read = Reading | string;

interface Spec {
  key: string;
  label: string;
  unit: HealthUnit;
  target: string;
  targetValue: number;
  explanation: string;
  advice: string[];
  noData: string;
}

function build(
  spec: Spec,
  dimension: string | null,
  weight: number,
  r: Read,
  s: Scale,
): HealthMetric {
  let status: HealthStatus = 'unknown';
  let points: number | null = null;
  if (typeof r !== 'string') {
    const v = r.graded ?? r.value;
    status = passes(s.good, v) ? 'good' : passes(s.ok, v) ? 'ok' : 'bad';
    points = round1(Math.min(100, Math.max(0, curve(v, s.points))));
  }
  const suggestions =
    status === 'good'
      ? []
      : status === 'unknown'
        ? [spec.noData]
        : spec.advice.slice(0, status === 'ok' ? 2 : 3);
  return {
    key: spec.key,
    label: spec.label,
    dimension,
    value: typeof r === 'string' ? null : r.value,
    unit: spec.unit,
    value_text: typeof r === 'string' ? null : r.text,
    target: spec.target,
    target_value: spec.targetValue,
    status,
    points,
    weight,
    explanation: spec.explanation,
    computation: typeof r === 'string' ? r : r.computation,
    suggestions,
    amounts: typeof r === 'string' ? [] : r.amounts,
  };
}

const amount = (label: string, cents: number): HealthAmount => ({
  label,
  amount: cents,
  currency: BASE,
});

function needMonths(f: Facts): string | null {
  return enough(f)
    ? null
    : `Sem dados: são precisos ao menos ${MIN_MONTHS} meses completos de lançamentos (${n(f)} analisado${n(f) === 1 ? '' : 's'}).`;
}

const noIncome = (f: Facts) => `Sem dados: nenhuma receita registrada em ${periodLabel(f)}.`;

function weighted(items: [number | null, number][]): [number, number] | null {
  const known = items.filter((i): i is [number, number] => i[0] != null);
  const wsum = known.reduce((s, [, w]) => s + w, 0);
  if (known.length === 0 || wsum <= 0) return null;
  const total = items.reduce((s, [, w]) => s + w, 0);
  return [round1(known.reduce((s, [p, w]) => s + p * w, 0) / wsum), wsum / total];
}

function bandOf(bands: HealthBand[], score: number): HealthBand | null {
  const s = Math.min(100, Math.max(0, Math.round(score)));
  return bands.find((b) => s >= b.min && s <= b.max) ?? null;
}

// ── Leituras ─────────────────────────────────────────────────────

function readSavings(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  if (f.income <= 0) return noIncome(f);
  const net = f.income - f.expense;
  const r = net / f.income;
  return {
    value: r,
    text: percentBr(r),
    computation: `(receitas − despesas) ÷ receitas de ${periodLabel(f)} (${n(f)} meses, base ${basisLabel(f)}); transferências, pagamento de fatura e aportes não contam.`,
    amounts: [amount('Receitas', f.income), amount('Despesas', f.expense), amount('Sobra', net)],
  };
}

function readRegular(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  const withIncome = f.flows.filter((m) => m.income > 0);
  if (withIncome.length === 0) return noIncome(f);
  const good = withIncome.filter((m) => (m.savings_rate ?? -1) >= 0.1).length;
  return {
    value: good / withIncome.length,
    text: of(good, withIncome.length, 'meses'),
    computation: `Meses com taxa de poupança de 10% ou mais ÷ meses com receita, em ${periodLabel(f)}.`,
    amounts: [],
  };
}

function readReserve(f: Facts): Read {
  if (f.essentialMonths < MIN_MONTHS)
    return `Sem dados: são precisos ao menos ${MIN_MONTHS} meses completos de gastos para a média (${f.essentialMonths} analisado${f.essentialMonths === 1 ? '' : 's'}).`;
  if (f.essentialAvg <= 0)
    return 'Sem dados: nenhuma despesa em categoria essencial nos últimos meses. Marque as categorias essenciais em Ajustes › Categorias.';
  const months = Math.max(0, f.reserve) / f.essentialAvg;
  const source = f.reserveFromGoal
    ? 'o valor atual da meta de reserva de emergência'
    : 'o saldo das contas corrente, poupança e dinheiro';
  return {
    value: months,
    text: monthsBr(months),
    computation: `Reserva (${source}) ÷ média mensal das despesas em categorias essenciais dos últimos ${f.essentialMonths} meses completos.`,
    amounts: [amount('Reserva', f.reserve), amount('Gasto essencial por mês', f.essentialAvg)],
  };
}

function readDebt(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  if (f.income <= 0) return noIncome(f);
  const r = f.debtService / f.income;
  const what =
    f.loans === 0
      ? 'nenhum empréstimo ou financiamento com parcela no período'
      : `parcelas de ${f.loans} contrato${f.loans === 1 ? '' : 's'} (empréstimos e financiamentos)`;
  return {
    value: r,
    text: percentBr(r),
    computation: `Parcelas com vencimento em ${periodLabel(f)} ÷ receitas do período; ${what}. Parcelamentos no cartão contam como despesa, não aqui.`,
    amounts: [amount('Parcelas', f.debtService), amount('Receitas', f.income)],
  };
}

function readHousing(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  if (f.housing == null)
    return 'Sem dados: não há categoria de despesa "Moradia" nem financiamento imobiliário.';
  if (f.income <= 0) return noIncome(f);
  const r = f.housing / f.income;
  return {
    value: r,
    text: percentBr(r),
    computation: `Despesas em Moradia (aluguel, condomínio, contas da casa…) e parcelas de financiamento imobiliário lançadas fora dela ÷ receitas de ${periodLabel(f)}.`,
    amounts: [amount('Moradia', f.housing), amount('Receitas', f.income)],
  };
}

function readCredit(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  if (f.checkingAccounts === 0 && f.cardsTracked === 0)
    return 'Sem dados: nenhuma conta corrente nem cartão com pagamento de fatura registrado.';
  const months = new Set([...f.overdraft, ...f.revolving]);
  const parts: string[] = [];
  if (f.overdraft.size) parts.push(`cheque especial em ${pluralMonths(f.overdraft.size)}`);
  if (f.revolving.size) parts.push(`fatura não paga inteira em ${pluralMonths(f.revolving.size)}`);
  let computation = `Meses de ${periodLabel(f)} com saldo negativo em conta corrente (cheque especial) ou fatura de cartão não paga inteira até ${GRACE_DAYS} dias após o vencimento (rotativo): ${parts.length ? parts.join('; ') : 'nenhum uso'}.`;
  if (f.cardsTracked === 0)
    computation += ' Cartões sem pagamento de fatura registrado ficam de fora.';
  return {
    value: months.size,
    graded: months.size / Math.max(1, n(f)),
    text: months.size === 0 ? 'nenhum mês' : of(months.size, n(f), 'meses'),
    computation,
    amounts: [],
  };
}

function readNetWorth(f: Facts): Read {
  const pts = f.netWorth;
  if (pts.length < MIN_MONTHS + 1)
    return `Sem dados: são precisos ao menos ${MIN_MONTHS + 1} meses de patrimônio (${pts.length} encontrado${pts.length === 1 ? '' : 's'}).`;
  const a = pts[0]!;
  const b = pts[pts.length - 1]!;
  const delta = b.net - a.net;
  const base = Math.max(Math.abs(a.net), f.expense, 1);
  const g = delta / base;
  const scale =
    base === Math.abs(a.net)
      ? 'o patrimônio inicial'
      : 'as despesas do período (o patrimônio inicial é pequeno)';
  return {
    value: g,
    text: `${g > 0 ? '+' : ''}${percentBr(g)}`,
    computation: `Variação do patrimônio líquido (contas, carteira, bens e dívidas) de ${monthLabel(a.month)} a ${monthLabel(b.month)} ÷ ${scale}. Valor nominal, sem descontar a inflação.`,
    amounts: [
      amount(`Patrimônio em ${monthLabel(a.month)}`, a.net),
      amount(`Patrimônio em ${monthLabel(b.month)}`, b.net),
      amount('Variação', delta),
    ],
  };
}

function readOnTime(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  const on = f.instOnTime + f.billsOnTime;
  const total = on + f.instLate + f.billsLate;
  if (total === 0)
    return 'Sem dados: nenhuma parcela de empréstimo nem fatura de cartão com pagamento registrado venceu no período.';
  return {
    value: on / total,
    text: of(on, total, 'em dia'),
    computation: `Parcelas de empréstimos (${f.instOnTime} em dia, ${f.instLate} em atraso) e faturas de cartão (${f.billsOnTime} pagas inteiras, ${f.billsLate} não) vencidas em ${periodLabel(f)}, com folga de ${GRACE_DAYS} dias. Parcela não conciliada depois do vencimento conta como em atraso.`,
    amounts: [],
  };
}

function readBudget(f: Facts): Read {
  if (f.budgetMonths === 0)
    return `Sem dados: nenhum mês de ${periodLabel(f)} tem orçamento de despesas.`;
  return {
    value: f.budgetWithin / f.budgetMonths,
    text: of(f.budgetWithin, f.budgetMonths, 'meses'),
    computation: `Meses com despesas até o total previsto ÷ meses com orçamento em ${periodLabel(f)} (base ${basisLabel(f)}).`,
    amounts: [],
  };
}

function readPositive(f: Facts): Read {
  const need = needMonths(f);
  if (need) return need;
  const active = f.flows.filter((m) => m.income !== 0 || m.expense !== 0);
  if (active.length === 0) return 'Sem dados: nenhum movimento no período.';
  const pos = active.filter((m) => m.net >= 0).length;
  return {
    value: pos / active.length,
    text: of(pos, active.length, 'meses'),
    computation: `Meses em que as receitas cobriram as despesas ÷ meses com movimento, em ${periodLabel(f)} (base ${basisLabel(f)}).`,
    amounts: [],
  };
}

// ── Indicadores ──────────────────────────────────────────────────

const ESSENTIAL_DATA =
  'Registre ou importe as receitas e despesas de pelo menos 3 meses completos.';

const SAVINGS: Spec = {
  key: 'taxa_poupanca',
  label: 'Taxa de poupança',
  unit: 'percent',
  target: '≥ 20% da renda',
  targetValue: 0.2,
  explanation: 'Quanto da renda sobra depois das despesas, nos últimos 12 meses.',
  advice: [
    'Pague-se primeiro: programe uma transferência para a poupança ou a corretora no dia do salário.',
    'Reveja as três maiores categorias de gasto em Relatórios › Composição e ponha limites no orçamento.',
    'Corte assinaturas e contas fixas que não usa (Relatórios › Recorrentes).',
  ],
  noData: ESSENTIAL_DATA,
};
const REGULAR: Spec = {
  key: 'poupanca_regular',
  label: 'Poupança regular',
  unit: 'ratio',
  target: '10% ou mais em todo mês',
  targetValue: 1,
  explanation: 'Em quantos meses foi possível guardar ao menos 10% da renda.',
  advice: [
    'Automatize um valor fixo de poupança todo mês, mesmo que pequeno.',
    'Provisione ao longo do ano os gastos sazonais (IPVA, IPTU, material escolar) para eles não zerarem a sobra.',
    'Nos meses mais fracos, veja em Relatórios › Comparação o que mudou em relação a um mês bom.',
  ],
  noData: ESSENTIAL_DATA,
};
const RESERVE: Spec = {
  key: 'reserva',
  label: 'Reserva de emergência',
  unit: 'months',
  target: '≥ 6 meses de gastos essenciais',
  targetValue: 6,
  explanation: 'Por quantos meses a reserva cobre os gastos essenciais sem nenhuma renda.',
  advice: [
    'Defina (ou reforce) a meta de reserva de emergência em 05 · Orçamento › Metas, com alvo de 6 meses de gastos essenciais.',
    'Guarde a reserva em aplicação de liquidez diária e baixo risco (Tesouro Selic ou CDB com liquidez diária).',
    'Direcione para a reserva extras como 13º, restituição do IR e PLR até chegar ao alvo.',
  ],
  noData:
    'Marque as categorias essenciais em Ajustes › Categorias e registre ao menos 3 meses de gastos.',
};
const DEBT: Spec = {
  key: 'comprometimento',
  label: 'Renda comprometida com parcelas',
  unit: 'percent',
  target: '≤ 15% da renda',
  targetValue: 0.15,
  explanation: 'Parte da renda que vai para parcelas de empréstimos e financiamentos.',
  advice: [
    'Evite novas parcelas até o comprometimento ficar abaixo de 15% da renda.',
    'Simule portabilidade ou amortização extraordinária do contrato mais caro em 07 · Dívidas.',
    'Quite primeiro as dívidas de juros mais altos (cartão, cheque especial, pessoal).',
  ],
  noData:
    'Registre as receitas de pelo menos 3 meses completos e cadastre os contratos em 07 · Dívidas.',
};
const HOUSING: Spec = {
  key: 'moradia',
  label: 'Custo da moradia',
  unit: 'percent',
  target: '≤ 30% da renda',
  targetValue: 0.3,
  explanation:
    'Parte da renda que vai para morar: aluguel ou financiamento, condomínio e contas da casa.',
  advice: [
    'Moradia acima de 30% da renda aperta o resto do orçamento: reveja contas de consumo, condomínio e seguros.',
    'Na renovação do aluguel ou num financiamento novo, mire num custo de até 30% da renda.',
    'Se o financiamento pesa, simule amortizar com o FGTS ou portar para uma taxa menor em 07 · Dívidas.',
  ],
  noData:
    'Lance os gastos da casa na categoria Moradia (ou cadastre o financiamento em 07 · Dívidas).',
};
const CREDIT: Spec = {
  key: 'credito_caro',
  label: 'Rotativo e cheque especial',
  unit: 'count',
  target: 'nenhum mês',
  targetValue: 0,
  explanation: 'Meses em que se usou crédito caro: cheque especial ou o rotativo do cartão.',
  advice: [
    'Pague a fatura do cartão inteira: o rotativo cobra os juros mais altos do mercado.',
    'Se faltar dinheiro, troque o rotativo ou o cheque especial por um crédito mais barato (consignado, por exemplo).',
    'Cadastre o limite e as taxas do cheque especial na conta, em 03 · Contas, para ver o custo estimado.',
  ],
  noData: 'Registre os pagamentos de fatura do cartão e os lançamentos da conta corrente.',
};
const NET_WORTH: Spec = {
  key: 'patrimonio',
  label: 'Evolução do patrimônio',
  unit: 'percent',
  target: 'crescer ≥ 5% em 12 meses',
  targetValue: 0.05,
  explanation: 'Quanto o patrimônio líquido cresceu (ou caiu) nos últimos 12 meses.',
  advice: [
    'Direcione parte da sobra de todo mês para investimentos ou para amortizar dívidas.',
    'Atualize o valor dos bens e as cotações da carteira para o patrimônio refletir a realidade.',
    'Reveja gastos grandes do ano em Relatórios › Tendência: eles explicam a queda.',
  ],
  noData: 'Cadastre as contas com o saldo inicial e mantenha alguns meses de lançamentos.',
};
const ON_TIME: Spec = {
  key: 'contas_em_dia',
  label: 'Contas em dia',
  unit: 'ratio',
  target: 'todas pagas até o vencimento',
  targetValue: 1,
  explanation: 'Parcelas de empréstimos e faturas de cartão pagas inteiras até o vencimento.',
  advice: [
    'Ative o débito automático das parcelas e da fatura do cartão.',
    'Concilie as parcelas em 07 · Dívidas: as pagas e não conciliadas contam como em atraso.',
    'Use os Próximos vencimentos do painel para se programar.',
  ],
  noData: 'Cadastre os contratos em 07 · Dívidas e registre os pagamentos de fatura do cartão.',
};
const BUDGET: Spec = {
  key: 'orcamento',
  label: 'Gastos dentro do orçamento',
  unit: 'ratio',
  target: 'todos os meses',
  targetValue: 1,
  explanation: 'Em quantos meses as despesas ficaram dentro do total previsto no orçamento.',
  advice: [
    'Reveja o orçamento dos meses em que estourou e ajuste os limites à realidade.',
    'Acompanhe o ritmo do gasto no card de orçamento do painel ao longo do mês.',
    'Deixe uma linha para gastos imprevistos no orçamento.',
  ],
  noData: 'Monte um orçamento mensal em 05 · Orçamento (copie do mês anterior ou use a sugestão).',
};
const POSITIVE: Spec = {
  key: 'meses_no_azul',
  label: 'Meses no azul',
  unit: 'ratio',
  target: 'todos os meses',
  targetValue: 1,
  explanation: 'Em quantos meses as receitas cobriram as despesas.',
  advice: [
    'Nos meses no vermelho, veja em Relatórios › Comparação o que mudou em relação a um mês no azul.',
    'Mantenha um colchão para gastos irregulares, para um mês ruim não virar dívida.',
    'Planeje as despesas anuais (seguros, impostos, viagens) como parcelas mensais no orçamento.',
  ],
  noData: ESSENTIAL_DATA,
};

// ── Modelos ──────────────────────────────────────────────────────

const bands = (list: [string, string, number, number][]): HealthBand[] =>
  list.map(([key, label, min, max]) => ({ key, label, min, max }));

const ISF_BANDS = bands([
  ['ruim', 'Ruim', 0, 36],
  ['muito_baixa', 'Muito baixa', 37, 49],
  ['baixa', 'Baixa', 50, 56],
  ['ok', 'Ok', 57, 60],
  ['boa', 'Boa', 61, 68],
  ['muito_boa', 'Muito boa', 69, 82],
  ['otima', 'Ótima', 83, 100],
]);
const RATIO_BANDS = bands([
  ['critica', 'Crítica', 0, 39],
  ['fragil', 'Frágil', 40, 59],
  ['estavel', 'Estável', 60, 79],
  ['saudavel', 'Saudável', 80, 100],
]);

const DIMS = [
  {
    key: 'comportamento',
    label: 'Comportamento',
    description: 'Pagar as contas em dia, gastar dentro do orçamento e fechar os meses no azul.',
  },
  {
    key: 'habilidade',
    label: 'Habilidade',
    description: 'Conseguir poupar parte da renda, com regularidade.',
  },
  {
    key: 'liberdade',
    label: 'Liberdade',
    description:
      'Quanto da renda está livre de parcelas e de crédito caro (rotativo e cheque especial).',
  },
  {
    key: 'seguranca',
    label: 'Segurança',
    description: 'Reserva para imprevistos e patrimônio que cresce.',
  },
];

function scoreStatus(s: number | null): HealthStatus {
  if (s == null) return 'unknown';
  const r = Math.round(s);
  return r >= 69 ? 'good' : r >= 57 ? 'ok' : 'bad';
}

function insufficient(f: Facts): string {
  return enough(f)
    ? 'Dados insuficientes: poucos indicadores puderam ser medidos.'
    : `Dados insuficientes: são precisos ao menos ${MIN_MONTHS} meses completos de lançamentos (há ${n(f)}).`;
}

function commonNotes(f: Facts, notes: string[]): void {
  if (!enough(f))
    notes.push(
      `Com menos de ${MIN_MONTHS} meses de histórico, os indicadores ficam sem dados em vez de penalizar.`,
    );
  if (f.missing.size)
    notes.push(
      `Sem câmbio para ${[...f.missing].sort().join(', ')}: esses valores ficaram fora das contas.`,
    );
}

function base(f: Facts, m: HealthModel) {
  const first = f.months[0];
  const last = f.months[f.months.length - 1];
  return {
    model: m,
    enabled: true,
    as_of: f.as_of,
    period: first && last ? { from: monthBounds(first).from, to: monthBounds(last).to } : null,
    months_analyzed: n(f),
    currency: BASE,
    missing_fx: [...f.missing].sort(),
    estimated: f.estimated,
    stale_inputs: [],
  };
}

function isf(f: Facts): HealthReport {
  const metrics = [
    build(ON_TIME, 'comportamento', 0.4, readOnTime(f), {
      points: [
        [0.7, 0],
        [0.9, 50],
        [1, 100],
      ],
      good: { atLeast: 1 },
      ok: { atLeast: 0.9 },
    }),
    build(BUDGET, 'comportamento', 0.3, readBudget(f), {
      points: [
        [0, 10],
        [0.5, 55],
        [0.83, 85],
        [1, 100],
      ],
      good: { atLeast: 0.83 },
      ok: { atLeast: 0.5 },
    }),
    build(POSITIVE, 'comportamento', 0.3, readPositive(f), {
      points: [
        [0.25, 0],
        [0.5, 40],
        [0.75, 75],
        [1, 100],
      ],
      good: { atLeast: 0.83 },
      ok: { atLeast: 0.6 },
    }),
    build(SAVINGS, 'habilidade', 0.6, readSavings(f), {
      points: [
        [-0.1, 0],
        [0, 30],
        [0.1, 60],
        [0.2, 85],
        [0.3, 100],
      ],
      good: { atLeast: 0.2 },
      ok: { atLeast: 0.1 },
    }),
    build(REGULAR, 'habilidade', 0.4, readRegular(f), {
      points: [
        [0, 0],
        [0.5, 50],
        [0.83, 85],
        [1, 100],
      ],
      good: { atLeast: 0.75 },
      ok: { atLeast: 0.5 },
    }),
    build(DEBT, 'liberdade', 0.6, readDebt(f), {
      points: [
        [0, 100],
        [0.15, 85],
        [0.3, 50],
        [0.4, 25],
        [0.6, 0],
      ],
      good: { atMost: 0.15 },
      ok: { atMost: 0.3 },
    }),
    build(CREDIT, 'liberdade', 0.4, readCredit(f), {
      points: [
        [0, 100],
        [1 / 12, 75],
        [0.25, 45],
        [0.5, 15],
        [1, 0],
      ],
      good: { atMost: 0 },
      ok: { atMost: 1 / 6 },
    }),
    build(RESERVE, 'seguranca', 0.6, readReserve(f), {
      points: [
        [0, 0],
        [1, 30],
        [3, 60],
        [6, 90],
        [12, 100],
      ],
      good: { atLeast: 6 },
      ok: { atLeast: 3 },
    }),
    build(NET_WORTH, 'seguranca', 0.4, readNetWorth(f), {
      points: [
        [-0.1, 0],
        [0, 40],
        [0.05, 70],
        [0.1, 100],
      ],
      good: { atLeast: 0.05 },
      ok: { atLeast: 0 },
    }),
  ];
  const dimensions: HealthDimension[] = DIMS.map((d) => {
    const mine = metrics.filter((m) => m.dimension === d.key);
    const score = weighted(mine.map((m) => [m.points, m.weight]))?.[0] ?? null;
    return {
      ...d,
      weight: 0.25,
      score,
      status: scoreStatus(score),
      metrics: mine.map((m) => m.key),
    };
  });
  const known = dimensions.filter((d) => d.score != null).length;
  const score =
    enough(f) && known >= 2
      ? (weighted(dimensions.map((d) => [d.score, d.weight]))?.[0] ?? null)
      : null;
  const band = score == null ? null : bandOf(ISF_BANDS, score);
  let summary = insufficient(f);
  if (score != null && band) {
    const cmp =
      score > NATIONAL_AVERAGE + 0.05
        ? 'acima da'
        : score < NATIONAL_AVERAGE - 0.05
          ? 'abaixo da'
          : 'igual à';
    const weakest = dimensions
      .filter((d) => d.score != null)
      .sort((a, b) => a.score! - b.score!)[0];
    summary = `Nota ${scoreBr(score)} (${band.label}), ${cmp} média nacional de ${scoreBr(NATIONAL_AVERAGE)}.${weakest ? ` Dimensão mais fraca: ${weakest.label}.` : ''}`;
  }
  const notes = [
    'Estimativa a partir dos seus dados (lançamentos, parcelas, faturas e saldos), no espírito do I-SFB da Febraban e do Banco Central; o índice oficial vem de um questionário.',
  ];
  commonNotes(f, notes);
  return {
    ...base(f, 'isf'),
    score,
    band,
    bands: ISF_BANDS,
    reference: {
      label: 'Média nacional do I-SFB em 2024',
      short_label: 'Média nacional',
      score: NATIONAL_AVERAGE,
      source_url: SOURCE_URL,
    },
    dimensions,
    metrics,
    summary,
    notes,
  };
}

function ratios(f: Facts): HealthReport {
  const metrics = [
    build(SAVINGS, null, 0.25, readSavings(f), {
      points: [
        [0, 0],
        [0.1, 60],
        [0.2, 100],
      ],
      good: { atLeast: 0.2 },
      ok: { atLeast: 0.1 },
    }),
    build(RESERVE, null, 0.25, readReserve(f), {
      points: [
        [0, 0],
        [3, 60],
        [6, 100],
      ],
      good: { atLeast: 6 },
      ok: { atLeast: 3 },
    }),
    build(DEBT, null, 0.2, readDebt(f), {
      points: [
        [0.15, 100],
        [0.3, 60],
        [0.4, 20],
        [0.5, 0],
      ],
      good: { atMost: 0.15 },
      ok: { atMost: 0.4 },
    }),
    build(HOUSING, null, 0.1, readHousing(f), {
      points: [
        [0.3, 100],
        [0.4, 50],
        [0.5, 0],
      ],
      good: { atMost: 0.3 },
      ok: { atMost: 0.4 },
    }),
    build(CREDIT, null, 0.1, readCredit(f), {
      points: [
        [0, 100],
        [1 / 12, 50],
        [0.25, 20],
        [0.5, 0],
      ],
      good: { atMost: 0 },
      ok: { atMost: -1 },
    }),
    build(NET_WORTH, null, 0.1, readNetWorth(f), {
      points: [
        [-0.1, 0],
        [0, 50],
        [0.05, 80],
        [0.1, 100],
      ],
      good: { atLeast: 0.05 },
      ok: { atLeast: 0 },
    }),
  ];
  const w = weighted(metrics.map((m) => [m.points, m.weight]));
  const score = w && enough(f) && w[1] >= 0.5 - 1e-9 ? w[0] : null;
  const band = score == null ? null : bandOf(RATIO_BANDS, score);
  let summary = insufficient(f);
  if (score != null && band) {
    const weakest = metrics
      .filter((m) => m.status !== 'good' && m.points != null)
      .sort((a, b) => a.points! - b.points!)[0];
    summary = `Nota ${scoreBr(score)} (${band.label}).${weakest ? ` Ponto mais fraco: ${weakest.label}.` : ' Todos os indicadores na meta.'}`;
  }
  const notes = [
    'Indicadores clássicos de finanças pessoais calculados com os seus dados; metas usuais da educação financeira.',
  ];
  commonNotes(f, notes);
  return {
    ...base(f, 'ratios'),
    score,
    band,
    bands: RATIO_BANDS,
    reference: null,
    dimensions: [],
    metrics,
    summary,
    notes,
  };
}

function score(asOfArg: string | undefined): HealthReport {
  const current = getLedger().today.slice(0, 7);
  const asOf = asOfArg ?? current;
  if (asOf > current) invalid('a saúde financeira só pode ser avaliada até o mês atual');
  if (model === 'off') {
    return {
      model,
      enabled: false,
      as_of: asOf,
      period: null,
      months_analyzed: 0,
      score: null,
      band: null,
      bands: [],
      reference: null,
      dimensions: [],
      metrics: [],
      summary: 'Análise de saúde financeira desativada em Ajustes.',
      notes: [],
      currency: BASE,
      missing_fx: [],
      estimated: false,
      stale_inputs: [],
    };
  }
  const f = loadFacts(asOf);
  return model === 'isf' ? isf(f) : ratios(f);
}

register({
  'health.score': (a) => score(optStr(a, 'as_of') ? monthArg(a, 'as_of') : undefined),
  'health.get_settings': () => settingsView(),
  'health.set_settings': (a) => {
    const m = a.model;
    if (m !== 'isf' && m !== 'ratios' && m !== 'off')
      invalid(`argumentos inválidos: unknown variant \`${String(m)}\``);
    model = m;
    return settingsView();
  },
});
