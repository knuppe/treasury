import { useCallback, useMemo } from 'react';

import type { ClassSummary } from '@/api/generated/ClassSummary';
import {
  baseOption,
  categoryAxis,
  ChartFrame,
  EChart,
  readChartTheme,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import type { EChartsOption } from '@/charts/echarts';
import { Money, TechTable } from '@/design/components';
import { t as tr } from '@/i18n';
import { formatPercent } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { usePrivacy, useUi } from '@/state/ui';

import { Pct } from '../components/Pct';
import { allocationSlices, type AllocationSlice } from '../transform';
import { numText, pctText } from './common';

function useSlices(byClass: readonly ClassSummary[]) {
  const theme = useUi((s) => s.theme);
  const privacy = useUi((s) => s.privacy);
  return useMemo(() => {
    const t = readChartTheme(theme, privacy);
    return allocationSlices(byClass, theme, t.textDim);
  }, [byClass, theme, privacy]);
}

/** FIG · Alocação por classe (donut), com o total no centro. */
export function AllocationDonut({
  index,
  byClass,
  total,
}: {
  index: string;
  byClass: readonly ClassSummary[];
  total: number;
}) {
  const slices = useSlices(byClass);
  const privacy = usePrivacy();
  const currency = useBaseCurrency();
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      const base = baseOption(t);
      return {
        ...base,
        legend: { show: false },
        title: {
          text: t.privacy ? '•••••' : tooltipMoney(total, t).replace(/,\d+$/, ''),
          subtext: tr('carteira-investido'),
          left: 'center',
          top: '40%',
          textStyle: { color: t.text, fontFamily: t.fontNumeric, fontSize: 15, fontWeight: 500 },
          subtextStyle: { color: t.textDim, fontFamily: t.fontNumeric, fontSize: 10.5 },
        },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'item',
          formatter: (p: unknown) => {
            const d = (p as { data: { slice: AllocationSlice } }).data.slice;
            const target =
              d.targetPct == null
                ? tr('carteira-sem-alvo')
                : tr('carteira-alvo-pct', { pct: pctText(d.targetPct, t, false) });
            return `<b>${d.label}</b><br/>${tooltipMoney(d.value, t)} · ${pctText(d.weight * 100, t, false)}<br/><span style="color:${t.textDim}">${target}</span>`;
          },
        },
        series: [
          {
            type: 'pie',
            radius: ['60%', '80%'],
            avoidLabelOverlap: true,
            label: { show: false },
            labelLine: { show: false },
            itemStyle: { borderColor: t.surface, borderWidth: 2, borderRadius: 2 },
            emphasis: { scale: false },
            data: slices.map((s) => ({
              name: s.label,
              value: s.value,
              slice: s,
              itemStyle: { color: s.color },
            })),
          },
        ],
      };
    },
    [slices, total],
  );
  return (
    <ChartFrame
      index={index}
      title={tr('carteira-alocacao-classe')}
      source={tr('carteira-alocacao-fonte', { moeda: currency })}
      chart={
        <div className="grid items-center gap-4 @min-[400px]:grid-cols-[200px_minmax(0,1fr)]">
          <EChart option={option} height={220} ariaLabel={tr('carteira-alocacao-aria')} />
          <ul className="flex flex-col gap-1.5">
            {slices.map((s) => (
              <li key={s.key} className="flex items-baseline gap-2 text-[12.5px]">
                <span
                  aria-hidden
                  className="inline-block size-2.5 shrink-0 rounded-[2px]"
                  style={{ background: s.color }}
                />
                <span className="min-w-0 flex-1 truncate text-fg">{s.label}</span>
                <span className="num text-dim">{privacy ? '•••••' : formatPercent(s.weight)}</span>
              </li>
            ))}
          </ul>
        </div>
      }
      table={
        <TechTable
          label={tr('carteira-alocacao-classe')}
          prefsId="Alocação por classe"
          dense
          rows={slices}
          getRowId={(s) => s.key}
          columns={[
            { key: 'label', header: tr('carteira-col-classe'), cell: (s) => s.label },
            {
              key: 'value',
              header: tr('carteira-col-valor'),
              numeric: true,
              cell: (s) => <Money cents={s.value} />,
            },
            {
              key: 'weight',
              header: tr('carteira-col-peso'),
              numeric: true,
              cell: (s) => <Pct value={s.weight} />,
            },
            {
              key: 'target',
              header: tr('carteira-col-alvo'),
              numeric: true,
              cell: (s) => (s.targetPct == null ? '—' : <Pct value={s.targetPct / 100} />),
            },
          ]}
        />
      }
    />
  );
}

