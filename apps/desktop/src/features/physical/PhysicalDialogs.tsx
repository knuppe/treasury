/**
 * Formulários do patrimônio físico: cadastrar/editar bem (com custos de aquisição, método
 * de avaliação, tabela FIPE e financiamento), registrar benfeitoria ou obra, avaliação,
 * venda e ligar um financiamento. O núcleo valida e calcula; aqui só as checagens de forma.
 */
import { useState, type FormEvent } from 'react';

import { useCountryFeatures } from '@/api/hooks/country';
import { useLoans } from '@/api/hooks/debt';
import { useMembers } from '@/api/hooks/household';
import {
  useAddImprovement,
  useAddValuation,
  useCreatePhysical,
  useFipeOptions,
  useLinkLoan,
  usePhysicalAssets,
  useSellPhysical,
  useUpdateImprovement,
  useUpdatePhysical,
} from '@/api/hooks/physical';
import { useUndoToast } from '@/api/hooks/undoToast';
import type { AcquisitionCostKind } from '@/api/generated/AcquisitionCostKind';
import type { FipeVehicleType } from '@/api/generated/FipeVehicleType';
import type { ImprovementCategory } from '@/api/generated/ImprovementCategory';
import type { PhysicalAssetInput } from '@/api/generated/PhysicalAssetInput';
import type { PhysicalAssetPatch } from '@/api/generated/PhysicalAssetPatch';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalImprovement } from '@/api/generated/PhysicalImprovement';
import type { PhysicalIndex } from '@/api/generated/PhysicalIndex';
import type { PhysicalKind } from '@/api/generated/PhysicalKind';
import type { PhysicalValuationMethod } from '@/api/generated/PhysicalValuationMethod';
import type { PhysicalValuationSource } from '@/api/generated/PhysicalValuationSource';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  MoneyInput,
  Select,
} from '@/design/components';
import { DecimalInput } from '@/features/portfolio/components/DecimalInput';
import { Checkbox, Segmented } from '@/features/transactions/ui/controls';
import { ProjectSelect } from '@/features/projects/ProjectDialogs';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { FormError } from '@/vault/FormError';

import {
  CATEGORIES,
  CATEGORY_LABEL,
  COST_KIND_LABEL,
  COST_KINDS,
  FIPE_TYPE_LABEL,
  INDEX_HINT,
  INDEX_LABEL,
  INDEXES,
  KIND_LABEL,
  KINDS,
  METHOD_HINT,
  METHOD_LABEL,
  SOURCE_LABEL,
  SOURCES,
} from './labels';
import { freeLoans } from './transform';

const NONE = '__none__';

/** Métodos que fazem sentido para cada tipo (o primeiro é o padrão do núcleo). A tabela FIPE
 * só existe no Brasil (docs/countries.md). */
export function methodsFor(kind: PhysicalKind, fipe = true): PhysicalValuationMethod[] {
  switch (kind) {
    case 'real_estate':
    case 'land':
      return ['index', 'none'];
    case 'vehicle':
      return fipe ? ['declining', 'fipe', 'none'] : ['declining', 'none'];
    case 'equipment':
      return ['straight_line', 'declining', 'none'];
    default:
      return ['none', 'index', 'declining'];
  }
}

interface CostRow {
  key: number;
  kind: AcquisitionCostKind;
  amount: number | null;
  description: string;
}

interface AssetForm {
  name: string;
  kind: PhysicalKind;
  purchaseDate: string | null;
  purchasePrice: number | null;
  members: string[];
  costs: CostRow[];
  method: PhysicalValuationMethod;
  index: PhysicalIndex;
  rate: string | null;
  life: string;
  residual: string | null;
  fipeType: FipeVehicleType;
  fipeBrand: string;
  fipeModel: string;
  fipeYear: string;
  fipeCode: string;
  address: string;
  description: string;
  notes: string;
  inNetWorth: boolean;
  loanId: string;
}

let costKey = 0;

