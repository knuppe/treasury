/**
 * Rankings (`research.rank`): qualquer indicador do catálogo vira ranking sobre o universo
 * do dia, na ordem natural do indicador (menor P/L primeiro, maior DY primeiro), com recorte
 * por classe, setor, segmento, liquidez mínima e "só carteira e watchlist".
 */
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router';

import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { RankItem } from '@/api/generated/RankItem';
import type { SortOrder } from '@/api/generated/SortOrder';
import type { UniverseFilter } from '@/api/generated/UniverseFilter';
import { useCatalog, useRank } from '@/api/hooks/research';
import { Chip, ChipGroup, Field, Select, TechTable, type TechColumn } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

import { Segmented } from '../portfolio/components/Segmented';
import { ErrorBlock, LoadingBlock } from '../portfolio/components/States';
import { CLASS_LABEL, safeDate } from '../portfolio/labels';
import { currencyOfClass, NoUniverse, subjectHref, TickerLink } from './common';
import { compactAmount, formatIndicatorValue, unitAmount } from './format';
import { GROUP_LABEL } from './labels';
import { cleanUniverse } from './query';
import { UniverseFilters } from './UniverseFilters';

type OrderMode = 'natural' | SortOrder;

interface Preset {
  label: string;
  indicator: string;
  filters?: UniverseFilter;
}

const presets = (): Preset[] => [
  { label: t('pesquisa-rank-maiores-dy'), indicator: 'dy' },
  { label: t('pesquisa-rank-menor-pl'), indicator: 'pl', filters: { classes: ['stock_br'] } },
  { label: t('pesquisa-rank-margem-graham'), indicator: 'graham_upside' },
  { label: t('pesquisa-rank-margem-bazin'), indicator: 'bazin_upside' },
  {
    label: t('pesquisa-rank-sem-prejuizo'),
    indicator: 'loss_years_10y',
    filters: { classes: ['stock_br'] },
  },
  { label: t('pesquisa-rank-maior-roe'), indicator: 'roe' },
  { label: t('pesquisa-rank-valor-mercado'), indicator: 'market_cap' },
  { label: t('pesquisa-rank-liquidez-fii'), indicator: 'liquidity', filters: { classes: ['fii'] } },
  { label: t('pesquisa-rank-pvp-fii'), indicator: 'pvp', filters: { classes: ['fii'] } },
];

export function naturalOrderLabel(info: IndicatorInfo | undefined): string {
  if (!info) return '';
  if (info.higher_is_better === false) return t('pesquisa-rank-menor-primeiro');
  return t('pesquisa-rank-maior-primeiro');
}

function columns(info: IndicatorInfo): TechColumn<RankItem>[] {
  return [
    { key: 'position', header: '#', numeric: true, width: '48px', cell: (r) => r.position },
    {
      key: 'ticker',
      header: t('pesquisa-scr-ticker'),
      width: '96px',
      cell: (r) => <TickerLink assetId={r.asset_id} ticker={r.ticker} />,
    },
    {
      key: 'name',
      header: t('pesquisa-rank-nome'),
      cell: (r) => (
        <span className="flex flex-col font-sans leading-tight">
          <span className="truncate">{r.name}</span>
          <span className="truncate text-[11.5px] text-dim">
            {[CLASS_LABEL[r.class], r.sector, r.segment].filter(Boolean).join(' · ')}
          </span>
        </span>
      ),
    },
    {
      key: 'value',
      header: info.label,
      numeric: true,
      cell: (r) => (
        <span className="font-medium text-fg">
          {formatIndicatorValue(r.value, info.unit, currencyOfClass(r.class))}
        </span>
      ),
    },
    {
      key: 'price',
      header: t('pesquisa-cotacao'),
      numeric: true,
      cell: (r) => (r.price == null ? '—' : unitAmount(r.price, currencyOfClass(r.class))),
    },
    {
      key: 'liquidity',
      header: t('pesquisa-rank-liquidez'),
      numeric: true,
      cell: (r) =>
        r.liquidity == null ? '—' : compactAmount(r.liquidity, currencyOfClass(r.class)),
    },
  ];
}

