/**
 * Cartões que só existem para famílias no Brasil (docs/taxes.md): para onde vai o imposto (o
 * total estimado repartido pela despesa do governo geral por função, do Tesouro Nacional) e a
 * arrecadação da cidade da família, por tributo e por habitante (relatório que a prefeitura
 * entrega ao Tesouro; a lista de municípios é do IBGE). O núcleo só manda esses dados em cofre
 * do Brasil.
 */
import { useState } from 'react';

import type { City } from '@/api/generated/City';
import type { Scenario } from '@/api/generated/Scenario';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import { useRefreshTaxCity, useSetTaxCity, useTaxCities, useTaxCity } from '@/api/hooks/taxBurden';
import { Button, Field, InstrumentCard, Measure, Money, Select, toast } from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, formatNumber } from '@/lib/format';

import { formatRate, pick, sourcesOf } from './model';
import { sourceDate } from './SourcesDialog';

/** Para onde vai o imposto: cada função de governo com a parte dela no total estimado. */
export function DestinationCard({
  estimate: e,
  scenario,
  dataset,
}: {
  estimate: TaxEstimate;
  scenario: Scenario;
  dataset: TaxDataset | undefined;
}) {
  const d = e.destination;
  if (!d) return null;
  const lines = [...d.lines].sort((a, b) => b.share - a.share);
  const top = lines[0]?.share ?? 1;
  return (
    <InstrumentCard
      index="10.11"
      title={t('impostos-destino-titulo')}
      className="@min-[1040px]:col-span-6"
      headingLevel={2}
    >
      <div className="flex flex-col gap-3">
        <p className="text-[12.5px] leading-relaxed text-dim">
          {t('impostos-destino-texto', { ano: String(d.year) })}
        </p>
        <ul className="flex flex-col gap-2" aria-label={d.label}>
          {lines.map((l) => (
            <li key={l.key} className="flex flex-col gap-1">
              <div className="flex items-baseline justify-between gap-3 text-[13px]">
                <span className="text-fg">{l.label}</span>
                <span className="flex items-baseline gap-3 whitespace-nowrap">
                  <span className="num text-dim">{formatRate(l.share, 1)}</span>
                  <Money cents={pick(l.amounts, scenario)} currency={e.currency} estimated />
                </span>
              </div>
              <div className="h-1 rounded-full bg-line/60" aria-hidden>
                <div
                  className="h-1 rounded-full bg-accent/70"
                  style={{ width: `${Math.max(1, (l.share / top) * 100)}%` }}
                />
              </div>
            </li>
          ))}
        </ul>
        <p className="text-[12px] leading-relaxed text-dim">{t('impostos-destino-ressalva')}</p>
        {sourcesOf(dataset, d.source_ids).map((s) => (
          <a
            key={s.id}
            href={s.url}
            target="_blank"
            rel="noreferrer"
            className="text-[12px] leading-snug text-accent underline-offset-2 hover:underline"
          >
            {s.publisher}: {s.title}
            {s.published ? ` (${sourceDate(s.published)})` : ''}
          </a>
        ))}
      </div>
    </InstrumentCard>
  );
}

/** Escolha da cidade: estado e município (lista do IBGE). */
function CityPicker({
  defaultUf,
  regions,
  onCancel,
}: {
  defaultUf: string | null;
  regions: readonly { code: string; name: string }[];
  onCancel?: () => void;
}) {
  const [uf, setUf] = useState(defaultUf ?? '');
  const [code, setCode] = useState('');
  const cities = useTaxCities(uf || null);
  const save = useSetTaxCity();
  const chosen = cities.data?.find((c) => c.code === code);
  const submit = () => {
    if (!chosen) return;
    save.mutate(
      { city: chosen },
      {
        onSuccess: () => onCancel?.(),
        onError: (err) =>
          toast({
            tone: 'error',
            title: t('impostos-cidade-erro-salvar'),
            description: errorMessage(err),
          }),
      },
    );
  };
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] leading-relaxed text-dim">{t('impostos-cidade-escolha-texto')}</p>
      <div className="flex flex-wrap items-end gap-3">
        <Field label={t('impostos-estado')} className="w-48">
          <Select
            aria-label={t('impostos-estado')}
            value={uf}
            placeholder={t('impostos-cidade-escolha-estado')}
            onValueChange={(v) => {
              setUf(v);
              setCode('');
            }}
            options={regions.map((r) => ({ value: r.code, label: r.name, hint: r.code }))}
          />
        </Field>
        <Field label={t('impostos-cidade')} className="w-64">
          <Select
            aria-label={t('impostos-cidade')}
            value={code}
            disabled={!uf || cities.isPending || cities.isError}
            placeholder={
              !uf
                ? t('impostos-cidade-escolha-estado')
                : cities.isPending
                  ? t('impostos-cidade-carregando')
                  : t('impostos-cidade-escolha')
            }
            onValueChange={setCode}
            options={(cities.data ?? []).map((c) => ({ value: c.code, label: c.name }))}
          />
        </Field>
        <Button onClick={submit} disabled={!chosen || save.isPending}>
          {t('impostos-cidade-salvar')}
        </Button>
        {onCancel ? (
          <Button variant="ghost" onClick={onCancel}>
            {t('impostos-cidade-cancelar')}
          </Button>
        ) : null}
      </div>
      {cities.isError ? (
        <p role="alert" className="text-[12.5px] leading-relaxed text-warning">
          {errorMessage(cities.error)}
        </p>
      ) : null}
    </div>
  );
}

