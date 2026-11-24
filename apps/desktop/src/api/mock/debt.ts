/**
 * Mock de `debt.*`: três contratos da Família Exemplo — o financiamento do apartamento
 * (SAC + TR, Caixa, com uma amortização pelo FGTS), o CDC do carro (Price, Itaú, com a
 * parcela de outubro atrasada) e um consignado (Price, Nubank). O cronograma é calculado
 * como no núcleo (`debt/schedule.rs`), a TR e o IPCA vêm das séries do mock de índices,
 * e amortizações com conta de origem viram lançamentos no livro-caixa do mock.
 */
import type { AmortSystem } from '@/api/generated/AmortSystem';
import type { DebtSummary } from '@/api/generated/DebtSummary';
import type { Installment } from '@/api/generated/Installment';
import type { LoanIndexer } from '@/api/generated/LoanIndexer';
import type { LoanInput } from '@/api/generated/LoanInput';
import type { LoanKind } from '@/api/generated/LoanKind';
import type { LoanPaymentCandidate } from '@/api/generated/LoanPaymentCandidate';
import type { LoanView } from '@/api/generated/LoanView';
import type { PortabilitySimulation } from '@/api/generated/PortabilitySimulation';
import type { PrepayMode } from '@/api/generated/PrepayMode';
import type { PrepaySimulation } from '@/api/generated/PrepaySimulation';
import type { SpreadMode } from '@/api/generated/SpreadMode';
import type { CorrectionReference } from '@/api/generated/CorrectionReference';
import type { InterestBasis } from '@/api/generated/InterestBasis';
import type { Prorata } from '@/api/generated/Prorata';
import type { UpfrontCosts } from '@/api/generated/UpfrontCosts';

import { rateString, solveLoanRate } from '@/lib/loanRate';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import {
  build,
  monthlyRate,
  round,
  type Extra,
  type Params,
  type Period,
  type Row,
} from './debt/schedule';
import { contractCet, realizedCet, upfrontTotal } from './debt/cet';
import { guarded } from './ledger/boot';
import { addDays, addMonths, diffDays, isIsoDate, monthOf, today } from './ledger/dates';
import { checkCategory, currencyAndKind, insertTxn } from './ledger/domain';
import { conflict, invalid, notFound, num, optStr, str } from './ledger/errors';
import { find, patch, rows, write } from './ledger/store';
import { setLoanLookup, type LoanLink } from './ledger/loanLinks';
import { deflatorFactor, seriesPoints } from './marketSim';

const KINDS: LoanKind[] = ['mortgage', 'vehicle', 'personal', 'payroll', 'student', 'other'];
const MODES: PrepayMode[] = ['reduce_term', 'reduce_installment'];

interface ExtraRec {
  id: string;
  date: string;
  amount: number;
  mode: PrepayMode;
  txn_id: string | null;
}

interface InstRec extends Row {
  id: string;
  /** Valores do extrato do banco (importado ou informados no pagamento). */
  statement?: boolean;
  paid: boolean;
  paid_date: string | null;
  paid_amount: number | null;
  txn_id: string | null;
}

interface LoanRec {
  id: string;
  name: string;
  contract_number: string | null;
  kind: LoanKind;
  institution_id: string | null;
  principal: number;
  annual_rate: string;
  system: AmortSystem;
  index_pct: string;
  spread_mode: SpreadMode;
  index_lag_days: number;
  correction_lag_months: number;
  correction_prorata: Prorata;
  deflation_floor: boolean;
  interest_basis: InterestBasis;
  correction_reference: CorrectionReference;
  indexer: LoanIndexer;
  contract_date: string;
  first_due_date: string;
  term_months: number;
  grace_months: number;
  insurance_monthly: number;
  fees_monthly: number;
  property_value: number | null;
  mip_rate: string | null;
  dfi_rate: string | null;
  mip_amount: number | null;
  dfi_amount: number | null;
  /** Extrato importado: última parcela com valores do banco e de onde a projeção parte. */
  rebase?: { number: number; balance: number; term: number } | null;
  cet_annual: string | null;
  /** Costs paid on day zero (IOF, one-time fees, single-premium insurance). */
  upfront: UpfrontCosts;
  debit_account_id: string | null;
  category_id: string | null;
  member_id: string | null;
  archived: boolean;
  notes: string | null;
  extras: ExtraRec[];
  installments: InstRec[];
}

interface State {
  today: string;
  loans: LoanRec[];
  seq: number;
}

let state: State | null = null;

onMockReset(() => {
  state = null;
});

function nextId(prefix: string): string {
  const s = st();
  s.seq += 1;
  return `${prefix}-${s.seq}`;
}

// ── Cronograma ───────────────────────────────────────────────────

const SERIES: Record<LoanIndexer, string | null> = {
  none: null,
  tr: 'tr',
  ipca: 'ipca',
  igpm: 'igpm',
  // Pós de taxa diária: entram nos juros de cada período, não na correção do saldo.
  cdi: null,
  selic: null,
  inpc: 'inpc',
  incc: 'incc',
  poupanca: 'poupanca',
};

/** CDI/Selic do mock: 14,90% a.a. (≈ 0,05501% a.d.); só fins de semana fora (sem feriados). */
const MOCK_DAILY_PCT = 0.05501;

function isWeekday(iso: string): boolean {
  const d = new Date(`${iso}T12:00:00Z`).getUTCDay();
  return d !== 0 && d !== 6;
}

function nextWeekday(iso: string): string {
  let d = iso;
  while (!isWeekday(d)) d = addDays(d, 1);
  return d;
}

/** Espelho simplificado de `postfixed.rs`: fator por período com CDI constante. */
function postPeriods(l: LoanRec, count: number, withIndex = true): Period[] {
  if (l.indexer !== 'cdi' && l.indexer !== 'selic') return [];
  const pct = Number(l.index_pct) / 100;
  const s = Number(l.annual_rate) / 100;
  const out: Period[] = [];
  let start = l.contract_date;
  for (let k = 0; k < count; k++) {
    const due = nextWeekday(addMonths(l.first_due_date, k));
    let du = 0;
    for (let d = start; d < due; d = addDays(d, 1)) if (isWeekday(d)) du += 1;
    const fdi = withIndex ? (1 + (MOCK_DAILY_PCT / 100) * pct) ** du : 1;
    const days = diffDays(due, start);
    const spread = l.spread_mode === 'lin360' ? 1 + (s * days) / 360 : (1 + s) ** (du / 252);
    out.push({ due, factor: fdi * spread, businessDays: du, estimated: due > today() });
    start = due;
  }
  return out;
}

/** Variação do índice no mês anterior a cada vencimento; sem dado, a última conhecida (estimada). */
function indexFactors(l: LoanRec, count: number): [number, boolean][] {
  const series = SERIES[l.indexer];
  if (!series) return [];
  const byMonth = new Map(seriesPoints(series).map((p) => [monthOf(p.date), p]));
  let last = 0;
  const out: [number, boolean][] = [];
  for (let k = 0; k < count; k++) {
    const ref = monthOf(addMonths(addMonths(l.first_due_date, k), -1));
    const p = byMonth.get(ref);
    if (p) {
      last = p.value;
      out.push([1 + p.value / 100, p.estimated]);
    } else out.push([1 + last / 100, true]);
  }
  return out;
}

