/**
 * Mock de `portfolio.*`: uma carteira fictícia realista (ações BR com vendas e um
 * desdobramento, FIIs com rendimentos mensais, ETF, Tesouro IPCA+, CDB 110% CDI, LCI,
 * ações e ETF nos EUA, cripto) e um motor simplificado com as mesmas regras do
 * núcleo (`portfolio/engine.rs`): preço médio ponderado com custos, renda fixa por
 * lotes (PEPS) na curva, eventos antes das operações da mesma data, XIRR nominal e
 * real, Modified Dietz mensal, rebalanceamento, agenda de dividendos e renda passiva.
 *
 * Usa ponto flutuante internamente (é um mock) e arredonda para centavos na saída.
 */
import type { AccountView } from '../generated/AccountView';
import type { AppSettings } from '../generated/AppSettings';
import type { Asset } from '../generated/Asset';
import type { AssetBrief } from '../generated/AssetBrief';
import type { BenchmarkSeries } from '../generated/BenchmarkSeries';
import type { BenchmarkTarget } from '../generated/BenchmarkTarget';
import type { Contribution } from '../generated/Contribution';
import type { ContributionReport } from '../generated/ContributionReport';
import type { BalanceDue } from '../generated/BalanceDue';
import type { BridgeAsset } from '../generated/BridgeAsset';
import type { BridgeLines } from '../generated/BridgeLines';
import type { AssetClass } from '../generated/AssetClass';
import type { ClassSummary } from '../generated/ClassSummary';
import type { ComeCotasRegime } from '../generated/ComeCotasRegime';
import type { FeeCost } from '../generated/FeeCost';
import type { ComeCotasRule } from '../generated/ComeCotasRule';
import type { CorporateAction } from '../generated/CorporateAction';
import type { CorporateActionKind } from '../generated/CorporateActionKind';
import type { Deflator } from '../generated/Deflator';
import type { DividendCalendarItem } from '../generated/DividendCalendarItem';
import type { IncomeEvent } from '../generated/IncomeEvent';
import type { IncomeKind } from '../generated/IncomeKind';
import type { IncomeLedger } from '../generated/IncomeLedger';
import type { IncomeLedgerEntry } from '../generated/IncomeLedgerEntry';
import type { IncomeLedgerPeriod } from '../generated/IncomeLedgerPeriod';
import type { InvestmentPlan } from '../generated/InvestmentPlan';
import type { InvestmentPlanView } from '../generated/InvestmentPlanView';
import type { PlanDue } from '../generated/PlanDue';
import type { PassiveIncome } from '../generated/PassiveIncome';
import type { PendingIncome } from '../generated/PendingIncome';
import type { Performance } from '../generated/Performance';
import type { PerformanceBridge } from '../generated/PerformanceBridge';
import type { PortfolioSummary } from '../generated/PortfolioSummary';
import type { PortfolioRisk } from '../generated/PortfolioRisk';
import type { RiskMetrics } from '../generated/RiskMetrics';
import type { PositionView } from '../generated/PositionView';
import type { QuickIncomeLine } from '../generated/QuickIncomeLine';
import type { QuickIncomeReport } from '../generated/QuickIncomeReport';
import type { PriceOn } from '../generated/PriceOn';
import type { Rebalance } from '../generated/Rebalance';
import type { ReturnRisk } from '../generated/ReturnRisk';
import type { CostMethod } from '../generated/CostMethod';
import type { CostMethodState } from '../generated/CostMethodState';
import type { SaleRow } from '../generated/SaleRow';
import type { SalesGroup } from '../generated/SalesGroup';
import type { SalesReport } from '../generated/SalesReport';
import type { SalesTotals } from '../generated/SalesTotals';
import type { Target } from '../generated/Target';
import type { Trade } from '../generated/Trade';
import type { TradeSideTotals } from '../generated/TradeSideTotals';
import type { TradeSummary } from '../generated/TradeSummary';
import type { TradeSide } from '../generated/TradeSide';
import type { TwrMethod } from '../generated/TwrMethod';
import type { Valuation } from '../generated/Valuation';
import type { WatchlistItem } from '../generated/WatchlistItem';
import {
  extraMockHandler,
  mockFail,
  onMockReset,
  registerMockHandlers,
  requireUnlocked,
} from '../mockRegistry';
import {
  allAssets,
  classLabel,
  closeOn,
  getAsset,
  hasAsset,
  setDeleteHook,
  setHeldProbe,
  setOpenProbe,
  setUsageProbe,
} from './assets';
import {
  concentrationAlerts,
  concentrationReport,
  concentrationRoom,
  setConcentrationLimit,
} from './concentration';
import { compareFixedIncome, simulateFixedIncomeSale } from './fixedIncomeCalc';
import {
  addDays,
  addMonths,
  businessDays,
  cdiRates,
  cents,
  daysBetween,
  dec,
  deflatorFactor,
  fxToBrl,
  isBusinessDay,
  mockToday,
  monthEnd,
  monthOf,
  monthStart,
  previousBusinessDay,
  SERIES,
  seriesPoints,
  txt,
} from './marketSim';

// ── Contas (o mock de contas, se existir, tem prioridade) ────────

const ACCOUNTS: {
  id: string;
  name: string;
  kind: AccountView['kind'];
  currency: string;
  institution: string;
}[] = [
  { id: 'acc-xp', name: 'XP Investimentos', kind: 'brokerage', currency: 'BRL', institution: 'XP' },
  {
    id: 'acc-inter',
    name: 'Inter Invest',
    kind: 'brokerage',
    currency: 'BRL',
    institution: 'Banco Inter',
  },
  {
    id: 'acc-avenue',
    name: 'Avenue (EUA)',
    kind: 'brokerage',
    currency: 'USD',
    institution: 'Avenue',
  },
  {
    id: 'acc-binance',
    name: 'Binance',
    kind: 'brokerage',
    currency: 'USD',
    institution: 'Binance',
  },
];

function fallbackAccounts(): AccountView[] {
  return ACCOUNTS.map((a, i) => ({
    id: a.id,
    name: a.name,
    kind: a.kind,
    currency: a.currency,
    institution_id: null,
    institution_name: a.institution,
    member_ids: [],
    opening_balance: 0,
    opening_date: '2024-01-01',
    include_in_net_worth: true,
    archived: false,
    color: null,
    notes: null,
    sort_order: i,
    card: null,
    overdraft: null,
    overdraft_status: null,
    branch: null,
    account_number: null,
    retirement_plan: null,
    retirement_coverage: null,
    balance: 0,
    projected_balance: 0,
    reconciled_balance: 0,
    balance_base: 0,
    fx_estimated: false,
    last_txn_date: null,
    last_import_at: null,
    staleness_days: null,
    balance_status: 'fresh',
    missed_month: null,
    available_credit: null,
  }));
}

/** Contas conhecidas: as do mock de contas (outra equipe) ou as daqui. */
function accounts(): AccountView[] {
  const external = extraMockHandler('accounts.list');
  if (external && external !== accountsHandler) {
    try {
      return external({}) as AccountView[];
    } catch {
      return fallbackAccounts();
    }
  }
  return fallbackAccounts();
}

function account(id: string): AccountView {
  const a = accounts().find((x) => x.id === id) ?? fallbackAccounts().find((x) => x.id === id);
  if (!a) mockFail('not_found', `Não encontrado: conta ${id}.`);
  return a;
}

const accountsHandler = () => {
  requireUnlocked();
  return fallbackAccounts();
};

// ── Registros ────────────────────────────────────────────────────

interface TradeRec {
  id: string;
  asset_id: string;
  account_id: string;
  date: string;
  side: TradeSide;
  quantity: string;
  price: string;
  fees: number;
  taxes: number;
  currency: string;
  fx_rate: string | null;
  day_trade: boolean;
  txn_id: string | null;
  note_number: string | null;
  notes: string | null;
  seq: number;
}

interface DividendRec {
  asset_id: string;
  kind: IncomeKind;
  ex_date: string;
  pay_date: string;
  amount_per_share: number;
  currency: string;
  estimated: boolean;
  source: string;
}

interface State {
  trades: TradeRec[];
  actions: CorporateAction[];
  incomes: IncomeEvent[];
  targets: Target[];
  watch: {
    id: string;
    asset_id: string;
    target_price: string | null;
    alert_above: string | null;
    alert_below: string | null;
    notes: string | null;
  }[];
  seq: number;
}

/** Operações do seed: preço = fechamento simulado do dia (custo de emolumentos na B3). */
const SEED_TRADES: [string, string, string, TradeSide, string][] = [
  ['ast-itub4', 'acc-xp', '2024-01-22', 'buy', '300'],
  ['ast-bbas3', 'acc-xp', '2024-01-15', 'buy', '100'],
  ['ast-hglg11', 'acc-xp', '2024-01-10', 'buy', '40'],
  ['ast-mxrf11', 'acc-inter', '2024-02-05', 'buy', '1000'],
  ['ast-petr4', 'acc-xp', '2024-02-15', 'buy', '200'],
  ['ast-mglu3', 'acc-inter', '2024-03-05', 'buy', '300'],
  ['ast-btc', 'acc-binance', '2024-03-11', 'buy', '0.035'],
  ['ast-ipca35', 'acc-xp', '2024-03-20', 'buy', '3.5'],
  ['ast-knri11', 'acc-xp', '2024-04-08', 'buy', '50'],
  ['ast-bova11', 'acc-xp', '2024-05-13', 'buy', '80'],
  ['ast-wege3', 'acc-inter', '2024-06-12', 'buy', '100'],
  ['ast-aapl', 'acc-avenue', '2024-07-01', 'buy', '15'],
  ['ast-hglg11', 'acc-xp', '2024-08-12', 'buy', '20'],
  ['ast-petr4', 'acc-xp', '2024-09-10', 'buy', '100'],
  ['ast-msft', 'acc-avenue', '2024-11-04', 'buy', '8'],
  ['ast-mglu3', 'acc-inter', '2025-01-08', 'sell', '300'],
  ['ast-voo', 'acc-avenue', '2025-01-13', 'buy', '6.25'],
  ['ast-itub4', 'acc-xp', '2025-02-11', 'buy', '100'],
  ['ast-bbas3', 'acc-xp', '2025-03-14', 'buy', '100'],
  ['ast-mxrf11', 'acc-inter', '2025-04-07', 'buy', '500'],
  ['ast-aapl', 'acc-avenue', '2025-04-08', 'buy', '10.5'],
  ['ast-petr4', 'acc-xp', '2025-05-20', 'sell', '100'],
  ['ast-hglg11', 'acc-xp', '2025-06-09', 'buy', '15'],
  ['ast-ipca35', 'acc-xp', '2025-07-15', 'buy', '2.1'],
  ['ast-wege3', 'acc-inter', '2025-08-05', 'buy', '50'],
  ['ast-bova11', 'acc-xp', '2025-09-15', 'buy', '40'],
  ['ast-knri11', 'acc-xp', '2025-10-06', 'buy', '20'],
  ['ast-btc', 'acc-binance', '2025-11-20', 'buy', '0.012'],
  ['ast-bbas3', 'acc-xp', '2026-01-20', 'sell', '50'],
  ['ast-msft', 'acc-avenue', '2026-02-03', 'buy', '4'],
  ['ast-petr4', 'acc-xp', '2026-03-10', 'buy', '100'],
  ['ast-btc', 'acc-binance', '2026-04-02', 'sell', '0.007'],
  ['ast-voo', 'acc-avenue', '2026-06-01', 'buy', '3'],
];

/** Renda fixa: quantidade × PU de emissão. */
const SEED_FIXED: [string, string, string, string, string][] = [
  ['ast-cdb-inter', 'acc-inter', '2025-02-10', '20', '1000'],
  ['ast-lci-itau', 'acc-xp', '2025-06-02', '15', '1000'],
  // Fundo sem fonte automática de cota: fica avaliado pelo custo (valuation "missing").
  ['ast-xpmacro', 'acc-xp', '2025-03-03', '1200', '1.8532'],
];

/** Proventos por cota (na moeda do ativo) e calendário de cada pagador. */
const PAYERS: {
  asset_id: string;
  kind: IncomeKind;
  months: number[] | 'all';
  amount: number;
  exDay: number;
  payDay: number;
}[] = [
  { asset_id: 'ast-hglg11', kind: 'fii_income', months: 'all', amount: 1.1, exDay: 31, payDay: 14 },
  { asset_id: 'ast-knri11', kind: 'fii_income', months: 'all', amount: 1.0, exDay: 31, payDay: 15 },
  { asset_id: 'ast-mxrf11', kind: 'fii_income', months: 'all', amount: 0.1, exDay: 31, payDay: 14 },
  {
    asset_id: 'ast-xpml11',
    kind: 'fii_income',
    months: 'all',
    amount: 0.92,
    exDay: 31,
    payDay: 15,
  },
  { asset_id: 'ast-itub4', kind: 'jcp', months: 'all', amount: 0.02, exDay: 1, payDay: 1 },
  { asset_id: 'ast-itub4', kind: 'dividend', months: [2, 8], amount: 0.45, exDay: 20, payDay: 30 },
  {
    asset_id: 'ast-petr4',
    kind: 'dividend',
    months: [2, 5, 8, 11],
    amount: 0.62,
    exDay: 22,
    payDay: 20,
  },
  {
    asset_id: 'ast-bbas3',
    kind: 'jcp',
    months: [3, 6, 9, 12],
    amount: 0.21,
    exDay: 11,
    payDay: 28,
  },
  { asset_id: 'ast-wege3', kind: 'jcp', months: [3, 9], amount: 0.18, exDay: 21, payDay: 10 },
  {
    asset_id: 'ast-taee11',
    kind: 'dividend',
    months: [3, 8, 11],
    amount: 0.95,
    exDay: 18,
    payDay: 28,
  },
  { asset_id: 'ast-vale3', kind: 'dividend', months: [3, 8], amount: 2.14, exDay: 11, payDay: 25 },
  { asset_id: 'ast-bova11', kind: 'dividend', months: [6], amount: 0.38, exDay: 2, payDay: 12 },
  {
    asset_id: 'ast-aapl',
    kind: 'dividend',
    months: [2, 5, 8, 11],
    amount: 0.26,
    exDay: 10,
    payDay: 14,
  },
  {
    asset_id: 'ast-msft',
    kind: 'dividend',
    months: [2, 5, 8, 11],
    amount: 0.83,
    exDay: 20,
    payDay: 12,
  },
  {
    asset_id: 'ast-voo',
    kind: 'dividend',
    months: [3, 6, 9, 12],
    amount: 1.74,
    exDay: 26,
    payDay: 30,
  },
];

let state: State = { trades: [], actions: [], incomes: [], targets: [], watch: [], seq: 0 };
let calendarCache: { to: string; rows: DividendRec[] } | null = null;
state = seed();

// Aportes programados (`portfolio/plans.rs`): planos e as datas já decididas.
let plans: InvestmentPlan[] = [];
let planRuns = new Map<string, 'confirmed' | 'skipped'>();

onMockReset(() => {
  calendarCache = null;
  state = seed();
  plans = [];
  planRuns = new Map();
});

function planDueDates(p: InvestmentPlan, from: string, to: string): string[] {
  const out: string[] = [];
  let m = monthOf(p.start_date);
  while (`${m}-01` <= to) {
    const d = `${m}-${String(p.day_of_month).padStart(2, '0')}`;
    if (d >= p.start_date && d >= from && d <= to && (!p.end_date || d <= p.end_date)) out.push(d);
    m = nextMonth(m);
  }
  return out;
}

function planPendingDates(p: InvestmentPlan): string[] {
  if (!p.active) return [];
  const today = mockToday();
  return planDueDates(p, addMonths(today, -12), today).filter((d) => !planRuns.has(`${p.id}|${d}`));
}

function planView(p: InvestmentPlan): InvestmentPlanView {
  const a = getAsset(p.asset_id);
  const today = mockToday();
  const next = p.active ? planDueDates(p, addDays(today, 1), addMonths(today, 2))[0] : undefined;
  return {
    ...p,
    asset_name: a.name,
    ticker: a.ticker,
    account_name: account(p.account_id).name,
    currency: a.currency,
    goal_name: null,
    next_due: next ?? null,
    pending: planPendingDates(p).length,
    confirmed: [...planRuns.entries()].filter(
      ([k, v]) => k.startsWith(`${p.id}|`) && v === 'confirmed',
    ).length,
  };
}

function planDues(): PlanDue[] {
  const out: PlanDue[] = [];
  for (const p of plans) {
    const a = getAsset(p.asset_id);
    for (const d of planPendingDates(p)) {
      const c = closeOn(p.asset_id, d);
      const whole = ['stock_br', 'fii', 'etf_br', 'bdr'].includes(a.class);
      const raw = c && c.close > 0 ? p.amount / 100 / c.close : 0;
      const q = whole ? Math.floor(raw) : Math.floor(raw * 1e6) / 1e6;
      out.push({
        plan_id: p.id,
        plan_name: p.name,
        asset_id: p.asset_id,
        asset_name: a.name,
        ticker: a.ticker,
        account_id: p.account_id,
        account_name: account(p.account_id).name,
        due_date: d,
        amount: p.amount,
        currency: a.currency,
        price: c ? c.price.close : null,
        price_date: c ? c.price.date : null,
        quantity: q > 0 ? String(q) : null,
        goal_id: p.goal_id,
      });
    }
  }
  return out.sort((x, y) => x.due_date.localeCompare(y.due_date));
}

