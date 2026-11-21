/**
 * Mock de `receipts.*` (docs/receipts.md): qualquer link, PDF ou texto vira o mesmo cupom de
 * exemplo (o núcleo é que lê a SEFAZ, só no Brasil); os itens informados direto (`manual`) valem
 * como vieram, em qualquer país.
 * O imposto rateado e o resumo por segmento e membro seguem as regras do núcleo, simplificadas
 * (no mock o segmento é o grupo de imposto do item).
 */
import type { LineItem } from '@/api/generated/LineItem';
import type { PhotoReading } from '@/api/generated/PhotoReading';
import type { GroupSpend } from '@/api/generated/GroupSpend';
import type { ItemGroup } from '@/api/generated/ItemGroup';
import type { MemberSpend } from '@/api/generated/MemberSpend';
import type { ParsedReceipt } from '@/api/generated/ParsedReceipt';
import type { Receipt } from '@/api/generated/Receipt';
import type { ReceiptItem } from '@/api/generated/ReceiptItem';
import type { ReceiptPreview } from '@/api/generated/ReceiptPreview';
import type { ReceiptSummary } from '@/api/generated/ReceiptSummary';
import { todayIso } from '@/lib/format';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { familyCountry } from './country';
import { guarded } from './ledger/boot';
import { invalid, notFound } from './ledger/errors';
import { newId, rows } from './ledger/store';

const LABELS: Record<ItemGroup, string> = {
  hortifruti: 'Hortifrúti',
  carnes: 'Carnes, aves e peixes',
  laticinios: 'Leite, frios e ovos',
  mercearia: 'Mercearia (arroz, feijão, massas, óleo, café)',
  padaria: 'Padaria',
  doces_lanches: 'Doces, biscoitos e salgadinhos',
  congelados: 'Congelados e prontos',
  bebidas: 'Bebidas não alcoólicas',
  bebidas_alcoolicas: 'Bebidas alcoólicas',
  limpeza: 'Limpeza',
  higiene: 'Higiene e beleza',
  bebe: 'Bebê',
  pet: 'Pet',
  farmacia: 'Farmácia',
  casa: 'Casa e utilidades',
  vestuario: 'Vestuário',
  calcados: 'Calçados',
  eletronicos: 'Eletrônicos',
  eletrodomesticos: 'Eletrodomésticos',
  casa_decoracao: 'Casa e decoração',
  ferramentas: 'Ferramentas e construção',
  papelaria: 'Papelaria',
  brinquedos: 'Brinquedos',
  cigarros: 'Cigarros',
  outros: 'Outros',
};

const RATE: Partial<Record<ItemGroup, number>> = {
  mercearia: 17.51,
  hortifruti: 23.3,
  limpeza: 37.05,
  bebidas_alcoolicas: 39.07,
};

const GUESS: [RegExp, ItemGroup][] = [
  [/arroz|feij|macarr|caf/i, 'mercearia'],
  [/banana|tomate|alface|ma[cç]a/i, 'hortifruti'],
  [/deterg|sab|amaci/i, 'limpeza'],
  [/cerv|vinho/i, 'bebidas_alcoolicas'],
  [/camis|bermuda|cal[cç]a|blusa|vestido/i, 'vestuario'],
  [/t[eê]nis|sapato|sand[aá]lia|chinelo/i, 'calcados'],
];

const SAMPLE: ParsedReceipt = {
  access_key: '41261012345678000190650010000123451000123452',
  store_name: 'SUPERMERCADO EXEMPLO LTDA',
  store_cnpj: '12345678000190',
  uf: 'PR',
  date: todayIso(),
  issued_at: null,
  items: [
    {
      code: null,
      description: 'ARROZ TIO JOAO T1 5KG',
      quantity: '1',
      unit: 'UN',
      unit_price: 2490,
      total: 2490,
    },
    {
      code: null,
      description: 'BANANA PRATA KG',
      quantity: '1,235',
      unit: 'KG',
      unit_price: 599,
      total: 740,
    },
    {
      code: null,
      description: 'DETERG YPE NEUTRO 500ML',
      quantity: '2',
      unit: 'UN',
      unit_price: 279,
      total: 558,
    },
    {
      code: null,
      description: 'CERV BRAHMA LT 350ML',
      quantity: '6',
      unit: 'UN',
      unit_price: 349,
      total: 2094,
    },
  ],
  items_total: 5882,
  discount: 182,
  paid: 5700,
  tax_total: 1530,
  tax_federal: null,
  tax_state: null,
  tax_municipal: null,
  warnings: [],
};

