/**
 * Proventos a confirmar (`portfolio.pending_income`): proventos anunciados pela fonte de
 * mercado que a família tinha direito (posição na data-com) e ninguém lançou. Confirmar
 * grava com `portfolio.create_income` (o `input` vem pronto; a data de pagamento é
 * editável quando a fonte não a informou). Dispensar esconde só nesta sessão.
 */
import { useState } from 'react';
import { create } from 'zustand';

import type { PendingIncome } from '@/api/generated/PendingIncome';
import { useCreateIncome, useInvestmentAccounts, usePendingIncome } from '@/api/hooks/portfolio';
import {
  Button,
  DateInput,
  InstrumentCard,
  Money,
  OriginBadge,
  Private,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';

import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { ErrorBlock } from '../portfolio/components/States';
import { formatQuantity, formatUnitPrice } from '../portfolio/decimal';
import { INCOME_KIND_LABEL, safeDate, sourceName } from '../portfolio/labels';

/** Chave do provento pendente (evento × conta). */
export const pendingKey = (p: Pick<PendingIncome, 'dividend_event_id' | 'account_id'>) =>
  `${p.dividend_event_id}:${p.account_id}`;

/** Dispensados nesta sessão (some ao reiniciar o app). */
export const useDismissedIncome = create<{
  keys: ReadonlySet<string>;
  dismiss: (k: string) => void;
  clear: () => void;
}>()((set) => ({
  keys: new Set(),
  dismiss: (k) => set((s) => ({ keys: new Set([...s.keys, k]) })),
  clear: () => set({ keys: new Set() }),
}));

function PayDateCell({
  p,
  value,
  onChange,
}: {
  p: PendingIncome;
  value: string;
  onChange: (d: string) => void;
}) {
  if (!p.pay_date_guessed) return <>{safeDate(p.pay_date)}</>;
  return (
    <span className="flex flex-col gap-0.5">
      <span className="w-[112px]">
        <DateInput
          aria-label={t('ativos-pendentes-data-pagamento', {
            nome: p.asset.ticker ?? p.asset.name,
          })}
          value={value}
          onValueChange={(d) => d && onChange(d)}
        />
      </span>
      <span className="font-sans text-[11px] text-warning italic">
        {t('ativos-pendentes-palpite')}
      </span>
    </span>
  );
}

export function PendingIncomeBlock({ assetId }: { assetId?: string } = {}) {
  const q = usePendingIncome();
  const accounts = useInvestmentAccounts();
  const createIncome = useCreateIncome();
  const dismissed = useDismissedIncome((s) => s.keys);
  const dismiss = useDismissedIncome((s) => s.dismiss);
  const [payDates, setPayDates] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState<string | null>(null);
  const rows = (q.data ?? []).filter(
    (p) => !dismissed.has(pendingKey(p)) && (!assetId || p.asset.id === assetId),
  );
  if (q.isError) return <ErrorBlock error={q.error} title={t('ativos-pendentes-titulo')} />;
  if (!rows.length) return null;
  const accountName = (id: string) =>
    accounts.data?.find((a) => a.id === id)?.name ?? t('ativos-pendentes-conta');

  const confirm = (p: PendingIncome) => {
    const k = pendingKey(p);
    const pay = payDates[k] ?? p.pay_date;
    setBusy(k);
    createIncome.mutate(
      { ...p.input, pay_date: pay },
      {
        onSuccess: () =>
          toast({
            tone: 'success',
            title: t('ativos-pendentes-lancado', { nome: p.asset.ticker ?? p.asset.name }),
            description: t('ativos-pendentes-pagamento-em', {
              data: safeDate(pay),
              conta: accountName(p.account_id),
            }),
          }),
        onError: (e) =>
          toast({ tone: 'error', title: t('ativos-pendentes-nao-lancou'), description: e.message }),
        onSettled: () => setBusy(null),
      },
    );
  };

  const columns: TechColumn<PendingIncome>[] = [
    {
      key: 'asset',
      header: t('ativos-form-rotulo'),
      cell: (p) => (
        <span className="flex flex-col leading-tight">
          <span className="font-medium">{p.asset.ticker ?? p.asset.name}</span>
          <span className="truncate font-sans text-[11.5px] text-dim">
            {INCOME_KIND_LABEL[p.kind]} · {accountName(p.account_id)}
          </span>
        </span>
      ),
    },
    {
      key: 'ex',
      header: t('ativos-anunciados-data-com'),
      width: '96px',
      cell: (p) => safeDate(p.ex_date),
    },
    {
      key: 'pay',
      header: t('ativos-pagamento'),
      width: '130px',
      cell: (p) => (
        <PayDateCell
          p={p}
          value={payDates[pendingKey(p)] ?? p.pay_date}
          onChange={(d) => setPayDates((s) => ({ ...s, [pendingKey(p)]: d }))}
        />
      ),
    },
    {
      key: 'qty',
      header: t('ativos-qtd'),
      numeric: true,
      cell: (p) => <Private>{formatQuantity(p.quantity)}</Private>,
    },
    {
      key: 'per',
      header: t('ativos-por-cota'),
      numeric: true,
      cell: (p) => formatUnitPrice(p.amount_per_share, p.currency),
    },
    {
      key: 'net',
      header: t('ativos-prev-liquido'),
      numeric: true,
      cell: (p) => (
        <span className="flex flex-col items-end leading-tight">
          <Money cents={p.net} currency={p.currency} />
          {p.tax_withheld ? (
            <span className="text-[11px] text-dim">
              {withSlots(t('ativos-pendentes-ir', { valor: slot(0) }), [
                <Money key="ir" cents={p.tax_withheld} currency={p.currency} />,
              ])}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'src',
      header: '',
      width: '28px',
      cell: (p) => (
        <OriginBadge
          source={sourceName(p.source)}
          date={p.ex_date}
          note={t('ativos-pendentes-origem')}
        />
      ),
    },
    {
      key: 'actions',
      header: t('ativos-acoes'),
      width: '200px',
      cell: (p) => (
        <span className="flex items-center gap-1.5">
          <Button
            size="sm"
            variant="primary"
            loading={busy === pendingKey(p)}
            aria-label={t('ativos-pendentes-confirmar-de', {
              nome: p.asset.ticker ?? p.asset.name,
            })}
            onClick={() => confirm(p)}
          >
            {t('ativos-pendentes-confirmar')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('ativos-pendentes-dispensar-de', {
              nome: p.asset.ticker ?? p.asset.name,
            })}
            onClick={() => dismiss(pendingKey(p))}
          >
            {t('ativos-pendentes-dispensar')}
          </Button>
        </span>
      ),
    },
  ];

  return (
    <InstrumentCard
      index="00"
      title={t('ativos-pendentes-titulo-n', { n: rows.length })}
      padded={false}
      footer={<p className="text-[12px] text-dim">{t('ativos-pendentes-ajuda')}</p>}
    >
      <TechTable
        label={t('ativos-pendentes-titulo')}
        prefsId="Proventos a confirmar"
        rows={rows}
        columns={columns}
        getRowId={pendingKey}
        dense
      />
    </InstrumentCard>
  );
}