function planCheckDue(planId: string, due: string): InvestmentPlan {
  const p = plans.find((x) => x.id === planId);
  if (!p) mockFail('not_found', `não encontrado: aporte programado ${planId}`);
  if (!planDueDates(p, due, due).includes(due) || planRuns.has(`${planId}|${due}`))
    mockFail('invalid', 'essa data não é um aporte pendente deste plano');
  return p;
}

function clampDay(month: string, day: number): string {
  const last = Number(monthEnd(month).slice(8, 10));
  return `${month}-${String(Math.min(day, last)).padStart(2, '0')}`;
}

function nextMonth(month: string): string {
  return monthOf(addMonths(monthStart(month), 1));
}

/** Eventos de dividendos (`dividend_event`): passados e até 4 meses à frente. */
function buildCalendar(): DividendRec[] {
  const today = mockToday();
  const until = monthOf(addMonths(today, 4));
  const out: DividendRec[] = [];
  for (const p of PAYERS) {
    const asset = getAsset(p.asset_id);
    for (let m = '2024-01'; m <= until; m = nextMonth(m)) {
      const mm = Number(m.slice(5, 7));
      if (p.months !== 'all' && !p.months.includes(mm)) continue;
      const ex = previousBusinessDay(clampDay(m, p.exDay));
      const payMonth = p.payDay <= p.exDay ? nextMonth(m) : m;
      let pay = clampDay(payMonth, p.payDay);
      while (!isBusinessDay(pay)) pay = addDays(pay, 1);
      // Anunciado até ~3 semanas antes da data-com; depois disso, projeção do radar.
      const estimated = ex > addDays(today, 21);
      const wobble = 1 + (((mm * 7 + Number(m.slice(2, 4))) % 5) - 2) * 0.015;
      out.push({
        asset_id: p.asset_id,
        kind: p.kind,
        ex_date: ex,
        pay_date: pay,
        amount_per_share: Number(
          (p.amount * (p.kind === 'fii_income' ? wobble : 1)).toFixed(p.amount < 0.1 ? 4 : 3),
        ),
        currency: asset.currency,
        estimated,
        source: estimated ? 'radar' : 'yahoo',
      });
    }
  }
  return out.sort((a, b) => a.pay_date.localeCompare(b.pay_date));
}

function calendar(): DividendRec[] {
  const today = mockToday();
  if (!calendarCache || calendarCache.to !== today)
    calendarCache = { to: today, rows: buildCalendar() };
  return calendarCache.rows;
}

function seed(): State {
  const s: State = { trades: [], actions: [], incomes: [], targets: [], watch: [], seq: 0 };
  const priceOf = (asset: Asset, date: string): number => closeOn(asset.id, date)?.close ?? 0;
  for (const [asset_id, account_id, date, side, qty] of SEED_TRADES) {
    const asset = getAsset(asset_id);
    const price = priceOf(asset, date);
    const gross = Number(qty) * price;
    const fees =
      asset.currency === 'BRL'
        ? cents(gross * 0.0003)
        : asset.class === 'crypto'
          ? cents(gross * 0.001)
          : 0;
    s.seq += 1;
    s.trades.push({
      id: `trd-${s.seq}`,
      asset_id,
      account_id,
      date,
      side,
      quantity: qty,
      price: dec(price, asset.class === 'crypto' ? 2 : 2),
      fees,
      taxes: side === 'sell' && asset.currency === 'BRL' ? cents(gross * 0.00005) : 0,
      currency: asset.currency,
      fx_rate: asset.currency === 'BRL' ? null : dec(fxToBrl(asset.currency, date)?.rate ?? 5, 4),
      day_trade: false,
      txn_id: `txn-trd-${s.seq}`,
      note_number: asset.currency === 'BRL' ? String(48_100 + s.seq * 37) : null,
      notes: null,
      seq: s.seq,
    });
  }
  for (const [asset_id, account_id, date, qty, price] of SEED_FIXED) {
    s.seq += 1;
    s.trades.push({
      id: `trd-${s.seq}`,
      asset_id,
      account_id,
      date,
      side: 'buy',
      quantity: qty,
      price,
      fees: 0,
      taxes: 0,
      currency: 'BRL',
      fx_rate: null,
      day_trade: false,
      txn_id: `txn-trd-${s.seq}`,
      note_number: null,
      notes: null,
      seq: s.seq,
    });
  }
  s.actions.push({
    id: 'act-bbas3-split',
    asset_id: 'ast-bbas3',
    date: '2024-04-16',
    kind: 'split',
    factor: '2',
    unit_cost: null,
    amount: null,
    new_asset_id: null,
    notes: 'Desdobramento 1:2 aprovado em AGE',
    source: 'yahoo',
    account_id: null,
    quantity: null,
  });
  s.targets = [
    ['stock_br', '22'],
    ['fii', '14'],
    ['etf_br', '8'],
    ['fixed_income', '24'],
    ['treasury_br', '10'],
    ['stock_us', '12'],
    ['etf_us', '5'],
    ['crypto', '5'],
  ].map(([key, pct], i) => ({
    id: `tgt-${i}`,
    scope: 'class',
    key: key!,
    label: classLabel(key as AssetClass),
    target_pct: pct!,
    tolerance_pct: '5',
  }));
  s.watch = [
    {
      id: 'wl-1',
      asset_id: 'ast-vale3',
      target_price: '64',
      alert_above: null,
      alert_below: '55',
      notes: 'Esperar dividendos de agosto',
    },
    {
      id: 'wl-2',
      asset_id: 'ast-taee11',
      target_price: '38.5',
      alert_above: '37',
      alert_below: null,
      notes: null,
    },
    {
      id: 'wl-3',
      asset_id: 'ast-xpml11',
      target_price: '112',
      alert_above: null,
      alert_below: '100',
      notes: 'P/VP abaixo de 0,95',
    },
    {
      id: 'wl-4',
      asset_id: 'ast-nvda',
      target_price: '150',
      alert_above: null,
      alert_below: '160',
      notes: null,
    },
  ];
  state = s;
  // Proventos recebidos: os eventos passados em que a família tinha a cota na data-com.
  const today = mockToday();
  for (const ev of buildCalendar()) {
    if (ev.pay_date > today) continue;
    const asset = getAsset(ev.asset_id);
    const holder = s.trades.find((t) => t.asset_id === ev.asset_id)?.account_id;
    if (!holder) continue;
    const qty = replay(ev.asset_id, null, ev.ex_date).qty;
    if (qty <= 0) continue;
    const gross = cents(qty * ev.amount_per_share);
    if (gross <= 0) continue;
    const tax = defaultTax(ev.kind, asset.class, gross, creditOn(ev.ex_date, ev.pay_date));
    s.seq += 1;
    s.incomes.push({
      id: `inc-${s.seq}`,
      asset_id: ev.asset_id,
      ticker: asset.ticker,
      asset_name: asset.name,
      account_id: holder,
      kind: ev.kind,
      ex_date: ev.ex_date,
      pay_date: ev.pay_date,
      quantity: dec(qty),
      gross,
      tax_withheld: tax,
      net: gross - tax,
      currency: asset.currency,
      fx_rate:
        asset.currency === 'BRL' ? null : dec(fxToBrl(asset.currency, ev.pay_date)?.rate ?? 5, 4),
      status: 'received',
      txn_id: `txn-inc-${s.seq}`,
      source: 'manual',
    });
  }
  return s;
}

/** Como `jcp_rate` do núcleo: 15% até 2025, 17,5% desde 2026 (LC 224/2025), pela data do crédito. */
function jcpRate(on: string): number {
  return on >= '2026-01-01' ? 0.175 : 0.15;
}

/** Data do crédito (data-com) ou do pagamento, a primeira. */
function creditOn(ex: string | null | undefined, pay: string | null | undefined): string {
  const days = [ex, pay].filter((d): d is string => !!d).sort();
  return days[0] ?? mockToday();
}

function defaultTax(kind: IncomeKind, cls: AssetClass, gross: number, on: string): number {
  if (kind === 'jcp') return Math.round(gross * jcpRate(on));
  if (kind === 'dividend' && (cls === 'stock_us' || cls === 'etf_us' || cls === 'reit')) {
    return Math.round(gross * 0.3);
  }
  return 0;
}

setHeldProbe((id) => state.trades.some((t) => t.asset_id === id));
setOpenProbe(
  (id) =>
    state.trades
      .filter((t) => t.asset_id === id)
      .reduce((q, t) => q + (t.side === 'buy' ? 1 : -1) * Number(t.quantity), 0) > 1e-9,
);
setUsageProbe(
  (id) =>
    state.trades.some((t) => t.asset_id === id) || state.incomes.some((i) => i.asset_id === id),
);
setDeleteHook((id) => {
  state.watch = state.watch.filter((w) => w.asset_id !== id);
  state.actions = state.actions.filter((a) => a.asset_id !== id);
  state.targets = state.targets.filter((t) => !(t.scope === 'asset' && t.key === id));
});

// ── Motor ────────────────────────────────────────────────────────

interface Lot {
  qty: number;
  principal: number;
  principalBase: number;
  date: string;
}

interface Book {
  qty: number;
  cost: number;
  costBase: number;
  realized: number;
  realizedBase: number;
  lots: Lot[];
  estimated: boolean;
  warnings: string[];
}

function minorFactor(ccy: string): number {
  return ccy === 'JPY' ? 1 : 100;
}

function isCurve(a: Asset): boolean {
  return (
    a.class === 'fixed_income' ||
    (a.class === 'treasury_br' && a.fixed_income != null && !closeOn(a.id, mockToday()))
  );
}

function tradeFx(t: TradeRec): number {
  if (t.fx_rate) return Number(t.fx_rate);
  return fxToBrl(t.currency, t.date)?.rate ?? 0;
}

/** Um passo da reprodução visto de fora (livro de vendas), como `engine::Step`. */
interface Step {
  date: string;
  sell: TradeRec | null;
  qtySold: number;
  costBefore: number;
  costAfter: number;
  proceedsBase: number;
  lotsOut: [string, number][];
}

/** Reproduz operações e eventos até `until` (inclusive); eventos antes das operações do dia. */
function replay(
  assetId: string,
  accountIds: string[] | null,
  until: string,
  obs?: (s: Step) => void,
): Book {
  const asset = getAsset(assetId);
  const curve = asset.class === 'fixed_income';
  const b: Book = {
    qty: 0,
    cost: 0,
    costBase: 0,
    realized: 0,
    realizedBase: 0,
    lots: [],
    estimated: false,
    warnings: [],
  };
  const trades = state.trades
    .filter(
      (t) =>
        t.asset_id === assetId &&
        t.date <= until &&
        (!accountIds || accountIds.includes(t.account_id)),
    )
    .sort((a, b2) => a.date.localeCompare(b2.date) || a.seq - b2.seq);
  const actions = state.actions
    .filter((a) => a.asset_id === assetId && a.date <= until)
    .sort((a, b2) => a.date.localeCompare(b2.date));
  let ti = 0;
  let ai = 0;
  while (ti < trades.length || ai < actions.length) {
    const t = trades[ti];
    const a = actions[ai];
    if (a && (!t || a.date <= t.date)) {
      ai += 1;
      const before = b.costBase;
      const f = a.factor ? Number(a.factor) : null;
      if ((a.kind === 'split' || a.kind === 'reverse_split') && f) {
        b.qty *= f;
        for (const l of b.lots) l.qty *= f;
      } else if (a.kind === 'bonus' && f) {
        const extra = b.qty * f;
        const unit = a.unit_cost ? Number(a.unit_cost) : 0;
        b.qty += extra;
        b.cost += extra * unit;
        b.costBase += extra * unit;
      } else if (a.kind === 'capital_return' && a.amount) {
        b.cost = Math.max(0, b.cost - a.amount / 100);
        b.costBase = Math.max(0, b.costBase - a.amount / 100);
      } else if (a.kind === 'spin_off' || a.kind === 'subscription') {
        b.warnings.push('cisão/subscrição: registre a operação resultante manualmente');
      }
      obs?.({
        date: a.date,
        sell: null,
        qtySold: 0,
        costBefore: before,
        costAfter: b.costBase,
        proceedsBase: 0,
        lotsOut: [],
      });
      continue;
    }
    if (!t) break;
    ti += 1;
    const fx = asset.currency === 'BRL' ? 1 : tradeFx(t);
    if (!fx) {
      b.estimated = true;
      b.warnings.push(`sem câmbio ${asset.currency}/BRL em ${t.date}`);
    }
    const q = Number(t.quantity);
    const p = Number(t.price);
    const costs = (t.fees + t.taxes) / minorFactor(t.currency);
    const before = b.costBase;
    const lotsOut: [string, number][] = [];
    let soldQty = 0;
    let proceedsBase = 0;
    if (t.side === 'buy') {
      b.qty += q;
      b.cost += q * p + costs;
      b.costBase += (q * p + costs) * fx;
      if (curve)
        b.lots.push({
          qty: q,
          principal: q * p + costs,
          principalBase: (q * p + costs) * fx,
          date: t.date,
        });
    } else {
      const qty = Math.min(q, b.qty);
      if (qty < q) b.warnings.push(`venda maior que a posição em ${t.date}`);
      if (qty <= 0) continue;
      const proceeds = qty * p - costs;
      if (curve) {
        let left = qty;
        let out = 0;
        let outBase = 0;
        while (left > 1e-12 && b.lots.length) {
          const lot = b.lots[0]!;
          const take = Math.min(left, lot.qty);
          const share = take / lot.qty;
          out += lot.principal * share;
          outBase += lot.principalBase * share;
          lotsOut.push([lot.date, lot.principalBase * share]);
          lot.principal -= lot.principal * share;
          lot.principalBase -= lot.principalBase * share;
          lot.qty -= take;
          left -= take;
          if (lot.qty <= 1e-12) b.lots.shift();
        }
        b.qty -= qty;
        b.cost = Math.max(0, b.cost - out);
        b.costBase = Math.max(0, b.costBase - outBase);
        b.realized += proceeds - out;
        b.realizedBase += proceeds * fx - outBase;
        soldQty = qty;
        proceedsBase = proceeds * fx;
      } else {
        const avg = b.cost / b.qty;
        const avgBase = b.costBase / b.qty;
        b.cost -= avg * qty;
        b.costBase -= avgBase * qty;
        b.realized += proceeds - avg * qty;
        b.realizedBase += proceeds * fx - avgBase * qty;
        b.qty -= qty;
        soldQty = qty;
        proceedsBase = proceeds * fx;
      }
    }
    if (Math.abs(b.qty) < 1e-10) {
      b.qty = 0;
      b.cost = 0;
      b.costBase = 0;
    }
    obs?.({
      date: t.date,
      sell: t.side === 'sell' && soldQty > 0 ? t : null,
      qtySold: soldQty,
      costBefore: before,
      costAfter: b.costBase,
      proceedsBase,
      lotsOut,
    });
  }
  return b;
}

