/**
 * Mock de `tax_burden.*` (estimativa de impostos pagos) sobre o livro-caixa fictício: os
 * mesmos dados de referência do núcleo (os `data/<país>.json` do crate, lidos como texto,
 * como o `include_str!` do Rust), o mapeamento padrão das categorias (`taxBurden/mapping.ts`)
 * e as regras de IRPF e INSS (`taxBurden/rules.ts`). A estimativa segue
 * `tax_burden::estimate`: consumo por categoria, impostos lançados, renda do trabalho pelos
 * informes ou a partir do líquido, IPVA pelos veículos e IR retido nos proventos.
 */
import type { AmountTriple } from '@/api/generated/AmountTriple';
import type { City } from '@/api/generated/City';
import type { CityRevenue } from '@/api/generated/CityRevenue';
import type { CityState } from '@/api/generated/CityState';
import type { ConsumptionLine } from '@/api/generated/ConsumptionLine';
import type { DirectLine } from '@/api/generated/DirectLine';
import type { IncomeEvent } from '@/api/generated/IncomeEvent';
import type { IncomeReport } from '@/api/generated/IncomeReport';
import type { PayrollLine } from '@/api/generated/PayrollLine';
import type { PayrollRules } from '@/api/generated/PayrollRules';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PropertyLine } from '@/api/generated/PropertyLine';
import type { RateSpec } from '@/api/generated/RateSpec';
import type { RateTriple } from '@/api/generated/RateTriple';
import type { SpendingByFunction } from '@/api/generated/SpendingByFunction';
import type { TaxCategoryView } from '@/api/generated/TaxCategoryView';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxDestination } from '@/api/generated/TaxDestination';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import type { TaxPart } from '@/api/generated/TaxPart';
import type { TaxPartKind } from '@/api/generated/TaxPartKind';
import type { TaxRegion } from '@/api/generated/TaxRegion';
import type { TaxSettings } from '@/api/generated/TaxSettings';
import type { UnestimatedLine } from '@/api/generated/UnestimatedLine';
import { extraMockHandler, onMockReset } from '@/api/mockRegistry';

import brJson from '../../../../../crates/treasury-core/src/tax_burden/data/br.json?raw';
import euJson from '../../../../../crates/treasury-core/src/tax_burden/data/eu.json?raw';
import usJson from '../../../../../crates/treasury-core/src/tax_burden/data/us.json?raw';
import { mockToday } from './ledger/clock';
import { register } from './ledger/common';
import { invalid, notFound } from './ledger/errors';
import { BASE, CATEGORIES, MEMBERS } from './ledger/fixture';
import { getLedger } from './ledger/ledger';
import { spending } from './ledger/reports';
import { fold } from './ledger/text';
import {
  DIRECT,
  kindOf,
  resolve,
  type CategoryChoice,
  type CategoryNode,
} from './taxBurden/mapping';
import { grossFromNet, payroll } from './taxBurden/rules';

/** Meses sem revisão até a tela marcar os dados como desatualizados (`STALE_MONTHS`). */
const STALE_MONTHS = 6;

const DATASETS: readonly TaxDataset[] = [brJson, usJson, euJson].map(
  (text) => JSON.parse(text) as TaxDataset,
);

function dataset(country: string): TaxDataset {
  const d = DATASETS.find((x) => x.country.toLowerCase() === country.toLowerCase());
  if (!d) notFound(`dados de impostos de ${country}`);
  return d;
}

interface State {
  city: City | null;
  revenue: CityRevenue | null;
  settings: TaxSettings;
  choices: Map<string, CategoryChoice>;
  reports: IncomeReport[];
  seq: number;
}

const fresh = (): State => ({
  city: null,
  revenue: null,
  settings: { country: 'BR', region: null, dependents: 0 },
  choices: new Map(),
  reports: [],
  seq: 0,
});

let state = fresh();
onMockReset(() => {
  state = fresh();
});

// ── Categorias ───────────────────────────────────────────────────

function nodes(): Map<string, CategoryNode> {
  return new Map(CATEGORIES.map((c) => [c.id, { name: c.name, parent_id: c.parent_id }]));
}

