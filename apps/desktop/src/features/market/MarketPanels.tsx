/**
 * Tela Mercado (docs/design/market-explorer.md, seção 7): o panorama por setor e o
 * mapa Saúde × Atratividade do universo, com o score calculado na coleta para todo o mercado.
 * É uma lista de quem cumpre critérios públicos, nunca recomendação (CVM Res. 20/2021): o
 * aviso fica à vista em cada aba.
 */
import { useCallback, useMemo, useState } from 'react';
import { useNavigate } from 'react-router';

import type { MarketMapPoint } from '@/api/generated/MarketMapPoint';
import type { MarketSector } from '@/api/generated/MarketSector';
import { useMarketMap, useMarketOverview } from '@/api/hooks/research';
import { EChart } from '@/charts/EChart';
import type { EChartsOption } from '@/charts/echarts';
import { baseOption, valueAxis, type ChartTheme } from '@/charts/theme';
import { tooltipHtml } from '@/charts/tooltip';
import {
  EmptyState,
  InstrumentCard,
  Measure,
  TechTable,
  type TechColumn,
} from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatDateTime, formatNumber } from '@/lib/format';

import { Pct } from '../portfolio/components/Pct';
import { Segmented } from '../portfolio/components/Segmented';
import { ErrorBlock, LoadingBlock, Notice } from '../portfolio/components/States';
import { subjectHref } from '../research/common';

/** Aviso fixo das abas do Mercado. */
export function NotAdvice() {
  return (
    <p role="note" className="text-[12px] text-dim">
      {t('mercado-aviso')}
    </p>
  );
}

const num = (v: number | null | undefined, d = 1) => (v == null ? '—' : formatNumber(v, d));

/** `onOpen`: o clique no nome do setor abre o explorador (o duplo clique e o Enter na linha
 * também). */
function sectorColumns(
  label: string,
  onOpen: (s: MarketSector) => void,
): TechColumn<MarketSector>[] {
  return [
    {
      key: 'sector',
      header: label,
      cell: (s) => (
        <button
          type="button"
          className="text-left font-sans font-medium text-fg underline-offset-2 hover:text-accent hover:underline"
          title={t('mercado-abrir-setor', { setor: s.sector })}
          onClick={(e) => {
            e.stopPropagation();
            onOpen(s);
          }}
        >
          {s.sector}
        </button>
      ),
      sortValue: (s) => s.sector,
    },
    {
      key: 'count',
      header: t('mercado-col-papeis'),
      width: '9%',
      numeric: true,
      cell: (s) => s.count,
      sortValue: (s) => s.count,
    },
    {
      key: 'healthy',
      header: t('mercado-col-saudaveis'),
      width: '12%',
      numeric: true,
      cell: (s) => `${s.healthy} / ${s.scored}`,
      sortValue: (s) => (s.scored ? s.healthy / s.scored : 0),
    },
    {
      key: 'score',
      header: t('mercado-col-score'),
      width: '10%',
      numeric: true,
      cell: (s) => num(s.median_score, 0),
      sortValue: (s) => s.median_score ?? -1,
    },
    {
      key: 'pl',
      header: t('mercado-col-pl'),
      width: '9%',
      numeric: true,
      cell: (s) => num(s.median_pl),
      sortValue: (s) => s.median_pl ?? Number.MAX_VALUE,
    },
    {
      key: 'pvp',
      header: t('mercado-col-pvp'),
      width: '9%',
      numeric: true,
      cell: (s) => num(s.median_pvp, 2),
      sortValue: (s) => s.median_pvp ?? Number.MAX_VALUE,
    },
    {
      key: 'dy',
      header: t('mercado-col-dy'),
      width: '9%',
      numeric: true,
      cell: (s) => <Pct value={s.median_dy} />,
      sortValue: (s) => s.median_dy ?? -1,
    },
  ];
}

/** Panorama: quantos papéis, quantos com score e as medianas por setor (ações) e segmento (FIIs). */
export function MarketOverviewPanel({
  onSector,
}: {
  onSector: (sector: string, kind: 'acoes' | 'fiis') => void;
}) {
  const q = useMarketOverview();
  const [kind, setKind] = useState<'acoes' | 'fiis'>('acoes');
  if (q.isPending) return <LoadingBlock />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const o = q.data;
  if (o.stocks + o.fiis === 0)
    return (
      <EmptyState title={t('mercado-sem-universo')} description={t('mercado-sem-universo-texto')} />
    );
  const rows = kind === 'acoes' ? o.stock_sectors : o.fii_segments;
  return (
    <div className="flex flex-col gap-4">
      <InstrumentCard index="10.1" title={t('mercado-panorama-titulo')}>
        <div className="grid gap-5 @min-[720px]:grid-cols-4">
          <Measure size="md" label={t('mercado-acoes')} value={o.stocks} />
          <Measure size="md" label={t('mercado-fiis')} value={o.fiis} />
          <Measure
            size="md"
            label={t('mercado-com-score')}
            value={o.scored}
            hint={
              o.scores_as_of
                ? t('mercado-score-de', { data: formatDate(o.scores_as_of) })
                : t('mercado-score-pendente')
            }
          />
          <Measure
            size="md"
            label={t('mercado-dados')}
            value={formatDate(o.as_of)}
            hint={
              o.last_sync_at
                ? t('mercado-coleta-em', { data: formatDateTime(o.last_sync_at) })
                : t('mercado-sem-coleta')
            }
          />
        </div>
      </InstrumentCard>
      <div className="flex flex-wrap items-center gap-3">
        <Segmented<'acoes' | 'fiis'>
          label={t('mercado-panorama-de')}
          value={kind}
          onChange={setKind}
          options={[
            { value: 'acoes', label: t('mercado-acoes-por-setor') },
            { value: 'fiis', label: t('mercado-fiis-por-segmento') },
          ]}
        />
      </div>
      <TechTable
        label={kind === 'acoes' ? t('mercado-acoes-por-setor') : t('mercado-fiis-por-segmento')}
        prefsId={`mercado-panorama-${kind}`}
        rows={rows}
        columns={sectorColumns(
          kind === 'acoes' ? t('mercado-col-setor') : t('mercado-col-segmento'),
          (s) => onSector(s.sector, kind),
        )}
        getRowId={(s) => s.sector}
        onActivate={(s) => onSector(s.sector, kind)}
        defaultSort={{ key: 'count', direction: 'desc' }}
      />
      <p className="text-[12px] text-dim">{t('mercado-panorama-rodape')}</p>
      <NotAdvice />
    </div>
  );
}

