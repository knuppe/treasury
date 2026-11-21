/**
 * Registrar um evento corporativo: desdobramento, grupamento, bonificação,
 * devolução de capital, mudança de ticker/incorporação (ativo resultante), cisão e
 * subscrição. O motor da carteira ajusta quantidade e preço médio retroativamente.
 */
import { useState, type FormEvent } from 'react';

import { useAssets } from '@/api/hooks/assets';
import { useCreateAction } from '@/api/hooks/portfolio';
import type { CorporateActionKind } from '@/api/generated/CorporateActionKind';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';

import { AssetSelect } from './components/Fields';
import { DecimalInput } from './components/DecimalInput';
import { compareDecimal, formatDecimal } from './decimal';
import { ACTION_KIND_LABEL } from './labels';

const NEEDS_FACTOR: CorporateActionKind[] = ['split', 'reverse_split', 'bonus'];

function factorHint(kind: CorporateActionKind): string | undefined {
  switch (kind) {
    case 'split':
      return t('carteira-ev-fator-split');
    case 'reverse_split':
      return t('carteira-ev-fator-grupamento');
    case 'bonus':
      return t('carteira-ev-fator-bonificacao');
    case 'merger':
      return t('carteira-ev-fator-incorporacao');
    case 'ticker_change':
      return t('carteira-ev-fator-ticker');
    default:
      return undefined;
  }
}

export function ActionDialog({
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
      size="md"
      index="06"
      title={t('carteira-ev-titulo')}
      description={t('carteira-ev-texto')}
    >
      {open ? <ActionForm assetId={assetId} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function factorError(kind: CorporateActionKind, factor: string | null): string | null {
  if (!NEEDS_FACTOR.includes(kind)) return null;
  if (!factor) return t('carteira-ev-erro-fator');
  if (kind === 'split' && compareDecimal(factor, '1') <= 0) return t('carteira-ev-erro-split');
  if (
    kind === 'reverse_split' &&
    (compareDecimal(factor, '0') <= 0 || compareDecimal(factor, '1') >= 0)
  ) {
    return t('carteira-ev-erro-grupamento');
  }
  if (kind === 'bonus' && compareDecimal(factor, '0') <= 0)
    return t('carteira-ev-erro-bonificacao');
  return null;
}

function ActionForm({ assetId: initial, onDone }: { assetId?: string; onDone: () => void }) {
  const assetsQ = useAssets({ include_archived: true });
  const create = useCreateAction();
  const [assetId, setAssetId] = useState(initial);
  const [kind, setKind] = useState<CorporateActionKind>('split');
  const [date, setDate] = useState<string | null>(todayIso());
  const [factor, setFactor] = useState<string | null>(null);
  const [unitCost, setUnitCost] = useState<string | null>(null);
  const [amount, setAmount] = useState<number | null>(null);
  const [newAsset, setNewAsset] = useState<string | undefined>();
  const [notes, setNotes] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const asset = assetsQ.data?.find((a) => a.id === assetId);
  const base = useBaseCurrency();

  const moves = kind === 'ticker_change' || kind === 'merger';
  const errors = {
    asset: !assetId ? t('carteira-erro-escolha-ativo') : null,
    date: !date ? t('carteira-ev-erro-data') : null,
    factor: factorError(kind, factor),
    amount:
      kind === 'capital_return' && (amount == null || amount <= 0)
        ? t('carteira-ev-erro-valor')
        : null,
    newAsset: moves && !newAsset ? t('carteira-ev-erro-ativo-resultante') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? e : null) ?? undefined;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid || !assetId || !date) return;
    create.mutate(
      {
        asset_id: assetId,
        date,
        kind,
        ...(factor && (NEEDS_FACTOR.includes(kind) || moves) ? { factor } : {}),
        ...(kind === 'bonus' && unitCost ? { unit_cost: unitCost } : {}),
        ...(kind === 'capital_return' && amount != null ? { amount } : {}),
        ...(moves && newAsset ? { new_asset_id: newAsset } : {}),
        ...(notes.trim() ? { notes: notes.trim() } : {}),
      },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title: t('carteira-ev-registrado'),
            description: ACTION_KIND_LABEL[kind],
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
      aria-label={t('carteira-evento-corporativo')}
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('carteira-ativo')} error={show(errors.asset)}>
          <AssetSelect assets={assetsQ.data ?? []} value={assetId} onChange={setAssetId} />
        </Field>
        <Field label={t('carteira-tipo')}>
          <Select
            value={kind}
            onValueChange={(v) => setKind(v as CorporateActionKind)}
            options={Object.entries(ACTION_KIND_LABEL).map(([value, label]) => ({ value, label }))}
          />
        </Field>
        <Field label={t('carteira-ev-data')} error={show(errors.date)}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        {NEEDS_FACTOR.includes(kind) || moves ? (
          <Field label={t('carteira-ev-fator')} error={show(errors.factor)} hint={factorHint(kind)}>
            <DecimalInput
              value={factor}
              onValueChange={setFactor}
              placeholder={kind === 'split' ? '2' : formatDecimal('0.1')}
            />
          </Field>
        ) : null}
        {kind === 'bonus' ? (
          <Field
            label={t('carteira-ev-custo-por-acao', { moeda: asset?.currency ?? base })}
            hint={t('carteira-ev-custo-dica')}
          >
            <DecimalInput
              value={unitCost}
              onValueChange={setUnitCost}
              placeholder={formatDecimal('0', { minDecimals: 2 })}
            />
          </Field>
        ) : null}
        {kind === 'capital_return' ? (
          <Field
            label={t('carteira-ev-valor-devolvido')}
            error={show(errors.amount)}
            hint={t('carteira-ev-valor-devolvido-dica')}
          >
            <MoneyInput value={amount} onValueChange={setAmount} />
          </Field>
        ) : null}
        {moves ? (
          <Field
            label={t('carteira-ev-ativo-resultante')}
            error={show(errors.newAsset)}
            hint={t('carteira-ev-ativo-resultante-dica')}
          >
            <AssetSelect
              assets={(assetsQ.data ?? []).filter((a) => a.id !== assetId)}
              value={newAsset}
              onChange={setNewAsset}
            />
          </Field>
        ) : null}
        <Field
          label={t('carteira-observacoes')}
          hint={t('carteira-opcional')}
          className="sm:col-span-2"
        >
          <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
      </div>
      {kind === 'spin_off' || kind === 'subscription' ? (
        <p className="text-[12.5px] text-dim">{t('carteira-ev-cisao-texto')}</p>
      ) : null}
      <FormError error={create.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={create.isPending}>
          {t('carteira-ev-registrar')}
        </Button>
      </div>
    </form>
  );
}