function categories(): TaxCategoryView[] {
  const n = nodes();
  return CATEGORIES.filter((c) => c.kind === 'expense')
    .map((c): TaxCategoryView => {
      const r = resolve(c.id, n, state.choices);
      return {
        category_id: c.id,
        path: c.path,
        kind: 'expense',
        group_key: r?.group ?? null,
        origin: r?.origin ?? null,
        default_group_key: resolve(c.id, n, new Map())?.group ?? null,
        rate: state.choices.get(c.id)?.rate ?? null,
      };
    })
    .sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

function knownGroup(ds: TaxDataset, key: string): boolean {
  return kindOf(key) !== 'consumption' || ds.consumption.some((g) => g.key === key);
}

/** `undefined` = ausente (não mexe); `null` = limpar. */
function patchField<T>(args: Record<string, unknown>, name: string, ok: (v: unknown) => v is T) {
  if (!(name in args) || args[name] === undefined) return undefined;
  const v = args[name];
  if (v === null) return null;
  if (!ok(v)) invalid(`argumentos inválidos: ${name}`);
  return v;
}

const isString = (v: unknown): v is string => typeof v === 'string';
const isNumber = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v);

function setCategory(args: Record<string, unknown>): void {
  const id = args.category_id;
  if (typeof id !== 'string' || !CATEGORIES.some((c) => c.id === id && c.kind === 'expense'))
    notFound(`categoria de despesa ${String(id)}`);
  const cur = state.choices.get(id);
  let group = cur?.group_key ?? null;
  let rate = cur?.rate ?? null;
  const g = patchField(args, 'group_key', isString);
  if (g !== undefined) {
    group = g && g.trim() ? g : null;
    if (group && !knownGroup(dataset(state.settings.country), group))
      invalid(`grupo de imposto desconhecido: ${group}`);
  }
  const r = patchField(args, 'rate', isNumber);
  if (r !== undefined) {
    if (r != null && !(r >= 0 && r < 100))
      invalid('a alíquota própria fica entre 0% e 100% do preço');
    rate = r;
  }
  if (group == null && rate == null) state.choices.delete(id);
  else state.choices.set(id, { group_key: group, rate });
}

// ── Ajustes ──────────────────────────────────────────────────────

function setSettings(args: Record<string, unknown>): TaxSettings {
  const s = { ...state.settings };
  if (typeof args.country === 'string') {
    s.country = dataset(args.country).country;
    if (!('region' in args)) s.region = null;
  }
  if ('region' in args) {
    const r = args.region;
    s.region = typeof r === 'string' && r.trim() ? r : null;
  }
  if (typeof args.dependents === 'number') {
    if (!Number.isInteger(args.dependents) || args.dependents < 0)
      invalid('argumentos inválidos: dependents');
    if (args.dependents > 20) invalid('dependentes demais');
    s.dependents = args.dependents;
  }
  if (s.region) {
    const ds = dataset(s.country);
    if (!ds.regions.some((x) => x.code.toLowerCase() === s.region!.toLowerCase()))
      invalid(`estado ${s.region} sem dados em ${ds.name}`);
  }
  state.settings = s;
  return { ...s };
}

// ── Informes de rendimentos ──────────────────────────────────────

function reports(year: number | null): IncomeReport[] {
  return state.reports
    .filter((r) => year == null || r.year === year)
    .sort((a, b) => b.year - a.year || a.payer.localeCompare(b.payer));
}

function money(args: Record<string, unknown>, name: string, required = false): number {
  const v = args[name];
  if (v == null) {
    if (required) invalid(`argumentos inválidos: missing field \`${name}\``);
    return 0;
  }
  if (typeof v !== 'number' || !Number.isInteger(v)) invalid(`argumentos inválidos: ${name}`);
  return v;
}

function saveReport(args: Record<string, unknown>): number {
  const payer = typeof args.payer === 'string' ? args.payer.trim() : '';
  if (!payer) invalid('informe a fonte pagadora');
  const year = args.year;
  if (typeof year !== 'number' || year < 2000 || year > 2100) invalid('ano do informe inválido');
  const values = [
    money(args, 'gross', true),
    money(args, 'social_security'),
    money(args, 'income_tax'),
    money(args, 'thirteenth_gross'),
    money(args, 'thirteenth_tax'),
  ] as const;
  if (values.some((v) => v < 0)) invalid('valores do informe não podem ser negativos');
  const member = typeof args.member_id === 'string' && args.member_id ? args.member_id : null;
  if (member && !MEMBERS.some((m) => m.id === member)) notFound(`membro ${member}`);
  const notes = typeof args.notes === 'string' && args.notes.trim() ? args.notes.trim() : null;
  const rec = (id: string): IncomeReport => ({
    id,
    year,
    member_id: member,
    payer,
    gross: values[0],
    social_security: values[1],
    income_tax: values[2],
    thirteenth_gross: values[3],
    thirteenth_tax: values[4],
    notes,
  });
  if (typeof args.id === 'string') {
    const i = state.reports.findIndex((r) => r.id === args.id);
    if (i < 0) notFound(`informe ${args.id}`);
    state.reports[i] = rec(args.id);
  } else {
    state.seq += 1;
    state.reports.push(rec(`tir-${state.seq}`));
  }
  return year;
}