function initial(
  a: PhysicalAssetView | null | undefined,
  kind: PhysicalKind = 'real_estate',
): AssetForm {
  const k = a?.kind ?? kind;
  return {
    name: a?.name ?? '',
    kind: k,
    purchaseDate: a?.purchase_date ?? todayIso(),
    purchasePrice: a?.purchase_price ?? null,
    members: a?.member_ids ?? [],
    costs: (a?.acquisition_costs ?? []).map((c) => ({
      key: ++costKey,
      kind: c.kind,
      amount: c.amount,
      description: c.description ?? '',
    })),
    method: a?.valuation_method ?? methodsFor(k)[0]!,
    index: a?.index_series ?? 'ipca',
    rate: a?.depreciation_rate ?? '12',
    life: String(a?.useful_life_years ?? 5),
    residual: a?.residual_pct ?? null,
    fipeType: a?.fipe_vehicle_type ?? 'carros',
    fipeBrand: a?.fipe_brand_code ?? '',
    fipeModel: a?.fipe_model_code ?? '',
    fipeYear: a?.fipe_year_code ?? '',
    fipeCode: a?.fipe_code ?? '',
    address: a?.address ?? '',
    description: a?.description ?? '',
    notes: a?.notes ?? '',
    inNetWorth: a?.include_in_net_worth ?? true,
    loanId: NONE,
  };
}

export function AssetFormDialog({
  open,
  onOpenChange,
  asset,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset?: PhysicalAssetView | null;
  onSaved?: (a: PhysicalAssetView) => void;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="06"
      title={asset ? t('bens-editar-bem') : t('bens-novo')}
      description={t('bens-form-descricao')}
    >
      {open ? (
        <AssetFormBody
          asset={asset}
          onDone={(v) => {
            if (onSaved) onSaved(v);
            else onOpenChange(false);
          }}
        />
      ) : null}
    </Dialog>
  );
}

const intOf = (s: string) => (/^\d+$/.test(s.trim()) ? Number(s.trim()) : null);

