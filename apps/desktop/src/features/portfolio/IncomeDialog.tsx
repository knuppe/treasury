/**
 * Registrar um provento (dividendos, JCP, rendimento de FII, cupom…): valor bruto em
 * centavos, ou valor por cota × quantidade calculado na hora; imposto retido com o
 * padrão do núcleo (JCP: 15% até 2025 e 17,5% desde 2026, pela data do crédito ou do
 * pagamento; 30% em dividendos de ações americanas).
 */
import { useState, type FormEvent } from 'react';

import { useAssets } from '@/api/hooks/assets';
import { useCreateIncome, useInvestmentAccounts, usePosition } from '@/api/hooks/portfolio';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { IncomeKind } from '@/api/generated/IncomeKind';
import type { IncomeStatus } from '@/api/generated/IncomeStatus';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';

import { AccountSelect, AssetSelect, Check } from './components/Fields';
import { DecimalInput } from './components/DecimalInput';
import { Segmented } from './components/Segmented';
import { formatDecimal, formatQuantity, isPositiveDecimal, productToMinor } from './decimal';
import { INCOME_KIND_LABEL } from './labels';

export function defaultIncomeKind(cls: AssetClass | undefined): IncomeKind {
  if (cls === 'fii') return 'fii_income';
  if (cls === 'fixed_income' || cls === 'treasury_br') return 'coupon';
  return 'dividend';
}

/** Imposto retido padrão do núcleo (`create_income`), em centavos. */
export function defaultWithholding(
  kind: IncomeKind,
  cls: AssetClass | undefined,
  gross: number,
  /** Data do crédito (data-com) ou do pagamento, a primeira (LC 224/2025). */
  on: string,
): number {
  if (kind === 'jcp') return Math.round(gross * (on >= '2026-01-01' ? 0.175 : 0.15));
  if (kind === 'dividend' && (cls === 'stock_us' || cls === 'etf_us' || cls === 'reit')) {
    return Math.round(gross * 0.3);
  }
  return 0;
}

