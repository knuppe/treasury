import { describe, expect, it } from 'vitest';

import cinemaCss from './cinema.css?raw';
import tokensCss from './tokens.css?raw';

describe('arte animada da Órbita e do Neon', () => {
  it('só os dois temas ganham a camada animada', () => {
    const selectors = [...cinemaCss.matchAll(/:root\[data-theme='([\w-]+)'\] body::after/g)].map(
      (m) => m[1],
    );
    expect(new Set(selectors)).toEqual(new Set(['interstellar', 'bladerunner', 'lofi']));
  });

  it('as animações só mexem em transform', () => {
    const keyframes = [...cinemaCss.matchAll(/@keyframes ([\w-]+) \{([\s\S]*?)\n\}\n/g)];
    expect(keyframes.map((k) => k[1])).toEqual(['gargantua-spin', 'rain-fall', 'leaves-drift']);
    for (const [, name, body] of keyframes) {
      const props = [...body!.matchAll(/([\w-]+):/g)].map((m) => m[1]);
      for (const p of props) expect(['transform'], `${name}: ${p}`).toContain(p);
    }
  });

  it('a chuva fecha o ciclo sem emenda: o passo é o período vertical das três camadas', () => {
    const sizes = /background-size:([^;]+);/.exec(cinemaCss)![1]!;
    // As folhas do Lo-fi andam exatamente um ladrilho por ciclo.
    expect(cinemaCss).toMatch(/background-size: 320px 360px;/);
    expect(cinemaCss).toContain('translate(320px, 360px)');
    const heights = [...sizes.matchAll(/\d+px (\d+)px/g)].map((m) => m[1]);
    expect(heights).toEqual(['210', '210', '210']);
    expect(cinemaCss).toContain('translateY(210px)');
  });

  it('com movimento reduzido, o disco e a chuva ficam parados', () => {
    const reduced = cinemaCss.slice(cinemaCss.indexOf('@media (prefers-reduced-motion: reduce)'));
    expect(reduced).toContain(":root[data-theme='interstellar'] body::after");
    expect(reduced).toContain(":root[data-theme='bladerunner'] body::after");
    expect(reduced).toContain('animation: none !important');
  });

  it('o buraco negro (sombra, anel de fótons e disco) está na arte parada, para as miniaturas', () => {
    const at = tokensCss.indexOf("[data-theme='interstellar'] {");
    const block = tokensCss.slice(at, tokensCss.indexOf('\n}\n', at));
    expect(block).toContain('#000 0 91%');
    expect(block).toContain('rgb(255 220 160 / 0.9) 89%');
  });
});