/** Depois de quantas parcelas vencidas até `date` a amortização entra. */
function afterNumber(firstDue: string, date: string): number {
  let n = 0;
  while (addMonths(firstDue, n) <= date) n += 1;
  return n;
}

function paramsFor(l0: LoanRec, extra?: Extra): Params {
  // Extrato importado: a projeção parte do último saldo real, com o prazo que falta.
  const rb = l0.rebase;
  const l: LoanRec = rb
    ? {
        ...l0,
        contract_date: addMonths(l0.first_due_date, rb.number - 1),
        first_due_date: addMonths(l0.first_due_date, rb.number),
        principal: rb.balance,
        term_months: rb.term,
        grace_months: 0,
        correction_prorata: 'none',
      }
    : l0;
  const extras: Extra[] = l.extras
    .filter((e) => !rb || e.date > l.contract_date)
    .map((e) => ({
      afterNumber: afterNumber(l.first_due_date, e.date),
      amount: e.amount,
      mode: e.mode,
    }));
  if (extra) extras.push(extra);
  return {
    principal: l.principal,
    annualRatePct: Number(l.annual_rate),
    system: l.system,
    term: l.term_months,
    grace: l.grace_months,
    firstDue: l.first_due_date,
    insurance: l.insurance_monthly,
    fees: l.fees_monthly,
    ...housing(l),
    indexFactors: indexFactors(l, l.term_months + 120),
    extras,
    periods: postPeriods(l, l.term_months + 120),
  };
}

/** Contract as signed with the floating index at zero (art. 5º of Res. 4.881), for the CET. */
function contractParams(l: LoanRec): Params {
  return {
    principal: l.principal,
    annualRatePct: Number(l.annual_rate),
    system: l.system,
    term: l.term_months,
    grace: l.grace_months,
    firstDue: l.first_due_date,
    insurance: l.insurance_monthly,
    fees: l.fees_monthly,
    ...housing(l),
    indexFactors: [],
    extras: [],
    periods: postPeriods(l, l.term_months, false),
  };
}

const INDEX_LABEL: Record<LoanIndexer, string | null> = {
  none: null,
  tr: 'TR',
  ipca: 'IPCA',
  igpm: 'IGP-M',
  cdi: 'CDI',
  selic: 'Selic',
  inpc: 'INPC',
  incc: 'INCC',
  poupanca: 'Poupança',
};

/** Seguros do financiamento imobiliário: MIP em fração do saldo e DFI do mês. */
function housing(l: LoanRec): { mipRate: number; mipFixed: number; dfi: number } {
  const pct = (v: string | null) => (v ? Number(v) / 100 : 0);
  return {
    mipRate: l.mip_amount ? 0 : pct(l.mip_rate),
    mipFixed: l.mip_amount ?? 0,
    dfi: l.dfi_amount ?? round((l.property_value ?? 0) * pct(l.dfi_rate)),
  };
}

/** Espelho de `debt::set_housing_insurance`: valida e grava valor do imóvel, MIP e DFI. */
function setHousing(
  l: LoanRec,
  propertyValue: number | null,
  mip: string | null,
  dfi: string | null,
  mipAmount: number | null = null,
  dfiAmount: number | null = null,
): void {
  const clean = (v: string | null, what: string): string | null => {
    if (v == null || !v.trim()) return null;
    const n = Number(v.trim().replace(',', '.'));
    if (!Number.isFinite(n) || n < 0 || n > 5)
      invalid(`${what}: informe a taxa ao mês em % (ex.: 0,0247)`);
    return String(n);
  };
  if (propertyValue != null && propertyValue < 0)
    invalid('o valor do imóvel não pode ser negativo');
  if ((mipAmount ?? 0) < 0 || (dfiAmount ?? 0) < 0)
    invalid('o valor do seguro não pode ser negativo');
  const ma = mipAmount && mipAmount > 0 ? mipAmount : null;
  const da = dfiAmount && dfiAmount > 0 ? dfiAmount : null;
  const m = ma ? null : clean(mip, 'taxa do MIP');
  const d = da ? null : clean(dfi, 'taxa do DFI');
  if (d && !propertyValue)
    invalid(
      'o DFI em percentual é calculado sobre o valor do imóvel: informe o valor do imóvel (ou o DFI em reais)',
    );
  l.property_value = propertyValue;
  l.mip_rate = m;
  l.dfi_rate = d;
  l.mip_amount = ma;
  l.dfi_amount = da;
}

// ── Extrato do contrato (espelho de `debt::statement`) ─────────────────────────

const money = (s: string | undefined): number | null => {
  if (!s) return null;
  const t = s.replace(/[^\d,.-]/g, '');
  if (!t || t === '-') return null;
  const n = Number(t.includes(',') ? t.replace(/\./g, '').replace(',', '.') : t);
  return Number.isFinite(n) ? Math.round(n * 100) : null;
};
const brDate = (s: string | undefined): string | null => {
  const m = /^(\d{2})\/(\d{2})\/(\d{4})$/.exec((s ?? '').trim());
  return m ? `${m[3]}-${m[2]}-${m[1]}` : isIsoDate((s ?? '').trim()) ? s!.trim() : null;
};
const fold = (s: string) => s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
type ColKey =
  'date' | 'total' | 'amort' | 'interest' | 'ins' | 'fees' | 'bal' | 'adj' | 'paid' | 'skip';
const colOf = (h: string): ColKey => {
  const t = fold(h);
  if (t.startsWith('data')) return 'date';
  if (t.includes('amortiza')) return 'amort';
  if (t.includes('juros')) return 'interest';
  if (t.includes('seguro')) return 'ins';
  if (t.includes('taxa') || t.includes('tarifa')) return 'fees';
  if (t.includes('saldo')) return 'bal';
  if (t.includes('correc') || t.includes('atualiza')) return 'adj';
  if (t.includes('pago')) return 'paid';
  if (t.includes('prestac') || (t.includes('encargo') && !t.includes('liquid'))) return 'total';
  return 'skip';
};
const CAIXA_COLS: ColKey[] = [
  'date',
  'total',
  'skip',
  'amort',
  'interest',
  'ins',
  'fees',
  'skip',
  'skip',
  'bal',
  'adj',
  'paid',
];

interface StRow {
  date: string;
  total: number;
  amortization: number;
  interest: number;
  insurance: number;
  fees: number;
  adjustment: number;
  balance_after: number;
  paid: number | null;
}

function parseStatement(text: string): { rows: StRow[]; warnings: string[] } {
  let cols: ColKey[] | null = null;
  const rows: StRow[] = [];
  const bad: string[] = [];
  for (const line of text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean)) {
    const c = (
      line.includes('\t')
        ? line.split('\t')
        : line.includes(';')
          ? line.split(';')
          : line.split(/ {2,}/)
    ).map((x) => x.trim());
    if (!cols && !brDate(c[0])) {
      const m = c.map(colOf);
      if (m.includes('date') && m.includes('bal')) cols = m;
      continue;
    }
    const map = cols ?? CAIXA_COLS;
    const get = (k: ColKey) => {
      const i = map.indexOf(k);
      return i >= 0 ? c[i] : undefined;
    };
    const d = brDate(get('date'));
    const am = money(get('amort'));
    const ju = money(get('interest'));
    const bal = money(get('bal'));
    if (!d || am == null || ju == null || bal == null) {
      bad.push(line.slice(0, 80));
      continue;
    }
    const ins = money(get('ins')) ?? 0;
    const fees = money(get('fees')) ?? 0;
    rows.push({
      date: d,
      total: money(get('total')) ?? am + ju + ins + fees,
      amortization: am,
      interest: ju,
      insurance: ins,
      fees,
      adjustment: money(get('adj')) ?? 0,
      balance_after: bal,
      paid: money(get('paid')),
    });
  }
  rows.sort((a, b) => a.date.localeCompare(b.date));
  return {
    rows,
    warnings: bad.length
      ? [`${bad.length} linha(s) não entendida(s), por exemplo: "${bad[0]}"`]
      : [],
  };
}

