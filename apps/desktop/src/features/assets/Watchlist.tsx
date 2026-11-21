/**
 * Watchlist: ativos acompanhados com preço-alvo (e o potencial até ele) e alertas
 * de cotação acima/abaixo, que aparecem como disparados quando a cotação cruza.
 */
import { useState, type FormEvent } from 'react';
import { useNavigate } from 'react-router';

import { useAssets } from '@/api/hooks/assets';
import { useQuoteMap } from '@/api/hooks/market';
import { useUnwatch, useWatch, useWatchlist } from '@/api/hooks/portfolio';
import type { WatchlistItem } from '@/api/generated/WatchlistItem';
import {
  Button,
  Dialog,
  DialogClose,
  EmptyState,
  Field,
  Icon,
  Input,
  Private,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';

import { AssetPicker } from '../market/AssetPicker';
import { QuoteChange } from '../market/QuoteBits';
import { AssetSelect } from '../portfolio/components/Fields';
import { DecimalInput } from '../portfolio/components/DecimalInput';
import { Pct } from '../portfolio/components/Pct';
import { PriceWithOrigin } from '../portfolio/components/Quote';
import { ConfirmButton, ErrorBlock, LoadingBlock } from '../portfolio/components/States';
import { formatUnitPrice } from '../portfolio/decimal';
import { assetMenuItems } from './assetMenu';

export function WatchDialog({
  open,
  onOpenChange,
  item,
  assetId,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  item?: WatchlistItem | null;
  assetId?: string;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      index="07"
      title={
        item
          ? t('ativos-watch-titulo', { nome: item.asset.ticker ?? item.asset.name })
          : t('ativos-watch-adicionar-titulo')
      }
      description={t('ativos-watch-descricao')}
    >
      {open ? (
        <WatchForm item={item ?? null} assetId={assetId} onDone={() => onOpenChange(false)} />
      ) : null}
    </Dialog>
  );
}

function WatchForm({
  item,
  assetId: initial,
  onDone,
}: {
  item: WatchlistItem | null;
  assetId?: string;
  onDone: () => void;
}) {
  const assets = useAssets({}, { enabled: !item });
  const watch = useWatch();
  const [assetId, setAssetId] = useState(item?.asset.id ?? initial);
  const [target, setTarget] = useState<string | null>(item?.target_price ?? null);
  const [above, setAbove] = useState<string | null>(item?.alert_above ?? null);
  const [below, setBelow] = useState<string | null>(item?.alert_below ?? null);
  const [notes, setNotes] = useState(item?.notes ?? '');
  const [submitted, setSubmitted] = useState(false);
  const base = useBaseCurrency();
  const currency =
    item?.asset.currency ?? assets.data?.find((a) => a.id === assetId)?.currency ?? base;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (!assetId) return;
    watch.mutate(
      {
        asset_id: assetId,
        ...(target ? { target_price: target } : {}),
        ...(above ? { alert_above: above } : {}),
        ...(below ? { alert_below: below } : {}),
        ...(notes.trim() ? { notes: notes.trim() } : {}),
      },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title: item ? t('ativos-watch-atualizada') : t('ativos-watch-adicionado'),
          });
          onDone();
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('ativos-aba-watchlist')}
    >
      {!item ? (
        <>
          <Field label={t('ativos-form-buscar')} hint={t('ativos-watch-buscar-dica')}>
            <AssetPicker onSelect={(a) => setAssetId(a.id)} />
          </Field>
          <Field
            label={t('ativos-form-rotulo')}
            error={submitted && !assetId ? t('ativos-watch-escolha') : undefined}
          >
            <AssetSelect assets={assets.data ?? []} value={assetId} onChange={setAssetId} />
          </Field>
        </>
      ) : null}
      <div className="grid gap-4 @min-[400px]:grid-cols-3">
        <Field
          label={t('ativos-watch-alvo-moeda', { moeda: currency })}
          hint={t('ativos-watch-alvo-dica')}
        >
          <DecimalInput value={target} onValueChange={setTarget} placeholder="0,00" />
        </Field>
        <Field label={t('ativos-watch-alertar-acima')}>
          <DecimalInput value={above} onValueChange={setAbove} placeholder="—" />
        </Field>
        <Field label={t('ativos-watch-alertar-abaixo')}>
          <DecimalInput value={below} onValueChange={setBelow} placeholder="—" />
        </Field>
      </div>
      <Field label={t('ativos-watch-notas')}>
        <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
      </Field>
      <FormError error={watch.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={watch.isPending}>
          {t('comum-salvar')}
        </Button>
      </div>
    </form>
  );
}

