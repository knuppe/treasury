/**
 * Ajuste de uma categoria na estimativa (`tax_burden.set_category`): trocar o grupo de
 * imposto, fixar uma alíquota própria (vale nos três cenários) ou voltar ao padrão; e a
 * base de um grupo de consumo (produtos por trás do número, alíquotas e fontes).
 *
 * `useCategoryEditor` guarda qual diálogo está aberto: a tela monta `dialogs` uma vez e as
 * tabelas chamam `open`.
 */
import { useState, type ReactNode } from 'react';

import type { TaxCategoryView } from '@/api/generated/TaxCategoryView';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import { useSetTaxCategory } from '@/api/hooks/taxBurden';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Input,
  KeyValue,
  Select,
  toast,
  type SelectOption,
} from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import {
  consumptionGroup,
  directGroups,
  formatRate,
  groupLabel,
  groupRated,
  intlLocale,
  notConsumptionGroups,
  ORIGIN_LABEL,
  rateInputText,
  rateSpecText,
  sourcesOf,
} from './model';
import { SourceList } from './SourcesDialog';

const DEFAULT = '__padrao';

/** Opções de grupo: padrão, grupos de consumo do país, imposto pago e fora do consumo. */
export function groupOptions(ds: TaxDataset | undefined, withDefault?: string): SelectOption[] {
  const consumption = [...(ds?.consumption ?? [])].sort((a, b) =>
    a.label.localeCompare(b.label, intlLocale()),
  );
  return [
    ...(withDefault ? [{ value: DEFAULT, label: withDefault }] : []),
    ...consumption.map((g) => ({
      value: g.key,
      label: g.label,
      hint: groupRated(g) ? rateSpecText(g.central) : t('impostos-sem-dado'),
    })),
    ...directGroups().map(([value, label]) => ({
      value,
      label,
      hint: t('impostos-imposto-pago'),
    })),
    ...notConsumptionGroups().map(([value, label]) => ({
      value,
      label,
      hint: t('impostos-fora-consumo'),
    })),
  ];
}

/** `12,5` → 12.5; `null` se vazio ou inválido. */
export function parseRate(text: string): number | null {
  const clean = text.trim().replaceAll('%', '').replaceAll(',', '.');
  if (!clean) return null;
  const n = Number(clean);
  return Number.isFinite(n) ? n : null;
}

export type EditorKind = 'group' | 'rate' | 'basis';

interface Target {
  kind: EditorKind;
  categoryId: string;
  path: string;
  /** Grupo para "Ver base" (pode vir da linha da estimativa). */
  groupKey?: string;
}

export function useCategoryEditor({
  dataset,
  categories,
}: {
  dataset: TaxDataset | undefined;
  categories: readonly TaxCategoryView[] | undefined;
}) {
  const [target, setTarget] = useState<Target | null>(null);
  const set = useSetTaxCategory();
  const view = target ? categories?.find((c) => c.category_id === target.categoryId) : undefined;
  const close = () => setTarget(null);

  const save = (patch: { group_key?: string | null; rate?: number | null }, message: string) => {
    if (!target) return;
    set.mutate(
      { category_id: target.categoryId, ...patch },
      {
        onSuccess: () => {
          toast({ tone: 'success', title: message, description: target.path });
          close();
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('impostos-erro-salvar'), description: errorMessage(e) }),
      },
    );
  };

  const reset = (categoryId: string, path: string) =>
    set.mutate(
      { category_id: categoryId, group_key: null, rate: null },
      {
        onSuccess: () =>
          toast({ tone: 'success', title: t('impostos-categoria-padrao'), description: path }),
        onError: (e) =>
          toast({ tone: 'error', title: t('impostos-erro-salvar'), description: errorMessage(e) }),
      },
    );

  const dialogs = target ? (
    target.kind === 'group' ? (
      <GroupDialog
        key={`g-${target.categoryId}`}
        path={target.path}
        view={view}
        dataset={dataset}
        pending={set.isPending}
        onCancel={close}
        onSave={(g) =>
          save({ group_key: g }, g ? t('impostos-grupo-trocado') : t('impostos-grupo-padrao'))
        }
      />
    ) : target.kind === 'rate' ? (
      <RateDialog
        key={`r-${target.categoryId}`}
        path={target.path}
        view={view}
        pending={set.isPending}
        onCancel={close}
        onSave={(r) =>
          save({ rate: r }, r == null ? t('impostos-aliquota-pais') : t('impostos-aliquota-salva'))
        }
      />
    ) : (
      <BasisDialog
        key={`b-${target.categoryId}`}
        path={target.path}
        groupKey={target.groupKey ?? view?.group_key ?? null}
        dataset={dataset}
        onClose={close}
      />
    )
  ) : null;

  return {
    open: (kind: EditorKind, categoryId: string, path: string, groupKey?: string) =>
      setTarget({ kind, categoryId, path, groupKey }),
    reset,
    pending: set.isPending,
    dialogs,
  };
}

export type CategoryEditor = ReturnType<typeof useCategoryEditor>;

