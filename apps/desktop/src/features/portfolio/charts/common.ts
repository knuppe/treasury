/**
 * Peças comuns dos gráficos da carteira sobre a base de `@/charts` (tema, eixos,
 * barras e linhas finas): tooltip de eixo, percentuais e cores por chave.
 */
import { assignColors, type ChartTheme } from '@/charts';
import { formatNumber, MINUS } from '@/lib/format';

const MASK = '•••••';

/** `12.3` (pontos percentuais) → `+12,3%`. */
export function pctText(
  v: number | null | undefined,
  t: Pick<ChartTheme, 'privacy'>,
  signed = true,
): string {
  if (t.privacy) return MASK;
  if (v == null || !Number.isFinite(v)) return '—';
  const abs = formatNumber(Math.abs(v), 1);
  const sign = v < 0 ? MINUS : signed && v > 0 ? '+' : '';
  return `${sign}${abs}%`;
}

/** Número com até `max` casas, sem zeros sobrando (`12`, `12,5`), no formato do idioma. */
export function numText(v: number, max = 1): string {
  const f = 10 ** max;
  const r = Math.round(Math.abs(v) * f) / f;
  const decimals = r === 0 ? 0 : Math.min(max, (String(r).split('.')[1] ?? '').length);
  return `${v < 0 && r !== 0 ? MINUS : ''}${formatNumber(r, decimals)}`;
}

/** Rótulo de eixo em % (sem casas para inteiros). */
export function pctAxis(v: number): string {
  return `${numText(v, 1)}%`;
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}

interface AxisParam {
  seriesName?: string;
  value?: unknown;
  color?: unknown;
  axisValueLabel?: string;
  name?: string;
}

/** Tooltip de eixo: título (mês) e uma linha por série, com a cor da série. */
export function axisTooltip(
  t: ChartTheme,
  format: (value: number | null, seriesName: string) => string,
): Record<string, unknown> {
  return {
    trigger: 'axis',
    axisPointer: { type: 'line', lineStyle: { color: t.lineStrong, width: 1 } },
    formatter: (raw: unknown) => {
      const params = (Array.isArray(raw) ? raw : [raw]) as AxisParam[];
      const head = params[0]?.axisValueLabel ?? params[0]?.name ?? '';
      const rows = params
        .map((p) => {
          const v = typeof p.value === 'number' ? p.value : null;
          const color = typeof p.color === 'string' ? p.color : t.accent;
          return `<div style="display:flex;gap:12px;justify-content:space-between;align-items:center">
            <span style="display:inline-flex;align-items:center;gap:6px;color:${t.textDim}">
              <span style="display:inline-block;width:8px;height:8px;border-radius:2px;background:${color}"></span>${escapeHtml(p.seriesName ?? '')}
            </span>
            <span style="font-family:${t.fontNumeric};color:${t.text}">${escapeHtml(format(v, p.seriesName ?? ''))}</span>
          </div>`;
        })
        .join('');
      return `<div style="font-family:${t.fontNumeric};font-size:11px;color:${t.textDim};margin-bottom:4px;text-transform:uppercase;letter-spacing:.08em">${escapeHtml(head)}</div>${rows}`;
    },
  };
}

/** Cores por chave estável (fora do limite do tema: `textDim`). */
export function colorsFor(keys: readonly string[], t: ChartTheme): (key: string) => string {
  const map = assignColors(keys, t.id);
  return (key) => map.get(key) ?? t.textDim;
}
