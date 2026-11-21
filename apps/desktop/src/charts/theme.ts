/**
 * Tema dos gráficos a partir dos tokens CSS do tema ativo (tokens.css), para que
 * ECharts siga cada tema sem cores fixas no código das telas.
 */
import type { ThemeId } from '@/design/themes';
import { t as tr } from '@/i18n';
import {
  baseCurrency,
  currencyPrefix,
  currencySuffix,
  formatLocale,
  formatMoney,
  formatNumber,
} from '@/lib/format';

import { categoricalPalette, sequentialRamp } from './palette';

export interface ChartTheme {
  id: ThemeId;
  surface: string;
  surface2: string;
  line: string;
  lineStrong: string;
  text: string;
  textDim: string;
  accent: string;
  positive: string;
  negative: string;
  warning: string;
  fontText: string;
  fontNumeric: string;
  series: string[];
  sequential: string[];
  /** Modo privacidade: eixos e tooltips sem valores. */
  privacy: boolean;
}

let probe: HTMLElement | null = null;

/**
 * Lê os tokens de um tema num elemento escondido com `data-theme`, e não no <html>:
 * na troca de tema o atributo do <html> só muda no quadro seguinte (View Transition),
 * e o gráfico já precisa das cores novas ao redesenhar.
 */
function themeStyle(id: ThemeId): CSSStyleDeclaration | null {
  if (typeof window === 'undefined' || typeof document === 'undefined') return null;
  if (!probe || !probe.isConnected) {
    probe = document.createElement('div');
    probe.setAttribute('aria-hidden', 'true');
    probe.style.display = 'none';
    document.body.appendChild(probe);
  }
  probe.dataset.theme = id;
  return getComputedStyle(probe);
}

export function readChartTheme(id: ThemeId, privacy: boolean): ChartTheme {
  const style = themeStyle(id);
  const cssVar = (name: string, fallback: string): string =>
    style?.getPropertyValue(name).trim() || fallback;
  return {
    id,
    surface: cssVar('--surface', '#0c1117'),
    surface2: cssVar('--surface-2', '#121921'),
    line: cssVar('--line', '#1a232d'),
    lineStrong: cssVar('--line-strong', '#5a6a7c'),
    text: cssVar('--text', '#e8eef4'),
    textDim: cssVar('--text-dim', '#8d9aa8'),
    accent: cssVar('--accent', '#56c7ef'),
    positive: cssVar('--positive', '#3fd69a'),
    negative: cssVar('--negative', '#ff6b6b'),
    warning: cssVar('--warning', '#f5b73b'),
    fontText: cssVar('--font-text', 'Inter, system-ui, sans-serif'),
    fontNumeric: cssVar('--font-numeric', 'JetBrains Mono, ui-monospace, monospace'),
    series: categoricalPalette(id),
    sequential: sequentialRamp(id),
    privacy,
  };
}

const MASK = '•••••';

/** Número com até uma casa (12,9 · 3), no idioma do app. */
function upToOneDecimal(n: number): string {
  const r = Math.round(n * 10) / 10;
  return formatNumber(r, Number.isInteger(r) ? 0 : 1);
}

/**
 * Valor de eixo compacto no idioma do app e na moeda pedida (padrão: a moeda base da família):
 * `R$ 12,9 mil` · `R$ 1,2 mi` · `$12.9K`.
 */
export function compactMoney(
  cents: number,
  t: Pick<ChartTheme, 'privacy'>,
  currency: string = baseCurrency(),
): string {
  if (t.privacy) return MASK;
  const v = cents / 100;
  const abs = Math.abs(v);
  const sign = v < 0 ? '−' : '';
  const suffix = currencySuffix(currency);
  const prefix = currencyPrefix(currency);
  const glue = formatLocale() === 'en' && prefix.length <= 2 ? '' : ' ';
  const amount =
    abs >= 1_000_000
      ? tr('graficos-compacto-milhao', { valor: upToOneDecimal(abs / 1_000_000) })
      : abs >= 1_000
        ? tr('graficos-compacto-mil', { valor: upToOneDecimal(abs / 1_000) })
        : upToOneDecimal(abs);
  return suffix ? `${sign}${amount} ${suffix}` : `${sign}${prefix}${glue}${amount}`;
}