// ── Estimativa ───────────────────────────────────────────────────

const same = (v: number): AmountTriple => ({ conservative: v, probable: v, high: v });
const taxOf = (spent: number, rate: number) => Math.round((spent * rate) / 100);
const ratio = (a: number, b: number) => (b > 0 ? a / b : 0);

function specValue(spec: RateSpec, region: TaxRegion | undefined): number | null {
  const share = (r: number | undefined) => (r == null ? null : (r / (100 + r)) * 100);
  if (typeof spec === 'number') return spec;
  switch (spec) {
    case 'exempt':
      return 0;
    case 'vat_standard':
      return share(region?.vat_standard);
    case 'vat_reduced':
      return share(region?.vat_reduced ?? region?.vat_standard);
    case 'vat_super_reduced':
      return share(region?.vat_super_reduced ?? region?.vat_reduced ?? region?.vat_standard);
    case 'sales_tax':
      return share(region?.sales_tax);
    default:
      return null;
  }
}

function monthsSince(date: string, today: string): number {
  const [y, m] = date.slice(0, 7).split('-').map(Number) as [number, number];
  const [ty, tm] = today.slice(0, 7).split('-').map(Number) as [number, number];
  return (ty - y) * 12 + tm - m;
}

/** Salário e férias são mensais; o 13º é tributado à parte. */
function wageKind(name: string): 'monthly' | 'thirteenth' | null {
  const f = fold(name);
  if (f === 'salario' || f === 'ferias') return 'monthly';
  // O núcleo hoje só reconhece "13o salario"; o nome do conjunto inicial é "13º salário".
  if (['13o salario', '13º salario', '13 salario', 'decimo terceiro'].includes(f))
    return 'thirteenth';
  return null;
}

function rulesFor(ds: TaxDataset, month: string): PayrollRules | undefined {
  return ds.payroll
    .filter((r) => r.from <= month)
    .sort((a, b) => (a.from < b.from ? 1 : -1))
    .at(0);
}

/** Ordem do `BTreeMap<Option<Id>>` do núcleo: sem titular primeiro, depois por id. */
const memberOrder = (a: string | null, b: string | null) =>
  a === b ? 0 : a == null ? -1 : b == null ? 1 : a < b ? -1 : 1;

