/**
 * Gráfico intradiário (1D/5D) da página do ativo: pontos de 5 min do Yahoo (~15 min de
 * atraso), linha tracejada no fechamento anterior, selo de atraso e situação do pregão.
 * Ao abrir a página, pede uma coleta se a cotação tiver mais de 5 min. No modo
 * privacidade, a escala e os valores somem.
 */
import {
  ColorType,
  createChart,
  CrosshairMode,
  LineSeries,
  LineStyle,
  type IChartApi,
  type IPriceLine,
  type ISeriesApi,
  type UTCTimestamp,
} from 'lightweight-charts';
import { useEffect, useRef, useState } from 'react';

import type { Asset } from '@/api/generated/Asset';
import { useIntraday, useRefreshQuotes } from '@/api/hooks/market';
import { readChartTheme } from '@/charts';
import { Button, Private } from '@/design/components';
import { t } from '@/i18n';
import { formatLocale } from '@/lib/format';
import { useUi } from '@/state/ui';

import { Pct } from '../portfolio/components/Pct';
import { Segmented } from '../portfolio/components/Segmented';
import { formatUnitPrice } from '../portfolio/decimal';
import { DelayBadge } from './QuoteBits';

type Range = '1d' | '5d';

const STALE_MS = 5 * 60_000;

function tzTime(ts: number, timezone: string | null, withDay: boolean): string {
  try {
    return new Intl.DateTimeFormat(formatLocale(), {
      timeZone: timezone ?? undefined,
      hour: '2-digit',
      minute: '2-digit',
      ...(withDay ? { day: '2-digit', month: '2-digit' } : {}),
    }).format(new Date(ts * 1000));
  } catch {
    return new Date(ts * 1000).toISOString().slice(11, 16);
  }
}

