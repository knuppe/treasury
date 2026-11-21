/**
 * Gráfico ECharts com o tema do app. Recebe uma função que monta a opção a partir do
 * tema (cores, fontes, privacidade), redesenha ao trocar de tema ou de privacidade,
 * acompanha o tamanho do contêiner e expõe cliques para o drill-down.
 *
 * Acessibilidade: `ariaLabel` descreve o gráfico; para ≥ 2 séries use legenda; a
 * visão em tabela fica no `ChartFrame` (componente irmão).
 */
import { useEffect, useMemo, useRef } from 'react';

import { useUi } from '@/state/ui';

import { echarts, type ECharts, type EChartsOption } from './echarts';
import { type ChartTheme, readChartTheme } from './theme';

export interface ChartClick {
  name: string;
  seriesName?: string;
  dataIndex: number;
  value: unknown;
  data: unknown;
}

export interface EChartProps {
  option: (theme: ChartTheme) => EChartsOption;
  height?: number | string;
  ariaLabel: string;
  onClick?: (e: ChartClick) => void;
  className?: string;
}

/**
 * Resposta imediata ao mouse em qualquer gráfico, mesmo nos que não usam `baseOption`:
 * tooltip sem transição (o padrão de 0,4 s o deixa atrás do ponteiro) e destaque das séries
 * sem esmaecimento animado. Valores dados pelo próprio gráfico prevalecem.
 */
export function responsive(opt: EChartsOption): EChartsOption {
  const tip = (t: unknown) => (t && typeof t === 'object' ? { transitionDuration: 0, ...t } : t);
  const tooltip = Array.isArray(opt.tooltip) ? opt.tooltip.map(tip) : tip(opt.tooltip);
  return {
    stateAnimation: { duration: 0 },
    ...opt,
    ...(opt.tooltip ? { tooltip } : {}),
  };
}

/**
 * Opções do `echarts.init`. Sem `useDirtyRect`: com ele, o destaque do Sankey (`focus:
 * 'adjacency'`) às vezes redesenhava só um pedaço do canvas e as outras fitas sumiam ao
 * passar o mouse na vertical sobre um nó grande.
 */
export const INIT_OPTS = { renderer: 'canvas', useDirtyRect: false } as const;

export function EChart({ option, height = 280, ariaLabel, onClick, className }: EChartProps) {
  const ref = useRef<HTMLDivElement>(null);
  const chart = useRef<ECharts | null>(null);
  const themeId = useUi((s) => s.theme);
  const privacy = useUi((s) => s.privacy);
  const clickRef = useRef(onClick);
  useEffect(() => {
    clickRef.current = onClick;
  }, [onClick]);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const c = echarts.init(el, undefined, INIT_OPTS);
    chart.current = c;
    c.on('click', (p: unknown) => {
      const e = p as ChartClick;
      clickRef.current?.({
        name: e.name,
        seriesName: e.seriesName,
        dataIndex: e.dataIndex,
        value: e.value,
        data: e.data,
      });
    });
    const ro = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(() => c.resize()) : null;
    ro?.observe(el);
    return () => {
      ro?.disconnect();
      c.dispose();
      chart.current = null;
    };
  }, []);

  const opt = useMemo(
    () => responsive(option(readChartTheme(themeId, privacy))),
    [option, themeId, privacy],
  );

  useEffect(() => {
    chart.current?.setOption(opt, { notMerge: true, lazyUpdate: true });
  }, [opt]);

  return (
    <div
      ref={ref}
      role="img"
      aria-label={ariaLabel}
      className={className}
      style={{ width: '100%', height, cursor: onClick ? 'pointer' : undefined }}
    />
  );
}
