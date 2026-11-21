/**
 * Razão de contraste WCAG 2.x entre duas cores sólidas.
 * Aceita `#rrggbb`, `#rgb` e `rgb(r g b)` / `rgb(r, g, b)` (o que o
 * getComputedStyle devolve), para conferir os temas nos testes e no catálogo.
 */

type Rgb = [number, number, number];

export function parseColor(color: string): Rgb | null {
  const c = color.trim();
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(c);
  if (hex) {
    let h = hex[1]!;
    if (h.length === 3) h = [...h].map((x) => x + x).join('');
    return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)) as Rgb;
  }
  const rgb = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)/i.exec(c);
  if (rgb) return [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])];
  return null;
}

function channel(value: number): number {
  const c = value / 255;
  return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

export function relativeLuminance([r, g, b]: Rgb): number {
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

export function contrastRatio(a: string, b: string): number {
  const ca = parseColor(a);
  const cb = parseColor(b);
  if (!ca || !cb) return Number.NaN;
  const la = relativeLuminance(ca);
  const lb = relativeLuminance(cb);
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}
