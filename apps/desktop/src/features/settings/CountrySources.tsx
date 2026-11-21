/**
 * Ajustes › Fontes de dados: país de referência (catálogo de países: inflação, juros e
 * câmbio de cada um, `indices.catalog` / `indices.set_country`) e a situação de cada
 * provedor de dados (`indices.providers`).
 */
import { useMemo } from 'react';

import type { CatalogSeries } from '@/api/generated/CatalogSeries';
import type { CountryProfile } from '@/api/generated/CountryProfile';
import type { ProviderStatus } from '@/api/generated/ProviderStatus';
import { useCountryCatalog, useProviderStatus, useSetReferenceCountry } from '@/api/hooks/indices';
import {
  Field,
  InstrumentCard,
  KeyValue,
  Select,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { sourceName } from '@/features/portfolio/labels';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { useBaseCurrency } from '@/state/currency';
import { formatDate, formatDateTime } from '@/lib/format';

const AUTO = '__auto';

/** Fonte do câmbio de cada perfil de país (nomes próprios ficam como estão). */
const FX_PROVIDER: Record<string, string> = {
  get bcb_ptax() {
    return t('ajustes-cambio-bcb');
  },
  get ecb() {
    return t('ajustes-cambio-ecb');
  },
  boc: 'Bank of Canada (Valet)',
  boe: 'Bank of England',
};

/** País que o modo automático escolhe pela moeda base. */
const AUTO_COUNTRY: Record<string, string> = {
  get BRL() {
    return t('ajustes-pais-br');
  },
  get USD() {
    return t('ajustes-pais-us');
  },
  get EUR() {
    return t('ajustes-pais-zona-euro');
  },
  get CAD() {
    return t('ajustes-pais-ca');
  },
  get GBP() {
    return t('ajustes-pais-gb');
  },
};

function seriesList(list: readonly CatalogSeries[]) {
  if (list.length === 0) return <span className="text-dim">—</span>;
  return (
    <span className="flex flex-col gap-0.5 font-sans">
      {list.map((s, i) => (
        <span key={s.id} className={cx(i > 0 && 'text-dim')}>
          {s.name}
          <span className="text-dim">
            {' · '}
            {sourceName(s.source)}
          </span>
          {i > 0 && s.priority > 0 ? (
            <span className="text-dim"> {t('ajustes-pais-ref-alternativa')}</span>
          ) : null}
        </span>
      ))}
    </span>
  );
}

function ProfileDetails({ p }: { p: CountryProfile }) {
  const main = p.inflation[0];
  return (
    <div className="flex flex-col gap-3">
      <KeyValue
        layout="stacked"
        items={[
          {
            label: t('ajustes-pais-ref-deflator'),
            value: (
              <span className="font-sans">
                {t('ajustes-pais-ref-deflator-valor', {
                  pais: p.name,
                  serie: main ? main.name : '—',
                })}
              </span>
            ),
          },
          { label: t('ajustes-pais-ref-inflacao'), value: seriesList(p.inflation) },
          { label: t('ajustes-pais-ref-juros'), value: seriesList(p.rates) },
          {
            label: t('ajustes-sync-cambio'),
            value: (
              <span className="flex flex-col gap-0.5 font-sans">
                <span>{FX_PROVIDER[p.fx_provider] ?? p.fx_provider}</span>
                {p.fx.length ? (
                  <span className="text-dim">{p.fx.map((s) => s.name).join(', ')}</span>
                ) : null}
              </span>
            ),
          },
          { label: t('ajustes-pais-ref-moeda'), value: p.currency },
        ]}
      />
      {p.notes ? <p className="text-[12.5px] text-dim">{p.notes}</p> : null}
    </div>
  );
}

/** 11.20 · País de referência. */
export function ReferenceCountryCard() {
  const q = useCountryCatalog();
  const set = useSetReferenceCountry();
  const cat = q.data;
  const family = useBaseCurrency();
  const base = cat?.base_currency ?? family;
  const current = cat?.countries.find((c) => c.country === cat.country);
  const options = [
    {
      value: AUTO,
      label: t('ajustes-pais-ref-auto', {
        moeda: base,
        pais: AUTO_COUNTRY[base] ?? t('ajustes-pais-br'),
      }),
    },
    ...(cat?.countries ?? []).map((c) => ({ value: c.country, label: c.name, hint: c.country })),
  ];
  return (
    <InstrumentCard
      index="12.20"
      title={t('ajustes-pais-ref-titulo')}
      className="@min-[1040px]:col-span-2"
    >
      <div className="grid gap-5 @min-[790px]:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <div className="flex flex-col gap-3">
          <p className="max-w-[64ch] text-[13px] leading-relaxed text-dim">
            {t('ajustes-pais-ref-texto')}
          </p>
          <QueryState loading={q.isLoading} error={q.error}>
            <Field label={t('ajustes-pais-ref-pais')}>
              <Select
                aria-label={t('ajustes-pais-ref-titulo')}
                value={cat ? (cat.automatic ? AUTO : cat.country) : undefined}
                options={options}
                disabled={set.isPending}
                onValueChange={(v) =>
                  set.mutate(v === AUTO ? null : v, {
                    onSuccess: (c) =>
                      toast({
                        tone: 'success',
                        title: t('ajustes-pais-ref-alterado'),
                        description: t('ajustes-pais-ref-alterado-dica', {
                          pais: c.countries.find((p) => p.country === c.country)?.name ?? c.country,
                        }),
                      }),
                  })
                }
              />
            </Field>
          </QueryState>
          <InlineError error={set.error} />
          <p
            role="note"
            className="rounded-card border border-line px-3 py-2 text-[12.5px] leading-relaxed text-dim"
          >
            {t('ajustes-pais-ref-nota', { moeda: base })}
          </p>
        </div>
        {current ? <ProfileDetails p={current} /> : null}
      </div>
    </InstrumentCard>
  );
}

const STATUS: Record<string, { readonly label: string; tone: string }> = {
  ok: {
    get label() {
      return t('ajustes-provedor-em-dia');
    },
    tone: 'text-positive',
  },
  error: {
    get label() {
      return t('ajustes-provedor-erro');
    },
    tone: 'text-warning',
  },
  never: {
    get label() {
      return t('ajustes-provedor-nunca');
    },
    tone: 'text-dim',
  },
  idle: {
    get label() {
      return t('ajustes-provedor-fora');
    },
    tone: 'text-dim',
  },
};

/** Colunas da tabela de provedores (os títulos saem no idioma atual). */
const providerColumns = (): TechColumn<ProviderStatus>[] => [
  {
    key: 'name',
    header: t('ajustes-provedor-col'),
    width: '40%',
    cell: (p) => (
      <span className="flex flex-col leading-tight">
        <a
          className="font-sans font-medium text-fg hover:underline"
          href={p.info.homepage}
          target="_blank"
          rel="noreferrer"
        >
          {p.info.name}
        </a>
        <span className="font-sans text-[11.5px] text-dim">
          {p.info.terms}
          {p.info.key_required ? ` ${t('ajustes-provedor-exige-chave')}` : ''}
        </span>
      </span>
    ),
    sortValue: (p) => p.info.name,
  },
  {
    key: 'series',
    header: t('ajustes-provedor-col-uso'),
    numeric: true,
    width: '9%',
    cell: (p) => t('ajustes-provedor-uso', { uso: p.series_in_use, total: p.series_total }),
    sortValue: (p) => p.series_in_use,
  },
  {
    key: 'fetched',
    header: t('ajustes-provedor-col-coletado'),
    numeric: true,
    width: '16%',
    cell: (p) => (p.last_fetched_at ? formatDateTime(p.last_fetched_at) : '—'),
    sortValue: (p) => p.last_fetched_at,
  },
  {
    key: 'date',
    header: t('ajustes-provedor-col-ate'),
    numeric: true,
    width: '12%',
    cell: (p) => (p.last_date ? formatDate(p.last_date) : '—'),
    sortValue: (p) => p.last_date,
  },
  {
    key: 'status',
    header: t('ajustes-situacao'),
    width: '23%',
    cell: (p) => {
      const s = STATUS[p.status] ?? { label: p.status, tone: 'text-dim' };
      return (
        <span className="flex flex-col font-sans leading-tight">
          <span className={s.tone}>{s.label}</span>
          {p.error ? (
            <span className="truncate text-[11.5px] text-warning" title={p.error}>
              {p.error}
            </span>
          ) : null}
        </span>
      );
    },
    sortValue: (p) => p.status,
  },
];

/** 11.21 · Provedores de dados (em uso primeiro). */
export function ProvidersCard() {
  const q = useProviderStatus();
  const rows = [...(q.data ?? [])].sort((a, b) => Number(b.in_use) - Number(a.in_use));
  const columns = useMemo(() => providerColumns(), []);
  return (
    <InstrumentCard
      index="12.21"
      title={t('ajustes-provedores-titulo')}
      className="@min-[1040px]:col-span-2"
    >
      <div className="flex flex-col gap-3">
        <p className="max-w-[80ch] text-[13px] leading-relaxed text-dim">
          {t('ajustes-provedores-texto')}
        </p>
        <QueryState loading={q.isLoading} error={q.error}>
          <div className="rounded-card border border-line bg-surface">
            <TechTable
              className="[&_table]:table-fixed"
              label={t('ajustes-provedores-titulo')}
              prefsId="Provedores de dados"
              rows={rows}
              columns={columns}
              getRowId={(p) => p.info.id}
              dense
              empty={t('ajustes-provedores-vazio')}
            />
          </div>
        </QueryState>
      </div>
    </InstrumentCard>
  );
}
