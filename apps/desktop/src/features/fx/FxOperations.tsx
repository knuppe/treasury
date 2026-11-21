/** Operações de câmbio de uma conta, com a taxa, os encargos e o VET; e o botão para lançar. */
import { useState } from 'react';

import type { FxOperation } from '@/api/generated/FxOperation';
import { useFxOperations } from '@/api/hooks/fx';
import { Button, Money, TechTable, type TechColumn } from '@/design/components';
import { t } from '@/i18n';
import { baseCurrency, formatDate } from '@/lib/format';

import { localDecimal } from './decimal';
import { ExchangeDialog } from './ExchangeDialog';

/** Colunas da tabela (função: os títulos saem no idioma da hora). */
const columns = (): TechColumn<FxOperation>[] => [
  { key: 'date', header: t('cambio-data'), cell: (o) => formatDate(o.date) },
  {
    key: 'pair',
    header: t('cambio-par'),
    cell: (o) => `${o.from_currency} → ${o.to_currency}`,
  },
  {
    key: 'from',
    header: t('cambio-saiu'),
    numeric: true,
    cell: (o) => <Money cents={o.amount_from} currency={o.from_currency} />,
  },
  {
    key: 'to',
    header: t('cambio-entrou'),
    numeric: true,
    cell: (o) => <Money cents={o.amount_to} currency={o.to_currency} />,
  },
  { key: 'rate', header: t('cambio-taxa'), numeric: true, cell: (o) => localDecimal(o.rate) },
  {
    key: 'costs',
    header: t('cambio-iof-tarifa'),
    numeric: true,
    // O IOF é na moeda da casa (a base da família); a tarifa só soma quando também sai dela.
    cell: (o) => {
      const base = baseCurrency();
      return (
        <Money
          cents={o.iof_amount + (o.from_currency === base ? o.fee_amount : 0)}
          currency={base}
        />
      );
    },
  },
  {
    key: 'vet',
    header: t('cambio-vet'),
    numeric: true,
    cell: (o) => (o.vet ? localDecimal(o.vet) : '—'),
  },
];

export function FxOperations({ accountId }: { accountId: string }) {
  const q = useFxOperations(accountId);
  const [open, setOpen] = useState(false);
  return (
    <section aria-label={t('cambio-operacoes')} className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-3">
        <h3 className="label-caps">{t('cambio-operacoes')}</h3>
        <Button size="sm" variant="secondary" onClick={() => setOpen(true)}>
          {t('cambio-nova-operacao')}
        </Button>
      </div>
      {q.data?.length ? (
        <TechTable
          label={t('cambio-operacoes-conta')}
          prefsId="Operações de câmbio da conta"
          columns={columns()}
          rows={q.data}
          getRowId={(o) => o.id}
        />
      ) : (
        <p className="text-[12.5px] text-dim">{t('cambio-nenhuma-operacao')}</p>
      )}
      {open ? <ExchangeDialog open onOpenChange={setOpen} defaultToAccountId={accountId} /> : null}
    </section>
  );
}
