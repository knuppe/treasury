/**
 * Tela 06 · Consumo (docs/consumption.md): a visão geral do mês (o total e cada segmento contra o mês
 * anterior, a comida em casa e fora, as classes e as categorias do segmento escolhido, os
 * produtos que mais pesaram e os que mais subiram, os seis meses até este) e os produtos com o
 * preço por kg, L ou unidade ao longo do tempo. O lançamento é o dinheiro; a compra é o detalhe:
 * só entram compras com lançamento, e as sem lançamento aparecem à parte.
 */
import { useState } from 'react';

import type { FoodClass } from '@/api/generated/FoodClass';
import type { FoodMonth } from '@/api/generated/FoodMonth';
import type { FoodPricePoint } from '@/api/generated/FoodPricePoint';
import type { FoodProduct } from '@/api/generated/FoodProduct';
import type { FoodSummary } from '@/api/generated/FoodSummary';
import {
  useFoodProducts,
  useFoodSummary,
  useMergeProducts,
  useProductCategories,
  useProductHistory,
  useRelinkProducts,
  useUpdateProduct,
} from '@/api/hooks/food';
import {
  Button,
  Chip,
  ChipGroup,
  Field,
  Input,
  InstrumentCard,
  Measure,
  Money,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatMoney } from '@/lib/format';

import { BarcodeField } from './BarcodeField';
import { UnitPrice } from './units';

import { MergeSuggestionsCard } from './MergeSuggestions';
import { ProductHealthSection, SameLineList } from './ProductHealth';

const CLASS_KEY = {
  essential: 'alimentacao-classe-essencial',
  complementary: 'alimentacao-classe-complementar',
  superfluous: 'alimentacao-classe-superfluo',
} as const;
export const classLabel = (c: FoodClass | null | undefined) =>
  c ? t(CLASS_KEY[c]) : t('alimentacao-classe-sem');
const CLASS_COLOR: Record<string, string> = {
  essential: 'bg-accent',
  complementary: 'bg-accent/45',
  superfluous: 'bg-warning',
  none: 'bg-line',
};
const UNIT_KEY = {
  kg: 'alimentacao-por-kg',
  l: 'alimentacao-por-l',
  un: 'alimentacao-por-un',
} as const;
export const unitLabel = (u: string | null | undefined) =>
  u && u in UNIT_KEY ? t(UNIT_KEY[u as keyof typeof UNIT_KEY]) : '';
export const pct = (v: number | null | undefined) =>
  v == null
    ? '—'
    : `${v > 0 ? '+' : ''}${(v * 100).toLocaleString(undefined, { maximumFractionDigits: 1 })}%`;
const INHERIT = '__herdar__';

/** Variação contra o mês anterior (`null` sem base). */
const change = (now: number, before: number) => (before > 0 ? now / before - 1 : null);

/** Os segmentos (raízes da árvore de categorias de produto), para filtrar. */
export function SegmentChips({
  value,
  onChange,
}: {
  value: string | null;
  onChange: (id: string | null) => void;
}) {
  const roots = (useProductCategories().data ?? []).filter((c) => c.depth === 0);
  const chip = (id: string | null, label: string) => (
    <Chip key={id ?? 'todos'} pressed={value === id} onClick={() => onChange(id)}>
      {label}
    </Chip>
  );
  return (
    <ChipGroup label={t('alimentacao-segmentos')}>
      {chip(null, t('alimentacao-todos'))}
      {roots.map((c) => chip(c.id, c.name))}
    </ChipGroup>
  );
}

