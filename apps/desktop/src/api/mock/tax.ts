/**
 * Mock de `tax.*`: apuração mensal de IR em renda variável como `tax/engine.rs` (isenção
 * de R$ 20 mil em ações, prejuízos por grupo — comum, day trade e FII —, IRRF compensado
 * no ano, mínimo de R$ 10 para o DARF 6015), DARFs (gerar, pagar, cancelar) e o relatório
 * anual.
 *
 * As vendas da apuração são um roteiro próprio de 2025–2026 nas corretoras do mock (XP e
 * Inter), com os mesmos ativos da carteira: a carteira do mock quase não vende, e assim a
 * apuração mostra todos os casos (isento, imposto pago, prejuízo compensado, abaixo do
 * mínimo, day trade e um DARF vencido). Bens e direitos e proventos do relatório anual vêm
 * da carteira do mock (`portfolio.summary`, `portfolio.list_income`).
 */
import type { Asset } from '@/api/generated/Asset';
import type { AssetBrief } from '@/api/generated/AssetBrief';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { AssetHolding } from '@/api/generated/AssetHolding';
import type { CategoryMonth } from '@/api/generated/CategoryMonth';
import type { Darf } from '@/api/generated/Darf';
import type { IncomeEvent } from '@/api/generated/IncomeEvent';
import type { IncomeLine } from '@/api/generated/IncomeLine';
import type { LateCharges } from '@/api/generated/LateCharges';
import type { LossPool } from '@/api/generated/LossPool';
import type { MonthStatus } from '@/api/generated/MonthStatus';
import type { PoolMonth } from '@/api/generated/PoolMonth';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import type { RealizedItem } from '@/api/generated/RealizedItem';
import type { TaxCategory } from '@/api/generated/TaxCategory';
import type { TaxMonth } from '@/api/generated/TaxMonth';
import type { TaxYear } from '@/api/generated/TaxYear';

import { extraMockHandler, onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addMonths, isIsoDate, monthEnd, today } from './ledger/dates';
import { currencyAndKind, insertTxn } from './ledger/domain';
import { conflict, invalid, notFound, optStr, str } from './ledger/errors';
import { write } from './ledger/store';

// ── Regras (tax/rules.rs) ────────────────────────────────────────

const RULES = {
  swingRate: 0.15,
  dayTradeRate: 0.2,
  fiiRate: 0.2,
  stockExemptionLimit: 2_000_000,
  darfMinimum: 1_000,
  darfCode: '6015',
  foreignRate: 0.15,
};

const LABEL: Record<TaxCategory, string> = {
  stocks: 'Ações (operações comuns)',
  etf: 'ETFs (operações comuns)',
  bdr: 'BDRs (operações comuns)',
  fii: 'FIIs',
  day_trade: 'Day trade',
};

const CATEGORIES: TaxCategory[] = ['stocks', 'etf', 'bdr', 'fii', 'day_trade'];

const DISCLAIMER =
  'Apuração auxiliar calculada com os seus dados; confira no programa da Receita Federal. Não é aconselhamento tributário.';

// ── Roteiro de vendas ────────────────────────────────────────────

interface Sale {
  date: string;
  asset_id: string;
  ticker: string;
  category: TaxCategory;
  quantity: number;
  /** Preço de venda unitário (R$). */
  price: number;
  /** Custo médio unitário (R$). */
  avg: number;
  /** Corretagem e emolumentos (centavos). */
  fees: number;
  /** IRRF da nota (centavos); `null` = estimado pela regra. */
  irrf: number | null;
}

