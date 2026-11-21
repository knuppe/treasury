/**
 * Gráfico do painel de uso (função pura): custo estimado por mês, uma série de barras
 * verticais num só eixo (USD). Tokens e chamadas ficam no tooltip e na tabela, nunca num
 * segundo eixo.
 */
import type { UsageGroup } from '@/api/generated/UsageGroup';
import type { EChartsOption } from '@/charts/echarts';
import { barStyle, baseOption, categoryAxis, valueAxis, type ChartTheme } from '@/charts/theme';
import { motion, tooltipHtml } from '@/charts/tooltip';
import { t } from '@/i18n';
import { formatMonthShort } from '@/lib/format';

import { formatCost, formatTokens } from '../format';

const MASK = '•••••';

export interface UsageMonth {
  key: string;
  label: string;
  costMicros: number;
  calls: number;
  inputTokens: number;
  outputTokens: number;
}

/** Meses de `from` a `to` (inclusive), com zero onde não houve uso. */
export function usageMonths(groups: readonly UsageGroup[], from: string, to: string): UsageMonth[] {
  const out: UsageMonth[] = [];
  let y = Number(from.slice(0, 4));
  let m = Number(from.slice(5, 7));
  const endY = Number(to.slice(0, 4));
  const endM = Number(to.slice(5, 7));
  while (y < endY || (y === endY && m <= endM)) {
    const key = `${y}-${String(m).padStart(2, '0')}`;
    const g = groups.find((x) => x.key === key);
    out.push({
      key,
      label: formatMonthShort(key),
      costMicros: g?.totals.cost_micros ?? 0,
      calls: g?.totals.calls ?? 0,
      inputTokens: g?.totals.input_tokens ?? 0,
      outputTokens: g?.totals.output_tokens ?? 0,
    });
    m += 1;
    if (m > 12) {
      m = 1;
      y += 1;
    }
  }
  return out;
}

/** Valor do eixo em dólares compactos (`USD 0,25`). */
function axisCost(micros: number): string {
  return formatCost(micros);
}

export function usageBarsOption(
  th: ChartTheme,
  months: readonly UsageMonth[],
  currency = 'USD',
): EChartsOption {
  const base = baseOption(th);
  return {
    ...base,
    ...motion(),
    legend: { show: false },
    grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
    tooltip: {
      ...(base.tooltip as object),
      trigger: 'item',
      formatter: (p: { dataIndex: number }) => {
        const mo = months[p.dataIndex];
        if (!mo) return '';
        return tooltipHtml(th, mo.label, [
          {
            label: t('agente-custo-estimado'),
            value: th.privacy ? MASK : formatCost(mo.costMicros, currency),
            strong: true,
          },
          { label: t('agente-chamadas'), value: th.privacy ? MASK : String(mo.calls) },
          {
            label: t('agente-tokens-entrada-saida-tooltip'),
            value: th.privacy
              ? MASK
              : `${formatTokens(mo.inputTokens)} / ${formatTokens(mo.outputTokens)}`,
          },
        ]);
      },
    },
    xAxis: categoryAxis(
      th,
      months.map((mo) => mo.label),
    ),
    yAxis: valueAxis(th, axisCost),
    series: [
      {
        type: 'bar',
        name: t('agente-custo-estimado'),
        color: th.accent,
        ...barStyle(th),
        data: months.map((mo) => mo.costMicros),
      },
    ],
  };
}