function payrollLines(
  ds: TaxDataset,
  s: TaxSettings,
  year: number,
  from: string,
  to: string,
  notes: string[],
): PayrollLine[] {
  const name = (m: string | null) => (m ? (MEMBERS.find((x) => x.id === m)?.name ?? null) : null);
  const reported = new Map<string | null, [number, number, number]>();
  for (const r of reports(year)) {
    const e = reported.get(r.member_id) ?? [0, 0, 0];
    e[0] += r.gross + r.thirteenth_gross;
    e[1] += r.social_security;
    e[2] += r.income_tax + r.thirteenth_tax;
    reported.set(r.member_id, e);
  }
  const wageCats = new Map<string, 'monthly' | 'thirteenth'>();
  for (const c of CATEGORIES) {
    const k = c.kind === 'income' ? wageKind(c.name) : null;
    if (k) wageCats.set(c.id, k);
  }
  const net = new Map<string | null, { months: Map<string, number>; thirteenth: number }>();
  for (const t of getLedger().txns) {
    if (t.kind !== 'income' || t.status === 'planned' || t.date < from || t.date > to) continue;
    const k = t.category_id ? wageCats.get(t.category_id) : undefined;
    if (!k) continue;
    const e = net.get(t.member_id) ?? { months: new Map<string, number>(), thirteenth: 0 };
    if (k === 'monthly') {
      const m = t.date.slice(0, 7);
      e.months.set(m, (e.months.get(m) ?? 0) + t.amount);
    } else e.thirteenth += t.amount;
    net.set(t.member_id, e);
  }
  const out: PayrollLine[] = [];
  for (const member of [...reported.keys()].sort(memberOrder)) {
    const [gross, inss, ir] = reported.get(member)!;
    const n = net.get(member);
    out.push({
      member_id: member,
      member_name: name(member),
      origin: 'reported',
      net: n ? [...n.months.values()].reduce((a, b) => a + b, 0) + n.thirteenth : 0,
      gross,
      social_security: inss,
      income_tax: ir,
      months: n?.months.size ?? 0,
      legal_basis: [],
    });
  }
  if (ds.payroll.length === 0) {
    if (net.size > 0)
      notes.push(
        `Os dados de ${ds.name} não têm regras de renda: o imposto sobre o salário não é estimado (lance os informes de rendimentos).`,
      );
    return out;
  }
  for (const member of [...net.keys()].sort(memberOrder)) {
    if (reported.has(member)) continue;
    const { months, thirteenth } = net.get(member)!;
    let gross = 0;
    let inss = 0;
    let ir = 0;
    let received = 0;
    const basis: string[] = [];
    const run = (v: number, month: string) => {
      const r = rulesFor(ds, month);
      if (!r) return;
      const g = grossFromNet(v, s.dependents, r);
      const [i, t] = payroll(g, s.dependents, r);
      gross += g;
      inss += i;
      ir += t;
      received += v;
      if (!basis.includes(r.legal_basis)) basis.push(r.legal_basis);
    };
    for (const [month, v] of [...months.entries()].sort(([a], [b]) => (a < b ? -1 : 1)))
      run(v, month);
    if (thirteenth > 0) run(thirteenth, `${year}-12`);
    out.push({
      member_id: member,
      member_name: name(member),
      origin: 'estimated',
      net: received,
      gross,
      social_security: inss,
      income_tax: ir,
      months: months.size,
      legal_basis: basis,
    });
  }
  if (out.some((p) => p.origin === 'estimated'))
    notes.push(
      'Imposto e INSS do salário estimados a partir do líquido recebido, sem descontos como plano de saúde ou pensão: com o informe de rendimentos, a estimativa usa os valores dele.',
    );
  return out;
}

function physicalAssets(asOf: string): PhysicalAssetView[] {
  const list = extraMockHandler('physical.list');
  return list ? (list({ as_of: asOf, include_archived: true }) as PhysicalAssetView[]) : [];
}

function ipvaLines(region: TaxRegion | undefined, to: string, notes: string[]): PropertyLine[] {
  const vehicles = physicalAssets(to).filter(
    (a) =>
      a.kind === 'vehicle' && a.purchase_date <= to && (a.sale_date == null || a.sale_date > to),
  );
  if (vehicles.length === 0) return [];
  if (region?.ipva_car == null) {
    notes.push(
      'Há veículo cadastrado e nenhum IPVA lançado: escolha o estado para estimar o IPVA.',
    );
    return [];
  }
  const out = vehicles.map((v): PropertyLine => {
    const moto = v.fipe_vehicle_type === 'motos';
    const rate = (moto ? (region.ipva_motorcycle ?? region.ipva_car) : region.ipva_car) ?? 0;
    return {
      asset_id: v.id,
      name: v.name,
      tax: 'ipva',
      value: v.figures.value,
      rate,
      amount: taxOf(v.figures.value, rate),
      source_ids: [...region.source_ids],
    };
  });
  notes.push(
    'IPVA estimado pelo valor de cada veículo no fim do período e a alíquota do estado: lance o IPVA pago para usar o valor real.',
  );
  return out;
}

const PART_LABEL: Record<TaxPartKind, string> = {
  consumption: 'Consumo (embutido nos preços)',
  income_tax: 'Imposto de renda',
  social_security: 'Previdência (INSS)',
  property: 'Patrimônio (IPTU, IPVA)',
  investments: 'IR retido em proventos',
  other: 'IOF e outros tributos',
};
const PART_ORDER: readonly TaxPartKind[] = [
  'consumption',
  'income_tax',
  'social_security',
  'property',
  'investments',
  'other',
];

