/**
 * 06 · Consumo › Compras (docs/consumption.md): as compras do período (digitadas e cupons fiscais),
 * com busca por loja ou item e filtro por segmento. "Adicionar compra" é a única porta de
 * entrada: soltar a foto, o PDF ou o XML, colar o link do QR, ou digitar os itens. Abrir uma
 * compra mostra os itens; toda compra se edita (no cupom fiscal, só a categoria dos itens, o
 * membro e o lançamento).
 */
import { useMemo, useState } from 'react';

import type { Purchase } from '@/api/generated/Purchase';
import { useCountryFeatures } from '@/api/hooks/country';
import { useDeletePurchase, useFoodProducts, usePurchases } from '@/api/hooks/food';
import { useDeleteReceipt, useReceipt } from '@/api/hooks/receipts';
import {
  Button,
  Dialog,
  Field,
  Icon,
  Input,
  InstrumentCard,
  Money,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { ReceiptReader } from '@/features/receipts/ReceiptReader';
import { PurchasesCard } from '@/features/receipts/ReceiptsPanel';
import { QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatMoney } from '@/lib/format';
import { monthBounds, shiftMonth } from '@/lib/months';

import { SegmentChips } from './FoodPanels';
import { UnitPrice } from './units';
import { PurchaseDialog } from './PurchaseDialog';

type Range = 'mes' | '3m' | '12m';

/** Do primeiro dia do período ao último dia do mês escolhido. */
function period(month: string, range: Range): { from: string; to: string } {
  const back = range === 'mes' ? 0 : range === '3m' ? 2 : 11;
  return { from: monthBounds(shiftMonth(month, -back)).from, to: monthBounds(month).to };
}

export function PurchasesPanel({
  month,
  segment,
  onSegment,
}: {
  month: string;
  segment: string | null;
  onSegment: (id: string | null) => void;
}) {
  const features = useCountryFeatures();
  const [range, setRange] = useState<Range>('mes');
  const [query, setQuery] = useState('');
  const [adding, setAdding] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [repeating, setRepeating] = useState<string | null>(null);
  const [taxPeriod, setTaxPeriod] = useState('mes');
  const r = useMemo(() => period(month, range), [month, range]);
  const q = usePurchases({
    ...r,
    ...(segment ? { segment_id: segment } : {}),
    ...(query.trim() ? { query: query.trim() } : {}),
  });
  const removeManual = useDeletePurchase();
  const removeReceipt = useDeleteReceipt();
  const rows = q.data ?? [];
  const remove = (p: Purchase) => {
    const done = {
      onSuccess: () => {
        if (selected === p.receipt.id) setSelected(null);
        toast({ tone: 'success', title: t('alimentacao-compra-apagada') });
      },
    };
    if (p.receipt.origin === 'manual') removeManual.mutate({ id: p.receipt.id }, done);
    else removeReceipt.mutate({ id: p.receipt.id }, done);
  };
  const columns: TechColumn<Purchase>[] = [
    {
      key: 'date',
      header: t('alimentacao-data'),
      width: '96px',
      cell: (p) => formatDate(p.receipt.date),
    },
    {
      key: 'store',
      header: t('alimentacao-loja'),
      cell: (p) => (
        <span className="flex flex-col">
          <span className="text-fg">{p.receipt.store_name ?? t('alimentacao-sem-loja')}</span>
          <span className="text-[11.5px] text-dim">
            {t('alimentacao-compra-linha', {
              n: p.receipt.item_count,
              membro: p.receipt.member_name ?? t('alimentacao-familia'),
            })}
            {' · '}
            {p.receipt.origin === 'manual'
              ? t('alimentacao-origem-manual')
              : t('alimentacao-origem-cupom')}
          </span>
        </span>
      ),
    },
    {
      key: 'segments',
      header: t('alimentacao-segmentos'),
      cell: (p) => (
        <span className="text-[12.5px] text-dim">
          {p.segments
            .slice(0, 2)
            .map((s) => s.name)
            .join(', ')}
          {p.segments.length > 2 ? ` +${p.segments.length - 2}` : ''}
        </span>
      ),
    },
    {
      key: 'txn',
      header: t('alimentacao-lancamento'),
      cell: (p) =>
        p.receipt.txn_id ? (
          <span className="text-[12.5px]">{t('alimentacao-com-lancamento')}</span>
        ) : (
          <span className="text-[12.5px] text-warning">{t('alimentacao-sem-lancamento')}</span>
        ),
    },
    {
      key: 'paid',
      header: t('alimentacao-pago'),
      numeric: true,
      cell: (p) => <Money cents={p.receipt.paid} />,
    },
    ...(features.receipts
      ? [
          {
            key: 'tax',
            header: t('alimentacao-imposto'),
            numeric: true,
            cell: (p: Purchase) => <Money cents={p.receipt.tax} />,
          },
        ]
      : []),
    {
      key: 'actions',
      header: '',
      cell: (p) => (
        <span className="flex justify-end gap-1">
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('alimentacao-repetir-de', {
              loja: p.receipt.store_name ?? t('alimentacao-sem-loja'),
            })}
            onClick={(e) => {
              e.stopPropagation();
              setRepeating(p.receipt.id);
            }}
          >
            {t('alimentacao-repetir')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('alimentacao-editar-de', {
              loja: p.receipt.store_name ?? t('alimentacao-sem-loja'),
            })}
            onClick={(e) => {
              e.stopPropagation();
              setEditing(p.receipt.id);
            }}
          >
            {t('comum-editar')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name="x" size={12} />}
            aria-label={t('alimentacao-apagar-de', {
              loja: p.receipt.store_name ?? t('alimentacao-sem-loja'),
            })}
            onClick={(e) => {
              e.stopPropagation();
              remove(p);
            }}
          >
            {t('comum-apagar')}
          </Button>
        </span>
      ),
    },
  ];
  return (
    <div className="flex flex-col gap-4">
      <SegmentChips value={segment} onChange={onSegment} />
      <div className="flex flex-wrap items-end gap-3">
        <Field label={t('alimentacao-buscar-compra')} className="min-w-60 flex-1 sm:max-w-sm">
          <Input value={query} onChange={(e) => setQuery(e.target.value)} />
        </Field>
        <Field label={t('alimentacao-periodo')}>
          <Select
            aria-label={t('alimentacao-periodo')}
            value={range}
            onValueChange={(v) => setRange(v as Range)}
            options={[
              { value: 'mes', label: t('alimentacao-periodo-mes') },
              { value: '3m', label: t('alimentacao-periodo-3m') },
              { value: '12m', label: t('alimentacao-periodo-12m') },
            ]}
          />
        </Field>
        <div className="ml-auto flex flex-wrap gap-2">
          <Button
            variant="primary"
            icon={<Icon name="plus" size={12} />}
            onClick={() => setAdding(true)}
          >
            {t('alimentacao-adicionar-compra')}
          </Button>
        </div>
      </div>
      <InstrumentCard index="06.10" title={t('alimentacao-compras-titulo')} padded={false}>
        <QueryState loading={q.isLoading} error={q.error} onRetry={() => void q.refetch()}>
          <TechTable
            label={t('alimentacao-compras-titulo')}
            prefsId="Compras do consumo"
            dense
            rows={rows}
            getRowId={(p) => p.receipt.id}
            selectedId={selected}
            onSelectedChange={(id) => setSelected(id)}
            columns={columns}
            empty={<p className="p-4 text-[13px] text-dim">{t('alimentacao-sem-compras-lista')}</p>}
          />
        </QueryState>
      </InstrumentCard>
      {selected ? (
        <PurchaseDetail
          id={selected}
          onEdit={() => setEditing(selected)}
          onClose={() => setSelected(null)}
        />
      ) : null}
      {features.receipts ? <PurchasesCard period={taxPeriod} onPeriod={setTaxPeriod} /> : null}
      <Dialog
        open={adding}
        onOpenChange={setAdding}
        index="06.09"
        title={t('alimentacao-adicionar-compra')}
        description={t('alimentacao-adicionar-compra-dica')}
        size="lg"
      >
        <ReceiptReader brazil={features.receipts} onSaved={() => setAdding(false)} />
      </Dialog>
      {editing ? <EditPurchase id={editing} onClose={() => setEditing(null)} /> : null}
      {repeating ? <RepeatPurchase id={repeating} onClose={() => setRepeating(null)} /> : null}
    </div>
  );
}

