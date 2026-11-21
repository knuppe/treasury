import { describe, expect, it } from 'vitest';

import indexHtml from '../../index.html?raw';
import { contrastRatio } from './contrast';
import { THEMES } from './themes';
import tokensCss from './tokens.css?raw';

/** Corpo do bloco `[data-theme='x'] { … }` de tokens.css (o primeiro, o do tema). */
function themeBlock(name: string): string {
  const css = tokensCss.replace(/\/\*[\s\S]*?\*\//g, '');
  const re = new RegExp(`\\[data-theme='${name}'\\]\\s*\\{([^}]*)\\}`);
  const match = re.exec(css);
  if (!match) throw new Error(`tema ${name} não encontrado em tokens.css`);
  return match[1]!;
}

/** Variáveis hex de um tema. */
function readTheme(name: string): Record<string, string> {
  const vars: Record<string, string> = {};
  for (const match of themeBlock(name).matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
    vars[match[1]!] = match[2]!;
  }
  return vars;
}

/** Variáveis percentuais (`--glass-alpha: 50%`) de um tema, de 0 a 1. */
function readPercents(name: string): Record<string, number> {
  const vars: Record<string, number> = {};
  for (const match of themeBlock(name).matchAll(/--([\w-]+):\s*([\d.]+)%\s*;/g)) {
    vars[match[1]!] = Number(match[2]) / 100;
  }
  return vars;
}

/** `tint` com opacidade `alpha` sobre `under` (composição sRGB, como o color-mix). */
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

describe('tokens de cor', () => {
  it('há um bloco de tokens para cada tema, e só para eles', () => {
    const declared = [...tokensCss.matchAll(/\[data-theme='([\w-]+)'\]\s*\{/g)].map((m) => m[1]);
    expect(new Set(declared)).toEqual(new Set(IDS));
  });

  it.each(IDS)('tema %s define todos os tokens', (id) => {
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

  it.each(IDS)('tema %s tem contraste WCAG AA para texto', (id) => {
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

  it.each(IDS)('tema %s tem bordas de controle com 3:1', (id) => {
    const vars = readTheme(id);
    for (const bg of BACKGROUNDS) {
      const ratio = contrastRatio(vars['line-strong']!, vars[bg]!);
      expect(ratio, `--line-strong sobre --${bg} no tema ${id}`).toBeGreaterThanOrEqual(3);
    }
  });

  it('o fundo do tema escuro é o da referência (#070b0f)', () => {
    expect(readTheme('dark').bg).toBe('#070b0f');
  });

  it('o script de index.html (tema antes da primeira pintura) conhece todos os temas', () => {
    const match = /\^\(([\w|]+)\)\$/.exec(indexHtml);
    expect(match, 'regex de temas em index.html').not.toBeNull();
    expect(new Set(match![1]!.split('|'))).toEqual(new Set(IDS));
  });
});

describe('tema Vidro: pior caso das superfícies translúcidas', () => {
  const vars = readTheme('glass');
  const pct = readPercents('glass');
  const mesh = ['mesh-1', 'mesh-2', 'mesh-3'].map((k) => vars[k]!);

  it('declara a malha e o vidro', () => {
    for (const m of mesh) expect(m).toMatch(/^#[0-9a-f]{6}$/i);
    for (const k of ['glass-alpha', 'glass-alpha-2', 'glass-pop-alpha']) {
      expect(pct[k], `--${k}`).toBeGreaterThan(0);
    }
  });

  it('as cores sólidas são o vidro composto sobre o fundo', () => {
    expect(vars.surface).toBe(over(vars['glass-tint']!, pct['glass-alpha']!, vars.bg!));
    expect(vars['surface-2']).toBe(over(vars['glass-tint-2']!, pct['glass-alpha-2']!, vars.bg!));
  });

  // O desfoque só aproxima as cores da média: o ponto mais claro da malha, sem desfoque,
  // é o pior caso para texto claro.
  const worst: [string, string][] = mesh.flatMap((m, i) => [
    [`malha ${i + 1}`, m],
    [`card sobre malha ${i + 1}`, over(vars['glass-tint']!, pct['glass-alpha']!, m)],
    [`card elevado sobre malha ${i + 1}`, over(vars['glass-tint-2']!, pct['glass-alpha-2']!, m)],
  ]);

  it.each(worst)('texto com 4,5:1 sobre %s', (_, bg) => {
    for (const fg of TEXT_TOKENS) {
      expect(contrastRatio(vars[fg]!, bg), `--${fg} sobre ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  });

  it.each(worst.filter(([name]) => name.startsWith('card')))(
    'bordas de controle com 3:1 sobre %s',
    (_, bg) => {
      expect(contrastRatio(vars['line-strong']!, bg)).toBeGreaterThanOrEqual(3);
    },
  );

  it('popovers continuam legíveis mesmo sobre branco puro atrás do vidro', () => {
    const bg = over(vars['glass-pop']!, pct['glass-pop-alpha']!, '#ffffff');
    for (const fg of TEXT_TOKENS) {
      expect(contrastRatio(vars[fg]!, bg), `--${fg} sobre ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  });

  it('volta às cores sólidas sem backdrop-filter e com transparência reduzida', () => {
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

describe('temas com arte animada: cards opacos, sem desfoque', () => {
  // Com a arte se movendo atrás (disco, chuva), um backdrop-filter teria de ser refeito a
  // cada quadro: os cards ficam com a cor sólida do tema.
  it.each(['interstellar', 'bladerunner'])('%s não troca o material opaco', (id) => {
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
  it('calcula a razão de contraste WCAG', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21, 5);
    expect(contrastRatio('#777777', '#777777')).toBeCloseTo(1, 5);
  });
});