function estimate(args: Record<string, unknown>): TaxEstimate {
  const today = mockToday();
  const thisYear = Number(today.slice(0, 4));
  const year = typeof args.year === 'number' ? args.year : thisYear;
  if (year > thisYear) invalid('o ano ainda não começou');
  const from = `${year}-01-01`;
  const to = year === thisYear ? today : `${year}-12-31`;
  const s = { ...state.settings };
  const ds = dataset(s.country);
  const region = s.region
    ? ds.regions.find((r) => r.code.toLowerCase() === s.region!.toLowerCase())
    : undefined;
  const notes: string[] = [];
  if (!region && ds.regions.length > 0)
    notes.push(
      ds.country === 'BR'
        ? 'Escolha o estado (UF): sem ele, o ICMS fica na média nacional das tabelas e o IPVA não é estimado.'
        : ds.country === 'US'
          ? 'Escolha o estado: sem ele, o imposto sobre vendas não é estimado.'
          : 'Escolha o país: sem ele, o IVA não é estimado.',
    );

  const spent = spending(
    { from, to, basis: 'accrual', kind: 'expense', group_by: 'category' },
    'accrual',
  );
  const n = nodes();
  const icmsAdjust =
    ds.reference_icms != null && region?.icms_modal != null
      ? region.icms_modal - ds.reference_icms
      : 0;
  const consumption: ConsumptionLine[] = [];
  const unestimated: UnestimatedLine[] = [];
  const direct: DirectLine[] = [];
  for (const g of spent.groups) {
    const path = g.path ?? g.label;
    if (!g.key) {
      unestimated.push({
        category_id: null,
        path: 'Sem categoria',
        amount: g.amount,
        reason: 'uncategorized',
      });
      continue;
    }
    const cat = g.key;
    const r = resolve(cat, n, state.choices);
    if (!r) {
      unestimated.push({ category_id: cat, path, amount: g.amount, reason: 'unmapped' });
      continue;
    }
    const kind = kindOf(r.group);
    if (kind === 'direct_tax') {
      direct.push({
        category_id: cat,
        path,
        label: DIRECT.find(([k]) => k === r.group)?.[1] ?? '',
        group_key: r.group,
        amount: g.amount,
      });
      continue;
    }
    if (kind === 'not_consumption') {
      unestimated.push({ category_id: cat, path, amount: g.amount, reason: 'not_consumption' });
      continue;
    }
    const own = state.choices.get(cat)?.rate ?? null;
    const def = ds.consumption.find((x) => x.key === r.group);
    let rates: RateTriple;
    let adjust = 0;
    let sources: string[] = [];
    if (own != null) {
      rates = { conservative: own, probable: own, high: own };
    } else {
      const adj = def?.icms_modal ? icmsAdjust : 0;
      const v = (sp: RateSpec | null | undefined) => {
        const x = sp == null ? null : specValue(sp, region);
        return x == null ? null : Math.max(x + adj, 0);
      };
      const [c, p, h] = def ? [v(def.low), v(def.central), v(def.high)] : [null, null, null];
      if (c == null || p == null || h == null) {
        unestimated.push({ category_id: cat, path, amount: g.amount, reason: 'no_rate' });
        continue;
      }
      rates = { conservative: c, probable: p, high: h };
      adjust = adj;
      sources = [...def!.source_ids];
    }
    consumption.push({
      category_id: cat,
      path,
      spent: g.amount,
      group_key: r.group,
      group_label: def?.label ?? r.group,
      origin: r.origin,
      rates,
      own_rate: own,
      region_adjustment: adjust,
      taxes: {
        conservative: taxOf(g.amount, rates.conservative),
        probable: taxOf(g.amount, rates.probable),
        high: taxOf(g.amount, rates.high),
      },
      source_ids: sources,
    });
  }
  consumption.sort((a, b) => b.taxes.probable - a.taxes.probable);
  const unmapped = unestimated.filter((u) => u.reason === 'unmapped').length;
  if (unmapped > 0)
    notes.push(
      `${unmapped} categoria(s) sem grupo de imposto ficaram de fora: escolha o grupo delas.`,
    );

  const payrollRows = payrollLines(ds, s, year, from, to, notes);

  const incomes = extraMockHandler('portfolio.list_income');
  const investmentsWithheld = incomes
    ? (incomes({ from, to }) as IncomeEvent[])
        .filter((i) => i.currency === BASE)
        .reduce((sum, i) => sum + i.tax_withheld, 0)
    : 0;

  const ipvaRecorded = direct.some((d) => d.group_key === 'tax_property_ipva' && d.amount > 0);
  const property = ipvaRecorded ? [] : ipvaLines(region, to, notes);
  const iptuRecorded = direct.some((d) => d.group_key === 'tax_property_iptu');
  const hasRealEstate = physicalAssets(to).some(
    (a) => a.kind === 'real_estate' && !a.archived && (a.sale_date == null || a.sale_date >= from),
  );
  if (ds.country === 'BR' && hasRealEstate && !iptuRecorded)
    notes.push(
      'Há imóvel cadastrado e nenhum IPTU lançado no ano: o IPTU não entra na estimativa.',
    );

  const acc = new Map<TaxPartKind, { amount: AmountTriple; estimated: boolean }>();
  const add = (k: TaxPartKind, v: AmountTriple, est: boolean) => {
    const e = acc.get(k) ?? { amount: same(0), estimated: false };
    e.amount = {
      conservative: e.amount.conservative + v.conservative,
      probable: e.amount.probable + v.probable,
      high: e.amount.high + v.high,
    };
    e.estimated ||= est;
    acc.set(k, e);
  };
  for (const c of consumption) add('consumption', c.taxes, true);
  for (const d of direct)
    add(
      d.group_key === 'tax_income'
        ? 'income_tax'
        : d.group_key === 'tax_property_iptu' || d.group_key === 'tax_property_ipva'
          ? 'property'
          : 'other',
      same(d.amount),
      false,
    );
  for (const p of payrollRows) {
    const est = p.origin === 'estimated';
    add('income_tax', same(p.income_tax), est);
    add('social_security', same(p.social_security), est);
  }
  if (investmentsWithheld > 0) add('investments', same(investmentsWithheld), false);
  for (const p of property) add('property', same(p.amount), true);
  const parts: TaxPart[] = PART_ORDER.filter((k) => acc.has(k)).map((kind) => ({
    kind,
    label: PART_LABEL[kind],
    ...acc.get(kind)!,
  }));
  const totals = parts.reduce(
    (t, p) => ({
      conservative: t.conservative + p.amount.conservative,
      probable: t.probable + p.amount.probable,
      high: t.high + p.amount.high,
    }),
    same(0),
  );
  const consumptionSpent = consumption.reduce((sum, c) => sum + c.spent, 0);
  const consumptionTax = acc.get('consumption')?.amount ?? same(0);
  const received = getLedger()
    .txns.filter(
      (t) =>
        t.kind === 'income' &&
        t.status !== 'planned' &&
        t.date >= from &&
        t.date <= to &&
        t.currency === BASE,
    )
    .reduce((sum, t) => sum + t.amount, 0);
  const withheld = payrollRows.reduce((sum, p) => sum + p.gross - p.net, 0) + investmentsWithheld;
  const incomeBase = received + Math.max(withheld, 0);
  const share = (v: AmountTriple, base: number): RateTriple => ({
    conservative: ratio(v.conservative, base),
    probable: ratio(v.probable, base),
    high: ratio(v.high, base),
  });
  return {
    year,
    from,
    to,
    currency: BASE,
    settings: s,
    region_name: region?.name ?? null,
    dataset: {
      country: ds.country,
      name: ds.name,
      version: ds.version,
      updated: ds.updated,
      coverage: ds.coverage,
      stale: monthsSince(ds.updated, today) > STALE_MONTHS,
    },
    totals,
    parts,
    consumption,
    consumption_spent: consumptionSpent,
    unestimated,
    direct,
    payroll: payrollRows,
    investments_withheld: investmentsWithheld,
    property,
    income_base: incomeBase,
    share_of_income: share(totals, incomeBase),
    share_of_consumption: share(consumptionTax, consumptionSpent),
    // Dias de trabalho para pagar os impostos (imposto ÷ renda × dias do ano), como no núcleo.
    work_days:
      incomeBase > 0
        ? (() => {
            const days = new Date(year, 1, 29).getMonth() === 1 ? 366 : 365;
            const sh = share(totals, incomeBase);
            const d = (x: number) => Math.round(x * days * 10) / 10;
            return { conservative: d(sh.conservative), probable: d(sh.probable), high: d(sh.high) };
          })()
        : null,
    work_days_reference: ds.work_days ?? null,
    destination: ds.spending && totals.probable > 0 ? destination(ds.spending, totals) : null,
    context: ds.context ?? null,
    notes,
  };
}

