/**
 * A Compra (docs/consumption.md): a única tela de revisão, em três estados. **Digitada**: data, loja,
 * membro, os itens em linhas (descrição com sugestão dos produtos já comprados, quantidade,
 * unidade, preço unitário, total e categoria) e como foi paga: criar a despesa junto, ligar a um
 * lançamento ou deixar sem lançamento. **Leitura a conferir**: o cupom lido da foto
 * (docs/ocr-receipts.md), com o quanto a leitura conferiu e a ajuda da IA. **Documento fiscal**:
 * a NFC-e (link, PDF, XML), com o que o documento diz travado (itens, valores, loja, data) e a
 * categoria de cada item, o membro e o lançamento para escolher; vale para o cupom novo e para
 * o já gravado.
 */
import { useEffect, useId, useRef, useState, type KeyboardEvent } from 'react';

import { call } from '@/api/client';
import type { FoodGuess } from '@/api/generated/FoodGuess';
import type { FoodProduct } from '@/api/generated/FoodProduct';
import type { ForeignReceipt } from '@/api/generated/ForeignReceipt';
import type { PhotoReading } from '@/api/generated/PhotoReading';
import type { Purchase } from '@/api/generated/Purchase';
import type { PurchaseInput } from '@/api/generated/PurchaseInput';
import type { PurchasePayment } from '@/api/generated/PurchasePayment';
import type { Receipt } from '@/api/generated/Receipt';
import type { ReceiptArgs } from '@/api/generated/ReceiptArgs';
import type { ReceiptPreview } from '@/api/generated/ReceiptPreview';
import { useAccounts } from '@/api/hooks/accounts';
import {
  useFoodProducts,
  useProductCategories,
  usePurchaseCandidates,
  useSavePurchase,
  useStores,
} from '@/api/hooks/food';
import { useMembers } from '@/api/hooks/household';
import { useImportReceipt, useUpdateReceipt } from '@/api/hooks/receipts';
import { useTxn } from '@/api/hooks/transactions';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Icon,
  IconButton,
  Input,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { canReadBarcode, readBarcode } from '@/features/receipts/barcode';
import { PhotoPane } from '@/features/receipts/PhotoPane';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import { FileInput, InlineError } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';

import { useConsumptionProfile } from './units';
import { t } from '@/i18n';
import { formatDate, formatLocale, formatMoney, todayIso } from '@/lib/format';

const UNITS = [
  { value: 'un', label: 'un' },
  { value: 'kg', label: 'kg' },
  { value: 'g', label: 'g' },
  { value: 'l', label: 'L' },
  { value: 'ml', label: 'mL' },
] as const;
/** Onde se usa libra e galão (EUA), além das métricas. */
const US_UNITS = [
  { value: 'lb', label: 'lb' },
  { value: 'oz', label: 'oz' },
  { value: 'gal', label: 'gal' },
  { value: 'floz', label: 'fl oz' },
] as const;
const ALL_UNITS: readonly { value: string; label: string }[] = [...UNITS, ...US_UNITS];
const AUTO = '__auto__';

interface Line {
  key: number;
  description: string;
  quantity: string;
  unit: string;
  unitPrice: number | null;
  total: number | null;
  /** O total foi digitado (não segue mais quantidade × preço). */
  totalTyped: boolean;
  /** O preço unitário foi digitado (senão ele sai do total e da quantidade). */
  priceTyped?: boolean;
  productId: string | null;
  categoryId: string | null;
  /** Código de barras (opcional; a linha mostra o campo quando a pessoa abre). */
  gtin: string;
  showGtin: boolean;
  /** A categoria com que a linha abriu (a do produto ou a prevista): só a trocada é enviada. */
  initialCategory?: string | null;
  /** A categoria que o app vai usar se a pessoa não escolher, e de onde ela veio. */
  guess?: FoodGuess | undefined;
  /** Marca de imposto lida do cupom, onde o imposto é somado no fim (EUA, Canadá). */
  taxable?: boolean | undefined;
}

/** Sem acentos e em minúsculas, para comparar nomes. */
const fold = (v: string) =>
  v
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .trim()
    .toLowerCase();

/** "0,734 kg", "500g", "2 L": a quantidade e a unidade digitadas juntas. */
const QTY_UNIT = /^([\d.,]+)\s*(kg|g|l|ml|un|lb|oz|gal)$/i;

const REASON_KEY = {
  known: 'alimentacao-palpite-conhecido',
  open_data: 'alimentacao-palpite-dados-abertos',
  words: 'alimentacao-palpite-palavras',
  none: 'alimentacao-palpite-nenhum',
} as const;

/** O lançamento que a compra vai detalhar (aberta por "Detalhar itens" em Lançamentos). */
export interface TxnSeed {
  id: string;
  date: string;
  description: string;
  amount: number;
  member_id: string | null;
  account_name: string;
}

type Mode = 'create' | 'link' | 'none';
const PAYMENT_KEY = {
  create: 'alimentacao-pagamento-criar',
  link: 'alimentacao-pagamento-ligar',
  none: 'alimentacao-pagamento-sem',
} as const;

let seq = 0;
const blank = (): Line => ({
  key: (seq += 1),
  description: '',
  quantity: '1',
  unit: 'un',
  unitPrice: null,
  total: null,
  totalTyped: false,
  productId: null,
  categoryId: null,
  gtin: '',
  showGtin: false,
});