/** Livro de vendas, como `portfolio::sales` (camadas de custo por data de compra). */
function sales(a: Record<string, unknown>): SalesReport {
  const deflator = (typeof a.deflator === 'string' ? a.deflator : 'ipca') as Deflator;
  const to = typeof a.to === 'string' ? a.to : mockToday();
  const from = typeof a.from === 'string' ? a.from : null;
  const classes = Array.isArray(a.classes) ? (a.classes as AssetClass[]) : [];
  const accountIds = Array.isArray(a.account_ids) ? (a.account_ids as string[]) : [];
  const ids = [
    ...new Set(
      state.trades
        .filter(
          (t) =>
            t.side === 'sell' &&
            t.date <= to &&
            (!from || t.date >= from) &&
            (typeof a.asset_id !== 'string' || t.asset_id === a.asset_id),
        )
        .map((t) => t.asset_id),
    ),
  ];
  const rows: SaleRow[] = [];
  for (const id of ids) {
    const asset = getAsset(id);
    if (classes.length && !classes.includes(asset.class)) continue;
    let layers: [string, number][] = [];
    // Proventos da posição (como `portfolio::sales`): pela data-com, fora amortização.
    const events = state.incomes
      .filter((i) => i.asset_id === id && i.status === 'received' && i.kind !== 'amortization')
      .map((i) => ({ on: i.ex_date || i.pay_date, pay: i.pay_date, value: i.net }))
      .sort((x, y) => x.on.localeCompare(y.on) || x.pay.localeCompare(y.pay));
    let next = 0;
    let pool: [string, number][] = [];
    replay(id, null, to, (st) => {
      while (next < events.length && events[next]!.on < st.date) {
        pool.push([events[next]!.pay, events[next]!.value]);
        next++;
      }
      if (!st.sell) {
        const delta = st.costAfter - st.costBefore;
        if (delta > 1e-9) layers.push([st.date, delta]);
        else if (delta < -1e-9 && st.costBefore > 0) {
          const keep = st.costAfter / st.costBefore;
          layers = layers.map(([d, c]) => [d, c * keep]);
        }
        return;
      }
      const incShare =
        st.costBefore > 0 ? Math.min(1, (st.costBefore - st.costAfter) / st.costBefore) : 0;
      const income = pool.map(([d, v]) => [d, v * incShare] as [string, number]);
      pool = pool
        .map(([d, v]) => [d, v - v * incShare] as [string, number])
        .filter(([, v]) => v !== 0);
      let out: [string, number][];
      if (st.lotsOut.length) out = st.lotsOut;
      else {
        const share = st.costBefore > 0 ? (st.costBefore - st.costAfter) / st.costBefore : 0;
        out = layers.map(([d, c]) => [d, c * share]);
        layers = layers.map(([d, c]) => [d, c - c * share]);
      }
      const t = st.sell;
      if ((from && st.date < from) || (accountIds.length && !accountIds.includes(t.account_id)))
        return;
      const cost = cents(st.costBefore - st.costAfter);
      const proceeds = cents(st.proceedsBase);
      let real: number | null = 0;
      let estimated = false;
      let weight = 0;
      let days = 0;
      for (const [d, c] of out) {
        weight += c;
        days += c * daysBetween(d, st.date);
        const f = deflatorFactor(deflator, d, st.date);
        if (!f) real = null;
        else {
          if (real != null) real += c * f.factor;
          estimated ||= f.estimated;
        }
      }
      if (!out.length) real = st.costBefore - st.costAfter;
      const realCost = real == null ? null : cents(real);
      const incomeC = Math.round(income.reduce((acc, [, v]) => acc + v, 0));
      let realInc: number | null = 0;
      for (const [d, v] of income) {
        const f = deflatorFactor(deflator, d, st.date);
        if (!f) realInc = null;
        else if (realInc != null) realInc += v * f.factor;
      }
      const realIncome = realInc == null ? null : Math.round(realInc);
      const realTotal =
        realCost == null || realIncome == null ? null : proceeds - realCost + realIncome;
      rows.push({
        trade_id: t.id,
        date: st.date,
        asset_id: id,
        ticker: asset.ticker,
        name: asset.name,
        class: asset.class,
        account_id: t.account_id,
        account_name: account(t.account_id).name,
        quantity: fmtQty(st.qtySold),
        proceeds,
        cost,
        result: proceeds - cost,
        result_pct: cost > 0 ? (proceeds - cost) / cost : null,
        real_cost: realCost,
        real_result: realCost == null ? null : proceeds - realCost,
        real_pct: realCost != null && realCost > 0 ? (proceeds - realCost) / realCost : null,
        index_change: realCost != null && cost > 0 ? realCost / cost - 1 : null,
        holding_days: weight > 0 ? Math.round(days / weight) : null,
        estimated,
        income: incomeC,
        total_result: proceeds - cost + incomeC,
        total_pct: cost > 0 ? (proceeds - cost + incomeC) / cost : null,
        real_income: realIncome,
        real_total_result: realTotal,
        real_total_pct:
          realCost != null && realCost > 0 && realTotal != null ? realTotal / realCost : null,
        // Retorno anualizado aproximado (o núcleo usa a TIR com a data de cada compra).
        irr:
          weight > 0 && cost > 0 && Math.round(days / weight) > 0
            ? ((proceeds + incomeC) / cost) ** (365 / Math.round(days / weight)) - 1
            : null,
      });
    });
  }
  rows.sort((x, y) => y.date.localeCompare(x.date) || x.name.localeCompare(y.name));
  const empty = (): SalesTotals => ({
    count: 0,
    gains: 0,
    losses: 0,
    real_gains: 0,
    real_losses: 0,
    proceeds: 0,
    cost: 0,
    result: 0,
    real_cost: null,
    real_result: null,
    inflation_effect: null,
    missing: 0,
    income: 0,
    total_result: 0,
    real_total_result: null,
  });
  const addTo = (t: SalesTotals, r: SaleRow) => {
    t.count += 1;
    t.proceeds += r.proceeds;
    t.cost += r.cost;
    t.result += r.result;
    t.income += r.income;
    t.total_result += r.total_result;
    if (r.real_total_result != null)
      t.real_total_result = (t.real_total_result ?? 0) + r.real_total_result;
    if (r.result > 0) t.gains += 1;
    else if (r.result < 0) t.losses += 1;
    if (r.real_cost != null && r.real_result != null) {
      t.real_cost = (t.real_cost ?? 0) + r.real_cost;
      t.real_result = (t.real_result ?? 0) + r.real_result;
      t.inflation_effect = (t.inflation_effect ?? 0) + r.real_cost - r.cost;
      if (r.real_result > 0) t.real_gains += 1;
      else if (r.real_result < 0) t.real_losses += 1;
    } else t.missing += 1;
  };
  const group = (key: (r: SaleRow) => string): SalesGroup[] => {
    const m = new Map<string, SalesTotals>();
    for (const r of rows) {
      const k = key(r);
      if (!m.has(k)) m.set(k, empty());
      addTo(m.get(k)!, r);
    }
    return [...m.entries()]
      .sort(([x], [y]) => x.localeCompare(y))
      .map(([k, totals]) => ({ key: k, totals }));
  };
  const totals = empty();
  for (const r of rows) addTo(totals, r);
  return {
    deflator,
    // O mock só faz preço médio (a família fictícia mora no Brasil).
    cost_method: 'average',
    cost_method_automatic: costMethodChoice == null,
    currency: 'BRL',
    from,
    to,
    rows,
    totals,
    by_month: group((r) => r.date.slice(0, 7)),
    by_year: group((r) => r.date.slice(0, 4)),
    by_class: group((r) => r.class),
    by_asset: group((r) => r.asset_id),
  };
}

const IR_TABLE: [number, number][] = [
  [180, 0.225],
  [360, 0.2],
  [720, 0.175],
  [Infinity, 0.15],
];
const IOF = [
  96, 93, 90, 86, 83, 80, 76, 73, 70, 66, 63, 60, 56, 53, 50, 46, 43, 40, 36, 33, 30, 26, 23, 20,
  16, 13, 10, 6, 3, 0,
];

function netRedemption(principal: number, value: number, days: number, exempt: boolean): number {
  const gain = Math.max(0, value - principal);
  if (exempt || gain === 0) return value;
  const iof = days < 30 ? gain * (IOF[Math.max(0, days - 1)]! / 100) : 0;
  const rate = IR_TABLE.find(([d]) => days <= d)![1];
  return value - iof - (gain - iof) * rate;
}

/** Valor na curva de um lote (CDI pós-fixado, IPCA+ ou prefixado), base 252. */
function curveValue(
  a: Asset,
  principal: number,
  start: string,
  at: string,
): { value: number; estimated: boolean } {
  const t = a.fixed_income!;
  const end = t.maturity_date && t.maturity_date < at ? t.maturity_date : at;
  if (end <= start) return { value: principal, estimated: false };
  const rate = Number(t.rate);
  const du = businessDays(addDays(start, 1), end).length;
  if (t.indexer === 'cdi' || t.indexer === 'selic') {
    const rates = cdiRates(start, end);
    if (!rates.length) return { value: principal, estimated: true };
    let v = principal;
    for (const r of rates) v *= 1 + (r.value / 100) * (rate / 100);
    return { value: v, estimated: false };
  }
  if (t.indexer === 'prefixado')
    return { value: principal * (1 + rate / 100) ** (du / 252), estimated: false };
  const f = deflatorFactor('ipca', start, end);
  if (!f) return { value: principal * (1 + rate / 100) ** (du / 252), estimated: true };
  return { value: principal * f.factor * (1 + rate / 100) ** (du / 252), estimated: f.estimated };
}

interface Valued {
  value: number;
  valueBase: number | null;
  price: PriceOn | null;
  valuation: Valuation;
  netBase: number | null;
  estimated: boolean;
}

function valueBook(asset: Asset, b: Book, at: string): Valued {
  if (b.qty === 0)
    return {
      value: 0,
      valueBase: 0,
      price: null,
      valuation: 'market',
      netBase: null,
      estimated: false,
    };
  const fx = fxToBrl(asset.currency, at);
  let estimated = fx?.estimated ?? false;
  const market = asset.class === 'fixed_income' ? null : closeOn(asset.id, at);
  if (market) {
    estimated ||= market.price.stale && daysBetween(market.price.date, at) > 7;
    const v = b.qty * market.close;
    return {
      value: v,
      valueBase: fx ? v * fx.rate : null,
      price: market.price,
      valuation: 'market',
      netBase: null,
      estimated,
    };
  }
  if (isCurve(asset) && asset.fixed_income) {
    let gross = 0;
    let net = 0;
    for (const lot of b.lots) {
      const cv = curveValue(asset, lot.principal, lot.date, at);
      estimated ||= cv.estimated;
      gross += cv.value;
      net += netRedemption(
        lot.principal,
        cv.value,
        daysBetween(lot.date, at),
        asset.fixed_income.ir_exempt,
      );
    }
    const r = fx?.rate ?? null;
    return {
      value: gross,
      valueBase: r == null ? null : gross * r,
      price: null,
      valuation: 'curve',
      netBase: r == null ? null : net * r,
      estimated,
    };
  }
  return {
    value: b.cost,
    valueBase: b.costBase,
    price: null,
    valuation: 'missing',
    netBase: null,
    estimated: true,
  };
}

interface Flow {
  date: string;
  amount: number;
}

function xirr(flows: Flow[]): number | null {
  if (flows.length < 2) return null;
  const hasNeg = flows.some((f) => f.amount < 0);
  const hasPos = flows.some((f) => f.amount > 0);
  if (!hasNeg || !hasPos) return null;
  const sorted = [...flows].sort((a, b) => a.date.localeCompare(b.date));
  const t0 = sorted[0]!.date;
  const ts = sorted.map((f) => daysBetween(t0, f.date) / 365);
  if (ts[ts.length - 1]! < 1 / 365) return null;
  const npv = (r: number) => sorted.reduce((s, f, i) => s + f.amount / (1 + r) ** ts[i]!, 0);
  let lo = -0.9999;
  let hi = 10;
  let flo = npv(lo);
  const fhi = npv(hi);
  if (flo * fhi > 0) return null;
  for (let i = 0; i < 200; i++) {
    const mid = (lo + hi) / 2;
    const fm = npv(mid);
    if (Math.abs(fm) < 1e-7) return mid;
    if (fm * flo < 0) hi = mid;
    else {
      lo = mid;
      flo = fm;
    }
  }
  return (lo + hi) / 2;
}

function incomesOf(assetId: string, accountIds: string[] | null, until: string): IncomeEvent[] {
  return state.incomes.filter(
    (i) =>
      i.asset_id === assetId &&
      i.status === 'received' &&
      i.pay_date <= until &&
      (!accountIds || accountIds.includes(i.account_id)),
  );
}

function incomeFx(i: IncomeEvent): number {
  if (i.currency === 'BRL') return 1;
  if (i.fx_rate) return Number(i.fx_rate);
  return fxToBrl(i.currency, i.pay_date)?.rate ?? 0;
}

function flowsOf(assetId: string, accountIds: string[] | null, until: string): Flow[] {
  const out: Flow[] = [];
  for (const t of state.trades) {
    if (
      t.asset_id !== assetId ||
      t.date > until ||
      (accountIds && !accountIds.includes(t.account_id))
    )
      continue;
    const fx = t.currency === 'BRL' ? 1 : tradeFx(t);
    const gross = Number(t.quantity) * Number(t.price);
    const costs = (t.fees + t.taxes) / minorFactor(t.currency);
    out.push({ date: t.date, amount: (t.side === 'buy' ? -(gross + costs) : gross - costs) * fx });
  }
  for (const i of incomesOf(assetId, accountIds, until)) {
    out.push({ date: i.pay_date, amount: (i.net / minorFactor(i.currency)) * incomeFx(i) });
  }
  return out;
}

function deflate(flows: Flow[], to: string, deflator: string): Flow[] | null {
  const out: Flow[] = [];
  for (const f of flows) {
    const fac = deflatorFactor(deflator, f.date, to);
    if (!fac) return null;
    out.push({ date: f.date, amount: f.amount * fac.factor });
  }
  return out;
}

function brief(a: Asset): AssetBrief {
  return {
    id: a.id,
    ticker: a.ticker,
    name: a.name,
    class: a.class,
    class_label: a.class_label,
    currency: a.currency,
  };
}

function positionView(
  assetId: string,
  accountIds: string[] | null,
  at: string,
  deflator: string,
  accountId: string | null,
): [PositionView, Flow[]] {
  const asset = getAsset(assetId);
  const b = replay(assetId, accountIds, at);
  const v = valueBook(asset, b, at);
  const income = incomesOf(assetId, accountIds, at).reduce(
    (s, i) => s + (i.net / minorFactor(i.currency)) * incomeFx(i),
    0,
  );
  const flows = flowsOf(assetId, accountIds, at);
  const open = b.qty !== 0;
  const nominal = [...flows];
  if (open && v.valueBase != null) flows.push({ date: at, amount: v.valueBase });
  const irr = xirr(flows);
  let irrReal: number | null = irr;
  if (irr != null && deflator !== 'none') {
    const real = deflate(nominal, at, deflator);
    if (real) {
      if (open && v.valueBase != null) real.push({ date: at, amount: v.valueBase });
      irrReal = xirr(real);
    } else irrReal = null;
  }
  // Efeito da inflação no resultado (como no núcleo): fluxos próprios corrigidos até a data.
  let effect: number | null = 0;
  if (deflator !== 'none') {
    const real = deflate(nominal, at, deflator);
    effect = real ? cents(real.reduce((s, r, i) => s + nominal[i]!.amount - r.amount, 0)) : null;
  }
  const costBase = cents(b.costBase);
  const mvBase = v.valueBase == null ? null : cents(v.valueBase);
  const unrealized = mvBase == null ? null : mvBase - costBase;
  const realizedBase = cents(b.realizedBase);
  const incomeBase = cents(income);
  const warnings = [...b.warnings];
  if (v.valuation === 'missing') warnings.push('sem cotação: avaliado pelo custo');
  const first = state.trades
    .filter((t) => t.asset_id === assetId)
    .sort((x, y) => x.date.localeCompare(y.date))[0];
  const f = minorFactor(asset.currency);
  return [
    {
      asset: brief(asset),
      account_id: accountId,
      quantity: dec(b.qty),
      avg_price: b.qty > 0 ? dec(b.cost / b.qty) : null,
      cost: Math.round(b.cost * f),
      cost_base: costBase,
      price: v.price,
      market_value: Math.round(v.value * f),
      market_value_base: mvBase,
      unrealized_base: unrealized,
      realized_base: realizedBase,
      income_base: incomeBase,
      total_result_base: unrealized == null ? null : unrealized + realizedBase + incomeBase,
      real_result_base:
        unrealized == null || effect == null
          ? null
          : unrealized + realizedBase + incomeBase - effect,
      inflation_effect_base: effect,
      // Como no núcleo: abaixo de 30 dias, só o retorno do período (sem anualizado).
      ...(() => {
        const days = first
          ? Math.max(0, Math.round((Date.parse(at) - Date.parse(first.date)) / 86_400_000))
          : null;
        const basis: PositionView['return_basis'] =
          days == null || days < 30 ? 'period' : days < 365 ? 'period+annualized' : 'annualized';
        const total = unrealized == null ? null : unrealized + realizedBase + incomeBase;
        const real = unrealized == null || effect == null ? null : total! - effect;
        return {
          xirr: basis === 'period' ? null : irr,
          xirr_real: basis === 'period' ? null : irrReal,
          holding_days: days,
          period_return: total != null && costBase > 0 ? total / costBase : null,
          period_return_real: real != null && costBase > 0 ? real / costBase : null,
          return_basis: basis,
        };
      })(),
      weight: null,
      valuation: v.valuation,
      net_redemption_base: v.netBase == null ? null : cents(v.netBase),
      first_date: first?.date ?? null,
      estimated: b.estimated || v.estimated,
      warnings,
    },
    flows,
  ];
}

function tradedPairs(accountIds: string[] | null): [string, string][] {
  const seen = new Set<string>();
  const out: [string, string][] = [];
  for (const t of state.trades) {
    const k = `${t.asset_id}|${t.account_id}`;
    if (seen.has(k) || (accountIds && !accountIds.includes(t.account_id))) continue;
    seen.add(k);
    out.push([t.asset_id, t.account_id]);
  }
  return out;
}

function classTargets(): Map<string, [number, number]> {
  const m = new Map<string, [number, number]>();
  for (const t of state.targets)
    if (t.scope === 'class') m.set(t.key, [Number(t.target_pct), Number(t.tolerance_pct)]);
  return m;
}

interface SummaryArgs {
  as_of?: string;
  account_ids?: string[];
  deflator?: string;
  by_account?: boolean;
  include_closed?: boolean;
}

/**
 * Risco de exemplo: séries mensais determinísticas por classe (o núcleo usa os retornos
 * reais de cada ativo), com as métricas calculadas como no núcleo.
 */