function statementPreview(l: LoanRec, a: Record<string, unknown>) {
  const given = Array.isArray(a.rows) ? (a.rows as StRow[]) : null;
  const parsed = given?.length
    ? { rows: [...given].sort((x, y) => x.date.localeCompare(y.date)), warnings: [] as string[] }
    : typeof a.text === 'string'
      ? parseStatement(a.text)
      : invalid('cole o extrato do contrato (ou envie as linhas)');
  const { rows, warnings } = parsed;
  if (!rows.length)
    invalid(
      'nenhuma linha do extrato foi entendida: cole com o cabeçalho (data, amortização, juros, saldo devedor…)',
    );
  const extras: { date: string; amount: number }[] = [];
  for (let k = 1; k < rows.length; k += 1) {
    const prev = rows[k - 1]!;
    const cur = rows[k]!;
    const extra = prev.balance_after + cur.adjustment - cur.amortization - cur.balance_after;
    if (extra > Math.max(5_000, cur.total / 2)) extras.push({ date: cur.date, amount: extra });
  }
  const recent = rows
    .slice(-6)
    .filter((r) => r.balance_after + r.amortization > 0 && r.interest > 0)
    .map((r) => r.interest / (r.balance_after + r.amortization));
  const m = recent.length ? recent.reduce((x, y) => x + y, 0) / recent.length : null;
  const last = rows.at(-1)!;
  let remaining = typeof a.remaining_term === 'number' ? a.remaining_term : null;
  if (remaining == null) {
    const before = last.balance_after + last.amortization;
    if (l.system === 'sac' && last.amortization > 0)
      remaining = Math.round(before / last.amortization) - 1;
    else if (before > 0 && last.interest > 0) {
      const i = last.interest / before;
      const x = 1 - (i * before) / (last.amortization + last.interest);
      if (x > 0) remaining = Math.round(-Math.log(x) / Math.log(1 + i)) - 1;
    }
  }
  const contractM = (1 + Number(l.annual_rate) / 100) ** (1 / 12) - 1;
  const eff = m != null ? ((1 + m) ** 12 - 1) * 100 : null;
  const rateMismatch = m != null && Math.abs(m - contractM) > 0.00002;
  if (rateMismatch)
    warnings.push(
      `os juros do extrato equivalem a ${(m * 100).toFixed(4)}% ao mês (${(m * 1200).toFixed(2)}% a.a. nominal, ${eff!.toFixed(4)}% a.a. efetiva); o contrato está com ${l.annual_rate}% a.a. efetiva. Ajuste a taxa para a projeção bater com o banco`,
    );
  if (remaining == null)
    warnings.push(
      'não deu para calcular o prazo que falta pela última prestação: informe o prazo remanescente',
    );
  return {
    rows,
    first_date: rows[0]!.date,
    last_date: last.date,
    last_balance: last.balance_after,
    remaining_term: remaining,
    extras,
    implied_monthly_pct: m != null ? String(Math.round(m * 1e6) / 1e4) : null,
    implied_nominal_annual_pct: m != null ? String(Math.round(m * 1200 * 100) / 100) : null,
    implied_effective_annual_pct: eff != null ? String(Math.round(eff * 1e4) / 1e4) : null,
    contract_rate_pct: l.annual_rate,
    warnings,
    rate_mismatch: rateMismatch,
  };
}

function importStatement(l: LoanRec, a: Record<string, unknown>) {
  const p = statementPreview(l, a);
  if (p.remaining_term == null)
    invalid('informe o prazo remanescente (quantas parcelas faltam depois da última linha)');
  let last = 0;
  for (const r of p.rows) {
    const n =
      (Number(r.date.slice(0, 4)) - Number(l.first_due_date.slice(0, 4))) * 12 +
      Number(r.date.slice(5, 7)) -
      Number(l.first_due_date.slice(5, 7)) +
      1;
    if (n < 1) invalid('a linha é anterior ao primeiro vencimento do contrato');
    const extra = p.extras.find((e) => e.date === r.date)?.amount ?? 0;
    const cur = l.installments.find((i) => i.number === n);
    const rec: InstRec = {
      ...(cur ?? {
        id: nextId('inst'),
        insurance_mip: 0,
        insurance_dfi: 0,
        index_factor: null,
        business_days: null,
        txn_id: null,
      }),
      number: n,
      due_date: r.date,
      amortization: r.amortization,
      interest: r.interest,
      adjustment: r.adjustment,
      insurance: r.insurance,
      insurance_mip: 0,
      insurance_dfi: 0,
      fees: r.fees,
      total: r.total,
      balance_after: r.balance_after,
      estimated: false,
      extra_payment: extra,
      paid: true,
      paid_date: cur?.paid_date ?? r.date,
      paid_amount: r.paid ?? r.total,
      statement: true,
    };
    l.installments = [...l.installments.filter((i) => i.number !== n), rec];
    last = Math.max(last, n);
  }
  let marked = 0;
  if (a.mark_previous_paid !== false) {
    for (const i of l.installments)
      if (!i.paid && i.due_date < p.first_date) {
        i.paid = true;
        i.paid_date = i.due_date;
        i.paid_amount = i.total;
        marked += 1;
      }
  }
  l.rebase = { number: last, balance: p.last_balance, term: p.remaining_term };
  regenerate(l);
  return { preview: p, imported: p.rows.length, marked_previous: marked };
}

/** Regrava as parcelas não pagas a partir do cronograma recalculado (as pagas ficam). */
function regenerate(l: LoanRec): void {
  const keep = l.rebase?.number ?? 0;
  const paid = l.installments.filter((i) => i.paid || i.number <= keep);
  const paidNumbers = new Set(paid.map((i) => i.number));
  const fresh = build(paramsFor(l))
    .map((r) => ({ ...r, number: r.number + keep }))
    .filter((r) => !paidNumbers.has(r.number) && r.number > keep)
    .map((r): InstRec => ({
      ...r,
      id: nextId('inst'),
      paid: false,
      paid_date: null,
      paid_amount: null,
      txn_id: null,
    }));
  l.installments = [...paid, ...fresh].sort((a, b) => a.number - b.number);
}

/** Versões do contrato, como o núcleo: amortização do extrato abre a versão na própria
 *  parcela; a do cronograma, na seguinte. */
function withVersions(list: InstRec[], t: string): Installment[] {
  let version = 1;
  let k = 0;
  let prevSchedule = false;
  return [...list]
    .sort((a, b) => a.number - b.number)
    .map((i) => {
      const starts = (i.statement === true && i.extra_payment > 0) || prevSchedule;
      if (starts && k > 0) {
        version += 1;
        k = 0;
      }
      k += 1;
      prevSchedule = i.statement !== true && i.extra_payment > 0;
      return { ...toInstallment(i, t), version, version_number: k };
    });
}

