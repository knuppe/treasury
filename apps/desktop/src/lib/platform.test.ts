import { describe, expect, it } from 'vitest';

import { ariaShortcut, formatShortcut, matchesShortcut } from './platform';

const key = (
  k: string,
  mods: Partial<Record<'meta' | 'ctrl' | 'shift' | 'alt', boolean>> = {},
) => ({
  key: k,
  code: /^[a-z]$/i.test(k) ? `Key${k.toUpperCase()}` : k,
  metaKey: mods.meta ?? false,
  ctrlKey: mods.ctrl ?? false,
  shiftKey: mods.shift ?? false,
  altKey: mods.alt ?? false,
});

describe('keyboard shortcuts', () => {
  it('the hint and the key come from the same list', () => {
    expect(formatShortcut(['mod', 'Backspace'], true)).toBe('⌘⌫');
    expect(formatShortcut(['mod', 'Backspace'], false)).toBe('Ctrl+Backspace');
    expect(formatShortcut(['mod', 'C'], false)).toBe('Ctrl+C');
    expect(ariaShortcut(['mod', 'C'])).toBe('Meta+C Control+C');
    expect(ariaShortcut(['enter'])).toBe('Enter');
  });

  it('mod accepts ⌘ or Ctrl; the other modifiers match exactly', () => {
    expect(matchesShortcut(key('c', { meta: true }), ['mod', 'C'])).toBe(true);
    expect(matchesShortcut(key('c', { ctrl: true }), ['mod', 'C'])).toBe(true);
    expect(matchesShortcut(key('C', { ctrl: true, shift: true }), ['mod', 'C'])).toBe(false);
    expect(matchesShortcut(key('c'), ['mod', 'C'])).toBe(false);
    expect(matchesShortcut(key('c'), ['C'])).toBe(true);
    expect(matchesShortcut(key('c', { meta: true }), ['C'])).toBe(false);
    expect(matchesShortcut(key('Backspace', { meta: true }), ['mod', 'Backspace'])).toBe(true);
    expect(matchesShortcut(key('Enter'), ['enter'])).toBe(true);
    expect(matchesShortcut(key('Escape'), ['esc'])).toBe(true);
    expect(matchesShortcut(key(' '), ['Espaço'])).toBe(true);
    // ⌥ changes the `key` on macOS; the letter counts by its physical code.
    expect(matchesShortcut({ ...key('c', { alt: true }), key: 'ç' }, ['alt', 'C'])).toBe(true);
  });
});
