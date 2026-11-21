import { describe, expect, it } from 'vitest';

import indexHtml from '../../index.html?raw';
import { contrastRatio } from './contrast';
import { THEMES } from './themes';
import tokensCss from './tokens.css?raw';

/** Body of the `[data-theme='x'] { … }` block in tokens.css (the first one, the theme's). */
function themeBlock(name: string): string {
  const css = tokensCss.replace(/\/\*[\s\S]*?\*\//g, '');
  const re = new RegExp(`\\[data-theme='${name}'\\]\\s*\\{([^}]*)\\}`);
  const match = re.exec(css);
  if (!match) throw new Error(`tema ${name} não encontrado em tokens.css`);
  return match[1]!;
}

/** Hex variables of a theme. */
function readTheme(name: string): Record<string, string> {
  const vars: Record<string, string> = {};
  for (const match of themeBlock(name).matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
    vars[match[1]!] = match[2]!;
  }
  return vars;
}

/** Percentage variables (`--glass-alpha: 50%`) of a theme, from 0 to 1. */
function readPercents(name: string): Record<string, number> {
  const vars: Record<string, number> = {};
  for (const match of themeBlock(name).matchAll(/--([\w-]+):\s*([\d.]+)%\s*;/g)) {
    vars[match[1]!] = Number(match[2]) / 100;
  }
  return vars;
}

/** `tint` with opacity `alpha` over `under` (sRGB compositing, like color-mix). */
function over(tint: string, alpha: number, under: string): string {
  const rgb = (h: string) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
  const a = rgb(tint);
  const b = rgb(under);
  return `#${a
    .map((v, i) =>
      Math.round(alpha * v + (1 - alpha) * b[i]!)
        .toString(16)
        .padStart(2, '0'),
    )
    .join('')}`;
}

const TEXT_TOKENS = ['text', 'text-dim', 'accent', 'positive', 'negative', 'warning', 'demo'];
const BACKGROUNDS = ['bg', 'surface', 'surface-2'];
const IDS = THEMES.map((t) => t.id);

describe('color tokens', () => {
  it('there is one token block per theme, and only for them', () => {
    const declared = [...tokensCss.matchAll(/\[data-theme='([\w-]+)'\]\s*\{/g)].map((m) => m[1]);
    expect(new Set(declared)).toEqual(new Set(IDS));
  });

  it.each(IDS)('theme %s defines all tokens', (id) => {
    const vars = readTheme(id);
    for (const token of [
      ...TEXT_TOKENS,
      ...BACKGROUNDS,
      'line',
      'line-strong',
      'accent-ink',
      'demo-ink',
    ]) {
      expect(vars[token], `--${token} em ${id}`).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it.each(IDS)('theme %s has WCAG AA contrast for text', (id) => {
    const vars = readTheme(id);
    for (const fg of TEXT_TOKENS) {
      for (const bg of BACKGROUNDS) {
        const ratio = contrastRatio(vars[fg]!, vars[bg]!);
        expect(ratio, `--${fg} sobre --${bg} no tema ${id}`).toBeGreaterThanOrEqual(4.5);
      }
    }
    expect(contrastRatio(vars['accent-ink']!, vars.accent!)).toBeGreaterThanOrEqual(4.5);
    expect(contrastRatio(vars['demo-ink']!, vars.demo!)).toBeGreaterThanOrEqual(4.5);
  });

  it.each(IDS)('theme %s has control borders with 3:1', (id) => {
    const vars = readTheme(id);
    for (const bg of BACKGROUNDS) {
      const ratio = contrastRatio(vars['line-strong']!, vars[bg]!);
      expect(ratio, `--line-strong sobre --${bg} no tema ${id}`).toBeGreaterThanOrEqual(3);
    }
  });

  it('the dark theme background is the reference one (#070b0f)', () => {
    expect(readTheme('dark').bg).toBe('#070b0f');
  });

  it('the index.html script (theme before first paint) knows all themes', () => {
    const match = /\^\(([\w|]+)\)\$/.exec(indexHtml);
    expect(match, 'regex de temas em index.html').not.toBeNull();
    expect(new Set(match![1]!.split('|'))).toEqual(new Set(IDS));
  });
});

describe('Glass theme: worst case of the translucent surfaces', () => {
  const vars = readTheme('glass');
  const pct = readPercents('glass');
  const mesh = ['mesh-1', 'mesh-2', 'mesh-3'].map((k) => vars[k]!);

  it('declares the mesh and the glass', () => {
    for (const m of mesh) expect(m).toMatch(/^#[0-9a-f]{6}$/i);
    for (const k of ['glass-alpha', 'glass-alpha-2', 'glass-pop-alpha']) {
      expect(pct[k], `--${k}`).toBeGreaterThan(0);
    }
  });

  it('the solid colors are the glass composited over the background', () => {
    expect(vars.surface).toBe(over(vars['glass-tint']!, pct['glass-alpha']!, vars.bg!));
    expect(vars['surface-2']).toBe(over(vars['glass-tint-2']!, pct['glass-alpha-2']!, vars.bg!));
  });

  // Blur only moves the colors toward the average: the lightest point of the mesh, unblurred,
  // is the worst case for light text.
  const worst: [string, string][] = mesh.flatMap((m, i) => [
    [`malha ${i + 1}`, m],
    [`card sobre malha ${i + 1}`, over(vars['glass-tint']!, pct['glass-alpha']!, m)],
    [`card elevado sobre malha ${i + 1}`, over(vars['glass-tint-2']!, pct['glass-alpha-2']!, m)],
  ]);

  it.each(worst)('text with 4.5:1 over %s', (_, bg) => {
    for (const fg of TEXT_TOKENS) {
      expect(contrastRatio(vars[fg]!, bg), `--${fg} sobre ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  });

  it.each(worst.filter(([name]) => name.startsWith('card')))(
    'control borders with 3:1 over %s',
    (_, bg) => {
      expect(contrastRatio(vars['line-strong']!, bg)).toBeGreaterThanOrEqual(3);
    },
  );

  it('popovers stay legible even over pure white behind the glass', () => {
    const bg = over(vars['glass-pop']!, pct['glass-pop-alpha']!, '#ffffff');
    for (const fg of TEXT_TOKENS) {
      expect(contrastRatio(vars[fg]!, bg), `--${fg} sobre ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  });

  it('falls back to solid colors without backdrop-filter and with reduced transparency', () => {
    expect(themeBlock('glass')).toContain('--panel-backdrop: blur(');
    for (const query of [
      /@supports not \(\(backdrop-filter/,
      /prefers-reduced-transparency: reduce/,
    ]) {
      const at = tokensCss.search(query);
      expect(at, String(query)).toBeGreaterThan(0);
      const rule = tokensCss.slice(at, tokensCss.indexOf('}\n}', at));
      expect(rule).toContain('--surface-fill: var(--surface)');
      expect(rule).toContain('--pop-fill: var(--surface-2)');
    }
  });
});

describe('themes with animated art: opaque cards, no blur', () => {
  // With the art moving behind (disc, rain), a backdrop-filter would have to be redone on
  // every frame: the cards keep the theme's solid color.
  it.each(['interstellar', 'bladerunner'])('%s does not swap the opaque material', (id) => {
    const block = themeBlock(id);
    for (const prop of [
      '--surface-fill',
      '--surface-2-fill',
      '--panel-backdrop',
      '--pop-backdrop',
      '--chrome-backdrop',
    ]) {
      expect(block, `${prop} em ${id}`).not.toContain(`${prop}:`);
    }
  });
});

describe('contrastRatio', () => {
  it('computes the WCAG contrast ratio', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21, 5);
    expect(contrastRatio('#777777', '#777777')).toBeCloseTo(1, 5);
  });
});
