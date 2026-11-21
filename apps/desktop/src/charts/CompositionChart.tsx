import { useCallback } from 'react';

import { compositionOption, type CompositionForm, type CompositionNode } from './composition';
import { EChart } from './EChart';
import type { ChartTheme } from './theme';

export interface CompositionChartProps {
  nodes: readonly CompositionNode[];
  form: CompositionForm;
  currency: string;
  ariaLabel: string;
  height?: number;
  onNodeClick?: (id: string) => void;
}

/** Treemap ou sunburst de categorias e subcategorias. */
export function CompositionChart({
  nodes,
  form,
  currency,
  ariaLabel,
  height = 420,
  onNodeClick,
}: CompositionChartProps) {
  const option = useCallback(
    (t: ChartTheme) => compositionOption(t, nodes, form, currency),
    [nodes, form, currency],
  );
  return (
    <EChart
      option={option}
      height={height}
      ariaLabel={ariaLabel}
      onClick={
        onNodeClick
          ? (e) => {
              const id = (e.data as { id?: string } | undefined)?.id;
              if (id) onNodeClick(id);
            }
          : undefined
      }
    />
  );
}
