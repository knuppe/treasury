import { useCallback } from 'react';

import { EChart, type ChartClick } from './EChart';
import { sankeyHeight, sankeyOption, type SankeyInput, type SankeyLinkInput } from './sankey';
import type { ChartTheme } from './theme';

export interface SankeyChartProps {
  input: SankeyInput;
  ariaLabel: string;
  onNodeClick?: (id: string) => void;
  onLinkClick?: (link: SankeyLinkInput) => void;
}

/** Sankey (receitas → orçamento → categorias → subcategorias). */
export function SankeyChart({ input, ariaLabel, onNodeClick, onLinkClick }: SankeyChartProps) {
  const option = useCallback((t: ChartTheme) => sankeyOption(t, input), [input]);
  const handleClick = (e: ChartClick) => {
    const d = e.data as Partial<SankeyLinkInput> | undefined;
    if (d?.source && d.target)
      onLinkClick?.({ source: d.source, target: d.target, value: d.value ?? 0 });
    else onNodeClick?.(e.name);
  };
  return (
    <EChart
      option={option}
      height={sankeyHeight(input)}
      ariaLabel={ariaLabel}
      onClick={onNodeClick || onLinkClick ? handleClick : undefined}
    />
  );
}