function toInstallment(i: InstRec, t: string): Installment {
  const txn = i.txn_id ? find('txn', i.txn_id) : undefined;
  return {
    id: i.id,
    number: i.number,
    version: 1,
    version_number: i.number,
    due_date: i.due_date,
    amortization: i.amortization,
    interest: i.interest,
    adjustment: i.adjustment,
    insurance: i.insurance,
    insurance_mip: i.insurance_mip,
    insurance_dfi: i.insurance_dfi,
    fees: i.fees,
    total: i.total,
    balance_after: i.balance_after,
    extra_payment: i.extra_payment,
    estimated: i.estimated,
    paid: i.paid,
    paid_date: i.paid_date,
    paid_amount: i.paid_amount,
    txn_id: i.txn_id,
    txn_date: txn?.date ?? null,
    txn_description: txn?.description ?? null,
    txn_account_id: txn?.account_id ?? null,
    txn_account_name: txn ? (find('account', txn.account_id)?.name ?? null) : null,
    overdue: !i.paid && i.due_date < t,
    index_factor: i.index_factor,
    business_days: i.business_days,
  };
}

function balanceAt(l: LoanRec, at: string): number {
  if (l.contract_date > at) return 0;
  const last = [...l.installments].reverse().find((i) => i.due_date <= at);
  return last ? last.balance_after : l.principal;
}

function view(l: LoanRec): LoanView {
  const t = today();
  const inst = withVersions(l.installments, t);
  const paid = inst.filter((i) => i.paid);
  const balance = balanceAt(l, t);
  const realizedCost = realizedCet(
    l.principal - upfrontTotal(l.upfront),
    l.principal,
    l.contract_date,
    inst,
  );
  const realized = realizedCost.rate;
  let realCost: number | null = null;
  if (realized != null) {
    const f = deflatorFactor('ipca', l.contract_date, t);
    if (f) {
      const days = Math.max(diffDays(t, l.contract_date), 1);
      const infl = f.factor ** (365 / days) - 1;
      realCost = (1 + realized) / (1 + infl) - 1;
    }
  }
  return {
    id: l.id,
    name: l.name,
    contract_number: l.contract_number,
    kind: l.kind,
    institution_id: l.institution_id,
    principal: l.principal,
    currency: 'BRL',
    annual_rate: l.annual_rate,
    index_pct: l.index_pct,
    spread_mode: l.spread_mode,
    index_lag_days: l.index_lag_days,
    correction_lag_months: l.correction_lag_months,
    correction_prorata: l.correction_prorata,
    deflation_floor: l.deflation_floor,
    interest_basis: l.interest_basis,
    correction_reference: l.correction_reference,
    monthly_rate: String(Number((monthlyRate(Number(l.annual_rate)) * 100).toFixed(4))),
    system: l.system,
    indexer: l.indexer,
    contract_date: l.contract_date,
    first_due_date: l.first_due_date,
    term_months: l.term_months,
    grace_months: l.grace_months,
    insurance_monthly: l.insurance_monthly,
    property_value: l.property_value,
    mip_rate: l.mip_rate,
    dfi_rate: l.dfi_rate,
    mip_amount: l.mip_amount,
    dfi_amount: l.dfi_amount,
    statement_until: l.rebase?.number ?? null,
    fees_monthly: l.fees_monthly,
    cet_annual: l.cet_annual,
    upfront: l.upfront,
    cet: contractCet(
      build(contractParams(l)),
      l.contract_date,
      l.principal,
      l.upfront,
      INDEX_LABEL[l.indexer],
      l.cet_annual,
    ),
    debit_account_id: l.debit_account_id,
    category_id: l.category_id,
    member_id: l.member_id,
    archived: l.archived,
    notes: l.notes,
    balance,
    paid_count: paid.length,
    remaining_count: inst.length - paid.length,
    overdue_count: inst.filter((i) => i.overdue).length,
    next_installment: inst.find((i) => !i.paid) ?? null,
    interest_paid: paid.reduce((s, i) => s + i.interest, 0),
    interest_remaining: inst.filter((i) => !i.paid).reduce((s, i) => s + i.interest, 0),
    charges_paid: paid.reduce((s, i) => s + i.insurance + i.fees, 0),
    realized_cet: realized,
    realized_drops: realizedCost.drops,
    real_cost: realCost,
  };
}

// ── Seed ─────────────────────────────────────────────────────────

/** Mesmo id do seed do livro-caixa (`inst-<nome>`). */
const instId = (name: string) => `inst-${name.toLowerCase().replace(/\W+/g, '-')}`;

type Seed = Omit<
  LoanRec,
  | 'id'
  | 'archived'
  | 'extras'
  | 'installments'
  | 'index_pct'
  | 'spread_mode'
  | 'index_lag_days'
  | 'correction_lag_months'
  | 'correction_prorata'
  | 'deflation_floor'
  | 'interest_basis'
  | 'correction_reference'
  | 'category_id'
  | 'property_value'
  | 'mip_rate'
  | 'dfi_rate'
  | 'mip_amount'
  | 'dfi_amount'
  | 'upfront'
> & {
  upfront?: UpfrontCosts;
  extras: Omit<ExtraRec, 'id'>[];
  /** Parcelas vencidas que ficam em aberto (atrasadas): os n últimos vencimentos. */
  unpaidRecent?: number;
};

const SEEDS: Seed[] = [
  {
    name: 'Financiamento do apartamento',
    contract_number: '8.4444.0123456-7',
    kind: 'mortgage',
    institution_id: instId('Caixa'),
    principal: 42_000_000,
    annual_rate: '9.5',
    system: 'sac',
    indexer: 'tr',
    contract_date: '2023-03-15',
    first_due_date: '2023-04-15',
    term_months: 360,
    grace_months: 0,
    insurance_monthly: 11_850,
    fees_monthly: 2_500,
    cet_annual: '10.42',
    debit_account_id: 'acc-itau',
    member_id: 'mem-ana',
    notes: 'SFH · TR + 9,5% a.a. · seguros MIP e DFI na parcela.',
    extras: [
      { date: '2025-06-20', amount: 2_000_000, mode: 'reduce_term', txn_id: null }, // FGTS
    ],
  },
  {
    name: 'CDC do Onix',
    contract_number: null,
    kind: 'vehicle',
    institution_id: instId('Itaú Unibanco'),
    principal: 6_500_000,
    annual_rate: '22.4',
    system: 'price',
    indexer: 'none',
    contract_date: '2024-08-02',
    first_due_date: '2024-09-05',
    term_months: 48,
    grace_months: 0,
    insurance_monthly: 0,
    fees_monthly: 0,
    cet_annual: '24.9',
    upfront: { iof: 168_400, fees: 85_000, insurance: 0 },
    debit_account_id: 'acc-itau',
    member_id: 'mem-bruno',
    notes: null,
    extras: [],
    unpaidRecent: 1,
  },
  {
    name: 'Consignado',
    contract_number: null,
    kind: 'payroll',
    institution_id: instId('Nubank'),
    principal: 2_500_000,
    annual_rate: '19.6',
    system: 'price',
    indexer: 'none',
    contract_date: '2025-11-10',
    first_due_date: '2025-12-07',
    term_months: 36,
    grace_months: 0,
    insurance_monthly: 0,
    fees_monthly: 0,
    cet_annual: '20.8',
    debit_account_id: 'acc-nuconta',
    member_id: 'mem-bruno',
    notes: 'Desconto em folha.',
    extras: [],
  },
];