function AssetFormBody({
  asset,
  onDone,
}: {
  asset?: PhysicalAssetView | null;
  onDone: (a: PhysicalAssetView) => void;
}) {
  const editing = Boolean(asset);
  const [f, setF] = useState<AssetForm>(() => initial(asset));
  const [submitted, setSubmitted] = useState(false);
  const set = <K extends keyof AssetForm>(k: K, v: AssetForm[K]) => setF((s) => ({ ...s, [k]: v }));
  const members = useMembers().data ?? [];
  const loans = useLoans().data ?? [];
  const assets = usePhysicalAssets({ include_archived: true }).data ?? [];
  const available = freeLoans(loans, assets);
  const create = useCreatePhysical();
  const update = useUpdatePhysical();
  const undoToast = useUndoToast();
  const mutation = editing ? update : create;
  const fipe = useCountryFeatures().fipe;
  const methods = methodsFor(f.kind, fipe).includes(f.method)
    ? methodsFor(f.kind, fipe)
    : [f.method, ...methodsFor(f.kind, fipe)];
  const life = intOf(f.life);

  const errors = {
    name: !f.name.trim() ? t('bens-erro-nome') : null,
    date: !f.purchaseDate ? t('bens-erro-data-compra') : null,
    price: f.purchasePrice == null || f.purchasePrice < 0 ? t('bens-erro-preco-compra') : null,
    rate:
      (f.method === 'declining' || f.method === 'fipe') && (f.rate == null || Number(f.rate) > 100)
        ? t('bens-erro-taxa')
        : null,
    life:
      f.method === 'straight_line' && !(life != null && life >= 1 && life <= 100)
        ? t('bens-erro-vida-util')
        : null,
    fipe:
      f.method === 'fipe' && !(f.fipeBrand && f.fipeModel && f.fipeYear)
        ? t('bens-erro-fipe')
        : null,
    costs: f.costs.some((c) => c.amount == null) ? t('bens-erro-valor-invalido') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? (e ?? undefined) : undefined);

  const setKind = (k: PhysicalKind) => setF((s) => ({ ...s, kind: k, method: methodsFor(k)[0]! }));

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    const usesRate = f.method === 'declining' || f.method === 'fipe';
    const fipe =
      f.method === 'fipe' || (f.kind === 'vehicle' && f.fipeBrand && f.fipeModel && f.fipeYear);
    const common = {
      name: f.name.trim(),
      kind: f.kind,
      purchase_date: f.purchaseDate!,
      purchase_price: f.purchasePrice!,
      member_ids: f.members,
      acquisition_costs: f.costs
        .filter((c) => (c.amount ?? 0) > 0)
        .map((c) => ({
          kind: c.kind,
          amount: c.amount!,
          ...(c.description.trim() ? { description: c.description.trim() } : {}),
        })),
      valuation_method: f.method,
      include_in_net_worth: f.inNetWorth,
    };
    if (asset) {
      const patch: PhysicalAssetPatch = {
        id: asset.id,
        ...common,
        index_series: f.method === 'index' ? f.index : null,
        depreciation_rate: usesRate ? f.rate : null,
        useful_life_years: f.method === 'straight_line' ? life : null,
        residual_pct: f.method === 'straight_line' ? f.residual : null,
        fipe_vehicle_type: fipe ? f.fipeType : null,
        fipe_brand_code: fipe ? f.fipeBrand || null : null,
        fipe_model_code: fipe ? f.fipeModel || null : null,
        fipe_year_code: fipe ? f.fipeYear || null : null,
        fipe_code: fipe ? f.fipeCode.trim() || null : null,
        address: f.address.trim() || null,
        description: f.description.trim() || null,
        notes: f.notes.trim() || null,
      };
      update.mutate(patch, {
        onSuccess: (v) => {
          undoToast({ tone: 'success', title: t('bens-alterado'), description: v.name });
          onDone(v);
        },
      });
      return;
    }
    const input: PhysicalAssetInput = {
      ...common,
      ...(f.method === 'index' ? { index_series: f.index } : {}),
      ...(usesRate && f.rate ? { depreciation_rate: f.rate } : {}),
      ...(f.method === 'straight_line' ? { useful_life_years: life! } : {}),
      ...(f.method === 'straight_line' && f.residual ? { residual_pct: f.residual } : {}),
      ...(fipe
        ? {
            fipe_vehicle_type: f.fipeType,
            fipe_brand_code: f.fipeBrand,
            fipe_model_code: f.fipeModel,
            fipe_year_code: f.fipeYear,
            ...(f.fipeCode.trim() ? { fipe_code: f.fipeCode.trim() } : {}),
          }
        : {}),
      ...(f.address.trim() ? { address: f.address.trim() } : {}),
      ...(f.description.trim() ? { description: f.description.trim() } : {}),
      ...(f.notes.trim() ? { notes: f.notes.trim() } : {}),
      ...(f.loanId !== NONE ? { loan_ids: [f.loanId] } : {}),
    };
    create.mutate(input, {
      onSuccess: (v) => {
        undoToast({ tone: 'success', title: t('bens-cadastrado'), description: v.name });
        onDone(v);
      },
    });
  };

  return (
    <form onSubmit={submit} noValidate className="flex flex-col gap-4" aria-label={t('bens-bem')}>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('bens-nome-bem')} error={show(errors.name)}>
          <Input
            value={f.name}
            onChange={(e) => set('name', e.target.value)}
            placeholder={t('bens-nome-exemplo')}
          />
        </Field>
        <Field label={t('bens-tipo')}>
          <Select
            value={f.kind}
            onValueChange={(v) => v && setKind(v as PhysicalKind)}
            options={KINDS.map((k) => ({ value: k, label: KIND_LABEL[k] }))}
          />
        </Field>
        <Field label={t('bens-data-compra')} error={show(errors.date)}>
          <DateInput value={f.purchaseDate} onValueChange={(v) => set('purchaseDate', v)} />
        </Field>
        <Field
          label={t('bens-preco-compra')}
          error={show(errors.price)}
          hint={t('bens-preco-compra-dica')}
        >
          <MoneyInput value={f.purchasePrice} onValueChange={(v) => set('purchasePrice', v)} />
        </Field>
      </div>

      <fieldset className="flex flex-col gap-2">
        <legend className="label-caps mb-1">{t('bens-custos-aquisicao')}</legend>
        <p className="text-[12.5px] text-dim">{t('bens-custos-aquisicao-dica')}</p>
        {f.costs.map((c, i) => (
          <div key={c.key} className="grid items-end gap-2 sm:grid-cols-[1fr_1fr_1.4fr_auto]">
            <Field label={t('bens-custo-n', { n: i + 1 })}>
              <Select
                value={c.kind}
                onValueChange={(v) =>
                  set(
                    'costs',
                    f.costs.map((x) =>
                      x.key === c.key ? { ...x, kind: v as AcquisitionCostKind } : x,
                    ),
                  )
                }
                options={COST_KINDS.map((k) => ({ value: k, label: COST_KIND_LABEL[k] }))}
              />
            </Field>
            <Field
              label={t('bens-valor')}
              error={c.amount == null ? show(errors.costs) : undefined}
            >
              <MoneyInput
                value={c.amount}
                onValueChange={(v) =>
                  set(
                    'costs',
                    f.costs.map((x) => (x.key === c.key ? { ...x, amount: v } : x)),
                  )
                }
              />
            </Field>
            <Field label={t('bens-descricao')} hint={t('bens-opcional')}>
              <Input
                value={c.description}
                onChange={(e) =>
                  set(
                    'costs',
                    f.costs.map((x) =>
                      x.key === c.key ? { ...x, description: e.target.value } : x,
                    ),
                  )
                }
              />
            </Field>
            <Button
              variant="ghost"
              aria-label={t('bens-remover-custo', { n: i + 1 })}
              onClick={() =>
                set(
                  'costs',
                  f.costs.filter((x) => x.key !== c.key),
                )
              }
            >
              {t('bens-remover')}
            </Button>
          </div>
        ))}
        <div>
          <Button
            variant="ghost"
            icon={<Icon name="plus" size={12} />}
            onClick={() =>
              set('costs', [
                ...f.costs,
                {
                  key: ++costKey,
                  kind: f.costs.some((c) => c.kind === 'itbi') ? 'registry' : 'itbi',
                  amount: 0,
                  description: '',
                },
              ])
            }
          >
            {t('bens-adicionar-custo')}
          </Button>
        </div>
      </fieldset>

      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('bens-estimado-por')} hint={METHOD_HINT[f.method]}>
          <Select
            value={f.method}
            onValueChange={(v) => v && set('method', v as PhysicalValuationMethod)}
            options={methods.map((m) => ({ value: m, label: METHOD_LABEL[m] }))}
          />
        </Field>
        {f.method === 'index' ? (
          <Field label={t('bens-indice-correcao')} hint={INDEX_HINT[f.index]}>
            <Select
              value={f.index}
              onValueChange={(v) => v && set('index', v as PhysicalIndex)}
              options={INDEXES.map((k) => ({ value: k, label: INDEX_LABEL[k] }))}
            />
          </Field>
        ) : null}
        {f.method === 'declining' || f.method === 'fipe' ? (
          <Field
            label={t('bens-depreciacao-aa')}
            error={show(errors.rate)}
            hint={
              f.method === 'fipe' ? t('bens-depreciacao-fipe-dica') : t('bens-depreciacao-dica')
            }
          >
            <DecimalInput
              value={f.rate}
              onValueChange={(v) => set('rate', v)}
              maxDecimals={4}
              suffix="%"
            />
          </Field>
        ) : null}
        {f.method === 'straight_line' ? (
          <>
            <Field label={t('bens-vida-util')} error={show(errors.life)}>
              <Input
                inputMode="numeric"
                numeric
                value={f.life}
                onChange={(e) => set('life', e.target.value.replace(/\D/g, ''))}
              />
            </Field>
            <Field label={t('bens-residual')} hint={t('bens-residual-dica')}>
              <DecimalInput
                value={f.residual}
                onValueChange={(v) => set('residual', v)}
                maxDecimals={2}
                suffix="%"
              />
            </Field>
          </>
        ) : null}
      </div>

      {f.kind === 'vehicle' ? (
        <FipeFields f={f} set={set} required={f.method === 'fipe'} error={show(errors.fipe)} />
      ) : null}

      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('bens-endereco')} hint={t('bens-endereco-dica')}>
          <Input value={f.address} onChange={(e) => set('address', e.target.value)} />
        </Field>
        <Field label={t('bens-descricao')} hint={t('bens-opcional')}>
          <Input value={f.description} onChange={(e) => set('description', e.target.value)} />
        </Field>
        {!editing ? (
          <Field label={t('bens-financiamento')} hint={t('bens-financiamento-dica')}>
            <Select
              value={f.loanId}
              onValueChange={(v) => v && set('loanId', v)}
              options={[
                { value: NONE, label: t('bens-nenhum') },
                ...available.map((l) => ({ value: l.id, label: l.name })),
              ]}
            />
          </Field>
        ) : null}
        <Field label={t('bens-observacoes')} hint={t('bens-opcional')}>
          <Input value={f.notes} onChange={(e) => set('notes', e.target.value)} />
        </Field>
      </div>

      <div className="flex flex-wrap gap-x-6 gap-y-2">
        {members.map((m) => (
          <Checkbox
            key={m.id}
            checked={f.members.includes(m.id)}
            onChange={(c) =>
              set('members', c ? [...f.members, m.id] : f.members.filter((x) => x !== m.id))
            }
            label={t('bens-de-membro', { nome: m.name })}
          />
        ))}
        <Checkbox
          checked={f.inNetWorth}
          onChange={(c) => set('inNetWorth', c)}
          label={t('bens-entra-patrimonio')}
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
          {editing ? t('bens-salvar-alteracoes') : t('bens-cadastrar')}
        </Button>
      </div>
    </form>
  );
}

