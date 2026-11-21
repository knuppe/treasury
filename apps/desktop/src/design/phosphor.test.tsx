import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ACTIVITY_COMMANDS } from '@/api/client';
import { flash, startActivity, useActivity } from '@/state/activity';
import { useUi } from '@/state/ui';

import { Decode, PhosphorActivity, PhosphorRainHost } from './components';
import phosphorCss from './phosphor.css?raw';
import tokensCss from './tokens.css?raw';

const reset = () => useActivity.setState({ tasks: {}, last: null });

beforeEach(reset);
afterEach(() => {
  vi.useRealTimers();
  delete document.documentElement.dataset.theme;
  reset();
});

describe('app activity (state/activity.ts)', () => {
  it('keeps the tasks in progress and the last one that finished', () => {
    const stop = startActivity('a', 'Importação');
    useActivity.getState().start('b', 'Cotações');
    expect(Object.keys(useActivity.getState().tasks)).toEqual(['a', 'b']);
    stop('failed');
    expect(useActivity.getState().last).toMatchObject({ id: 'a', status: 'failed' });
    useActivity.getState().end('b');
    expect(useActivity.getState().tasks).toEqual({});
    expect(useActivity.getState().last).toMatchObject({ id: 'b', status: 'done' });
    // Ending what is not running changes nothing.
    useActivity.getState().end('nada');
    expect(useActivity.getState().last?.id).toBe('b');
  });

  it('import, receipt, e-mail, scenario and quotes count as activity', () => {
    for (const cmd of [
      'import.commit',
      'receipts.read_photo',
      'mail.sync_now',
      'scenarios.simulate',
      'market.sync',
    ]) {
      expect(ACTIVITY_COMMANDS[cmd], cmd).toBeTypeOf('function');
    }
    expect(ACTIVITY_COMMANDS['vault.status']).toBeUndefined();
  });

  it('the flash marks the element and clears itself', () => {
    vi.useFakeTimers();
    const el = document.createElement('div');
    flash(el);
    expect(el.classList.contains('activity-flash-ok')).toBe(true);
    flash(el, true);
    expect(el.classList.contains('activity-flash-ok')).toBe(false);
    expect(el.classList.contains('activity-flash-error')).toBe(true);
    vi.advanceTimersByTime(400);
    expect(el.className).toBe('');
  });
});

describe('Phosphor chip', () => {
  it('shows processing, the count and the failure for 3s, in an aria-live region', () => {
    vi.useFakeTimers();
    const { container } = render(<PhosphorActivity />);
    const live = container.querySelector('.phosphor-activity')!;
    expect(live.getAttribute('aria-live')).toBe('polite');
    expect(live.textContent).toBe('');

    act(() => useActivity.getState().start('a', 'Importação'));
    expect(live.textContent).toBe('Processando');
    act(() => useActivity.getState().start('b', 'Cotações'));
    expect(live.textContent).toBe('2 tarefas');

    act(() => useActivity.getState().end('b'));
    act(() => useActivity.getState().end('a', 'failed'));
    expect(screen.getByText('Falha').closest('.phosphor-chip')!.getAttribute('data-state')).toBe(
      'failed',
    );
    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(live.textContent).toBe('');
  });

  it('the block and the cursor live in CSS, and stop with reduced motion', () => {
    expect(phosphorCss).toContain(".phosphor-chip::before {\n  content: '▮\\00a0';");
    expect(phosphorCss).toContain(".phosphor-cursor::after {\n  content: '_';");
    expect(phosphorCss).toMatch(/\.phosphor-cursor \{\s*animation: phosphor-blink 1s steps\(1\)/);
    const reduced = phosphorCss.slice(
      phosphorCss.indexOf('@media (prefers-reduced-motion: reduce)'),
    );
    expect(reduced).toContain('.phosphor-cursor');
    expect(reduced).toContain('animation: none');
  });
});

describe('Phosphor pieces', () => {
  it('only appear in Phosphor', () => {
    const css = tokensCss.replace(/\/\*[\s\S]*?\*\//g, '');
    const derived = /:root,\s*\[data-theme\]\s*\{([^}]*)\}/.exec(css)![1]!;
    expect(derived).toContain('--phosphor-display: none');
    expect(derived).toContain('--phosphor-hide: initial');
    const block = /\[data-theme='matrix'\]\s*\{([^}]*)\}/.exec(css)![1]!;
    expect(block).toContain('--phosphor-display: block');
    expect(block).toContain('--phosphor-hide: none');
    expect(css.match(/--phosphor-display: block/g)).toHaveLength(1);
    for (const cls of ['.phosphor-rain', '.phosphor-activity', '.phosphor-only']) {
      const at = phosphorCss.indexOf(`${cls} {`);
      expect(phosphorCss.slice(at, phosphorCss.indexOf('}', at)), cls).toContain(
        'display: var(--phosphor-display)',
      );
    }
  });

  it('the rain stays faint in the top strip and speeds up to the full screen in 600ms', () => {
    const rule = (sel: string) => {
      const at = phosphorCss.indexOf(`${sel} {`);
      return phosphorCss.slice(at, phosphorCss.indexOf('}', at));
    };
    expect(rule('.phosphor-rain')).toContain('opacity: 0.15');
    expect(rule('.phosphor-rain')).toContain('--phosphor-reach: 180px');
    expect(rule('.phosphor-rain')).toContain('--phosphor-reach 600ms');
    expect(rule(".phosphor-rain[data-active='true']")).toContain('opacity: 0.35');
  });

  it('the rain only mounts in Phosphor and without reduced motion', () => {
    useUi.setState({ theme: 'dark' });
    const { rerender } = render(<PhosphorRainHost />);
    expect(document.querySelector('canvas.phosphor-rain')).toBeNull();
    act(() => {
      useUi.setState({ theme: 'matrix' });
    });
    rerender(<PhosphorRainHost />);
    expect(document.querySelector('canvas.phosphor-rain')).not.toBeNull();
    const spy = vi.spyOn(window, 'matchMedia').mockImplementation(
      (q) =>
        ({
          matches: q.includes('reduce'),
          media: q,
          addEventListener: () => {},
          removeEventListener: () => {},
        }) as unknown as MediaQueryList,
    );
    const again = render(<PhosphorRainHost />);
    expect(again.container.ownerDocument.querySelectorAll('canvas.phosphor-rain')).toHaveLength(1);
    spy.mockRestore();
    act(() => {
      useUi.setState({ theme: 'dark' });
    });
    rerender(<PhosphorRainHost />);
    expect(document.querySelector('canvas.phosphor-rain')).toBeNull();
  });

  it('the decode only scrambles in Phosphor and keeps the real text', () => {
    const { container, unmount } = render(<Decode text="R$ 1.234,56" />);
    const el = container.querySelector('.decode')!;
    expect(el.classList.contains('decoding')).toBe(false);
    unmount();
    document.documentElement.dataset.theme = 'matrix';
    const again = render(<Decode text="R$ 1.234,56" />);
    const el2 = again.container.querySelector('.decode')!;
    expect(el2.textContent).toBe('R$ 1.234,56');
    expect(el2.classList.contains('decoding')).toBe(true);
    const scramble = el2.getAttribute('data-scramble')!;
    expect(scramble).toHaveLength('R$ 1.234,56'.length);
    // The format stays: the "R$", the space and the separators do not scramble.
    expect(scramble.slice(0, 3)).toBe('R$ ');
    expect(scramble[4]).toBe('.');
  });
});