function risk(n: number): PortfolioRisk {
  const s = summary({});
  const now = new Date();
  const months = Array.from({ length: n }, (_, i) => {
    const d = new Date(now.getFullYear(), now.getMonth() - n + i, 1);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
  });
  const vol: Record<string, number> = {
    stock_br: 0.07,
    fii: 0.04,
    fixed_income: 0.002,
    treasury_br: 0.01,
    stock_us: 0.06,
    crypto: 0.18,
  };
  const series = (seed: number, sd: number) =>
    months.map((_, i) => Math.round((0.008 + sd * Math.sin(seed * 1.7 + i * 1.3)) * 1e6) / 1e6);
  const cdi = months.map(() => 0.009);
  const metrics = (r: (number | null)[]): RiskMetrics | null => {
    const v = r.filter((x): x is number => x != null);
    if (v.length < 2) return null;
    const mean = v.reduce((a, b) => a + b, 0) / v.length;
    const sd = Math.sqrt(v.reduce((a, b) => a + (b - mean) ** 2, 0) / (v.length - 1));
    let level = 1;
    let peak = 1;
    let worst = 0;
    let pi = -1;
    let wp = -1;
    let wt = -1;
    v.forEach((x, i) => {
      level *= 1 + x;
      if (level > peak) {
        peak = level;
        pi = i;
      }
      if (level / peak - 1 < worst) {
        worst = level / peak - 1;
        wp = pi;
        wt = i;
      }
    });
    const growth = v.reduce((a, b) => a * (1 + b), 1);
    return {
      months: v.length,
      annual_return: growth ** (12 / v.length) - 1,
      volatility: sd * Math.sqrt(12),
      max_drawdown: worst,
      drawdown_peak: wt >= 0 ? (wp >= 0 ? months[wp]! : 'início') : null,
      drawdown_trough: wt >= 0 ? months[wt]! : null,
      drawdown_recovered: null,
      sharpe: sd > 0 ? ((mean - 0.009) / sd) * Math.sqrt(12) : null,
      cdi_annual: 1.009 ** 12 - 1,
      best_month: Math.max(...v),
      worst_month: Math.min(...v),
    };
  };
  const total = s.market_value || 1;
  const assets = s.positions
    .filter((p) => (p.market_value_base ?? 0) > 0)
    .map((p, i) => ({
      p,
      weight: (p.market_value_base ?? 0) / total,
      r: series(i + 1, vol[p.asset.class] ?? 0.03),
    }));
  const port = months.map((_, i) => assets.reduce((a, x) => a + (x.r[i] ?? 0) * x.weight, 0));
  const classes = [...new Set(assets.map((a) => a.p.asset.class))].map((c) => {
    const rows = assets.filter((a) => a.p.asset.class === c);
    const w = rows.reduce((a, x) => a + x.weight, 0);
    return {
      c,
      label: rows[0]!.p.asset.class_label,
      w,
      r: months.map((_, i) => rows.reduce((a, x) => a + (x.r[i] ?? 0) * x.weight, 0) / (w || 1)),
    };
  });
  const corr = (a: (number | null)[], b: (number | null)[]) => {
    const xs = a as number[];
    const ys = b as number[];
    const ma = xs.reduce((s2, x) => s2 + x, 0) / xs.length;
    const mb = ys.reduce((s2, x) => s2 + x, 0) / ys.length;
    let c = 0;
    let va = 0;
    let vb = 0;
    xs.forEach((x, i) => {
      c += (x - ma) * (ys[i]! - mb);
      va += (x - ma) ** 2;
      vb += (ys[i]! - mb) ** 2;
    });
    return va > 0 && vb > 0 ? Math.round((c / Math.sqrt(va * vb)) * 1e4) / 1e4 : null;
  };
  const matrix = (rows: { id: string; label: string; r: (number | null)[] }[]) => ({
    ids: rows.map((r) => r.id),
    labels: rows.map((r) => r.label),
    values: rows.map((a) => rows.map((b) => (a.id === b.id ? 1 : corr(a.r, b.r)))),
  });
  const pm = metrics(port);
  const share = (w: number, r: (number | null)[]) => {
    const c = corr(r, port);
    const sd = (x: (number | null)[]) => metrics(x)?.volatility ?? 0;
    return c == null || !pm ? null : (w * c * sd(r)) / (pm.volatility || 1);
  };
  return {
    from_month: months[0]!,
    to_month: months[n - 1]!,
    months,
    portfolio: pm,
    realized: pm,
    portfolio_monthly: port,
    cdi_monthly: cdi,
    by_class: classes.map((c) => ({
      class: c.c,
      label: c.label,
      weight: c.w,
      metrics: metrics(c.r),
      risk_share: share(c.w, c.r),
    })),
    assets: assets.map((a) => ({
      asset_id: a.p.asset.id,
      ticker: a.p.asset.ticker,
      name: a.p.asset.name,
      class: a.p.asset.class,
      weight: a.weight,
      metrics: metrics(a.r),
      risk_share: share(a.weight, a.r),
    })),
    class_correlation: matrix(classes.map((c) => ({ id: c.c, label: c.label, r: c.r }))),
    asset_correlation: matrix(
      assets.slice(0, 15).map((a) => ({
        id: a.p.asset.id,
        label: a.p.asset.ticker ?? a.p.asset.name,
        r: a.r,
      })),
    ),
    market_value: s.market_value,
    notes: ['Risco de exemplo (séries fictícias).'],
  };
}

function summary(args: SummaryArgs): PortfolioSummary {
  const at = args.as_of ?? mockToday();
  const deflator = args.deflator ?? 'ipca';
  const accountIds = args.account_ids?.length ? args.account_ids : null;
  const pairs = tradedPairs(accountIds);
  const units: [string, string | null][] = [];
  for (const [a, acc] of pairs) {
    if (args.by_account) units.push([a, acc]);
    else if (!units.some(([x]) => x === a)) units.push([a, null]);
  }
  const positions: PositionView[] = [];
  const allFlows: Flow[] = [];
  const missing: string[] = [];
  const balanceDue: BalanceDue[] = [];
  let estimated = false;
  let mv = 0;
  let cost = 0;
  let realized = 0;
  let income = 0;
  let effect: number | null = 0;
  for (const [assetId, acc] of units) {
    const scope = acc ? [acc] : accountIds;
    const [pv, flows] = positionView(assetId, scope, at, deflator, acc);
    realized += pv.realized_base;
    income += pv.income_base;
    effect =
      effect == null || pv.inflation_effect_base == null ? null : effect + pv.inflation_effect_base;
    allFlows.push(...flows);
    const open = pv.quantity !== '0';
    if (open) {
      mv += pv.market_value_base ?? pv.cost_base;
      cost += pv.cost_base;
      // Como o núcleo: previdência sem saldo (ou com saldo de mais de 90 dias) pede o saldo,
      // em vez de entrar nos avaliados pelo custo.
      const priceDate = pv.price?.date ?? null;
      if (
        pv.asset.class === 'pension' &&
        (pv.valuation === 'missing' || (priceDate != null && daysBetween(priceDate, at) > 90))
      ) {
        if (!balanceDue.some((b) => b.asset_id === pv.asset.id))
          balanceDue.push({ asset_id: pv.asset.id, name: pv.asset.name, as_of: priceDate });
      } else if (pv.valuation === 'missing') missing.push(pv.asset.ticker ?? pv.asset.name);
      estimated ||= pv.estimated;
    }
    if (open || args.include_closed) positions.push(pv);
  }
  for (const p of positions)
    if (mv > 0 && p.market_value_base != null) p.weight = p.market_value_base / mv;
  positions.sort((a, b) => (b.market_value_base ?? 0) - (a.market_value_base ?? 0));
  const irr = xirr(allFlows);
  let irrReal = irr;
  if (deflator !== 'none') {
    const finals = allFlows.filter((f) => f.date === at && f.amount > 0);
    const rest = allFlows.filter((f) => !(f.date === at && f.amount > 0));
    const real = deflate(rest, at, deflator);
    irrReal = real ? xirr([...real, ...finals]) : null;
  }
  const targets = classTargets();
  const by = new Map<string, [number, number]>();
  for (const p of positions) {
    if (p.quantity === '0') continue;
    const e = by.get(p.asset.class) ?? [0, 0];
    e[0] += p.market_value_base ?? p.cost_base;
    e[1] += p.cost_base;
    by.set(p.asset.class, e);
  }
  for (const k of targets.keys()) if (!by.has(k)) by.set(k, [0, 0]);
  const byClass: ClassSummary[] = [...by.entries()]
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([k, [v, c]]) => {
      const weight = mv > 0 ? v / mv : 0;
      const t = targets.get(k);
      const dev = t ? weight * 100 - t[0] : null;
      return {
        class: k as AssetClass,
        label: classLabel(k as AssetClass),
        market_value_base: v,
        cost_base: c,
        weight,
        target_pct: t ? t[0] : null,
        deviation_pp: dev,
        out_of_band: dev != null && t != null && Math.abs(dev) > t[1],
        xirr: null,
      };
    });
  const out: PortfolioSummary = {
    as_of: at,
    base_currency: 'BRL',
    deflator: deflator as PortfolioSummary['deflator'],
    market_value: mv,
    cost,
    unrealized: mv - cost,
    realized,
    income,
    total_result: mv - cost + realized + income,
    real_result: effect == null ? null : mv - cost + realized + income - effect,
    inflation_effect: effect,
    xirr: irr,
    xirr_real: irrReal,
    by_class: byClass,
    positions,
    missing_prices: missing,
    balance_due: balanceDue,
    estimated,
    concentration_alerts: [],
  };
  out.concentration_alerts = concentrationAlerts(out, targets);
  return out;
}

interface PerfArgs {
  from_month?: string;
  to_month?: string;
  deflator?: string;
  asset_id?: string;
  class?: string;
  method?: TwrMethod;
}

/** Ativos com operação, pelo filtro de ativo e classe da rentabilidade e da ponte. */
function perfAssets(args: PerfArgs): string[] {
  return [...new Set(state.trades.map((t) => t.asset_id))]
    .filter((id) => !args.asset_id || id === args.asset_id)
    .filter((id) => !args.class || getAsset(id).class === args.class);
}

/** Ajuste avançado `twr_daily` do mock de ajustes (desligado sem ele). */
function twrDaily(): boolean {
  try {
    const s = extraMockHandler('settings.get')?.({}) as AppSettings | undefined;
    return s?.twr_daily === true;
  } catch {
    return false;
  }
}

/** Entradas (compras) e saídas (vendas líquidas e proventos) de um dia, em reais. */
function dayFlows(assets: string[], d: string): { inflow: number; outflow: number } {
  let inflow = 0;
  let outflow = 0;
  for (const t of state.trades) {
    if (t.date !== d || !assets.includes(t.asset_id)) continue;
    const fx = t.currency === 'BRL' ? 1 : tradeFx(t);
    const gross = Number(t.quantity) * Number(t.price);
    const costs = (t.fees + t.taxes) / minorFactor(t.currency);
    if (t.side === 'buy') inflow += (gross + costs) * fx;
    else outflow += (gross - costs) * fx;
  }
  for (const i of state.incomes) {
    if (i.pay_date !== d || i.status !== 'received' || !assets.includes(i.asset_id)) continue;
    outflow += (i.net / minorFactor(i.currency)) * incomeFx(i);
  }
  return { inflow, outflow };
}

/**
 * Semidesvio, volatilidade e pior queda de uma série de retornos (como
 * `engine::return_risk`): nível 1 em `start`, `perYear` 12 (mensal) ou 252 (diário).
 */
function returnRisk(series: [string, number][], start: string, perYear: number): ReturnRisk | null {
  if (series.length < 2) return null;
  const n = series.length;
  const mean = series.reduce((s, [, r]) => s + r, 0) / n;
  const variance = series.reduce((s, [, r]) => s + (r - mean) ** 2, 0) / (n - 1);
  const down = series.filter(([, r]) => r < mean).reduce((s, [, r]) => s + (r - mean) ** 2, 0) / n;
  let level = 1;
  let peak = 1;
  let peakAt = start;
  let worst = 0;
  let worstPeak: string | null = null;
  let worstTrough: string | null = null;
  let longest = 0;
  let ongoing = false;
  let underwater = false;
  for (const [d, r] of series) {
    level *= 1 + r;
    if (level >= peak) {
      if (underwater) {
        longest = Math.max(longest, daysBetween(peakAt, d));
        underwater = false;
      }
      if (level > peak) {
        peak = level;
        peakAt = d;
      }
      continue;
    }
    underwater = true;
    const dd = level / peak - 1;
    if (dd < worst) {
      worst = dd;
      worstPeak = peakAt;
      worstTrough = d;
    }
  }
  const last = series[series.length - 1]![0];
  if (underwater) {
    const open = daysBetween(peakAt, last);
    if (open >= longest) {
      longest = open;
      ongoing = true;
    }
  }
  let recovered: string | null = null;
  if (worstTrough) {
    let lv = 1;
    let top: number | null = worstPeak === start ? 1 : null;
    for (const [d, r] of series) {
      lv *= 1 + r;
      if (d === worstPeak) top = lv;
      if (d > worstTrough && top != null && lv >= top) {
        recovered = d;
        break;
      }
    }
  }
  const round6 = (v: number) => Math.round(v * 1e6) / 1e6;
  return {
    periods: n,
    semideviation: round6(Math.sqrt(down) * Math.sqrt(perYear)),
    volatility: round6(Math.sqrt(variance) * Math.sqrt(perYear)),
    max_drawdown: round6(worst),
    drawdown_peak: worstPeak,
    drawdown_trough: worstTrough,
    drawdown_recovered: recovered,
    longest_drawdown_days: longest,
    longest_drawdown_ongoing: ongoing,
  };
}

function performance(args: PerfArgs): Performance {
  const today = mockToday();
  const deflator = args.deflator ?? 'ipca';
  const method: TwrMethod = args.method ?? (twrDaily() ? 'daily' : 'monthly');
  const assets = perfAssets(args);
  const first = state.trades
    .filter((t) => assets.includes(t.asset_id))
    .map((t) => t.date)
    .sort()[0];
  const startMonth = args.from_month ?? (first ? monthOf(first) : monthOf(today));
  const endMonth = args.to_month ?? monthOf(today);
  const months: string[] = [];
  for (let m = startMonth; m <= endMonth; m = nextMonth(m)) {
    months.push(m);
    if (months.length > 600) mockFail('invalid', 'período longo demais');
  }
  const periodStart = addDays(monthStart(startMonth), -1);
  const valueAt = (d: string) => {
    let total = 0;
    let est = false;
    for (const id of assets) {
      const a = getAsset(id);
      const v = valueBook(a, replay(id, null, d), d);
      total += v.valueBase ?? 0;
      est ||= v.estimated;
    }
    return { total, est };
  };
  const out: Performance = {
    months: [],
    monthly: [],
    cumulative: [],
    cumulative_real: [],
    cdi_cumulative: [],
    benchmark_label: 'CDI',
    ibov_cumulative: [],
    sp500_brl_cumulative: [],
    usd_cumulative: [],
    inflation_cumulative: [],
    deflator: deflator as Performance['deflator'],
    value_base: [],
    net_contributions: [],
    estimated: false,
    method,
    risk: null,
  };
  // Série para o risco: (fim do mês, retorno) no mensal; (dia útil, retorno) no diário.
  const series: [string, number][] = [];
  let { total: vPrev, est: estimated } = valueAt(periodStart);
  let acc = 1;
  let contrib = 0;
  for (const m of months) {
    const firstDay = monthStart(m);
    const end = monthEnd(m) < today ? monthEnd(m) : today;
    const { total: vEnd, est } = valueAt(end);
    estimated ||= est;
    const dim = daysBetween(firstDay, end) + 1;
    let flows = 0;
    let weighted = 0;
    let inc = 0;
    for (const t of state.trades) {
      if (!assets.includes(t.asset_id) || t.date < firstDay || t.date > end) continue;
      const fx = t.currency === 'BRL' ? 1 : tradeFx(t);
      const gross = Number(t.quantity) * Number(t.price);
      const costs = (t.fees + t.taxes) / minorFactor(t.currency);
      const f = (t.side === 'buy' ? gross + costs : -(gross - costs)) * fx;
      flows += f;
      weighted += (f * (daysBetween(t.date, end) + 1)) / dim;
    }
    for (const i of state.incomes) {
      if (
        !assets.includes(i.asset_id) ||
        i.status !== 'received' ||
        i.pay_date < firstDay ||
        i.pay_date > end
      )
        continue;
      inc += (i.net / minorFactor(i.currency)) * incomeFx(i);
    }
    contrib += flows;
    let r: number | null;
    if (method === 'daily') {
      // Cada dia útil: (valor final + saídas) ÷ (valor inicial + entradas) − 1, encadeado.
      let chain = 1;
      let any = false;
      let prev = vPrev;
      const days = businessDays(firstDay, end);
      if (!days.includes(end)) days.push(end);
      for (const d of days) {
        const { total: v, est: e } = valueAt(d);
        estimated ||= e;
        const { inflow, outflow } = dayFlows(assets, d);
        const den = prev + inflow;
        if (den > 0) {
          const rd = (v + outflow) / den - 1;
          chain *= 1 + rd;
          series.push([d, rd]);
          any = true;
        }
        prev = v;
      }
      r = any ? chain - 1 : null;
    } else {
      const denom = vPrev + weighted;
      r = denom > 0 ? (vEnd - vPrev - flows + inc) / denom : null;
      if (r != null) series.push([end, r]);
    }
    if (r != null) acc *= 1 + r;
    const cum = acc - 1;
    const infl = deflatorFactor(deflator, addDays(periodStart, 1), addDays(end, 1));
    const cdi = cdiRates(periodStart, end);
    out.months.push(m);
    out.monthly.push(r);
    out.cumulative.push(cum);
    out.inflation_cumulative.push(infl ? infl.factor - 1 : null);
    out.cumulative_real.push(infl ? (1 + cum) / infl.factor - 1 : null);
    if (infl?.estimated) estimated = true;
    out.cdi_cumulative.push(
      cdi.length ? cdi.reduce((s, p) => s * (1 + p.value / 100), 1) - 1 : null,
    );
    out.ibov_cumulative.push(benchGrowth('ibov', periodStart, end, false));
    out.sp500_brl_cumulative.push(benchGrowth('sp500', periodStart, end, true));
    const fx0 = fxToBrl('USD', periodStart);
    const fx1 = fxToBrl('USD', end);
    out.usd_cumulative.push(fx0 && fx1 ? fx1.rate / fx0.rate - 1 : null);
    out.value_base.push(cents(vEnd));
    out.net_contributions.push(cents(contrib));
    vPrev = vEnd;
  }
  out.estimated = estimated;
  out.risk = returnRisk(series, periodStart, method === 'daily' ? 252 : 12);
  return out;
}

