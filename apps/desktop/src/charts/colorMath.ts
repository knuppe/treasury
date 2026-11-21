/** Aritmética de cor (hex sRGB) para passos de matiz e texto legível sobre preenchimentos. */

/** Mistura `hex` com `base` (0 = hex, 1 = base): passos mais claros/escuros do mesmo matiz. */
export function mix(hex: string, base: string, amount: number): string {
  const a = parseHex(hex);
  const b = parseHex(base);
  if (!a || !b) return hex;
  const c = a.map((v, i) => Math.round(v + (b[i]! - v) * amount));
  return `#${c.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

export function parseHex(hex: string): [number, number, number] | null {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return null;
  const n = parseInt(m[1]!, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

/** Luminância relativa (WCAG). */
export function luminance(hex: string): number {
  const rgb = parseHex(hex);
  if (!rgb) return 0;
  const [r, g, b] = rgb.map((v) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** Texto legível dentro de um preenchimento colorido: tinta escura ou branco. */
export function inkOn(fill: string): string {
  return luminance(fill) > 0.19 ? '#0b1218' : '#ffffff';
}

/**
 * Referência de cor: um hex (cor de entidade, já atribuída pela paleta) ou um token do tema
 * resolvido na hora de desenhar — `@positive`, `@negative`, `@warning`, `@accent`, `@text`,
 * `@dim`, `@gray` (cinza de "Outras") e `@s0`…`@s7` (posições da paleta categórica).
 */
export type ColorRef = string;

export interface ColorTokens {
  positive: string;
  negative: string;
  warning: string;
  accent: string;
  text: string;
  textDim: string;
  lineStrong: string;
  series: readonly string[];
}

export function resolveColor(t: ColorTokens, c: ColorRef): string {
  if (!c.startsWith('@')) return c;
  const name = c.slice(1);
  if (/^s\d$/.test(name)) return t.series[Number(name.slice(1))] ?? t.lineStrong;
  switch (name) {
    case 'positive':
      return t.positive;
    case 'negative':
      return t.negative;
    case 'warning':
      return t.warning;
    case 'accent':
      return t.accent;
    case 'text':
      return t.text;
    case 'dim':
      return t.textDim;
    default:
      return t.lineStrong;
  }
}

/** A mesma referência como cor CSS, para componentes HTML (tokens viram `var(--…)`). */
export function cssColor(c: ColorRef): string {
  if (!c.startsWith('@')) return c;
  const map: Record<string, string> = {
    positive: 'var(--positive)',
    negative: 'var(--negative)',
    warning: 'var(--warning)',
    accent: 'var(--accent)',
    text: 'var(--text)',
    dim: 'var(--text-dim)',
  };
  return map[c.slice(1)] ?? 'var(--line-strong)';
}
