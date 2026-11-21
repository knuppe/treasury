/**
 * Substituto do `EChart` nos testes de tela (jsdom não tem canvas): guarda as props de cada
 * gráfico pelo `ariaLabel`, para o teste ler a opção montada e simular cliques.
 *
 * ```ts
 * vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));
 * chartProps(/Fluxo de/).onClick?.({ name: 'expense:cat-moradia', dataIndex: 0, value: 0, data: {} });
 * ```
 */
import type { ChartClick, EChartProps } from '@/charts/EChart';
import { readChartTheme } from '@/charts/theme';

const charts = new Map<string, EChartProps>();

export type { ChartClick, EChartProps };

export function EChart(props: EChartProps) {
  charts.set(props.ariaLabel, props);
  return <div role="img" aria-label={props.ariaLabel} data-testid="echart" />;
}

export function chartProps(label: RegExp): EChartProps {
  const found = [...charts.entries()].reverse().find(([k]) => label.test(k));
  if (!found)
    throw new Error(`Gráfico não encontrado: ${String(label)} (${[...charts.keys()].join(' | ')})`);
  return found[1];
}

/** Opção montada pelo gráfico no tema escuro. */
export function chartOption(label: RegExp, privacy = false): Record<string, unknown> {
  return chartProps(label).option(readChartTheme('dark', privacy));
}

export function clearCharts(): void {
  charts.clear();
}
