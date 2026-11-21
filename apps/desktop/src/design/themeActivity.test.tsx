import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import activityCss from './activity.css?raw';
import { ThemeActivity } from './components';
import tokensCss from './tokens.css?raw';

// Id salvo do tema (D35: só o id fica com o nome antigo) e o nome da peça.
const PIECES = [
  ['interstellar', 'orbit'],
  ['bladerunner', 'neon'],
  ['lofi', 'lofi'],
] as const;
const THEMES = PIECES.map(([, piece]) => piece);
const idOf = (piece: string) => PIECES.find(([, p]) => p === piece)![0];

describe('atividade dos temas de cinema e do Lo-fi', () => {
  const reduced = activityCss.slice(activityCss.indexOf('@media (prefers-reduced-motion: reduce)'));

  it('é decorativa e tem uma peça para cada tema', () => {
    const { container } = render(<ThemeActivity />);
    const root = container.querySelector('.theme-activity')!;
    expect(root.getAttribute('aria-hidden')).toBe('true');
    for (const theme of THEMES) expect(root.querySelector(`.activity-${theme}`)).not.toBeNull();
    expect(root.querySelectorAll('.activity-spinner')).toHaveLength(2);
    expect(root.querySelectorAll('.activity-leaf')).toHaveLength(2);
  });

  it('ligada por padrão; desligada, cada peça apaga e as animações pausam', () => {
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

  it('cada tema mostra só a sua peça, e nenhum outro mostra alguma', () => {
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
      // A regra própria da peça (não a lista de seletores que ela divide com as outras).
      const own = new RegExp(`\\}\\s*\\.activity-${theme} \\{([^}]*)\\}`).exec(rules)![1]!;
      expect(own, theme).toContain(`display: var(--activity-${theme}-display)`);
    }
    // Nenhum outro tema liga alguma peça.
    expect(css.match(/--activity-[\w-]+-display: block/g)).toHaveLength(THEMES.length);
  });

  it('as animações só mexem em transform e opacidade', () => {
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
    // Nada de filtro nem desfoque, que o WebKit refaria a cada quadro.
    expect(activityCss).not.toMatch(/filter|blur\(/);
  });

  it('com movimento reduzido, nada se move', () => {
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