const qtyOf = (q: string) => {
  const v = Number(q.replace(',', '.'));
  return Number.isFinite(v) && v > 0 ? v : null;
};

/** Casas da quantidade pela unidade: kg e L com três (1,000 kg), g, mL e un inteiros. */
const unitDecimals = (u: string) => (u === 'kg' || u === 'l' ? 3 : u === 'lb' ? 2 : 0);

/**
 * A quantidade escrita do jeito da unidade ("1" em kg vira "1,000"). Sem número, ou quebrada
 * numa unidade inteira, fica como está (não arredonda o que a pessoa digitou).
 */
function qtyText(q: string, unit: string): string {
  const v = qtyOf(q);
  if (v == null || (!unitDecimals(unit) && !Number.isInteger(v))) return q;
  const text = v.toFixed(unitDecimals(unit));
  return formatLocale() === 'en' ? text : text.replace('.', ',');
}

/** O rótulo da unidade (kg, L, mL…). */
const unitLabelOf = (u: string) => ALL_UNITS.find((x) => x.value === u)?.label ?? u;

const unitOf = (u: string | null | undefined) => {
  const k = (u ?? '').trim().toLowerCase();
  return ALL_UNITS.some((x) => x.value === k) ? k : 'un';
};

/** A linha com o total recalculado (quantidade × preço) enquanto ele não foi digitado. */
function withTotal(l: Line): Line {
  const q = qtyOf(l.quantity);
  // Total digitado e preço não: o preço unitário sai do total e da quantidade.
  if (l.totalTyped && !l.priceTyped && l.total != null && q && l.productId == null)
    return { ...l, unitPrice: Math.round(l.total / q) };
  if (l.totalTyped || l.unitPrice == null) return l;
  return { ...l, total: q == null ? null : Math.round(q * l.unitPrice) };
}

function linesOf(r: Receipt, products: FoodProduct[]): Line[] {
  return r.items.map((it) => {
    const p = products.find((x) => x.id === it.product_id);
    const category = it.category_id ?? p?.category_id ?? null;
    return {
      key: (seq += 1),
      description: it.description,
      quantity: qtyText(it.quantity ?? '1', unitOf(it.unit)),
      unit: unitOf(it.unit),
      unitPrice: it.unit_price,
      total: it.total,
      totalTyped: true,
      productId: it.product_id ?? null,
      categoryId: category,
      initialCategory: category,
      gtin: it.gtin ?? '',
      showGtin: Boolean(it.gtin),
    };
  });
}

/** Cupom fiscal lido (NFC-e pelo link, PDF ou XML), para revisar antes de gravar. */
export interface FiscalDraft {
  args: ReceiptArgs;
  preview: ReceiptPreview;
}

function fiscalLines(d: FiscalDraft): Line[] {
  return d.preview.items.map((it) => ({
    key: (seq += 1),
    description: it.description,
    quantity: it.quantity ?? '1',
    unit: (it.unit ?? 'un').toLowerCase(),
    unitPrice: it.unit_price,
    total: it.total,
    totalTyped: true,
    productId: null,
    categoryId: it.category_id ?? null,
    initialCategory: it.category_id ?? null,
    gtin: it.gtin ?? '',
    showGtin: false,
  }));
}

/** Os itens de uma compra anterior, com o último preço de cada produto, para repetir. */
function repeatLines(r: Receipt, products: FoodProduct[]): Line[] {
  return r.items.map((it) => {
    const p = products.find((x) => x.id === it.product_id);
    const unit = unitOf(it.unit);
    const price = p?.last_unit_price ?? it.unit_price;
    return withTotal({
      key: (seq += 1),
      description: it.description,
      quantity: qtyText(it.quantity ?? '1', unit),
      unit,
      unitPrice: price,
      total: price == null ? it.total : null,
      totalTyped: price == null,
      productId: it.product_id ?? null,
      categoryId: null,
      gtin: '',
      showGtin: false,
    });
  });
}

/** Cupom lido da foto, para revisar antes de gravar. */
export interface PhotoDraft {
  reading: PhotoReading;
  /** A foto (URL local da memória), mostrada ao lado dos campos para conferir e digitar. */
  imageUrl?: string;
  /** Pedir à IA (quando a conta não fechou e há motor que lê imagens). */
  onAskAi?: () => void;
  askingAi?: boolean;
}

function draftLines(d: PhotoDraft): Line[] {
  return d.reading.receipt.items.map((it) => ({
    key: (seq += 1),
    description: it.description,
    // Quantidade quebrada no cupom é peso.
    quantity: qtyText(it.quantity ?? '1', it.quantity && !/^\d+$/.test(it.quantity) ? 'kg' : 'un'),
    unit: it.quantity && !/^\d+$/.test(it.quantity) ? 'kg' : 'un',
    unitPrice: it.unit_price ?? null,
    total: it.total,
    totalTyped: true,
    productId: null,
    categoryId: null,
    gtin: '',
    showGtin: false,
    taxable: it.taxable,
  }));
}

const TIER_KEY = {
  checked: 'cupons-foto-faixa-conferido',
  balanced: 'cupons-foto-faixa-fechou',
  unbalanced: 'cupons-foto-faixa-nao-fechou',
} as const;
const ENGINE_KEY = {
  vision: 'cupons-foto-motor-vision',
  paddle: 'cupons-foto-motor-paddle',
  ai: 'cupons-foto-motor-ia',
  text: 'cupons-foto-motor-texto',
} as const;

