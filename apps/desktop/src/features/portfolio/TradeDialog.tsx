/**
 * Registrar (ou editar) uma operação de compra/venda: ativo, conta da corretora,
 * data, quantidade e preço em decimal exato, custos e impostos em centavos, câmbio
 * para ativos em outra moeda e a movimentação no caixa da corretora. Uma venda
 * maior que a posição na conta na data é barrada antes de chegar ao núcleo (que
 * também recusa).
 */
import { useMemo, useState, type FormEvent } from 'react';

import { useAssets } from '@/api/hooks/assets';
import {
  useCreateTrade,
  useCustodyAccounts,
  usePosition,
  useUpdateTrade,
} from '@/api/hooks/portfolio';
import type { Trade } from '@/api/generated/Trade';
import type { TradePatch } from '@/api/generated/TradePatch';
import type { TradeSide } from '@/api/generated/TradeSide';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  toast,
} from '@/design/components';
import { t } from '@/i18n';
import { currencyPrefix, formatDate, formatMoney, formatPercent, todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';

import { AssetPicker } from '../market/AssetPicker';
import { Check, AccountSelect, AssetSelect } from './components/Fields';
import { DecimalInput } from './components/DecimalInput';
import { Segmented } from './components/Segmented';
import {
  addDecimal,
  compareDecimal,
  divideDecimal,
  multiplyDecimal,
  formatDecimal,
  formatQuantity,
  formatUnitPrice,
  isPositiveDecimal,
  minorToDecimal,
  productToMinor,
} from './decimal';
import { assetLabel } from './labels';

export interface TradeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Ativo já escolhido (página do ativo, barra de comando). */
  assetId?: string;
  side?: TradeSide;
  /** Editar esta operação (ativo e conta não mudam numa edição). */
  trade?: Trade | null;
}

interface FormState {
  assetId: string | undefined;
  accountId: string | undefined;
  side: TradeSide;
  date: string | null;
  quantity: string | null;
  price: string | null;
  fees: number | null;
  taxes: number | null;
  fxRate: string | null;
  dayTrade: boolean;
  noteNumber: string;
  notes: string;
  cashMovement: boolean;
  /** Renda fixa pelo valor total (bruto da operação, em centavos) em vez de qtd × PU. */
  byTotal: boolean;
  total: number | null;
  /** Venda: resgata a posição inteira na conta (a quantidade vem da posição). */
  whole: boolean;
}

function initialState(props: TradeDialogProps): FormState {
  const t = props.trade;
  if (t) {
    return {
      assetId: t.asset_id,
      accountId: t.account_id,
      side: t.side,
      date: t.date,
      quantity: t.quantity,
      price: t.price,
      fees: t.fees,
      taxes: t.taxes,
      fxRate: t.fx_rate,
      dayTrade: t.day_trade,
      noteNumber: t.note_number ?? '',
      notes: t.notes ?? '',
      cashMovement: t.txn_id != null,
      byTotal: false,
      total: t.gross,
      whole: false,
    };
  }
  return {
    assetId: props.assetId,
    accountId: undefined,
    side: props.side ?? 'buy',
    date: todayIso(),
    quantity: null,
    price: null,
    fees: 0,
    taxes: 0,
    fxRate: null,
    dayTrade: false,
    noteNumber: '',
    notes: '',
    cashMovement: true,
    byTotal: true,
    total: null,
    whole: true,
  };
}

export function TradeDialog(props: TradeDialogProps) {
  const { open, onOpenChange } = props;
  // O formulário é recriado a cada abertura (estado inicial limpo).
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="06"
      title={props.trade ? t('carteira-op-editar') : t('carteira-op-registrar')}
      description={props.trade ? t('carteira-op-editar-texto') : t('carteira-op-registrar-texto')}
    >
      {open ? <TradeForm {...props} /> : null}
    </Dialog>
  );
}

