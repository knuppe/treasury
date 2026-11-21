import { useCallback } from 'react';

import { EChart } from './EChart';
import { stackedBarsOption, type StackedBarsInput } from './stackedBarsOption';
import type { ChartTheme } from './theme';

export interface StackedBarsProps {
  input: StackedBarsInput;
  ariaLabel: string;
  height?: number;
  /** Clique num segmento (série) ou numa linha, no mês `index`. */
  onSegmentClick?: (seriesKey: string, monthIndex: number) => void;
}

/** Barras empilhadas por categoria, mês a mês, com média móvel e receitas de referência. */
export function StackedBars({ input, ariaLabel, height = 340, onSegmentClick }: StackedBarsProps) {
  const option = useCallback((t: ChartTheme) => stackedBarsOption(t, input), [input]);
  return (
    <EChart
      option={option}
      height={height}
      ariaLabel={ariaLabel}
      onClick={
        onSegmentClick
          ? (e) => {
              const key =
                input.bars.find((b) => b.name === e.seriesName)?.key ??
                input.lines.find((l) => l.name === e.seriesName)?.key;
              if (key) onSegmentClick(key, e.dataIndex);
            }
          : undefined
      }
    />
  );
}