function GroupDialog({
  path,
  view,
  dataset,
  pending,
  onCancel,
  onSave,
}: {
  path: string;
  view: TaxCategoryView | undefined;
  dataset: TaxDataset | undefined;
  pending: boolean;
  onCancel: () => void;
  onSave: (group: string | null) => void;
}) {
  const userGroup = view?.origin === 'user' ? view.group_key : null;
  const [value, setValue] = useState<string>(userGroup ?? DEFAULT);
  const options = groupOptions(dataset, t('impostos-padrao-app-longo'));
  const chosen = value === DEFAULT ? null : value;
  const g = chosen ? consumptionGroup(dataset, chosen) : undefined;
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onCancel()}
      title={t('impostos-trocar-grupo-titulo')}
      description={withSlots(t('impostos-trocar-grupo-descricao', { categoria: slot(0) }), [
        <span key="c" className="text-fg">
          {path}
        </span>,
      ])}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" loading={pending} onClick={() => onSave(chosen)}>
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      {view?.group_key ? (
        <p className="text-[12.5px] text-dim">
          {t('impostos-hoje', {
            grupo: groupLabel(view.group_key, dataset),
            origem: ORIGIN_LABEL[view.origin ?? ''] ?? '—',
          })}
        </p>
      ) : null}
      <Field label={t('impostos-grupo')}>
        <Select
          value={value}
          onValueChange={setValue}
          options={options}
          aria-label={t('impostos-grupo')}
        />
      </Field>
      {g ? <p className="text-[12.5px] leading-relaxed text-dim">{g.basis}</p> : null}
    </Dialog>
  );
}

function RateDialog({
  path,
  view,
  pending,
  onCancel,
  onSave,
}: {
  path: string;
  view: TaxCategoryView | undefined;
  pending: boolean;
  onCancel: () => void;
  onSave: (rate: number | null) => void;
}) {
  const [text, setText] = useState(rateInputText(view?.rate));
  const rate = parseRate(text);
  const error =
    text.trim() && (rate == null || rate < 0 || rate >= 100)
      ? t('impostos-aliquota-faixa')
      : undefined;
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onCancel()}
      title={t('impostos-aliquota-propria')}
      description={withSlots(t('impostos-aliquota-descricao', { categoria: slot(0) }), [
        <span key="c" className="text-fg">
          {path}
        </span>,
      ])}
      footer={
        <>
          {view?.rate != null ? (
            <Button variant="ghost" disabled={pending} onClick={() => onSave(null)}>
              {t('impostos-usar-pais')}
            </Button>
          ) : null}
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            loading={pending}
            disabled={rate == null || Boolean(error)}
            onClick={() => onSave(rate)}
          >
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <Field label={t('impostos-aliquota-pct')} error={error}>
        <Input
          numeric
          inputMode="decimal"
          value={text}
          onChange={(e) => setText(e.target.value)}
          suffix="%"
          placeholder={t('impostos-aliquota-exemplo')}
          autoFocus
        />
      </Field>
    </Dialog>
  );
}

function Line({ label, children }: { label: string; children: ReactNode }) {
  return (
    <p className="text-[13px] leading-relaxed text-fg">
      <span className="label-caps mr-2">{label}</span>
      {children}
    </p>
  );
}

function BasisDialog({
  path,
  groupKey,
  dataset,
  onClose,
}: {
  path: string;
  groupKey: string | null;
  dataset: TaxDataset | undefined;
  onClose: () => void;
}) {
  const g = groupKey ? consumptionGroup(dataset, groupKey) : undefined;
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={g ? t('impostos-base-de', { grupo: g.label }) : t('impostos-base-grupo')}
      description={<span className="text-fg">{path}</span>}
      size="lg"
      footer={
        <DialogClose asChild>
          <Button>{t('comum-fechar')}</Button>
        </DialogClose>
      }
    >
      {g ? (
        <>
          <Line label={t('impostos-produtos-servicos')}>{g.basis}</Line>
          <KeyValue
            items={[
              { label: t('impostos-cenario-conservador'), value: rateSpecText(g.low) },
              { label: t('impostos-cenario-provavel'), value: rateSpecText(g.central) },
              { label: t('impostos-cenario-elevado'), value: rateSpecText(g.high) },
            ]}
          />
          {g.icms_modal && dataset?.reference_icms != null ? (
            <p className="text-[12.5px] text-dim">
              {t('impostos-icms-modal', { icms: formatRate(dataset.reference_icms) })}
            </p>
          ) : null}
          <section aria-label={t('impostos-fontes-grupo')} className="flex flex-col gap-2">
            <h3 className="label-caps text-fg">{t('impostos-fontes')}</h3>
            <SourceList sources={sourcesOf(dataset, g.source_ids)} />
          </section>
        </>
      ) : (
        <p className="text-[13px] text-dim">
          {groupKey
            ? t('impostos-valor-como-esta', { grupo: groupLabel(groupKey, dataset) })
            : t('impostos-categoria-sem-grupo')}
        </p>
      )}
    </Dialog>
  );
}
