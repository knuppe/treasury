/** Opção ECharts do Sankey (função pura): nós coloridos pela entidade, fitas translúcidas. */
import { resolveColor, type ColorRef } from './colorMath';
import type { EChartsOption } from './echarts';
import { baseOption, tooltipMoney, type ChartTheme } from './theme';
import { formatMoney, formatPercent } from '@/lib/format';
import { t as tr } from '@/i18n';

import { motion, pct, tooltipHtml, type TooltipRow } from './tooltip';

/** O nó no período anterior equivalente. */
export interface NodeCompare {
  previous: number;
  delta: number;
  /** `null` quando o anterior é zero. */
  deltaPct: number | null;
}

export interface SankeyNodeInput {
  id: string;
  name: string;
  color: ColorRef;
  /** Coluna (0 = receitas). */
  depth: number;
  compare?: NodeCompare;
}

export interface SankeyLinkInput {
  source: string;
  target: string;
  value: number;
}

export interface SankeyInput {
  nodes: readonly SankeyNodeInput[];
  links: readonly SankeyLinkInput[];
  /** Base dos percentuais no tooltip (total de receitas, ou de despesas no déficit). */
  total: number;
  /** Rótulo do percentual no tooltip (padrão "Do total"). */
  shareLabel?: string;
  /** Nome do período anterior ("setembro de 2026"), quando há comparação. */
  previousLabel?: string;
  currency: string;
}

const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** Percentual com sinal (`+12,5%`), escondido no modo privacidade. */
const signedPct = (r: number, t: Pick<ChartTheme, 'privacy'>) =>
  t.privacy ? '•••••' : formatPercent(r, { signed: true });

/** Altura que cabe a coluna mais cheia sem sobrepor rótulos. */
export function sankeyHeight(input: SankeyInput): number {
  const perDepth = new Map<number, number>();
  for (const n of input.nodes) perDepth.set(n.depth, (perDepth.get(n.depth) ?? 0) + 1);
  const busiest = Math.max(0, ...perDepth.values());
  return Math.max(360, busiest * 24 + 40);
}

/** Valor de cada nó: o maior entre o que entra e o que sai. */
export function nodeValues(input: SankeyInput): Map<string, number> {
  const inSum = new Map<string, number>();
  const outSum = new Map<string, number>();
  for (const l of input.links) {
    outSum.set(l.source, (outSum.get(l.source) ?? 0) + l.value);
    inSum.set(l.target, (inSum.get(l.target) ?? 0) + l.value);
  }
  const out = new Map<string, number>();
  for (const n of input.nodes) out.set(n.id, Math.max(inSum.get(n.id) ?? 0, outSum.get(n.id) ?? 0));
  return out;
}

const LABEL_SIZE = 11.5;
const SHARE_SIZE = 10.5;
/** Espaço entre o fim do nó e o rótulo, mais o respiro até a borda. */
const LABEL_PAD = 14;
/** Faixa da coluna de rótulos da última coluna: nunca menor que isso… */
export const LABEL_COLUMN_MIN = 120;
/** …nem maior (cerca de um terço do gráfico na janela mínima com o painel do agente aberto). */
export const LABEL_COLUMN_MAX = 260;

let ctx2d: CanvasRenderingContext2D | null | undefined;
/** Largura do texto na fonte dada: pelo canvas quando existe; senão, uma estimativa por letra. */
export function textWidth(text: string, size: number, family: string, mono = false): number {
  if (ctx2d === undefined) {
    try {
      // Sem DOM, ou no jsdom dos testes (que não desenha), fica a estimativa por letra.
      const fake = typeof navigator !== 'undefined' && navigator.userAgent.includes('jsdom');
      ctx2d =
        typeof document === 'undefined' || fake
          ? null
          : document.createElement('canvas').getContext('2d');
    } catch {
      ctx2d = null;
    }
  }
  if (ctx2d) {
    ctx2d.font = `${size}px ${family}`;
    const w = ctx2d.measureText(text).width;
    if (w > 0) return w;
  }
  return text.length * size * (mono ? 0.62 : 0.56);
}

/** Corta o nome com reticências até caber em `max` px (o nome inteiro fica no tooltip). */
export function fitText(text: string, max: number, size: number, family: string): string {
  if (textWidth(text, size, family) <= max) return text;
  let lo = 0;
  let hi = text.length;
  while (lo < hi) {
    const mid = Math.ceil((lo + hi) / 2);
    if (textWidth(`${text.slice(0, mid).trimEnd()}…`, size, family) <= max) lo = mid;
    else hi = mid - 1;
  }
  return `${text.slice(0, lo).trimEnd()}…`;
}

/** O nó central: o que recebe de uma coluna e distribui para a seguinte (`hub`). */
function hubId(input: SankeyInput): string {
  return input.nodes.find((n) => n.id === 'hub')?.id ?? 'hub';
}

