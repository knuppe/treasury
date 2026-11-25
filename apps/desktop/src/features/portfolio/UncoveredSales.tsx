/**
 * Sales without a recorded purchase (`portfolio.uncovered_sales`): an official source (the B3
 * investor area, a brokerage note) reported a sale whose purchase is older than the history
 * in the vault. The sale is recorded as reported and stays out of the average cost, of the
 * realized result and of the tax until the person decides, row by row:
 *
 * - record the purchase (the trade dialog opens with the account, the missing quantity and
 *   the day before the sale; the date and the cost are theirs to fill in);
 * - accept it without a cost (it stops asking and stays out of the results);
 * - delete the sale.
 *
 * Importing the older history takes the rows off the list by itself.
 */
import { useState } from 'react';

import type { UncoveredSale } from '@/api/generated/UncoveredSale';
import { useAcceptUncoveredSale, useDeleteTrades, useUncoveredSales } from '@/api/hooks/portfolio';
import { Button, Money, Private, toast } from '@/design/components';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { useDeleteRecords } from './RecordsPanels';
import { TradeDialog } from './TradeDialog';
import { errorMessage, Notice } from './components/States';
import { formatQuantity } from './decimal';

/** The day before `iso` (`YYYY-MM-DD`): the latest date the missing purchase can have. */
function dayBefore(iso: string): string {
  const d = new Date(`${iso}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() - 1);
  return d.toISOString().slice(0, 10);
}

type SaleRecord = UncoveredSale & { id: string };

function saleText(r: UncoveredSale): string {
  return t('carteira-venda-de', {
    qtd: formatQuantity(r.quantity),
    ativo: r.ticker ?? r.name,
    data: formatDate(r.date),
  });
}

export function UncoveredSalesCard({ assetId }: { assetId?: string }) {
  const q = useUncoveredSales(assetId ? { asset_id: assetId } : {});
  const accept = useAcceptUncoveredSale();
  const [buying, setBuying] = useState<UncoveredSale | null>(null);
  const [showAccepted, setShowAccepted] = useState(false);
  const { ask: askDelete, dialog: deleteDialog } = useDeleteRecords<SaleRecord>(useDeleteTrades(), {
    title: (n) => t('carteira-excluir-operacoes', { n }),
    item: saleText,
    note: () => t('carteira-vendas-excluir-nota'),
    done: (n) => t('carteira-operacoes-excluidas', { n }),
  });

  const data = q.data;
  if (!data || data.rows.length === 0) return null;
  const open = data.rows.filter((r) => !r.accepted);
  const accepted = data.rows.filter((r) => r.accepted);
  if (open.length === 0 && !showAccepted) {
    // Everything was accepted: one quiet line to bring them back.
    return (
      <p className="text-[12.5px] text-dim">
        {t('carteira-sem-compra-aceitas', { n: accepted.length })}{' '}
        <button
          type="button"
          className="text-accent underline-offset-2 hover:underline"
          onClick={() => setShowAccepted(true)}
        >
          {t('carteira-sem-compra-ver-aceitas')}
        </button>
      </p>
    );
  }

  const setAccepted = (r: UncoveredSale, on: boolean) =>
    accept.mutate(
      { trade_id: r.trade_id, accepted: on },
      {
        onSuccess: () =>
          toast({
            tone: 'success',
            title: on ? t('carteira-sem-compra-aceita') : t('carteira-sem-compra-reaberta'),
          }),
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('carteira-sem-compra-erro'),
            description: errorMessage(e),
          }),
      },
    );

  const row = (r: UncoveredSale) => (
    <li
      key={r.trade_id}
      className="flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-line py-2 first:border-t-0"
    >
      <span className="flex min-w-48 flex-1 flex-col">
        <span className="text-[13px] text-fg">
          <span className="num">{formatDate(r.date)}</span> · {r.ticker ?? r.name}
        </span>
        <span className="text-[11.5px] text-dim">
          {r.account_name} ·{' '}
          <Private>
            {t('carteira-sem-compra-falta', {
              falta: formatQuantity(r.missing_quantity),
              total: formatQuantity(r.quantity),
            })}
          </Private>{' '}
          · <Money cents={r.missing_value} currency={r.currency} />
        </span>
      </span>
      <span className="flex flex-wrap items-center gap-2">
        {r.accepted ? (
          <Button
            size="sm"
            variant="ghost"
            loading={accept.isPending}
            onClick={() => setAccepted(r, false)}
          >
            {t('carteira-sem-compra-reabrir')}
          </Button>
        ) : (
          <>
            <Button size="sm" variant="secondary" onClick={() => setBuying(r)}>
              {t('carteira-sem-compra-informar')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              loading={accept.isPending}
              onClick={() => setAccepted(r, true)}
            >
              {t('carteira-sem-compra-aceitar')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => askDelete([{ ...r, id: r.trade_id }])}>
              {t('carteira-sem-compra-excluir')}
            </Button>
          </>
        )}
      </span>
    </li>
  );

  return (
    <>
      <Notice
        tone={open.length > 0 ? 'warning' : 'info'}
        title={t('carteira-sem-compra-titulo', { n: open.length })}
      >
        <p>{t('carteira-sem-compra-texto')}</p>
        <ul className="mt-2 flex flex-col" aria-label={t('carteira-sem-compra-lista')}>
          {open.map(row)}
          {showAccepted ? accepted.map(row) : null}
        </ul>
        {accepted.length > 0 ? (
          <button
            type="button"
            className="mt-1 text-accent underline-offset-2 hover:underline"
            onClick={() => setShowAccepted((v) => !v)}
          >
            {showAccepted
              ? t('carteira-sem-compra-ocultar-aceitas')
              : t('carteira-sem-compra-mostrar-aceitas', { n: accepted.length })}
          </button>
        ) : null}
      </Notice>
      <TradeDialog
        open={buying != null}
        onOpenChange={(o) => !o && setBuying(null)}
        side="buy"
        {...(buying
          ? {
              assetId: buying.asset_id,
              preset: {
                accountId: buying.account_id,
                date: dayBefore(buying.date),
                quantity: buying.missing_quantity,
              },
            }
          : {})}
      />
      {deleteDialog}
    </>
  );
}
