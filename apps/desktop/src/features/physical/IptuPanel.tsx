/**
 * IPTU do imóvel ou terreno (só em cofres do Brasil; docs/brazil/iptu.md): o carnê de cada ano com
 * o valor venal, os impostos, as taxas, os descontos e as parcelas, o pagamento de cada
 * parcela (ligado ao lançamento sugerido ou por data) e os números derivados que o núcleo
 * calcula: alíquota efetiva, variação contra o ano anterior e o IPCA, valor venal × valor de
 * mercado, cota única × parcelado e o histórico do valor venal.
 */
import { useState } from 'react';

import type { IptuInstallment } from '@/api/generated/IptuInstallment';
import type { IptuView } from '@/api/generated/IptuView';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import { useCountryFeatures } from '@/api/hooks/country';
import { useDeleteIptu, useIptu, usePayIptu, useUnpayIptu } from '@/api/hooks/physical';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, InstrumentCard, KeyValue, Money, TechTable, toast } from '@/design/components';
import { Notice } from '@/features/portfolio/components/States';
import { ConfirmDialog, QueryState } from '@/features/transactions/ui/controls';
import { useT } from '@/i18n';
import { formatDate, formatMoney } from '@/lib/format';

import { IptuDialog, IptuPayDialog } from './IptuDialogs';

/** Imóveis e terrenos têm IPTU. */
export const hasIptu = (asset: PhysicalAssetView) =>
  asset.kind === 'real_estate' || asset.kind === 'land';

const pct = (f: number, digits = 1) =>
  `${(f * 100).toFixed(digits).replace('.', ',').replace('-', '−')}%`;
const signed = (f: number) => `${f >= 0 ? '+' : ''}${pct(f)}`;

export function IptuPanel({ asset }: { asset: PhysicalAssetView }) {
  const features = useCountryFeatures();
  if (!features.iptu || !hasIptu(asset)) return null;
  return <IptuBody asset={asset} />;
}

function IptuBody({ asset }: { asset: PhysicalAssetView }) {
  const t = useT();
  const list = useIptu(asset.id);
  const bills = list.data ?? [];
  const [editing, setEditing] = useState<IptuView | 'new' | null>(null);
  const [paying, setPaying] = useState<{ bill: IptuView; inst: IptuInstallment } | null>(null);
  const [deleting, setDeleting] = useState<IptuView | null>(null);
  const remove = useDeleteIptu();
  const undoToast = useUndoToast();
  const year = new Date().getFullYear();
  const hasThisYear = bills.some((b) => b.year === year);

  return (
    <InstrumentCard
      index="07.10"
      title={t('iptu-titulo')}
      actions={
        <Button size="sm" variant="ghost" onClick={() => setEditing('new')}>
          {hasThisYear ? t('iptu-registrar-outro') : t('iptu-registrar', { ano: String(year) })}
        </Button>
      }
    >
      <div className="flex flex-col gap-4">
        <p className="text-[12.5px] text-dim">{t('iptu-descricao')}</p>
        <QueryState loading={list.isPending} error={list.error} onRetry={() => void list.refetch()}>
          {bills.length === 0 ? (
            <p className="text-[13px] text-dim">{t('iptu-vazio')}</p>
          ) : (
            <>
              {bills.map((b, i) => (
                <BillCard
                  key={b.id}
                  bill={b}
                  open={i === 0}
                  onEdit={() => setEditing(b)}
                  onDelete={() => setDeleting(b)}
                  onPay={(inst) => setPaying({ bill: b, inst })}
                />
              ))}
              {bills.length > 1 ? <History bills={bills} /> : null}
            </>
          )}
        </QueryState>
      </div>
      <IptuDialog
        asset={asset}
        bill={editing === 'new' ? null : editing}
        defaultYear={hasThisYear ? year + 1 : year}
        previous={bills[0] ?? null}
        open={editing != null}
        onOpenChange={(o) => (!o ? setEditing(null) : undefined)}
      />
      {paying ? (
        <IptuPayDialog
          bill={paying.bill}
          installment={paying.inst}
          open
          onOpenChange={(o) => (!o ? setPaying(null) : undefined)}
        />
      ) : null}
      <ConfirmDialog
        open={deleting != null}
        onOpenChange={(o) => (!o ? setDeleting(null) : undefined)}
        title={t('iptu-apagar-titulo', { ano: String(deleting?.year ?? year) })}
        description={t('iptu-apagar-texto')}
        confirmLabel={t('iptu-apagar')}
        pending={remove.isPending}
        error={remove.error}
        onConfirm={() =>
          deleting &&
          remove.mutate(
            { id: deleting.id },
            {
              onSuccess: () => {
                undoToast({ tone: 'success', title: t('iptu-historico-apagado') });
                setDeleting(null);
              },
            },
          )
        }
      />
    </InstrumentCard>
  );
}