const SALES: Sale[] = [
  // Prejuízo em FII que atravessa o ano.
  {
    date: '2025-11-18',
    asset_id: 'ast-knri11',
    ticker: 'KNRI11',
    category: 'fii',
    quantity: 20,
    price: 141,
    avg: 159.6,
    fees: 85,
    irrf: 14,
  },
  // Vendas de ações até R$ 20 mil: ganho isento.
  {
    date: '2026-01-20',
    asset_id: 'ast-bbas3',
    ticker: 'BBAS3',
    category: 'stocks',
    quantity: 50,
    price: 28.4,
    avg: 24.1,
    fees: 43,
    irrf: 7,
  },
  // Acima de R$ 20 mil com lucro: 15%, DARF pago.
  {
    date: '2026-02-12',
    asset_id: 'ast-petr4',
    ticker: 'PETR4',
    category: 'stocks',
    quantity: 800,
    price: 38,
    avg: 32.8,
    fees: 912,
    irrf: 152,
  },
  // Prejuízo em ações: fica para compensar.
  {
    date: '2026-04-08',
    asset_id: 'ast-itub4',
    ticker: 'ITUB4',
    category: 'stocks',
    quantity: 900,
    price: 33,
    avg: 35.04,
    fees: 891,
    irrf: 149,
  },
  // Day trade com lucro: 20%; IRRF de 1% estimado; DARF gerado e ainda não pago (vencido).
  {
    date: '2026-05-14',
    asset_id: 'ast-wege3',
    ticker: 'WEGE3',
    category: 'day_trade',
    quantity: 400,
    price: 52.6,
    avg: 50.95,
    fees: 1_960,
    irrf: null,
  },
  // ETF com lucro, todo compensado pelo prejuízo de abril.
  {
    date: '2026-06-09',
    asset_id: 'ast-bova11',
    ticker: 'BOVA11',
    category: 'etf',
    quantity: 120,
    price: 128,
    avg: 120.4,
    fees: 461,
    irrf: 77,
  },
  // FII com lucro pequeno, compensado pelo prejuízo de novembro.
  {
    date: '2026-07-21',
    asset_id: 'ast-mxrf11',
    ticker: 'MXRF11',
    category: 'fii',
    quantity: 500,
    price: 10.42,
    avg: 10.3,
    fees: 156,
    irrf: 26,
  },
  // FII com lucro: imposto abaixo de R$ 10 acumula para o mês seguinte.
  {
    date: '2026-08-11',
    asset_id: 'ast-hglg11',
    ticker: 'HGLG11',
    category: 'fii',
    quantity: 50,
    price: 162,
    avg: 154.75,
    fees: 243,
    irrf: 40,
  },
  // Acima de R$ 20 mil com lucro, parte compensada: DARF a gerar (vence em 30/10).
  {
    date: '2026-09-16',
    asset_id: 'ast-petr4',
    ticker: 'PETR4',
    category: 'stocks',
    quantity: 600,
    price: 41,
    avg: 35.2,
    fees: 738,
    irrf: 123,
  },
];

const cents = (reais: number) => Math.round(reais * 100);
const roundAway = (v: number) => Math.sign(v) * Math.round(Math.abs(v) + 1e-9);

interface MonthAgg {
  sales: Map<TaxCategory, number>;
  result: Map<TaxCategory, number>;
  irrf: number;
  irrfEstimated: boolean;
  realized: RealizedItem[];
}

function aggregate(t: string): Map<string, MonthAgg> {
  const out = new Map<string, MonthAgg>();
  SALES.filter((s) => s.date <= t).forEach((s, n) => {
    const m = s.date.slice(0, 7);
    const agg: MonthAgg = out.get(m) ?? {
      sales: new Map<TaxCategory, number>(),
      result: new Map<TaxCategory, number>(),
      irrf: 0,
      irrfEstimated: false,
      realized: [],
    };
    const sale = cents(s.quantity * s.price);
    const cost = cents(s.quantity * s.avg);
    const proceeds = sale - s.fees;
    const result = proceeds - cost;
    agg.sales.set(s.category, (agg.sales.get(s.category) ?? 0) + sale);
    agg.result.set(s.category, (agg.result.get(s.category) ?? 0) + result);
    if (s.irrf != null) agg.irrf += s.irrf;
    else {
      const est =
        s.category === 'day_trade'
          ? Math.max(roundAway(result * 0.01), 0)
          : roundAway(sale * 0.00005);
      agg.irrf += est;
      agg.irrfEstimated = true;
    }
    agg.realized.push({
      date: s.date,
      asset_id: s.asset_id,
      ticker: s.ticker,
      category: s.category,
      quantity: String(s.quantity),
      sale_value: sale,
      proceeds,
      cost,
      result,
      trade_ids: [`trd-ir-${n + 1}`],
    });
    out.set(m, agg);
  });
  return out;
}

// ── Calendário ───────────────────────────────────────────────────

const HOLIDAYS = new Set([
  '01-01',
  '04-21',
  '05-01',
  '09-07',
  '10-12',
  '11-02',
  '11-15',
  '11-20',
  '12-25',
]);

