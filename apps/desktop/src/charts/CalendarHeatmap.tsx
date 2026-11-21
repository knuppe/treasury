import { useCallback } from 'react';

import { calendarOption, type CalendarDayInput } from './calendar';
import { EChart } from './EChart';
import type { ChartTheme } from './theme';

export interface CalendarHeatmapProps {
  days: readonly CalendarDayInput[];
  range: { from: string; to: string };
  currency: string;
  ariaLabel: string;
  onDayClick?: (date: string) => void;
}

/** Heatmap de calendário de um ano (um quadrado por dia). */
export function CalendarHeatmap({
  days,
  range,
  currency,
  ariaLabel,
  onDayClick,
}: CalendarHeatmapProps) {
  const option = useCallback(
    (t: ChartTheme) => calendarOption(t, days, range, currency),
    [days, range, currency],
  );
  return (
    <EChart
      option={option}
      height={210}
      ariaLabel={ariaLabel}
      onClick={
        onDayClick
          ? (e) => {
              const d = e.data as [string, number] | undefined;
              if (d && d[1] > 0) onDayClick(d[0]);
            }
          : undefined
      }
    />
  );
}