export function sankeyOption(t: ChartTheme, input: SankeyInput): EChartsOption {
  const names = new Map(input.nodes.map((n) => [n.id, n.name]));
  const colors = new Map(input.nodes.map((n) => [n.id, resolveColor(t, n.color)]));
  const values = nodeValues(input);
  const share = (v: number) => (input.total > 0 ? pct(v / input.total, t) : '');
  const shareLabel = input.shareLabel ?? tr('graficos-do-total');
  const compareOf = new Map(input.nodes.map((n) => [n.id, n.compare]));
  // Os rótulos da última coluna ficam à direita dos nós, numa faixa reservada (`right`). Ela
  // cresce com o rótulo mais longo, até o teto; o que ainda não couber leva reticências no
  // nome, nunca no percentual (antes a faixa era fixa e o ECharts cortava na borda).
  const lastDepth = Math.max(0, ...input.nodes.map((n) => n.depth));
  const lastIds = new Set(input.nodes.filter((n) => n.depth === lastDepth).map((n) => n.id));
  const shareText = (id: string) => (t.privacy ? '' : `  ${share(values.get(id) ?? 0)}`);
  const shareWidth = (id: string) =>
    t.privacy ? 0 : textWidth(shareText(id), SHARE_SIZE, t.fontNumeric, true);
  const labelWidth = (id: string) =>
    textWidth(names.get(id) ?? id, LABEL_SIZE, t.fontText) + shareWidth(id);
  const widest = Math.max(0, ...[...lastIds].map(labelWidth));
  const column = Math.round(
    Math.min(LABEL_COLUMN_MAX, Math.max(LABEL_COLUMN_MIN, widest + LABEL_PAD)),
  );
  const labelName = (id: string) => {
    const name = names.get(id) ?? id;
    if (!lastIds.has(id)) return name;
    return fitText(name, column - LABEL_PAD - shareWidth(id), LABEL_SIZE, t.fontText);
  };
  const rows = (v: number, nodeId: string): TooltipRow[] => {
    const out: TooltipRow[] = [
      { label: tr('graficos-valor'), value: tooltipMoney(v, t, input.currency), strong: true },
    ];
    if (input.total > 0) out.push({ label: shareLabel, value: share(v) });
    const c = compareOf.get(nodeId);
    if (input.previousLabel && c) {
      out.push({
        label: capitalize(input.previousLabel),
        value: tooltipMoney(c.previous, t, input.currency),
      });
      const arrow = c.delta > 0 ? '▲ ' : c.delta < 0 ? '▼ ' : '';
      const money = t.privacy ? '•••••' : formatMoney(c.delta, input.currency, { signed: true });
      const rel = c.deltaPct === null ? '' : ` (${signedPct(c.deltaPct, t)})`;
      out.push({ label: tr('graficos-variacao'), value: `${arrow}${money}${rel}` });
    }
    return out;
  };
  return {
    ...baseOption(t),
    ...motion(),
    tooltip: {
      ...(baseOption(t).tooltip as object),
      trigger: 'item',
      formatter: (p: {
        dataType: 'node' | 'edge';
        name: string;
        data: { source?: string; target?: string; value?: number };
      }) => {
        if (p.dataType === 'edge') {
          const v = p.data.value ?? 0;
          const source = p.data.source ?? '';
          const target = p.data.target ?? '';
          // A fita compara a ponta que não é o nó central (a mesma da cor).
          return tooltipHtml(
            t,
            `${names.get(source) ?? ''} → ${names.get(target) ?? ''}`,
            rows(v, target === hubId(input) ? source : target),
            tr('graficos-clique-lancamentos'),
          );
        }
        const v = values.get(p.name) ?? 0;
        return tooltipHtml(
          t,
          names.get(p.name) ?? p.name,
          rows(v, p.name),
          tr('graficos-clique-lancamentos'),
        );
      },
    },
    series: [
      {
        type: 'sankey',
        left: 4,
        right: column,
        top: 8,
        bottom: 8,
        nodeAlign: 'left',
        layoutIterations: 0,
        nodeWidth: 10,
        nodeGap: 12,
        draggable: false,
        emphasis: { focus: 'adjacency' },
        // Nós muito finos e vizinhos: o rótulo que não cabe some (a tabela tem todos).
        labelLayout: { hideOverlap: true },
        data: input.nodes.map((n) => ({
          name: n.id,
          depth: n.depth,
          itemStyle: { color: colors.get(n.id), borderWidth: 0 },
        })),
        // A fita leva a cor da ponta que não é o nó central (receita → orçamento → categoria).
        links: input.links.map((l) => ({
          ...l,
          lineStyle: { color: colors.get(l.target === hubId(input) ? l.source : l.target) },
        })),
        lineStyle: { opacity: 0.24, curveness: 0.5 },
        label: {
          color: t.text,
          fontFamily: t.fontText,
          fontSize: LABEL_SIZE,
          formatter: (p: { name: string }) => {
            const name = labelName(p.name);
            return t.privacy ? name : `${name}  {v|${share(values.get(p.name) ?? 0)}}`;
          },
          rich: { v: { color: t.textDim, fontFamily: t.fontNumeric, fontSize: SHARE_SIZE } },
        },
      },
    ],
  };
}