interface Stored extends Receipt {
  items: ReceiptItem[];
}
let store: Stored[] = [];
onMockReset(() => {
  store = [];
});

/** Preço por kg, L ou unidade, como `food::pack` no núcleo (simplificado). */
export function baseOf(
  description: string,
  quantity: string | null,
  unit: string | null,
  total: number,
): { base_unit: string | null; base_price: number | null } {
  const q = Number((quantity ?? '1').replace(',', '.')) || 1;
  const u = (unit ?? '').toLowerCase();
  let qty = q;
  let base = 'un';
  if (u === 'kg') base = 'kg';
  else {
    const m = /(\d+(?:[.,]\d+)?)\s*(kg|g|ml|l)\b/i.exec(description);
    if (m) {
      const v = Number(m[1]!.replace(',', '.'));
      const k = m[2]!.toLowerCase();
      qty = q * (k === 'g' || k === 'ml' ? v / 1000 : v);
      base = k === 'g' || k === 'kg' ? 'kg' : 'l';
    }
  }
  return { base_unit: base, base_price: qty > 0 ? Math.round(total / qty) : null };
}

/** Os cupons gravados no mock (para a tela Consumo). */
export function mockReceipts(): readonly Receipt[] {
  return store;
}

/** Grava (ou troca, pelo id) uma compra vinda da tela Consumo (cadastrada à mão). */
export function mockSaveReceipt(r: Receipt): void {
  store = [...store.filter((x) => x.id !== r.id), r];
}

export function mockDeleteReceipt(id: string): void {
  store = store.filter((x) => x.id !== id);
}

/** Grupo do cupom pela descrição (simplificado). */
export function guessGroup(description: string): ItemGroup {
  return GUESS.find(([re]) => re.test(description))?.[1] ?? 'outros';
}

export function groupLabel(g: ItemGroup): string {
  return LABELS[g];
}

function itemsOf(p: ParsedReceipt, groups: { line: number; group: ItemGroup }[]): ReceiptItem[] {
  const items: ReceiptItem[] = p.items.map((it, i) => {
    const line = i + 1;
    const auto = GUESS.find(([re]) => re.test(it.description))?.[1] ?? 'outros';
    const group = groups.find((g) => g.line === line)?.group ?? auto;
    const rate = RATE[group] ?? 28.45;
    return {
      line,
      code: it.code,
      description: it.description,
      quantity: it.quantity,
      unit: it.unit,
      unit_price: it.unit_price,
      total: it.total,
      group,
      group_label: LABELS[group],
      group_origin: groups.some((g) => g.line === line) ? 'manual' : 'auto',
      tax_rate: rate,
      tax: Math.round((it.total * rate) / 100),
      tax_origin: 'estimated',
      gtin: it.gtin ?? null,
      product_id: null,
      category_id: null,
      category_reason: 'none',
      ...baseOf(it.description, it.quantity, it.unit, it.total),
    };
  });
  if (p.tax_total) {
    const w = items.reduce((s, i) => s + Math.max(1, i.tax), 0);
    let given = 0;
    items.forEach((it, k) => {
      const share =
        k === items.length - 1
          ? p.tax_total! - given
          : Math.round((p.tax_total! * Math.max(1, it.tax)) / w);
      given += share;
      it.tax = share;
      it.tax_origin = 'nfce';
    });
  }
  return items;
}

function parsedOf(args: Record<string, unknown>): ParsedReceipt {
  const m = args.manual as
    | { date: string; store_name?: string; items: { description: string; total: number }[] }
    | undefined;
  if (m) {
    const items = m.items.map((i) => ({
      code: null,
      description: i.description,
      quantity: null,
      unit: null,
      unit_price: null,
      total: i.total,
    }));
    const total = items.reduce((s, i) => s + i.total, 0);
    return {
      ...SAMPLE,
      access_key: null,
      store_name: m.store_name ?? null,
      date: m.date,
      items,
      items_total: total,
      discount: 0,
      paid: total,
      tax_total: null,
    };
  }
  if (!args.url && !args.text && !args.files)
    invalid('envie o link do QR do cupom, o PDF ou o texto da página da NFC-e');
  return SAMPLE;
}

