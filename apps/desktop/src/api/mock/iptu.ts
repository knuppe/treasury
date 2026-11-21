/**
 * Mock do IPTU (`physical.iptu_*`, espelho simplificado de `physical::iptu`): o carnê do ano do
 * apartamento da Vila Mariana (cota única com desconto ou 10 parcelas), a situação de cada
 * parcela, o pago e o que falta, e a comparação cota única × parcelado. As percepções e a
 * comparação com o ano anterior e o mercado ficam vazias (o núcleo calcula).
 */
import type { IptuInput } from '@/api/generated/IptuInput';
import type { IptuInstallment } from '@/api/generated/IptuInstallment';
import type { IptuPlan } from '@/api/generated/IptuPlan';
import type { IptuTxnMatch } from '@/api/generated/IptuTxnMatch';
import type { IptuView } from '@/api/generated/IptuView';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addMonths, today } from './ledger/dates';
import { invalid, notFound, str } from './ledger/errors';
import { find, rows } from './ledger/store';

interface Inst {
  id: string;
  number: number;
  due_date: string;
  amount: number;
  paid_on: string | null;
  txn_id: string | null;
}

interface Bill extends Omit<IptuInput, 'installments' | 'plan'> {
  id: string;
  plan: IptuPlan | null;
  installments: Inst[];
}

let bills: Bill[] = [];
let seq = 0;

function seed(): Bill[] {
  const year = Number(today().slice(0, 4));
  const first = `${year}-02-10`;
  return [
    {
      id: 'iptu-apto',
      asset_id: 'phys-apto',
      year,
      registration: '034.112.0021-4',
      city: 'São Paulo',
      land_area: '62',
      built_area: '78',
      land_value: 21_000_000,
      building_value: 31_000_000,
      land_tax: 210_000,
      building_tax: 310_000,
      taxes_total: 520_000,
      lighting_fee: 0,
      waste_fee: 0,
      other_fees: 0,
      discount: 0,
      good_payer_discount: 0,
      total_due: 520_000,
      plan: 'installments',
      notes: undefined,
      installments: [
        {
          id: 'iptu-apto-0',
          number: 0,
          due_date: first,
          amount: 504_400,
          paid_on: null,
          txn_id: null,
        },
        ...Array.from({ length: 10 }, (_, k) => ({
          id: `iptu-apto-${k + 1}`,
          number: k + 1,
          due_date: addMonths(first, k),
          amount: 52_000,
          paid_on: addMonths(first, k) < today() ? addMonths(first, k) : null,
          txn_id: null,
        })),
      ],
    },
  ];
}

onMockReset(() => {
  bills = seed();
  seq = 0;
});
bills = seed();

const nid = (p: string) => `${p}-${++seq}`;

function effectivePlan(b: Bill): IptuPlan | null {
  if (b.plan) return b.plan;
  if (b.installments.some((i) => i.number > 0 && i.paid_on)) return 'installments';
  if (b.installments.some((i) => i.number === 0 && i.paid_on)) return 'single';
  return null;
}

const belongs = (plan: IptuPlan | null, n: number) =>
  plan === 'single' ? n === 0 : plan === 'installments' ? n > 0 : true;

function matches(due: string, amount: number): IptuTxnMatch[] {
  const used = new Set(bills.flatMap((b) => b.installments.map((i) => i.txn_id)).filter(Boolean));
  const tol = Math.max(Math.round(amount / 50), 100);
  return rows('txn')
    .filter(
      (t) =>
        !used.has(t.id) &&
        t.status !== 'planned' &&
        t.amount >= -amount - tol &&
        t.amount <= -amount + tol &&
        t.date >= addMonths(due, -1) &&
        t.date <= addMonths(due, 1),
    )
    .map((t) => ({
      txn_id: t.id,
      date: t.date,
      description: t.description,
      amount: t.amount,
      named: t.description.toLowerCase().includes('iptu'),
    }))
    .sort((a, b) => Number(b.named) - Number(a.named));
}