function isBusinessDay(iso: string): boolean {
  const wd = new Date(`${iso}T12:00:00Z`).getUTCDay();
  return wd !== 0 && wd !== 6 && !HOLIDAYS.has(iso.slice(5));
}

/** Último dia útil do mês seguinte. */
function darfDueDate(month: string): string {
  let d = monthEnd(addMonths(`${month}-01`, 1));
  while (!isBusinessDay(d)) {
    const dt = new Date(`${d}T12:00:00Z`);
    dt.setUTCDate(dt.getUTCDate() - 1);
    d = dt.toISOString().slice(0, 10);
  }
  return d;
}

function nextMonth(month: string): string {
  return addMonths(`${month}-01`, 1).slice(0, 7);
}

// ── Estado (DARFs) ───────────────────────────────────────────────

interface State {
  darfs: Darf[];
  seq: number;
}

let state: State | null = null;

onMockReset(() => {
  state = null;
});

function st(): State {
  if (state) return state;
  state = { darfs: [], seq: 0 };
  // Fevereiro pago em dia; maio gerado e ainda não pago.
  const feb = computeMonths(null, '2026-02', today()).find((m) => m.month === '2026-02');
  if (feb && feb.darf_amount > 0) {
    state.darfs.push(newDarf(null, feb, { status: 'paid', paid_at: '2026-03-27' }));
  }
  const may = computeMonths(null, '2026-05', today()).find((m) => m.month === '2026-05');
  if (may && may.darf_amount > 0) state.darfs.push(newDarf(null, may));
  return state;
}

function newDarf(memberId: string | null, m: TaxMonth, over: Partial<Darf> = {}): Darf {
  const s = state!;
  s.seq += 1;
  return {
    id: `darf-${s.seq}`,
    member_id: memberId,
    period: m.month,
    revenue_code: m.revenue_code,
    principal: m.darf_amount,
    fine: 0,
    interest: 0,
    total: m.darf_amount,
    due_date: m.due_date,
    status: 'pending',
    paid_at: null,
    txn_id: null,
    notes: null,
    ...over,
  };
}

// ── Apuração (compute_months) ────────────────────────────────────

function poolMonth(pool: LossPool, result: number, loss: { v: number }, rate: number): PoolMonth {
  const before = loss.v;
  let base = 0;
  let used = 0;
  if (result <= 0) loss.v += -result;
  else {
    used = Math.min(result, loss.v);
    loss.v -= used;
    base = result - used;
  }
  return {
    pool,
    result,
    loss_before: before,
    loss_used: used,
    base,
    rate: String(rate),
    tax: roundAway(base * rate),
    loss_after: loss.v,
  };
}