function preview(args: Record<string, unknown>): ReceiptPreview {
  const parsed = parsedOf(args);
  const groups = (args.groups as { line: number; group: ItemGroup }[] | undefined) ?? [];
  const items = itemsOf(parsed, groups);
  const printed = (parsed.tax_total ?? 0) > 0;
  const txn = rows('txn').find(
    (t) =>
      t.kind === 'expense' && t.amount === -parsed.paid && !store.some((r) => r.txn_id === t.id),
  );
  if (!args.manual && familyCountry() !== 'BR') invalid('a leitura de cupons fiscais é do Brasil');
  if (typeof args.url === 'string' && args.url.trim()) {
    const host = /^https?:\/\/([^/?#]+)/i.exec(args.url.trim())?.[1]?.toLowerCase() ?? '';
    if (!host.endsWith('.gov.br'))
      invalid(
        `Esse link não é de uma nota fiscal do Brasil (${host}). O app lê as notas do Brasil (SEFAZ) e da Argentina (QR da AFIP); para as de outros países, use a foto do cupom ou cadastre a compra à mão.`,
      );
  }
  return {
    origin: args.manual ? 'agent' : args.url ? 'nfce' : args.files ? 'pdf' : 'text',
    parsed,
    items,
    tax: printed ? parsed.tax_total! : items.reduce((s, i) => s + i.tax, 0),
    tax_printed: printed,
    duplicate: store.find((r) => r.access_key && r.access_key === parsed.access_key)?.id ?? null,
    txn: txn
      ? {
          txn_id: txn.id,
          date: txn.date,
          amount: txn.amount,
          description: txn.description,
          account_name: 'Conta',
          member_id: txn.member_id ?? null,
        }
      : null,
    warnings: parsed.warnings,
  };
}

const memberName = (id: string | null) =>
  id ? (rows('member').find((m) => m.id === id)?.name ?? null) : null;

function summary(args: Record<string, unknown>): ReceiptSummary {
  const from = typeof args.from === 'string' ? args.from : '0000-01-01';
  const to = typeof args.to === 'string' ? args.to : '9999-12-31';
  const list = store.filter((r) => r.date >= from && r.date <= to);
  const add = (m: Map<ItemGroup, GroupSpend>, it: ReceiptItem) => {
    const g = m.get(it.group) ?? {
      segment_id: it.group,
      label: LABELS[it.group],
      spent: 0,
      tax: 0,
      items: 0,
    };
    g.spent += it.total;
    g.tax += it.tax;
    g.items += 1;
    m.set(it.group, g);
  };
  const groups = new Map<ItemGroup, GroupSpend>();
  const members = new Map<string, { id: string | null; m: Map<ItemGroup, GroupSpend> }>();
  const top = new Map<
    string,
    { description: string; segment: string; spent: number; times: number }
  >();
  for (const r of list)
    for (const it of r.items) {
      add(groups, it);
      const k = r.member_id ?? '';
      const e = members.get(k) ?? { id: r.member_id, m: new Map<ItemGroup, GroupSpend>() };
      add(e.m, it);
      members.set(k, e);
      const t = top.get(it.description) ?? {
        description: it.description,
        segment: LABELS[it.group],
        spent: 0,
        times: 0,
      };
      t.spent += it.total;
      t.times += 1;
      top.set(it.description, t);
    }
  const sorted = (m: Map<ItemGroup, GroupSpend>) =>
    [...m.values()].sort((a, b) => b.spent - a.spent);
  const memberRows: MemberSpend[] = [...members.values()].map(({ id, m }) => {
    const g = sorted(m);
    return {
      member_id: id,
      member_name: memberName(id) ?? 'Família',
      spent: g.reduce((s, x) => s + x.spent, 0),
      tax: g.reduce((s, x) => s + x.tax, 0),
      groups: g,
    };
  });
  const all = sorted(groups);
  return {
    from,
    to,
    receipts: list.length,
    spent: list.reduce((s, r) => s + r.paid, 0),
    tax: all.reduce((s, g) => s + g.tax, 0),
    tax_printed: list.filter((r) => r.tax_printed).reduce((s, r) => s + r.tax, 0),
    groups: all,
    members: memberRows.sort((a, b) => b.spent - a.spent),
    top_items: [...top.values()].sort((a, b) => b.spent - a.spent).slice(0, 15),
  };
}

/** Itens colados como texto: uma linha por item, com o valor no fim (o núcleo usa o leitor do OCR). */
function readText(text: string): PhotoReading {
  const items: LineItem[] = text
    .split('\n')
    .map((l) => /^(.*\S)\s+(\d+)[.,](\d{2})$/.exec(l.trim()))
    .filter((m): m is RegExpExecArray => m !== null && !/^total\b/i.test(m[1]!))
    .map((m) => ({
      description: m[1]!,
      quantity: null,
      unit_price: null,
      total: Number(m[2]) * 100 + Number(m[3]),
      line_checked: null,
    }));
  if (!items.length) invalid('Nenhum item foi lido.');
  const sum = items.reduce((s, i) => s + i.total, 0);
  return {
    engine: 'text',
    tier: 'unbalanced',
    reading: {
      items,
      discount: 0,
      total: null,
      items_sum: sum,
      tier: 'unbalanced',
      printed_count: null,
      rows: [],
    },
    receipt: {
      date: todayIso(),
      items: items.map((i) => ({ description: i.description, total: i.total })),
    },
    ai_available: false,
    warnings: [],
  };
}

/** Cupom pela foto (docs/ocr-receipts.md), inventado: o OCR deixa uma linha de fora; a IA acerta. */
function readPhoto(args: Record<string, unknown>): PhotoReading {
  const ai = args.use_ai === true;
  const items: LineItem[] = [
    {
      description: 'PAO FRANCES KG',
      quantity: '0.512',
      unit_price: 1599,
      total: 819,
      line_checked: true,
    },
    {
      description: 'LEITE INTEGRAL 1L',
      quantity: '2',
      unit_price: 549,
      total: 1098,
      line_checked: true,
    },
    ...(ai
      ? [
          {
            description: 'CAFE TORRADO 500G',
            quantity: '1',
            unit_price: 1890,
            total: 1890,
            line_checked: null,
          },
        ]
      : []),
  ];
  const sum = items.reduce((s, i) => s + i.total, 0);
  const tier = ai ? 'balanced' : 'unbalanced';
  return {
    engine: ai ? 'ai' : 'paddle',
    tier,
    reading: {
      items,
      discount: 0,
      total: 3807,
      items_sum: sum,
      tier,
      printed_count: null,
      rows: [],
    },
    receipt: {
      date: '2026-10-21',
      store_name: 'PADARIA EXEMPLO',
      items: items.map((i) => ({
        description: i.description,
        quantity: i.quantity ?? undefined,
        unit_price: i.unit_price ?? undefined,
        total: i.total,
      })),
      paid: 3807,
      from_photo: true,
    },
    ai_available: true,
    warnings: ai
      ? []
      : ['A soma dos itens não bate com o total. Confira os itens antes de gravar.'],
  };
}

registerMockHandlers(
  guarded({
    'receipts.read_photo': (args) => readPhoto(args),
    'receipts.read_text': (args) => readText(typeof args.text === 'string' ? args.text : ''),
    'receipts.preview': (args) => preview(args),
    // Nota da Argentina pelo QR da AFIP (receipts/foreign.rs): JSON em base64 no `p=`.
    'receipts.foreign_qr': (args) => foreignQr(typeof args.url === 'string' ? args.url : ''),
    'receipts.import': (args): Receipt => {
      const p = preview(args);
      if (p.duplicate && args.replace !== true) invalid('este cupom já foi importado');
      if (p.duplicate) store = store.filter((r) => r.id !== p.duplicate);
      const member =
        typeof args.member_id === 'string' ? args.member_id : (p.txn?.member_id ?? null);
      // A forma de pagamento da Compra: ligar ao escolhido, ou ficar sem lançamento.
      const pay = args.payment as { mode: string; txn_id?: string } | undefined;
      const txnId = pay ? (pay.mode === 'link' ? (pay.txn_id ?? null) : null) : null;
      const r: Stored = {
        id: newId(),
        access_key: p.parsed.access_key,
        date: p.parsed.date ?? todayIso(),
        issued_at: p.parsed.issued_at,
        store_name: p.parsed.store_name,
        store_cnpj: p.parsed.store_cnpj,
        uf: p.parsed.uf,
        items_total: p.parsed.items_total,
        discount: p.parsed.discount,
        paid: p.parsed.paid,
        tax: p.tax,
        tax_printed: p.tax_printed,
        origin: p.origin,
        txn_id: pay ? txnId : (p.txn?.txn_id ?? null),
        txn_created: false,
        tax_added: 0,
        member_id: member,
        member_name: memberName(member),
        item_count: p.items.length,
        items: p.items,
      };
      // A categoria escolhida na revisão fica no item.
      const chosen = (args.categories as { line: number; category_id: string }[] | undefined) ?? [];
      r.items = r.items.map((it) => ({
        ...it,
        category_id: chosen.find((c) => c.line === it.line)?.category_id ?? null,
      }));
      store.push(r);
      return r;
    },
    'receipts.list': (args) =>
      store
        .filter(
          (r) =>
            r.date >= (typeof args.from === 'string' ? args.from : '0000') &&
            r.date <= (typeof args.to === 'string' ? args.to : '9999'),
        )
        .map((r) => ({ ...r, items: [] })),
    'receipts.get': (args) =>
      store.find((r) => r.id === args.id) ?? notFound(`cupom ${String(args.id)}`),
    'receipts.summary': (args) => summary(args),
    'receipts.update': (args) => {
      const r = store.find((x) => x.id === args.id) ?? notFound('cupom');
      for (const c of (args.categories as { line: number; category_id: string }[] | undefined) ??
        []) {
        const it = r.items.find((i) => i.line === c.line) ?? notFound('item');
        it.category_id = c.category_id;
      }
      if (typeof args.member_id === 'string') {
        r.member_id = args.member_id;
        r.member_name = memberName(args.member_id);
      }
      const pay = args.payment as { mode: string; txn_id?: string } | undefined;
      if (pay) r.txn_id = pay.mode === 'link' ? (pay.txn_id ?? null) : null;
      return r;
    },
    'receipts.delete': (args) => {
      if (!store.some((r) => r.id === args.id)) notFound(`cupom ${String(args.id)}`);
      store = store.filter((r) => r.id !== args.id);
      return null;
    },
  }),
);

/** Lê o QR da AFIP como o núcleo (sem cotação do peso no mock). */
function foreignQr(url: string) {
  const host = /^https?:\/\/([^/?#]+)/i.exec(url.trim())?.[1]?.toLowerCase() ?? '';
  if (!/(^|\.)(afip|arca)\.gob\.ar$/.test(host))
    invalid(
      `Esse link não é de uma nota fiscal do Brasil (${host}). O app lê as notas do Brasil (SEFAZ) e da Argentina (QR da AFIP); para as de outros países, use a foto do cupom ou cadastre a compra à mão.`,
    );
  const bad = () =>
    invalid(
      'Não deu para ler o QR da AFIP: o link não traz os dados da nota. Confira se o link está inteiro ou use a foto do cupom.',
    );
  const p = /[?&]p=([^&#]+)/.exec(url)?.[1];
  if (!p) return bad();
  let j: Record<string, string | number | null | undefined>;
  try {
    j = JSON.parse(atob(decodeURIComponent(p).replace(/-/g, '+').replace(/_/g, '/'))) as Record<
      string,
      string | number | null | undefined
    >;
  } catch {
    return bad();
  }
  const num = (v: string | number | null | undefined) => Number(String(v ?? '').replace(',', '.'));
  const cuit = String(Math.trunc(num(j.cuit))).padStart(11, '0');
  const docs: Record<number, string> = {
    1: 'Factura A',
    6: 'Factura B',
    11: 'Factura C',
    83: 'Tique',
  };
  const tipo = num(j.tipoCmp);
  const moeda = String(j.moneda ?? 'PES').toUpperCase();
  return {
    country: 'AR',
    authority: 'AFIP',
    date: String(j.fecha ?? '').slice(0, 10),
    issuer_tax_id: `${cuit.slice(0, 2)}-${cuit.slice(2, 10)}-${cuit.slice(10)}`,
    document: docs[tipo] ?? `Comprobante ${tipo}`,
    number: `${String(num(j.ptoVta)).padStart(4, '0')}-${String(num(j.nroCmp)).padStart(8, '0')}`,
    credit: [3, 8, 13].includes(tipo),
    total: Math.round(num(j.importe) * 100),
    currency: moeda === 'PES' ? 'ARS' : moeda === 'DOL' ? 'USD' : moeda,
    converted: null,
    authorization: j.codAut != null ? String(j.codAut) : null,
    tax_id_label: 'CUIT',
    url: url.trim(),
  };
}
