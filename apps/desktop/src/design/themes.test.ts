import { afterEach, describe, expect, it, vi } from 'vitest';

import { categoricalPalette, MAX_SERIES, sequentialRamp, seriesCap } from '@/charts/palette';

import { applyTheme, THEME_FAMILIES, themeInfo, THEMES, themeIcon, toggledTheme } from './themes';

function stubReducedMotion(reduce: boolean) {
  vi.spyOn(window, 'matchMedia').mockImplementation(
    (query: string) =>
      ({
        matches: reduce && query.includes('prefers-reduced-motion'),
        media: query,
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      }) as MediaQueryList,
  );
}

describe('themes', () => {
  afterEach(() => {
    delete (document as { startViewTransition?: unknown }).startViewTransition;
    document.documentElement.classList.remove('theme-switching');
  });

  it('classic (with Mono), special and fun; without the phosphor themes', () => {
    const ids: string[] = THEMES.map((t) => t.id);
    expect(ids).toEqual(expect.arrayContaining(['dark', 'light', 'midnight', 'paper', 'glass']));
    for (const gone of ['green', 'amber', 'blue']) expect(ids).not.toContain(gone);
    expect(THEMES.filter((t) => t.family === 'classic').map((t) => t.id)).toEqual([
      'dark',
      'light',
      'mono',
    ]);
    expect(THEMES.filter((t) => t.singleHue).map((t) => t.id)).toEqual(['mono']);
    expect(THEME_FAMILIES.map((f) => f.id)).toEqual(['classic', 'special', 'fun']);
    for (const t of THEMES) expect(t.description.length).toBeGreaterThan(10);
  });

  it('has Scanner among the fun ones, dark and with full colors', () => {
    const kitt = THEMES.find((t) => t.id === 'kitt')!;
    expect(kitt).toMatchObject({ label: 'Scanner', family: 'fun', scheme: 'dark' });
    expect(kitt.singleHue).toBe(false);
    expect(themeIcon('kitt')).toBe('moon');
    expect(toggledTheme('kitt')).toBe('light');
  });

  it('the fun ones are Scanner, Orbit, Neon, Lo-fi and Phosphor, with no movie names', () => {
    const fun = THEMES.filter((t) => t.family === 'fun');
    expect(fun.map((t) => t.label)).toEqual(['Scanner', 'Órbita', 'Neon', 'Lo-fi', 'Fósforo']);
    expect(themeInfo('matrix').scheme).toBe('dark');
    for (const gone of ['bttf', 'win95'])
      expect(THEMES.map((t) => t.id as string)).not.toContain(gone);
    expect(themeInfo('lofi').scheme).toBe('light');
    expect(themeIcon('lofi')).toBe('sun');
  });

  it("toggles light/dark by the theme's scheme", () => {
    expect(toggledTheme('dark')).toBe('light');
    expect(toggledTheme('light')).toBe('dark');
    expect(toggledTheme('paper')).toBe('dark');
    expect(toggledTheme('midnight')).toBe('light');
    expect(toggledTheme('mono')).toBe('light');
    expect(themeIcon('paper')).toBe('sun');
    expect(themeIcon('glass')).toBe('moon');
    expect(themeIcon('mono')).toBe('contrast');
  });

  it('applies directly, without transition, when fading is not requested', () => {
    const root = document.createElement('div');
    applyTheme('glass', root);
    expect(root.dataset.theme).toBe('glass');
  });

  it('fades the switch with View Transitions when available', () => {
    stubReducedMotion(false);
    const start = vi.fn((update: () => void) => update());
    (document as { startViewTransition?: unknown }).startViewTransition = start;
    const root = document.createElement('div');
    applyTheme('paper', root, { animate: true });
    expect(start).toHaveBeenCalledOnce();
    expect(root.dataset.theme).toBe('paper');
  });

  it('does not repeat the transition for a theme already on its way', () => {
    stubReducedMotion(false);
    const start = vi.fn();
    (document as { startViewTransition?: unknown }).startViewTransition = start;
    const root = document.createElement('div');
    applyTheme('midnight', root, { animate: true });
    applyTheme('midnight', root, { animate: true });
    expect(start).toHaveBeenCalledOnce();
  });

  it('without View Transitions, uses the color transition class and removes it afterwards', () => {
    vi.useFakeTimers();
    stubReducedMotion(false);
    const root = document.documentElement;
    applyTheme('light', root, { animate: true });
    expect(root.classList.contains('theme-switching')).toBe(true);
    expect(root.dataset.theme).toBe('light');
    vi.runAllTimers();
    expect(root.classList.contains('theme-switching')).toBe(false);
    vi.useRealTimers();
    applyTheme('dark', root);
  });

  it('with reduced motion, switches immediately, without transition', () => {
    stubReducedMotion(true);
    const start = vi.fn();
    (document as { startViewTransition?: unknown }).startViewTransition = start;
    const root = document.createElement('div');
    applyTheme('glass', root, { animate: true });
    expect(start).not.toHaveBeenCalled();
    expect(root.dataset.theme).toBe('glass');
    expect(root.classList.contains('theme-switching')).toBe(false);
  });
});

describe('chart palettes per theme', () => {
  it.each(THEMES.map((t) => t.id))('%s has its own palette and ramp', (id) => {
    const info = THEMES.find((t) => t.id === id)!;
    const pal = categoricalPalette(id);
    expect(pal.length).toBe(info.singleHue ? 4 : MAX_SERIES);
    expect(new Set(pal).size).toBe(pal.length);
    expect(seriesCap(id)).toBe(info.singleHue ? 4 : MAX_SERIES);
    expect(sequentialRamp(id)[0]).toBe('transparent');
  });

  it('the special themes do not reuse the Dark or Light palette', () => {
    for (const id of [
      'midnight',
      'paper',
      'glass',
      'kitt',
      'interstellar',
      'bladerunner',
      'matrix',
    ] as const) {
      expect(categoricalPalette(id)).not.toEqual(categoricalPalette('dark'));
      expect(categoricalPalette(id)).not.toEqual(categoricalPalette('light'));
    }
  });
});