/** Valor completo para tooltips, respeitando a privacidade (padrão: a moeda base da família). */
export function tooltipMoney(
  cents: number,
  t: Pick<ChartTheme, 'privacy'>,
  currency: string = baseCurrency(),
): string {
  return t.privacy ? MASK : formatMoney(cents, currency);
}

/** Caixa do tooltip: sombra e cantos do app, fundo opaco e nenhum desfoque. */
export const TOOLTIP_CSS =
  'box-shadow: var(--shadow-pop); border-radius: var(--radius-sm); background-color: var(--surface-2) !important;';

/**
 * Opções comuns: grade, eixos recessivos (linha fina sólida), tooltip no estilo do app.
 *
 * Resposta ao mouse: o tooltip vai direto para o ponteiro (sem a transição padrão de 0,4 s,
 * que o deixava correndo atrás do mouse) e o destaque/esmaecimento das séries é imediato
 * (`stateAnimation`). O tooltip é opaco e sem `backdrop-filter`: um desfoque que se move
 * a cada movimento do mouse custa caro no WebKit. `EChart` garante o mesmo para gráficos
 * que não partem daqui.
 */
export function baseOption(t: ChartTheme): Record<string, unknown> {
  return {
    backgroundColor: 'transparent',
    color: t.series,
    animationDuration: 280,
    animationDurationUpdate: 180,
    stateAnimation: { duration: 0 },
    textStyle: { fontFamily: t.fontText, color: t.text },
    grid: { left: 8, right: 16, top: 28, bottom: 8, containLabel: true },
    tooltip: {
      backgroundColor: t.surface2,
      borderColor: t.line,
      borderWidth: 1,
      padding: [8, 10],
      transitionDuration: 0,
      textStyle: { color: t.text, fontFamily: t.fontText, fontSize: 12 },
      extraCssText: TOOLTIP_CSS,
    },
    legend: {
      type: 'scroll',
      top: 0,
      left: 0,
      icon: 'roundRect',
      itemWidth: 10,
      itemHeight: 10,
      textStyle: { color: t.textDim, fontSize: 11 },
      pageTextStyle: { color: t.textDim },
    },
    // A descrição acessível é a do app (`EChart` põe role="img" e o aria-label em
    // português); a automática do ECharts trocava esse rótulo por um texto em inglês.
    aria: { enabled: false },
  };
}

export function valueAxis(t: ChartTheme, format: (v: number) => string): Record<string, unknown> {
  return {
    type: 'value',
    axisLine: { show: false },
    axisTick: { show: false },
    splitLine: { lineStyle: { color: t.line, width: 1, type: 'solid' } },
    axisLabel: {
      color: t.textDim,
      fontFamily: t.fontNumeric,
      fontSize: 11,
      formatter: (v: number) => (t.privacy ? '' : format(v)),
    },
  };
}

export function categoryAxis(t: ChartTheme, data: string[]): Record<string, unknown> {
  return {
    type: 'category',
    data,
    axisLine: { lineStyle: { color: t.line } },
    axisTick: { show: false },
    axisLabel: { color: t.textDim, fontFamily: t.fontNumeric, fontSize: 11 },
  };
}

/** Barras: no máximo 24 px, ponta arredondada de 4 px e vão de 2 px na cor da superfície. */
export function barStyle(t: ChartTheme, horizontal = false): Record<string, unknown> {
  return {
    barMaxWidth: 24,
    itemStyle: {
      borderRadius: horizontal ? [0, 4, 4, 0] : [4, 4, 0, 0],
      borderColor: t.surface,
      borderWidth: 1,
    },
  };
}

export function lineStyle(): Record<string, unknown> {
  return { lineStyle: { width: 2, cap: 'round', join: 'round' }, symbolSize: 8, showSymbol: false };
}