function FipeFields({
  f,
  set,
  required,
  error,
}: {
  f: AssetForm;
  set: <K extends keyof AssetForm>(k: K, v: AssetForm[K]) => void;
  required: boolean;
  error?: string;
}) {
  const [lookup, setLookup] = useState(required || Boolean(f.fipeBrand));
  const brands = useFipeOptions(lookup ? { vehicle_type: f.fipeType } : null);
  const models = useFipeOptions(
    lookup && f.fipeBrand ? { vehicle_type: f.fipeType, brand_code: f.fipeBrand } : null,
  );
  const years = useFipeOptions(
    lookup && f.fipeBrand && f.fipeModel
      ? { vehicle_type: f.fipeType, brand_code: f.fipeBrand, model_code: f.fipeModel }
      : null,
  );
  const opts = (list: { code: string; name: string }[] | undefined, current: string) => {
    const o = (list ?? []).map((x) => ({ value: x.code, label: x.name }));
    return current && !o.some((x) => x.value === current)
      ? [{ value: current, label: current }, ...o]
      : o;
  };
  const failed = brands.error ?? models.error ?? years.error;
  return (
    <fieldset className="flex flex-col gap-2 rounded-control border border-line px-3 py-3">
      <legend className="label-caps px-1">{t('bens-metodo-fipe')}</legend>
      {!lookup ? (
        <div className="flex flex-wrap items-center gap-3">
          <p className="text-[12.5px] text-dim">{t('bens-fipe-explica')}</p>
          <Button variant="ghost" onClick={() => setLookup(true)}>
            {t('bens-fipe-buscar')}
          </Button>
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label={t('bens-fipe-tipo-veiculo')}>
            <Segmented<FipeVehicleType>
              size="sm"
              label={t('bens-fipe-tipo-veiculo')}
              value={f.fipeType}
              onChange={(v) => {
                set('fipeType', v);
                set('fipeBrand', '');
                set('fipeModel', '');
                set('fipeYear', '');
              }}
              options={(['carros', 'motos', 'caminhoes'] as const).map((v) => ({
                value: v,
                label: FIPE_TYPE_LABEL[v],
              }))}
            />
          </Field>
          <Field label={t('bens-fipe-marca')} error={!f.fipeBrand ? error : undefined}>
            <Select
              value={f.fipeBrand || undefined}
              placeholder={brands.isPending ? t('comum-carregando') : t('bens-fipe-escolha-marca')}
              onValueChange={(v) => {
                set('fipeBrand', v);
                set('fipeModel', '');
                set('fipeYear', '');
              }}
              options={opts(brands.data, f.fipeBrand)}
            />
          </Field>
          <Field
            label={t('bens-fipe-modelo')}
            error={f.fipeBrand && !f.fipeModel ? error : undefined}
          >
            <Select
              value={f.fipeModel || undefined}
              disabled={!f.fipeBrand}
              placeholder={t('bens-fipe-escolha-modelo')}
              onValueChange={(v) => {
                set('fipeModel', v);
                set('fipeYear', '');
              }}
              options={opts(models.data, f.fipeModel)}
            />
          </Field>
          <Field label={t('bens-fipe-ano')} error={f.fipeModel && !f.fipeYear ? error : undefined}>
            <Select
              value={f.fipeYear || undefined}
              disabled={!f.fipeModel}
              placeholder={t('bens-fipe-escolha-ano')}
              onValueChange={(v) => set('fipeYear', v)}
              options={opts(years.data, f.fipeYear)}
            />
          </Field>
          <Field label={t('bens-fipe-codigo')} hint={t('bens-fipe-codigo-dica')}>
            <Input
              value={f.fipeCode}
              onChange={(e) => set('fipeCode', e.target.value)}
              placeholder="004478-4"
            />
          </Field>
          {failed ? (
            <p role="alert" className="text-[12.5px] text-negative sm:col-span-2">
              {failed.message}
            </p>
          ) : null}
        </div>
      )}
    </fieldset>
  );
}