/** Para onde vai o imposto, como `tax_burden::destination` do núcleo. */
function destination(sp: SpendingByFunction, totals: AmountTriple): TaxDestination {
  const sum = sp.functions.reduce((a, f) => a + f.share, 0) || 1;
  const part = (total: number, share: number) => Math.round((total * share) / sum);
  const lines = sp.functions.map((f) => ({
    key: f.key,
    label: f.label,
    share: f.share,
    amounts: {
      conservative: part(totals.conservative, f.share),
      probable: part(totals.probable, f.share),
      high: part(totals.high, f.share),
    },
  }));
  const top = lines.reduce((best, l) => (l.share > best.share ? l : best), lines[0]!);
  for (const k of ['conservative', 'probable', 'high'] as const)
    top.amounts[k] += totals[k] - lines.reduce((a, l) => a + l.amounts[k], 0);
  return { year: sp.year, label: sp.label, source_ids: sp.source_ids, lines };
}

/** Municípios inventados (a lista de verdade vem do IBGE). */
const MOCK_CITIES = ['Alto Exemplo', 'Porto Exemplo', 'Vila Exemplo'];

const cityState = (): CityState => ({
  city: state.city,
  revenue: state.revenue,
  source_enabled: true,
});

function setCity(a: Record<string, unknown>): CityState {
  const c = a.city as City | null | undefined;
  if (c && !/^\d{7}$/.test(c.code)) invalid('cidade inválida: escolha o município na lista');
  state.city = c ? { code: c.code, name: c.name, uf: c.uf.toUpperCase() } : null;
  if (state.revenue?.city.code !== state.city?.code) state.revenue = null;
  return cityState();
}

