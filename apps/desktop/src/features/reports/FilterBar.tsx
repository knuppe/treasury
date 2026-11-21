/** Filtros numa linha acima dos gráficos: período, base, membro e conta. */
import type { Basis } from '@/api/generated/Basis';
import { useAccounts, useMembers } from '@/api/hooks/reports';
import { DateInput, Icon, IconButton, Select } from '@/design/components';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';

import { BASIS_OPTIONS, Segmented } from './controls';
import {
  PRESETS,
  presetPeriod,
  stepFilters,
  type PresetId,
  type ReportFilters,
} from './model/period';
import { sortedMembers } from './model/series';

const ALL = '__todos__';

export function FilterBar({
  filters,
  onChange,
  defaultBasis,
  today,
  usesPeriod = false,
}: {
  filters: ReportFilters;
  onChange: (f: ReportFilters) => void;
  defaultBasis: Basis;
  today: string;
  /**
   * A visualização usa o período: mostra o seletor e as setas. As que olham para a
   * frente (recorrentes, cartões) não têm o campo, que não mudaria nada nelas.
   */
  usesPeriod?: boolean;
}) {
  const members = useMembers();
  const accounts = useAccounts();
  const base = useBaseCurrency();
  const set = (patch: Partial<ReportFilters>) => onChange({ ...filters, ...patch });
  return (
    <div
      role="toolbar"
      aria-label={t('relatorios-filtros')}
      className="flex flex-wrap items-end gap-x-4 gap-y-3 rounded-card border border-line bg-surface px-4 py-3"
    >
      {usesPeriod ? (
        <>
          <div className="flex flex-col gap-1">
            <span className="label-caps" aria-hidden>
              {t('relatorios-periodo')}
            </span>
            <div className="flex items-center gap-1">
              <IconButton
                variant="secondary"
                label={t('relatorios-periodo-anterior')}
                icon={<Icon name="chevronLeft" size={14} />}
                onClick={() => onChange(stepFilters(filters, today, -1))}
              />
              <Select
                aria-label={t('relatorios-periodo')}
                className="w-48"
                value={filters.preset}
                onValueChange={(v) => {
                  const preset = v as PresetId;
                  if (preset === 'personalizado') {
                    const p = presetPeriod(filters.preset, today, filters);
                    set({ preset, from: p.from, to: p.to });
                  } else set({ preset });
                }}
                options={PRESETS.map((p) => ({ value: p.id, label: p.label }))}
              />
              <IconButton
                variant="secondary"
                label={t('relatorios-proximo-periodo')}
                icon={<Icon name="chevronRight" size={14} />}
                onClick={() => onChange(stepFilters(filters, today, 1))}
              />
            </div>
          </div>
          {filters.preset === 'personalizado' ? (
            <>
              <label className="flex flex-col gap-1">
                <span className="label-caps">{t('relatorios-de')}</span>
                <DateInput
                  aria-label={t('relatorios-data-inicial')}
                  wrapperClassName="w-32"
                  value={filters.from}
                  onValueChange={(v) => {
                    if (v) set({ from: v, to: v > filters.to ? v : filters.to });
                  }}
                />
              </label>
              <label className="flex flex-col gap-1">
                <span className="label-caps">{t('relatorios-ate')}</span>
                <DateInput
                  aria-label={t('relatorios-data-final')}
                  wrapperClassName="w-32"
                  value={filters.to}
                  onValueChange={(v) => {
                    if (v) set({ to: v, from: v < filters.from ? v : filters.from });
                  }}
                />
              </label>
            </>
          ) : null}
        </>
      ) : null}
      <div className="flex flex-col gap-1">
        <span className="label-caps">{t('relatorios-base')}</span>
        <Segmented
          label={t('relatorios-base-datas')}
          value={filters.basis ?? defaultBasis}
          onChange={(b) => set({ basis: b })}
          options={BASIS_OPTIONS}
        />
      </div>
      <label className="flex flex-col gap-1">
        <span className="label-caps">{t('relatorios-membro')}</span>
        <Select
          aria-label={t('relatorios-membro')}
          className="w-40"
          value={filters.member ?? ALL}
          onValueChange={(v) => set({ member: v === ALL ? null : v })}
          options={[
            { value: ALL, label: t('relatorios-todos') },
            ...sortedMembers(members.data ?? []).map((m) => ({ value: m.id, label: m.name })),
          ]}
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="label-caps">{t('relatorios-conta')}</span>
        <Select
          aria-label={t('relatorios-conta')}
          className="w-52"
          value={filters.account ?? ALL}
          onValueChange={(v) => set({ account: v === ALL ? null : v })}
          options={[
            { value: ALL, label: t('relatorios-todas') },
            ...(accounts.data ?? [])
              .filter((a) => !a.archived)
              .map((a) => ({
                value: a.id,
                label: a.name,
                hint: a.currency !== base ? a.currency : undefined,
              })),
          ]}
        />
      </label>
    </div>
  );
}