// ── Benfeitoria ──────────────────────────────────────────────────

export function ImprovementDialog({
  open,
  onOpenChange,
  asset,
  improvement,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset: PhysicalAssetView;
  improvement?: PhysicalImprovement | null;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="06"
      title={improvement ? t('bens-editar-benfeitoria') : t('bens-registrar-benfeitoria-obra')}
      description={t('bens-benfeitoria-descricao', { nome: asset.name })}
    >
      {open ? (
        <ImprovementBody
          asset={asset}
          improvement={improvement}
          onDone={() => onOpenChange(false)}
        />
      ) : null}
    </Dialog>
  );
}

function ImprovementBody({
  asset,
  improvement,
  onDone,
}: {
  asset: PhysicalAssetView;
  improvement?: PhysicalImprovement | null;
  onDone: () => void;
}) {
  const [date, setDate] = useState<string | null>(improvement?.date ?? todayIso());
  const [description, setDescription] = useState(improvement?.description ?? '');
  const [category, setCategory] = useState<ImprovementCategory>(
    improvement?.category ?? 'renovation',
  );
  const [amount, setAmount] = useState<number | null>(improvement?.entered_amount ?? null);
  const [notes, setNotes] = useState(improvement?.notes ?? '');
  // Projeto com os gastos da obra: escolhido, novo (com o nome da benfeitoria) ou nenhum.
  const [project, setProject] = useState<string | null>(improvement?.project_id ?? null);
  const [submitted, setSubmitted] = useState(false);
  const add = useAddImprovement();
  const update = useUpdateImprovement();
  const mutation = improvement ? update : add;
  const undoToast = useUndoToast();
  const errors = {
    date: !date ? t('bens-erro-data') : null,
    description: !description.trim() ? t('bens-erro-descreva') : null,
    amount:
      !(amount != null && amount > 0) && project == null ? t('bens-erro-valor-projeto') : null,
  };
  const projectArgs =
    project === 'new' ? { create_project: true } : { project_id: project ?? null };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? (e ?? undefined) : undefined);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    const done = {
      onSuccess: () => {
        undoToast({
          tone: 'success',
          title: improvement ? t('bens-benfeitoria-alterada') : t('bens-benfeitoria-registrada'),
          description: `${description.trim()} · ${CATEGORY_LABEL[category]}`,
        });
        onDone();
      },
    };
    if (improvement)
      update.mutate(
        {
          id: improvement.id,
          date: date!,
          description: description.trim(),
          category,
          ...(amount != null && amount > 0 ? { amount } : {}),
          notes: notes.trim() || null,
          ...projectArgs,
        },
        done,
      );
    else
      add.mutate(
        {
          asset_id: asset.id,
          date: date!,
          description: description.trim(),
          category,
          amount: amount ?? 0,
          ...(notes.trim() ? { notes: notes.trim() } : {}),
          ...(project === 'new'
            ? { create_project: true }
            : project
              ? { project_id: project }
              : {}),
        },
        done,
      );
  };
  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('bens-benfeitoria')}
    >
      <Field label={t('bens-tipo')}>
        <Segmented<ImprovementCategory>
          label={t('bens-tipo-obra')}
          value={category}
          onChange={setCategory}
          options={CATEGORIES.map((c) => ({ value: c, label: CATEGORY_LABEL[c] }))}
        />
      </Field>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field
          label={t('bens-descricao')}
          error={show(errors.description)}
          className="sm:col-span-2"
        >
          <Input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t('bens-benfeitoria-exemplo')}
          />
        </Field>
        <Field label={t('bens-data')} error={show(errors.date)}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field
          label={project ? t('bens-valor-estimado-campo') : t('bens-valor')}
          error={show(errors.amount)}
          hint={
            project
              ? t('bens-valor-projeto-dica')
              : category === 'maintenance'
                ? t('bens-fica-fora-custo')
                : t('bens-soma-custo')
          }
        >
          <MoneyInput value={amount} onValueChange={setAmount} />
        </Field>
        <Field
          label={t('bens-projeto-gastos')}
          hint={t('bens-projeto-gastos-dica')}
          className="sm:col-span-2"
        >
          <ProjectSelect
            aria-label={t('bens-projeto-gastos')}
            value={project}
            onChange={setProject}
            allowNew
            newLabel={t('bens-novo-projeto-benfeitoria')}
          />
          {project === 'new' ? (
            <span className="text-[12px] text-dim">
              {t('bens-sera-criado', { nome: description.trim() || t('bens-a-benfeitoria') })}
            </span>
          ) : null}
        </Field>
        <Field label={t('bens-observacoes')} hint={t('bens-opcional')} className="sm:col-span-2">
          <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
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
          {improvement ? t('comum-salvar') : t('bens-registrar')}
        </Button>
      </div>
    </form>
  );
}

