import { describe, expect, it } from 'vitest';

import cinemaCss from './cinema.css?raw';
import tokensCss from './tokens.css?raw';

describe('animated art of Orbit and Neon', () => {
  it('only the two themes get the animated layer', () => {
    const selectors = [...cinemaCss.matchAll(/:root\[data-theme='([\w-]+)'\] body::after/g)].map(
      (m) => m[1],
    );
    expect(new Set(selectors)).toEqual(new Set(['interstellar', 'bladerunner', 'lofi']));
  });

  it('the animations only touch transform', () => {
    const keyframes = [...cinemaCss.matchAll(/@keyframes ([\w-]+) \{([\s\S]*?)\n\}\n/g)];
    expect(keyframes.map((k) => k[1])).toEqual(['gargantua-spin', 'rain-fall', 'leaves-drift']);
    for (const [, name, body] of keyframes) {
      const props = [...body!.matchAll(/([\w-]+):/g)].map((m) => m[1]);
      for (const p of props) expect(['transform'], `${name}: ${p}`).toContain(p);
    }
  });

  it('the rain closes the cycle seamlessly: the step is the vertical period of the three layers', () => {
    const sizes = /background-size:([^;]+);/.exec(cinemaCss)![1]!;
    // The Lo-fi leaves move exactly one tile per cycle.
    expect(cinemaCss).toMatch(/background-size: 320px 360px;/);
    expect(cinemaCss).toContain('translate(320px, 360px)');
    const heights = [...sizes.matchAll(/\d+px (\d+)px/g)].map((m) => m[1]);
    expect(heights).toEqual(['210', '210', '210']);
    expect(cinemaCss).toContain('translateY(210px)');
  });

  it('with reduced motion, the disc and the rain stay still', () => {
    const reduced = cinemaCss.slice(cinemaCss.indexOf('@media (prefers-reduced-motion: reduce)'));
    expect(reduced).toContain(":root[data-theme='interstellar'] body::after");
    expect(reduced).toContain(":root[data-theme='bladerunner'] body::after");
    expect(reduced).toContain('animation: none !important');
  });

  it('the black hole (shadow, photon ring and disc) is in the still art, for the thumbnails', () => {
    const at = tokensCss.indexOf("[data-theme='interstellar'] {");
    const block = tokensCss.slice(at, tokensCss.indexOf('\n}\n', at));
    expect(block).toContain('#000 0 91%');
    expect(block).toContain('rgb(255 220 160 / 0.9) 89%');
  });
});