export function Rankings() {
  const navigate = useNavigate();
  const catalog = useCatalog();
  const [indicator, setIndicator] = useState('dy');
  const [mode, setMode] = useState<OrderMode>('natural');
  const [limit, setLimit] = useState(50);
  const [filters, setFilters] = useState<UniverseFilter>({});
  const [filtersKey, setFiltersKey] = useState(0);
  const universe = cleanUniverse(filters);
  const q = useRank({
    indicator,
    limit,
    ...(mode === 'natural' ? {} : { order: mode }),
    ...(universe ? { filters: universe } : {}),
  });
  const info = catalog.data?.find((i) => i.key === indicator);
  const options = useMemo(
    () =>
      (catalog.data ?? [])
        .filter((i) => i.key !== 'price')
        .map((i) => ({ value: i.key, label: i.label, hint: GROUP_LABEL[i.group] })),
    [catalog.data],
  );
  const applyPreset = (p: Preset) => {
    setIndicator(p.indicator);
    setMode('natural');
    setFilters(p.filters ?? {});
    setFiltersKey((k) => k + 1);
  };

  return (
    <div className="flex flex-col gap-4">
      <ChipGroup label={t('pesquisa-rank-prontos')}>
        {presets().map((p) => (
          <Chip
            key={p.label}
            pressed={p.indicator === indicator && mode === 'natural'}
            onClick={() => applyPreset(p)}
          >
            {p.label}
          </Chip>
        ))}
      </ChipGroup>
      <div className="flex flex-wrap items-end gap-3">
        <Field label={t('pesquisa-rank-indicador')} className="w-72">
          <Select value={indicator} onValueChange={setIndicator} options={options} />
        </Field>
        <div className="flex flex-col gap-1.5">
          <span className="label-caps">{t('pesquisa-rank-ordem')}</span>
          <Segmented<OrderMode>
            label={t('pesquisa-rank-ordem-rotulo')}
            value={mode}
            onChange={setMode}
            options={[
              {
                value: 'natural',
                label: t('pesquisa-ordem-natural'),
                hint: naturalOrderLabel(info),
              },
              { value: 'desc', label: t('pesquisa-maior-primeiro') },
              { value: 'asc', label: t('pesquisa-menor-primeiro') },
            ]}
          />
        </div>
        <Field label={t('pesquisa-rank-mostrar')} className="w-28">
          <Select
            value={String(limit)}
            onValueChange={(v) => setLimit(Number(v))}
            options={[20, 50, 100, 500].map((n) => ({ value: String(n), label: String(n) }))}
          />
        </Field>
      </div>
      {info ? (
        <p className="text-[12.5px] text-dim">
          <span className="text-fg">{info.label}</span> · {info.formula}
          {mode === 'natural'
            ? ` · ${t('pesquisa-rank-ordem-natural', { ordem: naturalOrderLabel(info) })}`
            : ''}
        </p>
      ) : null}
      <UniverseFilters key={filtersKey} value={filters} onChange={setFilters} />
      {q.isPending || catalog.isPending ? (
        <LoadingBlock label={t('pesquisa-rank-montando')} />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.total === 0 && q.data.without_value === 0 && !universe ? (
        <NoUniverse what="rankings" />
      ) : (
        <>
          <div
            className={cx(
              'rounded-card border border-line bg-surface transition-opacity',
              q.isFetching && 'opacity-70',
            )}
          >
            <TechTable
              className="[&_table]:table-fixed"
              label={t('pesquisa-rank-tabela', { indicador: q.data.indicator.label })}
              prefsId={`Ranking · ${q.data.indicator.label}`}
              rows={q.data.items}
              columns={columns(q.data.indicator)}
              getRowId={(r) => r.ticker}
              onActivate={(r) => void navigate(subjectHref(r.asset_id, r.ticker))}
              empty={t('pesquisa-rank-vazio')}
              maxHeight="640px"
              dense
            />
          </div>
          <p className="text-[12px] text-dim">
            {[
              t('pesquisa-rank-total', { n: q.data.total }),
              ...(q.data.without_value
                ? [
                    q.data.indicator.unit === 'multiple'
                      ? t('pesquisa-rank-sem-valor-multiplo', { n: q.data.without_value })
                      : t('pesquisa-rank-sem-valor', { n: q.data.without_value }),
                  ]
                : []),
              t('pesquisa-rank-rodape', { data: safeDate(q.data.as_of) }),
            ].join(' · ')}
          </p>
        </>
      )}
    </div>
  );
}
