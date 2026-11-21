/**
 * Figura 02.R1 · Renda por mês (função pura): os meses passados com holerite (real) seguidos
 * da previsão dos próximos meses pelo último holerite. As barras do real são cheias e as da
 * previsão ficam esmaecidas sobre uma faixa "Previsto"; a linha tracejada é o líquido previsto
 * pelo último holerite em todos os meses, para ver no passado quanto o real se afastou dela.
 */
import type { IncomeMonth } from '@/api/generated/IncomeMonth';
import type { IncomeSummary } from '@/api/generated/IncomeSummary';
import {
  baseOption,
  categoryAxis,
  compactMoney,
  tooltipMoney,
  valueAxis,
  type ChartTheme,
} from '@/charts';
import { motion, tooltipHtml, type TooltipRow } from '@/charts/tooltip';
import { t } from '@/i18n';
import { formatMonthLong, formatMonthShort } from '@/lib/format';
import { monthsBetween, shiftMonth } from '@/lib/months';

export interface IncomeRow {
  month: string;
  /** `real` = holerite do mês; `forecast` = previsão; `empty` = mês passado sem holerite. */
  kind: 'real' | 'forecast' | 'empty';
  /** O que entrou (real) ou o que se prevê (previsão). */
  value: IncomeMonth;
  /** Previsão pelo último holerite para o mesmo mês (no real e na previsão). */
  expected: IncomeMonth | null;
}

/** Linhas da figura: do primeiro mês com holerite até o fim da previsão, sem buracos. */
export function incomeRows(s: IncomeSummary): IncomeRow[] {
  // O mês corrente sem holerite já entra como previsão (antes dos próximos meses).
  const months = s.current_month ? [s.current_month, ...s.monthly] : s.monthly;
  const forecast: IncomeRow[] = months.map((m) => ({
    month: m.month,
    kind: 'forecast',
    value: m,
    expected: m,
  }));
  const first = s.actual[0]?.real.month;
  const lastPast = months[0] ? shiftMonth(months[0].month, -1) : undefined;
  if (!first || !lastPast || first > lastPast) return forecast;
  const byMonth = new Map(s.actual.map((a) => [a.real.month, a]));
  const past: IncomeRow[] = monthsBetween(first, lastPast).map((month) => {
    const a = byMonth.get(month);
    return a
      ? { month, kind: 'real', value: a.real, expected: a.expected }
      : { month, kind: 'empty', value: emptyMonth(month), expected: null };
  });
  // Só o mês em andamento sem holerite (e sem previsão) sai; um mês passado sem holerite fica,
  // vazio, para o eixo não pular meses.
  const thisMonth = shiftMonth(s.from, -1);
  if (past.at(-1)?.kind === 'empty' && past.at(-1)!.month >= thisMonth) past.pop();
  return [...past, ...forecast];
}

function emptyMonth(month: string): IncomeMonth {
  return {
    month,
    gross: 0,
    social_security: 0,
    income_tax: 0,
    other_deductions: 0,
    net: 0,
    extras: [],
  };
}

export interface IncomeChartLabels {
  /** "INSS + IRRF" (ou os nomes do país). */
  taxes: string;
  /** "holerite" (ou o nome do país). */
  payslip: string;
  /** Competência do holerite que serve de base (`YYYY-MM`). */
  baseMonth: string | null;
}