/** Uma barra só com as classes (essencial, complementar, supérfluo, sem classe). */
function ClassBar({ s }: { s: FoodSummary }) {
  const total = s.classes.reduce((a, c) => a + Math.max(0, c.spent), 0);
  if (total <= 0) return <p className="text-[12.5px] text-dim">{t('alimentacao-sem-compras')}</p>;
  return (
    <div className="flex flex-col gap-3">
      <div className="flex h-3 overflow-hidden rounded-full bg-surface-2" aria-hidden>
        {s.classes.map((c) => (
          <span
            key={c.class ?? 'none'}
            className={CLASS_COLOR[c.class ?? 'none']}
            style={{ width: `${(Math.max(0, c.spent) / total) * 100}%` }}
          />
        ))}
      </div>
      <ul className="flex flex-col gap-1 text-[13px]" aria-label={t('alimentacao-por-classe')}>
        {s.classes.map((c) => (
          <li key={c.class ?? 'none'} className="flex items-center justify-between gap-3">
            <span className="flex items-center gap-2">
              <span className={cx('size-2.5 rounded-full', CLASS_COLOR[c.class ?? 'none'])} />
              {c.label}
            </span>
            <span>
              <Money cents={c.spent} />{' '}
              <span className="text-dim">· {Math.round((c.spent / total) * 100)}%</span>
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function SegmentTiles({
  s,
  value,
  onChange,
}: {
  s: FoodSummary;
  value: string | null;
  onChange: (id: string | null) => void;
}) {
  if (!s.segments.length)
    return <p className="text-[12.5px] text-dim">{t('alimentacao-sem-compras')}</p>;
  return (
    <ul
      className="grid gap-2 @min-[400px]:grid-cols-2 @min-[790px]:grid-cols-4"
      aria-label={t('alimentacao-segmentos')}
    >
      {s.segments.map((g) => {
        const v = change(g.spent, g.previous);
        const selected = g.category_id != null && g.category_id === value;
        return (
          <li key={g.category_id ?? 'sem'}>
            <button
              type="button"
              aria-pressed={selected}
              disabled={g.category_id == null}
              onClick={() => onChange(selected ? null : g.category_id)}
              className="flex w-full flex-col gap-1 rounded-card border border-line px-3 py-2 text-left hover:border-accent aria-pressed:border-accent aria-pressed:bg-accent-soft disabled:hover:border-line"
            >
              <span className="text-[12.5px] text-dim">{g.name}</span>
              <span className="text-[18px] text-fg">
                <Money cents={g.spent} />
              </span>
              <span className="text-[11.5px] text-dim">
                {t('alimentacao-itens', { n: g.items })}
                {v != null ? (
                  <>
                    {' · '}
                    <span className={v > 0 ? 'text-warning' : 'text-positive'}>{pct(v)}</span>
                  </>
                ) : null}
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

function MonthsTable({ s }: { s: FoodSummary }) {
  const columns: TechColumn<FoodMonth>[] = [
    { key: 'month', header: t('alimentacao-mes'), width: '90px' },
    {
      key: 'total',
      header: t('alimentacao-total'),
      numeric: true,
      cell: (r) => <Money cents={r.segments.reduce((a, g) => a + g.spent, 0)} />,
    },
    {
      key: 'home',
      header: t('alimentacao-em-casa'),
      numeric: true,
      cell: (r) => <Money cents={r.home} />,
    },
    {
      key: 'out',
      header: t('alimentacao-fora'),
      numeric: true,
      cell: (r) => <Money cents={r.out} />,
    },
    {
      key: 'non_food',
      header: t('alimentacao-outros-segmentos'),
      numeric: true,
      cell: (r) => <Money cents={r.non_food} />,
    },
  ];
  return (
    <TechTable
      label={t('alimentacao-ultimos-meses')}
      prefsId="Consumo por mês"
      columns={columns}
      rows={s.months}
      getRowId={(r) => r.month}
      dense
    />
  );
}

/** Visão geral do mês, com o segmento escolhido em detalhe. */
export function ConsumoOverview({
  month,
  segment,
  onSegment,
}: {
  month: string;
  segment: string | null;
  onSegment: (id: string | null) => void;
}) {
  const q = useFoodSummary({ month, ...(segment ? { segment_id: segment } : {}) });
  const relink = useRelinkProducts();
  const s = q.data;
  const chosen = s?.segments.find((g) => g.category_id === segment) ?? null;
  const v = s ? change(s.total, s.previous_total) : null;
  return (
    <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
      {s ? (
        <div className="flex flex-col gap-4">
          {s.unlinked_items > 0 ? (
            <div className="flex flex-wrap items-center gap-3 rounded-card border border-accent/50 bg-accent/5 px-3 py-2 text-[12.5px]">
              <span>{t('alimentacao-itens-sem-produto', { n: s.unlinked_items })}</span>
              <Button
                size="sm"
                variant="secondary"
                loading={relink.isPending}
                onClick={() =>
                  relink.mutate(
                    {},
                    {
                      onSuccess: (r) =>
                        toast({
                          tone: 'success',
                          title: t('alimentacao-ligados', { n: r.linked }),
                        }),
                    },
                  )
                }
              >
                {t('alimentacao-ligar')}
              </Button>
            </div>
          ) : null}
          <InstrumentCard index="06.1" title={t('alimentacao-resumo')}>
            <div className="grid gap-5 @min-[400px]:grid-cols-2 @min-[790px]:grid-cols-4">
              <Measure
                size="md"
                label={t('alimentacao-total-mes')}
                value={<Money cents={s.total} />}
                changeKey={s.total}
                hint={
                  v != null
                    ? t('alimentacao-contra-anterior', {
                        variacao: pct(v),
                        anterior: formatMoney(s.previous_total),
                      })
                    : undefined
                }
              />
              <Measure
                size="md"
                label={t('alimentacao-em-casa')}
                value={<Money cents={s.home} />}
                hint={
                  s.market_without_receipt > 0
                    ? t('alimentacao-em-casa-dica', {
                        cupons: formatMoney(s.receipt_items),
                        sem: formatMoney(s.market_without_receipt),
                      })
                    : undefined
                }
              />
              <Measure
                size="md"
                label={t('alimentacao-fora')}
                value={<Money cents={s.out} />}
                hint={t('alimentacao-por-pessoa-dica', {
                  n: s.people,
                  valor: formatMoney(s.per_person),
                })}
              />
              <Measure
                size="md"
                label={t('alimentacao-compras-sem-lancamento')}
                value={<Money cents={s.receipts_without_txn_amount} />}
                hint={t('alimentacao-compras-sem-lancamento-dica', {
                  n: s.receipts_without_txn,
                  total: s.receipts,
                })}
              />
            </div>
          </InstrumentCard>
          <InstrumentCard index="06.2" title={t('alimentacao-segmentos')}>
            <SegmentTiles s={s} value={segment} onChange={onSegment} />
            <p className="mt-3 text-[12px] text-dim">{t('alimentacao-segmentos-dica')}</p>
          </InstrumentCard>
          <div className="grid gap-4 @min-[790px]:grid-cols-2">
            <InstrumentCard
              index="06.3"
              title={
                chosen
                  ? t('alimentacao-categorias-de', { segmento: chosen.name })
                  : t('alimentacao-por-categoria')
              }
              actions={
                segment ? (
                  <Button size="sm" variant="ghost" onClick={() => onSegment(null)}>
                    {t('alimentacao-ver-todos')}
                  </Button>
                ) : undefined
              }
            >
              {s.categories.length ? (
                <ul className="flex flex-col gap-1.5 text-[13px]">
                  {s.categories.map((c) => {
                    const top = s.categories[0]?.spent || 1;
                    return (
                      <li key={c.category_id ?? 'sem'} className="flex flex-col gap-0.5">
                        <span className="flex justify-between gap-3">
                          <span>
                            {c.name} <span className="text-dim">· {classLabel(c.class)}</span>
                          </span>
                          <Money cents={c.spent} />
                        </span>
                        <span className="h-1 rounded-full bg-surface-2">
                          <span
                            className="block h-1 rounded-full bg-accent"
                            style={{ width: `${Math.max(2, (c.spent / top) * 100)}%` }}
                          />
                        </span>
                      </li>
                    );
                  })}
                </ul>
              ) : (
                <p className="text-[12.5px] text-dim">{t('alimentacao-sem-compras')}</p>
              )}
              {!segment && s.out_categories.length ? (
                <div className="mt-4 border-t border-line pt-3">
                  <span className="label-caps text-dim">{t('alimentacao-fora')}</span>
                  <ul className="mt-1 flex flex-col gap-1 text-[13px]">
                    {s.out_categories.map((c) => (
                      <li key={c.category_id} className="flex justify-between gap-3">
                        <span>{c.name}</span>
                        <Money cents={c.spent} />
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </InstrumentCard>
            <InstrumentCard index="06.4" title={t('alimentacao-por-classe')}>
              <ClassBar s={s} />
              <p className="mt-3 text-[12px] text-dim">{t('alimentacao-classes-dica')}</p>
            </InstrumentCard>
          </div>
          <div className="grid gap-4 @min-[790px]:grid-cols-2">
            <InstrumentCard index="06.5" title={t('alimentacao-mais-pesaram')}>
              {s.top_products.length ? (
                <ol className="flex flex-col gap-1 text-[13px]">
                  {s.top_products.map((p) => (
                    <li key={p.product_id} className="flex justify-between gap-3">
                      <span className="min-w-0 truncate">
                        {p.name}{' '}
                        <span className="text-dim">· {t('alimentacao-itens', { n: p.items })}</span>
                      </span>
                      <Money cents={p.spent} />
                    </li>
                  ))}
                </ol>
              ) : (
                <p className="text-[12.5px] text-dim">{t('alimentacao-sem-compras')}</p>
              )}
            </InstrumentCard>
            <InstrumentCard index="06.6" title={t('alimentacao-subiram')}>
              {s.risers.length ? (
                <ol className="flex flex-col gap-1 text-[13px]">
                  {s.risers.map((p) => (
                    <li key={p.id} className="flex justify-between gap-3">
                      <span className="min-w-0 truncate">{p.name}</span>
                      <span className="shrink-0">
                        <UnitPrice cents={p.last_price ?? 0} unit={p.base_unit} />{' '}
                        <span className="text-warning">{pct(p.change)}</span>
                      </span>
                    </li>
                  ))}
                </ol>
              ) : (
                <p className="text-[12.5px] text-dim">{t('alimentacao-sem-altas')}</p>
              )}
            </InstrumentCard>
          </div>
          <InstrumentCard index="06.7" title={t('alimentacao-ultimos-meses')}>
            <MonthsTable s={s} />
            {s.home + s.out > 0 ? (
              <p className="mt-2 text-[12px] text-dim">
                {t('alimentacao-fora-parte', {
                  pct: Math.round((s.out / (s.home + s.out)) * 100),
                })}
              </p>
            ) : null}
          </InstrumentCard>
        </div>
      ) : null}
    </QueryState>
  );
}

/** Só a última parte do caminho da categoria ("Alimentação > Bebidas > Sucos" → "Sucos"). */
const leaf = (path: string | null) => (path ? (path.split(/\s*[>›]\s*/).pop() ?? path) : null);

const PRODUCT_COLUMNS: TechColumn<FoodProduct>[] = [
  {
    key: 'name',
    get header() {
      return t('alimentacao-produto');
    },
    hideable: false,
    cell: (r) => (
      <span className="line-clamp-2 text-fg" title={r.name}>
        {r.name}
      </span>
    ),
  },
  {
    key: 'category_path',
    get header() {
      return t('alimentacao-categoria');
    },
    width: '180px',
    sortValue: (r) => leaf(r.category_path) ?? '',
    cell: (r) => (
      <span className="block truncate text-dim" title={r.category_path ?? undefined}>
        {leaf(r.category_path) ?? '—'}
      </span>
    ),
  },
  {
    key: 'effective_class',
    get header() {
      return t('alimentacao-classe');
    },
    width: '120px',
    cell: (r) => classLabel(r.effective_class),
  },
  {
    key: 'purchases',
    get header() {
      return t('alimentacao-compras');
    },
    width: '90px',
    numeric: true,
  },
  {
    key: 'last_price',
    get header() {
      return t('alimentacao-preco-unidade');
    },
    width: '140px',
    numeric: true,
    cell: (r) =>
      r.last_price != null ? (
        <span>
          <UnitPrice cents={r.last_price} unit={r.base_unit} />
        </span>
      ) : (
        '—'
      ),
  },
  {
    key: 'change',
    get header() {
      return t('alimentacao-variacao');
    },
    width: '90px',
    numeric: true,
    cell: (r) => (
      <span className={r.change != null && r.change > 0 ? 'text-warning' : 'text-dim'}>
        {pct(r.change)}
      </span>
    ),
  },
  {
    key: 'last_date',
    get header() {
      return t('alimentacao-ultima-compra');
    },
    width: '110px',
    numeric: true,
    cell: (r) => (r.last_date ? formatDate(r.last_date) : '—'),
  },
];

const HISTORY_COLUMNS: TechColumn<FoodPricePoint>[] = [
  {
    key: 'date',
    get header() {
      return t('alimentacao-data');
    },
    width: '100px',
    numeric: true,
    cell: (r) => formatDate(r.date),
  },
  {
    key: 'store_name',
    get header() {
      return t('alimentacao-loja');
    },
    cell: (r) => r.store_name ?? '—',
  },
  {
    key: 'total',
    get header() {
      return t('alimentacao-valor');
    },
    numeric: true,
    cell: (r) => <Money cents={r.total} />,
  },
  {
    key: 'base_price',
    get header() {
      return t('alimentacao-preco-unidade');
    },
    numeric: true,
    cell: (r) =>
      r.base_price != null ? (
        <span>
          <UnitPrice cents={r.base_price} unit={r.base_unit} />
        </span>
      ) : (
        '—'
      ),
  },
];

/** Produtos do segmento escolhido, com o histórico de preço do escolhido ao lado. */
export function FoodProductsPanel({
  segment,
  onSegment,
}: {
  segment: string | null;
  onSegment: (id: string | null) => void;
}) {
  const [query, setQuery] = useState('');
  const [barcode, setBarcode] = useState<'all' | 'with' | 'without'>('all');
  const [selected, setSelected] = useState<string | null>(null);
  const q = useFoodProducts({
    ...(segment ? { category_id: segment } : {}),
    ...(query.trim() ? { query: query.trim() } : {}),
  });
  const rows = (q.data ?? []).filter((p) =>
    barcode === 'all' ? true : barcode === 'with' ? Boolean(p.gtin) : !p.gtin,
  );
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-3">
        <SegmentChips
          value={segment}
          onChange={(id) => {
            setSelected(null);
            onSegment(id);
          }}
        />
        <div className="flex flex-wrap items-end gap-3">
          <Field label={t('alimentacao-buscar')} className="w-full max-w-sm">
            <Input value={query} onChange={(e) => setQuery(e.target.value)} />
          </Field>
          <Field label={t('alimentacao-codigo-barras')}>
            <Select
              aria-label={t('alimentacao-codigo-barras')}
              value={barcode}
              onValueChange={(v) => setBarcode(v as 'all' | 'with' | 'without')}
              options={[
                { value: 'all', label: t('alimentacao-codigo-todos') },
                { value: 'with', label: t('alimentacao-codigo-com') },
                { value: 'without', label: t('alimentacao-codigo-sem') },
              ]}
            />
          </Field>
        </div>
      </div>
      <MergeSuggestionsCard />
      {/* A tabela fica com a largura toda (nome legível); o produto escolhido abre embaixo. */}
      <InstrumentCard index="06.8" title={t('alimentacao-produtos')} padded={false}>
        <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
          <TechTable
            label={t('alimentacao-produtos')}
            prefsId="Produtos do consumo (largura toda)"
            columns={PRODUCT_COLUMNS}
            rows={rows}
            getRowId={(r) => r.id}
            selectedId={selected}
            onSelectedChange={(id) => setSelected(id)}
            maxHeight="520px"
            minWidth="760px"
            dense
            empty={<p className="p-4 text-[13px] text-dim">{t('alimentacao-sem-produtos')}</p>}
          />
        </QueryState>
      </InstrumentCard>
      {selected ? (
        <ProductDetail id={selected} products={rows} onMerged={() => setSelected(null)} />
      ) : (
        <p className="rounded-card border border-dashed border-line p-4 text-[12.5px] text-dim">
          {t('alimentacao-escolha-produto')}
        </p>
      )}
    </div>
  );
}

function ProductDetail({
  id,
  products,
  onMerged,
}: {
  id: string;
  products: FoodProduct[];
  onMerged: () => void;
}) {
  const h = useProductHistory(id);
  const cats = useProductCategories().data ?? [];
  const update = useUpdateProduct();
  const merge = useMergeProducts();
  const [into, setInto] = useState<string>('');
  const [name, setName] = useState<string | null>(null);
  const p = h.data?.product;
  const rename = () => {
    const v = name?.trim();
    if (p && v && v !== p.name) update.mutate({ id: p.id, name: v });
    setName(null);
  };
  return (
    <QueryState loading={h.isLoading} error={h.error}>
      {p ? (
        <InstrumentCard index="06.9" title={p.name}>
          <div className="grid items-start gap-5 @min-[900px]:grid-cols-[minmax(260px,2fr)_minmax(0,3fr)]">
            <div>
              {p.change != null ? (
                <p className="mb-3 text-[12.5px] text-dim">
                  {t('alimentacao-variacao-frase', {
                    primeiro: formatMoney(p.first_price ?? 0),
                    ultimo: formatMoney(p.last_price ?? 0),
                    unidade: unitLabel(p.base_unit),
                    variacao: pct(p.change),
                  })}
                </p>
              ) : null}
              <div className="mb-3 grid gap-3">
                <Field
                  label={t('alimentacao-nome-produto')}
                  hint={t('alimentacao-nome-produto-dica')}
                >
                  <Input
                    value={name ?? p.name}
                    onChange={(e) => setName(e.target.value)}
                    onBlur={rename}
                    onKeyDown={(e) => e.key === 'Enter' && rename()}
                  />
                </Field>
                <BarcodeField product={p} />
                <Field label={t('alimentacao-categoria')}>
                  <Select
                    aria-label={t('alimentacao-categoria')}
                    value={p.category_id ?? ''}
                    onValueChange={(v) => update.mutate({ id: p.id, category_id: v || null })}
                    options={cats.map((c) => ({ value: c.id, label: c.path }))}
                  />
                </Field>
                <Field label={t('alimentacao-classe')} hint={t('alimentacao-classe-dica')}>
                  <Select
                    aria-label={t('alimentacao-classe')}
                    value={p.class ?? INHERIT}
                    onValueChange={(v) =>
                      update.mutate({ id: p.id, class: v === INHERIT ? null : (v as FoodClass) })
                    }
                    options={[
                      { value: INHERIT, label: t('alimentacao-classe-da-categoria') },
                      ...(['essential', 'complementary', 'superfluous'] as const).map((c) => ({
                        value: c,
                        label: classLabel(c),
                      })),
                    ]}
                  />
                </Field>
                <Field label={t('alimentacao-juntar')} hint={t('alimentacao-juntar-dica')}>
                  <div className="flex gap-2">
                    <Select
                      aria-label={t('alimentacao-juntar')}
                      value={into}
                      onValueChange={setInto}
                      options={products
                        .filter((x) => x.id !== p.id)
                        .map((x) => ({ value: x.id, label: x.name }))}
                    />
                    <Button
                      size="sm"
                      variant="secondary"
                      disabled={!into}
                      loading={merge.isPending}
                      onClick={() =>
                        merge.mutate(
                          { from_id: p.id, into_id: into },
                          {
                            onSuccess: () => {
                              toast({ tone: 'success', title: t('alimentacao-juntados') });
                              onMerged();
                            },
                          },
                        )
                      }
                    >
                      {t('alimentacao-juntar-botao')}
                    </Button>
                  </div>
                </Field>
              </div>
            </div>
            <TechTable
              label={t('alimentacao-historico', { nome: p.name })}
              prefsId="Histórico de preço"
              columns={HISTORY_COLUMNS}
              rows={h.data?.points ?? []}
              getRowId={(r) => `${r.receipt_id}-${r.description}-${r.date}`}
              maxHeight="420px"
              dense
            />
          </div>
          {h.data?.same_line.length ? (
            <div className="mt-5">
              <SameLineList rows={h.data.same_line} />
            </div>
          ) : null}
          {h.data?.health ? (
            <div className="mt-5">
              <ProductHealthSection health={h.data.health} />
            </div>
          ) : null}
        </InstrumentCard>
      ) : null}
    </QueryState>
  );
}
