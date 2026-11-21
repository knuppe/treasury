import { useCallback } from 'react';

import { EChart } from './EChart';
import { inflationOption, type InflationChartInput } from './inflationOption';
import type { ChartTheme } from './theme';

/** Linhas do gasto nominal e corrigido pela inflação (figura 07). */
export function InflationChart({
  input,
  ariaLabel,
  height = 320,
}: {
  input: InflationChartInput;
  ariaLabel: string;
  height?: number;
}) {
  const option = useCallback((t: ChartTheme) => inflationOption(t, input), [input]);
  return <EChart option={option} height={height} ariaLabel={ariaLabel} />;
}
