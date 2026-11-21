/**
 * Rentabilidade: de onde veio o resultado (ponte em dinheiro), acumulado mensal da carteira
 * × CDI × inflação (deflator escolhido) × Ibovespa e S&P 500 × as séries que a pessoa escolheu
 * comparar, o acumulado real, o risco da série (semidesvio, pior queda, tempo abaixo do topo),
 * o retorno de cada mês (barras e mapa de calor), patrimônio × aportes e as maiores
 * contribuições.
 */
import type { ReturnRisk } from '@/api/generated/ReturnRisk';
import { usePerformance } from '@/api/hooks/portfolio';
import { EmptyState, KeyValue, OriginBadge, Private } from '@/design/components';
import { t } from '@/i18n';
import { formatDate, formatMonthLong, formatPercent, todayIso } from '@/lib/format';

import { BridgeCard } from './BridgeCard';

import {
  CumulativeReturnChart,
  MonthlyReturnChart,
  ValueVsContributionsChart,
} from './charts/PerformanceCharts';
import { DeflatorSelect } from './components/DeflatorSelect';
import { Pct } from './components/Pct';
import { PeriodSelect, periodRange, usePeriod } from './components/PeriodSelect';
import { ErrorBlock, LoadingBlock } from './components/States';
import { CompareWith, MonthlyHeatmap, TopContributors, useCompareLines } from './PortfolioCards';
import { useDeflatorLabel } from './useDeflatorLabel';
import { usePortfolioUi } from './state';

/** Pior queda com as datas do topo, do fundo e da volta ao topo. */
function worstDrop(r: ReturnRisk): string {
  if (!r.drawdown_trough || r.max_drawdown >= 0) return t('carteira-risco-sem-queda');
  const queda = t('carteira-risco-queda-datas', {
    pct: formatPercent(r.max_drawdown, { decimals: 1 }),
    pico: r.drawdown_peak ? formatDate(r.drawdown_peak) : '—',
    fundo: formatDate(r.drawdown_trough),
  });
  return r.drawdown_recovered
    ? t('carteira-risco-voltou-em', { queda, data: formatDate(r.drawdown_recovered) })
    : t('carteira-risco-sem-volta', { queda });
}

/** Linha de risco da série de retornos (a mesma do método da rentabilidade). */
function RiskLine({ risk }: { risk: ReturnRisk }) {
  const days = t('carteira-risco-dias', { n: risk.longest_drawdown_days });
  return (
    <KeyValue
      className="mt-2 border-t border-line pt-2"
      items={[
        {
          label: t('carteira-risco-semidesvio'),
          value: (
            <Private>
              {t('carteira-pct-aa', { pct: formatPercent(risk.semideviation, { decimals: 1 }) })}
            </Private>
          ),
        },
        { label: t('carteira-risco-pior-queda'), value: <Private>{worstDrop(risk)}</Private> },
        {
          label: t('carteira-risco-tempo-abaixo'),
          value: risk.longest_drawdown_ongoing
            ? `${days} · ${t('carteira-risco-ainda-abaixo')}`
            : days,
        },
      ]}
    />
  );
}

export function PerformancePanel() {
  const deflatorLabel = useDeflatorLabel();
  const deflator = usePortfolioUi((s) => s.deflator);
  const [period, setPeriod] = usePeriod('rentabilidade', 'all');
  const today = todayIso();
  const range = periodRange(period, today);
  const args = {
    deflator,
    ...(range.fromMonth ? { from_month: range.fromMonth } : {}),
    ...(range.toMonth ? { to_month: range.toMonth } : {}),
  };
  const q = usePerformance(args);
  const first = q.data?.months[0];
  const lines = useCompareLines(q.data);

  const controls = (
    <div className="flex flex-wrap items-center gap-3">
      <PeriodSelect scope="rentabilidade" value={period} onChange={setPeriod} />
      <DeflatorSelect />
    </div>
  );

  if (q.isPending)
    return (
      <div className="flex flex-col gap-4">
        {controls}
        <LoadingBlock label={t('carteira-calculando-rentabilidade')} />
      </div>
    );
  if (q.isError)
    return (
      <div className="flex flex-col gap-4">
        {controls}
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      </div>
    );
  const perf = q.data;
  if (perf.months.length === 0 || perf.value_base.every((v) => v === 0)) {
    return (
      <div className="flex flex-col gap-4">
        {controls}
        <EmptyState
          title={t('carteira-sem-historico')}
          description={t('carteira-sem-historico-texto')}
        />
      </div>
    );
  }
  const n = perf.months.length - 1;
  const cum = perf.cumulative[n] ?? null;
  const real = perf.cumulative_real[n] ?? null;
  const cdi = perf.cdi_cumulative[n] ?? null;
  const infl = perf.inflation_cumulative[n] ?? null;
  const pctOfCdi = cum != null && cdi != null && cdi !== 0 ? cum / cdi : null;
  // No Brasil, o CDI; fora, o juro de referência do país (docs/countries.md).
  const rate = perf.benchmark_label || 'CDI';
  const rateSource =
    rate === 'CDI' ? 'CDI BCB/SGS 12' : t('carteira-fonte-catalogo', { indice: rate });
  const source = (
    <span className="inline-flex flex-wrap items-center gap-1.5">
      {t('carteira-rent-fonte', { taxa: rateSource, deflator: deflatorLabel(perf.deflator) })}
      <OriginBadge
        source={`${rateSource} · ${deflatorLabel(perf.deflator)} · ${perf.ibov_cumulative.some((x) => x != null) ? 'Ibovespa/S&P 500' : 'S&P 500'} (Yahoo)`}
        date={today}
        status={perf.estimated ? 'estimado' : 'oficial'}
        note={perf.estimated ? t('carteira-rent-estimado') : undefined}
      />
    </span>
  );

  const method =
    perf.method === 'daily' ? t('carteira-risco-metodo-diario') : t('carteira-risco-metodo-mensal');

  return (
    <div className="flex flex-col gap-5">
      {controls}
      <BridgeCard index="01" deflator={deflator} />
      <div className="rounded-card border border-line bg-surface px-4 py-3">
        <KeyValue
          items={[
            {
              label: t('carteira-desde', { mes: first ? formatMonthLong(first) : '' }),
              value: <Pct value={cum} tone="auto" />,
            },
            { label: rate, value: <Pct value={cdi} signed /> },
            {
              label: t('carteira-pct-do', { indice: rate }),
              value: <Pct value={pctOfCdi} decimals={0} />,
            },
            ...(perf.deflator !== 'none'
              ? [
                  {
                    label: deflatorLabel(perf.deflator),
                    value: <Pct value={infl} signed estimated={perf.estimated} />,
                  },
                  {
                    label: t('carteira-acumulado-real'),
                    value: <Pct value={real} tone="auto" estimated={perf.estimated} />,
                  },
                ]
              : []),
          ]}
        />
        {perf.risk ? <RiskLine risk={perf.risk} /> : null}
      </div>
      <CompareWith rate={rate} lines={lines} />
      <CumulativeReturnChart index="02" perf={perf} extra={lines} source={source} />
      <div className="grid gap-5 @min-[1040px]:grid-cols-2">
        <MonthlyReturnChart index="03" perf={perf} />
        <ValueVsContributionsChart index="04" perf={perf} />
      </div>
      <MonthlyHeatmap index="05" perf={perf} rate={rate} />
      <TopContributors index="06" range={range} />
      <p className="text-[12px] text-dim">{t('carteira-risco-como', { metodo: method })}</p>
    </div>
  );
}