interface BridgeAcc {
  start: number;
  end: number;
  gainLocal: number;
  buyGross: number;
  sellGross: number;
  income: number;
  fees: number;
  taxes: number;
  contributions: number;
  withdrawals: number;
  otherOut: number;
  flows: Flow[];
}

const ZERO_LINES: BridgeLines = {
  start_value: 0,
  price_gain: 0,
  fx_gain: 0,
  income: 0,
  fees: 0,
  taxes: 0,
  contributions: 0,
  withdrawals: 0,
  end_value: 0,
  delta: 0,
};

/**
 * Ponte do resultado, como `portfolio::bridge`: valor no início + ganho de preço (com o
 * câmbio) + proventos brutos − taxas − impostos + aportes − retiradas = valor no fim.
 */
function bridge(args: PerfArgs): PerformanceBridge {
  const today = mockToday();
  const deflator = (args.deflator ?? 'ipca') as Deflator;
  const ids = perfAssets(args);
  const first = state.trades
    .filter((t) => ids.includes(t.asset_id))
    .map((t) => t.date)
    .sort()[0];
  const fromMonth = args.from_month ?? (first ? monthOf(first) : monthOf(today));
  const toMonth = args.to_month ?? monthOf(today);
  const from = monthStart(fromMonth);
  const to = monthEnd(toMonth) < today ? monthEnd(toMonth) : today;
  if (to < from) mockFail('invalid', 'o fim do período vem antes do início');
  const start = addDays(from, -1);
  const inside = (d: string) => d > start && d <= to;
  let estimated = false;
  const total: BridgeLines = { ...ZERO_LINES };
  const rows: BridgeAsset[] = [];
  const allFlows: Flow[] = [];
  for (const id of ids) {
    const asset = getAsset(id);
    const a: BridgeAcc = {
      start: 0,
      end: 0,
      gainLocal: 0,
      buyGross: 0,
      sellGross: 0,
      income: 0,
      fees: 0,
      taxes: 0,
      contributions: 0,
      withdrawals: 0,
      otherOut: 0,
      flows: [],
    };
    const v0 = valueBook(asset, replay(id, null, start), start);
    const v1 = valueBook(asset, replay(id, null, to), to);
    estimated ||= v0.estimated || v1.estimated;
    a.start = v0.valueBase ?? 0;
    a.end = v1.valueBase ?? 0;
    a.gainLocal = v1.value - v0.value;
    for (const t of state.trades) {
      if (t.asset_id !== id || !inside(t.date)) continue;
      const fx = asset.currency === 'BRL' ? 1 : tradeFx(t);
      const gross = Number(t.quantity) * Number(t.price);
      const m = minorFactor(t.currency);
      const costs = (t.fees + t.taxes) / m;
      a.fees += (t.fees / m) * fx;
      a.taxes += (t.taxes / m) * fx;
      if (t.side === 'buy') {
        a.buyGross += gross * fx;
        a.gainLocal -= gross;
        a.contributions += (gross + costs) * fx;
        a.flows.push({ date: t.date, amount: (gross + costs) * fx });
      } else {
        a.sellGross += gross * fx;
        a.gainLocal += gross;
        a.withdrawals += (gross - costs) * fx;
        a.flows.push({ date: t.date, amount: -(gross - costs) * fx });
      }
    }
    for (const i of state.incomes) {
      if (i.asset_id !== id || i.status !== 'received' || !inside(i.pay_date)) continue;
      const fx = incomeFx(i);
      const m = minorFactor(i.currency);
      a.income += ((i.net + i.tax_withheld) / m) * fx;
      a.taxes += (i.tax_withheld / m) * fx;
      a.withdrawals += (i.net / m) * fx;
      a.flows.push({ date: i.pay_date, amount: -(i.net / m) * fx });
    }
    for (const ac of state.actions) {
      if (ac.asset_id !== id || ac.kind !== 'capital_return' || !ac.amount || !inside(ac.date))
        continue;
      // Restituição de capital: sai do ativo como provento.
      const fx = fxToBrl(asset.currency, ac.date)?.rate ?? 0;
      const v = ac.amount / 100;
      a.income += v * fx;
      a.withdrawals += v * fx;
      a.otherOut += v * fx;
      a.gainLocal += v;
      a.flows.push({ date: ac.date, amount: -v * fx });
    }
    let fxEnd: number | null = null;
    if (asset.currency !== 'BRL') {
      const f = fxToBrl(asset.currency, to);
      estimated ||= f?.estimated ?? true;
      fxEnd = f?.rate ?? null;
    }
    const price = a.end - a.start - a.buyGross + a.sellGross + a.otherOut;
    const l: BridgeLines = {
      start_value: cents(a.start),
      price_gain: 0,
      fx_gain: fxEnd == null ? 0 : cents(price - a.gainLocal * fxEnd),
      income: cents(a.income),
      fees: cents(a.fees),
      taxes: cents(a.taxes),
      contributions: cents(a.contributions),
      withdrawals: cents(a.withdrawals),
      end_value: cents(a.end),
      delta: 0,
    };
    // O ganho de preço fecha a ponte em centavos.
    l.price_gain =
      l.end_value - l.start_value - l.income + l.fees + l.taxes - l.contributions + l.withdrawals;
    l.delta = l.end_value - l.start_value - l.contributions + l.withdrawals;
    if (Object.values(l).every((v) => v === 0)) continue;
    for (const k of Object.keys(total) as (keyof BridgeLines)[]) total[k] += l[k];
    allFlows.push({ date: from, amount: a.start }, ...a.flows);
    rows.push({ asset: brief(asset), lines: l });
  }
  rows.sort(
    (x, y) => y.lines.end_value - x.lines.end_value || x.asset.name.localeCompare(y.asset.name),
  );
  // Perda para a inflação: o valor inicial e cada aporte e retirada corrigidos até o fim.
  let loss: number | null = deflator === 'none' ? null : 0;
  const endNext = addDays(to, 1);
  for (const f of allFlows) {
    if (loss == null) break;
    if (f.amount === 0) continue;
    const fac = deflatorFactor(deflator, f.date, endNext);
    if (!fac) {
      loss = null;
      break;
    }
    estimated ||= fac.estimated;
    loss += f.amount * (fac.factor - 1);
  }
  const inflationLoss = loss == null ? null : cents(loss);
  return {
    from,
    to,
    currency: 'BRL',
    lines: total,
    deflator,
    inflation_loss: inflationLoss,
    delta_real: inflationLoss == null ? null : total.delta - inflationLoss,
    assets: rows,
    estimated,
  };
}

/** Nível do benchmark (em R$ para o S&P 500) até 10 dias antes, como no núcleo. */
function benchLevel(id: string, date: string, inBrl: boolean): number | null {
  let hit: { date: string; value: number } | null = null;
  for (const p of seriesPoints(id)) {
    if (p.date > date) break;
    hit = p;
  }
  if (!hit || daysBetween(hit.date, date) > 10) return null;
  if (!inBrl) return hit.value;
  const fx = fxToBrl('USD', date);
  return fx ? hit.value * fx.rate : null;
}

function benchGrowth(id: string, from: string, to: string, inBrl: boolean): number | null {
  const a = benchLevel(id, from, inBrl);
  const b = benchLevel(id, to, inBrl);
  return a && b ? b / a - 1 : null;
}

/** Como `portfolio::benchmark`: acumulado da série nos meses da rentabilidade. */
function benchmarkSeries(months: string[], target: BenchmarkTarget): BenchmarkSeries {
  const today = mockToday();
  if (months.length === 0) return { label: '', cumulative: [], estimated: false };
  const base = addDays(monthStart(months[0]!), -1);
  const daily = (id: string, from: string, to: string, extra: number) => {
    const pts = seriesPoints(id).filter((p) => p.date > from && p.date <= to);
    const dayExtra = (1 + extra) ** (1 / 252);
    return pts.length ? pts.reduce((acc, p) => acc * (1 + p.value / 100) * dayExtra, 1) - 1 : null;
  };
  let label: string;
  let growth: (end: string) => number | null;
  let estimated = false;
  if (target.kind === 'asset') {
    const asset = getAsset(target.id);
    label = asset.ticker ?? asset.name;
    const level = (d: string) => {
      const c = closeOn(asset.id, d);
      const fx = fxToBrl(asset.currency, d);
      return c && fx ? c.close * fx.rate : null;
    };
    const b = level(base);
    growth = (end) => {
      const n = level(end);
      return b && n ? n / b - 1 : null;
    };
  } else if (target.kind === 'index') {
    const def = SERIES.find((x) => x.id === target.id);
    if (!def) mockFail('not_found', 'série de índice desconhecida');
    label = def.name;
    if (def.kind === 'level') {
      growth = (end) => benchGrowth(def.id, base, end, false);
    } else if (def.kind === 'daily_rate') {
      growth = (end) => daily(def.id, base, end, 0);
    } else if (def.kind === 'monthly_rate') {
      growth = (end) => {
        const pts = seriesPoints(def.id).filter(
          (p) => monthOf(p.date) > monthOf(base) && monthOf(p.date) <= monthOf(end),
        );
        estimated ||= pts.some((p) => p.estimated);
        return pts.length ? pts.reduce((acc, p) => acc * (1 + p.value / 100), 1) - 1 : null;
      };
    } else {
      // Taxa anual: a vigente no fim, pró-rata pelos dias (o núcleo integra as mudanças).
      growth = (end) => {
        const last = seriesPoints(def.id)
          .filter((p) => p.date <= end)
          .at(-1);
        return last ? (1 + last.value / 100) ** (daysBetween(base, end) / 365) - 1 : null;
      };
    }
  } else {
    const spread = target.spread;
    label = `CDI + ${(spread * 100).toFixed(2).replace('.', ',')}% ao ano`;
    growth = (end) => daily('cdi', base, end, spread);
  }
  const cumulative = months.map((m) => growth(monthEnd(m) < today ? monthEnd(m) : today));
  return { label, cumulative, estimated };
}

/** Como `portfolio::contrib`: resultado de cada ativo no período e os pontos do retorno. */
function contributions(fromMonth?: string, toMonth?: string): ContributionReport {
  const today = mockToday();
  const ids = [...new Set(state.trades.map((t) => t.asset_id))].filter(hasAsset);
  const first = state.trades.map((t) => t.date).sort()[0];
  const start = addDays(monthStart(fromMonth ?? (first ? monthOf(first) : monthOf(today))), -1);
  const end = toMonth && monthEnd(toMonth) < today ? monthEnd(toMonth) : today;
  const span = Math.max(1, daysBetween(start, end));
  let denom = 0;
  const raw: { asset: AssetBrief; result: number; income: number }[] = [];
  for (const id of ids) {
    const a = getAsset(id);
    const v0 = valueBook(a, replay(id, null, start), start).valueBase ?? 0;
    const v1 = valueBook(a, replay(id, null, end), end).valueBase ?? 0;
    let flows = 0;
    let weighted = 0;
    let traded = false;
    for (const t of state.trades) {
      if (t.asset_id !== id || t.date <= start || t.date > end) continue;
      const fx = t.currency === 'BRL' ? 1 : tradeFx(t);
      const gross = Number(t.quantity) * Number(t.price);
      const costs = (t.fees + t.taxes) / minorFactor(t.currency);
      const f = (t.side === 'buy' ? gross + costs : -(gross - costs)) * fx;
      flows += f;
      weighted += (f * daysBetween(t.date, end)) / span;
      traded = true;
    }
    let income = 0;
    for (const i of state.incomes) {
      if (i.asset_id !== id || i.status !== 'received' || i.pay_date <= start || i.pay_date > end)
        continue;
      income += (i.net / minorFactor(i.currency)) * incomeFx(i);
    }
    if (v0 === 0 && v1 === 0 && !traded && income === 0) continue;
    denom += v0 + weighted;
    raw.push({ asset: brief(a), result: v1 - v0 - flows + income, income });
  }
  const rows: Contribution[] = raw
    .map((r) => ({
      asset: r.asset,
      result: cents(r.result),
      income: cents(r.income),
      points: denom > 0 ? r.result / denom : null,
    }))
    .sort((x, y) => y.result - x.result);
  return {
    from: start,
    to: end,
    base_currency: 'BRL',
    rows,
    total: rows.reduce((s, r) => s + r.result, 0),
    total_points: denom > 0 ? rows.reduce((s, r) => s + (r.points ?? 0), 0) : null,
  };
}

function rebalance(contribution: number): Rebalance {
  const s = summary({ deflator: 'none' });
  const c = Math.max(0, contribution);
  const totalAfter = s.market_value + c;
  const targets = classTargets();
  const sum = [...targets.values()].reduce((x, [p]) => x + p, 0);
  const lines = s.by_class
    .filter((k) => k.target_pct != null)
    .map((k) => {
      const tp = k.target_pct ?? 0;
      const ideal = Math.round((totalAfter * tp) / 100);
      return {
        class: k.class,
        label: k.label,
        current: k.market_value_base,
        current_pct: k.weight * 100,
        target_pct: tp,
        gap: ideal - k.market_value_base,
        suggested: 0,
        out_of_band: k.out_of_band,
      };
    });
  const positive = lines.reduce((x, l) => x + Math.max(0, l.gap), 0);
  if (c > 0 && positive > 0) {
    let given = 0;
    lines.forEach((l, i) => {
      if (l.gap <= 0) return;
      const share = i + 1 === lines.length ? c - given : Math.floor((c * l.gap) / positive);
      l.suggested = Math.max(0, Math.min(share, c - given));
      given += l.suggested;
    });
    if (given < c) {
      const top = lines.filter((l) => l.gap > 0).sort((a, b) => b.gap - a.gap)[0];
      if (top) top.suggested += c - given;
    }
  }
  // Sem alvo de classe, o aporte vai pelos limites de concentração (como no núcleo).
  const concentration = lines.length === 0 ? concentrationRoom(s, c) : [];
  const room = concentration.reduce((x, l) => x + (l.over_limit ? 0 : l.room), 0);
  return {
    total: s.market_value,
    contribution: c,
    lines,
    assets: [],
    targets_incomplete: Math.abs(sum - 100) > 0.01,
    concentration,
    unallocated: lines.length === 0 && concentration.length ? Math.max(0, c - room) : 0,
    disclaimer:
      'Análise baseada nos seus dados e nos alvos que você definiu; não é recomendação de investimento.',
  };
}

function passiveIncome(target: number | null): PassiveIncome {
  const today = mockToday();
  const start = addMonths(monthStart(monthOf(today)), -11);
  const byMonth = new Map<string, number>();
  for (let i = 0; i < 12; i++) byMonth.set(monthOf(addMonths(start, i)), 0);
  const byAsset = new Map<string, number>();
  for (const ev of state.incomes) {
    if (ev.status !== 'received' || ev.pay_date < start || ev.pay_date > today) continue;
    const v = cents((ev.net / minorFactor(ev.currency)) * incomeFx(ev));
    byMonth.set(monthOf(ev.pay_date), (byMonth.get(monthOf(ev.pay_date)) ?? 0) + v);
    byAsset.set(ev.asset_id, (byAsset.get(ev.asset_id) ?? 0) + v);
  }
  const s = summary({ deflator: 'none' });
  const last12 = [...byMonth.values()].reduce((a, b) => a + b, 0);
  const y = s.market_value > 0 ? last12 / s.market_value : null;
  return {
    last_12m: last12,
    monthly_avg: Math.trunc(last12 / 12),
    by_month: [...byMonth.entries()].map(([month, amount]) => ({ month, amount })),
    by_asset: [...byAsset.entries()]
      .map(([id, v]) => {
        const mv = s.positions.find((p) => p.asset.id === id)?.market_value_base;
        return { asset: brief(getAsset(id)), last_12m: v, yield_12m: mv && mv > 0 ? v / mv : null };
      })
      .sort((a, b) => b.last_12m - a.last_12m),
    target_monthly: target,
    capital_needed: target != null && y && y > 0 ? Math.round((target * 12) / y) : null,
    portfolio_yield_12m: y,
  };
}

// ── Validação e gravação ─────────────────────────────────────────

function num(v: unknown, what: string): number {
  const s = txt(v).trim().replace(',', '.');
  if (!/^[+-]?(\d+\.?\d*|\.\d+)$/.test(s)) mockFail('invalid', `${what} inválido: ${String(v)}`);
  return Number(s);
}

function date(v: unknown): string {
  if (typeof v !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(v))
    mockFail('invalid', `data inválida: ${String(v)}`);
  return v;
}

function intArg(v: unknown): number {
  if (v == null) return 0;
  if (typeof v !== 'number' || !Number.isSafeInteger(v))
    mockFail('invalid', 'argumentos inválidos: esperado inteiro');
  return v;
}

function fmtQty(q: number): string {
  return dec(q).replace('.', ',');
}

function toTrade(t: TradeRec): Trade {
  const asset = getAsset(t.asset_id);
  const f = minorFactor(t.currency);
  const gross = Math.round(Number(t.quantity) * Number(t.price) * f);
  return {
    id: t.id,
    asset_id: t.asset_id,
    ticker: asset.ticker,
    asset_name: asset.name,
    by_value: asset.class === 'fixed_income',
    account_id: t.account_id,
    account_name: account(t.account_id).name,
    date: t.date,
    side: t.side,
    quantity: t.quantity,
    price: t.price,
    fees: t.fees,
    taxes: t.taxes,
    currency: t.currency,
    fx_rate: t.fx_rate,
    day_trade: t.day_trade,
    gross,
    cash_amount: t.side === 'buy' ? -(gross + t.fees + t.taxes) : gross - t.fees - t.taxes,
    txn_id: t.txn_id,
    note_number: t.note_number,
    notes: t.notes,
    import_id: null,
  };
}

