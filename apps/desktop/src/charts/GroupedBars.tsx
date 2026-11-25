import { useCallback } from 'react';

import { EChart } from './EChart';
import { groupedBarsOption, groupedHeight, type GroupedBarsInput } from './groupedBarsOption';
import type { ChartTheme } from './theme';

export interface GroupedBarsProps {
  input: GroupedBarsInput;
  ariaLabel: string;
  onBarClick?: (rowKey: string, groupKey: string) => void;
}

/** Barras agrupadas (categorias × titulares). */
export function GroupedBars({ input, ariaLabel, onBarClick }: GroupedBarsProps) {
  const option = useCallback(
    (t: ChartTheme) =>
      groupedBarsOption(t, { ...input, clickable: input.clickable ?? Boolean(onBarClick) }),
    [input, onBarClick],
  );
  return (
    <EChart
      option={option}
      height={groupedHeight(input)}
      ariaLabel={ariaLabel}
      onClick={
        onBarClick
          ? (e) => {
              const row = input.rows[input.rows.length - 1 - e.dataIndex];
              const group = input.groups.find((g) => g.name === e.seriesName);
              if (row && group) onBarClick(row.key, group.key);
            }
          : undefined
      }
    />
  );
}
