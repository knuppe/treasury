import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Brand, BrandMark } from '@/app/Brand';

import motionCss from './motion.css?raw';

describe('motion', () => {
  const reduced = motionCss.slice(motionCss.indexOf('@media (prefers-reduced-motion: reduce)'));

  it('with reduced motion, turns off the mark, the cards and the theme switch', () => {
    expect(reduced.length).toBeGreaterThan(0);
    expect(reduced).toContain('.brand-mark *');
    expect(reduced).toContain('.card-arrive');
    expect(reduced).toContain('::view-transition-new(root)');
    expect(reduced).toContain('animation: none !important');
    expect(reduced).toContain('transition: none !important');
  });

  it('the mark stops animating by itself outside the waiting screens (SVG animation costs CPU)', () => {
    const rules = motionCss.slice(0, motionCss.indexOf("[data-motion='loop']"));
    expect(rules).not.toContain('infinite');
    expect(motionCss).toMatch(
      /\[data-motion='loop'\] \.brand-signal \{\s*animation: brand-loop [^;]*infinite/,
    );
  });

  it('the Glass mesh stays still (blur over an animated background costs on every frame)', () => {
    expect(motionCss).not.toMatch(/data-theme='glass'\] body::before \{[^}]*animation/);
  });

  it('the animations only touch opacity, transform and the SVG stroke', () => {
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

  it('the mark draws the signal on mount, loops while waiting, and can stay still', () => {
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

  it('two marks on the same screen do not share the same gradient id', () => {
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