const BAND_COLOR: Record<string, keyof ChartTheme> = {
  robusto: 'positive',
  saudavel: 'accent',
  atencao: 'warning',
  risco: 'negative',
};

function mapOption(t0: ChartTheme, points: MarketMapPoint[]): EChartsOption {
  const base = baseOption(t0);
  const caps = points.map((p) => p.market_cap ?? 0).filter((v) => v > 0);
  const max = Math.max(1, ...caps);
  const size = (v: number | null) => 6 + 18 * Math.sqrt(Math.max(0, v ?? 0) / max);
  const color = (p: MarketMapPoint) => String(t0[BAND_COLOR[p.band ?? ''] ?? 'textDim']);
  return {
    ...base,
    legend: { show: false },
    grid: { left: 8, right: 24, top: 24, bottom: 28, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'item',
      formatter: (params: { dataIndex: number }) => {
        const p = points[params.dataIndex];
        if (!p) return '';
        return tooltipHtml(t0, `${p.ticker} · ${p.name}`, [
          { label: t('mercado-mapa-saude'), value: formatNumber(p.health, 0) },
          { label: t('mercado-mapa-atratividade'), value: formatNumber(p.attractiveness, 0) },
          {
            label: t('mercado-col-score'),
            value: p.score == null ? '—' : formatNumber(p.score, 0),
          },
          { label: t('mercado-col-setor'), value: p.sector ?? '—' },
        ]);
      },
    },
    xAxis: {
      ...valueAxis(t0, (v) => formatNumber(v, 0)),
      min: 0,
      max: 100,
      name: t('mercado-mapa-atratividade'),
      nameLocation: 'middle',
      nameGap: 22,
      nameTextStyle: { color: t0.textDim, fontSize: 11 },
    },
    yAxis: {
      ...valueAxis(t0, (v) => formatNumber(v, 0)),
      min: 0,
      max: 100,
      name: t('mercado-mapa-saude'),
      nameTextStyle: { color: t0.textDim, fontSize: 11 },
    },
    series: [
      {
        type: 'scatter',
        data: points.map((p) => ({
          value: [p.attractiveness, p.health],
          symbolSize: size(p.market_cap),
          itemStyle: {
            color: color(p),
            opacity: 0.75,
            borderColor: p.held || p.watched ? String(t0.text) : 'transparent',
            borderWidth: p.held || p.watched ? 2 : 0,
          },
        })),
        markArea: {
          silent: true,
          itemStyle: { color: String(t0.positive), opacity: 0.06 },
          data: [
            [
              { xAxis: 60, yAxis: 60 },
              { xAxis: 100, yAxis: 100 },
            ],
          ],
        },
      },
    ],
  };
}

/** Mapa: Saúde (vertical) × Atratividade (horizontal), bolha pelo valor de mercado. */
export function MarketMapPanel() {
  const q = useMarketMap();
  const navigate = useNavigate();
  const [kind, setKind] = useState<'todos' | 'stock_br' | 'fii'>('todos');
  const points = useMemo(
    () => (q.data ?? []).filter((p) => kind === 'todos' || p.class === kind),
    [q.data, kind],
  );
  const option = useCallback((th: ChartTheme) => mapOption(th, points), [points]);
  if (q.isPending) return <LoadingBlock />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  if (!q.data.length)
    return (
      <EmptyState title={t('mercado-mapa-vazio')} description={t('mercado-mapa-vazio-texto')} />
    );
  return (
    <div className="flex flex-col gap-3">
      <Segmented<'todos' | 'stock_br' | 'fii'>
        label={t('mercado-mapa-classe')}
        value={kind}
        onChange={setKind}
        options={[
          { value: 'todos', label: t('mercado-todos') },
          { value: 'stock_br', label: t('mercado-acoes') },
          { value: 'fii', label: t('mercado-fiis') },
        ]}
      />
      <InstrumentCard index="10.2" title={t('mercado-mapa-titulo')}>
        <EChart
          option={option}
          height={420}
          ariaLabel={t('mercado-mapa-aria', { n: points.length })}
          onClick={(e) => {
            const p = points[e.dataIndex];
            if (p) void navigate(subjectHref(p.asset_id, p.ticker));
          }}
        />
      </InstrumentCard>
      <Notice title={t('mercado-mapa-leitura')}>{t('mercado-mapa-leitura-texto')}</Notice>
      <NotAdvice />
    </div>
  );
}
