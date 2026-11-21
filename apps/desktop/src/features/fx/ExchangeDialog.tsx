/**
 * Operação de câmbio entre contas próprias: conta de origem e de destino (moedas
 * diferentes), o valor conhecido (o que sai ou o que entra), a referência do dia (a PTAX no
 * Brasil), a taxa aplicada (ou o spread), IOF (só no Brasil) e tarifa, com a prévia do total e
 * do VET antes de gravar.
 */
import { useDeferredValue, useState } from 'react';

import type { ExchangeArgs } from '@/api/generated/ExchangeArgs';
import { useAccounts } from '@/api/hooks/accounts';
import { useExchange, useExchangeQuote, useReferenceRate } from '@/api/hooks/fx';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { InlineError, Segmented } from '@/features/transactions/ui/controls';
import { useBaseCurrency } from '@/state/currency';

import { localDecimal } from './decimal';

const decimal = (v: string) => v.replace(/[^\d.,]/g, '');

export function ExchangeDialog({
  open,
  onOpenChange,
  defaultToAccountId,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  defaultToAccountId?: string;
}) {
  const { data: accounts = [] } = useAccounts();
  const [fromId, setFromId] = useState<string>('');
  const [toId, setToId] = useState<string>(defaultToAccountId ?? '');
  const [date, setDate] = useState<string | null>(todayIso());
  const [side, setSide] = useState<'to' | 'from'>('to');
  const [amount, setAmount] = useState<number | null>(null);
  const [rate, setRate] = useState('');
  const [spread, setSpread] = useState('');
  const [iof, setIof] = useState('');
  const [fee, setFee] = useState<number | null>(null);
  const [notes, setNotes] = useState('');
  const exchange = useExchange();
  const base = useBaseCurrency();

  const from = accounts.find((a) => a.id === fromId);
  const to = accounts.find((a) => a.id === toId);
  // Moeda da casa = a moeda base da família: a taxa é "1 estrangeira = X da moeda da casa",
  // com o IOF e o VET nela. Entre duas moedas estrangeiras, a taxa é destino por origem e não
  // há câmbio de referência que sirva.
  const foreign =
    from && to
      ? from.currency === base
        ? to.currency
        : to.currency === base
          ? from.currency
          : null
      : null;
  const ref = useReferenceRate(foreign, date);
  // IOF é imposto brasileiro. Típico: 1,1% na remessa para conta própria no exterior, 0,38% no
  // ingresso.
  const brazil = base === 'BRL';
  const iofDefault = !brazil
    ? ''
    : from?.currency === base
      ? localDecimal('1.1')
      : to?.currency === base
        ? localDecimal('0.38')
        : '';
  const effectiveIof = iof || iofDefault;
  const args: ExchangeArgs | null =
    from && to && date && amount && from.currency !== to.currency && (rate || ref.data)
      ? {
          from_account_id: from.id,
          to_account_id: to.id,
          date,
          ...(side === 'to' ? { amount_to: amount } : { amount_from: amount }),
          ...(rate ? { rate } : {}),
          ...(ref.data ? { reference_rate: ref.data.sell, reference_source: ref.data.source } : {}),
          ...(spread && !rate ? { spread_pct: spread } : {}),
          ...(effectiveIof ? { iof_pct: effectiveIof } : {}),
          ...(fee ? { fee } : {}),
          ...(notes.trim() ? { notes: notes.trim() } : {}),
        }
      : null;
  const deferred = useDeferredValue(args);
  const quote = useExchangeQuote(deferred);
  const q = quote.data;
  const options = accounts.map((a) => ({ value: a.id, label: a.name, hint: a.currency }));

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="03"
      size="lg"
      resizable
      resizeId="exchange"
      title={t('cambio-titulo')}
      description={t('cambio-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            disabled={!args || !q}
            loading={exchange.isPending}
            onClick={() =>
              args &&
              exchange.mutate(args, {
                onSuccess: () => {
                  toast({
                    tone: 'success',
                    title: t('cambio-lancado'),
                    description: `${from?.name} → ${to?.name}`,
                  });
                  onOpenChange(false);
                },
              })
            }
          >
            {t('cambio-lancar')}
          </Button>
        </>
      }
    >
      <div className="grid gap-3 @min-[560px]:grid-cols-2">
        <Field label={t('cambio-de-conta')}>
          <Select value={fromId || undefined} onValueChange={setFromId} options={options} />
        </Field>
        <Field label={t('cambio-para-conta')}>
          <Select value={toId || undefined} onValueChange={setToId} options={options} />
        </Field>
        <Field label={t('cambio-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('cambio-valor-conhecido')}>
          <Segmented<'to' | 'from'>
            size="sm"
            label={t('cambio-valor-conhecido')}
            value={side}
            onChange={setSide}
            options={[
              {
                value: 'to',
                label: to
                  ? t('cambio-que-entra-moeda', { moeda: to.currency })
                  : t('cambio-que-entra'),
              },
              {
                value: 'from',
                label: from
                  ? t('cambio-que-sai-moeda', { moeda: from.currency })
                  : t('cambio-que-sai'),
              },
            ]}
          />
        </Field>
        <Field
          label={t('cambio-valor-moeda', { moeda: (side === 'to' ? to : from)?.currency ?? '—' })}
        >
          <MoneyInput
            currency={(side === 'to' ? to : from)?.currency ?? base}
            value={amount}
            onValueChange={setAmount}
          />
        </Field>
        <Field
          label={
            foreign
              ? t('cambio-referencia-moeda', { moeda: foreign, base })
              : t('cambio-referencia', { base })
          }
          hint={
            ref.data
              ? t(ref.data.stale ? 'cambio-referencia-dica-outro-dia' : 'cambio-referencia-dica', {
                  boletim:
                    ref.data.bulletin === 'coletada' ? t('cambio-fechamento') : ref.data.bulletin,
                  data: formatDate(ref.data.date),
                  base,
                })
              : ref.isError
                ? t('cambio-sem-referencia')
                : undefined
          }
        >
          <Input numeric readOnly value={ref.data ? localDecimal(ref.data.sell) : '—'} />
        </Field>
        <Field label={t('cambio-taxa-aplicada-opcional')} hint={t('cambio-taxa-aplicada-dica')}>
          <Input
            numeric
            inputMode="decimal"
            value={rate}
            onChange={(e) => setRate(decimal(e.target.value))}
            placeholder={localDecimal('5.4540')}
          />
        </Field>
        <Field
          label={t('cambio-spread')}
          hint={rate ? t('cambio-spread-calculado') : t('cambio-spread-sobre')}
        >
          <Input
            numeric
            inputMode="decimal"
            disabled={Boolean(rate)}
            value={rate ? (q?.spread_pct ? localDecimal(q.spread_pct) : '') : spread}
            onChange={(e) => setSpread(decimal(e.target.value))}
            placeholder={localDecimal('1.0')}
          />
        </Field>
        {brazil ? (
          <Field label={t('cambio-iof')} hint={t('cambio-iof-dica')}>
            <Input
              numeric
              inputMode="decimal"
              value={iof}
              onChange={(e) => setIof(decimal(e.target.value))}
              placeholder={iofDefault || '0'}
            />
          </Field>
        ) : null}
        <Field label={t('cambio-tarifa-moeda', { moeda: from?.currency ?? '—' })}>
          <MoneyInput currency={from?.currency ?? base} value={fee} onValueChange={setFee} />
        </Field>
        <Field label={t('cambio-observacoes')} className="@min-[560px]:col-span-2">
          <Input
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            placeholder={t('cambio-observacoes-exemplo')}
          />
        </Field>
      </div>
      {from && to && from.currency === to.currency ? (
        <p className="text-[12.5px] text-warning">
          {t('cambio-mesma-moeda', { moeda: from.currency })}
        </p>
      ) : null}
      {q ? (
        <div
          role="status"
          aria-label={t('cambio-previa')}
          className="rounded-card border border-line p-3"
        >
          <KeyValue
            items={[
              { label: t('cambio-taxa-aplicada'), value: localDecimal(q.rate) },
              {
                label: t('cambio-sai'),
                value: <Money cents={q.amount_from} currency={q.from_currency} />,
              },
              ...(brazil || q.iof_amount > 0
                ? [
                    {
                      label: t('cambio-iof-rotulo'),
                      value: <Money cents={q.iof_amount} currency={base} />,
                    },
                  ]
                : []),
              {
                label: t('cambio-tarifa'),
                value: <Money cents={q.fee_amount} currency={q.from_currency} />,
              },
              {
                label: t('cambio-total-sai'),
                value: <Money cents={q.total_from} currency={q.from_currency} />,
              },
              {
                label: t('cambio-entra'),
                value: <Money cents={q.amount_to} currency={q.to_currency} />,
              },
              ...(q.vet
                ? [{ label: t('cambio-vet-rotulo', { base }), value: localDecimal(q.vet) }]
                : []),
            ]}
          />
        </div>
      ) : null}
      <InlineError error={quote.error ?? exchange.error} />
    </Dialog>
  );
}
