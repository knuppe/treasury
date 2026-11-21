/**
 * Controles da tela de impostos: ano, país e estado dos dados, dependentes (para o IR
 * estimado a partir do líquido) e a barra fixa no topo com o cenário e o total.
 */
import { useState, type ReactNode } from 'react';

import type { Scenario } from '@/api/generated/Scenario';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxSettings } from '@/api/generated/TaxSettings';
import type { TaxSettingsPatch } from '@/api/generated/TaxSettingsPatch';
import { useSetTaxSettings } from '@/api/hooks/taxBurden';
import { Field, Input, Money, Select, toast } from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { Segmented } from '@/features/reports/controls';
import { t } from '@/i18n';

import { isDraft, regionLabel, SCENARIOS } from './model';

const NO_REGION = '__nenhum';

export function YearSelect({
  year,
  years,
  onChange,
}: {
  year: number;
  years: readonly number[];
  onChange: (y: number) => void;
}) {
  return (
    <Field label={t('impostos-ano')} className="w-28">
      <Select
        aria-label={t('impostos-ano')}
        value={String(year)}
        onValueChange={(v) => onChange(Number(v))}
        options={years.map((y) => ({ value: String(y), label: String(y) }))}
      />
    </Field>
  );
}

export function PlaceControls({
  settings,
  datasets,
}: {
  settings: TaxSettings;
  datasets: readonly TaxDataset[];
}) {
  const set = useSetTaxSettings();
  const ds = datasets.find((d) => d.country === settings.country);
  const update = (patch: TaxSettingsPatch, errorTitle: string) =>
    set.mutate(patch, {
      onError: (e) =>
        toast({
          tone: 'error',
          title: errorTitle,
          description: errorMessage(e),
        }),
    });
  const regions = [...(ds?.regions ?? [])].sort((a, b) => a.name.localeCompare(b.name, 'pt-BR'));
  return (
    <>
      <Field label={t('impostos-pais-dados')} className="w-52">
        <Select
          aria-label={t('impostos-pais-dados')}
          value={settings.country}
          disabled={set.isPending}
          onValueChange={(v) =>
            v !== settings.country && update({ country: v }, t('impostos-erro-trocar-pais'))
          }
          options={datasets.map((d) => ({
            value: d.country,
            label: d.name,
            hint: isDraft(d.coverage) ? t('impostos-esboco') : undefined,
          }))}
        />
      </Field>
      <Field label={regionLabel(settings.country)} className="w-52">
        <Select
          aria-label={regionLabel(settings.country)}
          value={settings.region ?? NO_REGION}
          disabled={set.isPending || regions.length === 0}
          placeholder={regions.length === 0 ? t('impostos-sem-dados-estado') : undefined}
          onValueChange={(v) =>
            update(
              { region: v === NO_REGION ? null : v },
              t('impostos-erro-trocar-regiao', { regiao: regionLabel(settings.country) }),
            )
          }
          options={[
            {
              value: NO_REGION,
              label:
                regions.length === 0
                  ? t('impostos-sem-dados-estado')
                  : t('impostos-nao-informado-opcao'),
            },
            ...regions.map((r) => ({ value: r.code, label: r.name, hint: r.code })),
          ]}
        />
      </Field>
      {ds && ds.payroll.length > 0 ? (
        <DependentsInput
          key={settings.dependents}
          value={settings.dependents}
          pending={set.isPending}
          onCommit={(n) => update({ dependents: n }, t('impostos-erro-trocar-dependentes'))}
        />
      ) : null}
    </>
  );
}

function DependentsInput({
  value,
  pending,
  onCommit,
}: {
  value: number;
  pending: boolean;
  onCommit: (n: number) => void;
}) {
  const [text, setText] = useState(String(value));
  const n = Number(text);
  const valid = /^\d{1,2}$/.test(text.trim()) && n <= 20;
  const commit = () => {
    if (valid && n !== value) onCommit(n);
    else if (!valid) setText(String(value));
  };
  return (
    <Field label={t('impostos-dependentes')} className="w-28">
      <Input
        aria-label={t('impostos-dependentes')}
        title={t('impostos-dependentes-dica')}
        numeric
        inputMode="numeric"
        value={text}
        disabled={pending}
        invalid={!valid}
        onChange={(e) => setText(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            commit();
          }
        }}
      />
    </Field>
  );
}

/** Barra fixa no topo da área da tela: cenário (sempre à vista), total e ações. */
export function ScenarioBar({
  scenario,
  onScenario,
  total,
  currency,
  actions,
}: {
  scenario: Scenario;
  onScenario: (s: Scenario) => void;
  total: number | null;
  currency: string;
  actions?: ReactNode;
}) {
  return (
    <div
      role="region"
      aria-label={t('impostos-cenario-estimativa')}
      className="sticky top-0 z-20 -mx-8 flex flex-wrap items-center gap-x-5 gap-y-2 border-b border-line px-8 py-2.5 sticky-surface"
    >
      <Segmented<Scenario>
        label={t('impostos-cenario')}
        value={scenario}
        onChange={onScenario}
        options={SCENARIOS.map((s) => ({ value: s.value, label: s.label, hint: s.hint }))}
      />
      {total != null ? (
        <span className="flex items-baseline gap-2" aria-live="polite">
          <span className="label-caps">{t('impostos-total-estimado')}</span>
          <Money cents={total} currency={currency} estimated className="text-[16px] text-fg" />
        </span>
      ) : null}
      {actions ? (
        <span className="ml-auto flex flex-wrap items-center gap-2">{actions}</span>
      ) : null}
    </div>
  );
}
