/**
 * Gráfico de preço (TradingView Lightweight Charts): fechamentos, compras e vendas
 * marcadas e a linha do preço médio em degraus, com as cores do tema. Uma legenda
 * no topo faz o papel do tooltip: acompanha a mira e mostra data, cotação, preço
 * médio e a operação do dia.
 */
import {
  ColorType,
  createChart,
  createSeriesMarkers,
  isBusinessDay,
  CrosshairMode,
  LineSeries,
  LineStyle,
  LineType,
  type IChartApi,
  type ISeriesApi,
  type ISeriesMarkersPluginApi,
  type MouseEventParams,
  type SeriesMarker,
  type Time,
} from 'lightweight-charts';
import { useEffect, useMemo, useRef, useState } from 'react';

import type { CorporateAction } from '@/api/generated/CorporateAction';
import type { PricePoint } from '@/api/generated/PricePoint';
import type { Trade } from '@/api/generated/Trade';
import { readChartTheme, type ChartTheme } from '@/charts';
import { t } from '@/i18n';
import { formatDate, formatLocale } from '@/lib/format';
import { useUi } from '@/state/ui';

import { formatQuantity, formatUnitPrice } from '../decimal';
import { priceChartData, type PriceChartData } from '../transform';

export interface PriceChartProps {
  prices: readonly PricePoint[];
  trades: readonly Trade[];
  actions?: readonly CorporateAction[];
  currency: string;
  height?: number;
  ariaLabel: string;
}

function money(v: number, currency: string, privacy: boolean): string {
  if (privacy) return '•••••';
  return formatUnitPrice(String(Number(v.toFixed(8))), currency, { maxDecimals: 4 });
}

function applyTheme(
  chart: IChartApi,
  price: ISeriesApi<'Line'>,
  avg: ISeriesApi<'Line'>,
  t: ChartTheme,
  currency: string,
) {
  chart.applyOptions({
    layout: {
      background: { type: ColorType.Solid, color: 'transparent' },
      textColor: t.textDim,
      fontFamily: t.fontNumeric,
      fontSize: 11,
    },
    grid: { vertLines: { color: t.line }, horzLines: { color: t.line } },
    rightPriceScale: { borderColor: t.line, visible: !t.privacy },
    timeScale: { borderColor: t.line },
    crosshair: {
      mode: CrosshairMode.Magnet,
      vertLine: { color: t.lineStrong, labelBackgroundColor: t.surface2 },
      horzLine: { color: t.lineStrong, labelBackgroundColor: t.surface2, labelVisible: !t.privacy },
    },
    localization: {
      locale: formatLocale() === 'xx' ? 'pt-BR' : formatLocale(),
      dateFormat: formatLocale() === 'en' ? 'MM/dd/yyyy' : 'dd/MM/yyyy',
      priceFormatter: (p: number) => money(p, currency, t.privacy),
    },
  });
  price.applyOptions({
    color: t.accent,
    lineWidth: 2,
    lastValueVisible: !t.privacy,
    priceLineVisible: false,
  });
  avg.applyOptions({
    color: t.warning,
    lineWidth: 2,
    lineStyle: LineStyle.Dashed,
    lineType: LineType.WithSteps,
    lastValueVisible: !t.privacy,
    priceLineVisible: false,
    crosshairMarkerVisible: false,
  });
}

function markersFor(data: PriceChartData, t: ChartTheme): SeriesMarker<Time>[] {
  return data.markers.map((m) => ({
    time: m.time,
    position: m.side === 'buy' ? 'belowBar' : 'aboveBar',
    shape: m.side === 'buy' ? 'arrowUp' : 'arrowDown',
    color: m.side === 'buy' ? t.positive : t.negative,
    text: m.text,
    size: 1,
  }));
}

interface Readout {
  date: string;
  close: number | null;
  avg: number | null;
  trades: Trade[];
}

