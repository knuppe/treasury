/**
 * Mock de `physical.*`: o patrimônio físico da Família Exemplo — o apartamento da Vila
 * Mariana (financiado pelo `loan-1`, corrigido pelo IGP-M a partir da última avaliação,
 * com reforma, ampliação e manutenção), o Onix (ligado ao CDC `loan-2`, com consultas à
 * tabela FIPE e depreciação de 12% a.a. entre elas) e um terreno em Atibaia corrigido
 * pelo IPCA. O valor sai de `./physical/engine.ts` (o mesmo cálculo do núcleo); saldos
 * devedores vêm do mock de dívidas. Escritas entram no histórico com desfazer.
 */
import type { AcquisitionCostInput } from '@/api/generated/AcquisitionCostInput';
import type { FipeOption } from '@/api/generated/FipeOption';
import type { FipeVehicleType } from '@/api/generated/FipeVehicleType';
import type { ImprovementCategory } from '@/api/generated/ImprovementCategory';
import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import type { PhysicalAcquisitionCost } from '@/api/generated/PhysicalAcquisitionCost';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalFigures } from '@/api/generated/PhysicalFigures';
import type { PhysicalHistoryPoint } from '@/api/generated/PhysicalHistoryPoint';
import type { PhysicalImprovement } from '@/api/generated/PhysicalImprovement';
import type { PhysicalIndex } from '@/api/generated/PhysicalIndex';
import type { PhysicalKind } from '@/api/generated/PhysicalKind';
import type { PhysicalLinkedLoan } from '@/api/generated/PhysicalLinkedLoan';
import type { PhysicalSummary } from '@/api/generated/PhysicalSummary';
import type { PhysicalValuation } from '@/api/generated/PhysicalValuation';
import type { PhysicalValuationMethod } from '@/api/generated/PhysicalValuationMethod';
import type { PhysicalValuationSource } from '@/api/generated/PhysicalValuationSource';

import { extraMockHandler, onMockReset, registerMockHandlers } from '../mockRegistry';
import { xirr } from './debt/schedule';
import { guarded } from './ledger/boot';
import { addMonths, diffDays, isIsoDate, monthEnd, today } from './ledger/dates';
import { conflict, invalid, notFound, str } from './ledger/errors';
import {
  createProject,
  projectOutflows,
  projectRow,
  projectSpent,
  setPhysicalLookup,
} from './ledger/projectsState';
import { find, recordExternal, rows, USER, write } from './ledger/store';
import { deflatorFactor } from './marketSim';
import { costBasis, valueAt, type Estimate, type Model, type Rule } from './physical/engine';

const KINDS: PhysicalKind[] = [
  'real_estate',
  'vehicle',
  'land',
  'equipment',
  'collectible',
  'other',
];
const METHODS: PhysicalValuationMethod[] = ['none', 'index', 'declining', 'straight_line', 'fipe'];
const CATEGORIES: ImprovementCategory[] = ['renovation', 'expansion', 'improvement', 'maintenance'];
const SOURCES: PhysicalValuationSource[] = ['manual', 'appraisal', 'report', 'fipe', 'index'];
const SOURCE_LABEL: Record<PhysicalValuationSource, string> = {
  manual: 'estimativa manual',
  appraisal: 'avaliação',
  report: 'laudo',
  fipe: 'FIPE',
  index: 'índice',
};
const INDEX_LABEL: Record<PhysicalIndex, string> = { ipca: 'IPCA', igpm: 'IGP-M', incc: 'INCC' };
const DEFAULT_RATE = '12';
const DEFAULT_LIFE = 5;

interface AssetRec {
  id: string;
  kind: PhysicalKind;
  name: string;
  member_ids: string[];
  purchase_date: string;
  purchase_price: number;
  currency: string;
  acquisition_costs: PhysicalAcquisitionCost[];
  valuation_method: PhysicalValuationMethod;
  index_series: PhysicalIndex | null;
  depreciation_rate: string | null;
  useful_life_years: number | null;
  residual_pct: string | null;
  fipe_vehicle_type: FipeVehicleType | null;
  fipe_brand_code: string | null;
  fipe_model_code: string | null;
  fipe_year_code: string | null;
  fipe_code: string | null;
  address: string | null;
  description: string | null;
  notes: string | null;
  include_in_net_worth: boolean;
  archived: boolean;
  sale_date: string | null;
  sale_price: number | null;
  sale_costs: number;
  improvements: PhysicalImprovement[];
  valuations: PhysicalValuation[];
  loan_ids: string[];
}

