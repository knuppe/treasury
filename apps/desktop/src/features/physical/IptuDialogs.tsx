/**
 * Diálogos do IPTU: registrar ou editar o carnê do ano (campos do carnê, a forma de pagamento,
 * a cota única e as parcelas mês a mês) e marcar uma parcela como paga (o lançamento provável
 * ou uma data).
 */
import { useState, type FormEvent } from 'react';

import type { IptuInstallment } from '@/api/generated/IptuInstallment';
import type { IptuInstallmentInput } from '@/api/generated/IptuInstallmentInput';
import type { IptuPlan } from '@/api/generated/IptuPlan';
import type { IptuView } from '@/api/generated/IptuView';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import { useIptuMatches, usePayIptu, useSaveIptu } from '@/api/hooks/physical';
import { useUndoToast } from '@/api/hooks/undoToast';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  Money,
  MoneyInput,
} from '@/design/components';
import { Segmented } from '@/features/transactions/ui/controls';
import { useT } from '@/i18n';
import { formatDate, todayIso } from '@/lib/format';
import { addMonthsToDate } from '@/lib/months';
import { FormError } from '@/vault/FormError';

type PlanChoice = IptuPlan | 'none';

export function IptuDialog({
  asset,
  bill,
  defaultYear,
  previous,
  open,
  onOpenChange,
}: {
  asset: PhysicalAssetView;
  /** Editar este carnê (`null`: novo). */
  bill: IptuView | null;
  defaultYear: number;
  /** O carnê mais recente, para trazer cadastro, município e áreas. */
  previous: IptuView | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useT();
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      title={
        bill
          ? t('iptu-editar-titulo', { ano: String(bill.year), imovel: asset.name })
          : t('iptu-novo-titulo', { imovel: asset.name })
      }
      description={t('iptu-descricao')}
    >
      {open ? (
        <IptuForm
          asset={asset}
          bill={bill}
          defaultYear={defaultYear}
          previous={previous}
          onDone={() => onOpenChange(false)}
        />
      ) : null}
    </Dialog>
  );
}

