import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import activityCss from './activity.css?raw';
import { ThemeActivity } from './components';
import tokensCss from './tokens.css?raw';

// Saved theme id (D35: only the id keeps the old name) and the name of the piece.
const PIECES = [
  ['interstellar', 'orbit'],
  ['bladerunner', 'neon'],
  ['lofi', 'lofi'],
] as const;
const THEMES = PIECES.map(([, piece]) => piece);
const idOf = (piece: string) => PIECES.find(([, p]) => p === piece)![0];

describe('activity of the cinema themes and Lo-fi', () => {
  const reduced = activityCss.slice(activityCss.indexOf('@media (prefers-reduced-motion: reduce)'));

  it('is decorative and has one piece per theme', () => {
    const { container } = render(<ThemeActivity />);
    const root = container.querySelector('.theme-activity')!;
    expect(root.getAttribute('aria-hidden')).toBe('true');
    for (const theme of THEMES) expect(root.querySelector(`.activity-${theme}`)).not.toBeNull();
    expect(root.querySelectorAll('.activity-spinner')).toHaveLength(2);
    expect(root.querySelectorAll('.activity-leaf')).toHaveLength(2);
  });

  it('on by default; when off, each piece disappears and the animations pause', () => {
    const { container, rerender } = render(<ThemeActivity />);
    const root = () => container.querySelector('.theme-activity')!;
    expect(root().getAttribute('data-active')).toBe('true');
    rerender(<ThemeActivity active={false} />);
    expect(root().getAttribute('data-active')).toBe('false');
    for (const theme of THEMES) {
      const at = activityCss.indexOf(`.theme-activity[data-active='false'] .activity-${theme} {`);
      expect(at, theme).toBeGreaterThan(-1);
      expect(activityCss.slice(at, activityCss.indexOf('}', at))).toContain('opacity: 0');
    }
    expect(activityCss).toContain(".theme-activity[data-active='false'] *::after");
    expect(activityCss).toContain('animation-play-state: paused');
  });

  it('each theme shows only its own piece, and no other shows any', () => {
    const css = tokensCss.replace(/\/\*[\s\S]*?\*\//g, '');
    const derived = /:root,\s*\[data-theme\]\s*\{([^}]*)\}/.exec(css)![1]!;
    const rules = activityCss.replace(/\/\*[\s\S]*?\*\//g, '');
    for (const theme of THEMES) {
      expect(derived).toContain(`--activity-${theme}-display: none`);
      const block = new RegExp(`\\[data-theme='${idOf(theme)}'\\]\\s*\\{([^}]*)\\}`).exec(css)![1]!;
      expect(block).toContain(`--activity-${theme}-display: block`);
      for (const other of THEMES.filter((o) => o !== theme)) {
        expect(block).not.toContain(`--activity-${other}-display`);
      }
      // The piece's own rule (not the selector list it shares with the others).
      const own = new RegExp(`\\}\\s*\\.activity-${theme} \\{([^}]*)\\}`).exec(rules)![1]!;
      expect(own, theme).toContain(`display: var(--activity-${theme}-display)`);
    }
    // No other theme turns on any piece.
    expect(css.match(/--activity-[\w-]+-display: block/g)).toHaveLength(THEMES.length);
  });

  it('the animations only touch transform and opacity', () => {
    const keyframes = [...activityCss.matchAll(/@keyframes ([\w-]+) \{([\s\S]*?)\n\}\n/g)];
    expect(keyframes.map((k) => k[1])).toEqual([
      'activity-orbit',
      'activity-ring',
      'activity-ignite',
      'activity-flicker',
      'activity-spinner',
      'activity-drift',
      'activity-sway',
    ]);
    for (const [, name, body] of keyframes) {
      const props = [...body!.matchAll(/([\w-]+):/g)].map((m) => m[1]);
      for (const p of props) expect(['opacity', 'transform'], `${name}: ${p}`).toContain(p);
    }
    // No filters or blur, which WebKit would redo on every frame.
    expect(activityCss).not.toMatch(/filter|blur\(/);
  });

  it('with reduced motion, nothing moves', () => {
    expect(reduced.length).toBeGreaterThan(0);
    for (const sel of [
      '.activity-star',
      '.activity-star::after',
      '.activity-tube',
      '.activity-spinner',
      '.activity-leaf',
      '.activity-leaf::after',
      ".theme-activity[data-active='true'] .activity-neon",
    ]) {
      expect(reduced).toContain(sel);
    }
    expect(reduced).toContain('animation: none !important');
    expect(reduced).toContain('transform: translateX(575%)');
  });
});
