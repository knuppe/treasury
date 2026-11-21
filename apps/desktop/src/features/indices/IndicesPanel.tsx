/**
 * Índices e cotações: séries (IPCA, CDI, Selic, IGP-M, M2, US CPI, US M2, Fed Funds,
 * Ibovespa, S&P 500) com a última data oficial e os estimados, gráfico de uma série,
 * câmbio PTAX e o botão que coleta índices + câmbio + cotações (`market.sync`),
 * mostrando o relatório da coleta.
 */
import { useCallback, useMemo, useState, type FormEvent } from 'react';

import {
  useDataSources,
  useIndexSeries,
  useLatestFx,
  useMarketSync,
  useSeriesPoints,
  useSetFredKey,
} from '@/api/hooks/indices';
import type { DataSyncReport } from '@/api/generated/DataSyncReport';
import type { IndexSeries } from '@/api/generated/IndexSeries';
import type { PriceSyncResult } from '@/api/generated/PriceSyncResult';
import type { SeriesSyncResult } from '@/api/generated/SeriesSyncResult';
import {
  barStyle,
  baseOption,
  categoryAxis,
  ChartFrame,
  EChart,
  lineStyle,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';
import {
  Button,
  Field,
  Icon,
  InstrumentCard,
  KeyValue,
  LiveIndicator,
  OriginBadge,
  PasswordInput,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { formatDate, formatDateTime, formatNumber, todayIso } from '@/lib/format';
import { t } from '@/i18n';
import { FormError } from '@/vault/FormError';

import { axisTooltip } from '../portfolio/charts/common';
import { Segmented } from '../portfolio/components/Segmented';
import { ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { formatDecimal } from '../portfolio/decimal';
import { safeDate, sourceName } from '../portfolio/labels';
import { indexChartValues } from '../portfolio/transform';
import { seriesUse } from './labels';

type Range = '1a' | '2a' | '5a';

function back(r: Range, today: string): string {
  const [y, m, d] = today.split('-').map(Number) as [number, number, number];
  const years = r === '1a' ? 1 : r === '2a' ? 2 : 5;
  return new Date(Date.UTC(y - years, m - 1, d)).toISOString().slice(0, 10);
}

function safeDateTime(iso: string | null | undefined): string {
  if (!iso) return '—';
  try {
    return formatDateTime(iso);
  } catch {
    return iso;
  }
}

const FREQ: Record<string, 'indices-freq-diaria' | 'indices-freq-mensal'> = {
  daily: 'indices-freq-diaria',
  monthly: 'indices-freq-mensal',
};

const freqLabel = (f: string) => (FREQ[f] ? t(FREQ[f]) : f);

const SERIES_COLUMNS: TechColumn<IndexSeries>[] = [
  {
    key: 'name',
    get header() {
      return t('indices-serie');
    },
    width: '21%',
    cell: (s) => (
      <span className="flex flex-col leading-tight">
        <span className="font-medium text-fg">{s.name}</span>
        <span className="truncate font-sans text-[11.5px] text-dim">{seriesUse(s.id)}</span>
      </span>
    ),
    sortValue: (s) => s.name,
  },
  {
    key: 'country',
    get header() {
      return t('mercado-pais');
    },
    cell: (s) => s.country,
    width: '5%',
  },
  {
    key: 'unit',
    get header() {
      return t('indices-unidade');
    },
    cell: (s) => s.unit,
    width: '7%',
  },
  {
    key: 'freq',
    get header() {
      return t('indices-freq');
    },
    cell: (s) => freqLabel(s.frequency),
    width: '7%',
  },
  {
    key: 'source',
    width: '13%',
    get header() {
      return t('ativos-fonte');
    },
    cell: (s) => <span className="font-sans text-dim">{sourceName(s.source)}</span>,
  },
  {
    key: 'official',
    get header() {
      return t('indices-oficial-ate');
    },
    width: '11%',
    numeric: true,
    cell: (s) => safeDate(s.last_official_date),
    sortValue: (s) => s.last_official_date,
  },
  {
    key: 'last',
    get header() {
      return t('indices-estimado-ate');
    },
    width: '11%',
    numeric: true,
    cell: (s) =>
      s.last_date && s.last_date !== s.last_official_date ? (
        <span className="text-warning italic">≈ {safeDate(s.last_date)}</span>
      ) : (
        <span className="text-dim">—</span>
      ),
  },
  {
    key: 'count',
    width: '7%',
    get header() {
      return t('indices-pontos');
    },
    numeric: true,
    cell: (s) => formatNumber(s.count),
    sortValue: (s) => s.count,
  },
  {
    key: 'fetched',
    get header() {
      return t('indices-coletado-em');
    },
    width: '18%',
    numeric: true,
    cell: (s) => <span className="text-dim">{safeDateTime(s.last_fetched_at)}</span>,
    sortValue: (s) => s.last_fetched_at,
  },
];

function SeriesChart({ series }: { series: IndexSeries }) {
  const [range, setRange] = useState<Range>('2a');
  const today = todayIso();
  const q = useSeriesPoints({ id: series.id, from: back(range, today), to: today });
  const data = useMemo(() => indexChartValues(series.kind, q.data ?? []), [series.kind, q.data]);
  const bars = series.kind === 'monthly_rate';
  const unit = data.unit ?? series.unit;
  const fmt = useCallback(
    (v: number) =>
      series.kind === 'level' && Math.abs(v) >= 10_000
        ? formatNumber(v, 0)
        : formatNumber(v, bars ? 2 : 2),
    [series.kind, bars],
  );
  const option = useCallback(
    (theme: ChartTheme): EChartsOption => {
      const base = baseOption(theme);
      const labels = data.dates.map((d) =>
        series.frequency === 'monthly'
          ? d.slice(0, 7).split('-').reverse().join('/')
          : formatDate(d),
      );
      const official = data.values.map((v, i) => (data.estimated[i] ? null : v));
      // A série estimada liga no último ponto oficial para a linha não ter buraco.
      const lastOfficial = data.estimated.lastIndexOf(false);
      const estimated = data.values.map((v, i) =>
        data.estimated[i] || i === lastOfficial ? v : null,
      );
      const hasEstimated = data.estimated.some(Boolean);
      const ls = lineStyle();
      const bs = barStyle(theme);
      return {
        ...base,
        legend: {
          ...(base.legend as object),
          show: hasEstimated,
          data: hasEstimated ? [t('indices-oficial'), t('indices-estimado')] : [],
        },
        grid: { left: 8, right: 16, top: hasEstimated ? 36 : 16, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          ...axisTooltip(theme, (v) => (v == null ? '—' : `${fmt(v)} ${unit}`)),
        },
        xAxis: { ...categoryAxis(theme, labels), boundaryGap: bars },
        yAxis: { ...valueAxis(theme, fmt), scale: series.kind === 'level' },
        series: bars
          ? [
              {
                ...bs,
                type: 'bar',
                name: t('indices-oficial'),
                stack: 'v',
                data: official,
                itemStyle: { ...(bs.itemStyle as object), color: theme.accent },
              },
              {
                ...bs,
                type: 'bar',
                name: t('indices-estimado'),
                stack: 'v',
                data: data.values.map((v, i) => (data.estimated[i] ? v : null)),
                itemStyle: { ...(bs.itemStyle as object), color: theme.warning, opacity: 0.75 },
              },
            ]
          : [
              {
                ...ls,
                type: 'line',
                name: t('indices-oficial'),
                data: official,
                color: theme.accent,
                lineStyle: { ...(ls.lineStyle as object), color: theme.accent },
              },
              {
                ...ls,
                type: 'line',
                name: t('indices-estimado'),
                data: estimated.map((v) => (hasEstimated ? v : null)),
                color: theme.warning,
                lineStyle: { ...(ls.lineStyle as object), color: theme.warning, type: 'dashed' },
              },
            ],
      };
    },
    [data, series, unit, fmt, bars],
  );
  type Row = { date: string; value: number; estimated: boolean };
  const rows: Row[] = data.dates
    .map((d, i) => ({ date: d, value: data.values[i]!, estimated: data.estimated[i]! }))
    .reverse();
  return (
    <ChartFrame
      index="02"
      title={
        range === '1a'
          ? t('indices-titulo-meses', { serie: series.name, n: 12 })
          : range === '2a'
            ? t('indices-titulo-meses', { serie: series.name, n: 24 })
            : t('indices-titulo-anos', { serie: series.name, n: 5 })
      }
      actions={
        <Segmented<Range>
          size="sm"
          label={t('indices-periodo')}
          value={range}
          onChange={setRange}
          options={[
            { value: '1a', label: t('ativos-periodo-1a') },
            { value: '2a', label: t('ativos-periodo-2a') },
            { value: '5a', label: t('indices-periodo-5a') },
          ]}
        />
      }
      source={
        <span className="inline-flex items-center gap-1.5">
          {t('indices-fonte-unidade', { fonte: sourceName(series.source), unidade: unit })}
          <OriginBadge
            source={sourceName(series.source)}
            date={series.last_official_date ?? undefined}
            status={series.last_date !== series.last_official_date ? 'estimado' : 'oficial'}
            note={
              series.id === 'ipca'
                ? t('indices-nota-ipca')
                : series.id.startsWith('wb_cpi_')
                  ? t('indices-nota-wb')
                  : undefined
            }
          />
        </span>
      }
      chart={
        q.isPending ? (
          <LoadingBlock />
        ) : q.isError ? (
          <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
        ) : data.values.length === 0 ? (
          <p className="py-10 text-center text-[13px] text-dim">{t('indices-sem-valores')}</p>
        ) : (
          <EChart
            option={option}
            height={260}
            ariaLabel={t('indices-grafico-aria', { serie: series.name, unidade: unit })}
          />
        )
      }
      table={
        <TechTable
          label={t('indices-valores-de', { serie: series.name })}
          prefsId={`Valores de ${series.name}`}
          dense
          rows={rows}
          getRowId={(r) => r.date}
          columns={[
            { key: 'date', header: t('ativos-data'), cell: (r) => formatDate(r.date) },
            {
              key: 'value',
              header: unit,
              numeric: true,
              cell: (r) => (
                <span className={r.estimated ? 'text-warning italic' : undefined}>
                  {r.estimated ? '≈ ' : ''}
                  {fmt(r.value)}
                </span>
              ),
            },
            {
              key: 'status',
              header: t('indices-status'),
              cell: (r) => (r.estimated ? t('indices-estimado') : t('indices-oficial')),
            },
          ]}
        />
      }
    />
  );
}

function ResultTable<T>({
  label,
  prefsId,
  rows,
  columns,
  getRowId,
}: {
  label: string;
  prefsId: string;
  rows: T[];
  columns: TechColumn<T>[];
  getRowId: (r: T) => string;
}) {
  return (
    <div className="rounded-card border border-line bg-surface">
      <TechTable
        label={label}
        prefsId={prefsId}
        rows={rows}
        columns={columns}
        getRowId={getRowId}
        dense
        maxHeight="280px"
      />
    </div>
  );
}

const SYNC_SERIES_COLUMNS: TechColumn<SeriesSyncResult>[] = [
  {
    key: 'id',
    get header() {
      return t('indices-serie');
    },
    cell: (r) => r.series_id,
  },
  {
    key: 'written',
    get header() {
      return t('indices-gravados');
    },
    numeric: true,
    cell: (r) => formatNumber(r.written),
  },
  {
    key: 'last',
    get header() {
      return t('indices-ultimo-oficial');
    },
    numeric: true,
    cell: (r) => safeDate(r.last_date),
  },
  {
    key: 'status',
    get header() {
      return t('indices-resultado');
    },
    cell: (r) =>
      r.error ? (
        <span className="font-sans text-negative">{r.error}</span>
      ) : (
        <span className="text-positive">{t('indices-ok')}</span>
      ),
  },
];

const SYNC_PRICE_COLUMNS: TechColumn<PriceSyncResult>[] = [
  {
    key: 'ticker',
    get header() {
      return t('ativos-form-rotulo');
    },
    cell: (r) => <span className="font-medium">{r.ticker ?? r.asset_id}</span>,
  },
  {
    key: 'source',
    get header() {
      return t('ativos-fonte');
    },
    cell: (r) => <span className="font-sans text-dim">{sourceName(r.source)}</span>,
  },
  {
    key: 'prices',
    get header() {
      return t('indices-cotacoes');
    },
    numeric: true,
    cell: (r) => formatNumber(r.prices),
  },
  {
    key: 'dividends',
    get header() {
      return t('ativos-menu-proventos');
    },
    numeric: true,
    cell: (r) => formatNumber(r.dividends),
  },
  {
    key: 'splits',
    get header() {
      return t('indices-desdobramentos');
    },
    numeric: true,
    cell: (r) => formatNumber(r.splits_added),
  },
  {
    key: 'last',
    get header() {
      return t('indices-ultimo-pregao');
    },
    numeric: true,
    cell: (r) => safeDate(r.last_date),
  },
  {
    key: 'status',
    get header() {
      return t('indices-resultado');
    },
    cell: (r) =>
      r.error ? (
        <span className="font-sans text-negative">{r.error}</span>
      ) : (
        <span className="text-positive">{t('indices-ok')}</span>
      ),
  },
];

export function SyncReportView({ report }: { report: DataSyncReport }) {
  const failedSeries = [...report.indices.series, ...report.indices.fx].filter(
    (s) => s.error,
  ).length;
  const failedPrices = report.prices.assets.filter((a) => a.error).length;
  return (
    <section aria-label={t('indices-relatorio')} className="flex flex-col gap-3">
      <Notice
        tone={report.ok && failedPrices === 0 ? 'info' : 'warning'}
        title={
          report.ok && failedPrices === 0
            ? t('indices-coleta-concluida')
            : t('indices-coleta-falhas', { n: failedSeries + failedPrices })
        }
      >
        {t('indices-coleta-resumo', {
          inicio: safeDateTime(report.indices.started_at),
          fim: safeDateTime(report.prices.finished_at),
          series: report.indices.series.length,
          moedas: report.indices.fx.length,
          ativos: report.prices.assets.length,
        })}
        {report.indices.ipca_estimated_months
          ? t('indices-ipca-estimado', { n: report.indices.ipca_estimated_months })
          : ''}
      </Notice>
      <div className="grid gap-4 @min-[1040px]:grid-cols-2">
        <div className="flex flex-col gap-2">
          <h4 className="label-caps text-fg">{t('indices-indices-cambio')}</h4>
          <ResultTable
            label={t('indices-coleta-indices')}
            prefsId="Coleta de índices e câmbio"
            rows={[...report.indices.series, ...report.indices.fx]}
            columns={SYNC_SERIES_COLUMNS}
            getRowId={(r) => r.series_id}
          />
        </div>
        <div className="flex flex-col gap-2">
          <h4 className="label-caps text-fg">{t('indices-cotacoes')}</h4>
          {report.prices.assets.length ? (
            <ResultTable
              label={t('indices-coleta-cotacoes')}
              prefsId="Coleta de cotações"
              rows={report.prices.assets}
              columns={SYNC_PRICE_COLUMNS}
              getRowId={(r) => r.asset_id}
            />
          ) : (
            <p className="text-[13px] text-dim">{t('indices-sem-fonte')}</p>
          )}
        </div>
      </div>
    </section>
  );
}

function FredKeyForm({ isSet }: { isSet: boolean }) {
  const setKey = useSetFredKey();
  const [key, setKeyText] = useState('');
  const [editing, setEditing] = useState(!isSet);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setKey.mutate(key.trim(), {
      onSuccess: () => {
        setKeyText('');
        setEditing(false);
        toast({
          tone: 'success',
          title: key.trim() ? t('indices-fred-cadastrada') : t('indices-fred-removida'),
        });
      },
    });
  };
  if (!editing) {
    return (
      <div className="flex flex-wrap items-center gap-3 text-[12.5px]">
        <span className="text-positive">{t('indices-fred-cadastrada')}</span>
        <Button size="sm" variant="ghost" onClick={() => setEditing(true)}>
          {t('indices-trocar')}
        </Button>
      </div>
    );
  }
  return (
    <form onSubmit={submit} className="flex flex-col gap-2" aria-label={t('indices-fred')}>
      <Field label={t('indices-fred-rotulo')} hint={t('indices-fred-dica')}>
        <PasswordInput
          value={key}
          onChange={(e) => setKeyText(e.target.value)}
          autoComplete="off"
        />
      </Field>
      <FormError error={setKey.error} />
      <div className="flex gap-2">
        <Button size="sm" type="submit" variant="primary" loading={setKey.isPending}>
          {t('indices-salvar-chave')}
        </Button>
        {isSet ? (
          <Button size="sm" variant="ghost" onClick={() => setEditing(false)}>
            {t('comum-cancelar')}
          </Button>
        ) : null}
      </div>
    </form>
  );
}

export function IndicesPanel() {
  const seriesQ = useIndexSeries();
  const sources = useDataSources();
  const usd = useLatestFx('USD');
  const sync = useMarketSync();
  const [selected, setSelected] = useState<string | null>('ipca');
  // Séries em uso (as de sempre e as do país de referência) e as que já têm dados; o resto
  // do catálogo de países fica em Ajustes › Fontes de dados.
  const rows = useMemo(
    () => (seriesQ.data ?? []).filter((s) => s.in_use || s.count > 0),
    [seriesQ.data],
  );
  const selectedSeries = rows.find((s) => s.id === selected) ?? rows[0] ?? null;
  const usBlocked = sources.data && !sources.data.fred_key_set;

  return (
    <div className="flex flex-col gap-5">
      <div className="grid gap-4 @min-[1040px]:grid-cols-[minmax(0,7fr)_minmax(0,5fr)]">
        <InstrumentCard
          index="01"
          title={t('indices-coleta-titulo')}
          live={sync.isPending ? t('indices-coletando') : undefined}
          footer={[
            { label: t('indices-ultima-coleta'), value: safeDateTime(sources.data?.last_sync_at) },
            { label: t('indices-rotina'), value: t('indices-rotina-valor') },
          ]}
        >
          <div className="flex flex-col gap-3">
            <p className="text-[13px] leading-relaxed text-dim">{t('indices-coleta-ajuda')}</p>
            <div className="flex flex-wrap items-center gap-3">
              <Button
                variant="primary"
                icon={<Icon name="refresh" size={12} />}
                loading={sync.isPending}
                loadingText={t('indices-coletando-reticencias')}
                onClick={() =>
                  sync.mutate(undefined, {
                    onSuccess: (r) =>
                      toast({
                        tone: r.ok ? 'success' : 'warning',
                        title: r.ok ? t('indices-atualizados') : t('indices-concluida-com-falhas'),
                      }),
                    onError: (e) =>
                      toast({
                        tone: 'error',
                        title: t('indices-coleta-falhou'),
                        description: e.message,
                      }),
                  })
                }
              >
                {t('indices-atualizar')}
              </Button>
              {sync.isPending ? (
                <LiveIndicator label={t('indices-pode-demorar')} tone="accent" />
              ) : null}
            </div>
            {sync.isError ? (
              <ErrorBlock error={sync.error} title={t('indices-coleta-falhou')} />
            ) : null}
          </div>
        </InstrumentCard>
        <InstrumentCard index="02" title={t('indices-cambio-fontes')}>
          <div className="flex flex-col gap-3">
            <KeyValue
              layout="stacked"
              items={[
                {
                  label: 'USD → BRL',
                  value: usd.data ? (
                    <span className="inline-flex items-center gap-1.5">
                      {formatDecimal(usd.data.rate, { minDecimals: 4 })}
                      <OriginBadge source={sourceName(usd.data.source)} date={usd.data.date} />
                      <span className="text-dim">
                        {t('indices-em-data', { data: formatDate(usd.data.date) })}
                      </span>
                    </span>
                  ) : (
                    '—'
                  ),
                },
              ]}
            />
            {sources.data ? (
              <FredKeyForm
                key={String(sources.data.fred_key_set)}
                isSet={sources.data.fred_key_set}
              />
            ) : null}
          </div>
        </InstrumentCard>
      </div>

      {usBlocked ? (
        <Notice tone="warning" title={t('indices-sem-fred-titulo')}>
          {t('indices-sem-fred-texto')}
        </Notice>
      ) : null}

      {sync.data ? <SyncReportView report={sync.data} /> : null}

      <section className="flex flex-col gap-2">
        <h3 className="label-caps text-fg">
          {t('indices-figura-series', { figura: t('componentes-figura', { numero: '01' }) })}
        </h3>
        {seriesQ.isPending ? (
          <LoadingBlock />
        ) : seriesQ.isError ? (
          <ErrorBlock error={seriesQ.error} onRetry={() => void seriesQ.refetch()} />
        ) : (
          <div className="rounded-card border border-line bg-surface">
            <TechTable
              className="[&_table]:table-fixed"
              label={t('indices-series')}
              prefsId="Séries de índices"
              rows={rows}
              columns={SERIES_COLUMNS}
              getRowId={(s) => s.id}
              selectedId={selectedSeries?.id ?? null}
              onSelectedChange={(id) => setSelected(id)}
              empty={t('indices-nenhuma-serie')}
            />
          </div>
        )}
        <p className="text-[12px] text-dim">{t('indices-selecione')}</p>
      </section>

      {selectedSeries ? <SeriesChart key={selectedSeries.id} series={selectedSeries} /> : null}
    </div>
  );
}