export function IncomeDialog({
  open,
  onOpenChange,
  assetId,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  assetId?: string;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="06"
      title={t('carteira-prov-titulo')}
      description={t('carteira-prov-texto')}
    >
      {open ? <IncomeForm assetId={assetId} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function IncomeForm({ assetId: initialAsset, onDone }: { assetId?: string; onDone: () => void }) {
  const assetsQ = useAssets({ held_only: true });
  const accountsQ = useInvestmentAccounts();
  const create = useCreateIncome();
  const [assetId, setAssetId] = useState(initialAsset);
  const [accountId, setAccountId] = useState<string | undefined>();
  const asset = assetsQ.data?.find((a) => a.id === assetId);
  const [kind, setKind] = useState<IncomeKind | undefined>();
  const effectiveKind = kind ?? defaultIncomeKind(asset?.class);
  const [status, setStatus] = useState<IncomeStatus>('received');
  const [exDate, setExDate] = useState<string | null>(null);
  const [payDate, setPayDate] = useState<string | null>(todayIso());
  const [perShare, setPerShare] = useState<string | null>(null);
  const [quantity, setQuantity] = useState<string | null>(null);
  const [gross, setGross] = useState<number | null>(null);
  const [tax, setTax] = useState<number | null>(null);
  const [fxRate, setFxRate] = useState<string | null>(null);
  const [cash, setCash] = useState(true);
  const [submitted, setSubmitted] = useState(false);

  const accounts = accountsQ.data ?? [];
  const base = useBaseCurrency();
  const currency = asset?.currency ?? base;
  const account =
    accountId ||
    (asset ? (accounts.find((a) => a.currency === currency) ?? accounts[0])?.id : undefined);

  // Quantidade na data-com (ou no pagamento), como base do cálculo por cota.
  const positionQ = usePosition(
    assetId && (exDate ?? payDate)
      ? { asset_id: assetId, as_of: (exDate ?? payDate)!, deflator: 'none' }
      : null,
  );
  const baseQty =
    quantity ??
    (positionQ.data && positionQ.data.quantity !== '0' ? positionQ.data.quantity : null);
  const computedGross =
    perShare && baseQty && isPositiveDecimal(perShare)
      ? productToMinor([perShare, baseQty], currency)
      : null;
  const grossValue = gross ?? computedGross;
  const taxValue =
    tax ??
    (grossValue != null
      ? defaultWithholding(
          effectiveKind,
          asset?.class,
          grossValue,
          [exDate, payDate].filter((d): d is string => !!d).sort()[0] ?? todayIso(),
        )
      : 0);

  const errors = {
    asset: !assetId ? t('carteira-erro-escolha-ativo') : null,
    account: !account ? t('carteira-erro-escolha-conta') : null,
    pay: !payDate ? t('carteira-erro-data-pagamento') : null,
    gross: grossValue == null || grossValue <= 0 ? t('carteira-erro-bruto') : null,
    tax:
      grossValue != null && (taxValue < 0 || taxValue > grossValue)
        ? t('carteira-erro-imposto')
        : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? e : null) ?? undefined;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid || !assetId || !account || !payDate || grossValue == null) return;
    create.mutate(
      {
        asset_id: assetId,
        account_id: account,
        kind: effectiveKind,
        pay_date: payDate,
        gross: grossValue,
        status,
        ...(exDate ? { ex_date: exDate } : {}),
        ...(baseQty ? { quantity: baseQty } : {}),
        ...(tax != null ? { tax_withheld: tax } : {}),
        ...(currency !== base && fxRate ? { fx_rate: fxRate } : {}),
        cash_movement: status === 'received' && cash,
      },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title:
              status === 'received'
                ? t('carteira-prov-registrado')
                : t('carteira-prov-anunciado-registrado'),
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
      aria-label={t('carteira-prov-form')}
    >
      <Segmented<IncomeStatus>
        label={t('carteira-prov-situacao')}
        value={status}
        onChange={setStatus}
        options={[
          { value: 'received', label: t('carteira-recebido') },
          { value: 'announced', label: t('carteira-anunciado') },
        ]}
      />
      <div className="grid gap-4 sm:grid-cols-2">
        <Field
          label={t('carteira-ativo')}
          error={show(errors.asset)}
          hint={t('carteira-ativos-com-operacoes')}
        >
          <AssetSelect
            assets={assetsQ.data ?? []}
            value={assetId}
            onChange={(v) => {
              setAssetId(v);
              setKind(undefined);
              setAccountId(undefined);
            }}
          />
        </Field>
        <Field label={t('carteira-conta')} error={show(errors.account)}>
          <AccountSelect accounts={accounts} value={account} onChange={setAccountId} />
        </Field>
        <Field label={t('carteira-tipo')}>
          <Select
            value={effectiveKind}
            onValueChange={(v) => setKind(v as IncomeKind)}
            options={Object.entries(INCOME_KIND_LABEL).map(([value, label]) => ({ value, label }))}
          />
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label={t('carteira-data-com')} hint={t('carteira-opcional')}>
            <DateInput value={exDate} onValueChange={setExDate} />
          </Field>
          <Field label={t('carteira-pagamento')} error={show(errors.pay)}>
            <DateInput value={payDate} onValueChange={setPayDate} />
          </Field>
        </div>
        <Field
          label={t('carteira-valor-por-cota', { moeda: currency })}
          hint={t('carteira-valor-por-cota-dica')}
        >
          <DecimalInput
            value={perShare}
            onValueChange={setPerShare}
            placeholder={formatDecimal('0', { minDecimals: 2 })}
          />
        </Field>
        <Field
          label={t('carteira-quantidade-base')}
          hint={
            positionQ.data && !quantity
              ? t('carteira-na-data-qtd', { qtd: formatQuantity(positionQ.data.quantity) })
              : t('carteira-cotas-data-com')
          }
        >
          <DecimalInput
            value={quantity}
            onValueChange={setQuantity}
            placeholder={positionQ.data?.quantity ? formatQuantity(positionQ.data.quantity) : '0'}
          />
        </Field>
        <Field
          label={t('carteira-valor-bruto')}
          error={show(errors.gross)}
          hint={computedGross != null && gross == null ? t('carteira-bruto-calculado') : undefined}
        >
          <MoneyInput value={grossValue} onValueChange={setGross} currency={currency} />
        </Field>
        <Field
          label={t('carteira-imposto-retido')}
          error={show(errors.tax)}
          hint={tax == null ? t('carteira-imposto-padrao') : undefined}
        >
          <MoneyInput value={taxValue} onValueChange={setTax} currency={currency} />
        </Field>
        {currency !== base ? (
          <Field
            label={t('carteira-cambio-moeda', { moeda: currency, base })}
            hint={t('carteira-cambio-pagamento', { base })}
          >
            <DecimalInput
              value={fxRate}
              onValueChange={setFxRate}
              placeholder={base === 'BRL' ? 'PTAX' : undefined}
            />
          </Field>
        ) : null}
      </div>
      {status === 'received' ? (
        <Check checked={cash} onChange={setCash} label={t('carteira-lancar-credito')} />
      ) : null}
      <div className="rounded-control border border-line bg-bg px-3 py-2.5">
        <KeyValue
          items={[
            {
              label: t('carteira-bruto'),
              value: grossValue == null ? '—' : <Money cents={grossValue} currency={currency} />,
            },
            { label: t('carteira-imposto'), value: <Money cents={taxValue} currency={currency} /> },
            {
              label: t('carteira-liquido'),
              value:
                grossValue == null ? (
                  '—'
                ) : (
                  <Money cents={grossValue - taxValue} currency={currency} tone="positive" />
                ),
            },
          ]}
        />
      </div>
      <FormError error={create.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={create.isPending}>
          {t('carteira-prov-titulo')}
        </Button>
      </div>
    </form>
  );
}