export function WatchlistPanel() {
  const q = useWatchlist();
  const quotes = useQuoteMap();
  const unwatch = useUnwatch();
  const navigate = useNavigate();
  const [editing, setEditing] = useState<WatchlistItem | null>(null);
  const [adding, setAdding] = useState(false);
  const columns: TechColumn<WatchlistItem>[] = [
    {
      key: 'asset',
      header: t('ativos-form-rotulo'),
      width: '22%',
      cell: (w) => (
        <span className="flex flex-col leading-tight">
          <span className="font-medium text-fg">{w.asset.ticker ?? w.asset.name}</span>
          <span className="truncate font-sans text-[11.5px] text-dim">
            {w.asset.ticker ? w.asset.name : w.asset.class_label}
          </span>
        </span>
      ),
      sortValue: (w) => w.asset.ticker ?? w.asset.name,
    },
    {
      key: 'class',
      header: t('ativos-col-classe'),
      cell: (w) => <span className="font-sans text-dim">{w.asset.class_label}</span>,
    },
    {
      key: 'price',
      header: t('ativos-col-cotacao'),
      numeric: true,
      cell: (w) =>
        w.price ? (
          <Private>
            <PriceWithOrigin price={w.price} currency={w.asset.currency} />
          </Private>
        ) : (
          <span className="text-dim">{t('ativos-watch-sem-cotacao')}</span>
        ),
    },
    {
      key: 'today',
      header: t('ativos-watch-hoje'),
      numeric: true,
      cell: (w) => <QuoteChange quote={quotes.get(w.asset.id)} showPrice={false} />,
      sortValue: (w) => quotes.get(w.asset.id)?.change_pct ?? null,
    },
    {
      key: 'target',
      header: t('ativos-watch-alvo'),
      numeric: true,
      cell: (w) => (w.target_price ? formatUnitPrice(w.target_price, w.asset.currency) : '—'),
    },
    {
      key: 'upside',
      header: t('ativos-watch-ate-alvo'),
      numeric: true,
      cell: (w) => <Pct value={w.upside_to_target} tone="auto" />,
      sortValue: (w) => w.upside_to_target,
    },
    {
      key: 'alerts',
      header: t('ativos-watch-alertas'),
      cell: (w) => (
        <span className="flex flex-col text-[12px] leading-tight text-dim">
          {w.alert_above ? (
            <span>
              {t('ativos-watch-acima-de', {
                preco: formatUnitPrice(w.alert_above, w.asset.currency),
              })}
            </span>
          ) : null}
          {w.alert_below ? (
            <span>
              {t('ativos-watch-abaixo-de', {
                preco: formatUnitPrice(w.alert_below, w.asset.currency),
              })}
            </span>
          ) : null}
          {!w.alert_above && !w.alert_below ? '—' : null}
        </span>
      ),
    },
    {
      key: 'triggered',
      header: t('ativos-watch-situacao'),
      cell: (w) =>
        w.triggered.length ? (
          <span className="flex items-center gap-1.5 text-warning">
            <Icon name="alert" size={12} />
            <span className="font-sans text-[12px]">{w.triggered.join('; ')}</span>
          </span>
        ) : (
          <span className="text-dim">—</span>
        ),
      sortValue: (w) => w.triggered.length,
    },
    {
      key: 'notes',
      header: t('ativos-watch-notas'),
      cell: (w) => <span className="font-sans text-[12px] text-dim">{w.notes ?? ''}</span>,
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('ativos-acoes')}</span>,
      align: 'right',
      cell: (w) => (
        <span className="flex justify-end gap-1" onKeyDown={(e) => e.stopPropagation()}>
          <Button size="sm" variant="ghost" onClick={() => setEditing(w)}>
            {t('comum-editar')}
          </Button>
          <ConfirmButton
            label={t('ativos-remover')}
            confirmLabel={t('ativos-remover')}
            title={t('ativos-watch-remover-titulo')}
            description={t('ativos-watch-remover-descricao', {
              nome: w.asset.ticker ?? w.asset.name,
            })}
            pending={unwatch.isPending}
            onConfirm={() =>
              unwatch
                .mutateAsync({ asset_id: w.asset.id })
                .then(() => toast({ tone: 'success', title: t('ativos-watch-removido') }))
            }
          />
        </span>
      ),
    },
  ];
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <span className="text-[12.5px] text-dim">
          {q.data
            ? t('ativos-watch-resumo', {
                n: q.data.length,
                disparados: q.data.filter((w) => w.triggered.length).length,
              })
            : ''}
        </span>
        <Button
          className="ml-auto"
          variant="primary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setAdding(true)}
        >
          {t('ativos-adicionar')}
        </Button>
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState
          title={t('ativos-watch-vazia')}
          description={t('ativos-watch-vazia-descricao')}
        />
      ) : (
        <div className="rounded-card border border-line bg-surface">
          <TechTable
            label={t('ativos-aba-watchlist')}
            prefsId="Watchlist"
            rows={q.data}
            columns={columns}
            getRowId={(w) => w.id}
            onActivate={(w) => void navigate(`/ativos?ativo=${encodeURIComponent(w.asset.id)}`)}
            rowMenuLabel={(w) => t('ativos-acoes-de', { nome: w.asset.ticker ?? w.asset.name })}
            rowMenu={(w) => [
              ...assetMenuItems(w.asset, (path) => void navigate(path), { watched: true }),
              { separator: true },
              {
                label: t('ativos-watch-editar-alvo'),
                icon: 'sliders',
                onSelect: () => setEditing(w),
              },
            ]}
          />
        </div>
      )}
      <WatchDialog open={adding} onOpenChange={setAdding} />
      <WatchDialog
        open={editing != null}
        onOpenChange={(o) => !o && setEditing(null)}
        item={editing}
      />
    </div>
  );
}
