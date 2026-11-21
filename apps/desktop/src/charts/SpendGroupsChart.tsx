import { useCallback } from 'react';

import { EChart } from './EChart';
import { spendGroupsOption, type SpendGroupsInput } from './spendGroupsOption';
import type { ChartTheme } from './theme';

export interface SpendGroupsChartProps {
  input: SpendGroupsInput;
  ariaLabel: string;
  height?: number;
  /** Clique num segmento (grupo) no mês `index`. */
  onSegmentClick?: (groupKey: string, monthIndex: number) => void;
}

/** Grupos de gasto por mês: barras empilhadas ou linhas, em parte da renda ou valor. */
export function SpendGroupsChart({
  input,
  ariaLabel,
  height = 340,
  onSegmentClick,
}: SpendGroupsChartProps) {
  const option = useCallback((t: ChartTheme) => spendGroupsOption(t, input), [input]);
  return (
    <EChart
      option={option}
      height={height}
      ariaLabel={ariaLabel}
      onClick={
        onSegmentClick
          ? (e) => {
              const key = input.series.find((s) => s.name === e.seriesName)?.key;
              if (key) onSegmentClick(key, e.dataIndex);
            }
          : undefined
      }
    />
  );
}