// ── Avaliação ────────────────────────────────────────────────────

export function ValuationDialog({
  open,
  onOpenChange,
  asset,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset: PhysicalAssetView;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="06"
      title={t('bens-registrar-avaliacao')}
      description={t('bens-avaliacao-descricao', { nome: asset.name })}
    >
      {open ? <ValuationBody asset={asset} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function ValuationBody({ asset, onDone }: { asset: PhysicalAssetView; onDone: () => void }) {
  const [date, setDate] = useState<string | null>(todayIso());
  const [value, setValue] = useState<number | null>(null);
  const [source, setSource] = useState<PhysicalValuationSource>(
    asset.kind === 'vehicle' ? 'fipe' : 'appraisal',
  );
  const [notes, setNotes] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const add = useAddValuation();
  const undoToast = useUndoToast();
  const errors = {
    date: !date ? t('bens-erro-data') : null,
    value: !(value != null && value >= 0) ? t('bens-erro-valor') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? (e ?? undefined) : undefined);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    add.mutate(
      {
        asset_id: asset.id,
        date: date!,
        value: value!,
        source,
        ...(notes.trim() ? { notes: notes.trim() } : {}),
      },
      {
        onSuccess: () => {
          undoToast({
            tone: 'success',
            title: t('bens-avaliacao-registrada'),
            description: asset.name,
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
      aria-label={t('bens-avaliacao')}
    >
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('bens-data')} error={show(errors.date)}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('bens-valor')} error={show(errors.value)}>
          <MoneyInput value={value} onValueChange={setValue} />
        </Field>
        <Field label={t('bens-fonte')}>
          <Select
            value={source}
            onValueChange={(v) => v && setSource(v as PhysicalValuationSource)}
            options={SOURCES.map((s) => ({ value: s, label: SOURCE_LABEL[s] }))}
          />
        </Field>
        <Field label={t('bens-observacoes')} hint={t('bens-avaliacao-obs-dica')}>
          <Input value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
      </div>
      <FormError error={add.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          loading={add.isPending}
          disabled={submitted && invalid}
        >
          {t('bens-registrar-avaliacao')}
        </Button>
      </div>
    </form>
  );
}

// ── Venda ────────────────────────────────────────────────────────

export function SaleDialog({
  open,
  onOpenChange,
  asset,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset: PhysicalAssetView;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="06"
      title={t('bens-vender-nome', { nome: asset.name })}
      description={t('bens-venda-descricao')}
    >
      {open ? <SaleBody asset={asset} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

function SaleBody({ asset, onDone }: { asset: PhysicalAssetView; onDone: () => void }) {
  const [date, setDate] = useState<string | null>(todayIso());
  const [price, setPrice] = useState<number | null>(asset.figures.value || null);
  const [costs, setCosts] = useState<number | null>(0);
  const [archive, setArchive] = useState(true);
  const [submitted, setSubmitted] = useState(false);
  const sell = useSellPhysical();
  const undoToast = useUndoToast();
  const errors = {
    date: !date
      ? t('bens-erro-data')
      : date < asset.purchase_date
        ? t('bens-erro-venda-antes')
        : null,
    price: !(price != null && price >= 0) ? t('bens-erro-preco') : null,
    costs: costs == null ? t('bens-erro-valor-invalido') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? (e ?? undefined) : undefined);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    sell.mutate(
      { id: asset.id, sale_date: date!, sale_price: price!, sale_costs: costs ?? 0, archive },
      {
        onSuccess: () => {
          undoToast({
            tone: 'success',
            title: t('bens-venda-registrada'),
            description: asset.name,
          });
          onDone();
        },
      },
    );
  };
  return (
    <form onSubmit={submit} noValidate className="flex flex-col gap-4" aria-label={t('bens-venda')}>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label={t('bens-data-venda')} error={show(errors.date)}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        <Field label={t('bens-preco-venda')} error={show(errors.price)}>
          <MoneyInput value={price} onValueChange={setPrice} />
        </Field>
        <Field
          label={t('bens-custos-venda')}
          error={show(errors.costs)}
          hint={t('bens-custos-venda-dica')}
        >
          <MoneyInput value={costs} onValueChange={setCosts} />
        </Field>
      </div>
      <Checkbox checked={archive} onChange={setArchive} label={t('bens-arquivar-bem')} />
      <FormError error={sell.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          type="submit"
          variant="primary"
          loading={sell.isPending}
          disabled={submitted && invalid}
        >
          {t('bens-registrar-venda')}
        </Button>
      </div>
    </form>
  );
}

// ── Ligar financiamento ──────────────────────────────────────────

export function LinkLoanDialog({
  open,
  onOpenChange,
  asset,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  asset: PhysicalAssetView;
}) {
  const loans = useLoans().data ?? [];
  const assets = usePhysicalAssets({ include_archived: true }).data ?? [];
  const available = freeLoans(loans, assets);
  const [loanId, setLoanId] = useState<string>(NONE);
  const link = useLinkLoan();
  const undoToast = useUndoToast();
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      index="06"
      title={t('bens-ligar-financiamento')}
      description={t('bens-ligar-descricao', { nome: asset.name })}
    >
      <form
        className="flex flex-col gap-4"
        aria-label={t('bens-ligar-financiamento')}
        onSubmit={(e) => {
          e.preventDefault();
          if (loanId === NONE) return;
          link.mutate(
            { asset_id: asset.id, loan_id: loanId },
            {
              onSuccess: () => {
                undoToast({
                  tone: 'success',
                  title: t('bens-financiamento-ligado'),
                  description: asset.name,
                });
                onOpenChange(false);
              },
            },
          );
        }}
      >
        {available.length === 0 ? (
          <p className="text-[13px] text-dim">{t('bens-nenhum-contrato-livre')}</p>
        ) : (
          <Field label={t('bens-contrato')}>
            <Select
              value={loanId}
              onValueChange={(v) => v && setLoanId(v)}
              options={[
                { value: NONE, label: t('bens-escolha-contrato') },
                ...available.map((l) => ({ value: l.id, label: l.name })),
              ]}
            />
          </Field>
        )}
        <FormError error={link.error} />
        <div className="flex justify-end gap-2 border-t border-line pt-3">
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            type="submit"
            variant="primary"
            loading={link.isPending}
            disabled={loanId === NONE}
          >
            {t('bens-ligar-contrato')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