const NO_UPFRONT: UpfrontCosts = { iof: 0, fees: 0, insurance: 0 };

function seed(t: string): State {
  state = { today: t, loans: [], seq: 0 };
  SEEDS.forEach((s, n) => {
    const { unpaidRecent = 0, extras, upfront = NO_UPFRONT, ...rest } = s;
    const l: LoanRec = {
      index_pct: '100',
      spread_mode: 'exp252',
      index_lag_days: 0,
      correction_lag_months: 1,
      correction_prorata: 'none',
      deflation_floor: false,
      interest_basis: 'monthly',
      correction_reference: rest.indexer === 'tr' ? 'anniversary' : 'month',
      category_id: null,
      property_value: null,
      mip_rate: null,
      dfi_rate: null,
      mip_amount: null,
      dfi_amount: null,
      ...rest,
      upfront,
      id: `loan-${n + 1}`,
      archived: false,
      extras: extras.filter((e) => e.date <= t).map((e) => ({ ...e, id: nextId('extra') })),
      installments: [],
    };
    regenerate(l);
    const due = l.installments.filter((i) => i.due_date < t);
    due.slice(0, due.length - unpaidRecent).forEach((i, k) => {
      i.paid = true;
      // Débito automático: no vencimento, às vezes um dia depois.
      i.paid_date = k % 7 === 3 ? addDays(i.due_date, 1) : i.due_date;
      i.paid_amount = i.total;
    });
    state!.loans.push(l);
  });
  return state;
}

function st(): State {
  const t = today();
  if (!state || state.today !== t) return seed(t);
  return state;
}

function loan(id: string): LoanRec {
  return st().loans.find((l) => l.id === id) ?? notFound(`empréstimo ${id}`);
}

// ── Validação (mensagens do núcleo) ──────────────────────────────

function rate(v: unknown, what: string): number {
  const n = typeof v === 'string' ? Number(v.trim().replace(',', '.')) : NaN;
  if (!Number.isFinite(n)) invalid(`${what} inválido: ${String(v)}`);
  return n;
}

function date(v: unknown): string {
  if (typeof v !== 'string' || !isIsoDate(v)) invalid(`data inválida: ${String(v)}`);
  return v;
}

function cleanNumber(n: string | null | undefined): string | null {
  const t = n?.trim();
  return t ? t.slice(0, 80) : null;
}

function solvedRate(i: LoanInput): number {
  if (i.installment_amount == null)
    invalid('informe a taxa anual ou o valor da parcela para calcular a taxa');
  const s = solveLoanRate({
    principal: i.principal,
    installment: i.installment_amount,
    term: i.term_months,
    grace: i.grace_months ?? 0,
    system: i.system,
    charges: (i.insurance_monthly ?? 0) + (i.fees_monthly ?? 0),
  });
  if (!s)
    invalid(
      'com essa parcela e esse prazo a taxa não fecha: confira os valores (a parcela vezes o prazo precisa passar do valor financiado)',
    );
  return Number(rateString(s.annualPct));
}

/** Marca como pagas (no vencimento, pelo valor) as parcelas abertas vencidas antes de `before`. */
function markPaidUntil(l: LoanRec, before: string): number {
  let n = 0;
  for (const i of l.installments) {
    if (!i.paid && i.due_date < before) {
      i.paid = true;
      i.paid_date = i.due_date;
      i.paid_amount = i.total;
      n += 1;
    }
  }
  return n;
}

/** Espelho de `simulate_anticipation`: valor presente por dias corridos ÷ 30. */
function simulateAnticipation(a: Record<string, unknown>) {
  const l = loan(str(a, 'loan_id'));
  const d = date(a.date);
  const count = num(a, 'count');
  const order = a.order === 'next' ? 'next' : 'last';
  if (!(count > 0)) invalid('informe quantas parcelas antecipar');
  const open = l.installments.filter((i) => !i.paid && i.due_date > d);
  if (open.length === 0) invalid('não há parcelas futuras em aberto para antecipar');
  if (count > open.length) invalid(`só há ${open.length} parcela(s) futura(s) em aberto`);
  const picked = order === 'next' ? open.slice(0, count) : open.slice(open.length - count);
  const i = monthlyRate(Number(l.annual_rate));
  const installments = picked.map((p) => {
    const nominal = p.total - p.insurance - p.fees;
    const rate = p.index_factor ? Number(p.index_factor) - 1 : i;
    const pv = Math.round(nominal / (1 + rate) ** (diffDays(p.due_date, d) / 30));
    return { number: p.number, due_date: p.due_date, nominal, present_value: pv };
  });
  const nominal_total = installments.reduce((s, x) => s + x.nominal, 0);
  const to_pay = installments.reduce((s, x) => s + x.present_value, 0);
  return {
    l,
    sim: {
      installments,
      nominal_total,
      to_pay,
      discount: nominal_total - to_pay,
      charges_dropped: picked.reduce((s, p) => s + p.insurance + p.fees, 0),
      remaining_after: open.length - count,
    },
  };
}

function create(i: LoanInput): LoanRec {
  const l = fromInput(i);
  st().loans.push(l);
  return l;
}