/** Carrega a compra com os itens antes de abrir a edição. */
function EditPurchase({ id, onClose }: { id: string; onClose: () => void }) {
  const r = useReceipt(id);
  return r.data ? <PurchaseDialog purchase={r.data} onClose={onClose} /> : null;
}

/** Compra nova com os itens de uma anterior (feira, padaria, a compra do mês), na data de hoje. */
function RepeatPurchase({ id, onClose }: { id: string; onClose: () => void }) {
  const r = useReceipt(id);
  const products = useFoodProducts({});
  return r.data && products.data ? (
    <PurchaseDialog purchase={null} repeat={r.data} onClose={onClose} />
  ) : null;
}

function PurchaseDetail({
  id,
  onEdit,
  onClose,
}: {
  id: string;
  onEdit: () => void;
  onClose: () => void;
}) {
  const r = useReceipt(id);
  const products = useFoodProducts({}).data ?? [];
  const byId = new Map(products.map((p) => [p.id, p]));
  const c = r.data;
  return (
    <QueryState loading={r.isLoading} error={r.error}>
      {c ? (
        <InstrumentCard
          index="06.12"
          title={`${c.store_name ?? t('alimentacao-sem-loja')} · ${formatDate(c.date)}`}
          actions={
            <span className="flex gap-1">
              <Button size="sm" variant="secondary" onClick={onEdit}>
                {t('comum-editar')}
              </Button>
              <Button size="sm" variant="ghost" onClick={onClose}>
                {t('comum-fechar')}
              </Button>
            </span>
          }
          padded={false}
        >
          <TechTable
            label={t('alimentacao-itens-da-compra')}
            prefsId="Itens da compra"
            dense
            rows={c.items}
            getRowId={(i) => String(i.line)}
            columns={[
              { key: 'description', header: t('alimentacao-item-descricao') },
              {
                key: 'quantity',
                header: t('alimentacao-item-qtd'),
                numeric: true,
                cell: (i) => `${i.quantity ?? '1'} ${i.unit ?? ''}`.trim(),
              },
              {
                key: 'category',
                header: t('alimentacao-categoria'),
                cell: (i) => (
                  <span className="text-dim">
                    {(i.product_id && byId.get(i.product_id)?.category_path) || '—'}
                  </span>
                ),
              },
              {
                key: 'base_price',
                header: t('alimentacao-preco-unidade'),
                numeric: true,
                cell: (i) =>
                  i.base_price != null ? (
                    <span>
                      <UnitPrice cents={i.base_price} unit={i.base_unit} />
                    </span>
                  ) : (
                    '—'
                  ),
              },
              {
                key: 'total',
                header: t('alimentacao-item-total'),
                numeric: true,
                cell: (i) => <Money cents={i.total} />,
              },
            ]}
          />
          {c.discount > 0 ? (
            <p className="px-4 py-2 text-[12.5px] text-dim">
              {t('alimentacao-desconto-de', { valor: formatMoney(c.discount) })}
            </p>
          ) : null}
        </InstrumentCard>
      ) : null}
    </QueryState>
  );
}