function BillCard({
  bill,
  open,
  onEdit,
  onDelete,
  onPay,
}: {
  bill: IptuView;
  open: boolean;
  onEdit: () => void;
  onDelete: () => void;
  onPay: (i: IptuInstallment) => void;
}) {
  const t = useT();
  const pay = usePayIptu();
  const unpay = useUnpayIptu();
  const undoToast = useUndoToast();
  const cur = bill.currency;
  const m = (c: number | null) => (c == null ? '—' : <Money cents={c} currency={cur} />);
  const label = (i: IptuInstallment) =>
    i.number === 0 ? t('iptu-cota-unica') : t('iptu-parcela-n', { n: i.number });
  const status = (i: IptuInstallment) => {
    switch (i.status) {
      case 'paid':
        return (
          <span className="text-positive">
            {t('iptu-situacao-paga', { data: formatDate(i.paid_on ?? i.due_date) })}
          </span>
        );
      case 'late':
        return <span className="text-warning">{t('iptu-situacao-vencida')}</span>;
      case 'not_chosen':
        return <span className="text-dim">{t('iptu-situacao-outra-forma')}</span>;
      default:
        return <span>{t('iptu-situacao-aberta')}</span>;
    }
  };
  const fail = (e: Error) => toast({ tone: 'error', title: e.message });
  const confirmSuggestion = (i: IptuInstallment) =>
    i.suggestion &&
    pay.mutate(
      { installment_id: i.id, txn_id: i.suggestion.txn_id },
      {
        onSuccess: () => undoToast({ tone: 'success', title: t('iptu-pagamento-marcado') }),
        onError: fail,
      },
    );

  const facts = [
    bill.registration ? { label: t('iptu-cadastro'), value: bill.registration } : null,
    bill.city ? { label: t('iptu-cidade'), value: bill.city } : null,
    { label: t('iptu-valor-venal-total'), value: m(bill.venal_total) },
    { label: t('iptu-total-impostos'), value: m(bill.taxes_total) },
    bill.effective_rate != null
      ? { label: t('iptu-aliquota-efetiva'), value: pct(bill.effective_rate, 2) }
      : null,
    { label: t('iptu-total-devido'), value: m(bill.total_due) },
    { label: t('iptu-pago'), value: m(bill.paid) },
    { label: t('iptu-em-aberto'), value: m(bill.open) },
  ].filter((x) => x != null);

  const p = bill.previous;
  const derived = [
    p?.venal_change != null
      ? t('iptu-variacao-venal', { variacao: signed(p.venal_change), ano: String(p.year) })
      : null,
    p?.tax_change != null
      ? t('iptu-variacao-imposto', { variacao: signed(p.tax_change), ano: String(p.year) })
      : null,
    p?.ipca != null ? t('iptu-ipca-ano', { ano: String(p.year), ipca: pct(p.ipca) }) : null,
    bill.market
      ? t('iptu-mercado', {
          proporcao: pct(bill.market.ratio, 0),
          valor: `${bill.market.estimated ? '≈ ' : ''}${formatMoney(bill.market.value, cur)}`,
          data: formatDate(bill.market.as_of),
        })
      : null,
    bill.single_vs_installments
      ? t('iptu-cota-x-parcelas', {
          unica: formatMoney(bill.single_vs_installments.single, cur),
          n: bill.single_vs_installments.count,
          parcelas: formatMoney(bill.single_vs_installments.installments, cur),
        })
      : null,
  ].filter((x): x is string => x != null);

  return (
    <details open={open} className="rounded-card border border-line">
      <summary className="flex cursor-pointer flex-wrap items-center gap-3 px-3 py-2">
        <span className="label-caps">{bill.year}</span>
        <span className="text-[13px]">{m(bill.total_due)}</span>
        {bill.plan ? (
          <span className="text-[12px] text-dim">
            {bill.plan === 'single' ? t('iptu-forma-unica') : t('iptu-forma-parcelado')}
          </span>
        ) : null}
        <span className="ml-auto flex gap-1">
          <Button size="sm" variant="ghost" onClick={onEdit}>
            {t('iptu-editar')}
          </Button>
          <Button size="sm" variant="ghost" onClick={onDelete}>
            {t('iptu-apagar')}
          </Button>
        </span>
      </summary>
      <div className="flex flex-col gap-3 border-t border-line px-3 py-3">
        <KeyValue items={facts} />
        {bill.insights.map((i) => (
          <Notice key={i.kind} tone={i.warning ? 'warning' : 'info'} title={i.text} />
        ))}
        {derived.length > 0 ? (
          <ul className="flex flex-col gap-0.5 text-[12.5px] text-dim">
            {derived.map((d) => (
              <li key={d}>{d}</li>
            ))}
          </ul>
        ) : null}
        {bill.installments.length > 0 ? (
          <TechTable<IptuInstallment>
            label={t('iptu-parcelas-titulo', { ano: String(bill.year) })}
            dense
            customizable={false}
            rows={bill.installments}
            getRowId={(i) => i.id}
            columns={[
              { key: 'n', header: t('iptu-coluna-parcela'), cell: label },
              {
                key: 'due',
                header: t('iptu-coluna-vencimento'),
                cell: (i) => <span className="num">{formatDate(i.due_date)}</span>,
              },
              {
                key: 'amount',
                header: t('iptu-coluna-valor'),
                numeric: true,
                cell: (i) => m(i.amount),
              },
              { key: 'status', header: t('iptu-coluna-situacao'), cell: status },
              {
                key: 'actions',
                header: '',
                cell: (i) =>
                  i.status === 'paid' ? (
                    <Button
                      size="sm"
                      variant="ghost"
                      loading={unpay.isPending}
                      onClick={() =>
                        unpay.mutate(
                          { installment_id: i.id },
                          {
                            onSuccess: () =>
                              undoToast({ tone: 'success', title: t('iptu-pagamento-desfeito') }),
                            onError: fail,
                          },
                        )
                      }
                    >
                      {t('iptu-desfazer-pagamento')}
                    </Button>
                  ) : (
                    <span className="flex flex-wrap items-center justify-end gap-1">
                      {i.suggestion?.named ? (
                        <Button
                          size="sm"
                          variant="primary"
                          loading={pay.isPending}
                          title={t('iptu-sugestao', {
                            descricao: i.suggestion.description,
                            data: formatDate(i.suggestion.date),
                          })}
                          onClick={() => confirmSuggestion(i)}
                        >
                          {t('iptu-confirmar-sugestao')}
                        </Button>
                      ) : null}
                      <Button size="sm" variant="ghost" onClick={() => onPay(i)}>
                        {t('iptu-marcar-pago')}
                      </Button>
                    </span>
                  ),
              },
            ]}
          />
        ) : null}
        {bill.notes ? <p className="text-[12.5px] text-dim">{bill.notes}</p> : null}
      </div>
    </details>
  );
}

function History({ bills }: { bills: IptuView[] }) {
  const t = useT();
  return (
    <TechTable<IptuView>
      label={t('iptu-historico-titulo')}
      dense
      customizable={false}
      rows={bills}
      getRowId={(b) => b.id}
      columns={[
        { key: 'year', header: t('iptu-coluna-ano'), cell: (b) => b.year },
        {
          key: 'venal',
          header: t('iptu-valor-venal-total'),
          numeric: true,
          cell: (b) =>
            b.venal_total == null ? '—' : <Money cents={b.venal_total} currency={b.currency} />,
        },
        {
          key: 'taxes',
          header: t('iptu-total-impostos'),
          numeric: true,
          cell: (b) =>
            b.taxes_total == null ? '—' : <Money cents={b.taxes_total} currency={b.currency} />,
        },
        {
          key: 'total',
          header: t('iptu-total-devido'),
          numeric: true,
          cell: (b) => <Money cents={b.total_due} currency={b.currency} />,
        },
      ]}
    />
  );
}