/** Contrato a partir dos termos (validados como no núcleo), sem guardar. */
function fromInput(i: LoanInput): LoanRec {
  if (!i.name?.trim()) invalid('dê um nome ao contrato');
  if (!KINDS.includes(i.kind)) invalid(`argumentos inválidos: unknown variant \`${i.kind}\``);
  if (!(i.principal > 0)) invalid('o valor financiado deve ser positivo');
  const isPost = i.indexer === 'cdi' || i.indexer === 'selic';
  if (isPost && !i.annual_rate.trim() && i.installment_amount != null)
    invalid('nos pós-fixados a parcela varia com o índice: informe o spread (0 se não houver)');
  const r = i.annual_rate.trim() ? rate(i.annual_rate, 'taxa') : isPost ? 0 : solvedRate(i);
  if (r < 0 || r > 1000) invalid('taxa anual fora do intervalo');
  const cd = date(i.contract_date);
  const fd = date(i.first_due_date);
  if (fd < cd) invalid('a primeira parcela não pode vencer antes do contrato');
  if (!(i.term_months > 0) || (i.grace_months ?? 0) >= i.term_months)
    invalid('prazo inválido (a carência deve ser menor que o prazo)');
  if (i.cet_annual) rate(i.cet_annual, 'CET');
  const upfront: UpfrontCosts = {
    iof: i.upfront_iof ?? 0,
    fees: i.upfront_fees ?? 0,
    insurance: i.upfront_insurance ?? 0,
  };
  if (upfront.iof < 0 || upfront.fees < 0 || upfront.insurance < 0)
    invalid('os custos iniciais (IOF, tarifas, seguro) não podem ser negativos');
  if (upfrontTotal(upfront) >= i.principal)
    invalid('os custos iniciais somados precisam ser menores que o valor do contrato');
  if (i.debit_account_id) currencyAndKind(i.debit_account_id);
  const post = i.indexer === 'cdi' || i.indexer === 'selic';
  const l: LoanRec = {
    id: nextId('loan'),
    index_pct: post && i.index_pct ? String(rate(i.index_pct, 'percentual do índice')) : '100',
    spread_mode: i.spread_mode ?? 'exp252',
    index_lag_days: i.index_lag_days ?? 0,
    correction_lag_months: i.correction_lag_months ?? 1,
    correction_prorata: i.correction_prorata ?? 'none',
    deflation_floor: i.deflation_floor ?? false,
    interest_basis: i.interest_basis ?? 'monthly',
    correction_reference:
      i.correction_reference ??
      (i.indexer === 'tr' || i.indexer === 'poupanca' ? 'anniversary' : 'month'),
    name: i.name.trim(),
    contract_number: cleanNumber(i.contract_number),
    kind: i.kind,
    institution_id: i.institution_id ?? null,
    principal: i.principal,
    annual_rate: String(r),
    system: i.system,
    indexer: i.indexer ?? 'none',
    contract_date: cd,
    first_due_date: fd,
    term_months: i.term_months,
    grace_months: i.grace_months ?? 0,
    insurance_monthly: i.insurance_monthly ?? 0,
    property_value: null,
    mip_rate: null,
    dfi_rate: null,
    mip_amount: null,
    dfi_amount: null,
    fees_monthly: i.fees_monthly ?? 0,
    cet_annual: i.cet_annual ?? null,
    upfront,
    debit_account_id: i.debit_account_id ?? null,
    category_id: null,
    member_id: i.member_id ?? null,
    archived: false,
    notes: i.notes ?? null,
    extras: [],
    installments: [],
  };
  if (i.category_id) setCategory(l, i.category_id);
  setHousing(
    l,
    i.property_value ?? null,
    i.mip_rate ?? null,
    i.dfi_rate ?? null,
    i.mip_amount ?? null,
    i.dfi_amount ?? null,
  );
  regenerate(l);
  if (i.mark_past_paid) markPaidUntil(l, today());
  return l;
}

/** Espelho de `debt::set_category`: categoria de despesa, uma por contrato. */
function setCategory(l: LoanRec, category: string | null): void {
  if (category) {
    const c = find('category', category);
    if (!c) notFound('categoria');
    if (c.kind !== 'expense') invalid('a categoria dos pagamentos precisa ser de despesa');
    const other = st().loans.find(
      (x) => x.id !== l.id && !x.archived && x.category_id === category,
    );
    if (other) conflict(`essa categoria já está ligada ao contrato ${other.name}`);
  }
  l.category_id = category;
}

/**
 * Espelho de `debt::link_category_payments`: saída na categoria de um contrato paga a parcela
 * em aberto de vencimento mais próximo (até 20 dias).
 */
export function linkCategoryPayments(txnIds: string[]): number {
  const used = new Set(st().loans.flatMap((x) => x.installments.map((i) => i.txn_id)));
  let n = 0;
  for (const id of txnIds) {
    const t = find('txn', id);
    if (!t || t.amount >= 0 || !t.category_id || used.has(t.id)) continue;
    if (t.status !== 'cleared' && t.status !== 'reconciled') continue;
    const l = st().loans.find((x) => !x.archived && x.category_id === t.category_id);
    if (!l) continue;
    const inst = l.installments
      .filter((i) => !i.paid && Math.abs(diffDays(i.due_date, t.date)) <= 20)
      .sort(
        (a, b) =>
          Math.abs(diffDays(a.due_date, t.date)) - Math.abs(diffDays(b.due_date, t.date)) ||
          a.number - b.number,
      )[0];
    if (!inst) continue;
    inst.paid = true;
    inst.paid_date = t.date;
    inst.paid_amount = -t.amount;
    inst.txn_id = t.id;
    used.add(t.id);
    n += 1;
  }
  return n;
}

function simulatePrepayment(a: Record<string, unknown>): PrepaySimulation {
  const l = loan(str(a, 'loan_id'));
  const amount = num(a, 'amount');
  if (amount <= 0) invalid('o valor da amortização deve ser positivo');
  const d = date(a.date);
  const mode = a.mode as PrepayMode;
  if (!MODES.includes(mode)) invalid(`argumentos inválidos: unknown variant \`${String(mode)}\``);
  const after = afterNumber(l.first_due_date, d);
  const before = build(paramsFor(l));
  const sim = build(paramsFor(l, { afterNumber: after, amount, mode }));
  const interest = (rs: Row[]) =>
    rs.filter((r) => r.number > after).reduce((s, r) => s + r.interest, 0);
  const next = (rs: Row[]) => rs.find((r) => r.number === after + 1)?.total ?? 0;
  const ib = interest(before);
  const ia = interest(sim);
  return {
    mode,
    amount,
    interest_before: ib,
    interest_after: ia,
    interest_saved: ib - ia,
    months_before: before.length,
    months_after: sim.length,
    next_payment_before: next(before),
    next_payment_after: next(sim),
    schedule: sim.map((r) => ({
      ...r,
      id: '',
      paid: false,
      paid_date: null,
      paid_amount: null,
      txn_id: null,
      txn_date: null,
      txn_description: null,
      txn_account_id: null,
      txn_account_name: null,
      overdue: false,
      version: 1,
      version_number: r.number,
    })),
  };
}

function simulatePortability(a: Record<string, unknown>): PortabilitySimulation {
  const l = loan(str(a, 'loan_id'));
  const t = today();
  const remaining = l.installments.filter((i) => !i.paid && i.due_date >= t);
  const balance = balanceAt(l, t);
  const currentTotal = remaining.reduce((s, i) => s + i.total, 0);
  const term = Math.max(typeof a.term_months === 'number' ? a.term_months : remaining.length, 1);
  const system = a.system === 'sac' || a.system === 'price' ? a.system : l.system;
  const rs = build({
    principal: balance,
    annualRatePct: rate(a.annual_rate, 'taxa'),
    system,
    term,
    grace: 0,
    firstDue: remaining[0]?.due_date ?? t,
    insurance: l.insurance_monthly,
    fees: l.fees_monthly,
    ...housing(l),
    indexFactors: [],
    extras: [],
  });
  const newTotal = rs.reduce((s, r) => s + r.total, 0);
  const costs = typeof a.costs === 'number' ? a.costs : 0;
  return {
    balance,
    current_remaining_total: currentTotal,
    new_total: newTotal,
    costs,
    savings: currentTotal - newTotal - costs,
    current_next_payment: remaining[0]?.total ?? 0,
    new_first_payment: rs[0]?.total ?? 0,
    new_term_months: rs.length,
  };
}