export function PriceChart({
  prices,
  trades,
  actions = [],
  currency,
  height = 320,
  ariaLabel,
}: PriceChartProps) {
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<{
    chart: IChartApi;
    price: ISeriesApi<'Line'>;
    avg: ISeriesApi<'Line'>;
    markers: ISeriesMarkersPluginApi<Time>;
  } | null>(null);
  const themeId = useUi((s) => s.theme);
  const privacy = useUi((s) => s.privacy);
  const data = useMemo(() => priceChartData(prices, trades, actions), [prices, trades, actions]);
  const [hover, setHover] = useState<Readout | null>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const chart = createChart(el, {
      autoSize: true,
      height,
      // Logo com o link para a TradingView: a licença da biblioteca (Apache 2.0) pede a
      // atribuição e o link visíveis para quem usa (NOTICE, Ajustes › Sobre). Não desligar.
      layout: { attributionLogo: true },
      handleScale: { axisPressedMouseMove: { time: true, price: false } },
      rightPriceScale: { scaleMargins: { top: 0.12, bottom: 0.12 } },
    });
    const price = chart.addSeries(LineSeries, { title: '' });
    const avg = chart.addSeries(LineSeries, { title: t('carteira-grafico-pm') });
    const markers = createSeriesMarkers(price, []);
    chartRef.current = { chart, price, avg, markers };
    return () => {
      chartRef.current = null;
      chart.remove();
    };
  }, [height]);

  // Dados, marcadores e tema.
  useEffect(() => {
    const c = chartRef.current;
    if (!c) return;
    const t = readChartTheme(themeId, privacy);
    applyTheme(c.chart, c.price, c.avg, t, currency);
    c.price.setData(data.line.map((p) => ({ time: p.time, value: p.value })));
    c.avg.setData(data.avg.map((p) => ({ time: p.time, value: p.value })));
    c.markers.setMarkers(markersFor(data, t));
    c.chart.timeScale().fitContent();
  }, [data, themeId, privacy, currency]);

  // Legenda que acompanha a mira (o "tooltip" do gráfico).
  useEffect(() => {
    const c = chartRef.current;
    if (!c) return;
    const handler = (param: MouseEventParams<Time>) => {
      if (!param.time) {
        setHover(null);
        return;
      }
      const date = timeToIso(param.time);
      const close = param.seriesData.get(c.price) as { value?: number } | undefined;
      const avgPoint = [...data.avg].reverse().find((p) => p.time <= date);
      const ids = new Set(data.markers.filter((m) => m.time === date).map((m) => m.tradeId));
      setHover({
        date,
        close: close?.value ?? null,
        avg: avgPoint?.value ?? null,
        trades: trades.filter((tr) => ids.has(tr.id)),
      });
    };
    c.chart.subscribeCrosshairMove(handler);
    return () => c.chart.unsubscribeCrosshairMove(handler);
  }, [data, trades]);

  const last = data.line[data.line.length - 1];
  const lastAvg = data.avg[data.avg.length - 1];
  const shown: Readout | null =
    hover ??
    (last ? { date: last.time, close: last.value, avg: lastAvg?.value ?? null, trades: [] } : null);

  return (
    <div className="flex flex-col gap-2">
      <div
        aria-live="polite"
        className="num flex min-h-5 flex-wrap items-baseline gap-x-4 gap-y-1 text-[11.5px]"
      >
        {shown ? (
          <>
            <span className="label-caps text-fg">{safeDate(shown.date)}</span>
            <span className="text-dim">
              {data.hasPrices
                ? t('carteira-grafico-fechamento')
                : t('carteira-grafico-preco-operacao')}{' '}
              <span className="text-accent">
                {shown.close == null ? '—' : money(shown.close, currency, privacy)}
              </span>
            </span>
            <span className="text-dim">
              {t('carteira-col-preco-medio')}{' '}
              <span className="text-warning">
                {shown.avg == null ? '—' : money(shown.avg, currency, privacy)}
              </span>
            </span>
            {shown.trades.map((tr) => (
              <span key={tr.id} className={tr.side === 'buy' ? 'text-positive' : 'text-negative'}>
                {tr.side === 'buy' ? `▲ ${t('carteira-compra')}` : `▼ ${t('carteira-venda')}`}{' '}
                {privacy ? '•••••' : formatQuantity(tr.quantity)}
                {' @ '}
                {privacy ? '•••••' : formatUnitPrice(tr.price, currency)}
              </span>
            ))}
          </>
        ) : null}
      </div>
      <div ref={ref} role="img" aria-label={ariaLabel} style={{ width: '100%', height }} />
    </div>
  );
}

/** Tempo do Lightweight Charts (texto, dia útil ou timestamp UTC) → `YYYY-MM-DD`. */
function timeToIso(time: Time): string {
  if (typeof time === 'string') return time;
  if (isBusinessDay(time)) {
    return `${time.year}-${String(time.month).padStart(2, '0')}-${String(time.day).padStart(2, '0')}`;
  }
  return new Date(time * 1000).toISOString().slice(0, 10);
}

function safeDate(iso: string): string {
  try {
    return formatDate(iso);
  } catch {
    return iso;
  }
}