function IptuForm({
  asset,
  bill,
  defaultYear,
  previous,
  onDone,
}: {
  asset: PhysicalAssetView;
  bill: IptuView | null;
  defaultYear: number;
  previous: IptuView | null;
  onDone: () => void;
}) {
  const t = useT();
  const base = bill ?? previous;
  const single = bill?.installments.find((i) => i.number === 0);
  const parts = (bill?.installments ?? []).filter((i) => i.number > 0);
  const [year, setYear] = useState(String(bill?.year ?? defaultYear));
  const [registration, setRegistration] = useState(base?.registration ?? '');
  const [city, setCity] = useState(base?.city ?? '');
  const [landArea, setLandArea] = useState(base?.land_area?.replace('.', ',') ?? '');
  const [builtArea, setBuiltArea] = useState(base?.built_area?.replace('.', ',') ?? '');
  const [landValue, setLandValue] = useState<number | null>(bill?.land_value ?? null);
  const [buildingValue, setBuildingValue] = useState<number | null>(bill?.building_value ?? null);
  const [landTax, setLandTax] = useState<number | null>(bill?.land_tax ?? null);
  const [buildingTax, setBuildingTax] = useState<number | null>(bill?.building_tax ?? null);
  const [taxesTotal, setTaxesTotal] = useState<number | null>(bill?.taxes_total ?? null);
  const [lighting, setLighting] = useState<number | null>(bill?.lighting_fee || null);
  const [waste, setWaste] = useState<number | null>(bill?.waste_fee || null);
  const [otherFees, setOtherFees] = useState<number | null>(bill?.other_fees || null);
  const [discount, setDiscount] = useState<number | null>(bill?.discount || null);
  const [goodPayer, setGoodPayer] = useState<number | null>(bill?.good_payer_discount || null);
  const [totalDue, setTotalDue] = useState<number | null>(bill?.total_due ?? null);
  const [plan, setPlan] = useState<PlanChoice>(bill?.plan_chosen && bill.plan ? bill.plan : 'none');
  const [singleDue, setSingleDue] = useState<string | null>(single?.due_date ?? null);
  const [singleAmount, setSingleAmount] = useState<number | null>(single?.amount ?? null);
  const [count, setCount] = useState(parts.length > 0 ? String(parts.length) : '');
  const [firstDue, setFirstDue] = useState<string | null>(parts[0]?.due_date ?? null);
  const [partAmount, setPartAmount] = useState<number | null>(parts[0]?.amount ?? null);
  const [notes, setNotes] = useState(bill?.notes ?? '');
  const [submitted, setSubmitted] = useState(false);
  const save = useSaveIptu();
  const undoToast = useUndoToast();

  const taxesSum =
    landTax != null || buildingTax != null ? (landTax ?? 0) + (buildingTax ?? 0) : null;
  const taxes = taxesTotal ?? taxesSum;
  const calculated =
    taxes != null
      ? taxes +
        (lighting ?? 0) +
        (waste ?? 0) +
        (otherFees ?? 0) -
        (discount ?? 0) -
        (goodPayer ?? 0)
      : null;
  const venal =
    landValue != null || buildingValue != null ? (landValue ?? 0) + (buildingValue ?? 0) : null;
  const n = Number.parseInt(count, 10);
  const yearN = Number.parseInt(year, 10);
  const errors = {
    year: !(yearN >= 1990 && yearN <= 2100) ? t('iptu-erro-ano') : null,
    total: totalDue == null ? t('iptu-erro-sem-total') : null,
    count: count.trim() && !(n >= 1 && n <= 24) ? t('iptu-erro-parcela') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? (e ?? undefined) : undefined);

  const installments = (): IptuInstallmentInput[] => {
    const out: IptuInstallmentInput[] = [];
    if (singleDue && singleAmount) {
      out.push({ number: 0, due_date: singleDue, amount: singleAmount });
    }
    if (firstDue && partAmount && n >= 1) {
      // Mesmas parcelas de antes: mantém os vencimentos gravados; senão, mês a mês.
      const same = parts.length === n && firstDue === parts[0]?.due_date;
      for (let k = 0; k < n; k++) {
        const due = same ? parts[k]!.due_date : addMonthsToDate(firstDue, k);
        out.push({ number: k + 1, due_date: due, amount: partAmount });
      }
    }
    return out;
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    save.mutate(
      {
        asset_id: asset.id,
        year: yearN,
        ...(registration.trim() ? { registration: registration.trim() } : {}),
        ...(city.trim() ? { city: city.trim() } : {}),
        ...(landArea.trim() ? { land_area: landArea.trim() } : {}),
        ...(builtArea.trim() ? { built_area: builtArea.trim() } : {}),
        ...(landValue != null ? { land_value: landValue } : {}),
        ...(buildingValue != null ? { building_value: buildingValue } : {}),
        ...(landTax != null ? { land_tax: landTax } : {}),
        ...(buildingTax != null ? { building_tax: buildingTax } : {}),
        ...(taxesTotal != null ? { taxes_total: taxesTotal } : {}),
        ...(lighting != null ? { lighting_fee: lighting } : {}),
        ...(waste != null ? { waste_fee: waste } : {}),
        ...(otherFees != null ? { other_fees: otherFees } : {}),
        ...(discount != null ? { discount } : {}),
        ...(goodPayer != null ? { good_payer_discount: goodPayer } : {}),
        total_due: totalDue!,
        ...(plan !== 'none' ? { plan } : {}),
        installments: installments(),
        ...(notes.trim() ? { notes: notes.trim() } : {}),
      },
      {
        onSuccess: (v) => {
          undoToast({ tone: 'success', title: t('iptu-salvo', { ano: String(v.year) }) });
          onDone();
        },
      },
    );
  };

  const money = (
    label: string,
    value: number | null,
    set: (v: number | null) => void,
    extra?: { hint?: string; error?: string },
  ) => (
    <Field label={label} hint={extra?.hint} error={extra?.error}>
      <MoneyInput value={value} onValueChange={set} currency={asset.currency} />
    </Field>
  );

  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-5"
      aria-label={t('iptu-titulo')}
    >
      <div className="grid gap-4 sm:grid-cols-3">
        <Field label={t('iptu-ano')} error={show(errors.year)}>
          <Input
            inputMode="numeric"
            value={year}
            disabled={bill != null}
            onChange={(e) => setYear(e.target.value.replace(/\D/g, '').slice(0, 4))}
          />
        </Field>
        <Field label={t('iptu-cadastro')} hint={t('iptu-cadastro-ajuda')}>
          <Input value={registration} onChange={(e) => setRegistration(e.target.value)} />
        </Field>
        <Field label={t('iptu-cidade')}>
          <Input value={city} onChange={(e) => setCity(e.target.value)} />
        </Field>
      </div>

      <fieldset className="flex flex-col gap-3">
        <legend className="label-caps mb-2">{t('iptu-secao-venal')}</legend>
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label={t('iptu-area-terreno')}>
            <Input
              inputMode="decimal"
              value={landArea}
              onChange={(e) => setLandArea(e.target.value)}
            />
          </Field>
          <Field label={t('iptu-area-construida')}>
            <Input
              inputMode="decimal"
              value={builtArea}
              onChange={(e) => setBuiltArea(e.target.value)}
            />
          </Field>
          {money(t('iptu-valor-venal-terreno'), landValue, setLandValue)}
          {money(t('iptu-valor-venal-predial'), buildingValue, setBuildingValue)}
        </div>
        {venal != null ? (
          <p className="text-[12.5px] text-dim">
            {t('iptu-valor-venal-total')}: <Money cents={venal} currency={asset.currency} />
          </p>
        ) : null}
      </fieldset>

      <fieldset className="flex flex-col gap-3">
        <legend className="label-caps mb-2">{t('iptu-secao-impostos')}</legend>
        <div className="grid gap-4 sm:grid-cols-3">
          {money(t('iptu-imposto-territorial'), landTax, setLandTax)}
          {money(t('iptu-imposto-predial'), buildingTax, setBuildingTax)}
          <Field label={t('iptu-total-impostos')}>
            <MoneyInput
              value={taxesTotal ?? taxesSum}
              onValueChange={setTaxesTotal}
              currency={asset.currency}
            />
          </Field>
          {money(t('iptu-taxa-iluminacao'), lighting, setLighting)}
          {money(t('iptu-taxa-lixo'), waste, setWaste)}
          {money(t('iptu-outras-taxas'), otherFees, setOtherFees)}
          {money(t('iptu-desconto'), discount, setDiscount)}
          {money(t('iptu-desconto-bom-pagador'), goodPayer, setGoodPayer)}
          <Field label={t('iptu-total-devido')} error={show(errors.total)}>
            <div className="flex items-center gap-2">
              <MoneyInput value={totalDue} onValueChange={setTotalDue} currency={asset.currency} />
              {calculated != null && calculated !== totalDue ? (
                <Button
                  size="sm"
                  variant="ghost"
                  type="button"
                  onClick={() => setTotalDue(calculated)}
                >
                  {t('iptu-total-calcular')}
                </Button>
              ) : null}
            </div>
          </Field>
        </div>
      </fieldset>

      <fieldset className="flex flex-col gap-3">
        <legend className="label-caps mb-2">{t('iptu-secao-pagamento')}</legend>
        <Segmented<PlanChoice>
          label={t('iptu-forma')}
          value={plan}
          onChange={setPlan}
          options={[
            { value: 'none', label: t('iptu-forma-indefinida') },
            { value: 'single', label: t('iptu-forma-unica') },
            { value: 'installments', label: t('iptu-forma-parcelado') },
          ]}
        />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label={t('iptu-cota-unica-vencimento')}>
            <DateInput value={singleDue} onValueChange={setSingleDue} />
          </Field>
          {money(t('iptu-cota-unica-valor'), singleAmount, setSingleAmount)}
        </div>
        <div className="grid gap-4 sm:grid-cols-3">
          <Field label={t('iptu-parcelas-quantidade')} error={show(errors.count)}>
            <Input
              inputMode="numeric"
              value={count}
              onChange={(e) => setCount(e.target.value.replace(/\D/g, '').slice(0, 2))}
            />
          </Field>
          <Field label={t('iptu-parcelas-primeiro')}>
            <DateInput value={firstDue} onValueChange={setFirstDue} />
          </Field>
          {money(t('iptu-parcelas-valor'), partAmount, setPartAmount)}
        </div>
        <p className="text-[12px] text-dim">{t('iptu-parcelas-ajuda')}</p>
      </fieldset>

      <Field label={t('iptu-notas')}>
        <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
      </Field>
      <FormError error={save.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('iptu-cancelar')}</Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          loading={save.isPending}
          disabled={submitted && invalid}
        >
          {t('iptu-salvar')}
        </Button>
      </div>
    </form>
  );
}