/** Liga parcelas vencidas aos pagamentos na conta de débito (±1% e até 7 dias). */
function reconcile(l: LoanRec): number {
  const byCategory = l.category_id
    ? linkCategoryPayments(
        rows('txn')
          .filter((t) => t.category_id === l.category_id && t.amount < 0)
          .sort((a, b) => a.date.localeCompare(b.date))
          .map((t) => t.id),
      )
    : 0;
  if (!l.debit_account_id) {
    if (byCategory > 0) return byCategory;
    invalid('defina a conta de débito (ou a categoria dos pagamentos) do contrato para conciliar');
  }
  const horizon = addDays(today(), 7);
  const used = new Set(st().loans.flatMap((x) => x.installments.map((i) => i.txn_id)));
  let n = byCategory;
  for (const i of l.installments.filter((x) => !x.paid && x.due_date <= horizon)) {
    const tol = Math.max(Math.trunc(i.total / 100), 100);
    const hit = rows('txn')
      .filter(
        (t) =>
          t.account_id === l.debit_account_id &&
          (t.status === 'cleared' || t.status === 'reconciled') &&
          t.amount < 0 &&
          Math.abs(-t.amount - i.total) <= tol &&
          Math.abs(diffDays(t.date, i.due_date)) <= 7 &&
          !used.has(t.id),
      )
      .sort(
        (a, b) => Math.abs(diffDays(a.date, i.due_date)) - Math.abs(diffDays(b.date, i.due_date)),
      )[0];
    if (hit) {
      i.paid = true;
      i.paid_date = hit.date;
      i.paid_amount = -hit.amount;
      i.txn_id = hit.id;
      used.add(hit.id);
      if (l.category_id && !hit.category_id) patch('txn', hit.id, { category_id: l.category_id });
      n += 1;
    }
  }
  return n;
}

function findInstallment(id: string): InstRec {
  for (const l of st().loans) {
    const i = l.installments.find((x) => x.id === id);
    if (i) return i;
  }
  return notFound(`parcela ${id}`);
}

function findWithLoan(id: string): { l: LoanRec; i: InstRec } {
  for (const l of st().loans) {
    const i = l.installments.find((x) => x.id === id);
    if (i) return { l, i };
  }
  return notFound('parcela');
}

/** Espelho de `debt::link::payment_candidates`: saídas até 20 dias do vencimento, sem
 *  parcela (fora cartão e corretora), por valor mais próximo e depois por data. */
function paymentCandidates(installmentId: string): LoanPaymentCandidate[] {
  const { l, i } = findWithLoan(installmentId);
  const used = new Set(
    st().loans.flatMap((x) => [
      ...x.installments.map((y) => y.txn_id),
      ...x.extras.map((e) => e.txn_id),
    ]),
  );
  const from = addDays(i.due_date, -20);
  const to = addDays(i.due_date, 20);
  const gap = (t: { amount: number }) => Math.abs(-t.amount - i.total);
  const days = (t: { date: string }) => Math.abs(diffDays(t.date, i.due_date));
  return rows('txn')
    .filter(
      (t) =>
        t.amount < 0 &&
        !t.transfer_id &&
        t.status !== 'planned' &&
        t.currency === 'BRL' &&
        t.kind !== 'investment' &&
        t.kind !== 'card_payment' &&
        find('account', t.account_id)?.kind !== 'credit_card' &&
        t.date >= from &&
        t.date <= to &&
        !used.has(t.id),
    )
    .sort(
      (a, b) =>
        gap(a) - gap(b) ||
        days(a) - days(b) ||
        a.date.localeCompare(b.date) ||
        a.id.localeCompare(b.id),
    )
    .slice(0, 20)
    .map((t) => ({
      txn_id: t.id,
      date: t.date,
      description: t.description,
      account_id: t.account_id,
      account_name: find('account', t.account_id)?.name ?? '?',
      amount: t.amount,
      currency: t.currency,
      in_debit_account: t.account_id === l.debit_account_id,
    }));
}

/** Espelho de `debt::link::link_payment`. */
function linkPayment(installmentId: string, txnId: string): Installment {
  const { l, i } = findWithLoan(installmentId);
  const t = find('txn', txnId) ?? notFound(`lançamento ${txnId}`);
  if (t.amount >= 0) invalid('o pagamento de uma parcela precisa ser uma saída (valor negativo)');
  if (t.transfer_id)
    invalid('o lançamento é uma transferência entre suas contas e não pode pagar uma parcela');
  if (t.status === 'planned')
    invalid('o lançamento ainda está previsto; efetive-o antes de ligar à parcela');
  if (t.currency !== 'BRL') invalid(`o lançamento está em ${t.currency} e o contrato em BRL`);
  for (const x of st().loans) {
    const other = [...x.installments]
      .sort((a, b) => a.number - b.number)
      .find((o) => o.txn_id === t.id && o.id !== i.id);
    if (other)
      conflict(
        `o lançamento já está ligado à parcela ${other.number} de ${x.name}; desligue-o de lá antes`,
      );
  }
  const extra = st().loans.find((x) => x.extras.some((e) => e.txn_id === t.id));
  if (extra) conflict(`o lançamento é de uma amortização extraordinária de ${extra.name}`);
  i.paid = true;
  i.paid_date = t.date;
  i.paid_amount = -t.amount;
  i.txn_id = t.id;
  // Sem categoria: recebe a dos pagamentos do contrato (receita/despesa sem divisão).
  const split = rows('txn_split').some((x) => x.txn_id === t.id);
  if (l.category_id && !t.category_id && (t.kind === 'income' || t.kind === 'expense') && !split) {
    let fits = true;
    try {
      checkCategory(l.category_id, t.kind);
    } catch {
      fits = false;
    }
    if (fits) patch('txn', t.id, { category_id: l.category_id });
  }
  return withVersions(l.installments, today()).find((x) => x.id === i.id)!;
}

setLoanLookup({
  links: () => {
    const out = new Map<string, LoanLink>();
    for (const l of state?.loans ?? [])
      for (const i of [...l.installments].sort((a, b) => a.number - b.number))
        if (i.txn_id && !out.has(i.txn_id))
          out.set(i.txn_id, {
            loan_id: l.id,
            loan_name: l.name,
            installment_id: i.id,
            installment_number: i.number,
          });
    return out;
  },
  forget: (ids) => {
    for (const l of state?.loans ?? []) {
      for (const i of l.installments)
        if (i.txn_id && ids.has(i.txn_id)) {
          i.paid = false;
          i.paid_date = null;
          i.paid_amount = null;
          i.txn_id = null;
        }
      for (const e of l.extras) if (e.txn_id && ids.has(e.txn_id)) e.txn_id = null;
    }
  },
});

function list(includeArchived: boolean): LoanView[] {
  return st()
    .loans.filter((l) => includeArchived || !l.archived)
    .sort((a, b) => a.contract_date.localeCompare(b.contract_date))
    .map(view);
}

