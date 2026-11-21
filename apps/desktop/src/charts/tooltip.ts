/**
 * Tooltips dos gráficos: valor em destaque, série em segundo plano, chave de série em
 * traço curto (não caixa). Rótulos vêm de dados do usuário: sempre escapados.
 */
import { formatPercent } from '@/lib/format';

import type { ChartTheme } from './theme';

export function esc(s: string): string {
  return s.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );
}

export interface TooltipRow {
  label: string;
  value: string;
  /** Cor da série (traço à esquerda). */
  color?: string;
  dashed?: boolean;
  /** Linha de total/destaque. */
  strong?: boolean;
}

/** HTML do tooltip: título, linhas (valor à direita, em mono) e nota opcional. */
export function tooltipHtml(
  t: ChartTheme,
  title: string,
  rows: readonly TooltipRow[],
  note?: string,
): string {
  const key = (r: TooltipRow) =>
    r.color
      ? `<span style="display:inline-block;width:12px;height:0;border-top:2px ${r.dashed ? 'dashed' : 'solid'} ${r.color};margin-right:6px;vertical-align:middle"></span>`
      : '<span style="display:inline-block;width:18px"></span>';
  const body = rows
    .map(
      (r) =>
        `<div style="display:flex;align-items:baseline;gap:16px;justify-content:space-between;line-height:20px">` +
        `<span style="color:${r.strong ? t.text : t.textDim}">${key(r)}${esc(r.label)}</span>` +
        `<span style="font-family:${t.fontNumeric};font-variant-numeric:tabular-nums;color:${t.text};font-weight:${r.strong ? 600 : 500}">${esc(r.value)}</span>` +
        `</div>`,
    )
    .join('');
  const head = `<div style="font-family:${t.fontNumeric};font-size:11px;letter-spacing:0.08em;text-transform:uppercase;color:${t.textDim};margin-bottom:4px">${esc(title)}</div>`;
  const foot = note
    ? `<div style="margin-top:6px;font-size:11px;color:${t.textDim}">${esc(note)}</div>`
    : '';
  return `<div style="min-width:180px">${head}${body}${foot}</div>`;
}

/** Percentual curto no idioma do app, escondido no modo privacidade. */
export function pct(ratio: number, t: Pick<ChartTheme, 'privacy'>, decimals = 1): string {
  if (t.privacy) return '•••••';
  return formatPercent(ratio, { decimals });
}

/** Movimento reduzido do sistema: gráficos sem animação. */
export function prefersReducedMotion(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  );
}

export function motion(): Record<string, unknown> {
  return prefersReducedMotion() ? { animation: false } : {};
}