interface State {
  today: string;
  assets: AssetRec[];
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

// ── Seed ─────────────────────────────────────────────────────────

const base = (
  partial: Partial<AssetRec> &
    Pick<AssetRec, 'id' | 'kind' | 'name' | 'purchase_date' | 'purchase_price'>,
): AssetRec => ({
  member_ids: [],
  currency: 'BRL',
  acquisition_costs: [],
  valuation_method: 'none',
  index_series: null,
  depreciation_rate: null,
  useful_life_years: null,
  residual_pct: null,
  fipe_vehicle_type: null,
  fipe_brand_code: null,
  fipe_model_code: null,
  fipe_year_code: null,
  fipe_code: null,
  address: null,
  description: null,
  notes: null,
  include_in_net_worth: true,
  archived: false,
  sale_date: null,
  sale_price: null,
  sale_costs: 0,
  improvements: [],
  valuations: [],
  loan_ids: [],
  ...partial,
});

function cost(
  id: string,
  kind: PhysicalAcquisitionCost['kind'],
  amount: number,
  description: string | null = null,
): PhysicalAcquisitionCost {
  return { id, kind, amount, description };
}

function imp(
  asset: string,
  id: string,
  date: string,
  description: string,
  category: ImprovementCategory,
  amount: number,
): PhysicalImprovement {
  return {
    id,
    asset_id: asset,
    date,
    description,
    category,
    amount,
    txn_id: null,
    notes: null,
    capital: category !== 'maintenance',
    project_id: null,
    project_name: null,
    entered_amount: amount,
    linked_count: 0,
  };
}

/** Benfeitoria com o projeto: com lançamentos ligados, vale o gasto dele (cada um na data). */
function resolved(i: PhysicalImprovement): PhysicalImprovement & {
  flows: { date: string; amount: number }[];
} {
  const p = i.project_id ? find('project', i.project_id) : undefined;
  const flows = p ? projectOutflows(p.id) : [];
  return {
    ...i,
    project_id: p ? p.id : null,
    project_name: p?.name ?? null,
    entered_amount: i.entered_amount,
    amount: flows.length ? flows.reduce((s, f) => s + f.amount, 0) : i.entered_amount,
    linked_count: flows.length,
    flows: flows.length ? flows : [{ date: i.date, amount: i.entered_amount }],
  };
}

/** Valor guardado: o informado ou, ligada a projeto e sem valor, o gasto (ou orçamento). */
function storedAmount(amount: number, projectId: string | null): number {
  if (!projectId || amount > 0) return amount;
  const spent = projectSpent(projectId);
  const v = spent > 0 ? spent : (find('project', projectId)?.budget ?? 0);
  if (v <= 0) invalid('informe o valor (estimado) da obra ou ligue lançamentos ao projeto');
  return v;
}

function improvementProject(
  a: Record<string, unknown>,
  assetId: string,
  description: string,
  date: string,
  current: string | null,
): string | null {
  if (a.create_project === true)
    return write(`Projeto criado: ${description}`, () =>
      createProject({ name: description, physical_asset_id: assetId, start_date: date }),
    ).value;
  const id = 'project_id' in a ? clean(a.project_id) : current;
  if (id) projectRow(id);
  return id;
}

function val(
  asset: string,
  id: string,
  date: string,
  value: number,
  source: PhysicalValuationSource,
  notes: string | null = null,
): PhysicalValuation {
  return { id, asset_id: asset, date, value, source, notes };
}

function seed(t: string): State {
  const fipeDates = [addMonths(t, -14), addMonths(t, -2)].map((d) => `${d.slice(0, 7)}-03`);
  return {
    today: t,
    seq: 100,
    assets: [
      base({
        id: 'phys-apto',
        kind: 'real_estate',
        name: 'Apartamento Vila Mariana',
        member_ids: ['mem-ana', 'mem-bruno'],
        purchase_date: '2023-03-15',
        purchase_price: 60_000_000,
        acquisition_costs: [
          cost('pc-1', 'itbi', 1_800_000),
          cost('pc-2', 'deed', 450_000),
          cost('pc-3', 'registry', 320_000),
        ],
        valuation_method: 'index',
        index_series: 'igpm',
        address: 'Rua Domingos de Morais, 1200 · ap. 82 · São Paulo/SP',
        description: '78 m², 2 dormitórios, 1 vaga.',
        improvements: [
          imp('phys-apto', 'pi-1', '2024-02-10', 'Reforma da cozinha', 'renovation', 3_800_000),
          imp('phys-apto', 'pi-2', '2025-07-05', 'Fechamento da varanda', 'expansion', 1_200_000),
          imp(
            'phys-apto',
            'pi-3',
            '2025-09-20',
            'Pintura e revisão elétrica',
            'maintenance',
            650_000,
          ),
        ],
        valuations: [
          val(
            'phys-apto',
            'pv-1',
            '2023-02-20',
            61_000_000,
            'report',
            'Laudo da Caixa para o financiamento',
          ),
          val(
            'phys-apto',
            'pv-2',
            '2025-11-10',
            72_000_000,
            'appraisal',
            'Imobiliária, 3 comparáveis no prédio',
          ),
        ],
        loan_ids: ['loan-1'],
      }),
      base({
        id: 'phys-onix',
        kind: 'vehicle',
        name: 'Onix LTZ 2022',
        member_ids: ['mem-bruno'],
        purchase_date: '2024-08-02',
        purchase_price: 9_200_000,
        acquisition_costs: [cost('pc-4', 'other', 120_000, 'Transferência e emplacamento')],
        valuation_method: 'fipe',
        depreciation_rate: DEFAULT_RATE,
        fipe_vehicle_type: 'carros',
        fipe_brand_code: '23',
        fipe_model_code: '9213',
        fipe_year_code: '2022-1',
        fipe_code: '004478-4',
        improvements: [
          imp('phys-onix', 'pi-4', '2025-03-12', 'Revisão dos 30 mil km', 'maintenance', 98_000),
        ],
        valuations: [
          val(
            'phys-onix',
            'pv-3',
            fipeDates[0]!,
            8_190_000,
            'fipe',
            'FIPE · 004478-4 · Onix LTZ 1.0 Turbo 2022',
          ),
          val(
            'phys-onix',
            'pv-4',
            fipeDates[1]!,
            7_480_000,
            'fipe',
            'FIPE · 004478-4 · Onix LTZ 1.0 Turbo 2022',
          ),
        ],
        loan_ids: ['loan-2'],
      }),
      base({
        id: 'phys-terreno',
        kind: 'land',
        name: 'Terreno em Atibaia',
        member_ids: ['mem-ana'],
        purchase_date: '2023-05-10',
        purchase_price: 18_000_000,
        acquisition_costs: [cost('pc-5', 'itbi', 360_000), cost('pc-6', 'registry', 180_000)],
        valuation_method: 'index',
        index_series: 'ipca',
        address: 'Estrada do Rosário, lote 14 · Atibaia/SP',
        improvements: [
          imp('phys-terreno', 'pi-5', '2024-06-15', 'Muro e portão', 'improvement', 2_200_000),
        ],
      }),
    ],
  };
}

setPhysicalLookup({
  name: (id) => state?.assets.find((a) => a.id === id)?.name ?? null,
  exists: (id) => st().assets.some((a) => a.id === id),
  improvementIds: (projectId) =>
    (state?.assets ?? []).flatMap((a) =>
      a.improvements.filter((i) => i.project_id === projectId).map((i) => i.id),
    ),
  unlink: (projectId) => {
    for (const a of state?.assets ?? [])
      a.improvements = a.improvements.map((i) =>
        i.project_id === projectId ? { ...i, project_id: null, amount: i.entered_amount } : i,
      );
  },
});

function st(): State {
  const t = today();
  if (!state || state.today !== t) state = seed(t);
  return state;
}

function asset(id: string): AssetRec {
  return st().assets.find((a) => a.id === id) ?? notFound(`bem ${id}`);
}

/** Escrita com histórico: desfazer/refazer restauram o antes/depois do mock. */
function mutate<T>(
  reason: string,
  summary: Record<string, Record<string, number>>,
  fn: () => T,
): T {
  const s = st();
  const before = structuredClone(s.assets);
  const value = fn();
  const after = structuredClone(s.assets);
  recordExternal(reason, USER, {
    summary,
    undo: () => {
      st().assets = structuredClone(before);
    },
    redo: () => {
      st().assets = structuredClone(after);
    },
  });
  return value;
}

// ── Cálculo ──────────────────────────────────────────────────────

function model(a: AssetRec): Model {
  const rule: Rule =
    a.valuation_method === 'index'
      ? {
          kind: 'index',
          name: INDEX_LABEL[a.index_series ?? 'ipca'],
          series: a.index_series ?? 'ipca',
        }
      : a.valuation_method === 'declining' || a.valuation_method === 'fipe'
        ? {
            kind: 'declining',
            ratePct: Number(a.depreciation_rate ?? DEFAULT_RATE),
            fipe: a.valuation_method === 'fipe',
          }
        : a.valuation_method === 'straight_line'
          ? {
              kind: 'straight_line',
              lifeYears: a.useful_life_years ?? DEFAULT_LIFE,
              residualPct: Number(a.residual_pct ?? '0'),
            }
          : { kind: 'flat' };
  return {
    currency: a.currency,
    purchaseDate: a.purchase_date,
    initialCost: a.purchase_price + a.acquisition_costs.reduce((s, c) => s + c.amount, 0),
    capital: a.improvements
      .filter((i) => i.capital)
      .flatMap((i) => resolved(i).flows)
      .sort((x, y) => x.date.localeCompare(y.date)),
    appraisals: [...a.valuations]
      .sort((x, y) => x.date.localeCompare(y.date))
      .map((v) => ({ date: v.date, value: v.value, source: SOURCE_LABEL[v.source] })),
    rule,
    sale:
      a.sale_date && a.sale_price != null
        ? { date: a.sale_date, net: a.sale_price - a.sale_costs }
        : null,
  };
}

function loanView(id: string): LoanView | null {
  try {
    return extraMockHandler('debt.get')?.({ id }) as LoanView;
  } catch {
    return null;
  }
}

/** Saldo devedor de um contrato do mock numa data (pelo cronograma). */
function loanBalanceAt(id: string, at: string): number {
  const l = loanView(id);
  if (!l || l.contract_date > at) return 0;
  const sched = (extraMockHandler('debt.schedule')?.({ id }) ?? []) as Installment[];
  const last = [...sched].reverse().find((i) => i.due_date <= at);
  return last ? last.balance_after : l.principal;
}

/** Soma dos saldos devedores dos contratos ativos numa data (passivo do patrimônio). */
export function mockDebtsAt(at: string): number {
  try {
    const loans = (extraMockHandler('debt.list')?.({}) ?? []) as LoanView[];
    return loans.reduce((s, l) => s + loanBalanceAt(l.id, at), 0);
  } catch {
    return 0;
  }
}

const ratio = (a: number, b: number) => (b !== 0 ? a / b : null);

function figures(a: AssetRec, at: string, deflator: string): PhysicalFigures {
  const m = model(a);
  const e: Estimate = valueAt(m, at);
  const sold = e.basis === 'sold';
  const end = sold && m.sale ? m.sale.date : at;
  const cb = costBasis(m, end);
  const maintenance = a.improvements
    .filter((i) => !i.capital)
    .flatMap((i) => resolved(i).flows)
    .filter((f) => f.date <= at)
    .reduce((s, f) => s + f.amount, 0);
  const saleNet = sold && m.sale ? m.sale.net : null;
  const finalValue = saleNet ?? e.value;
  const contributions = [
    { date: m.purchaseDate, amount: m.initialCost },
    ...m.capital.filter((c) => c.date <= end),
  ];
  let realCost: number | null = 0;
  for (const c of contributions) {
    const f = deflatorFactor(deflator, c.date > m.purchaseDate ? c.date : m.purchaseDate, end);
    realCost = realCost != null && f ? realCost + Math.round(c.amount * f.factor) : null;
  }
  const realGain = realCost != null ? finalValue - realCost : null;
  const holding = Math.max(diffDays(end, m.purchaseDate), 0);
  const annualized =
    holding >= 30 && finalValue > 0
      ? xirr([
          ...contributions
            .filter((c) => c.amount > 0)
            .map((c) => ({ date: c.date, amount: -c.amount })),
          { date: end, amount: finalValue },
        ])
      : null;
  let realAnnualized: number | null = null;
  if (annualized != null && holding > 0) {
    const f = deflatorFactor(deflator, m.purchaseDate, end);
    if (f) realAnnualized = (1 + annualized) / f.factor ** (365 / holding) - 1;
  }
  const loanBalance = sold ? 0 : a.loan_ids.reduce((s, id) => s + loanBalanceAt(id, at), 0);
  return {
    as_of: at,
    initial_cost: m.initialCost,
    capital_improvements: cb - m.initialCost,
    maintenance,
    cost_basis: cb,
    value: e.value,
    value_basis: e.basis,
    estimated: e.estimated,
    formula: e.formula,
    source: e.source,
    anchor_date: e.anchorDate,
    anchor_value: e.anchorValue,
    nominal_gain: finalValue - cb,
    nominal_gain_pct: ratio(finalValue - cb, cb),
    deflator: deflator as PhysicalFigures['deflator'],
    real_cost_basis: realCost,
    real_gain: realGain,
    real_gain_pct: realCost != null && realGain != null ? ratio(realGain, realCost) : null,
    annualized,
    real_annualized: realAnnualized,
    loan_balance: loanBalance,
    equity: e.value - loanBalance,
    ltv: e.value > 0 ? ratio(loanBalance, e.value) : null,
    sold,
    sale_net: saleNet,
    holding_days: holding,
  };
}

function linkedLoans(a: AssetRec, at: string): PhysicalLinkedLoan[] {
  return a.loan_ids.flatMap((id) => {
    const l = loanView(id);
    return l
      ? [
          {
            loan_id: id,
            name: l.name,
            kind: l.kind,
            principal: l.principal,
            balance: loanBalanceAt(id, at),
            currency: l.currency,
            archived: l.archived,
          },
        ]
      : [];
  });
}

function view(a: AssetRec, at = today(), deflator = 'ipca'): PhysicalAssetView {
  const { loan_ids, ...rest } = a;
  void loan_ids;
  return {
    ...structuredClone(rest),
    improvements: [...a.improvements]
      .sort((x, y) => x.date.localeCompare(y.date))
      .map((i) => {
        const { flows, ...r } = resolved(i);
        void flows;
        return r;
      }),
    valuations: [...a.valuations].sort((x, y) => x.date.localeCompare(y.date)),
    loans: linkedLoans(a, at),
    figures: figures(a, at, deflator),
  };
}

function readArgs(args: Record<string, unknown>): { at: string; deflator: string } {
  const at = typeof args.as_of === 'string' ? date(args.as_of) : today();
  const deflator = typeof args.deflator === 'string' ? args.deflator : 'ipca';
  return { at, deflator };
}

function list(args: Record<string, unknown>): PhysicalAssetView[] {
  const { at, deflator } = readArgs(args);
  return st()
    .assets.filter((a) => args.include_archived === true || !a.archived)
    .sort((x, y) => x.purchase_date.localeCompare(y.purchase_date) || x.name.localeCompare(y.name))
    .map((a) => view(a, at, deflator));
}

/** Bens que entram no patrimônio numa data (usado pelo patrimônio do mock). */
export function physicalNetWorthItems(
  at: string,
): { id: string; name: string; value: number; estimated: boolean; formula: string }[] {
  return st()
    .assets.filter(
      (a) =>
        a.include_in_net_worth &&
        a.purchase_date <= at &&
        !(a.archived && !a.sale_date) &&
        !(a.sale_date && a.sale_date <= at),
    )
    .map((a) => {
      const e = valueAt(model(a), at);
      return { id: a.id, name: a.name, value: e.value, estimated: e.estimated, formula: e.formula };
    })
    .filter((i) => i.value !== 0);
}

// ── Validação (mensagens do núcleo) ──────────────────────────────

function date(v: unknown): string {
  if (typeof v !== 'string' || !isIsoDate(v)) invalid(`data inválida: ${String(v)}`);
  return v;
}

function pctText(v: unknown, what: string): string | null {
  if (v == null || (typeof v === 'string' && !v.trim())) return null;
  const n =
    typeof v === 'number' ? v : Number(typeof v === 'string' ? v.trim().replace(',', '.') : NaN);
  if (!Number.isFinite(n))
    invalid(`${what} inválido: ${typeof v === 'string' ? v : JSON.stringify(v)}`);
  if (n < 0 || n > 100) invalid(`${what} fora do intervalo`);
  return String(n);
}

const clean = (v: unknown): string | null => (typeof v === 'string' && v.trim() ? v.trim() : null);

function validate(a: AssetRec): void {
  a.name = a.name.trim();
  if (!a.name) invalid('dê um nome ao bem');
  if (!KINDS.includes(a.kind)) invalid(`argumentos inválidos: unknown variant \`${a.kind}\``);
  if (!METHODS.includes(a.valuation_method))
    invalid(`argumentos inválidos: unknown variant \`${a.valuation_method}\``);
  date(a.purchase_date);
  if (!(a.purchase_price >= 0)) invalid('o preço de compra não pode ser negativo');
  a.depreciation_rate = pctText(a.depreciation_rate, 'taxa de depreciação');
  a.residual_pct = pctText(a.residual_pct, 'valor residual');
  if (a.useful_life_years != null && !(a.useful_life_years >= 1 && a.useful_life_years <= 100))
    invalid('vida útil entre 1 e 100 anos');
  if (a.valuation_method === 'index' && !a.index_series) a.index_series = 'ipca';
  if (a.valuation_method === 'declining' && !a.depreciation_rate)
    a.depreciation_rate = DEFAULT_RATE;
  if (a.valuation_method === 'straight_line' && a.useful_life_years == null)
    a.useful_life_years = DEFAULT_LIFE;
  if (a.valuation_method === 'fipe') {
    if (!a.fipe_vehicle_type || !a.fipe_brand_code || !a.fipe_model_code || !a.fipe_year_code)
      invalid('para usar a tabela FIPE informe tipo, marca, modelo e ano do veículo');
    a.depreciation_rate ??= DEFAULT_RATE;
  }
  if (a.sale_date && a.sale_date < a.purchase_date) invalid('a venda não pode ser antes da compra');
}

function costs(v: unknown): PhysicalAcquisitionCost[] {
  const list = (Array.isArray(v) ? v : []) as AcquisitionCostInput[];
  if (list.some((c) => c.amount < 0)) invalid('custos de aquisição não podem ser negativos');
  return list
    .filter((c) => c.amount > 0)
    .map((c) => ({
      id: nextId('pc'),
      kind: c.kind,
      amount: c.amount,
      description: clean(c.description),
    }));
}

function members(v: unknown): string[] {
  const ids = [...new Set((Array.isArray(v) ? v : []) as string[])];
  for (const id of ids) if (!rows('member').some((m) => m.id === id)) notFound(`membro ${id}`);
  return ids;
}

function linkLoan(a: AssetRec, loanId: string): void {
  if (!loanView(loanId)) notFound(`contrato ${loanId}`);
  const other = st().assets.find((x) => x.loan_ids.includes(loanId));
  if (other && other.id !== a.id)
    conflict(`este contrato já financia outro bem (${other.name}); desligue-o de lá primeiro`);
  if (!a.loan_ids.includes(loanId)) a.loan_ids.push(loanId);
}

const FIELDS = [
  'name',
  'kind',
  'purchase_date',
  'purchase_price',
  'valuation_method',
  'index_series',
  'depreciation_rate',
  'useful_life_years',
  'residual_pct',
  'fipe_vehicle_type',
  'fipe_brand_code',
  'fipe_model_code',
  'fipe_year_code',
  'fipe_code',
  'address',
  'description',
  'notes',
  'include_in_net_worth',
  'archived',
] as const;

const NULLABLE = new Set<string>([
  'depreciation_rate',
  'residual_pct',
  'fipe_brand_code',
  'fipe_model_code',
  'fipe_year_code',
  'fipe_code',
  'address',
  'description',
  'notes',
]);

function defaultMethod(kind: PhysicalKind, fipe: boolean): PhysicalValuationMethod {
  if (kind === 'real_estate' || kind === 'land') return 'index';
  if (kind === 'vehicle') return fipe ? 'fipe' : 'declining';
  if (kind === 'equipment') return 'straight_line';
  return 'none';
}

function create(args: Record<string, unknown>): AssetRec {
  const kind = args.kind as PhysicalKind;
  const fipe = Boolean(
    args.fipe_vehicle_type && args.fipe_brand_code && args.fipe_model_code && args.fipe_year_code,
  );
  const a = base({
    id: nextId('phys'),
    kind,
    name: typeof args.name === 'string' ? args.name : '',
    purchase_date: date(args.purchase_date),
    purchase_price: typeof args.purchase_price === 'number' ? args.purchase_price : NaN,
    currency: typeof args.currency === 'string' ? args.currency : 'BRL',
    valuation_method:
      (args.valuation_method as PhysicalValuationMethod | undefined) ?? defaultMethod(kind, fipe),
  });
  for (const k of FIELDS) {
    if (k === 'name' || k === 'kind' || k === 'purchase_date' || k === 'purchase_price') continue;
    if (k === 'valuation_method' || k === 'archived') continue;
    if (k in args && args[k] !== undefined) (a as unknown as Record<string, unknown>)[k] = args[k];
  }
  validate(a);
  a.member_ids = members(args.member_ids);
  a.acquisition_costs = costs(args.acquisition_costs);
  for (const l of (args.loan_ids as string[] | undefined) ?? []) linkLoan(a, l);
  return a;
}

function improvementOf(id: string): { a: AssetRec; i: PhysicalImprovement } {
  for (const a of st().assets) {
    const i = a.improvements.find((x) => x.id === id);
    if (i) return { a, i };
  }
  return notFound(`benfeitoria ${id}`);
}

function checkImprovement(
  i: Pick<PhysicalImprovement, 'description' | 'amount' | 'date' | 'category'>,
) {
  if (!i.description.trim()) invalid('descreva a benfeitoria ou a obra');
  if (!(i.amount > 0)) invalid('o valor da obra deve ser positivo');
  if (!CATEGORIES.includes(i.category))
    invalid(`argumentos inválidos: unknown variant \`${i.category}\``);
  date(i.date);
}

// ── Tabela FIPE (respostas fixas, sem rede) ─────────────────────

const FIPE_BRANDS: FipeOption[] = [
  { code: '21', name: 'Fiat' },
  { code: '22', name: 'Ford' },
  { code: '23', name: 'GM - Chevrolet' },
  { code: '56', name: 'Toyota' },
  { code: '59', name: 'VW - VolksWagen' },
];
const FIPE_MODELS: FipeOption[] = [
  { code: '9213', name: 'ONIX HATCH LTZ 1.0 12V TB Flex Aut.' },
  { code: '9214', name: 'ONIX HATCH PREM. 1.0 12V TB Flex Aut.' },
  { code: '8655', name: 'TRACKER LT 1.0 Turbo 12V Flex Aut.' },
];
const FIPE_YEARS: FipeOption[] = [
  { code: '2024-1', name: '2024 Gasolina' },
  { code: '2023-1', name: '2023 Gasolina' },
  { code: '2022-1', name: '2022 Gasolina' },
];

function history(id: string, months = 240): PhysicalHistoryPoint[] {
  const a = asset(id);
  const m = model(a);
  const t = today();
  const end = m.sale && m.sale.date <= t ? m.sale.date : t;
  let start = addMonths(end, -(Math.min(Math.max(months, 1), 240) - 1));
  if (start < m.purchaseDate) start = m.purchaseDate;
  const out: PhysicalHistoryPoint[] = [];
  for (let month = start.slice(0, 7); ; month = addMonths(`${month}-01`, 1).slice(0, 7)) {
    const last = monthEnd(`${month}-01`);
    const at = last < end ? last : end;
    const e = valueAt(m, at);
    const loan = a.loan_ids.reduce((s, l) => s + loanBalanceAt(l, at), 0);
    const appraisal =
      [...a.valuations]
        .sort((x, y) => x.date.localeCompare(y.date))
        .reverse()
        .find((v) => v.date.startsWith(month) && v.date <= at)?.value ?? null;
    out.push({
      month,
      as_of: at,
      cost_basis: costBasis(m, at),
      value: e.value,
      estimated: e.estimated,
      appraisal,
      loan_balance: loan,
      equity: e.value - loan,
    });
    if (at >= end) break;
  }
  return out;
}

registerMockHandlers(
  guarded({
    'physical.list': (a) => list(a),
    'physical.get': (a) => {
      const { at, deflator } = readArgs(a);
      return view(asset(str(a, 'id')), at, deflator);
    },
    'physical.summary': (a): PhysicalSummary => {
      const assets = list(a);
      const live = assets.filter((v) => !v.figures.sold);
      const sum = (f: (v: PhysicalAssetView) => number) => live.reduce((s, v) => s + f(v), 0);
      const totalValue = sum((v) => v.figures.value);
      const totalCost = sum((v) => v.figures.cost_basis);
      const totalLoan = sum((v) => v.figures.loan_balance);
      return {
        as_of: readArgs(a).at,
        currency: 'BRL',
        count: assets.length,
        total_value: totalValue,
        total_cost_basis: totalCost,
        total_gain: totalValue - totalCost,
        total_loan_balance: totalLoan,
        total_equity: totalValue - totalLoan,
        net_worth_value: sum((v) =>
          v.include_in_net_worth && !(v.archived && !v.sale_date) ? v.figures.value : 0,
        ),
        estimated: live.some((v) => v.figures.estimated),
        missing_fx: [],
        assets,
      };
    },
    'physical.history': (a) =>
      history(str(a, 'id'), typeof a.months === 'number' ? a.months : undefined),
    'physical.create': (a) =>
      view(
        mutate(
          `Bem cadastrado: ${(typeof a.name === 'string' ? a.name : '').trim()}`,
          { insert: { physical_asset: 1 } },
          () => {
            const rec = create(a);
            st().assets.push(rec);
            return rec;
          },
        ),
      ),
    'physical.update': (a) =>
      view(
        mutate('Bem alterado', { update: { physical_asset: 1 } }, () => {
          const rec = asset(str(a, 'id'));
          const next = structuredClone(rec);
          for (const k of FIELDS)
            if (k in a && a[k] !== undefined)
              (next as unknown as Record<string, unknown>)[k] =
                typeof a[k] === 'string' && NULLABLE.has(k) ? clean(a[k]) : a[k];
          validate(next);
          if ('member_ids' in a) next.member_ids = members(a.member_ids);
          if ('acquisition_costs' in a) next.acquisition_costs = costs(a.acquisition_costs);
          Object.assign(rec, next);
          return rec;
        }),
      ),
    'physical.delete': (a) => {
      const id = str(a, 'id');
      asset(id);
      mutate('Bem removido', { delete: { physical_asset: 1 } }, () => {
        st().assets = st().assets.filter((x) => x.id !== id);
      });
      return null;
    },
    'physical.add_improvement': (a) => {
      const rec = asset(str(a, 'asset_id'));
      const description = (typeof a.description === 'string' ? a.description : '').trim();
      const day = typeof a.date === 'string' ? a.date : '';
      const projectId = improvementProject(a, rec.id, description, day, null);
      const amount = storedAmount(typeof a.amount === 'number' ? a.amount : NaN, projectId);
      const i: PhysicalImprovement = {
        id: nextId('pi'),
        asset_id: rec.id,
        date: day,
        description,
        category: a.category as ImprovementCategory,
        amount,
        txn_id: clean(a.txn_id),
        notes: clean(a.notes),
        capital: a.category !== 'maintenance',
        project_id: projectId,
        project_name: null,
        entered_amount: amount,
        linked_count: 0,
      };
      checkImprovement(i);
      mutate(
        `Benfeitoria registrada: ${i.description}`,
        { insert: { physical_improvement: 1 } },
        () => rec.improvements.push(i),
      );
      return view(rec);
    },
    'physical.update_improvement': (a) => {
      const { a: rec, i } = improvementOf(str(a, 'id'));
      const next: PhysicalImprovement = {
        ...i,
        ...(typeof a.date === 'string' ? { date: a.date } : {}),
        ...(typeof a.description === 'string' ? { description: a.description.trim() } : {}),
        ...(typeof a.category === 'string' ? { category: a.category as ImprovementCategory } : {}),
        ...(typeof a.amount === 'number' ? { entered_amount: a.amount } : {}),
        ...('txn_id' in a ? { txn_id: clean(a.txn_id) } : {}),
        ...('notes' in a ? { notes: clean(a.notes) } : {}),
      };
      next.capital = next.category !== 'maintenance';
      next.project_id = improvementProject(a, rec.id, next.description, next.date, i.project_id);
      next.entered_amount = storedAmount(next.entered_amount, next.project_id);
      next.amount = next.entered_amount;
      checkImprovement(next);
      mutate('Benfeitoria alterada', { update: { physical_improvement: 1 } }, () => {
        rec.improvements = rec.improvements.map((x) => (x.id === i.id ? next : x));
      });
      return view(rec);
    },
    'physical.delete_improvement': (a) => {
      const { a: rec, i } = improvementOf(str(a, 'id'));
      mutate('Benfeitoria removida', { delete: { physical_improvement: 1 } }, () => {
        rec.improvements = rec.improvements.filter((x) => x.id !== i.id);
      });
      return view(rec);
    },
    'physical.add_valuation': (a) => {
      const rec = asset(str(a, 'asset_id'));
      const source = a.source as PhysicalValuationSource;
      if (!SOURCES.includes(source))
        invalid(`argumentos inválidos: unknown variant \`${String(source)}\``);
      const value = typeof a.value === 'number' ? a.value : NaN;
      if (!(value >= 0)) invalid('o valor da avaliação não pode ser negativo');
      const v = val(rec.id, nextId('pv'), date(a.date), value, source, clean(a.notes));
      mutate('Avaliação registrada', { insert: { physical_valuation: 1 } }, () =>
        rec.valuations.push(v),
      );
      return view(rec);
    },
    'physical.delete_valuation': (a) => {
      const id = str(a, 'id');
      const rec = st().assets.find((x) => x.valuations.some((v) => v.id === id));
      if (!rec) notFound(`avaliação ${id}`);
      mutate('Avaliação removida', { delete: { physical_valuation: 1 } }, () => {
        rec.valuations = rec.valuations.filter((v) => v.id !== id);
      });
      return view(rec);
    },
    'physical.link_loan': (a) => {
      const rec = asset(str(a, 'asset_id'));
      const loanId = str(a, 'loan_id');
      mutate('Financiamento ligado ao bem', { insert: { physical_asset_loan: 1 } }, () =>
        linkLoan(rec, loanId),
      );
      return view(rec);
    },
    'physical.unlink_loan': (a) => {
      const rec = asset(str(a, 'asset_id'));
      const loanId = str(a, 'loan_id');
      if (!rec.loan_ids.includes(loanId)) notFound('ligação do contrato com o bem');
      mutate('Financiamento desligado do bem', { delete: { physical_asset_loan: 1 } }, () => {
        rec.loan_ids = rec.loan_ids.filter((l) => l !== loanId);
      });
      return view(rec);
    },
    'physical.sell': (a) => {
      const rec = asset(str(a, 'id'));
      const price = typeof a.sale_price === 'number' ? a.sale_price : NaN;
      const costsV = typeof a.sale_costs === 'number' ? a.sale_costs : 0;
      if (!(price >= 0) || costsV < 0) invalid('valores da venda não podem ser negativos');
      const d = date(a.sale_date);
      if (d < rec.purchase_date) invalid('a venda não pode ser antes da compra');
      mutate('Venda do bem registrada', { update: { physical_asset: 1 } }, () => {
        rec.sale_date = d;
        rec.sale_price = price;
        rec.sale_costs = costsV;
        rec.archived = a.archive !== false;
      });
      return view(rec);
    },
    'physical.fipe_options': (a) =>
      a.model_code ? FIPE_YEARS : a.brand_code ? FIPE_MODELS : FIPE_BRANDS,
    'physical.refresh_fipe': (a) => {
      const rec = asset(str(a, 'id'));
      if (
        !rec.fipe_vehicle_type ||
        !rec.fipe_brand_code ||
        !rec.fipe_model_code ||
        !rec.fipe_year_code
      )
        invalid('informe tipo, marca, modelo e ano do veículo na tabela FIPE');
      const t = today();
      const last = [...rec.valuations].sort((x, y) => x.date.localeCompare(y.date)).pop();
      const value = Math.round((last?.value ?? rec.purchase_price) * 0.994);
      mutate('Cotação FIPE', { insert: { physical_valuation: 1 } }, () => {
        rec.valuations = rec.valuations.filter((v) => !(v.source === 'fipe' && v.date === t));
        rec.valuations.push(
          val(
            rec.id,
            nextId('pv'),
            t,
            value,
            'fipe',
            `FIPE · ${rec.fipe_code ?? 'consulta'} · ${rec.name}`,
          ),
        );
      });
      return view(rec);
    },
  }),
);