export function IntradayChart({ asset, height = 220 }: { asset: Asset; height?: number }) {
  const [range, setRange] = useState<Range>('1d');
  const q = useIntraday(asset.id, range);
  const refresh = useRefreshQuotes();
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<{
    chart: IChartApi;
    line: ISeriesApi<'Line'>;
    prev: IPriceLine | null;
  } | null>(null);
  const themeId = useUi((s) => s.theme);
  const privacy = useUi((s) => s.privacy);
  const data = q.data;
  const quote = data?.quote ?? null;
  const tz = data?.timezone ?? null;
  const asked = useRef(false);

  // Cotação ausente ou velha: pede uma coleta (uma vez por abertura da página).
  useEffect(() => {
    if (!data || asked.current || refresh.isPending) return;
    const age = quote ? Date.now() - Date.parse(quote.fetched_at) : Infinity;
    if (age > STALE_MS) {
      asked.current = true;
      refresh.mutate({ asset_ids: [asset.id] });
    }
  }, [data, quote, asset.id, refresh]);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const chart = createChart(el, {
      autoSize: true,
      height,
      // Logo com o link para a TradingView: a licença da biblioteca (Apache 2.0) pede a
      // atribuição e o link visíveis para quem usa (NOTICE, Ajustes › Sobre). Não desligar.
      layout: { attributionLogo: true },
      handleScale: false,
      handleScroll: false,
      rightPriceScale: { scaleMargins: { top: 0.15, bottom: 0.1 } },
    });
    const line = chart.addSeries(LineSeries, { title: '' });
    chartRef.current = { chart, line, prev: null };
    return () => {
      chartRef.current = null;
      chart.remove();
    };
  }, [height]);

  useEffect(() => {
    const c = chartRef.current;
    if (!c) return;
    const th = readChartTheme(themeId, privacy);
    const withDay = range === '5d';
    c.chart.applyOptions({
      layout: {
        background: { type: ColorType.Solid, color: 'transparent' },
        textColor: th.textDim,
        fontFamily: th.fontNumeric,
        fontSize: 11,
      },
      grid: { vertLines: { color: th.line }, horzLines: { color: th.line } },
      rightPriceScale: { borderColor: th.line, visible: !th.privacy },
      timeScale: {
        borderColor: th.line,
        timeVisible: true,
        tickMarkFormatter: (time: number) => tzTime(time, tz, withDay),
      },
      crosshair: {
        mode: CrosshairMode.Magnet,
        vertLine: { color: th.lineStrong, labelBackgroundColor: th.surface2 },
        horzLine: {
          color: th.lineStrong,
          labelBackgroundColor: th.surface2,
          labelVisible: !th.privacy,
        },
      },
      localization: {
        locale: formatLocale(),
        timeFormatter: (time: number) => tzTime(time, tz, true),
        priceFormatter: (p: number) =>
          th.privacy ? '•••••' : formatUnitPrice(String(Number(p.toFixed(6))), asset.currency),
      },
    });
    const up = quote?.change_pct == null || quote.change_pct >= 0;
    c.line.applyOptions({
      color: up ? th.positive : th.negative,
      lineWidth: 2,
      lastValueVisible: !th.privacy,
      priceLineVisible: false,
    });
    c.line.setData(
      (data?.points ?? []).map((p) => ({ time: p.ts as UTCTimestamp, value: Number(p.price) })),
    );
    if (c.prev) {
      c.line.removePriceLine(c.prev);
      c.prev = null;
    }
    if (quote?.previous_close && range === '1d') {
      c.prev = c.line.createPriceLine({
        price: Number(quote.previous_close),
        color: th.textDim,
        lineStyle: LineStyle.Dashed,
        lineWidth: 1,
        axisLabelVisible: !th.privacy,
        title: th.privacy ? '' : t('mercado-fech-anterior'),
      });
    }
    c.chart.timeScale().fitContent();
  }, [data, quote, themeId, privacy, range, tz, asset.currency]);

  const empty = !q.isPending && (data?.points.length ?? 0) === 0;
  return (
    <div className="flex flex-col gap-2" aria-label={t('mercado-intradiaria')}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          {quote ? (
            <>
              <Private>
                <span className="num text-[18px] font-semibold text-fg">
                  {formatUnitPrice(quote.price, quote.currency, { maxDecimals: 4 })}
                </span>
              </Private>
              <Pct value={quote.change_pct} tone="auto" decimals={2} />
              <DelayBadge
                delayMinutes={quote.delay_minutes}
                state={data?.session_now ?? quote.market_state}
                quotedAt={quote.market_time}
                timezone={tz}
              />
            </>
          ) : (
            <span className="text-[12.5px] text-dim">
              {refresh.isPending ? t('mercado-buscando-cotacao') : t('mercado-sem-intradiaria')}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="ghost"
            loading={refresh.isPending}
            onClick={() => refresh.mutate({ asset_ids: [asset.id] })}
          >
            {t('ativos-atualizar')}
          </Button>
          <Segmented<Range>
            size="sm"
            label={t('mercado-periodo-intradiario')}
            value={range}
            onChange={setRange}
            options={[
              { value: '1d', label: t('mercado-periodo-1d') },
              { value: '5d', label: t('mercado-periodo-5d') },
            ]}
          />
        </div>
      </div>
      {refresh.isError ? (
        <p className="text-[12px] text-warning">{t('mercado-nao-buscou')}</p>
      ) : null}
      {refresh.data?.errors.length ? (
        <p className="text-[12px] text-warning">{refresh.data.errors[0]!.error}</p>
      ) : null}
      <div className="relative">
        <div
          ref={ref}
          role="img"
          aria-label={t('mercado-intradiaria-aria', {
            nome: asset.ticker ?? asset.name,
            dias: range === '1d' ? 1 : 5,
          })}
          style={{ width: '100%', height }}
        />
        {empty ? (
          <p className="absolute inset-0 flex items-center justify-center text-[12.5px] text-dim">
            {t('mercado-sem-pontos')}
          </p>
        ) : null}
      </div>
    </div>
  );
}