/** Quanto a leitura da foto conferiu, a diferença para o total do cupom e a ajuda da IA. */
function DraftNotice({ draft, paid }: { draft: PhotoDraft; paid: number }) {
  const r = draft.reading;
  const printed = r.reading.total;
  const diff = printed == null ? 0 : printed - paid;
  return (
    <section
      aria-label={t('cupons-foto-leitura')}
      className="flex flex-col gap-1.5 rounded-card border border-line px-3 py-2 text-[12.5px]"
    >
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
        <p className={cx('min-w-0 flex-1', r.tier === 'unbalanced' ? 'text-negative' : 'text-fg')}>
          {t(TIER_KEY[r.tier])} <span className="text-dim">{t(ENGINE_KEY[r.engine])}</span>
          {printed != null ? (
            <span className="text-dim">
              {' '}
              {withSlots(t('cupons-foto-total-impresso', { valor: slot(0) }), [
                <Money key="t" cents={printed} />,
              ])}
              {diff !== 0 ? (
                <>
                  {' '}
                  {withSlots(
                    t(diff > 0 ? 'cupons-foto-faltam' : 'cupons-foto-sobram', { valor: slot(0) }),
                    [<Money key="d" cents={Math.abs(diff)} />],
                  )}
                </>
              ) : null}
            </span>
          ) : null}
        </p>
        {r.tier === 'unbalanced' && r.ai_available && draft.onAskAi ? (
          <Button
            size="sm"
            variant="secondary"
            loading={draft.askingAi}
            onClick={draft.onAskAi}
            title={t('cupons-foto-pedir-ia-dica')}
          >
            {t('cupons-foto-pedir-ia')}
          </Button>
        ) : null}
      </div>
      {r.warnings.length ? <p className="text-dim">{r.warnings.join(' ')}</p> : null}
    </section>
  );
}

/** Uma linha só com o comprovante da nota de fora (o QR não traz os itens). */
function foreignLines(f: ForeignReceipt): Line[] {
  return [
    {
      ...blank(),
      description: `${f.document} ${f.number}`,
      total: f.converted ? Math.abs(f.converted.amount) : null,
      totalTyped: true,
    },
  ];
}

/** Nota de outro país lida pelo QR: o valor na moeda da nota, o convertido e o que falta. */
function ForeignNotice({ f }: { f: ForeignReceipt }) {
  return (
    <div
      role="note"
      className="flex flex-col gap-1 rounded-card border border-accent/40 bg-accent-soft/40 px-3 py-2 text-[13px]"
    >
      <span className="text-fg">
        {t('alimentacao-nota-exterior', {
          pais: f.country,
          orgao: f.authority,
          documento: `${f.document} ${f.number}`,
          rotulo: f.tax_id_label || 'CUIT',
          cuit: f.issuer_tax_id,
        })}
      </span>
      <span className="text-dim">
        {t('alimentacao-nota-exterior-valor')} <Money cents={f.total} currency={f.currency} />
        {f.converted ? (
          <>
            {' · '}
            {t('alimentacao-nota-exterior-convertido', {
              data: f.converted.rate_date ? formatDate(f.converted.rate_date) : '—',
            })}{' '}
            <Money cents={f.converted.amount} currency={f.converted.currency} />
          </>
        ) : (
          <> · {t('alimentacao-nota-exterior-sem-cotacao')}</>
        )}
      </span>
      <span className="text-dim">
        {t('alimentacao-nota-exterior-itens', { orgao: f.authority })}
      </span>
    </div>
  );
}