/** FIG · Desvio do alvo por classe, em pontos percentuais (barras horizontais). */
export function DeviationChart({
  index,
  byClass,
}: {
  index: string;
  byClass: readonly ClassSummary[];
}) {
  const theme = useUi((s) => s.theme);
  const privacy = usePrivacy();
  const rows = useMemo(() => {
    const t = readChartTheme(theme, privacy);
    const colors = new Map(
      allocationSlices(byClass, theme, t.textDim).map((s) => [s.key, s.color]),
    );
    return byClass
      .filter((c) => c.target_pct != null)
      .map((c) => ({ ...c, color: colors.get(c.class) ?? t.textDim }))
      .sort((a, b) => (b.deviation_pp ?? 0) - (a.deviation_pp ?? 0));
  }, [byClass, theme, privacy]);
  const option = useCallback(
    (t: ChartTheme): EChartsOption => {
      const base = baseOption(t);
      return {
        ...base,
        legend: { show: false },
        grid: { left: 8, right: 24, top: 8, bottom: 8, containLabel: true },
        tooltip: {
          ...(base.tooltip as object),
          trigger: 'item',
          formatter: (p: unknown) => {
            const r = (p as { data: { row: (typeof rows)[number] } }).data.row;
            const weights = tr('carteira-desvio-peso-alvo', {
              peso: pctText(r.weight * 100, t, false),
              alvo: pctText(r.target_pct, t, false),
            });
            const dev = tr('carteira-desvio-pp', { desvio: pctText(r.deviation_pp, t) });
            const band = r.out_of_band ? ` · ${tr('carteira-fora-tolerancia')}` : '';
            return `<b>${r.label}</b><br/>${weights}<br/>${dev}${band}`;
          },
        },
        yAxis: {
          ...categoryAxis(
            t,
            rows.map((r) => r.label),
          ),
          inverse: true,
        },
        xAxis: valueAxis(t, (v) => tr('carteira-valor-pp', { valor: numText(v, 1) })),
        series: [
          {
            type: 'bar',
            barMaxWidth: 16,
            data: rows.map((r) => ({
              value: Number((r.deviation_pp ?? 0).toFixed(2)),
              row: r,
              itemStyle: {
                color: r.color,
                borderColor: t.surface,
                borderWidth: 1,
                borderRadius: (r.deviation_pp ?? 0) < 0 ? [4, 0, 0, 4] : [0, 4, 4, 0],
                opacity: r.out_of_band ? 1 : 0.55,
              },
            })),
            markLine: {
              silent: true,
              symbol: 'none',
              lineStyle: { color: t.lineStrong, width: 1, type: 'solid' },
              label: { show: false },
              data: [{ xAxis: 0 }],
            },
          },
        ],
      };
    },
    [rows],
  );
  if (rows.length === 0) return null;
  return (
    <ChartFrame
      index={index}
      title={tr('carteira-desvio-titulo')}
      source={tr('carteira-desvio-fonte')}
      chart={
        <EChart
          option={option}
          height={Math.max(140, rows.length * 30 + 24)}
          ariaLabel={tr('carteira-desvio-aria')}
        />
      }
      table={
        <TechTable
          label={tr('carteira-desvio-tabela')}
          prefsId="Desvio do alvo"
          dense
          rows={rows}
          getRowId={(r) => r.class}
          columns={[
            { key: 'label', header: tr('carteira-col-classe'), cell: (r) => r.label },
            {
              key: 'weight',
              header: tr('carteira-col-peso'),
              numeric: true,
              cell: (r) => <Pct value={r.weight} />,
            },
            {
              key: 'target',
              header: tr('carteira-col-alvo'),
              numeric: true,
              cell: (r) => <Pct value={(r.target_pct ?? 0) / 100} />,
            },
            {
              key: 'dev',
              header: tr('carteira-col-desvio'),
              numeric: true,
              cell: (r) =>
                privacy
                  ? '•••••'
                  : tr('carteira-valor-pp', { valor: pctText(r.deviation_pp, { privacy }) }),
            },
          ]}
        />
      }
    />
  );
}
