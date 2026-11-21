/**
 * Cards da Carteira que olham o período (docs/portfolio-compare.md):
 *
 * - comparar a rentabilidade com qualquer série (ativo do cadastro ou da watchlist, índice do
 *   catálogo, CDI + x% ao ano), até três por vez, no gráfico do acumulado;
 * - mapa de calor dos retornos mensais (ano × mês, com o ano fechado e o CDI do ano);
 * - maiores contribuições do período, em dinheiro e em pontos do retorno;
 * - proventos por mês, ano contra ano (os três últimos anos).
 *
 * Tudo é desempenho passado lado a lado, sem ranking de compra (CVM Res. 20/2021).
 */
import { useCallback, useMemo, useState } from 'react';

import type { BenchmarkTarget } from '@/api/generated/BenchmarkTarget';
import type { Performance } from '@/api/generated/Performance';
import { useAssets } from '@/api/hooks/assets';
import { useIndexSeries } from '@/api/hooks/indices';
import { useBenchmarkSeries, useContributions, useIncome } from '@/api/hooks/portfolio';
import {
  barStyle,
  baseOption,
  categoryAxis,
  ChartFrame,
  compactMoney,
  EChart,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';
import { Button, Chip, ChipGroup, Money, Select, TechTable } from '@/design/components';
import { t } from '@/i18n';
import { baseCurrency, formatDate, monthName, todayIso } from '@/lib/format';
import { usePrivacy } from '@/state/ui';

import { axisTooltip, colorsFor, pctText } from './charts/common';
import type { ExtraLine } from './charts/PerformanceCharts';
import { DecimalInput } from './components/DecimalInput';
import type { PeriodRange } from './components/PeriodSelect';
import { ErrorBlock, LoadingBlock } from './components/States';
import { usePortfolioUi } from './state';

const MAX_COMPARE = 3;
const MONTHS = Array.from({ length: 12 }, (_, i) => i + 1);

/** `0.012` → `+1,2 p.p.` */
function pointsText(v: number | null | undefined, privacy: boolean): string {
  if (privacy) return '•••••';
  if (v == null || !Number.isFinite(v)) return '—';
  return `${pctText(v * 100, { privacy }).replace('%', '')}\u00a0${t('carteira-pp')}`;
}

/** O núcleo devolve a base (fim do mês anterior); a pessoa lê o período a partir do dia seguinte. */
function dayAfter(iso: string): string {
  const d = new Date(`${iso}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + 1);
  return d.toISOString().slice(0, 10);
}

function sameTarget(a: BenchmarkTarget, b: BenchmarkTarget): boolean {
  if (a.kind !== b.kind) return false;
  if (a.kind === 'cdi_plus' && b.kind === 'cdi_plus') return a.spread === b.spread;
  return 'id' in a && 'id' in b && a.id === b.id;
}

// ── Comparar com qualquer série ──────────────────────────────────

type CompareKind = BenchmarkTarget['kind'];

/** As séries escolhidas, já em % nos meses da rentabilidade, para o gráfico do acumulado. */
export function useCompareLines(perf: Performance | undefined): ExtraLine[] {
  const targets = usePortfolioUi((s) => s.compare);
  const results = useBenchmarkSeries(perf?.months ?? [], targets);
  const stamp = results.map((r) => r.dataUpdatedAt).join();
  return useMemo(
    () =>
      results.flatMap((r, i) =>
        r.data
          ? [
              {
                key: `cmp${i}`,
                name: r.data.label,
                values: r.data.cumulative.map((v) =>
                  v == null || !Number.isFinite(v) ? null : Number((v * 100).toFixed(4)),
                ),
              },
            ]
          : [],
      ),
    // `results` muda de identidade a cada render; os dados de cada consulta, não.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [stamp, targets],
  );
}

/** Escolha das séries: tipo, qual série (ou a taxa) e as escolhidas como chips removíveis. */
export function CompareWith({ rate, lines }: { rate: string; lines: readonly ExtraLine[] }) {
  const targets = usePortfolioUi((s) => s.compare);
  const setTargets = usePortfolioUi((s) => s.setCompare);
  const [kind, setKind] = useState<CompareKind>('index');
  const [choice, setChoice] = useState<string | undefined>();
  const [spread, setSpread] = useState<string | null>('2');
  const assets = useAssets({}, { enabled: kind === 'asset' });
  const series = useIndexSeries();

  const assetOptions = (assets.data ?? [])
    .filter((a) => a.class !== 'fixed_income' && a.ticker)
    .map((a) => ({ value: a.id, label: `${a.ticker} · ${a.name}` }))
    .sort((a, b) => a.label.localeCompare(b.label));
  const seriesOptions = (series.data ?? [])
    .filter((s) => s.count > 0 && s.unit !== 'BRL')
    .map((s) => ({ value: s.id, label: s.name }))
    .sort((a, b) => a.label.localeCompare(b.label));

  const next: BenchmarkTarget | null =
    kind === 'cdi_plus'
      ? spread != null && Number.isFinite(Number(spread))
        ? { kind: 'cdi_plus', spread: Number(spread) / 100 }
        : null
      : choice
        ? { kind, id: choice }
        : null;
  const full = targets.length >= MAX_COMPARE;
  const duplicate = next != null && targets.some((x) => sameTarget(x, next));

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[12px] uppercase tracking-[0.08em] text-dim">
          {t('carteira-comparar-com')}
        </span>
        <Select
          aria-label={t('carteira-comparar-tipo')}
          value={kind}
          onValueChange={(v) => {
            setKind(v as CompareKind);
            setChoice(undefined);
          }}
          className="!h-8 !w-auto min-w-[8rem]"
          options={[
            { value: 'index', label: t('carteira-comparar-indice') },
            { value: 'asset', label: t('carteira-comparar-ativo') },
            { value: 'cdi_plus', label: t('carteira-comparar-cdi-mais', { indice: rate }) },
          ]}
        />
        {kind === 'cdi_plus' ? (
          <label className="flex items-center gap-1.5 whitespace-nowrap text-[13px] text-dim">
            <DecimalInput
              aria-label={t('carteira-comparar-taxa')}
              value={spread}
              onValueChange={setSpread}
              maxDecimals={2}
              className="!h-8 w-20"
            />
            {t('carteira-comparar-ao-ano')}
          </label>
        ) : (
          <Select
            aria-label={t('carteira-comparar-escolha')}
            value={choice}
            onValueChange={setChoice}
            placeholder={t('carteira-comparar-escolha')}
            className="!h-8 !w-auto min-w-[12rem]"
            options={kind === 'asset' ? assetOptions : seriesOptions}
          />
        )}
        <Button
          size="sm"
          disabled={!next || full || duplicate}
          onClick={() => {
            if (next) setTargets([...targets, next]);
          }}
        >
          {t('carteira-comparar-adicionar')}
        </Button>
      </div>
      {targets.length > 0 ? (
        <ChipGroup label={t('carteira-comparar-escolhidas')}>
          {targets.map((x, i) => {
            const name = lines.find((l) => l.key === `cmp${i}`)?.name ?? '…';
            return (
              <Chip
                key={JSON.stringify(x)}
                pressed
                aria-label={t('carteira-comparar-remover', { serie: name })}
                onClick={() => setTargets(targets.filter((_, j) => j !== i))}
              >
                {name} <span aria-hidden>×</span>
              </Chip>
            );
          })}
        </ChipGroup>
      ) : null}
      <p className="text-[12px] text-dim">
        {full ? `${t('carteira-comparar-limite', { n: MAX_COMPARE })} ` : ''}
        {t('carteira-comparar-aviso')}
      </p>
    </div>
  );
}

// ── Mapa de calor dos retornos mensais ───────────────────────────

interface HeatRow {
  year: number;
  months: (number | null)[];
  total: number | null;
  rate: number | null;
}

export function heatmapRows(perf: Performance): HeatRow[] {
  const byYear = new Map<number, HeatRow>();
  let prevRate = 0;
  perf.months.forEach((m, i) => {
    const year = Number(m.slice(0, 4));
    const month = Number(m.slice(5, 7));
    let row = byYear.get(year);
    if (!row) {
      row = { year, months: Array<number | null>(12).fill(null), total: null, rate: null };
      byYear.set(year, row);
    }
    const r = perf.monthly[i] ?? null;
    row.months[month - 1] = r;
    if (r != null) row.total = (1 + (row.total ?? 0)) * (1 + r) - 1;
    // O CDI do ano sai do acumulado: (1 + fim do ano) ÷ (1 + fim do ano anterior).
    const cum = perf.cdi_cumulative[i];
    if (cum != null) {
      row.rate = (1 + cum) / (1 + prevRate) - 1;
      if (month === 12) prevRate = cum;
    }
  });
  return [...byYear.values()].sort((a, b) => b.year - a.year);
}

export function MonthlyHeatmap({
  index,
  perf,
  rate,
}: {
  index: string;
  perf: Performance;
  rate: string;
}) {
  const privacy = usePrivacy();
  const rows = useMemo(() => heatmapRows(perf), [perf]);
  const max = Math.max(0.01, ...rows.flatMap((r) => r.months.map((v) => Math.abs(v ?? 0))));
  const bg = (v: number | null) => {
    if (v == null || privacy) return undefined;
    const tone = v >= 0 ? 'var(--positive)' : 'var(--negative)';
    return `color-mix(in srgb, ${tone} ${Math.round((Math.abs(v) / max) * 40) + 6}%, transparent)`;
  };
  const cell = (v: number | null) => pctText(v == null ? null : v * 100, { privacy });
  const header = (
    <tr>
      <th className="pr-2 text-left font-normal text-dim">{t('carteira-mapa-ano')}</th>
      {MONTHS.map((m) => (
        <th key={m} className="px-1 pb-1 text-center font-normal text-dim">
          {monthName(m, true)}
        </th>
      ))}
      <th className="px-1 pb-1 text-center font-normal text-dim">{t('carteira-mapa-total')}</th>
      <th className="px-1 pb-1 text-center font-normal text-dim">{rate}</th>
    </tr>
  );
  const table = (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse text-[11.5px]" aria-label={t('carteira-mapa-aria')}>
        <thead>{header}</thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.year}>
              <th className="num pr-2 text-left font-normal text-dim">{r.year}</th>
              {r.months.map((v, i) => (
                <td
                  key={i}
                  className="num min-w-12 border border-line/40 px-1 py-1 text-center"
                  style={{ background: bg(v) }}
                  aria-label={`${monthName(i + 1)} ${r.year}: ${cell(v)}`}
                >
                  {v == null ? '·' : cell(v)}
                </td>
              ))}
              <td className="num min-w-14 border border-line/40 px-1 py-1 text-center font-medium">
                {cell(r.total)}
              </td>
              <td className="num min-w-14 border border-line/40 px-1 py-1 text-center text-dim">
                {cell(r.rate)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
  return (
    <ChartFrame
      index={index}
      title={t('carteira-mapa-titulo')}
      source={t('carteira-mapa-fonte', { indice: rate })}
      chart={table}
      table={table}
    />
  );
}

// ── Maiores contribuições ────────────────────────────────────────

export function TopContributors({ index, range }: { index: string; range: PeriodRange }) {
  const privacy = usePrivacy();
  const q = useContributions({
    ...(range.fromMonth ? { from_month: range.fromMonth } : {}),
    ...(range.toMonth ? { to_month: range.toMonth } : {}),
  });
  if (q.isPending) return <LoadingBlock label={t('carteira-contrib-calculando')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const r = q.data;
  const top = r.rows.filter((x) => x.result > 0).slice(0, 5);
  const bottom = r.rows
    .filter((x) => x.result < 0)
    .slice(-5)
    .reverse();
  const max = Math.max(1, ...r.rows.map((x) => Math.abs(x.result)));
  const name = (x: (typeof r.rows)[number]) => x.asset.ticker ?? x.asset.name;

  const list = (title: string, items: typeof r.rows, tone: 'positive' | 'negative') => (
    <div className="flex min-w-0 flex-col gap-1.5">
      <h4 className="text-[12px] uppercase tracking-[0.08em] text-dim">{title}</h4>
      {items.length === 0 ? (
        <p className="text-[13px] text-dim">—</p>
      ) : (
        <ul className="flex flex-col gap-1">
          {items.map((x) => (
            <li
              key={x.asset.id}
              className="grid grid-cols-[minmax(0,7rem)_minmax(0,1fr)_auto_auto] items-center gap-2 text-[13px]"
            >
              <span className="truncate" title={x.asset.name}>
                {name(x)}
              </span>
              <span className="h-2 rounded-sm bg-line/40" aria-hidden>
                <span
                  className="block h-2 rounded-sm"
                  style={{
                    width: `${Math.max(2, (Math.abs(x.result) / max) * 100)}%`,
                    background: `var(--${tone})`,
                  }}
                />
              </span>
              <Money cents={x.result} currency={r.base_currency} tone="auto" />
              <span className="num w-20 text-right text-dim">{pointsText(x.points, privacy)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );

  const chart =
    r.rows.length === 0 ? (
      <p className="text-[13px] text-dim">{t('carteira-contrib-vazio')}</p>
    ) : (
      <div className="grid gap-5 @min-[760px]:grid-cols-2">
        {list(t('carteira-contrib-somaram'), top, 'positive')}
        {list(t('carteira-contrib-tiraram'), bottom, 'negative')}
      </div>
    );

  return (
    <ChartFrame
      index={index}
      title={t('carteira-contrib-titulo')}
      source={t('carteira-contrib-fonte', {
        de: formatDate(dayAfter(r.from)),
        ate: formatDate(r.to),
      })}
      chart={chart}
      table={
        <TechTable
          label={t('carteira-contrib-tabela')}
          prefsId="Contribuição de cada ativo"
          dense
          rows={r.rows}
          getRowId={(x) => x.asset.id}
          columns={[
            { key: 'asset', header: t('carteira-contrib-ativo'), cell: (x) => name(x) },
            {
              key: 'result',
              header: t('carteira-contrib-resultado'),
              numeric: true,
              cell: (x) => <Money cents={x.result} currency={r.base_currency} tone="auto" />,
            },
            {
              key: 'income',
              header: t('carteira-contrib-proventos'),
              numeric: true,
              cell: (x) => <Money cents={x.income} currency={r.base_currency} />,
            },
            {
              key: 'points',
              header: t('carteira-contrib-pontos'),
              numeric: true,
              cell: (x) => pointsText(x.points, privacy),
            },
          ]}
        />
      }
    />
  );
}

// ── Proventos por mês, ano contra ano ────────────────────────────

/** Proventos recebidos por ano e mês, na moeda base (menor unidade). */
export function incomeByYearMonth(
  events: readonly {
    pay_date: string;
    net: number;
    currency: string;
    fx_rate: string | null;
    status: string;
  }[],
  years: readonly number[],
  base: string,
): Map<number, number[]> {
  const out = new Map(years.map((y) => [y, Array<number>(12).fill(0)]));
  for (const e of events) {
    if (e.status !== 'received') continue;
    const row = out.get(Number(e.pay_date.slice(0, 4)));
    if (!row) continue;
    const value =
      e.currency === base ? e.net : e.fx_rate ? Math.round(e.net * Number(e.fx_rate)) : null;
    if (value == null) continue;
    const m = Number(e.pay_date.slice(5, 7)) - 1;
    row[m] = (row[m] ?? 0) + value;
  }
  return out;
}

export function IncomeByMonth({ index = '01' }: { index?: string }) {
  const year = Number(todayIso().slice(0, 4));
  const years = useMemo(() => [year - 2, year - 1, year], [year]);
  const q = useIncome({ from: `${year - 2}-01-01` });
  const base = baseCurrency();
  const data = useMemo(() => incomeByYearMonth(q.data ?? [], years, base), [q.data, years, base]);
  const labels = MONTHS.map((m) => monthName(m, true));
  const option = useCallback(
    (th: ChartTheme): EChartsOption => {
      const color = colorsFor(years.map(String), th);
      const b = baseOption(th);
      const bs = barStyle(th);
      return {
        ...b,
        grid: { left: 8, right: 16, top: 36, bottom: 8, containLabel: true },
        legend: { ...(b.legend as object), data: years.map(String) },
        tooltip: {
          ...(b.tooltip as object),
          ...axisTooltip(th, (v) => (v == null ? '—' : tooltipMoney(v, th))),
        },
        xAxis: categoryAxis(th, labels),
        yAxis: valueAxis(th, (v) => compactMoney(v, th)),
        series: years.map((y) => ({
          ...bs,
          type: 'bar',
          name: String(y),
          data: data.get(y) ?? [],
          color: color(String(y)),
          itemStyle: { ...(bs.itemStyle as object), color: color(String(y)) },
        })),
      };
    },
    [data, years, labels],
  );
  if (q.isPending) return <LoadingBlock label={t('carteira-prov-mes-calculando')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const totals = years.map((y) => (data.get(y) ?? []).reduce((s, v) => s + v, 0));
  if (totals.every((v) => v === 0)) return null;

  type Row = { month: number; values: number[] };
  const rows: Row[] = MONTHS.map((m) => ({
    month: m,
    values: years.map((y) => data.get(y)?.[m - 1] ?? 0),
  }));
  return (
    <ChartFrame
      index={index}
      title={t('carteira-prov-mes-titulo')}
      source={t('carteira-prov-mes-fonte', { moeda: base })}
      chart={<EChart option={option} height={240} ariaLabel={t('carteira-prov-mes-aria')} />}
      table={
        <TechTable
          label={t('carteira-prov-mes-tabela')}
          prefsId="Proventos por mês e ano"
          dense
          rows={rows}
          getRowId={(r) => String(r.month)}
          columns={[
            { key: 'month', header: t('carteira-mes'), cell: (r) => monthName(r.month) },
            ...years.map((y, i) => ({
              key: String(y),
              header: String(y),
              numeric: true,
              cell: (r: Row) => <Money cents={r.values[i] ?? 0} currency={base} />,
            })),
          ]}
        />
      }
    />
  );
}