function formatBrl(c: number): string {
  return `R$ ${(c / 100).toLocaleString('pt-BR', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

function computeMonths(memberId: string | null, last: string, t: string): TaxMonth[] {
  const agg = aggregate(t);
  const first = [...agg.keys()].sort()[0];
  if (!first) return [];
  const pools = { common: { v: 0 }, dayTrade: { v: 0 }, fii: { v: 0 } };
  let irrfCarry = 0;
  let darfCarry = 0;
  let irrfYear = first.slice(0, 4);
  const out: TaxMonth[] = [];
  for (let m = first; m <= last; m = nextMonth(m)) {
    if (m.slice(0, 4) !== irrfYear) {
      irrfCarry = 0;
      irrfYear = m.slice(0, 4);
    }
    const a = agg.get(m);
    const get = (map: Map<TaxCategory, number> | undefined, c: TaxCategory) => map?.get(c) ?? 0;
    const stockSales = get(a?.sales, 'stocks');
    const stockResult = get(a?.result, 'stocks');
    const exempt = stockSales <= RULES.stockExemptionLimit;
    const exemptGain = exempt && stockResult > 0 ? stockResult : 0;
    const commonResult = stockResult - exemptGain + get(a?.result, 'etf') + get(a?.result, 'bdr');
    const pc = poolMonth('common', commonResult, pools.common, RULES.swingRate);
    const pd = poolMonth(
      'day_trade',
      get(a?.result, 'day_trade'),
      pools.dayTrade,
      RULES.dayTradeRate,
    );
    const pf = poolMonth('fii', get(a?.result, 'fii'), pools.fii, RULES.fiiRate);
    const tax = pc.tax + pd.tax + pf.tax;
    const irrf = a?.irrf ?? 0;
    const available = irrf + irrfCarry;
    const used = Math.min(available, tax);
    const irrfIn = Math.min(irrfCarry, tax);
    irrfCarry = available - used;
    const due = tax - used;
    const carriedIn = darfCarry;
    const total = due + carriedIn;
    const [darfAmount, carriedOut] = total >= RULES.darfMinimum ? [total, 0] : [0, total];
    darfCarry = carriedOut;
    const hasSales = (a?.sales.size ?? 0) > 0;
    const categories: CategoryMonth[] = CATEGORIES.filter(
      (c) => a?.sales.has(c) || a?.result.has(c),
    ).map((c) => ({
      category: c,
      label: LABEL[c],
      sales: get(a?.sales, c),
      result: get(a?.result, c),
      exempt_gain: c === 'stocks' ? exemptGain : 0,
    }));
    const darf = state?.darfs.find(
      (d) => (d.member_id ?? null) === memberId && d.period === m && d.status !== 'cancelled',
    );
    const status: MonthStatus =
      darf?.status === 'paid'
        ? 'paid'
        : darfAmount > 0
          ? 'to_pay'
          : carriedOut > 0
            ? 'below_minimum'
            : hasSales
              ? 'no_tax'
              : 'no_activity';
    const notes: string[] = [];
    if (hasSales && exempt && stockSales > 0)
      notes.push(
        `Vendas de ações no mês de ${formatBrl(stockSales)} (até R$ 20 mil): ganho isento; prejuízo continua compensável.`,
      );
    if (a?.irrfEstimated)
      notes.push(
        'IRRF estimado (0,005% das vendas; 1% do ganho em day trade): confira nas notas de corretagem.',
      );
    out.push({
      month: m,
      member_id: memberId,
      categories,
      pools: [pc, pd, pf],
      stock_sales: stockSales,
      stock_exempt: exempt,
      tax,
      irrf,
      irrf_estimated: a?.irrfEstimated ?? false,
      irrf_carried_in: irrfIn,
      irrf_carried_out: irrfCarry,
      // O mock não separa o IRRF de day trade.
      irrf_day_trade_carried_out: 0,
      due,
      carried_in: carriedIn,
      darf_amount: darfAmount,
      carried_out: carriedOut,
      due_date: darfDueDate(m),
      revenue_code: RULES.darfCode,
      status,
      darf_id: darf?.id ?? null,
      realized: a?.realized ?? [],
      notes,
      // O mock não apura cripto no Brasil (ganho de capital mensal, DARF 4600).
      crypto: null,
      // The mock's months come from a fixture where every sale has its purchase.
      incomplete: false,
    });
  }
  return out;
}

function monthArg(a: Record<string, unknown>): string {
  const m = str(a, 'month');
  if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(m)) invalid(`mês inválido: ${m}`);
  return m;
}

function taxMonth(memberId: string | null, month: string): TaxMonth {
  st();
  const found = computeMonths(memberId, month, today()).find((m) => m.month === month);
  return (
    found ?? {
      month,
      member_id: memberId,
      categories: [],
      pools: [],
      stock_sales: 0,
      stock_exempt: true,
      tax: 0,
      irrf: 0,
      irrf_estimated: false,
      irrf_carried_in: 0,
      irrf_carried_out: 0,
      irrf_day_trade_carried_out: 0,
      due: 0,
      carried_in: 0,
      darf_amount: 0,
      carried_out: 0,
      due_date: darfDueDate(month),
      revenue_code: RULES.darfCode,
      status: 'no_activity',
      darf_id: null,
      realized: [],
      notes: [],
      crypto: null,
      incomplete: false,
    }
  );
}

// ── Relatório anual ──────────────────────────────────────────────

const BENS: Record<AssetClass, [string, string]> = {
  stock_br: ['03.01', 'Ações'],
  fii: ['07.03', 'Fundos de Investimento Imobiliário (FII)'],
  etf_br: ['07.06', 'ETF (entidade de investimento; ETF de renda fixa vai em 07.08)'],
  bdr: ['04.04', 'BDR'],
  fixed_income: ['04.02', 'Títulos públicos e privados sujeitos à tributação'],
  treasury_br: ['04.02', 'Títulos públicos (Tesouro Direto)'],
  fund: ['07.01', 'Fundos de investimento sujeitos ao come-cotas (Lei 14.754/2023)'],
  stock_us: ['03.01', 'Ações/ETFs no exterior'],
  etf_us: ['03.01', 'Ações/ETFs no exterior'],
  reit: ['03.01', 'Ações/ETFs no exterior'],
  crypto: ['08.02', 'Outras criptomoedas (altcoins)'],
  pension: ['99.06', 'VGBL – Vida Gerador de Benefício Livre'],
  other: ['99.99', 'Outros bens e direitos'],
  stock_intl: ['03.01', 'Ações/ETFs no exterior'],
  etf_intl: ['03.01', 'Ações/ETFs no exterior'],
};

/** Como `bens_code` do núcleo (tabela do programa IRPF 2026): por tipo de renda fixa e por cripto. */
const STABLECOINS = new Set([
  'USDT',
  'USDC',
  'BRZ',
  'BUSD',
  'DAI',
  'TUSD',
  'GUSD',
  'USDP',
  'PAX',
  'PAXG',
  'FDUSD',
  'PYUSD',
  'USDE',
]);
function bensOf(asset: AssetBrief): [string, string] {
  if (asset.class === 'fixed_income') {
    const kind = call<Asset | null>('assets.get', { id: asset.id }, null)?.fixed_income?.kind;
    if (kind === 'poupanca') return ['04.01', 'Depósito em conta poupança'];
    if (kind && ['lci', 'lca', 'cri', 'cra', 'debenture_incentivada'].includes(kind))
      return [
        '04.03',
        'Títulos isentos de tributação (LCI, LCA, CRI, CRA, debêntures incentivadas)',
      ];
  }
  if (asset.class === 'crypto') {
    const base = (asset.ticker ?? '').toUpperCase().split(/[-/]/)[0] ?? '';
    if (base === 'BTC') return ['08.01', 'Criptoativo Bitcoin (BTC)'];
    if (STABLECOINS.has(base)) return ['08.03', 'Criptoativos stablecoins'];
  }
  return BENS[asset.class];
}

const FOREIGN: ReadonlySet<AssetClass> = new Set([
  'stock_us',
  'etf_us',
  'reit',
  'stock_intl',
  'etf_intl',
]);

/**
 * Como `tax/late.rs`: multa de 0,33% ao dia até 20% e juros de 1% no mês do pagamento mais a
 * Selic dos meses no meio (o mock não tem a Selic: usa 1% por mês e marca como estimado).
 */
function lateCharges(principal: number, due: string, paid: string): LateCharges {
  const days = Math.max(0, Math.round((Date.parse(paid) - Date.parse(due)) / 86_400_000));
  const finePct = Math.min(days * 0.33, 20);
  const months =
    (Number(paid.slice(0, 4)) - Number(due.slice(0, 4))) * 12 +
    Number(paid.slice(5, 7)) -
    Number(due.slice(5, 7));
  const between = Math.max(months - 1, 0);
  const interestPct = days > 0 ? between + 1 : 0;
  const fine = Math.round((principal * finePct) / 100);
  const interest = Math.round((principal * interestPct) / 100);
  return {
    days,
    fine_pct: String(Number(finePct.toFixed(2))),
    fine,
    interest_pct: String(interestPct),
    interest,
    total: principal + fine + interest,
    estimated: between > 0,
  };
}

function call<T>(cmd: string, args: Record<string, unknown>, fallback: T): T {
  const h = extraMockHandler(cmd);
  if (!h) return fallback;
  try {
    return h(args) as T;
  } catch {
    return fallback;
  }
}

function yearReport(memberId: string | null, year: number): TaxYear {
  if (!(year >= 2000 && year <= 2100)) invalid('ano inválido');
  st();
  const t = today();
  const all = computeMonths(memberId, `${year}-12`, t).filter((m) => m.month <= t.slice(0, 7));
  const months = all.filter((m) => m.month.startsWith(`${year}-`));
  const last = all.at(-1);
  const pool = (p: LossPool) => last?.pools.find((x) => x.pool === p)?.loss_after ?? 0;

  const assets = call<Asset[]>('assets.list', { include_archived: true }, []);
  const classOf = new Map(assets.map((a) => [a.id, a.class]));
  const incomes = call<IncomeEvent[]>(
    'portfolio.list_income',
    { from: `${year}-01-01`, to: `${year}-12-31` },
    [],
  ).filter((i) => i.status === 'received');
  const div = new Map<string, IncomeLine>();
  const fii = new Map<string, IncomeLine>();
  const jcp = new Map<string, IncomeLine>();
  let foreignIncome = 0;
  let foreignTaxPaid = 0;
  let foreignCredit = 0;
  for (const i of incomes) {
    const fx = i.currency === 'BRL' ? 1 : Number(i.fx_rate ?? NaN);
    const net = Number.isFinite(fx) ? roundAway(i.net * fx) : 0;
    const cls = classOf.get(i.asset_id) ?? 'other';
    if (FOREIGN.has(cls)) {
      // Como o núcleo (Lei 14.754/2023): bruto, e o imposto de lá como crédito até 15% dele.
      const gross = Number.isFinite(fx) ? roundAway(i.gross * fx) : 0;
      const paid = Number.isFinite(fx) ? roundAway(i.tax_withheld * fx) : 0;
      foreignIncome += gross;
      foreignTaxPaid += paid;
      foreignCredit += Math.min(paid, roundAway(gross * RULES.foreignRate));
      continue;
    }
    const target = i.kind === 'jcp' ? jcp : i.kind === 'fii_income' || cls === 'fii' ? fii : div;
    const line = target.get(i.asset_id) ?? {
      asset_id: i.asset_id,
      ticker: i.ticker,
      name: i.asset_name,
      amount: 0,
    };
    line.amount += net;
    target.set(i.asset_id, line);
  }

  const end = `${year}-12-31` < t ? `${year}-12-31` : t;
  const prevEnd = `${year - 1}-12-31`;
  const at = (asOf: string) =>
    call<PortfolioSummary | null>(
      'portfolio.summary',
      { as_of: asOf, deflator: 'none', include_closed: true },
      null,
    )?.positions ?? [];
  const now = at(end);
  const before = prevEnd < t ? at(prevEnd) : [];
  const ids = [...new Set([...now, ...before].map((p) => p.asset.id))];
  const holdings: AssetHolding[] = [];
  let foreignRealized = 0;
  for (const id of ids) {
    const p = now.find((x) => x.asset.id === id);
    const q = before.find((x) => x.asset.id === id);
    const asset = (p ?? q)!.asset;
    if (FOREIGN.has(asset.class) || asset.class === 'crypto')
      foreignRealized += (p?.realized_base ?? 0) - (q?.realized_base ?? 0);
    const qty = p?.quantity ?? '0';
    const qtyPrev = q?.quantity ?? '0';
    if (Number(qty) === 0 && Number(qtyPrev) === 0) continue;
    const [code, label] = bensOf(asset);
    holdings.push({
      asset_id: id,
      ticker: asset.ticker,
      name: asset.name,
      class_label: asset.class_label,
      group_code: code,
      description: `${label}: ${qty} ${asset.ticker ?? ''} — ${asset.name}. Custo médio de aquisição.`,
      quantity_prev: qtyPrev,
      cost_prev: Number(qtyPrev) === 0 ? 0 : (q?.cost_base ?? 0),
      quantity: qty,
      cost: Number(qty) === 0 ? 0 : (p?.cost_base ?? 0),
      country:
        asset.class === 'stock_intl' || asset.class === 'etf_intl'
          ? 'Exterior (confira o país)'
          : FOREIGN.has(asset.class)
            ? 'Estados Unidos'
            : 'Brasil',
    });
  }
  holdings.sort(
    (a, b) =>
      a.group_code.localeCompare(b.group_code) || (a.ticker ?? '').localeCompare(b.ticker ?? ''),
  );
  // O mock não leva prejuízo de anos anteriores: só o do próprio ano passa adiante.
  const foreignTax = roundAway((Math.max(foreignRealized, 0) + foreignIncome) * RULES.foreignRate);
  const foreignCreditUsed = Math.min(foreignCredit, foreignTax);
  const sortLines = (m: Map<string, IncomeLine>) =>
    [...m.values()].sort((a, b) => a.asset_id.localeCompare(b.asset_id));
  return {
    year,
    member_id: memberId,
    months,
    loss_common: pool('common'),
    loss_day_trade: pool('day_trade'),
    loss_fii: pool('fii'),
    irrf_lei_11033_left:
      all.filter((m) => m.month.startsWith(`${year}-`)).at(-1)?.irrf_carried_out ?? 0,
    irrf_day_trade_left: 0,
    total_tax: months.reduce((s, m) => s + m.tax, 0),
    total_darf: months.reduce((s, m) => s + m.darf_amount, 0),
    exempt_stock_gains: months
      .flatMap((m) => m.categories)
      .filter((c) => c.category === 'stocks')
      .reduce((s, c) => s + c.exempt_gain, 0),
    dividends: sortLines(div),
    dividends_over_limit: [],
    fii_income: sortLines(fii),
    jcp: sortLines(jcp),
    holdings,
    foreign: {
      realized: foreignRealized,
      income: foreignIncome,
      tax_paid_abroad: foreignTaxPaid,
      tax_credit: foreignCreditUsed,
      tax_without_credit: 0,
      loss_carried_in: 0,
      loss_carried_out: Math.max(-foreignRealized, 0),
      estimated_tax: foreignTax - foreignCreditUsed,
    },
    disclaimer: DISCLAIMER,
  };
}

// ── DARFs ────────────────────────────────────────────────────────

function darf(id: string): Darf {
  return st().darfs.find((d) => d.id === id) ?? notFound(`DARF ${id}`);
}

function memberArg(a: Record<string, unknown>): string | null {
  return optStr(a, 'member_id') ?? null;
}

registerMockHandlers(
  guarded({
    'tax.month': (a) => taxMonth(memberArg(a), monthArg(a)),
    'tax.year': (a) => {
      const y = a.year;
      if (typeof y !== 'number' || !Number.isInteger(y))
        invalid('argumentos inválidos: missing field `year`');
      return yearReport(memberArg(a), y);
    },
    'tax.list_darfs': (a) => {
      const member = memberArg(a);
      const year = typeof a.year === 'number' ? String(a.year) : null;
      return st()
        .darfs.filter((d) => !member || d.member_id === member)
        .filter((d) => !year || d.period.startsWith(year))
        .sort((x, y) => y.period.localeCompare(x.period));
    },
    // O mock não apura cripto no Brasil: nenhuma conta com cripto para escolher.
    'tax.crypto_custody': () => [],
    'tax.set_crypto_custody': () => [],
    'tax.generate_darf': (a) => {
      // Sem apuração de cripto no mock: não há DARF 4600 a gerar.
      if (a.revenue_code === '4600') invalid('não há imposto a pagar neste mês');
      const member = memberArg(a);
      const m = taxMonth(member, monthArg(a));
      if (m.darf_amount <= 0)
        invalid(
          m.status === 'below_minimum'
            ? 'imposto abaixo de R$ 10,00: acumula para o mês seguinte'
            : 'não há imposto a pagar neste mês',
        );
      if (m.darf_id)
        conflict('já existe um DARF para este mês; cancele o anterior para gerar outro');
      const d = newDarf(member, m);
      st().darfs.push(d);
      return d;
    },
    'tax.darf_charges': (a) => {
      const d = darf(str(a, 'id'));
      return lateCharges(d.principal, d.due_date, optStr(a, 'paid_at') ?? today());
    },
    'tax.mark_darf_paid': (a) => {
      const d = darf(str(a, 'id'));
      const paidAt = str(a, 'paid_at');
      if (!isIsoDate(paidAt)) invalid(`data inválida: ${paidAt}`);
      const late = lateCharges(d.principal, d.due_date, paidAt);
      Object.assign(d, { fine: late.fine, interest: late.interest, total: late.total });
      const acc = optStr(a, 'from_account_id');
      let txnId = optStr(a, 'txn_id') ?? null;
      if (!txnId && acc) {
        txnId = write(`Pagamento do DARF de ${d.period}`, () => {
          const { currency } = currencyAndKind(acc);
          return insertTxn({
            account_id: acc,
            date: paidAt,
            amount: -d.total,
            currency,
            kind: 'expense',
            category_id: 'cat-impostos-ir',
            description: `DARF ${d.revenue_code} — IR renda variável ${d.period}`,
          }).id;
        }).value;
      }
      Object.assign(d, { status: 'paid', paid_at: paidAt, txn_id: txnId });
      return d;
    },
    'tax.cancel_darf': (a) => {
      const d = darf(str(a, 'id'));
      if (d.status === 'paid') conflict('DARF pago não pode ser cancelado');
      d.status = 'cancelled';
      return d;
    },
  }),
);