export function PurchaseDialog({
  purchase,
  draft,
  foreign,
  fiscal,
  txn,
  repeat,
  onClose,
  onSaved,
}: {
  /** A compra a editar (com os itens), ou `null` para uma nova. */
  purchase: Receipt | null;
  /** Cupom fiscal lido (o que o documento diz fica travado). */
  fiscal?: FiscalDraft;
  /** O lançamento a detalhar (nova compra já ligada a ele). */
  txn?: TxnSeed;
  /** Compra anterior a repetir (nova compra com os mesmos itens, hoje). */
  repeat?: Receipt;
  /** Depois de gravar (além de fechar). */
  onSaved?: () => void;
  /** Cupom lido da foto (nova compra já preenchida). */
  draft?: PhotoDraft;
  /** Nota de outro país lida pelo QR (nova compra com data, loja e valor). */
  foreign?: ForeignReceipt;
  onClose: () => void;
}) {
  const listId = useId();
  const products = useFoodProducts({}).data ?? [];
  const cats = useProductCategories().data ?? [];
  const stores = useStores().data ?? [];
  const members = useMembers().data ?? [];
  const accounts = (useAccounts().data ?? []).filter((a) => !a.archived);
  const savePurchase = useSavePurchase();
  const importReceipt = useImportReceipt();
  const updateReceipt = useUpdateReceipt();
  /** Documento fiscal (novo ou já gravado): itens, valores, loja e data não se editam. */
  const locked = Boolean(fiscal) || Boolean(purchase && purchase.origin !== 'manual');
  const save = fiscal ? importReceipt : locked ? updateReceipt : savePurchase;

  const fromPhoto = draft?.reading.receipt;
  const [date, setDate] = useState<string | null>(
    purchase?.date ??
      fiscal?.preview.parsed.date ??
      fromPhoto?.date ??
      foreign?.date ??
      txn?.date ??
      todayIso(),
  );
  const [store, setStore] = useState(
    purchase?.store_name ??
      fiscal?.preview.parsed.store_name ??
      fromPhoto?.store_name ??
      (foreign
        ? `${foreign.tax_id_label || 'CUIT'} ${foreign.issuer_tax_id}`
        : (repeat?.store_name ?? txn?.description ?? '')),
  );
  const [member, setMember] = useState(
    purchase?.member_id ?? fiscal?.preview.txn?.member_id ?? txn?.member_id ?? AUTO,
  );
  const [lines, setLines] = useState<Line[]>(() =>
    purchase
      ? linesOf(purchase, products)
      : fiscal
        ? fiscalLines(fiscal)
        : repeat
          ? repeatLines(repeat, products)
          : draft && draft.reading.receipt.items.length
            ? draftLines(draft)
            : foreign
              ? foreignLines(foreign)
              : [blank()],
  );
  const qtyHeader =
    new Set(lines.map((l) => l.unit)).size === 1
      ? unitLabelOf(lines[0]!.unit)
      : t('alimentacao-item-qtd');
  const [discount, setDiscount] = useState<number | null>(
    purchase?.discount || fiscal?.preview.parsed.discount || fromPhoto?.discount || null,
  );
  // Onde o imposto é somado no fim do cupom (EUA, Canadá): pago = itens − desconto + imposto.
  const profile = useConsumptionProfile().data;
  const taxAdded = profile?.tax === 'added';
  const [tax, setTax] = useState<number | null>(
    purchase?.tax_added || fromPhoto?.tax_total || null,
  );
  const units = profile?.units === 'us' ? ALL_UNITS : UNITS;
  const [modeState, setModeState] = useState<Mode>(
    purchase
      ? purchase.txn_id
        ? 'link'
        : 'none'
      : (fiscal && fiscal.preview.txn) || txn
        ? 'link'
        : 'create',
  );
  /** A pessoa já escolheu a forma de pagamento (o app deixa de sugerir). */
  const [modeTouched, setModeTouched] = useState(false);
  const [account, setAccount] = useState('');
  const [txnCategory, setTxnCategory] = useState<string | null>(null);
  const [linkedState, setLinked] = useState<string | null>(
    purchase?.txn_id ?? fiscal?.preview.txn?.txn_id ?? txn?.id ?? null,
  );

  const itemsTotal = lines.reduce((s, l) => s + (l.description.trim() ? (l.total ?? 0) : 0), 0);
  // No documento fiscal o pago é o da nota (os descontos por item já estão nele).
  const paid = fiscal
    ? fiscal.preview.parsed.paid
    : locked && purchase
      ? purchase.paid
      : Math.max(0, itemsTotal - (discount ?? 0)) + (taxAdded ? (tax ?? 0) : 0);
  // Compra nova, digitada: se já existe a despesa com o mesmo valor (do extrato ou do cartão),
  // o app sugere ligar a ela em vez de criar outra.
  const suggests = !purchase && !fiscal && !txn && !modeTouched;
  const candidates = usePurchaseCandidates(
    { date: date ?? todayIso(), amount: paid, ...(purchase ? { purchase_id: purchase.id } : {}) },
    (modeState === 'link' || suggests) && paid > 0 && Boolean(date),
  );
  const found = candidates.data ?? [];
  const exact = suggests ? found.find((c) => -c.amount === paid) : undefined;
  const mode: Mode = exact ? 'link' : modeState;
  const linked = exact ? (linkedState ?? exact.txn_id) : linkedState;
  const setMode = (m: Mode) => {
    setModeState(m);
    setModeTouched(true);
  };
  // O lançamento que já paga esta compra (ou o que ela veio detalhar) continua na lista.
  const seeded =
    txn && !found.some((c) => c.txn_id === txn.id) ? [{ ...txn, txn_id: txn.id }, ...found] : found;
  const options =
    purchase?.txn_id && !seeded.some((c) => c.txn_id === purchase.txn_id)
      ? [{ txn_id: purchase.txn_id, current: true as const }, ...seeded]
      : seeded;
  // Quanto do lançamento escolhido fica sem detalhe (a compra pode ser só parte dele).
  const linkedTxn = useTxn(mode === 'link' ? linked : null).data;
  const rest = mode === 'link' && linkedTxn ? -linkedTxn.amount - paid : null;
  const accountId = account || accounts[0]?.id || '';

  const set = (key: number, patch: Partial<Line>) =>
    setLines((ls) => ls.map((l) => (l.key === key ? withTotal({ ...l, ...patch }) : l)));
  const describe = (key: number, typed: string) => {
    // O nome igual (sem contar acentos) ou o código de barras de um produto já comprado.
    const code = /^\d{8,14}$/.test(typed.trim()) ? typed.trim().replace(/^0+/, '') : null;
    const p = products.find((x) =>
      code
        ? x.gtins.some((g) => g.replace(/^0+/, '') === code)
        : fold(x.name) === fold(typed) && fold(typed) !== '',
    );
    const description = code && p ? p.name : typed;
    setLines((ls) =>
      ls.map((l) => {
        if (l.key !== key) return l;
        if (!p) return { ...l, description, productId: null, guess: undefined };
        return withTotal({
          ...l,
          description,
          productId: p.id,
          categoryId: l.categoryId ?? p.category_id,
          unit: p.last_unit ? unitOf(p.last_unit) : l.unit,
          unitPrice: l.unitPrice ?? p.last_unit_price,
        });
      }),
    );
  };

  /** A categoria prevista de uma linha (as mesmas regras da gravação), para a pessoa ver. */
  const guessLine = (l: Line) => {
    if (locked || !l.description.trim() || l.categoryId || l.guess) return;
    void call<FoodGuess>('food.guess', {
      description: l.description.trim(),
      ...(store.trim() ? { store_name: store.trim() } : {}),
      ...(l.gtin.trim() ? { gtin: l.gtin.trim() } : {}),
    })
      .then((guess) =>
        setLines((ls) => ls.map((x) => (x.key === l.key && !x.guess ? { ...x, guess } : x))),
      )
      .catch(() => undefined);
  };
  // As linhas que já vêm preenchidas (foto, texto colado, compra repetida).
  useEffect(() => {
    lines.forEach(guessLine);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  /** Começar pela última compra nesta loja (feira, padaria, a compra do mês). */
  const [loadingLast, setLoadingLast] = useState(false);
  const knownStore =
    !purchase &&
    !fiscal &&
    !draft &&
    !repeat &&
    lines.every((l) => !l.description.trim()) &&
    stores.some((x) => fold(x) === fold(store) && fold(store) !== '');
  const startFromLast = () => {
    setLoadingLast(true);
    void call<Purchase[]>('food.purchases', { query: store.trim() })
      .then((list) => {
        const last = list.find((x) => fold(x.receipt.store_name ?? '') === fold(store));
        return last ? call<Receipt>('receipts.get', { id: last.receipt.id }) : null;
      })
      .then((r) => {
        if (!r) return;
        const next = repeatLines(r, products);
        setLines(next);
        next.forEach(guessLine);
      })
      .catch(() => undefined)
      .finally(() => setLoadingLast(false));
  };

  /** Enter passa para o próximo campo; no último campo da última linha, cria outra. */
  const itemsRef = useRef<HTMLElement>(null);
  const onItemsKey = (e: KeyboardEvent<HTMLElement>) => {
    if (e.key !== 'Enter' || locked || !(e.target instanceof HTMLInputElement)) return;
    e.preventDefault();
    const inputs = [
      ...(itemsRef.current?.querySelectorAll<HTMLInputElement>('input:not([disabled])') ?? []),
    ];
    const at = inputs.indexOf(e.target);
    const next = inputs[at + 1];
    if (next) {
      next.focus();
      next.select();
      return;
    }
    e.target.blur();
    setLines((ls) => [...ls, blank()]);
    requestAnimationFrame(() => {
      const all = itemsRef.current?.querySelectorAll<HTMLInputElement>('input:not([disabled])');
      all?.[inputs.length]?.focus();
    });
  };

  const validPayment =
    (mode !== 'create' || Boolean(accountId)) &&
    (mode !== 'link' || Boolean(linked)) &&
    (rest == null || rest >= 0);
  const valid =
    validPayment &&
    (locked ||
      (Boolean(date) &&
        lines.some((l) => l.description.trim() && (l.total ?? 0) > 0) &&
        lines.every((l) => !l.description.trim() || ((l.total ?? 0) > 0 && qtyOf(l.quantity))) &&
        (discount ?? 0) < itemsTotal));

  const payment: PurchasePayment =
    mode === 'create'
      ? {
          mode: 'create',
          account_id: accountId,
          ...(txnCategory ? { category_id: txnCategory } : {}),
        }
      : mode === 'link' && linked
        ? { mode: 'link', txn_id: linked }
        : { mode: 'none' };
  const done = (edited: boolean) => ({
    onSuccess: () => {
      toast({
        tone: 'success' as const,
        title: edited ? t('alimentacao-compra-editada') : t('alimentacao-compra-gravada'),
      });
      onClose();
      onSaved?.();
    },
  });
  /** As categorias que a pessoa trocou (as outras ficam com a prevista ou a do produto). */
  const chosen = () =>
    lines.flatMap((l, i) =>
      l.categoryId && l.categoryId !== l.initialCategory
        ? [{ line: i + 1, category_id: l.categoryId }]
        : [],
    );

  const submit = () => {
    if (fiscal) {
      importReceipt.mutate(
        {
          ...fiscal.args,
          categories: chosen(),
          payment,
          ...(member !== AUTO ? { member_id: member } : {}),
          ...(fiscal.preview.duplicate ? { replace: true } : {}),
        },
        done(false),
      );
      return;
    }
    if (locked && purchase) {
      // O lançamento só vai quando mudou: o mesmo, já ligado, fica como está.
      const samePayment =
        (mode === 'link' && linked === purchase.txn_id) || (mode === 'none' && !purchase.txn_id);
      updateReceipt.mutate(
        {
          id: purchase.id,
          categories: chosen(),
          ...(samePayment ? {} : { payment }),
          ...(member !== AUTO && member !== purchase.member_id ? { member_id: member } : {}),
        },
        done(true),
      );
      return;
    }
    const input: PurchaseInput = {
      ...(purchase ? { id: purchase.id } : {}),
      date: date ?? todayIso(),
      ...(store.trim() ? { store_name: store.trim() } : {}),
      ...(member !== AUTO ? { member_id: member } : {}),
      items: lines
        .filter((l) => l.description.trim())
        .map((l) => ({
          description: l.description.trim(),
          quantity: l.quantity,
          unit: l.unit,
          ...(l.unitPrice != null ? { unit_price: l.unitPrice } : {}),
          total: l.total ?? 0,
          ...(l.productId ? { product_id: l.productId } : {}),
          ...(l.categoryId ? { category_id: l.categoryId } : {}),
          ...(l.gtin.trim() ? { gtin: l.gtin.trim() } : {}),
          ...(l.taxable != null ? { taxable: l.taxable } : {}),
        })),
      ...(discount ? { discount } : {}),
      ...(taxAdded && tax ? { tax } : {}),
      payment,
    };
    savePurchase.mutate(input, done(Boolean(purchase)));
  };

  const photo = draft?.imageUrl;
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      index="06.11"
      title={
        fiscal
          ? fiscal.preview.parsed.store_name
            ? t('cupons-cupom-de', { loja: fiscal.preview.parsed.store_name })
            : t('cupons-cupom-fiscal')
          : purchase
            ? t('alimentacao-editar-compra')
            : t('alimentacao-nova-compra')
      }
      {...(photo
        ? {}
        : {
            description: locked
              ? t('alimentacao-compra-fiscal-dica')
              : t('alimentacao-nova-compra-dica'),
          })}
      size="xl"
      // Com a foto ao lado: quase a tela toda, para caber a imagem e os campos.
      resizable={!photo}
      resizeId="compra-manual"
      {...(photo ? { className: '!top-[3vh] h-[94vh] !max-h-[94vh] !w-[min(97vw,1500px)]' } : {})}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" disabled={!valid} loading={save.isPending} onClick={submit}>
            {fiscal?.preview.duplicate ? t('cupons-substituir') : t('alimentacao-gravar-compra')}
          </Button>
        </>
      }
    >
      <div
        className={cx(
          photo &&
            'grid h-full min-h-0 gap-4 md:grid-cols-[minmax(300px,2fr)_minmax(0,3fr)] max-md:grid-rows-[40vh_auto]',
        )}
      >
        {photo ? <PhotoPane url={photo} /> : null}
        <div className={cx('flex flex-col', photo ? 'min-h-0 gap-4 overflow-auto pr-1' : 'gap-5')}>
          {/* Texto colado sem linha de total: não há o que conferir. */}
          {draft && !(draft.reading.engine === 'text' && draft.reading.reading.total == null) ? (
            <DraftNotice draft={draft} paid={paid} />
          ) : null}
          {foreign ? <ForeignNotice f={foreign} /> : null}
          {fiscal?.preview.warnings.length ? (
            <ul className="flex list-disc flex-col gap-1 pl-4 text-[12.5px] text-dim">
              {fiscal.preview.warnings.map((w) => (
                <li key={w}>{w}</li>
              ))}
            </ul>
          ) : null}
          <div className="grid gap-3 sm:grid-cols-3">
            <Field label={t('alimentacao-data')}>
              <DateInput value={date} onValueChange={setDate} disabled={locked} />
            </Field>
            <Field
              label={t('alimentacao-loja')}
              {...(photo ? {} : { hint: t('alimentacao-loja-dica') })}
            >
              <Input
                value={store}
                disabled={locked}
                list={`${listId}-lojas`}
                onChange={(e) => setStore(e.target.value)}
              />
            </Field>
            <Field label={t('alimentacao-membro')}>
              <Select
                aria-label={t('alimentacao-membro')}
                value={member}
                onValueChange={setMember}
                options={[
                  { value: AUTO, label: t('alimentacao-membro-auto') },
                  ...members.map((m) => ({ value: m.id, label: m.name })),
                ]}
              />
            </Field>
          </div>
          <datalist id={`${listId}-lojas`}>
            {stores.map((s) => (
              <option key={s} value={s} />
            ))}
          </datalist>
          <datalist id={`${listId}-produtos`}>
            {products.map((p) => (
              <option
                key={p.id}
                value={p.name}
                {...(p.last_unit_price != null
                  ? {
                      label: `${formatMoney(p.last_unit_price)} / ${unitLabelOf(unitOf(p.last_unit))}`,
                    }
                  : {})}
              />
            ))}
          </datalist>

          {knownStore ? (
            <div>
              <Button size="sm" variant="secondary" loading={loadingLast} onClick={startFromLast}>
                {t('alimentacao-comecar-ultima')}
              </Button>
            </div>
          ) : null}
          <section
            ref={itemsRef}
            aria-label={t('alimentacao-itens-da-compra')}
            className="flex flex-col gap-2"
            onKeyDown={onItemsKey}
          >
            <div className="hidden grid-cols-[minmax(0,3fr)_92px_76px_112px_112px_minmax(0,2fr)_64px] gap-2 label-caps text-dim md:grid">
              <span>{t('alimentacao-item-descricao')}</span>
              {/* Todas as linhas na mesma unidade: o cabeçalho é a unidade (KG, L…). */}
              <span>{qtyHeader}</span>
              <span>{t('alimentacao-item-unidade')}</span>
              <span>{t('alimentacao-item-preco')}</span>
              <span>{t('alimentacao-item-total')}</span>
              <span>{t('alimentacao-categoria')}</span>
              <span />
            </div>
            {lines.map((l, i) => {
              const n = i + 1;
              return (
                <div
                  key={l.key}
                  className="grid grid-cols-2 gap-2 border-b border-line pb-2 md:grid-cols-[minmax(0,3fr)_92px_76px_112px_112px_minmax(0,2fr)_64px] md:border-0 md:pb-0"
                >
                  <Input
                    className="col-span-2 md:col-span-1"
                    aria-label={t('alimentacao-item-descricao-n', { n })}
                    placeholder={t('alimentacao-item-descricao-exemplo')}
                    list={`${listId}-produtos`}
                    disabled={locked}
                    value={l.description}
                    onChange={(e) => describe(l.key, e.target.value)}
                    onBlur={() => guessLine(l)}
                  />
                  <Input
                    aria-label={t('alimentacao-item-qtd-n', { n })}
                    inputMode="decimal"
                    numeric
                    disabled={locked}
                    suffix={<span className="text-[12px] text-dim">{unitLabelOf(l.unit)}</span>}
                    value={l.quantity}
                    onChange={(e) => set(l.key, { quantity: e.target.value })}
                    onBlur={() => {
                      // "0,734 kg" digitado junto: a unidade vai para o campo dela.
                      const m = QTY_UNIT.exec(l.quantity.trim());
                      const unit = m ? m[2]!.toLowerCase() : l.unit;
                      set(l.key, { unit, quantity: qtyText(m ? m[1]! : l.quantity, unit) });
                    }}
                  />
                  <Select
                    aria-label={t('alimentacao-item-unidade-n', { n })}
                    value={l.unit}
                    disabled={locked}
                    onValueChange={(unit) =>
                      set(l.key, { unit, quantity: qtyText(l.quantity, unit) })
                    }
                    options={
                      units.some((u) => u.value === l.unit)
                        ? units
                        : [...units, { value: l.unit, label: unitLabelOf(l.unit) }]
                    }
                  />
                  <MoneyInput
                    aria-label={t('alimentacao-item-preco-n', { n })}
                    value={l.unitPrice}
                    disabled={locked}
                    onValueChange={(unitPrice) =>
                      set(l.key, { unitPrice, totalTyped: false, priceTyped: unitPrice != null })
                    }
                  />
                  <MoneyInput
                    aria-label={t('alimentacao-item-total-n', { n })}
                    value={l.total}
                    disabled={locked}
                    onValueChange={(total) => set(l.key, { total, totalTyped: total != null })}
                  />
                  <Select
                    className="col-span-2 md:col-span-1"
                    aria-label={t('alimentacao-item-categoria-n', { n })}
                    value={l.categoryId ?? AUTO}
                    onValueChange={(v) => set(l.key, { categoryId: v === AUTO ? null : v })}
                    options={[
                      {
                        value: AUTO,
                        // A categoria que o app vai usar e de onde ela veio.
                        label: l.guess
                          ? l.guess.category_id && l.guess.reason !== 'none'
                            ? `${cats.find((c) => c.id === l.guess!.category_id)?.path ?? ''} · ${t(REASON_KEY[l.guess.reason])}`
                            : t('alimentacao-palpite-nenhum')
                          : t('alimentacao-categoria-automatica'),
                      },
                      ...cats.map((c) => ({ value: c.id, label: c.path })),
                    ]}
                  />
                  <div className={cx('flex items-center gap-1', locked && 'invisible')}>
                    <IconButton
                      label={t('alimentacao-item-codigo-mostrar-n', { n })}
                      icon={<Icon name="reticle" size={12} />}
                      aria-pressed={l.showGtin}
                      onClick={() => set(l.key, { showGtin: !l.showGtin })}
                    />
                    <IconButton
                      label={t('alimentacao-item-remover-n', { n })}
                      icon={<Icon name="x" size={12} />}
                      disabled={lines.length === 1}
                      onClick={() => setLines((ls) => ls.filter((x) => x.key !== l.key))}
                    />
                  </div>
                  {l.showGtin ? (
                    <div className="col-span-2 flex flex-wrap items-center gap-2 md:col-span-full">
                      <Input
                        className="max-w-56"
                        aria-label={t('alimentacao-item-codigo-n', { n })}
                        placeholder={t('alimentacao-item-codigo-exemplo')}
                        inputMode="numeric"
                        value={l.gtin}
                        onChange={(e) => set(l.key, { gtin: e.target.value })}
                      />
                      {canReadBarcode() ? (
                        <FileInput
                          aria-label={t('alimentacao-item-codigo-foto-n', { n })}
                          accept="image/*"
                          onChange={async (e) => {
                            const f = e.target.files?.[0];
                            e.target.value = '';
                            if (!f) return;
                            const found = await readBarcode(f);
                            if (found) set(l.key, { gtin: found });
                            else
                              toast({ tone: 'warning', title: t('alimentacao-codigo-foto-nada') });
                          }}
                        />
                      ) : null}
                      <span className="text-[12px] text-dim">
                        {t('alimentacao-item-codigo-dica')}
                      </span>
                    </div>
                  ) : null}
                </div>
              );
            })}
            {locked ? null : (
              <div>
                <Button
                  size="sm"
                  variant="secondary"
                  icon={<Icon name="plus" size={12} />}
                  onClick={() => setLines((ls) => [...ls, blank()])}
                >
                  {t('alimentacao-item-adicionar')}
                </Button>
              </div>
            )}
          </section>

          <div
            className={cx(
              'grid items-end gap-3 border-t border-line pt-3',
              taxAdded ? 'sm:grid-cols-4' : 'sm:grid-cols-3',
            )}
          >
            <div className="text-[13px]">
              <span className="text-dim">{t('alimentacao-soma-itens')}</span>{' '}
              <Money cents={itemsTotal} />
            </div>
            <Field label={t('alimentacao-desconto')}>
              <MoneyInput value={discount} onValueChange={setDiscount} disabled={locked} />
            </Field>
            {taxAdded ? (
              <Field
                label={t('alimentacao-imposto-somado')}
                hint={t('alimentacao-imposto-somado-dica')}
              >
                <MoneyInput value={tax} onValueChange={setTax} disabled={locked} />
              </Field>
            ) : null}
            <div className="text-[15px]">
              <span className="text-dim">{t('alimentacao-total-pago')}</span> <Money cents={paid} />
            </div>
          </div>
          {fiscal && fiscal.preview.tax > 0 ? (
            <p className="text-[12.5px] text-dim">
              {withSlots(t('cupons-imposto-valor', { valor: slot(0) }), [
                <Money key="v" cents={fiscal.preview.tax} />,
              ])}{' '}
              {fiscal.preview.tax_printed ? t('cupons-tributos-nota') : t('cupons-estimado-itens')}
            </p>
          ) : null}

          <fieldset
            className={cx('flex flex-col border-t border-line pt-3', photo ? 'gap-2' : 'gap-3')}
          >
            <legend className="label-caps pb-2 text-dim">{t('alimentacao-pagamento')}</legend>
            <div
              role="radiogroup"
              aria-label={t('alimentacao-pagamento')}
              className="flex flex-wrap gap-1.5"
            >
              {(['create', 'link', 'none'] as const).map((m) => (
                <button
                  key={m}
                  type="button"
                  role="radio"
                  aria-checked={mode === m}
                  onClick={() => setMode(m)}
                  className="h-8 rounded-control border border-line px-3 text-[13px] text-dim hover:border-accent aria-checked:border-accent aria-checked:bg-accent-soft aria-checked:text-accent"
                >
                  {t(PAYMENT_KEY[m])}
                </button>
              ))}
            </div>
            {mode === 'create' ? (
              <div className="grid gap-3 sm:grid-cols-2">
                <Field label={t('alimentacao-conta')}>
                  <Select
                    aria-label={t('alimentacao-conta')}
                    value={accountId}
                    onValueChange={setAccount}
                    options={accounts.map((a) => ({ value: a.id, label: a.name }))}
                  />
                </Field>
                <Field
                  label={t('alimentacao-categoria-lancamento')}
                  {...(photo ? {} : { hint: t('alimentacao-categoria-lancamento-dica') })}
                >
                  <CategoryPicker
                    kind="expense"
                    value={txnCategory}
                    onChange={setTxnCategory}
                    className="[&>button]:h-9 [&>button]:w-full"
                  />
                </Field>
              </div>
            ) : null}
            {mode === 'link' ? (
              options.length ? (
                <ul className="flex flex-col gap-1" aria-label={t('alimentacao-candidatos')}>
                  {options.map((c) => (
                    <li key={c.txn_id}>
                      <label className="flex cursor-pointer items-center gap-2 rounded-control px-2 py-1 text-[13px] hover:bg-surface-2">
                        <input
                          type="radio"
                          name={`${listId}-lancamento`}
                          checked={linked === c.txn_id}
                          onChange={() => setLinked(c.txn_id)}
                        />
                        {'current' in c ? (
                          <span>
                            {purchase?.txn_created
                              ? t('alimentacao-lancamento-criado')
                              : t('alimentacao-lancamento-atual')}
                          </span>
                        ) : (
                          <span className="flex flex-1 justify-between gap-3">
                            <span>
                              {formatDate(c.date)} · {c.description}{' '}
                              <span className="text-dim">· {c.account_name}</span>
                            </span>
                            <Money cents={c.amount} />
                          </span>
                        )}
                      </label>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="text-[12.5px] text-dim">{t('alimentacao-sem-candidatos')}</p>
              )
            ) : null}
            {mode === 'link' && rest != null && rest !== 0 ? (
              <p
                role={rest < 0 ? 'alert' : 'note'}
                className={cx('text-[12.5px]', rest < 0 ? 'text-negative' : 'text-dim')}
              >
                {withSlots(
                  t(rest < 0 ? 'alimentacao-compra-passa' : 'alimentacao-sem-detalhe', {
                    valor: slot(0),
                  }),
                  [<Money key="v" cents={Math.abs(rest)} />],
                )}
              </p>
            ) : null}
            {mode === 'none' ? (
              <p className="text-[12.5px] text-dim">{t('alimentacao-pagamento-sem-dica')}</p>
            ) : null}
          </fieldset>
          <InlineError error={save.error} />
        </div>
      </div>
    </Dialog>
  );
}