function TradeForm(props: TradeDialogProps) {
  const editing = Boolean(props.trade);
  const [f, setF] = useState<FormState>(() => initialState(props));
  const [submitted, setSubmitted] = useState(false);
  const set = <K extends keyof FormState>(k: K, v: FormState[K]) => setF((s) => ({ ...s, [k]: v }));

  const assetsQ = useAssets({ include_archived: editing });
  // Corretoras e bancos (CDB, LCI/LCA, fundos do banco); a conta onde o ativo fica vem
  // primeiro e é a padrão.
  const accountsQ = useCustodyAccounts();
  const create = useCreateTrade();
  const update = useUpdateTrade();
  const mutation = editing ? update : create;

  const asset = assetsQ.data?.find((a) => a.id === f.assetId);
  const accounts = useMemo(() => {
    const all = accountsQ.data ?? [];
    const rank = (k: string) => (k === 'brokerage' || k === 'foreign' ? 0 : 1);
    return [...all].sort(
      (x, y) =>
        Number(y.id === asset?.account_id) - Number(x.id === asset?.account_id) ||
        rank(x.kind) - rank(y.kind),
    );
  }, [accountsQ.data, asset?.account_id]);
  const base = useBaseCurrency();
  const currency = asset?.currency ?? base;
  const foreign = currency !== base;
  const accountId =
    f.accountId ||
    (asset
      ? (
          accounts.find((a) => a.id === asset.account_id) ??
          accounts.find((a) => a.currency === currency && a.kind === 'brokerage') ??
          accounts.find((a) => a.currency === currency) ??
          accounts[0]
        )?.id
      : undefined);
  const account = accounts.find((a) => a.id === accountId);

  // Posição na conta na data (para barrar venda maior que a posição).
  const positionQ = usePosition(
    f.side === 'sell' && f.assetId && accountId && f.date
      ? { asset_id: f.assetId, account_id: accountId, as_of: f.date, deflator: 'none' }
      : null,
  );
  const held = positionQ.data?.quantity ?? null;
  // Na edição de uma venda, a própria operação já saiu da posição.
  const heldForSale =
    held != null && editing && props.trade?.side === 'sell' && props.trade.date <= (f.date ?? '')
      ? addDecimal(held, props.trade.quantity)
      : held;

  // Renda fixa pelo valor total: compra = valor × PU 1 (como o CDB é registrado); venda =
  // a posição inteira ou, no resgate parcial, a fração do saldo que o valor representa (saldo
  // pela curva na data; sem ele, pelo custo), sempre com o PU que dá o valor bruto. A pessoa
  // nunca informa quantidade: RDB, CDB e afins não têm cotas para ela.
  const totalMode = Boolean(asset?.fixed_income) && f.byTotal;
  const totalOk = f.total != null && f.total > 0;
  const balanceAtDate = positionQ.data?.market_value ?? positionQ.data?.cost ?? null;
  const partialQuantity =
    totalMode &&
    f.side === 'sell' &&
    !f.whole &&
    totalOk &&
    heldForSale != null &&
    isPositiveDecimal(heldForSale) &&
    balanceAtDate != null &&
    balanceAtDate > 0
      ? f.total! >= balanceAtDate
        ? heldForSale
        : divideDecimal(
            multiplyDecimal(heldForSale, minorToDecimal(f.total!, currency)),
            minorToDecimal(balanceAtDate, currency),
            8,
          )
      : null;
  const quantity = !totalMode
    ? f.quantity
    : f.side === 'buy'
      ? totalOk
        ? minorToDecimal(f.total!, currency)
        : null
      : f.whole
        ? heldForSale
        : partialQuantity;
  const price = !totalMode
    ? f.price
    : f.side === 'buy'
      ? '1'
      : totalOk && quantity && isPositiveDecimal(quantity)
        ? divideDecimal(minorToDecimal(f.total!, currency), quantity, 12)
        : null;
  const quantityOk = quantity != null && isPositiveDecimal(quantity);
  const priceOk = price != null && compareDecimal(price, '0') >= 0;
  const saleTooBig =
    f.side === 'sell' &&
    quantityOk &&
    heldForSale != null &&
    compareDecimal(quantity, heldForSale) > 0;
  const fxOk = f.fxRate == null || isPositiveDecimal(f.fxRate);

  const gross = quantityOk && priceOk ? productToMinor([quantity, price], currency) : null;
  const costs = (f.fees ?? 0) + (f.taxes ?? 0);
  const cash = gross == null ? null : f.side === 'buy' ? -(gross + costs) : gross - costs;
  const grossBase =
    gross != null && foreign && f.fxRate && fxOk
      ? productToMinor([quantity!, price!, f.fxRate], base)
      : null;

  const errors = {
    asset: !f.assetId ? t('carteira-erro-escolha-ativo') : null,
    account: !accountId ? t('carteira-erro-escolha-conta-corretora') : null,
    date: !f.date ? t('carteira-erro-data') : null,
    total: totalMode && !totalOk ? t('carteira-erro-valor-total') : null,
    quantity: !quantityOk
      ? totalMode && f.side === 'sell'
        ? f.whole || !totalOk
          ? f.whole
            ? t('carteira-erro-sem-posicao')
            : null
          : t('carteira-erro-sem-posicao')
        : totalMode && f.side === 'buy'
          ? null
          : t('carteira-erro-quantidade')
      : saleTooBig
        ? t('carteira-erro-venda-maior', {
            qtd: formatQuantity(quantity),
            posicao: formatQuantity(heldForSale),
            data: f.date ? formatDate(f.date) : t('carteira-na-data'),
          })
        : null,
    price: !priceOk && !totalMode ? t('carteira-erro-preco') : null,
    fees: f.fees == null ? t('carteira-valor-invalido') : null,
    taxes: f.taxes == null ? t('carteira-valor-invalido') : null,
    fx: !fxOk ? t('carteira-erro-cambio') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? e : null) ?? undefined;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid || !f.assetId || !accountId || !f.date || !quantity || !price) return;
    const done = (title: string) => {
      toast({
        tone: 'success',
        title,
        description: asset
          ? f.side === 'buy'
            ? t('carteira-compra-qtd', {
                qtd: formatQuantity(quantity),
                ativo: assetLabel(asset),
              })
            : t('carteira-venda-qtd', { qtd: formatQuantity(quantity), ativo: assetLabel(asset) })
          : undefined,
      });
      props.onOpenChange(false);
    };
    if (props.trade) {
      const old = props.trade;
      const patch: TradePatch = { id: old.id };
      if (f.date !== old.date) patch.date = f.date;
      if (f.side !== old.side) patch.side = f.side;
      if (quantity !== old.quantity) patch.quantity = quantity;
      if (price !== old.price) patch.price = price;
      if ((f.fees ?? 0) !== old.fees) patch.fees = f.fees ?? 0;
      if ((f.taxes ?? 0) !== old.taxes) patch.taxes = f.taxes ?? 0;
      if (f.fxRate !== old.fx_rate) patch.fx_rate = f.fxRate;
      if (f.dayTrade !== old.day_trade) patch.day_trade = f.dayTrade;
      if (f.noteNumber.trim() !== (old.note_number ?? ''))
        patch.note_number = f.noteNumber.trim() || null;
      if (f.notes.trim() !== (old.notes ?? '')) patch.notes = f.notes.trim() || null;
      if (f.cashMovement !== (old.txn_id != null)) patch.cash_movement = f.cashMovement;
      update.mutate(patch, { onSuccess: () => done(t('carteira-op-alterada')) });
      return;
    }
    create.mutate(
      {
        asset_id: f.assetId,
        account_id: accountId,
        date: f.date,
        side: f.side,
        quantity,
        price,
        fees: f.fees ?? 0,
        taxes: f.taxes ?? 0,
        ...(foreign && f.fxRate ? { fx_rate: f.fxRate } : {}),
        ...(f.dayTrade ? { day_trade: true } : {}),
        ...(f.noteNumber.trim() ? { note_number: f.noteNumber.trim() } : {}),
        ...(f.notes.trim() ? { notes: f.notes.trim() } : {}),
        cash_movement: f.cashMovement,
      },
      { onSuccess: () => done(t('carteira-op-registrada')) },
    );
  };

  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('carteira-op-form')}
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <Segmented<TradeSide>
          label={t('carteira-op-tipo')}
          value={f.side}
          onChange={(v) => set('side', v)}
          options={[
            { value: 'buy', label: `▲ ${t('carteira-compra')}` },
            { value: 'sell', label: `▼ ${t('carteira-venda')}` },
          ]}
        />
        {asset?.fixed_income ? (
          <Segmented<'total' | 'unit'>
            label={t('carteira-op-modo')}
            value={f.byTotal ? 'total' : 'unit'}
            onChange={(v) => set('byTotal', v === 'total')}
            options={[
              { value: 'total', label: t('carteira-op-modo-total') },
              { value: 'unit', label: t('carteira-op-modo-unitario') },
            ]}
          />
        ) : null}
      </div>

      {!editing ? (
        <Field label={t('carteira-buscar-ativo')} hint={t('carteira-buscar-ativo-dica')}>
          <AssetPicker
            onSelect={(a) =>
              setF((s) => ({ ...s, assetId: a.id, accountId: undefined, fxRate: null }))
            }
          />
        </Field>
      ) : null}
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('carteira-ativo')} error={show(errors.asset)}>
          <AssetSelect
            assets={assetsQ.data ?? []}
            value={f.assetId}
            onChange={(v) =>
              setF((s) => ({ ...s, assetId: v, accountId: undefined, fxRate: null }))
            }
            disabled={editing}
          />
        </Field>
        <Field
          label={t('carteira-conta-operacao')}
          error={show(errors.account)}
          hint={
            account && account.currency !== currency
              ? t('carteira-conta-moeda', { conta: account.currency, ativo: currency })
              : undefined
          }
        >
          <AccountSelect
            accounts={accounts}
            value={accountId}
            onChange={(v) => set('accountId', v)}
            disabled={editing}
          />
        </Field>
        <Field label={t('carteira-data-pregao')} error={show(errors.date)}>
          <DateInput value={f.date} onValueChange={(v) => set('date', v)} />
        </Field>
        {totalMode ? (
          <>
            <Field
              label={
                f.side === 'buy' ? t('carteira-valor-aplicado') : t('carteira-valor-bruto-resgate')
              }
              error={show(errors.total)}
              hint={f.side === 'sell' ? t('carteira-valor-bruto-resgate-dica') : undefined}
            >
              <MoneyInput
                value={f.total}
                onValueChange={(v) => set('total', v)}
                currency={currency}
              />
            </Field>
            {f.side === 'sell' ? (
              <div className="flex flex-col gap-2">
                <Check
                  checked={f.whole}
                  onChange={(v) => set('whole', v)}
                  label={t('carteira-resgate-total')}
                  hint={
                    balanceAtDate != null
                      ? t('carteira-saldo-na-data', {
                          valor: formatMoney(balanceAtDate, currency),
                        })
                      : undefined
                  }
                />
                {!f.whole && partialQuantity && heldForSale && balanceAtDate ? (
                  <p className="text-[12px] text-dim">
                    {t('carteira-resgate-parcial', {
                      pct: formatPercent(Math.min(1, (f.total ?? 0) / balanceAtDate), {
                        decimals: 1,
                      }),
                    })}
                  </p>
                ) : null}
                {errors.quantity && submitted ? (
                  <p className="text-[12px] text-negative">{errors.quantity}</p>
                ) : null}
              </div>
            ) : null}
          </>
        ) : (
          <>
            <Field
              label={t('carteira-quantidade')}
              error={errors.quantity && (submitted || saleTooBig) ? errors.quantity : undefined}
              hint={
                f.side === 'sell' && heldForSale != null && !saleTooBig
                  ? t('carteira-posicao-na-data', { qtd: formatQuantity(heldForSale) })
                  : t('carteira-fracoes')
              }
            >
              <DecimalInput
                value={f.quantity}
                onValueChange={(v) => set('quantity', v)}
                placeholder="0"
              />
            </Field>
            <Field
              label={t('carteira-preco-unitario', { moeda: currency })}
              error={show(errors.price)}
            >
              <DecimalInput
                value={f.price}
                onValueChange={(v) => set('price', v)}
                placeholder={formatDecimal('0', { minDecimals: 2 })}
                prefix={currencyPrefix(currency)}
              />
            </Field>
          </>
        )}
        {foreign ? (
          <Field
            label={t('carteira-cambio-moeda', { moeda: currency, base })}
            error={show(errors.fx)}
            hint={t('carteira-cambio-dica', { base })}
          >
            <DecimalInput
              value={f.fxRate}
              onValueChange={(v) => set('fxRate', v)}
              placeholder={t('carteira-ptax-do-dia', { base })}
            />
          </Field>
        ) : null}
        <Field label={t('carteira-custos-corretagem')} error={show(errors.fees)}>
          <MoneyInput value={f.fees} onValueChange={(v) => set('fees', v)} currency={currency} />
        </Field>
        <Field label={t('carteira-impostos-retidos')} error={show(errors.taxes)}>
          <MoneyInput value={f.taxes} onValueChange={(v) => set('taxes', v)} currency={currency} />
        </Field>
        <Field label={t('carteira-nota-corretagem')} hint={t('carteira-opcional')}>
          <Input value={f.noteNumber} onChange={(e) => set('noteNumber', e.target.value)} />
        </Field>
        <Field label={t('carteira-observacoes')} hint={t('carteira-opcional')}>
          <Input value={f.notes} onChange={(e) => set('notes', e.target.value)} />
        </Field>
      </div>

      <div className="flex flex-col gap-2">
        <Check
          checked={f.dayTrade}
          onChange={(v) => set('dayTrade', v)}
          label={t('carteira-day-trade')}
          hint={t('carteira-day-trade-dica')}
        />
        <Check
          checked={f.cashMovement}
          onChange={(v) => set('cashMovement', v)}
          label={t('carteira-lancar-caixa')}
          hint={
            editing && f.cashMovement && props.trade?.txn_id == null
              ? t('carteira-lancar-caixa-dica-edicao')
              : t('carteira-lancar-caixa-dica')
          }
        />
      </div>

      <div className="rounded-control border border-line bg-bg px-3 py-2.5">
        <KeyValue
          items={[
            {
              label: t('carteira-bruto'),
              value: gross == null ? '—' : <Money cents={gross} currency={currency} />,
            },
            { label: t('carteira-col-custos'), value: <Money cents={costs} currency={currency} /> },
            {
              label: f.side === 'buy' ? t('carteira-sai-caixa') : t('carteira-entra-caixa'),
              value: cash == null ? '—' : <Money cents={cash} currency={currency} tone="auto" />,
            },
            ...(grossBase != null
              ? [
                  {
                    label: t('carteira-bruto-moeda', { moeda: base }),
                    value: <Money cents={grossBase} currency={base} />,
                  },
                ]
              : []),
            ...(quantity && price && quantityOk && priceOk && !totalMode
              ? [
                  {
                    label: t('carteira-preco'),
                    value: `${formatQuantity(quantity)} × ${formatUnitPrice(price, currency)}`,
                  },
                ]
              : []),
          ]}
        />
      </div>

      <FormError error={mutation.error} />

      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          loading={mutation.isPending}
          disabled={submitted && invalid}
        >
          {editing
            ? t('carteira-salvar-alteracoes')
            : f.side === 'buy'
              ? t('carteira-registrar-compra')
              : t('carteira-registrar-venda')}
        </Button>
      </div>
      <span className="sr-only" aria-live="polite">
        {cash != null ? t('carteira-total-valor', { valor: formatMoney(cash, currency) }) : ''}
      </span>
    </form>
  );
}
