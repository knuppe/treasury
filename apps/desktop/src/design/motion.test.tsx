import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Brand, BrandMark } from '@/app/Brand';

import motionCss from './motion.css?raw';

describe('movimento', () => {
  const reduced = motionCss.slice(motionCss.indexOf('@media (prefers-reduced-motion: reduce)'));

  it('com movimento reduzido, desliga a marca, os cards e a troca de tema', () => {
    expect(reduced.length).toBeGreaterThan(0);
    expect(reduced).toContain('.brand-mark *');
    expect(reduced).toContain('.card-arrive');
    expect(reduced).toContain('::view-transition-new(root)');
    expect(reduced).toContain('animation: none !important');
    expect(reduced).toContain('transition: none !important');
  });

  it('a marca para de animar sozinha fora das telas de espera (animação de SVG custa CPU)', () => {
    const rules = motionCss.slice(0, motionCss.indexOf("[data-motion='loop']"));
    expect(rules).not.toContain('infinite');
    expect(motionCss).toMatch(
      /\[data-motion='loop'\] \.brand-signal \{\s*animation: brand-loop [^;]*infinite/,
    );
  });

  it('a malha do Vidro fica parada (desfoque por cima de um fundo animado custa a cada quadro)', () => {
    expect(motionCss).not.toMatch(/data-theme='glass'\] body::before \{[^}]*animation/);
  });

  it('as animações só mexem em opacidade, transform e no traço do SVG', () => {
    const keyframes = [...motionCss.matchAll(/@keyframes ([\w-]+) \{([\s\S]*?)\n {0,2}\}\n/g)];
    expect(keyframes.map((k) => k[1])).toEqual(
      expect.arrayContaining(['brand-trace', 'brand-breathe', 'brand-scan', 'card-in']),
    );
    for (const [, name, body] of keyframes) {
      const props = [...body!.matchAll(/([\w-]+):/g)].map((m) => m[1]);
      for (const p of props) {
        expect(['opacity', 'transform', 'stroke-dashoffset'], `${name}: ${p}`).toContain(p);
      }
    }
  });

  it('a marca traça o sinal ao montar, em ciclo na espera, e pode ficar parada', () => {
    const { container, rerender } = render(<Brand />);
    const mark = () => container.querySelector('svg.brand-mark')!;
    expect(container.querySelector('.brand')).not.toBeNull();
    expect(mark().getAttribute('data-motion')).toBe('reveal');
    expect(mark().querySelector('.brand-signal')!.getAttribute('pathLength')).toBe('1');
    expect(mark().querySelector('.brand-scan')).not.toBeNull();
    expect(mark().querySelector('.brand-dot')).not.toBeNull();
    rerender(<Brand motion="loop" />);
    expect(mark().getAttribute('data-motion')).toBe('loop');
    rerender(<BrandMark motion="static" />);
    expect(mark().getAttribute('data-motion')).toBe('static');
    expect(mark().querySelector('.brand-scan')).toBeNull();
  });

  it('duas marcas na mesma tela não dividem o mesmo id de gradiente', () => {
    const { container } = render(
      <>
        <BrandMark />
        <BrandMark />
      </>,
    );
    const ids = [...container.querySelectorAll('clipPath, linearGradient')].map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
