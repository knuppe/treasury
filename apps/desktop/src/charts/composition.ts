/**
 * Composição (treemap e sunburst), funções puras: categorias de primeiro nível na cor da
 * entidade, subcategorias em passos do mesmo matiz; rótulos dentro do bloco só quando cabem.
 */
import { inkOn, mix, resolveColor, type ColorRef, type ColorTokens } from './colorMath';
import type { EChartsOption } from './echarts';
import { baseOption, compactMoney, tooltipMoney, type ChartTheme } from './theme';
import { motion, pct, tooltipHtml } from './tooltip';
import { t as tr } from '@/i18n';

export interface CompositionNode {
  id: string;
  name: string;
  value: number;
  /** Cor da categoria de primeiro nível (subcategorias derivam dela). */
  color: ColorRef;
  children?: CompositionNode[];
}

export type CompositionForm = 'treemap' | 'sunburst';

/** Fração do total a partir da qual o bloco leva nome e valor; abaixo de NAME, sem rótulo. */
export const TREEMAP_FULL_LABEL = 0.03;
export const TREEMAP_NAME_LABEL = 0.012;

/** Passos do mesmo matiz para as subcategorias, em direção à superfície. */
export function childShades(color: string, surface: string, n: number): string[] {
  return Array.from({ length: n }, (_, i) => mix(color, surface, Math.min(0.12 + i * 0.1, 0.5)));
}

interface EData {
  id: string;
  name: string;
  value: number;
  itemStyle: { color: string };
  label?: { color: string };
  children?: EData[];
}

export function toEchartsTree(
  nodes: readonly CompositionNode[],
  t: ColorTokens & { surface: string },
): EData[] {
  return nodes.map((n) => {
    const color = resolveColor(t, n.color);
    const shades = childShades(color, t.surface, n.children?.length ?? 0);
    return {
      id: n.id,
      name: n.name,
      value: n.value,
      itemStyle: { color },
      label: { color: inkOn(color) },
      children: n.children?.map((c, i) => ({
        id: c.id,
        name: c.name,
        value: c.value,
        itemStyle: { color: shades[i]! },
        label: { color: inkOn(shades[i]!) },
      })),
    };
  });
}

export function compositionOption(
  t: ChartTheme,
  nodes: readonly CompositionNode[],
  form: CompositionForm,
  currency: string,
): EChartsOption {
  const total = nodes.reduce((s, n) => s + n.value, 0);
  const data = toEchartsTree(nodes, t);
  const tooltip = {
    ...(baseOption(t).tooltip as object),
    trigger: 'item',
    formatter: (p: { name: string; value: number; treePathInfo?: { name: string }[] }) => {
      const path = (p.treePathInfo ?? [])
        .map((x) => x.name)
        .filter(Boolean)
        .join(' › ');
      return tooltipHtml(
        t,
        path || p.name,
        [
          { label: tr('graficos-gasto'), value: tooltipMoney(p.value, t, currency), strong: true },
          { label: tr('graficos-do-total'), value: total > 0 ? pct(p.value / total, t) : '—' },
        ],
        tr('graficos-clique-lancamentos'),
      );
    },
  };
  const valueLine = (v: number) => (t.privacy ? '' : `\n{v|${compactMoney(v, t, currency)}}`);
  if (form === 'sunburst') {
    return {
      ...baseOption(t),
      ...motion(),
      tooltip,
      series: [
        {
          type: 'sunburst',
          data,
          radius: ['14%', '94%'],
          sort: undefined,
          nodeClick: false,
          itemStyle: { borderColor: t.surface, borderWidth: 2 },
          label: { fontFamily: t.fontText, fontSize: 11, minAngle: 10 },
          levels: [
            {},
            { r0: '14%', r: '52%', label: { rotate: 'tangential', minAngle: 16 } },
            { r0: '52%', r: '94%', label: { rotate: 'radial', minAngle: 7, fontSize: 10.5 } },
          ],
          emphasis: { focus: 'ancestor' },
        },
      ],
    };
  }
  return {
    ...baseOption(t),
    ...motion(),
    tooltip,
    series: [
      {
        type: 'treemap',
        data,
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
        roam: false,
        nodeClick: false,
        breadcrumb: { show: false },
        squareRatio: 0.9,
        label: {
          show: true,
          fontFamily: t.fontText,
          fontSize: 11.5,
          overflow: 'truncate',
          // Rótulo só em blocos que comportam o texto (o resto fica no tooltip e na tabela).
          formatter: (p: { name: string; value: number }) => {
            const share = total > 0 ? p.value / total : 0;
            if (share >= TREEMAP_FULL_LABEL) return `${p.name}${valueLine(p.value)}`;
            return share >= TREEMAP_NAME_LABEL ? p.name : '';
          },
          rich: { v: { fontFamily: t.fontNumeric, fontSize: 10.5, lineHeight: 16 } },
        },
        upperLabel: { show: false },
        itemStyle: { borderColor: t.surface, borderWidth: 0, gapWidth: 2 },
        levels: [
          {},
          {
            itemStyle: { borderColor: t.surface, borderWidth: 2, gapWidth: 2 },
            // A faixa do nome fica na cor da superfície (a borda do bloco): texto na cor do texto.
            upperLabel: {
              show: true,
              color: t.textDim,
              height: 20,
              fontFamily: t.fontNumeric,
              fontSize: 10.5,
              fontWeight: 600,
              overflow: 'truncate',
              formatter: (p: { name: string; value: number }) =>
                total > 0 && p.value / total >= TREEMAP_NAME_LABEL ? p.name.toUpperCase() : '',
            },
          },
          { itemStyle: { borderColor: t.surface, borderWidth: 0, gapWidth: 1 } },
        ],
      },
    ],
  };
}