function createTrade(a: Record<string, unknown>, keepId?: string): string {
  const d = date(a.date);
  const asset = getAsset(String(a.asset_id));
  const acc = account(String(a.account_id));
  const qty = num(a.quantity, 'quantidade');
  const price = num(a.price, 'preço');
  if (qty <= 0) mockFail('invalid', 'a quantidade deve ser positiva');
  if (price < 0) mockFail('invalid', 'o preço não pode ser negativo');
  const fees = intArg(a.fees);
  const taxes = intArg(a.taxes);
  if (fees < 0 || taxes < 0) mockFail('invalid', 'custos e impostos não podem ser negativos');
  const fx = a.fx_rate == null || a.fx_rate === '' ? null : num(a.fx_rate, 'câmbio');
  if (fx != null && fx <= 0) mockFail('invalid', 'o câmbio deve ser positivo');
  const side: TradeSide = a.side === 'sell' ? 'sell' : 'buy';
  if (side === 'sell') {
    const held = replay(asset.id, [acc.id], d).qty;
    if (qty > held + 1e-12) {
      mockFail(
        'invalid',
        `venda de ${fmtQty(qty)} maior que a posição de ${fmtQty(held)} nesta conta na data`,
      );
    }
  }
  const cash = a.cash_movement !== false;
  if (
    cash &&
    acc.currency !== asset.currency &&
    acc.currency === 'BRL' &&
    fx == null &&
    !fxToBrl(asset.currency, d)
  ) {
    mockFail(
      'invalid',
      `a conta é em ${acc.currency} e o ativo em ${asset.currency}: informe o câmbio ou desligue a movimentação de caixa`,
    );
  }
  if (cash && acc.currency !== asset.currency && acc.currency !== 'BRL') {
    mockFail(
      'invalid',
      `a conta é em ${acc.currency} e o ativo em ${asset.currency}: informe o câmbio ou desligue a movimentação de caixa`,
    );
  }
  state.seq += 1;
  const id = keepId ?? `trd-${state.seq}`;
  state.trades.push({
    id,
    asset_id: asset.id,
    account_id: acc.id,
    date: d,
    side,
    quantity: dec(qty),
    price: dec(price),
    fees,
    taxes,
    currency: asset.currency,
    fx_rate: fx == null ? null : dec(fx),
    day_trade: a.day_trade === true,
    txn_id: cash ? `txn-${id}` : null,
    note_number: typeof a.note_number === 'string' && a.note_number ? a.note_number : null,
    notes: typeof a.notes === 'string' && a.notes ? a.notes : null,
    seq: state.seq,
  });
  return id;
}

function findTrade(id: string): TradeRec {
  const t = state.trades.find((x) => x.id === id);
  if (!t) mockFail('not_found', `Não encontrado: operação ${id}.`);
  return t;
}

/**
 * `ids` de um lote (`portfolio.delete_trades` & cia.), sem repetidos; como no núcleo, um
 * id que não existe cancela o lote inteiro. O mock não guarda histórico: `change_set_id`
 * sai `null` (sem "Desfazer").
 */
function batchIds(a: Record<string, unknown>, exists: (id: string) => boolean, what: string) {
  const ids = [...new Set(Array.isArray(a.ids) ? a.ids.map(String) : [])];
  if (ids.length === 0) mockFail('invalid', 'nenhum registro selecionado');
  const missing = ids.find((id) => !exists(id));
  if (missing) mockFail('not_found', `Não encontrado: ${what} ${missing}.`);
  return new Set(ids);
}

const ACTION_KINDS: CorporateActionKind[] = [
  'split',
  'reverse_split',
  'bonus',
  'subscription',
  'merger',
  'ticker_change',
  'spin_off',
  'capital_return',
];

function createAction(a: Record<string, unknown>): CorporateAction {
  const d = date(a.date);
  const asset = getAsset(String(a.asset_id));
  const kind = a.kind as CorporateActionKind;
  if (!ACTION_KINDS.includes(kind))
    mockFail('invalid', `argumentos inválidos: tipo ${String(kind)}`);
  const factor = a.factor == null || a.factor === '' ? null : num(a.factor, 'fator');
  const amount = a.amount == null ? null : intArg(a.amount);
  if (kind === 'split' && (factor == null || factor <= 1))
    mockFail('invalid', 'desdobramento precisa de fator maior que 1 (ex.: 2 para 1→2)');
  if (kind === 'reverse_split' && (factor == null || factor <= 0 || factor >= 1))
    mockFail('invalid', 'grupamento precisa de fator entre 0 e 1 (ex.: 0,1 para 10→1)');
  if (kind === 'bonus' && (factor == null || factor <= 0))
    mockFail('invalid', 'bonificação precisa do fator (ações novas por ação, ex.: 0,1 = 10%)');
  if (kind === 'capital_return' && (amount == null || amount <= 0))
    mockFail('invalid', 'devolução de capital precisa do valor total');
  if ((kind === 'ticker_change' || kind === 'merger') && !a.new_asset_id)
    mockFail('invalid', 'informe o ativo resultante');
  state.seq += 1;
  const rec: CorporateAction = {
    id: `act-${state.seq}`,
    asset_id: asset.id,
    date: d,
    kind,
    factor: factor == null ? null : dec(factor),
    unit_cost: a.unit_cost == null || a.unit_cost === '' ? null : dec(num(a.unit_cost, 'custo')),
    amount,
    new_asset_id: typeof a.new_asset_id === 'string' ? a.new_asset_id : null,
    notes: typeof a.notes === 'string' && a.notes ? a.notes : null,
    source: 'manual',
    account_id: null,
    quantity: null,
  };
  state.actions.push(rec);
  return rec;
}

function createIncome(i: Record<string, unknown>): IncomeEvent {
  const pay = date(i.pay_date);
  if (i.ex_date != null) date(i.ex_date);
  const asset = getAsset(String(i.asset_id));
  const acc = account(String(i.account_id));
  const gross = intArg(i.gross);
  if (gross <= 0) mockFail('invalid', 'o valor bruto do provento deve ser positivo');
  const kind = (i.kind ?? 'dividend') as IncomeKind;
  const tax =
    i.tax_withheld == null
      ? defaultTax(
          kind,
          asset.class,
          gross,
          creditOn(i.ex_date as string | null, String(i.pay_date)),
        )
      : intArg(i.tax_withheld);
  if (tax < 0 || tax > gross) mockFail('invalid', 'imposto retido inválido');
  const status = i.status === 'announced' ? 'announced' : 'received';
  const fx = i.fx_rate == null || i.fx_rate === '' ? null : num(i.fx_rate, 'câmbio');
  const cash = status === 'received' && i.cash_movement !== false;
  if (
    cash &&
    acc.currency !== asset.currency &&
    (acc.currency !== 'BRL' || (fx == null && !fxToBrl(asset.currency, pay)))
  ) {
    mockFail(
      'invalid',
      `a conta é em ${acc.currency} e o ativo em ${asset.currency}: informe o câmbio ou desligue a movimentação de caixa`,
    );
  }
  state.seq += 1;
  const rec: IncomeEvent = {
    id: `inc-${state.seq}`,
    asset_id: asset.id,
    ticker: asset.ticker,
    asset_name: asset.name,
    account_id: acc.id,
    kind,
    ex_date: typeof i.ex_date === 'string' ? i.ex_date : null,
    pay_date: pay,
    quantity: i.quantity == null || i.quantity === '' ? null : dec(num(i.quantity, 'quantidade')),
    gross,
    tax_withheld: tax,
    net: gross - tax,
    currency: asset.currency,
    fx_rate: fx == null ? null : dec(fx),
    status,
    txn_id: cash ? `txn-inc-${state.seq}` : null,
    source: 'manual',
  };
  state.incomes.push(rec);
  return rec;
}

/**
 * Proventos anunciados que a família tinha direito e ninguém lançou
 * (`portfolio.pending_income`): dois eventos extras da fonte de mercado, fora da agenda
 * semeada, um deles sem data de pagamento (a data-ex vira o palpite).
 */
function pendingEvents(): DividendRec[] {
  const today = mockToday();
  return [
    {
      asset_id: 'ast-petr4',
      kind: 'dividend',
      ex_date: previousBusinessDay(addDays(today, -25)),
      pay_date: previousBusinessDay(addDays(today, -6)),
      amount_per_share: 0.38,
      currency: 'BRL',
      estimated: false,
      source: 'yahoo',
    },
    {
      asset_id: 'ast-wege3',
      kind: 'jcp',
      ex_date: previousBusinessDay(addDays(today, -40)),
      // Sem data de pagamento na fonte: o núcleo usa a data-ex.
      pay_date: '',
      amount_per_share: 0.12,
      currency: 'BRL',
      estimated: false,
      source: 'yahoo',
    },
    {
      // Fund amortization from the official notice (Fundos.NET) that nobody recorded, older
      // than the year that "To confirm" shows by default.
      asset_id: 'ast-knri11',
      kind: 'amortization',
      ex_date: previousBusinessDay(addDays(today, -400)),
      pay_date: previousBusinessDay(addDays(today, -393)),
      amount_per_share: 0.35,
      currency: 'BRL',
      estimated: false,
      source: 'fundos_net',
    },
  ];
}

const OFFICIAL_SOURCES = new Set(['b3_listados', 'fundos_net']);

/**
 * `portfolio.quick_income(_preview)`: data, ticker e valor recebido; o resto sai da agenda
 * (provento anunciado até 5 dias da data), das contas com operações e da retenção padrão.
 */
function quickIncome(a: Record<string, unknown>, write: boolean): QuickIncomeReport {
  const rows = Array.isArray(a.rows) ? (a.rows as Record<string, unknown>[]) : [];
  if (!rows.length) mockFail('invalid', 'nenhuma linha de provento');
  const today = mockToday();
  const seen = new Set<string>();
  const lines = rows.map((row, n): QuickIncomeLine => {
    const ticker = (typeof row.ticker === 'string' ? row.ticker : '').trim().toUpperCase();
    const pay = (typeof row.date === 'string' ? row.date : '').trim();
    const amount = Number(row.amount);
    const line: QuickIncomeLine = {
      row: n,
      date: pay,
      ticker,
      amount,
      state: 'error',
      asset_id: null,
      asset_name: null,
      currency: null,
      account_id: null,
      account_name: null,
      kind: null,
      ex_date: null,
      quantity: null,
      per_share: null,
      gross: null,
      tax_withheld: null,
      net: null,
      status: null,
      matched_source: null,
      verified: false,
      notes: [],
      error: null,
      needs_account: false,
    };
    const fail = (msg: string, needsAccount = false) => ({
      ...line,
      error: msg,
      needs_account: needsAccount,
    });
    if (!/^\d{4}-\d{2}-\d{2}$/.test(pay)) return fail(`data inválida: ${pay}`);
    if (!ticker) return fail('informe o ticker');
    if (!(amount > 0)) return fail('o valor deve ser positivo');
    const asset = allAssets().find((x) => x.ticker?.toUpperCase() === ticker);
    if (!asset) return fail(`${ticker} não está cadastrado na carteira`);
    Object.assign(line, { asset_id: asset.id, asset_name: asset.name, currency: asset.currency });
    const forced = typeof row.account_id === 'string' && row.account_id ? row.account_id : null;
    const accs = forced
      ? [forced]
      : [...new Set(state.trades.filter((t) => t.asset_id === asset.id).map((t) => t.account_id))];
    if (!accs.length)
      return fail(`nenhuma operação de ${ticker} registrada: escolha a conta`, true);
    const forcedKind = (row.kind as IncomeKind | undefined) ?? null;
    let best: {
      ev: DividendRec | null;
      acc: string;
      qty: number;
      diff: number;
      rank: number;
    } | null = null;
    for (const ev of [...pendingEvents(), ...calendar()]) {
      const evPay = ev.pay_date || ev.ex_date;
      if (ev.asset_id !== asset.id || ev.estimated) continue;
      if (Math.abs(daysBetween(evPay, pay)) > 5) continue;
      if (forcedKind && ev.kind !== forcedKind) continue;
      for (const acc of accs) {
        const qty = replay(asset.id, [acc], ev.ex_date).qty;
        if (qty <= 0) continue;
        const gross = cents(qty * ev.amount_per_share);
        const net =
          gross - defaultTax(ev.kind, asset.class, gross, creditOn(ev.ex_date, ev.pay_date));
        const diff = Math.min(Math.abs(amount - net) / net, Math.abs(amount - gross) / gross);
        const rank = diff + Math.abs(daysBetween(evPay, pay)) * 0.001;
        if (!best || rank < best.rank) best = { ev, acc, qty, diff, rank };
      }
    }
    if (!best) {
      const held = accs
        .map((acc) => ({ acc, qty: replay(asset.id, [acc], pay).qty }))
        .sort((x, y) => y.qty - x.qty);
      if (held[0]!.qty <= 0 && accs.length > 1) {
        return fail(`sem posição de ${ticker} na data: escolha a conta`, true);
      }
      best = { ev: null, acc: held[0]!.acc, qty: held[0]!.qty, diff: Infinity, rank: Infinity };
    }
    const kind: IncomeKind =
      forcedKind ??
      best.ev?.kind ??
      (asset.class === 'fii'
        ? 'fii_income'
        : asset.class === 'fixed_income' || asset.class === 'treasury_br'
          ? 'coupon'
          : 'dividend');
    const rate = defaultTax(kind, asset.class, 1_000_000, String(row.date)) / 1_000_000;
    const gross = rate > 0 ? Math.round(amount / (1 - rate)) : amount;
    line.kind = kind;
    line.gross = gross;
    line.tax_withheld = gross - amount;
    line.net = amount;
    line.account_id = best.acc;
    line.account_name = account(best.acc).name;
    if (best.ev) {
      line.ex_date = best.ev.ex_date;
      line.per_share = dec(best.ev.amount_per_share);
      line.quantity = dec(best.qty);
      line.matched_source = best.ev.source;
      line.verified = best.diff <= 0.02;
      line.notes.push(
        line.verified
          ? `confere: ${dec(best.qty)} × ${dec(best.ev.amount_per_share)}`
          : 'valor diferente do esperado: confira',
      );
    } else {
      line.notes.push('nenhum provento anunciado perto da data: tipo pela classe');
    }
    line.status = pay > today ? 'announced' : 'received';
    const key = `${asset.id}|${best.acc}|${pay}|${gross}`;
    if (
      state.incomes.some(
        (i) =>
          i.asset_id === asset.id &&
          i.account_id === best.acc &&
          i.pay_date === pay &&
          (i.net === amount || i.gross === gross),
      )
    ) {
      return { ...line, state: 'duplicate', error: 'já registrado' };
    }
    if (seen.has(key)) return { ...line, state: 'duplicate', error: 'repete uma linha acima' };
    seen.add(key);
    return { ...line, state: 'ready' };
  });
  let created = 0;
  if (write) {
    for (const l of lines) {
      if (l.state !== 'ready') continue;
      createIncome({
        asset_id: l.asset_id,
        account_id: l.account_id,
        kind: l.kind,
        ex_date: l.ex_date,
        pay_date: l.date,
        quantity: l.quantity,
        gross: l.gross,
        tax_withheld: l.tax_withheld,
        status: l.status,
      });
      created += 1;
    }
  }
  return { created, lines };
}

/** Conferência entre fontes de exemplo: PETR4 diverge (Yahoo × B3), o resto de fonte única. */
function pendingCheck(ev: DividendRec, pay: string): PendingIncome['check'] {
  const own = {
    source: ev.source,
    kind: ev.kind,
    ex_date: ev.ex_date,
    pay_date: ev.pay_date || null,
    amount_per_share: dec(ev.amount_per_share),
  };
  if (ev.asset_id === 'ast-petr4') {
    return {
      status: 'divergent',
      reference: 'b3_listados',
      sources: [
        { ...own, source: 'b3_listados', pay_date: pay },
        { ...own, amount_per_share: dec(ev.amount_per_share + 0.02) },
      ],
      note: `Yahoo dizia ${dec(ev.amount_per_share + 0.02).replace('.', ',')} por cota; usado o oficial (${dec(ev.amount_per_share).replace('.', ',')})`,
    };
  }
  return {
    status: 'single',
    reference: ev.source,
    sources: [own],
    note: 'só no Yahoo: a B3 não tem este provento (ainda não baixado ou fora da lista oficial)',
  };
}