/** Arrecadação da cidade da família, por tributo e por habitante. */
export function CityCard({
  estimate: e,
  dataset,
}: {
  estimate: TaxEstimate;
  dataset: TaxDataset | undefined;
}) {
  const state = useTaxCity(e.dataset.country === 'BR');
  const refresh = useRefreshTaxCity();
  const [picking, setPicking] = useState(false);
  if (e.dataset.country !== 'BR' || !state.data) return null;
  const { city, revenue, source_enabled: enabled } = state.data;
  const regions = [...(dataset?.regions ?? [])].sort((a, b) =>
    a.name.localeCompare(b.name, 'pt-BR'),
  );
  const fetchNow = () =>
    refresh.mutate(
      {},
      {
        onError: (err) =>
          toast({
            tone: 'error',
            title: t('impostos-cidade-erro-buscar'),
            description: errorMessage(err),
          }),
      },
    );
  const title = city
    ? t('impostos-cidade-titulo-de', { cidade: cityName(city) })
    : t('impostos-cidade-titulo');
  return (
    <InstrumentCard
      index="10.12"
      title={title}
      className="@min-[1040px]:col-span-6"
      headingLevel={2}
      actions={
        city && !picking ? (
          <Button variant="ghost" size="sm" onClick={() => setPicking(true)}>
            {t('impostos-cidade-trocar')}
          </Button>
        ) : null
      }
    >
      {!city || picking ? (
        <CityPicker
          defaultUf={city?.uf ?? e.settings.region}
          regions={regions}
          onCancel={city ? () => setPicking(false) : undefined}
        />
      ) : (
        <div className="flex flex-col gap-3">
          {revenue ? (
            <>
              <div className="grid grid-cols-2 gap-4">
                <Measure
                  size="md"
                  label={t('impostos-cidade-por-habitante')}
                  value={
                    revenue.per_capita != null ? (
                      <Money cents={revenue.per_capita} currency="BRL" />
                    ) : (
                      '—'
                    )
                  }
                  hint={
                    revenue.population != null
                      ? t('impostos-cidade-habitantes', { n: formatNumber(revenue.population) })
                      : undefined
                  }
                  origin={{ source: t('impostos-cidade-fonte'), status: 'oficial' }}
                />
                <Measure
                  size="md"
                  label={t('impostos-cidade-total')}
                  value={<Money cents={revenue.total} currency="BRL" />}
                  hint={t('impostos-cidade-periodo', { mes: formatMonthLong(revenue.until) })}
                  origin={{ source: t('impostos-cidade-fonte'), status: 'oficial' }}
                />
              </div>
              <ul className="flex flex-col gap-1.5" aria-label={t('impostos-cidade-por-tributo')}>
                {revenue.lines.map((l) => (
                  <li key={l.key} className="flex items-baseline justify-between gap-3 text-[13px]">
                    <span className="text-fg">{l.label}</span>
                    <span className="flex items-baseline gap-3 whitespace-nowrap">
                      {l.per_capita != null ? (
                        <span className="text-dim">
                          <Money cents={l.per_capita} currency="BRL" />{' '}
                          {t('impostos-cidade-por-hab')}
                        </span>
                      ) : null}
                      <Money cents={l.amount} currency="BRL" />
                    </span>
                  </li>
                ))}
              </ul>
              <p className="text-[12px] leading-relaxed text-dim">
                {t('impostos-cidade-ressalva')}
              </p>
              <p className="text-[12px] leading-relaxed text-dim">
                {t('impostos-cidade-conferido', { data: formatDate(revenue.fetched) })}{' '}
                <a
                  href={revenue.source_url}
                  target="_blank"
                  rel="noreferrer"
                  className="text-accent underline-offset-2 hover:underline"
                >
                  {t('impostos-cidade-fonte')}
                </a>
              </p>
            </>
          ) : (
            <p className="text-[12.5px] leading-relaxed text-dim">
              {enabled ? t('impostos-cidade-sem-dados') : t('impostos-cidade-fonte-desligada')}
            </p>
          )}
          {enabled ? (
            <div>
              <Button variant="secondary" size="sm" onClick={fetchNow} disabled={refresh.isPending}>
                {refresh.isPending
                  ? t('impostos-cidade-buscando')
                  : revenue
                    ? t('impostos-cidade-atualizar')
                    : t('impostos-cidade-buscar')}
              </Button>
            </div>
          ) : revenue ? (
            <p className="text-[12px] leading-relaxed text-dim">
              {t('impostos-cidade-fonte-desligada')}
            </p>
          ) : null}
        </div>
      )}
    </InstrumentCard>
  );
}

const cityName = (c: City) => `${c.name} (${c.uf})`;
