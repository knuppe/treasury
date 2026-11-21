import { useCallback } from 'react';

import {
  divergingOption,
  rowsHeight,
  slopeOption,
  type ComparisonInput,
  type ComparisonLabels,
} from './comparison';
import { EChart } from './EChart';
import type { ChartTheme } from './theme';

export interface ComparisonChartProps {
  rows: readonly ComparisonInput[];
  labels: ComparisonLabels;
  form: 'diverging' | 'slope';
  ariaLabel: string;
  onRowClick?: (key: string) => void;
}

/** Período A × B: barras divergentes da variação ou slope. */
export function ComparisonChart({
  rows,
  labels,
  form,
  ariaLabel,
  onRowClick,
}: ComparisonChartProps) {
  const option = useCallback(
    (t: ChartTheme) =>
      form === 'slope' ? slopeOption(t, rows, labels) : divergingOption(t, rows, labels),
    [rows, labels, form],
  );
  const ordered = form === 'slope' ? rows : [...rows].reverse();
  return (
    <EChart
      option={option}
      height={form === 'slope' ? 380 : rowsHeight(rows.length)}
      ariaLabel={ariaLabel}
      onClick={
        onRowClick
          ? (e) => {
              const row =
                form === 'slope'
                  ? rows.find((r) => r.label === e.seriesName)
                  : ordered[e.dataIndex];
              if (row) onRowClick(row.key);
            }
          : undefined
      }
    />
  );
}