/** Custo das taxas (mesma conta do núcleo): fundos de exemplo com 2% de administração. */
function feeCost(a: Record<string, unknown>): FeeCost {
  const gross = typeof a.real_return === 'number' ? a.real_return : 0.04;
  const alt = typeof a.alternative_fee === 'number' ? a.alternative_fee : 0.005;
  const monthly = typeof a.monthly_contribution === 'number' ? a.monthly_contribution : 0;
  const fv = (v: number, annual: number, m: number, years: number) => {
    const g = (1 + annual) ** (1 / 12) - 1;
    const n = years * 12;
    const growth = (1 + g) ** n;
    return v * growth + m * (Math.abs(g) < 1e-12 ? n : (growth - 1) / g);
  };
  const s = summary({});
  const funds = s.positions.filter(
    (p) =>
      (p.asset.class === 'fund' || p.asset.class === 'pension') && (p.market_value_base ?? 0) > 0,
  );
  const total = funds.reduce((t, p) => t + (p.market_value_base ?? 0), 0);
  const horizons = [10, 20, 30];
  const totals = horizons.map((years) => ({
    years,
    no_fees: 0,
    current: 0,
    alternative: 0,
    cost: 0,
    savings: 0,
  }));
  const holdings = funds.map((p) => {
    const value = p.market_value_base ?? 0;
    const admin = 0.02;
    const m = total ? (monthly * value) / total : 0;
    const projections = horizons.map((years, i) => {
      const noFees = fv(value, gross, m, years);
      const current = fv(value, (1 + gross) * (1 - admin) - 1, m, years);
      const alternative = fv(value, (1 + gross) * (1 - Math.min(alt, admin)) - 1, m, years);
      const row = {
        years,
        no_fees: Math.round(noFees),
        current: Math.round(current),
        alternative: Math.round(alternative),
        cost: Math.round(noFees - current),
        savings: Math.round(alternative - current),
      };
      const t = totals[i]!;
      t.no_fees += row.no_fees;
      t.current += row.current;
      t.alternative += row.alternative;
      t.cost += row.cost;
      t.savings += row.savings;
      return row;
    });
    return {
      asset: p.asset,
      value,
      admin_fee: admin,
      performance_fee: null,
      loading_fee: null,
      source: 'extrato do fundo na CVM',
      projections,
    };
  });
  return {
    real_return: gross,
    alternative_fee: alt,
    excess_return: 0,
    monthly_contribution: monthly,
    holdings,
    totals,
    missing: [],
    notes: [
      `Valores reais de hoje, com retorno de ${(gross * 100).toFixed(1).replace('.', ',')}% a.a. acima da inflação antes das taxas.`,
    ],
  };
}

/** Regra do come-cotas por fundo (escolhida; senão longo prazo). */
const comeCotasRules = new Map<string, ComeCotasRegime>();
onMockReset(() => comeCotasRules.clear());

// Método de custo (o núcleo segue o país da família; no mock, Brasil = preço médio).
let costMethodChoice: CostMethod | null = null;
onMockReset(() => {
  costMethodChoice = null;
});
function costMethodState(): CostMethodState {
  return {
    method: costMethodChoice ?? 'average',
    automatic: costMethodChoice == null,
    country: 'BR',
    country_method: 'average',
  };
}

function comeCotasRule(asset: Asset): ComeCotasRule {
  if (asset.class !== 'fund') {
    return { regime: 'exempt', origin: 'default', note: 'só fundos abertos têm come-cotas' };
  }
  const chosen = comeCotasRules.get(asset.id);
  if (chosen) return { regime: chosen, origin: 'user', note: 'regra escolhida por você' };
  return {
    regime: 'long',
    origin: 'default',
    note: 'fundo sem classificação baixada da CVM: longo prazo (15%); ajuste se for de ações ou curto prazo',
  };
}

function pendingIncome(a: Record<string, unknown>): PendingIncome[] {
  const today = mockToday();
  const since = typeof a.since === 'string' ? date(a.since) : addMonths(today, -12);
  const out: PendingIncome[] = [];
  for (const ev of pendingEvents()) {
    if (ev.ex_date < since || (ev.pay_date || ev.ex_date) > today) continue;
    if (!hasAsset(ev.asset_id)) continue;
    const asset = getAsset(ev.asset_id);
    const accounts = [
      ...new Set(state.trades.filter((t) => t.asset_id === ev.asset_id).map((t) => t.account_id)),
    ].filter((acc) => typeof a.account_id !== 'string' || acc === a.account_id);
    for (const acc of accounts) {
      if (
        state.incomes.some(
          (i) => i.asset_id === ev.asset_id && i.account_id === acc && i.ex_date === ev.ex_date,
        )
      )
        continue;
      const qty = replay(ev.asset_id, [acc], ev.ex_date).qty;
      if (qty <= 0) continue;
      const gross = cents(qty * ev.amount_per_share);
      if (gross <= 0) continue;
      const tax = defaultTax(ev.kind, asset.class, gross, creditOn(ev.ex_date, ev.pay_date));
      const pay = ev.pay_date || ev.ex_date;
      const quantity = dec(qty);
      out.push({
        dividend_event_id: `dev-${ev.asset_id}-${ev.ex_date}`,
        asset: brief(asset),
        account_id: acc,
        kind: ev.kind,
        ex_date: ev.ex_date,
        pay_date: pay,
        pay_date_guessed: !ev.pay_date,
        quantity,
        amount_per_share: dec(ev.amount_per_share),
        currency: asset.currency,
        gross,
        tax_withheld: tax,
        net: gross - tax,
        source: ev.source,
        check: pendingCheck(ev, pay),
        asset_warning: null,
        input: {
          asset_id: ev.asset_id,
          account_id: acc,
          kind: ev.kind,
          ex_date: ev.ex_date,
          pay_date: pay,
          quantity,
          gross,
          tax_withheld: tax,
        },
      });
    }
  }
  return out;
}

// Income ledger (`portfolio.income_ledger`): the recorded distributions match; what the
// family was entitled to and nobody recorded is missing (official when B3 is the source).
let incomeAuto: string | null = null;
onMockReset(() => {
  incomeAuto = null;
});

function ledgerEntries(a: Record<string, unknown>): IncomeLedgerEntry[] {
  const fxBase = (cur: string, on: string, v: number) =>
    cur === 'BRL' ? v : Math.round(v * (fxToBrl(cur, on)?.rate ?? 5));
  const out: IncomeLedgerEntry[] = [];
  for (const i of state.incomes) {
    if (typeof a.asset_id === 'string' && i.asset_id !== a.asset_id) continue;
    if (typeof a.account_id === 'string' && i.account_id !== a.account_id) continue;
    if (!hasAsset(i.asset_id)) continue;
    const asset = getAsset(i.asset_id);
    out.push({
      key: `rec:${i.id}`,
      asset: brief(asset),
      account_id: i.account_id,
      kind: i.kind,
      ex_date: i.ex_date,
      pay_date: i.pay_date,
      pay_date_estimated: false,
      quantity: i.quantity,
      amount_per_share: null,
      currency: i.currency,
      expected_gross: i.gross,
      expected_tax: i.tax_withheld,
      expected_net: i.net,
      recorded: {
        id: i.id,
        kind: i.kind,
        ex_date: i.ex_date,
        pay_date: i.pay_date,
        quantity: i.quantity,
        gross: i.gross,
        tax_withheld: i.tax_withheld,
        net: i.net,
        status: i.status,
        source: i.source,
        cash: i.txn_id != null,
      },
      state: 'matches',
      fix: null,
      official: true,
      source: i.source === 'manual' ? 'b3_listados' : i.source,
      period: null,
      exempt: asset.class === 'fii' ? true : null,
      check: null,
      note: null,
      net_base: i.status === 'received' ? fxBase(i.currency, i.pay_date, i.net) : 0,
      upcoming_base: 0,
      cash_pending: false,
    });
  }
  const pending = pendingIncome({
    since: '2000-01-01',
    ...(typeof a.account_id === 'string' ? { account_id: a.account_id } : {}),
  });
  for (const p of pending) {
    if (typeof a.asset_id === 'string' && p.asset.id !== a.asset_id) continue;
    const official = OFFICIAL_SOURCES.has(p.source);
    out.push({
      key: `${p.dividend_event_id}|${p.account_id}`,
      asset: p.asset,
      account_id: p.account_id,
      kind: p.kind,
      ex_date: p.ex_date,
      pay_date: p.pay_date,
      pay_date_estimated: p.pay_date_guessed,
      quantity: p.quantity,
      amount_per_share: p.amount_per_share,
      currency: p.currency,
      expected_gross: p.gross,
      expected_tax: p.tax_withheld,
      expected_net: p.net,
      recorded: null,
      state: 'missing',
      fix: 'create',
      official,
      source: p.source,
      period: null,
      exempt: null,
      check: p.check,
      note: null,
      net_base: fxBase(p.currency, p.pay_date, p.net),
      upcoming_base: 0,
      cash_pending: false,
    });
  }
  return out.sort((x, y) => (y.pay_date ?? '').localeCompare(x.pay_date ?? ''));
}

function incomeLedger(a: Record<string, unknown>): IncomeLedger {
  const all = ledgerEntries(a);
  const sum = (rows: IncomeLedgerEntry[], f: (e: IncomeLedgerEntry) => number) =>
    rows.reduce((acc, e) => acc + f(e), 0);
  const recordedBase = (e: IncomeLedgerEntry) =>
    e.recorded && e.recorded.status === 'received' ? e.net_base : 0;
  const periods = (len: number): IncomeLedgerPeriod[] => {
    const map = new Map<string, IncomeLedgerPeriod>();
    for (const e of all) {
      const k = (e.pay_date ?? e.ex_date ?? '').slice(0, len);
      if (!k) continue;
      const p = map.get(k) ?? {
        period: k,
        received_base: 0,
        upcoming_base: 0,
        recorded_base: 0,
        count: 0,
      };
      p.received_base += e.net_base;
      p.upcoming_base += e.upcoming_base;
      p.recorded_base += recordedBase(e);
      p.count += 1;
      map.set(k, p);
    }
    return [...map.values()].sort((x, y) => x.period.localeCompare(y.period));
  };
  const byAsset = new Map<string, IncomeLedgerEntry[]>();
  for (const e of all) byAsset.set(e.asset.id, [...(byAsset.get(e.asset.id) ?? []), e]);
  const missing = all.filter((e) => e.state === 'missing');
  const fixes = all.filter((e) => e.fix != null && (e.official || e.fix !== 'create'));
  const entries = a.only_problems === true ? all.filter((e) => e.state !== 'matches') : all;
  return {
    base_currency: 'BRL',
    entries,
    assets: [...byAsset.values()].map((rows) => {
      const first = state.trades
        .filter((t) => t.asset_id === rows[0]!.asset.id)
        .map((t) => t.date)
        .sort()[0];
      const years = new Map<string, IncomeLedgerPeriod>();
      for (const e of rows) {
        const k = (e.pay_date ?? '').slice(0, 4);
        const p = years.get(k) ?? {
          period: k,
          received_base: 0,
          upcoming_base: 0,
          recorded_base: 0,
          count: 0,
        };
        p.received_base += e.net_base;
        p.recorded_base += recordedBase(e);
        p.count += 1;
        years.set(k, p);
      }
      return {
        asset: rows[0]!.asset,
        first_date: first ?? null,
        received_base: sum(rows, (e) => e.net_base),
        upcoming_base: 0,
        recorded_base: sum(rows, recordedBase),
        years: [...years.values()].sort((x, y) => x.period.localeCompare(y.period)),
        fixes: rows.filter((e) => e.fix != null).length,
      };
    }),
    years: periods(4),
    months: periods(7),
    received_base: sum(all, (e) => e.net_base),
    upcoming_base: 0,
    recorded_base: sum(all, recordedBase),
    counts: {
      matches: all.filter((e) => e.state === 'matches').length,
      missing: missing.length,
      upcoming: 0,
      value_diff: 0,
      tax_diff: 0,
      duplicate: 0,
      unexpected: 0,
      fixes: fixes.length,
      unofficial_missing: missing.filter((e) => !e.official).length,
      cash_pending: 0,
    },
    auto: incomeAuto != null,
    auto_since: incomeAuto,
  };
}

function listWatch(): WatchlistItem[] {
  const today = mockToday();
  return state.watch.map((w) => {
    const asset = getAsset(w.asset_id);
    const hit = closeOn(asset.id, today);
    const p = hit?.close ?? null;
    const target = w.target_price ? Number(w.target_price) : null;
    const triggered: string[] = [];
    if (p != null && w.alert_above && p >= Number(w.alert_above))
      triggered.push(`cotação acima de ${w.alert_above}`);
    if (p != null && w.alert_below && p <= Number(w.alert_below))
      triggered.push(`cotação abaixo de ${w.alert_below}`);
    return {
      id: w.id,
      asset: brief(asset),
      target_price: w.target_price,
      alert_above: w.alert_above,
      alert_below: w.alert_below,
      notes: w.notes,
      price: hit?.price ?? null,
      upside_to_target: p && target != null ? target / p - 1 : null,
      triggered,
    };
  });
}

function normDec(v: unknown, what: string): string | null {
  if (v == null || v === '') return null;
  return dec(num(v, what));
}

// ── Comandos ─────────────────────────────────────────────────────

function args<T>(a: Record<string, unknown>): T {
  return a as T;
}

