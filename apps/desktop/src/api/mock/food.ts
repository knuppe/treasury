/**
 * Mock de `food.*` (docs/consumption.md, docs/consumption.md), simplificado: produtos pelos itens das
 * compras do mock (o mesmo produto pela descrição), categorias de produto da semente com os
 * segmentos como raízes e os grupos do cupom dentro deles, classes editáveis, a compra cadastrada
 * à mão (cria, liga ou deixa sem lançamento) e o resumo do mês por segmento, pelos itens das
 * compras com lançamento. Vale em qualquer país.
 */
import type { BasketInflation } from '@/api/generated/BasketInflation';
import type { FoodClass } from '../generated/FoodClass';
import type { FoodLineSibling } from '../generated/FoodLineSibling';
import type { FoodMergeSuggestion } from '../generated/FoodMergeSuggestion';
import type { HealthAllergenAlert } from '../generated/HealthAllergenAlert';
import type { HealthProduct } from '../generated/HealthProduct';
import type { HealthSummary } from '../generated/HealthSummary';
import type { MemberDiet } from '../generated/MemberDiet';
import type { OpenDataSync } from '../generated/OpenDataSync';
import type { ProductHealth } from '../generated/ProductHealth';
import type { ProductOpenData } from '../generated/ProductOpenData';
import type { FoodMonth } from '../generated/FoodMonth';
import type { FoodProduct } from '../generated/FoodProduct';
import type { FoodSummary } from '../generated/FoodSummary';
import type { ItemGroup } from '../generated/ItemGroup';
import type { ProductCategory } from '../generated/ProductCategory';
import type { Purchase } from '../generated/Purchase';
import type { PurchaseInput } from '../generated/PurchaseInput';
import type { Receipt } from '../generated/Receipt';
import type { ReceiptItem } from '../generated/ReceiptItem';
import type { TxnMatch } from '../generated/TxnMatch';
import { extraMockHandler, mockFail, onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { addMonths, today } from './ledger/dates';
import { newId, rows } from './ledger/store';
import {
  baseOf,
  groupLabel,
  guessGroup,
  mockDeleteReceipt,
  mockReceipts,
  mockSaveReceipt,
} from './receipts';
import { familyCountry } from './country';
import { sourceEnabled } from './sources';

/** Segmentos (raízes) e os grupos do cupom dentro de cada um. */
const SEGMENTS: [string, string, boolean, FoodClass | null, ItemGroup[]][] = [
  [
    'seg.alimentacao',
    'Alimentação',
    true,
    null,
    [
      'hortifruti',
      'carnes',
      'laticinios',
      'mercearia',
      'padaria',
      'doces_lanches',
      'congelados',
      'bebidas',
      'bebidas_alcoolicas',
    ],
  ],
  ['seg.casa_limpeza', 'Casa e limpeza', false, null, ['limpeza', 'casa']],
  ['seg.higiene_beleza', 'Higiene e beleza', false, null, ['higiene']],
  ['seg.saude', 'Saúde', false, 'essential', ['farmacia', 'bebe']],
  ['seg.vestuario', 'Vestuário e calçados', false, 'complementary', []],
  ['seg.eletronicos', 'Eletrônicos e eletrodomésticos', false, 'complementary', []],
  ['seg.lazer', 'Lazer', false, 'superfluous', []],
];
const GROUPS: [ItemGroup, string, boolean, FoodClass | null][] = [
  ['hortifruti', 'Hortifrúti', true, 'essential'],
  ['carnes', 'Carnes, aves e peixes', true, 'essential'],
  ['laticinios', 'Leite, frios e ovos', true, 'essential'],
  ['mercearia', 'Mercearia', true, 'essential'],
  ['padaria', 'Padaria', true, 'essential'],
  ['doces_lanches', 'Doces, biscoitos e salgadinhos', true, 'superfluous'],
  ['congelados', 'Congelados e prontos', true, 'complementary'],
  ['bebidas', 'Bebidas não alcoólicas', true, 'complementary'],
  ['bebidas_alcoolicas', 'Bebidas alcoólicas', true, 'superfluous'],
  ['limpeza', 'Limpeza', false, 'essential'],
  ['higiene', 'Higiene', false, 'essential'],
  ['bebe', 'Bebê', false, 'essential'],
  ['pet', 'Pet', false, 'complementary'],
  ['farmacia', 'Farmácia', false, 'essential'],
  ['casa', 'Casa e utilidades', false, 'complementary'],
  ['cigarros', 'Cigarros', false, 'superfluous'],
  ['outros', 'Outros', false, null],
];
const SUBS: [string, string, string, RegExp][] = [
  ['vestuario.roupas', 'seg.vestuario', 'Roupas', /camis|blusa|cal[cç]a|vestido|jaqueta|meia/i],
  ['vestuario.calcados', 'seg.vestuario', 'Calçados', /t[eê]nis|sapato|sand[aá]lia|chinelo/i],
  [
    'eletronicos.informatica',
    'seg.eletronicos',
    'Celulares e informática',
    /celular|fone|notebook|carregador/i,
  ],
  ['lazer.brinquedos', 'seg.lazer', 'Brinquedos', /brinquedo|boneca|lego/i],
];

interface Cat {
  id: string;
  parent_id: string | null;
  seed_key: string | null;
  name: string;
  is_food: boolean;
  class: FoodClass | null;
}
let cats: Cat[] = [];
const productClass = new Map<string, FoodClass | null>();
/** Códigos de barras digitados à mão, por produto (no núcleo: `product_gtin`, origem manual). */
const manualGtins = new Map<string, string[]>();

/** O mesmo teste do núcleo (`food::gtin::check`), simplificado. */
function checkGtin(code: string): string {
  const c = code.replace(/[\s-]/g, '');
  if (!/^\d+$/.test(c) || ![8, 12, 13, 14].includes(c.length))
    mockFail('invalid', 'O código de barras tem 8, 12, 13 ou 14 números (só números).');
  const d = [...c].map(Number);
  const check = d.pop() ?? 0;
  const sum = d.reverse().reduce((s, v, i) => s + (i % 2 === 0 ? v * 3 : v), 0);
  if ((10 - (sum % 10)) % 10 !== check)
    mockFail('invalid', 'O último número do código de barras não confere (dígito verificador).');
  const g = c.padStart(14, '0');
  if (g[1] === '2' || ['02', '04', '05'].includes(g.slice(1, 3)) || g[0] === '9')
    mockFail(
      'invalid',
      'Esse é um código de uso interno da loja, não o código de barras do produto.',
    );
  return g.replace(/^0/, '');
}
const productCategory = new Map<string, string | null>();
let seq = 0;
const catId = (key: string) => `pc-${key}`;
function seed() {
  if (cats.length) return;
  const parentOf = (g: ItemGroup) => SEGMENTS.find((s) => s[4].includes(g))?.[0] ?? null;
  cats = [
    ...SEGMENTS.map(([k, name, food, cls]) => ({
      id: catId(k),
      parent_id: null,
      seed_key: k,
      name,
      is_food: food,
      class: cls,
    })),
    ...GROUPS.map(([k, name, food, cls]) => ({
      id: catId(k),
      parent_id: parentOf(k) ? catId(parentOf(k)!) : null,
      seed_key: k,
      name,
      is_food: food,
      class: cls,
    })),
    ...SUBS.map(([k, parent, name]) => ({
      id: catId(k),
      parent_id: catId(parent),
      seed_key: k,
      name,
      is_food: false,
      class: null,
    })),
  ];
}
onMockReset(() => {
  cats = [];
  productClass.clear();
  productCategory.clear();
  manualGtins.clear();
});

function tree(): ProductCategory[] {
  seed();
  const out: ProductCategory[] = [];
  const walk = (
    parent: string | null,
    prefix: string,
    inherited: FoodClass | null,
    depth: number,
  ) => {
    for (const c of cats.filter((x) => x.parent_id === parent)) {
      const path = prefix ? `${prefix} › ${c.name}` : c.name;
      const eff = c.class ?? inherited;
      out.push({
        id: c.id,
        parent_id: c.parent_id,
        seed_key: c.seed_key,
        name: c.name,
        path,
        depth,
        is_food: c.is_food,
        class: c.class,
        effective_class: eff,
        products: products().filter((p) => p.category_id === c.id).length,
      });
      walk(c.id, path, eff, depth + 1);
    }
  };
  walk(null, '', null, 0);
  return out;
}

/** A raiz (o segmento) de uma categoria. */
function rootOf(id: string | null): string | null {
  let c = cats.find((x) => x.id === id);
  while (c?.parent_id) c = cats.find((x) => x.id === c!.parent_id);
  return c?.id ?? null;
}

const keyOf = (d: string) => d.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().trim();
const productId = (description: string) => `prod-${keyOf(description).replace(/\W+/g, '-')}`;

/** Categoria automática: a subcategoria pelas palavras, ou a do grupo do cupom. */
function autoCategory(it: ReceiptItem): string {
  const sub = SUBS.find(([, , , re]) => re.test(it.description));
  return sub ? catId(sub[0]) : catId(it.group);
}

function products(): FoodProduct[] {
  seed();
  const by = new Map<string, FoodProduct & { prices: [string, number | null][] }>();
  for (const r of mockReceipts()) {
    for (const it of r.items) {
      const id = productId(it.description);
      const cid = productCategory.has(id) ? productCategory.get(id)! : autoCategory(it);
      const path = tree0().find((c) => c.id === cid);
      const p = by.get(id) ?? {
        id,
        name: it.description,
        gtin: it.gtin,
        gtins: [],
        pack_qty: null,
        pack_unit: null,
        category_id: cid,
        category_path: path?.path ?? null,
        class: productClass.get(id) ?? null,
        effective_class: (productClass.get(id) ?? null) || (path?.effective_class ?? null),
        is_food: path?.is_food ?? false,
        purchases: 0,
        spent: 0,
        stores: 1,
        first_date: r.date,
        last_date: r.date,
        base_unit: it.base_unit,
        first_price: it.base_price,
        last_price: it.base_price,
        change: null,
        last_unit_price: it.unit_price,
        last_unit: it.unit,
        prices: [],
      };
      p.purchases += 1;
      p.spent += it.total;
      p.prices.push([r.date, it.base_price]);
      by.set(id, p);
    }
  }
  for (const [id, codes] of manualGtins) {
    const p = by.get(id);
    if (p) {
      p.gtins = [...new Set([...codes, ...(p.gtin ? [p.gtin] : [])])];
      p.gtin = p.gtins[0] ?? null;
    }
  }
  return [...by.values()].map(({ prices, ...p }) => {
    prices.sort((a, b) => a[0].localeCompare(b[0]));
    const first = prices[0]?.[1] ?? null;
    const last = prices[prices.length - 1]?.[1] ?? null;
    return {
      ...p,
      first_date: prices[0]?.[0] ?? null,
      last_date: prices[prices.length - 1]?.[0] ?? null,
      first_price: first,
      last_price: last,
      change: first && last ? last / first - 1 : null,
    };
  });
}

/** A árvore sem a contagem de produtos (para montar os produtos sem recursão). */
function tree0(): Omit<ProductCategory, 'products'>[] {
  seed();
  const out: Omit<ProductCategory, 'products'>[] = [];
  const walk = (parent: string | null, prefix: string, inherited: FoodClass | null, d: number) => {
    for (const c of cats.filter((x) => x.parent_id === parent)) {
      const path = prefix ? `${prefix} › ${c.name}` : c.name;
      const eff = c.class ?? inherited;
      out.push({ ...c, path, depth: d, effective_class: eff });
      walk(c.id, path, eff, d + 1);
    }
  };
  walk(null, '', null, 0);
  return out;
}

const label = (c: FoodClass | null) =>
  c === 'essential'
    ? 'Essencial'
    : c === 'complementary'
      ? 'Complementar'
      : c === 'superfluous'
        ? 'Supérfluo'
        : 'Sem classe';

function summary(a: Record<string, unknown>): FoodSummary {
  const month = typeof a.month === 'string' ? a.month : today().slice(0, 7);
  const segment = typeof a.segment_id === 'string' ? a.segment_id : null;
  const list = products();
  const byId = new Map(list.map((p) => [p.id, p]));
  const linked = (r: Receipt) => Boolean(r.txn_id);
  const itemsOf = (m: string) =>
    mockReceipts()
      .filter((r) => r.date.startsWith(m) && linked(r))
      .flatMap((r) => r.items)
      .map((it) => {
        const p = byId.get(productId(it.description));
        return { it, p, seg: rootOf(p?.category_id ?? null) };
      });
  const segName = (id: string | null) => cats.find((c) => c.id === id)?.name ?? 'Sem categoria';
  const monthOf = (m: string): FoodMonth => {
    const items = itemsOf(m);
    const out = rows('txn')
      .filter(
        (t) =>
          t.date.startsWith(m) &&
          t.kind === 'expense' &&
          /restaurante|delivery|ifood/i.test(t.description),
      )
      .reduce((s, t) => s - t.amount, 0);
    const segs = new Map<string | null, number>();
    for (const x of items) segs.set(x.seg, (segs.get(x.seg) ?? 0) + x.it.total);
    return {
      month: m,
      home: items.filter((x) => x.p?.is_food).reduce((s, x) => s + x.it.total, 0),
      out,
      non_food: items.filter((x) => !x.p?.is_food).reduce((s, x) => s + x.it.total, 0),
      segments: [...segs].map(([id, spent]) => ({ category_id: id, name: segName(id), spent })),
    };
  };
  const prevMonth = addMonths(`${month}-01`, -1).slice(0, 7);
  const now = itemsOf(month);
  const before = itemsOf(prevMonth);
  const sum = (xs: { it: ReceiptItem }[]) => xs.reduce((s, x) => s + x.it.total, 0);
  const segIds = [...new Set([...now, ...before].map((x) => x.seg))];
  const segments = segIds
    .map((id) => {
      const c = cats.find((x) => x.id === id);
      const mine = now.filter((x) => x.seg === id);
      return {
        category_id: id,
        seed_key: c?.seed_key ?? null,
        name: segName(id),
        is_food: c?.is_food ?? false,
        spent: sum(mine),
        items: mine.length,
        previous: sum(before.filter((x) => x.seg === id)),
      };
    })
    .sort((x, y) => y.spent - x.spent);
  const cur = monthOf(month);
  const detail = now.filter((x) => !segment || x.seg === segment);
  const classes = new Map<FoodClass | null, number>();
  const byCat = new Map<string, number>();
  const byProduct = new Map<string, [number, number]>();
  for (const x of detail) {
    const c = x.p?.effective_class ?? null;
    classes.set(c, (classes.get(c) ?? 0) + x.it.total);
    let k = segment ? (x.p?.category_id ?? '') : (x.seg ?? '');
    if (segment) {
      // A categoria logo abaixo do segmento.
      let c2 = cats.find((y) => y.id === k);
      while (c2?.parent_id && c2.parent_id !== segment)
        c2 = cats.find((y) => y.id === c2!.parent_id);
      k = c2?.id ?? k;
    }
    byCat.set(k, (byCat.get(k) ?? 0) + x.it.total);
    if (x.p) {
      const e = byProduct.get(x.p.id) ?? [0, 0];
      byProduct.set(x.p.id, [e[0] + x.it.total, e[1] + 1]);
    }
  }
  const people = rows('member').filter((m) => !m.archived).length || 1;
  const without = mockReceipts().filter((r) => r.date.startsWith(month) && !linked(r));
  return {
    month,
    segment_id: segment,
    total: sum(now),
    previous_total: sum(before),
    segments,
    receipt_items: cur.home,
    market_without_receipt: 0,
    home: cur.home,
    out: cur.out,
    out_categories: [],
    people,
    per_person: Math.round((cur.home + cur.out) / people || 0),
    classes: [...classes].map(([c, v]) => ({ class: c, label: label(c), spent: v })),
    categories: [...byCat]
      .map(([id, v]) => {
        const c = cats.find((x) => x.id === id);
        return {
          category_id: id || null,
          name: c?.name ?? 'Sem categoria',
          class: c?.class ?? null,
          spent: v,
          items: 0,
        };
      })
      .sort((x, y) => y.spent - x.spent),
    top_products: [...byProduct]
      .map(([id, [spent, items]]) => ({
        product_id: id,
        name: byId.get(id)?.name ?? id,
        category_path: byId.get(id)?.category_path ?? null,
        spent,
        items,
      }))
      .sort((x, y) => y.spent - x.spent)
      .slice(0, 8),
    risers: list
      .filter((p) => byProduct.has(p.id) && p.purchases >= 2 && (p.change ?? 0) > 0)
      .sort((x, y) => (y.change ?? 0) - (x.change ?? 0))
      .slice(0, 5),
    months: [5, 4, 3, 2, 1, 0].map((k) => monthOf(addMonths(`${month}-01`, -k).slice(0, 7))),
    receipts_without_txn: without.length,
    receipts_without_txn_amount: without.reduce((s, r) => s + r.paid, 0),
    receipts: mockReceipts().filter((r) => r.date.startsWith(month)).length,
    unlinked_items: 0,
  };
}

function purchases(a: Record<string, unknown>): Purchase[] {
  const from = typeof a.from === 'string' ? a.from : '0000';
  const to = typeof a.to === 'string' ? a.to : '9999';
  const q = typeof a.query === 'string' ? keyOf(a.query) : '';
  const segment = typeof a.segment_id === 'string' ? a.segment_id : null;
  const byId = new Map(products().map((p) => [p.id, p]));
  return mockReceipts()
    .filter((r) => r.date >= from && r.date <= to)
    .filter(
      (r) =>
        !q ||
        keyOf(r.store_name ?? '').includes(q) ||
        r.items.some((i) => keyOf(i.description).includes(q)),
    )
    .map((r) => {
      const segs = new Map<string | null, number>();
      for (const it of r.items) {
        const s = rootOf(byId.get(productId(it.description))?.category_id ?? null);
        segs.set(s, (segs.get(s) ?? 0) + it.total);
      }
      return {
        receipt: { ...r, items: [] },
        segments: [...segs]
          .map(([id, spent]) => ({
            category_id: id,
            name: cats.find((c) => c.id === id)?.name ?? 'Sem categoria',
            spent,
          }))
          .sort((x, y) => y.spent - x.spent),
      };
    })
    .filter((p) => !segment || p.segments.some((s) => s.category_id === segment))
    .sort((x, y) => y.receipt.date.localeCompare(x.receipt.date));
}

function candidates(a: Record<string, unknown>): TxnMatch[] {
  const amount = Math.abs(Number(a.amount) || 0);
  const date = typeof a.date === 'string' ? a.date : today();
  const own = typeof a.purchase_id === 'string' ? a.purchase_id : null;
  return rows('txn')
    .filter(
      (t) =>
        t.kind === 'expense' &&
        // A compra cabe no que ainda não foi detalhado do lançamento.
        -t.amount -
          mockReceipts()
            .filter((r) => r.txn_id === t.id && r.id !== own)
            .reduce((s, r) => s + r.paid, 0) >=
          amount &&
        Math.abs(Date.parse(t.date) - Date.parse(date)) <= 5 * 86_400_000,
    )
    .sort((x, y) => Number(x.amount !== -amount) - Number(y.amount !== -amount))
    .slice(0, 5)
    .map((t) => ({
      txn_id: t.id,
      date: t.date,
      amount: t.amount,
      description: t.description,
      account_name: rows('account').find((x) => x.id === t.account_id)?.name ?? '?',
      member_id: t.member_id ?? null,
    }));
}

const call = (cmd: string, args: Record<string, unknown>) =>
  (extraMockHandler(cmd) ?? mockFail('internal', cmd))(args);

function savePurchase(p: PurchaseInput): Receipt {
  const old = p.id ? mockReceipts().find((r) => r.id === p.id) : null;
  if (p.id && old?.origin !== 'manual')
    mockFail('invalid', 'Só a compra cadastrada à mão pode ser editada aqui.');
  const lines = p.items.filter((i) => i.description.trim());
  if (!lines.length) mockFail('invalid', 'Coloque pelo menos um item com descrição e valor.');
  if (lines.some((i) => i.total <= 0))
    mockFail('invalid', 'Cada item precisa de um valor maior que zero.');
  const itemsTotal = lines.reduce((s, i) => s + i.total, 0);
  const discount = Math.abs(p.discount ?? 0);
  // Imposto somado no fim do cupom (EUA, Canadá), como no núcleo.
  const taxAdded = ['US', 'CA'].includes(familyCountry()) ? Math.abs(p.tax ?? 0) : 0;
  const paid = itemsTotal - discount + taxAdded;
  const store = p.store_name?.trim() || null;
  const ownTxn = old?.txn_created ? old.txn_id : null;
  let txnId: string | null = null;
  let created = false;
  const pay = p.payment;
  if (pay.mode === 'link' && pay.txn_id === ownTxn) {
    call('transactions.update', {
      id: ownTxn,
      amount: -paid,
      date: p.date,
      description: store ?? 'Compra',
    });
    txnId = ownTxn;
    created = true;
  } else if (pay.mode === 'link') {
    const t =
      rows('txn').find((x) => x.id === pay.txn_id) ??
      mockFail('not_found', 'Lançamento não encontrado.');
    const used = mockReceipts()
      .filter((r) => r.txn_id === t.id && r.id !== p.id)
      .reduce((s, r) => s + r.paid, 0);
    if (paid > -t.amount - used)
      mockFail('invalid', 'A compra passa do que ainda cabe nesse lançamento.');
    txnId = t.id;
  } else if (pay.mode === 'create') {
    const t = call('transactions.create', {
      account_id: pay.account_id,
      date: p.date,
      amount: -paid,
      kind: 'expense',
      description: store ?? 'Compra',
      ...(pay.category_id ? { category_id: pay.category_id } : {}),
    }) as { id: string };
    txnId = t.id;
    created = true;
  }
  if (ownTxn && ownTxn !== txnId) call('transactions.delete', { ids: [ownTxn] });
  const items: ReceiptItem[] = lines.map((i, k) => {
    const group = guessGroup(i.description);
    const id = i.product_id ?? productId(i.description);
    if (i.category_id) productCategory.set(id, i.category_id);
    return {
      line: k + 1,
      code: null,
      description: i.description.trim(),
      quantity: i.quantity ?? null,
      unit: i.unit ?? null,
      unit_price: i.unit_price ?? null,
      total: i.total,
      group,
      group_label: groupLabel(group),
      group_origin: 'auto',
      tax_rate: 0,
      tax: 0,
      tax_origin: 'estimated',
      gtin: null,
      product_id: id,
      category_id: i.category_id ?? productCategory.get(id) ?? null,
      ...baseOf(i.description, i.quantity ?? null, i.unit ?? null, i.total),
    };
  });
  const member =
    p.member_id ?? (txnId ? (rows('txn').find((t) => t.id === txnId)?.member_id ?? null) : null);
  const r: Receipt = {
    id: old?.id ?? newId(),
    access_key: null,
    date: p.date,
    issued_at: null,
    store_name: store,
    store_cnpj: null,
    uf: null,
    items_total: itemsTotal,
    discount,
    paid,
    tax: 0,
    tax_printed: false,
    origin: 'manual',
    txn_id: txnId,
    txn_created: created,
    tax_added: taxAdded,
    member_id: member,
    member_name: member ? (rows('member').find((m) => m.id === member)?.name ?? null) : null,
    item_count: items.length,
    items,
  };
  mockSaveReceipt(r);
  return r;
}

/** Índice da cesta inventado: 12 meses, a cesta acima do IPCA de alimentos. */
function basket(): BasketInflation {
  const months = Array.from({ length: 12 }, (_, i) => {
    const d = new Date(2025, 10 + i, 1);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
  });
  return {
    months: months.map((month, i) => ({
      month,
      index: 100 * Math.pow(1.011, i),
      change: i === 0 ? null : 0.011,
      coverage: 0.62,
      official: 100 * Math.pow(1.006, i),
    })),
    change: Math.pow(1.011, 11) - 1,
    official_change: Math.pow(1.006, 11) - 1,
    official_name: 'IPCA · Alimentação no domicílio',
    same_basket_now: 126_800,
    same_basket_year_ago: 112_000,
    risers: [
      {
        product_id: 'cafe',
        name: 'Café Exemplo Torrado 500g',
        base_unit: 'kg',
        before: 4000,
        now: 5200,
        change: 0.3,
        impact: 1800,
      },
    ],
    fallers: [
      {
        product_id: 'arroz',
        name: 'Arroz Exemplo Tipo1 5kg',
        base_unit: 'kg',
        before: 600,
        now: 540,
        change: -0.1,
        impact: -600,
      },
    ],
    enough: true,
  };
}

/** Saúde da cesta inventada (docs/gtin-inflation.md §7): produtos fictícios, sem marcas reais. */
type OffMock = HealthProduct & { allergens: string[]; traces: string[] };
const OFF_PRODUCTS: OffMock[] = [
  {
    product_id: 'off-biscoito',
    name: 'Biscoito Recheado Exemplo 130g',
    spent: 4_200,
    nova: 4,
    high_in: ['added_sugar', 'saturated_fat'],
    protein_per_money: null,
    allergens: ['gluten', 'milk'],
    traces: ['soybeans'],
  },
  {
    product_id: 'off-refrigerante',
    name: 'Refrigerante Exemplo 2L',
    spent: 3_600,
    nova: 4,
    high_in: ['added_sugar'],
    protein_per_money: null,
    allergens: [],
    traces: [],
  },
  {
    product_id: 'off-salgadinho',
    name: 'Salgadinho Exemplo 90g',
    spent: 2_100,
    nova: 4,
    high_in: ['sodium', 'saturated_fat'],
    protein_per_money: null,
    allergens: [],
    traces: ['milk'],
  },
  {
    product_id: 'off-feijao',
    name: 'Feijão Carioca Exemplo 1kg',
    spent: 2_800,
    nova: 1,
    high_in: [],
    protein_per_money: 24.5,
    allergens: [],
    traces: [],
  },
  {
    product_id: 'off-frango',
    name: 'Peito de Frango Exemplo kg',
    spent: 6_400,
    nova: 1,
    high_in: [],
    protein_per_money: 12.8,
    allergens: [],
    traces: [],
  },
  {
    product_id: 'off-iogurte',
    name: 'Iogurte Natural Exemplo 170g',
    spent: 1_500,
    nova: 1,
    high_in: [],
    protein_per_money: 4.1,
    allergens: ['milk'],
    traces: [],
  },
  {
    product_id: 'off-pao',
    name: 'Pão de Forma Exemplo 500g',
    spent: 1_900,
    nova: 4,
    high_in: [],
    protein_per_money: 6.2,
    allergens: ['gluten'],
    traces: ['sesame-seeds'],
  },
];
/** O Open Food Facts já foi consultado (o botão "buscar dados"). */
let offSynced = false;
const merged = new Set<string>();
onMockReset(() => {
  offSynced = false;
  merged.clear();
});

/** A restrição bate com o produto: `false` contém, `true` só "pode conter". */
function conflicts(p: OffMock, r: string): boolean | null {
  const tag = r === 'sesame' ? 'sesame-seeds' : r;
  if (r === 'vegan')
    return p.allergens.some((a) => ['milk', 'eggs', 'fish', 'crustaceans'].includes(a))
      ? false
      : null;
  if (r === 'vegetarian')
    return p.allergens.some((a) => ['fish', 'crustaceans'].includes(a)) ? false : null;
  if (p.allergens.includes(tag)) return false;
  if (p.traces.includes(tag)) return true;
  return null;
}

function healthSummary(): HealthSummary {
  const on = sourceEnabled('open_food_facts');
  const ready = on && offSynced;
  const strip = ({ allergens: _a, traces: _t, ...p }: OffMock): HealthProduct => p;
  const allergens: HealthAllergenAlert[] = [];
  if (ready)
    for (const d of call('food.diet_list', {}) as MemberDiet[])
      for (const p of OFF_PRODUCTS)
        for (const r of d.restrictions) {
          const trace = conflicts(p, r);
          if (trace == null) continue;
          allergens.push({
            member_id: d.member_id,
            member_name: d.member_name,
            product_id: p.product_id,
            product_name: p.name,
            restriction: r,
            trace,
            last_date: today(),
          });
        }
  return {
    source_enabled: on,
    coverage: ready ? 0.64 : 0,
    enough: ready,
    ultra_share: ready ? 0.31 : null,
    ultra_share_year_ago: ready ? 0.38 : null,
    high_in: ready ? OFF_PRODUCTS.filter((p) => p.high_in.length).map(strip) : [],
    protein: ready
      ? OFF_PRODUCTS.filter((p) => p.protein_per_money != null)
          .sort((a, b) => (b.protein_per_money ?? 0) - (a.protein_per_money ?? 0))
          .map(strip)
      : [],
    allergens,
    pending: offSynced ? 0 : OFF_PRODUCTS.length,
  };
}

function openDataSync(): OpenDataSync {
  if (!sourceEnabled('open_food_facts'))
    mockFail('invalid', 'a fonte Open Food Facts está desligada em Ajustes › Fontes de dados');
  const fetched = offSynced ? 0 : OFF_PRODUCTS.length;
  offSynced = true;
  return { fetched, found: Math.max(0, fetched - 1), failed: 0, remaining: 0 };
}

/** Pares inventados de nomes parecidos (o mesmo produto lido de cupons diferentes). */
const MERGE: FoodMergeSuggestion[] = [
  {
    a_id: 'off-iogurte',
    a_name: 'Iogurte Natural Exemplo 170g',
    b_id: 'prod-iog-nat-exemplo-170g',
    b_name: 'IOG NAT EXEMPLO 170G',
    score: 0.82,
  },
  {
    a_id: 'off-feijao',
    a_name: 'Feijão Carioca Exemplo 1kg',
    b_id: 'prod-feijao-car-exemplo-1kg',
    b_name: 'FEIJAO CAR EXEMPLO 1KG',
    score: 0.77,
  },
];

/** Os dados do Open Food Facts de um produto do mock (inventados), quando há. */
function productHealth(name: string): ProductHealth | null {
  if (!sourceEnabled('open_food_facts') || !offSynced || !/LEITE/i.test(name)) return null;
  const data: ProductOpenData = {
    gtin: '7890000000017',
    found: true,
    fetched_at: `${today()}T10:00:00Z`,
    name: 'Leite Integral Exemplo 1L',
    brand: 'Exemplo',
    quantity: '1 L',
    categories: ['milks'],
    nova: 1,
    nutriscore: 'b',
    per: '100ml',
    energy_kcal: 61,
    sugars: 4.7,
    added_sugars: 0,
    saturated_fat: 2.1,
    sodium_mg: 50,
    protein: 3.1,
    fiber: null,
    allergens: ['milk'],
    traces: [],
    labels: [],
    page_url: 'https://world.openfoodfacts.org/product/7890000000017',
    source_updated: null,
  };
  return { data, high_in: [] };
}

/** Outro tamanho inventado do café (a mesma linha, para a reduflação). */
const sameLine = (name: string): FoodLineSibling[] =>
  /CAFE/i.test(name)
    ? [
        {
          id: 'prod-cafe-torrado-250g',
          name: 'CAFE TORRADO 250G',
          pack_qty: '250',
          pack_unit: 'g',
          last_price: 6_400,
          base_unit: 'kg',
          last_date: today(),
        },
      ]
    : [];

registerMockHandlers(
  guarded({
    'food.basket_inflation': () => basket(),
    'food.health_summary': () => healthSummary(),
    'food.open_data_sync': () => openDataSync(),
    'food.merge_suggestions': () => MERGE.filter((m) => !merged.has(m.b_id)),
    'food.summary': (a) => summary(a),
    'food.products': (a) => {
      const q = typeof a.query === 'string' ? keyOf(a.query) : '';
      const within =
        typeof a.category_id === 'string'
          ? (id: string | null) => {
              let c = cats.find((x) => x.id === id);
              while (c && c.id !== a.category_id) c = cats.find((x) => x.id === c!.parent_id);
              return Boolean(c);
            }
          : () => true;
      return products()
        .filter((p) => (typeof a.food === 'boolean' ? p.is_food === a.food : true))
        .filter((p) => within(p.category_id))
        .filter((p) => !q || keyOf(p.name).includes(q));
    },
    'food.product_history': (a) => {
      const p =
        products().find((x) => x.id === a.id) ?? mockFail('not_found', 'Produto não encontrado.');
      const points = mockReceipts().flatMap((r) =>
        r.items
          .filter((it) => productId(it.description) === p.id)
          .map((it) => ({
            date: r.date,
            store_name: r.store_name,
            description: it.description,
            quantity: it.quantity,
            unit: it.unit,
            total: it.total,
            base_qty: null,
            base_unit: it.base_unit,
            base_price: it.base_price,
            receipt_id: r.id,
          })),
      );
      return { product: p, points, same_line: sameLine(p.name), health: productHealth(p.name) };
    },
    'food.categories': () => tree(),
    // O perfil de consumo do país (food/profile.rs), resumido.
    'food.profile': () => {
      const c = familyCountry();
      const added = c === 'US' || c === 'CA';
      return {
        country: c,
        tax: added ? 'added' : 'included',
        item_tax: c === 'BR' ? 'estimated' : added ? 'printed' : 'none',
        units: c === 'US' ? 'us' : 'metric',
        words: c === 'US' ? ['en'] : ['pt'],
        reader: c === 'BR' ? 'nfce' : null,
        official_series: c === 'BR' ? ['ipca_food_home'] : [],
        high_in: c === 'BR',
      };
    },
    'food.guess': () => ({ category_id: null, reason: 'none', product_id: null }),
    'food.purchases': (a) => purchases(a),
    'food.purchase_candidates': (a) => candidates(a),
    'food.stores': () => [
      ...new Set(
        [...mockReceipts()]
          .sort((x, y) => y.date.localeCompare(x.date))
          .map((r) => r.store_name)
          .filter((s): s is string => Boolean(s)),
      ),
    ],
    'food.purchase_save': (a) => savePurchase(a as unknown as PurchaseInput),
    'food.purchase_delete': (a) => {
      const r =
        mockReceipts().find((x) => x.id === a.id) ??
        mockFail('not_found', 'Compra não encontrada.');
      mockDeleteReceipt(r.id);
      if (r.txn_created && r.txn_id) call('transactions.delete', { ids: [r.txn_id] });
      return null;
    },
    'food.relink': () => ({ linked: 0 }),
    'food.product_update': (a) => {
      const id = String(a.id);
      if ('class' in a) productClass.set(id, (a.class as FoodClass | null) ?? null);
      if ('category_id' in a) productCategory.set(id, (a.category_id as string | null) ?? null);
      return products().find((p) => p.id === id) ?? null;
    },
    'food.product_gtin_add': (a) => {
      const id = String(a.product_id);
      const g = checkGtin(String(a.code));
      for (const [other, codes] of manualGtins)
        if (other !== id && codes.includes(g))
          mockFail('conflict', 'Esse código de barras já é de outro produto.');
      manualGtins.set(id, [...new Set([g, ...(manualGtins.get(id) ?? [])])]);
      return products().find((p) => p.id === id) ?? null;
    },
    'food.product_gtin_remove': (a) => {
      const id = String(a.product_id);
      manualGtins.set(
        id,
        (manualGtins.get(id) ?? []).filter((g) => g !== String(a.code)),
      );
      return null;
    },
    'food.product_merge': (a) => {
      merged.add(String(a.from_id));
      return products().find((p) => p.id === a.into_id) ?? null;
    },
    'food.category_create': (a) => {
      seed();
      const parent =
        typeof a.parent_id === 'string' ? cats.find((c) => c.id === a.parent_id) : null;
      const id = `pc-novo-${(seq += 1)}`;
      cats.push({
        id,
        parent_id: parent?.id ?? null,
        seed_key: null,
        name: String(a.name).trim() || mockFail('invalid', 'Dê um nome.'),
        is_food: typeof a.is_food === 'boolean' ? a.is_food : (parent?.is_food ?? true),
        class: (a.class as FoodClass | undefined) ?? null,
      });
      return tree().find((c) => c.id === id);
    },
    'food.category_update': (a) => {
      seed();
      const c =
        cats.find((x) => x.id === a.id) ?? mockFail('not_found', 'Categoria não encontrada.');
      if (typeof a.name === 'string') c.name = a.name.trim();
      if ('class' in a) c.class = (a.class as FoodClass | null) ?? null;
      if (typeof a.is_food === 'boolean') c.is_food = a.is_food;
      if ('parent_id' in a) c.parent_id = (a.parent_id as string | null) ?? null;
      return tree();
    },
    'food.category_delete': (a) => {
      seed();
      const drop = new Set([String(a.id)]);
      let grew = true;
      while (grew) {
        grew = false;
        for (const c of cats)
          if (c.parent_id && drop.has(c.parent_id) && !drop.has(c.id)) {
            drop.add(c.id);
            grew = true;
          }
      }
      cats = cats.filter((c) => !drop.has(c.id));
      return tree();
    },
    'food.set_payments': (a) => mockReceipts().find((r) => r.id === a.receipt_id) ?? null,
  }),
);