export function IptuPayDialog({
  bill,
  installment,
  open,
  onOpenChange,
}: {
  bill: IptuView;
  installment: IptuInstallment;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useT();
  const matches = useIptuMatches(open ? installment.id : null);
  const list = matches.data ?? [];
  const [pick, setPick] = useState<string | null>(null);
  const [date, setDate] = useState<string | null>(todayIso());
  const chosen = pick ?? list[0]?.txn_id ?? 'none';
  const pay = usePayIptu();
  const undoToast = useUndoToast();
  const part =
    installment.number === 0
      ? t('iptu-cota-unica')
      : t('iptu-parcela-n', { n: installment.number });
  const submit = () =>
    pay.mutate(
      chosen === 'none'
        ? { installment_id: installment.id, paid_on: date ?? todayIso() }
        : { installment_id: installment.id, txn_id: chosen },
      {
        onSuccess: () => {
          undoToast({ tone: 'success', title: t('iptu-pagamento-marcado') });
          onOpenChange(false);
        },
      },
    );
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('iptu-pagar-titulo', { parcela: part.toLowerCase() })}
      description={`${bill.year} · ${formatDate(installment.due_date)}`}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('iptu-cancelar')}
          </Button>
          <Button variant="primary" loading={pay.isPending} onClick={submit}>
            {t('iptu-marcar-pago')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3">
        <p className="text-[13px]">
          <Money cents={installment.amount} currency={bill.currency} />
        </p>
        <p className="label-caps">{t('iptu-lancamentos-provaveis')}</p>
        {list.length === 0 && !matches.isPending ? (
          <p className="text-[12.5px] text-dim">{t('iptu-nenhum-lancamento')}</p>
        ) : null}
        <ul
          role="radiogroup"
          aria-label={t('iptu-lancamentos-provaveis')}
          className="flex flex-col gap-1.5"
        >
          {list.map((m) => (
            <li key={m.txn_id}>
              <label className="flex items-center gap-2 text-[13px]">
                <input
                  type="radio"
                  name="iptu-pay"
                  checked={chosen === m.txn_id}
                  onChange={() => setPick(m.txn_id)}
                />
                <span className="num text-dim">{formatDate(m.date)}</span>
                <span className="min-w-0 flex-1 truncate">{m.description}</span>
                <Money cents={m.amount} currency={bill.currency} />
              </label>
            </li>
          ))}
          <li>
            <label className="flex flex-wrap items-center gap-2 text-[13px]">
              <input
                type="radio"
                name="iptu-pay"
                checked={chosen === 'none'}
                onChange={() => setPick('none')}
              />
              <span>{t('iptu-pago-sem-lancamento')}</span>
              <DateInput
                aria-label={t('iptu-pago-sem-lancamento')}
                value={date}
                onValueChange={setDate}
                className="w-32"
              />
            </label>
          </li>
        </ul>
        <FormError error={pay.error} />
      </div>
    </Dialog>
  );
}