registerMockHandlers({
  ...(extraMockHandler('accounts.list') ? {} : { 'accounts.list': accountsHandler }),

  'portfolio.summary': (a) => {
    requireUnlocked();
    return summary(args<SummaryArgs>(a));
  },

  'portfolio.concentration': (a) => {
    requireUnlocked();
    return concentrationReport(summary(args<SummaryArgs>(a)), classTargets());
  },

  'portfolio.set_concentration_limit': (a) => {
    requireUnlocked();
    setConcentrationLimit(args(a));
    return concentrationReport(summary({}), classTargets());
  },

  'portfolio.position': (a) => {
    requireUnlocked();
    const id = String(a.asset_id);
    getAsset(id);
    const acc = typeof a.account_id === 'string' ? a.account_id : null;
    const at = typeof a.as_of === 'string' ? date(a.as_of) : mockToday();
    return positionView(
      id,
      acc ? [acc] : null,
      at,
      typeof a.deflator === 'string' ? a.deflator : 'ipca',
      acc,
    )[0];
  },

  'portfolio.performance': (a) => {
    requireUnlocked();
    return performance(args(a));
  },

  'portfolio.bridge': (a) => {
    requireUnlocked();
    return bridge(args(a));
  },

  'portfolio.benchmark_series': (a) => {
    requireUnlocked();
    const months = Array.isArray(a.months) ? a.months.map(String) : [];
    return benchmarkSeries(months, a.target as BenchmarkTarget);
  },

  'portfolio.contributions': (a) => {
    requireUnlocked();
    return contributions(
      typeof a.from_month === 'string' ? a.from_month : undefined,
      typeof a.to_month === 'string' ? a.to_month : undefined,
    );
  },

  'portfolio.sales': (a) => {
    requireUnlocked();
    return sales(a);
  },

  'portfolio.cost_method': () => {
    requireUnlocked();
    return costMethodState();
  },

  'portfolio.set_cost_method': (a) => {
    requireUnlocked();
    costMethodChoice = (a.method as CostMethod | undefined) ?? null;
    return costMethodState();
  },

  'portfolio.rebalance': (a) => {
    requireUnlocked();
    return rebalance(typeof a.contribution === 'number' ? a.contribution : 0);
  },

  'portfolio.passive_income': (a) => {
    requireUnlocked();
    return passiveIncome(typeof a.target_monthly === 'number' ? a.target_monthly : null);
  },

  'portfolio.dividend_calendar': (a) => {
    requireUnlocked();
    const from = date(a.from);
    const to = date(a.to);
    const onlyMine = a.only_mine !== false;
    const today = mockToday();
    const out: DividendCalendarItem[] = [];
    for (const ev of calendar()) {
      const key = ev.pay_date ?? ev.ex_date;
      if (key < from || key > to) continue;
      if (!hasAsset(ev.asset_id)) continue;
      const qdate = ev.ex_date < today ? ev.ex_date : today;
      const qty = replay(ev.asset_id, null, qdate).qty;
      if (onlyMine && qty === 0) continue;
      const asset = getAsset(ev.asset_id);
      const withholding = ev.kind === 'jcp' ? 1 - jcpRate(creditOn(ev.ex_date, ev.pay_date)) : 1;
      out.push({
        asset: brief(asset),
        kind: ev.kind,
        ex_date: ev.ex_date,
        pay_date: ev.pay_date,
        amount_per_share: dec(ev.amount_per_share),
        currency: ev.currency,
        quantity: qty ? dec(qty) : null,
        expected_net: qty
          ? Math.round(qty * ev.amount_per_share * withholding * minorFactor(ev.currency))
          : null,
        estimated: ev.estimated,
        source: ev.source,
      });
    }
    return out;
  },

  'portfolio.list_trades': (a) => {
    requireUnlocked();
    const limit = Math.min(5000, Math.max(1, typeof a.limit === 'number' ? a.limit : 500));
    const offset = typeof a.offset === 'number' ? a.offset : 0;
    return state.trades
      .filter((t) => !a.asset_id || t.asset_id === a.asset_id)
      .filter((t) => !a.account_id || t.account_id === a.account_id)
      .filter((t) => typeof a.date_from !== 'string' || t.date >= a.date_from)
      .filter((t) => typeof a.date_to !== 'string' || t.date <= a.date_to)
      .sort((x, y) => y.date.localeCompare(x.date) || y.seq - x.seq)
      .slice(offset, offset + limit)
      .map(toTrade);
  },

  // Totals of an asset's trades (mirror of `portfolio::trade_summary`).
  'portfolio.trade_summary': (a): TradeSummary => {
    requireUnlocked();
    const asset = getAsset(String(a.asset_id));
    const trades = state.trades
      .filter((t) => t.asset_id === asset.id)
      .filter((t) => !a.account_id || t.account_id === a.account_id)
      .map(toTrade);
    const dec = (n: number) => String(Number(n.toFixed(8)));
    const actions = state.actions.filter((x) => x.asset_id === asset.id);
    const isSplit = (k: string) => k === 'split' || k === 'reverse_split';
    // Quantities in today's units: later splits multiply them (events of a date come first).
    const factor = (date: string) =>
      actions
        .filter((x) => isSplit(x.kind) && x.date > date && Number(x.factor) > 0)
        .reduce((f, x) => f * Number(x.factor), 1);
    const side = (s: 'buy' | 'sell'): TradeSideTotals => {
      const rows = trades.filter((t) => t.side === s);
      const qty = rows.reduce((acc, t) => acc + Number(t.quantity) * factor(t.date), 0);
      const value = rows.reduce((acc, t) => acc + Number(t.quantity) * Number(t.price), 0);
      return {
        count: rows.length,
        quantity: dec(qty),
        gross: rows.reduce((acc, t) => acc + t.gross, 0),
        fees: rows.reduce((acc, t) => acc + t.fees, 0),
        taxes: rows.reduce((acc, t) => acc + t.taxes, 0),
        avg_price: qty ? dec(value / qty) : null,
      };
    };
    const buys = side('buy');
    const sells = side('sell');
    const costs = buys.fees + buys.taxes + sells.fees + sells.taxes;
    const volume = buys.gross + sells.gross;
    const dates = trades.map((t) => t.date).sort();
    return {
      asset_id: asset.id,
      currency: asset.currency,
      by_value: asset.class === 'fixed_income',
      buys,
      sells,
      costs,
      volume,
      cost_share: volume > 0 ? costs / volume : null,
      net_quantity: dec(Number(buys.quantity) - Number(sells.quantity)),
      split_adjusted: trades.some((t) => factor(t.date) !== 1),
      events: actions.filter((x) => !isSplit(x.kind)).length,
      first_date: dates[0] ?? null,
      last_date: dates[dates.length - 1] ?? null,
    };
  },

  'portfolio.get_trade': (a) => {
    requireUnlocked();
    return toTrade(findTrade(String(a.id)));
  },

  'portfolio.create_trade': (a) => {
    requireUnlocked();
    return toTrade(findTrade(createTrade(a)));
  },

  'portfolio.update_trade': (p) => {
    requireUnlocked();
    const cur = findTrade(String(p.id));
    const before = [...state.trades];
    state.trades = state.trades.filter((t) => t.id !== cur.id);
    const has = (k: string) => k in p && p[k] !== undefined;
    try {
      createTrade(
        {
          asset_id: cur.asset_id,
          account_id: cur.account_id,
          date: has('date') ? p.date : cur.date,
          side: has('side') ? p.side : cur.side,
          quantity: has('quantity') ? p.quantity : cur.quantity,
          price: has('price') ? p.price : cur.price,
          fees: has('fees') ? p.fees : cur.fees,
          taxes: has('taxes') ? p.taxes : cur.taxes,
          fx_rate: has('fx_rate') ? p.fx_rate : cur.fx_rate,
          day_trade: has('day_trade') ? p.day_trade : cur.day_trade,
          note_number: has('note_number') ? p.note_number : cur.note_number,
          notes: has('notes') ? p.notes : cur.notes,
          cash_movement: has('cash_movement') ? Boolean(p.cash_movement) : cur.txn_id != null,
        },
        cur.id,
      );
    } catch (e) {
      state.trades = before;
      throw e;
    }
    return toTrade(findTrade(cur.id));
  },

  'portfolio.delete_trade': (a) => {
    requireUnlocked();
    findTrade(String(a.id));
    state.trades = state.trades.filter((t) => t.id !== a.id);
    return null;
  },

  'portfolio.delete_trades': (a) => {
    requireUnlocked();
    const ids = batchIds(a, (id) => state.trades.some((t) => t.id === id), 'operação');
    state.trades = state.trades.filter((t) => !ids.has(t.id));
    return { updated: ids.size, change_set_id: null };
  },

  // Espelho simples de `portfolio.statement`: operações e eventos com a posição corrente.
  'portfolio.plans': () => {
    requireUnlocked();
    return plans.map(planView);
  },
  'portfolio.plans_pending': () => {
    requireUnlocked();
    return planDues();
  },
  'portfolio.plan_save': (a) => {
    requireUnlocked();
    const day = Number(a.day_of_month);
    if (typeof a.name !== 'string' || !a.name.trim())
      mockFail('invalid', 'informe o nome do aporte programado');
    if (!(Number(a.amount) > 0)) mockFail('invalid', 'o valor do aporte precisa ser positivo');
    if (!(day >= 1 && day <= 28)) mockFail('invalid', 'o dia do aporte precisa ficar entre 1 e 28');
    getAsset(String(a.asset_id));
    account(String(a.account_id));
    const id = typeof a.id === 'string' ? a.id : `plan-${++state.seq}`;
    const p: InvestmentPlan = {
      id,
      name: String(a.name).trim(),
      asset_id: String(a.asset_id),
      account_id: String(a.account_id),
      amount: Number(a.amount),
      day_of_month: day,
      start_date: String(a.start_date),
      end_date: typeof a.end_date === 'string' ? a.end_date : null,
      goal_id: typeof a.goal_id === 'string' ? a.goal_id : null,
      active: a.active !== false,
      notes: null,
    };
    plans = [...plans.filter((x) => x.id !== id), p];
    return planView(p);
  },
  'portfolio.plan_delete': (a) => {
    requireUnlocked();
    plans = plans.filter((x) => x.id !== String(a.id));
    return { change_set_id: null };
  },
  'portfolio.plan_confirm': (a) => {
    requireUnlocked();
    const p = planCheckDue(String(a.plan_id), String(a.due_date));
    const id = createTrade({
      asset_id: p.asset_id,
      account_id: p.account_id,
      date: typeof a.date === 'string' ? a.date : a.due_date,
      side: 'buy',
      quantity: a.quantity,
      price: a.price,
      fees: a.fees,
      notes: `Aporte programado: ${p.name}`,
    });
    planRuns.set(`${p.id}|${String(a.due_date)}`, 'confirmed');
    return { trade: toTrade(findTrade(id)), change_set_id: null };
  },
  'portfolio.plan_skip': (a) => {
    requireUnlocked();
    const p = planCheckDue(String(a.plan_id), String(a.due_date));
    planRuns.set(`${p.id}|${String(a.due_date)}`, 'skipped');
    return { change_set_id: null };
  },

  'portfolio.statement': (a) => {
    requireUnlocked();
    const id = String(a.asset_id);
    const trades = state.trades
      .filter((t) => t.asset_id === id)
      .sort((x, y) => x.date.localeCompare(y.date) || x.seq - y.seq);
    const actions = state.actions
      .filter((x) => x.asset_id === id)
      .sort((x, y) => x.date.localeCompare(y.date));
    let qty = 0;
    let cost = 0;
    let ti = 0;
    let ai = 0;
    const out: Record<string, unknown>[] = [];
    while (ti < trades.length || ai < actions.length) {
      const t = trades[ti];
      const ac = actions[ai];
      const takeAction = ac && (!t || ac.date <= t.date);
      const before = qty;
      if (takeAction) {
        ai += 1;
        const f = Number(ac.factor ?? '1');
        if (ac.kind === 'split' || ac.kind === 'reverse_split') qty = qty * f;
        else if (ac.kind === 'bonus') qty = qty + qty * f;
        const label =
          ac.kind === 'split'
            ? `Desdobramento 1:${f}`
            : ac.kind === 'reverse_split'
              ? `Grupamento ${Math.round(1 / f)}:1`
              : ac.kind === 'bonus'
                ? `Bonificação de ${f * 100}%`
                : 'Evento corporativo';
        out.push({
          date: ac.date,
          kind: ac.kind,
          label,
          quantity_change: String(qty - before),
          price: null,
          quantity_after: String(qty),
          average_price_after: qty ? String(cost / qty) : null,
          account_name: null,
          origin: ac.source === 'yahoo' ? 'coleta do Yahoo' : 'manual',
          trade_id: null,
          action_id: ac.id,
          notes: ac.notes ?? null,
          warning: null,
        });
      } else if (t) {
        ti += 1;
        const q = Number(t.quantity);
        const p = Number(t.price);
        if (t.side === 'buy') {
          qty += q;
          cost += q * p;
        } else {
          cost = qty ? cost * ((qty - q) / qty) : 0;
          qty -= q;
        }
        out.push({
          date: t.date,
          kind: t.side,
          label: t.side === 'buy' ? (p === 0 ? 'Entrada sem custo' : 'Compra') : 'Venda',
          quantity_change: String(qty - before),
          price: t.price,
          quantity_after: String(qty),
          average_price_after: qty ? String(Math.round((cost / qty) * 1e6) / 1e6) : null,
          account_name: null,
          origin: t.note_number ? `nota ${t.note_number}` : 'manual',
          trade_id: t.id,
          action_id: null,
          notes: t.notes ?? null,
          warning: null,
        });
      }
    }
    return out;
  },

  'portfolio.list_actions': (a) => {
    requireUnlocked();
    return state.actions
      .filter((x) => !a.asset_id || x.asset_id === a.asset_id)
      .sort((x, y) => x.date.localeCompare(y.date));
  },

  'portfolio.create_action': (a) => {
    requireUnlocked();
    return createAction(a);
  },

  'portfolio.delete_action': (a) => {
    requireUnlocked();
    if (!state.actions.some((x) => x.id === a.id))
      mockFail('not_found', `Não encontrado: evento ${String(a.id)}.`);
    state.actions = state.actions.filter((x) => x.id !== a.id);
    return null;
  },

  'portfolio.delete_actions': (a) => {
    requireUnlocked();
    const ids = batchIds(a, (id) => state.actions.some((x) => x.id === id), 'evento');
    state.actions = state.actions.filter((x) => !ids.has(x.id));
    return { updated: ids.size, change_set_id: null };
  },

  'portfolio.list_income': (a) => {
    requireUnlocked();
    return state.incomes
      .filter((i) => !a.asset_id || i.asset_id === a.asset_id)
      .filter((i) => typeof a.from !== 'string' || i.pay_date >= a.from)
      .filter((i) => typeof a.to !== 'string' || i.pay_date <= a.to)
      .sort((x, y) => y.pay_date.localeCompare(x.pay_date));
  },

  'portfolio.create_income': (a) => {
    requireUnlocked();
    return createIncome(a);
  },

  'portfolio.create_incomes': (a) => {
    requireUnlocked();
    const items = Array.isArray(a.items) ? (a.items as Record<string, unknown>[]) : [];
    if (!items.length) mockFail('invalid', 'nenhum provento para registrar');
    items.forEach((i) => createIncome(i));
    return { created: items.length, change_set_id: null };
  },

  'portfolio.fee_cost': (a) => {
    requireUnlocked();
    return feeCost(a);
  },

  'portfolio.compare_fixed_income': (a) => {
    requireUnlocked();
    return compareFixedIncome(a);
  },

  'portfolio.simulate_fixed_income_sale': (a) => {
    requireUnlocked();
    return simulateFixedIncomeSale(a);
  },

  'portfolio.risk': (a) => {
    requireUnlocked();
    return risk(typeof a.months === 'number' ? a.months : 36);
  },

  'portfolio.come_cotas': (a) => {
    requireUnlocked();
    const asset = getAsset(String(a.asset_id));
    return { rule: comeCotasRule(asset), events: [], warnings: [] };
  },

  'portfolio.set_come_cotas': (a) => {
    requireUnlocked();
    const asset = getAsset(String(a.asset_id));
    if (a.regime == null) comeCotasRules.delete(asset.id);
    else comeCotasRules.set(asset.id, a.regime as ComeCotasRegime);
    return comeCotasRule(asset);
  },

  'portfolio.quick_income_preview': (a) => {
    requireUnlocked();
    return quickIncome(a, false).lines;
  },

  'portfolio.quick_income': (a) => {
    requireUnlocked();
    const report = quickIncome(a, true);
    return report.created
      ? { status: 'applied', change_set_id: null, report }
      : { status: 'nothing_new', report };
  },

  'portfolio.pending_income': (a) => {
    requireUnlocked();
    const rows = pendingIncome(a);
    return incomeAuto ? rows.filter((p) => !OFFICIAL_SOURCES.has(p.source)) : rows;
  },

  'portfolio.income_ledger': (a) => {
    requireUnlocked();
    return incomeLedger(a);
  },

  'portfolio.income_rebuild': (a) => {
    requireUnlocked();
    const keys = Array.isArray(a.keys) ? (a.keys as string[]) : [];
    const chosen = ledgerEntries(a).filter(
      (e) => e.fix === 'create' && (keys.length ? keys.includes(e.key) : e.official),
    );
    for (const e of chosen) {
      createIncome({
        asset_id: e.asset.id,
        account_id: e.account_id,
        kind: e.kind,
        ex_date: e.ex_date,
        pay_date: e.pay_date ?? e.ex_date,
        quantity: e.quantity,
        gross: e.expected_gross,
        tax_withheld: e.expected_tax,
        cash_movement: false,
      });
    }
    if (typeof a.asset_id !== 'string') incomeAuto ??= mockToday();
    return {
      created: chosen.length,
      updated: 0,
      deleted: 0,
      change_set_id: null,
      auto: incomeAuto != null,
    };
  },

  'portfolio.income_auto': (a) => {
    requireUnlocked();
    incomeAuto = a.enabled === true ? (incomeAuto ?? mockToday()) : null;
    return { auto: incomeAuto != null };
  },

  // The mock ledger never has a distribution waiting for its credit; the command only answers.
  'portfolio.income_cash': (a) => {
    requireUnlocked();
    const ids = Array.isArray(a.ids) ? (a.ids as string[]) : [];
    return { created: ids.length, change_set_id: null };
  },

  'portfolio.delete_income': (a) => {
    requireUnlocked();
    if (!state.incomes.some((x) => x.id === a.id))
      mockFail('not_found', `Não encontrado: provento ${String(a.id)}.`);
    state.incomes = state.incomes.filter((x) => x.id !== a.id);
    return null;
  },

  'portfolio.delete_incomes': (a) => {
    requireUnlocked();
    const ids = batchIds(a, (id) => state.incomes.some((x) => x.id === id), 'provento');
    state.incomes = state.incomes.filter((x) => !ids.has(x.id));
    return { updated: ids.size, change_set_id: null };
  },

  'portfolio.list_targets': () => {
    requireUnlocked();
    return [...state.targets].sort(
      (a, b) => a.scope.localeCompare(b.scope) || a.key.localeCompare(b.key),
    );
  },

  'portfolio.set_targets': (a) => {
    requireUnlocked();
    const scope = a.scope === 'asset' ? 'asset' : 'class';
    const list = Array.isArray(a.targets) ? (a.targets as Record<string, unknown>[]) : [];
    let total = 0;
    for (const t of list) {
      if (t.scope !== scope) mockFail('invalid', 'todos os alvos devem ter o mesmo escopo');
      const pct = num(t.target_pct, 'alvo');
      if (pct < 0 || pct > 100) mockFail('invalid', 'o alvo deve estar entre 0% e 100%');
      total += pct;
      if (scope === 'class' && !classLabel(String(t.key) as AssetClass)) {
        mockFail('invalid', `classe de ativo inválida: ${String(t.key)}`);
      }
      if (scope === 'asset') getAsset(String(t.key));
    }
    if (scope === 'class' && total > 100 + 1e-9)
      mockFail('invalid', 'a soma dos alvos por classe passa de 100%');
    state.targets = state.targets.filter((t) => t.scope !== scope);
    for (const t of list) {
      state.seq += 1;
      state.targets.push({
        id: `tgt-${state.seq}`,
        scope,
        key: String(t.key),
        label:
          scope === 'class'
            ? classLabel(t.key as AssetClass)
            : (getAsset(String(t.key)).ticker ?? getAsset(String(t.key)).name),
        target_pct: dec(num(t.target_pct, 'alvo')),
        tolerance_pct:
          t.tolerance_pct == null || t.tolerance_pct === ''
            ? '5'
            : dec(num(t.tolerance_pct, 'tolerância')),
      });
    }
    return [...state.targets];
  },

  'portfolio.list_watchlist': () => {
    requireUnlocked();
    return listWatch();
  },

  'portfolio.watch': (a) => {
    requireUnlocked();
    const id = String(a.asset_id);
    getAsset(id);
    const rec = {
      asset_id: id,
      target_price: normDec(a.target_price, 'preço-alvo'),
      alert_above: normDec(a.alert_above, 'alerta'),
      alert_below: normDec(a.alert_below, 'alerta'),
      notes: typeof a.notes === 'string' && a.notes ? a.notes : null,
    };
    const existing = state.watch.find((w) => w.asset_id === id);
    if (existing) Object.assign(existing, rec);
    else {
      state.seq += 1;
      state.watch.push({ id: `wl-${state.seq}`, ...rec });
    }
    return listWatch();
  },

  'portfolio.unwatch': (a) => {
    requireUnlocked();
    state.watch = state.watch.filter((w) => w.asset_id !== a.asset_id);
    return listWatch();
  },
});

/** Ativos com operações ou na watchlist (os que a coleta de cotações visita). */
export function syncedAssetIds(): string[] {
  return [
    ...new Set([...state.trades.map((t) => t.asset_id), ...state.watch.map((w) => w.asset_id)]),
  ];
}

/** Operações da carteira como `Trade` (para o vínculo de lançamentos em `./operations.ts`). */
/** Valor de mercado da carteira hoje (para a classificação "Objetivo"). */
export function portfolioMarketValue(): number {
  return summary({}).market_value;
}

export function tradeViews(): Trade[] {
  return state.trades.map(toTrade);
}
