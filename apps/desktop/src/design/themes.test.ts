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

describe('temas', () => {
  afterEach(() => {
    delete (document as { startViewTransition?: unknown }).startViewTransition;
    document.documentElement.classList.remove('theme-switching');
  });

  it('clássicos (com o Mono), especiais e divertidos; sem os temas de fósforo', () => {
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

  it('tem o Scanner entre os divertidos, escuro e com cores completas', () => {
    const kitt = THEMES.find((t) => t.id === 'kitt')!;
    expect(kitt).toMatchObject({ label: 'Scanner', family: 'fun', scheme: 'dark' });
    expect(kitt.singleHue).toBe(false);
    expect(themeIcon('kitt')).toBe('moon');
    expect(toggledTheme('kitt')).toBe('light');
  });

  it('os divertidos são Scanner, Órbita, Neon, Lo-fi e Fósforo, sem nomes de filmes', () => {
    const fun = THEMES.filter((t) => t.family === 'fun');
    expect(fun.map((t) => t.label)).toEqual(['Scanner', 'Órbita', 'Neon', 'Lo-fi', 'Fósforo']);
    expect(themeInfo('matrix').scheme).toBe('dark');
    for (const gone of ['bttf', 'win95'])
      expect(THEMES.map((t) => t.id as string)).not.toContain(gone);
    expect(themeInfo('lofi').scheme).toBe('light');
    expect(themeIcon('lofi')).toBe('sun');
  });

  it('alterna claro/escuro pelo esquema do tema', () => {
    expect(toggledTheme('dark')).toBe('light');
    expect(toggledTheme('light')).toBe('dark');
    expect(toggledTheme('paper')).toBe('dark');
    expect(toggledTheme('midnight')).toBe('light');
    expect(toggledTheme('mono')).toBe('light');
    expect(themeIcon('paper')).toBe('sun');
    expect(themeIcon('glass')).toBe('moon');
    expect(themeIcon('mono')).toBe('contrast');
  });

  it('aplica direto, sem transição, quando não é pedido esmaecer', () => {
    const root = document.createElement('div');
    applyTheme('glass', root);
    expect(root.dataset.theme).toBe('glass');
  });

  it('esmaece a troca com View Transitions quando disponível', () => {
    stubReducedMotion(false);
    const start = vi.fn((update: () => void) => update());
    (document as { startViewTransition?: unknown }).startViewTransition = start;
    const root = document.createElement('div');
    applyTheme('paper', root, { animate: true });
    expect(start).toHaveBeenCalledOnce();
    expect(root.dataset.theme).toBe('paper');
  });

  it('não repete a transição para um tema que já está a caminho', () => {
    stubReducedMotion(false);
    const start = vi.fn();
    (document as { startViewTransition?: unknown }).startViewTransition = start;
    const root = document.createElement('div');
    applyTheme('midnight', root, { animate: true });
    applyTheme('midnight', root, { animate: true });
    expect(start).toHaveBeenCalledOnce();
  });

  it('sem View Transitions, usa a classe de transição de cores e a remove depois', () => {
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

  it('com movimento reduzido, troca na hora, sem transição', () => {
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

describe('paletas dos gráficos por tema', () => {
  it.each(THEMES.map((t) => t.id))('%s tem paleta e rampa próprias', (id) => {
    const info = THEMES.find((t) => t.id === id)!;
    const pal = categoricalPalette(id);
    expect(pal.length).toBe(info.singleHue ? 4 : MAX_SERIES);
    expect(new Set(pal).size).toBe(pal.length);
    expect(seriesCap(id)).toBe(info.singleHue ? 4 : MAX_SERIES);
    expect(sequentialRamp(id)[0]).toBe('transparent');
  });

  it('os temas especiais não reaproveitam a paleta do Escuro ou do Claro', () => {
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
