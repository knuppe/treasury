import { useCallback } from 'react';

import { EChart } from './EChart';
import { stackedAreaOption, type StackedAreaInput } from './stackedAreaOption';
import type { ChartTheme } from './theme';

export interface StackedAreaProps {
  input: StackedAreaInput;
  ariaLabel: string;
  height?: number;
  onPointClick?: (seriesKey: string, monthIndex: number) => void;
}

/** Área empilhada (patrimônio: contas × investimentos × dívidas; faturas futuras por cartão). */
export function StackedArea({ input, ariaLabel, height = 320, onPointClick }: StackedAreaProps) {
  const option = useCallback((t: ChartTheme) => stackedAreaOption(t, input), [input]);
  return (
    <EChart
      option={option}
      height={height}
      ariaLabel={ariaLabel}
      onClick={
        onPointClick
          ? (e) => {
              const s = input.series.find((x) => x.name === e.seriesName);
              if (s) onPointClick(s.key, e.dataIndex);
            }
          : undefined
      }
    />
  );
}