export function incomeOption(
  theme: ChartTheme,
  s: IncomeSummary,
  rows: readonly IncomeRow[],
  labels: IncomeChartLabels,
) {
  const base = baseOption(theme);
  const money = (v: number) => tooltipMoney(v, theme, s.currency);
  const faded = (kind: IncomeRow['kind']) => (kind === 'forecast' ? 0.42 : 1);
  const names = {
    net: t('familia-liquido'),
    taxes: labels.taxes,
    other: t('familia-outros-descontos'),
    expected: t('familia-renda-linha-prevista', { holerite: labels.payslip }),
  };
  const bar = (name: string, color: string, pick: (m: IncomeMonth) => number) => ({
    name,
    type: 'bar',
    stack: 'renda',
    color,
    barMaxWidth: 28,
    data: rows.map((r) => ({
      value: r.kind === 'empty' ? null : pick(r.value),
      itemStyle: { color, opacity: faded(r.kind) },
    })),
  });
  const firstForecast = rows.findIndex((r) => r.kind === 'forecast');
  const hasPast = firstForecast > 0;
  // O eixo é de categorias: a faixa se marca pelo rótulo do mês.
  const axisLabels = rows.map((r) => formatMonthShort(r.month));
  const baseLabel = labels.baseMonth ? formatMonthShort(labels.baseMonth) : null;
  const netColor = theme.series[0] ?? theme.accent;
  return {
    ...base,
    ...motion(),
    legend: { ...(base.legend as object), show: true, top: 0 },
    grid: { left: 8, right: 16, top: 40, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'axis',
      axisPointer: { type: 'shadow', shadowStyle: { color: theme.line, opacity: 0.35 } },
      formatter: (params: { dataIndex: number }[]) => {
        const r = rows[params[0]?.dataIndex ?? 0];
        if (!r) return '';
        const title = `${formatMonthLong(r.month)} · ${
          r.kind === 'forecast' ? t('familia-renda-previsto') : t('familia-renda-real')
        }`;
        if (r.kind === 'empty')
          return tooltipHtml(
            theme,
            title,
            [],
            t('familia-renda-sem-holerite', { holerite: labels.payslip }),
          );
        const v = r.value;
        const out: TooltipRow[] = [
          { label: names.net, value: money(v.net), color: netColor, strong: true },
        ];
        if (r.kind === 'real' && r.expected) {
          const diff = v.net - r.expected.net;
          out.push({
            label: names.expected,
            value: money(r.expected.net),
            dashed: true,
            color: theme.text,
          });
          out.push({
            label: t('familia-renda-diferenca'),
            value: diff === 0 ? t('familia-renda-igual') : `${diff > 0 ? '+' : ''}${money(diff)}`,
            strong: diff !== 0,
          });
        }
        out.push(
          { label: names.taxes, value: money(v.social_security + v.income_tax) },
          { label: names.other, value: money(v.other_deductions) },
          { label: t('familia-bruto'), value: money(v.gross) },
        );
        if (v.extras.length) out.push({ label: t('familia-extras'), value: v.extras.join(', ') });
        const note =
          r.kind === 'real'
            ? baseLabel
              ? t('familia-renda-nota-real', { holerite: labels.payslip, mes: baseLabel })
              : undefined
            : baseLabel
              ? t('familia-renda-nota-previsto', { holerite: labels.payslip, mes: baseLabel })
              : undefined;
        return tooltipHtml(theme, title, out, note);
      },
    },
    xAxis: categoryAxis(theme, axisLabels),
    yAxis: valueAxis(theme, (v) => compactMoney(v, theme, s.currency)),
    series: [
      {
        ...bar(names.net, netColor, (m) => m.net),
        // Faixas "Real" e "Previsto" atrás das barras.
        markArea: {
          silent: true,
          itemStyle: { color: theme.line, opacity: 0.35 },
          label: {
            color: theme.textDim,
            fontFamily: theme.fontNumeric,
            fontSize: 10.5,
            position: 'insideTop',
          },
          data: [
            ...(hasPast
              ? [
                  [
                    {
                      xAxis: axisLabels[0],
                      name: t('familia-renda-real-faixa'),
                      itemStyle: { color: 'transparent' },
                    },
                    { xAxis: axisLabels[firstForecast - 1] },
                  ],
                ]
              : []),
            ...(firstForecast >= 0
              ? [
                  [
                    { xAxis: axisLabels[firstForecast], name: t('familia-renda-previsto-faixa') },
                    { xAxis: axisLabels[rows.length - 1] },
                  ],
                ]
              : []),
          ],
        },
      },
      bar(names.taxes, theme.series[1] ?? theme.warning, (m) => m.social_security + m.income_tax),
      bar(names.other, theme.series[2] ?? theme.textDim, (m) => m.other_deductions),
      ...(hasPast
        ? [
            {
              name: names.expected,
              type: 'line',
              color: theme.text,
              symbol: 'circle',
              symbolSize: 5,
              showSymbol: true,
              lineStyle: { width: 1.5, type: 'dashed', color: theme.text, opacity: 0.8 },
              itemStyle: { color: theme.text },
              z: 5,
              data: rows.map((r) => (r.kind === 'real' && r.expected ? r.expected.net : null)),
            },
          ]
        : []),
    ],
  };
}
