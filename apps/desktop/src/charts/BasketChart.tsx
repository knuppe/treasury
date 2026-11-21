import { useCallback } from 'react';

import { basketOption, type BasketChartInput } from './basketOption';
import { EChart } from './EChart';
import type { ChartTheme } from './theme';

/** Sua cesta × índice oficial de alimentos (base 100). */
export function BasketChart({
  input,
  ariaLabel,
  height = 280,
}: {
  input: BasketChartInput;
  ariaLabel: string;
  height?: number;
}) {
  const option = useCallback((t: ChartTheme) => basketOption(t, input), [input]);
  return <EChart option={option} height={height} ariaLabel={ariaLabel} />;
}