registerMockHandlers(
  guarded({
    'debt.list': (a) => list(a.include_archived === true),
    'debt.get': (a) => view(loan(str(a, 'id'))),
    'debt.schedule': (a) => {
      const t = today();
      return withVersions(loan(str(a, 'id')).installments, t);
    },
    'debt.summary': (): DebtSummary => {
      const loans = list(false);
      return {
        total_balance: loans.reduce((s, l) => s + l.balance, 0),
        monthly_commitment: loans.reduce((s, l) => s + (l.next_installment?.total ?? 0), 0),
        interest_remaining: loans.reduce((s, l) => s + l.interest_remaining, 0),
        loans,
      };
    },
    'debt.create': (a) => view(create(a as unknown as LoanInput)),
    'debt.statement_preview': (a) => statementPreview(loan(str(a, 'id')), a),
    'debt.import_statement': (a) => importStatement(loan(str(a, 'id')), a),
    'debt.update': (a) => {
      const l = loan(str(a, 'id'));
      // Todos os termos: recalcula mantendo parcelas pagas e amortizações (como o núcleo).
      if (a.terms) {
        const t = a.terms as LoanInput;
        const fresh = fromInput({ ...t, mark_past_paid: false });
        Object.assign(l, {
          ...fresh,
          id: l.id,
          archived: l.archived,
          extras: l.extras,
          installments: l.installments,
        });
        regenerate(l);
        if (t.mark_past_paid) markPaidUntil(l, today());
      }
      if (typeof a.name === 'string') {
        if (!a.name.trim()) invalid('dê um nome ao contrato');
        l.name = a.name.trim();
      }
      if ('notes' in a) l.notes = (a.notes as string | null) ?? null;
      if ('contract_number' in a)
        l.contract_number = cleanNumber(a.contract_number as string | null);
      if ('debit_account_id' in a) {
        const acc = (a.debit_account_id as string | null) ?? null;
        if (acc) currencyAndKind(acc);
        l.debit_account_id = acc;
      }
      if ('category_id' in a) setCategory(l, (a.category_id as string | null) ?? null);
      if (typeof a.archived === 'boolean') l.archived = a.archived;
      let regen = false;
      if (typeof a.annual_rate === 'string') {
        const r = Number(a.annual_rate.trim().replace(',', '.'));
        if (!Number.isFinite(r) || r < 0 || r > 1000) invalid('taxa anual fora do intervalo');
        l.annual_rate = String(r);
        regen = true;
      }
      if (typeof a.insurance_monthly === 'number') {
        l.insurance_monthly = a.insurance_monthly;
        regen = true;
      }
      if (typeof a.fees_monthly === 'number') {
        l.fees_monthly = a.fees_monthly;
        regen = true;
      }
      if (
        'property_value' in a ||
        'mip_rate' in a ||
        'dfi_rate' in a ||
        'mip_amount' in a ||
        'dfi_amount' in a
      ) {
        setHousing(
          l,
          'property_value' in a ? ((a.property_value as number | null) ?? null) : l.property_value,
          'mip_rate' in a ? ((a.mip_rate as string | null) ?? null) : l.mip_rate,
          'dfi_rate' in a ? ((a.dfi_rate as string | null) ?? null) : l.dfi_rate,
          'mip_amount' in a ? ((a.mip_amount as number | null) ?? null) : l.mip_amount,
          'dfi_amount' in a ? ((a.dfi_amount as number | null) ?? null) : l.dfi_amount,
        );
        regen = true;
      }
      if (regen) regenerate(l);
      return view(l);
    },
    'debt.delete': (a) => {
      const s = st();
      const id = str(a, 'id');
      if (!s.loans.some((l) => l.id === id)) notFound('contrato');
      s.loans = s.loans.filter((l) => l.id !== id);
      return null;
    },
    'debt.add_extra_payment': (a) => {
      const l = loan(str(a, 'loan_id'));
      const amount = num(a, 'amount');
      if (amount <= 0) invalid('o valor da amortização deve ser positivo');
      const d = date(a.date);
      const mode = a.mode as PrepayMode;
      if (!MODES.includes(mode))
        invalid(`argumentos inválidos: unknown variant \`${String(mode)}\``);
      const acc =
        a.without_txn === true ? null : (optStr(a, 'from_account_id') ?? l.debit_account_id);
      let txnId: string | null = null;
      if (acc) {
        txnId = write('Amortização extraordinária', () => {
          const { currency } = currencyAndKind(acc);
          return insertTxn({
            account_id: acc,
            date: d,
            amount: -amount,
            currency,
            kind: 'adjustment',
            description: `Amortização extraordinária · ${l.name}`,
          }).id;
        }).value;
      }
      l.extras.push({ id: nextId('extra'), date: d, amount, mode, txn_id: txnId });
      regenerate(l);
      return view(l);
    },
    'debt.simulate_prepayment': simulatePrepayment,
    'debt.simulate_anticipation': (a) => simulateAnticipation(a).sim,
    'debt.anticipate': (a) => {
      const { l, sim } = simulateAnticipation(a);
      for (const x of sim.installments) {
        const inst = l.installments.find((i) => i.number === x.number)!;
        inst.paid = true;
        inst.paid_date = date(a.date);
        inst.paid_amount = x.present_value;
      }
      return sim;
    },
    'debt.mark_paid_until': (a) => ({
      marked: markPaidUntil(loan(str(a, 'loan_id')), date(a.before)),
    }),
    'debt.simulate_portability': simulatePortability,
    'debt.reconcile': (a) => ({ matched: reconcile(loan(str(a, 'id'))) }),
    'debt.record_payment': (a) => {
      const l = loan(str(a, 'id'));
      const d = date(a.date);
      const n =
        (Number(d.slice(0, 4)) - Number(l.first_due_date.slice(0, 4))) * 12 +
        Number(d.slice(5, 7)) -
        Number(l.first_due_date.slice(5, 7)) +
        1;
      const i =
        typeof a.installment_id === 'string'
          ? findInstallment(a.installment_id)
          : (l.installments.find((x) => x.number === n) ??
            invalid('parcela do mês do pagamento não encontrada'));
      const opt = (k: string) => (typeof a[k] === 'number' ? a[k] : null);
      for (const k of ['amortization', 'interest', 'insurance', 'fees', 'adjustment'] as const) {
        const v = opt(k);
        if (v != null) i[k] = v;
      }
      if (opt('amortization') != null || opt('interest') != null)
        i.total = i.amortization + i.interest + i.insurance + i.fees;
      i.paid = true;
      i.paid_date = d;
      i.paid_amount = opt('amount') ?? i.total;
      i.txn_id = optStr(a, 'txn_id') ?? i.txn_id;
      const bal = opt('balance_after');
      if (bal != null) {
        const prev = l.installments.find((x) => x.number === i.number - 1);
        const extra =
          prev && opt('amortization') != null
            ? Math.max(0, prev.balance_after + i.adjustment - i.amortization - bal)
            : 0;
        i.balance_after = bal;
        i.estimated = false;
        i.statement = true;
        if (extra > 5000) i.extra_payment = extra;
        if (!l.rebase || l.rebase.number <= i.number) {
          const term =
            opt('remaining_term') ?? l.installments.filter((x) => x.number > i.number).length;
          if (term === 0 && bal > 0)
            invalid('informe quantas parcelas faltam depois deste pagamento');
          l.rebase = { number: i.number, balance: bal, term };
          regenerate(l);
        }
      }
      return withVersions(l.installments, today()).find((x) => x.number === i.number)!;
    },
    'debt.mark_paid': (a) => {
      const i = findInstallment(str(a, 'installment_id'));
      i.paid = true;
      i.paid_date = date(a.date);
      i.paid_amount = typeof a.amount === 'number' ? a.amount : i.total;
      i.txn_id = optStr(a, 'txn_id') ?? null;
      return null;
    },
    'debt.payment_candidates': (a) => paymentCandidates(str(a, 'installment_id')),
    'debt.link_payment': (a) => linkPayment(str(a, 'installment_id'), str(a, 'txn_id')),
    'debt.unmark_paid': (a) => {
      const i = findInstallment(str(a, 'id'));
      i.paid = false;
      i.paid_date = null;
      i.paid_amount = null;
      i.txn_id = null;
      return null;
    },
  }),
);