/** Arrecadação inventada para a cidade escolhida (no app, vem do Tesouro Nacional). */
function refreshCity(): CityState {
  const city = state.city;
  if (!city) invalid('escolha a cidade da família primeiro');
  const population = 20_000;
  const line = (key: string, label: string, amount: number) => ({
    key,
    label,
    amount,
    per_capita: Math.round(amount / population),
  });
  const lines = [
    line('iptu', 'IPTU', 1_000_000_000),
    line('iss', 'ISS', 1_500_000_000),
    line('itbi', 'ITBI', 200_000_000),
    line('irrf', 'Imposto de renda retido dos servidores', 250_000_000),
    line('outros', 'Taxas e contribuições de melhoria', 50_000_000),
  ];
  const total = lines.reduce((a, l) => a + l.amount, 0);
  const today = mockToday();
  state.revenue = {
    city,
    year: Number(today.slice(0, 4)),
    bimester: 4,
    until: `${today.slice(0, 4)}-08`,
    total,
    lines,
    population,
    per_capita: Math.round(total / population),
    fetched: today,
    source_url: 'https://apidatalake.tesouro.gov.br/ords/siconfi/tt/rreo',
  };
  return cityState();
}

const yearArg = (args: Record<string, unknown>) =>
  typeof args.year === 'number' ? args.year : null;

register({
  'tax_burden.datasets': () => DATASETS,
  'tax_burden.dataset': (a) => {
    if (typeof a.country !== 'string') invalid('argumentos inválidos: missing field `country`');
    return dataset(a.country);
  },
  'tax_burden.settings': () => ({ ...state.settings }),
  'tax_burden.set_settings': (a) => setSettings(a),
  'tax_burden.categories': () => categories(),
  'tax_burden.set_category': (a) => {
    setCategory(a);
    return categories();
  },
  'tax_burden.income_reports': (a) => reports(yearArg(a)),
  'tax_burden.save_income_report': (a) => reports(saveReport(a)),
  'tax_burden.delete_income_report': (a) => {
    const i = state.reports.findIndex((r) => r.id === a.id);
    if (i < 0) notFound(`informe ${String(a.id)}`);
    state.reports.splice(i, 1);
    return null;
  },
  'tax_burden.estimate': (a) => estimate(a),
  'tax_burden.city': () => cityState(),
  'tax_burden.cities': (a) => {
    const uf = typeof a.uf === 'string' ? a.uf.toUpperCase() : '';
    if (!/^[A-Z]{2}$/.test(uf)) invalid('estado (UF) inválido');
    return MOCK_CITIES.map((name, i) => ({
      code: `99999${String(i + 1).padStart(2, '0')}`,
      name,
      uf,
    }));
  },
  'tax_burden.set_city': (a) => setCity(a),
  'tax_burden.refresh_city': () => refreshCity(),
});