function view(b: Bill): IptuView {
  const plan = effectivePlan(b);
  const now = today();
  const installments: IptuInstallment[] = b.installments
    .slice()
    .sort((x, y) => x.number - y.number)
    .map((i) => {
      const status = i.paid_on
        ? 'paid'
        : !belongs(plan, i.number)
          ? 'not_chosen'
          : i.due_date < now
            ? 'late'
            : 'open';
      return {
        ...i,
        status,
        suggestion:
          status === 'open' || status === 'late'
            ? (matches(i.due_date, i.amount)[0] ?? null)
            : null,
      };
    });
  const venal =
    b.land_value != null || b.building_value != null
      ? (b.land_value ?? 0) + (b.building_value ?? 0)
      : null;
  const taxes =
    b.taxes_total ??
    (b.land_tax != null || b.building_tax != null
      ? (b.land_tax ?? 0) + (b.building_tax ?? 0)
      : null);
  const sum = (p: IptuPlan) =>
    b.installments
      .filter((i) => !i.paid_on && belongs(p, i.number))
      .reduce((s, i) => s + i.amount, 0);
  const single = b.installments.find((i) => i.number === 0);
  const parts = b.installments.filter((i) => i.number > 0);
  const partsTotal = parts.reduce((s, i) => s + i.amount, 0);
  return {
    id: b.id,
    asset_id: b.asset_id,
    year: b.year,
    registration: b.registration ?? null,
    city: b.city ?? null,
    land_area: b.land_area ?? null,
    built_area: b.built_area ?? null,
    land_value: b.land_value ?? null,
    building_value: b.building_value ?? null,
    venal_total: venal,
    land_tax: b.land_tax ?? null,
    building_tax: b.building_tax ?? null,
    taxes_total: taxes,
    lighting_fee: b.lighting_fee ?? 0,
    waste_fee: b.waste_fee ?? 0,
    other_fees: b.other_fees ?? 0,
    discount: b.discount ?? 0,
    good_payer_discount: b.good_payer_discount ?? 0,
    total_due: b.total_due,
    currency: 'BRL',
    plan,
    plan_chosen: b.plan != null,
    notes: b.notes ?? null,
    installments,
    effective_rate: taxes != null && venal ? taxes / venal : null,
    paid: b.installments.filter((i) => i.paid_on).reduce((s, i) => s + i.amount, 0),
    open: plan
      ? sum(plan)
      : b.installments.length === 0
        ? b.total_due
        : Math.min(...[sum('single'), sum('installments')].filter((x) => x > 0), Infinity) || 0,
    previous: null,
    market: null,
    single_vs_installments:
      single && parts.length > 0
        ? {
            single: single.amount,
            installments: partsTotal,
            count: parts.length,
            saving: partsTotal - single.amount,
            monthly_rate: null,
            cdi_monthly: null,
            best: single.amount < partsTotal ? 'single' : 'installments',
          }
        : null,
    insights: [],
  };
}

const bill = (id: string) => bills.find((b) => b.id === id) ?? notFound('carnê de IPTU');
const byInstallment = (id: string) =>
  bills.find((b) => b.installments.some((i) => i.id === id)) ?? notFound('parcela do IPTU');

registerMockHandlers(
  guarded({
    'physical.iptu_list': (a) =>
      bills
        .filter((b) => b.asset_id === str(a, 'asset_id'))
        .sort((x, y) => y.year - x.year)
        .map(view),
    'physical.iptu_matches': (a) => {
      const id = str(a, 'installment_id');
      const i = byInstallment(id).installments.find((x) => x.id === id)!;
      return matches(i.due_date, i.amount);
    },
    'physical.iptu_save': (a) => {
      const input = a as unknown as IptuInput;
      if (!(input.year >= 1990 && input.year <= 2100)) invalid('Ano do IPTU inválido.');
      const cur = bills.find((b) => b.asset_id === input.asset_id && b.year === input.year);
      const old = cur?.installments ?? [];
      const next: Bill = {
        ...input,
        id: cur?.id ?? nid('iptu'),
        plan: input.plan ?? null,
        installments: input.installments.map((i) => {
          const kept = old.find((o) => o.number === i.number);
          return {
            id: kept?.id ?? nid('iptu-parc'),
            number: i.number,
            due_date: i.due_date,
            amount: i.amount,
            paid_on: kept?.paid_on ?? null,
            txn_id: kept?.txn_id ?? null,
          };
        }),
      };
      bills = [...bills.filter((b) => b !== cur), next];
      return view(next);
    },
    'physical.iptu_delete': (a) => {
      const b = bill(str(a, 'id'));
      bills = bills.filter((x) => x !== b);
      return null;
    },
    'physical.iptu_pay': (a) => {
      const id = str(a, 'installment_id');
      const b = byInstallment(id);
      const i = b.installments.find((x) => x.id === id)!;
      const txnId = typeof a.txn_id === 'string' ? a.txn_id : null;
      const t = txnId ? (find('txn', txnId) ?? notFound('lançamento')) : null;
      const paidOn = t?.date ?? (typeof a.paid_on === 'string' ? a.paid_on : null);
      if (!paidOn) invalid('Escolha o lançamento do pagamento ou informe a data.');
      i.paid_on = paidOn;
      i.txn_id = txnId;
      b.plan = i.number === 0 ? 'single' : 'installments';
      return view(b);
    },
    'physical.iptu_unpay': (a) => {
      const id = str(a, 'installment_id');
      const b = byInstallment(id);
      const i = b.installments.find((x) => x.id === id)!;
      i.paid_on = null;
      i.txn_id = null;
      return view(b);
    },
  }),
);
