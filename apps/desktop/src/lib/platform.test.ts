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

describe('atalhos de teclado', () => {
  it('a dica e a tecla saem da mesma lista', () => {
    expect(formatShortcut(['mod', 'Backspace'], true)).toBe('⌘⌫');
    expect(formatShortcut(['mod', 'Backspace'], false)).toBe('Ctrl+Backspace');
    expect(formatShortcut(['mod', 'C'], false)).toBe('Ctrl+C');
    expect(ariaShortcut(['mod', 'C'])).toBe('Meta+C Control+C');
    expect(ariaShortcut(['enter'])).toBe('Enter');
  });

  it('mod aceita ⌘ ou Ctrl; os outros modificadores batem exatamente', () => {
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
    // ⌥ muda o `key` no macOS; a letra vale pelo código físico.
    expect(matchesShortcut({ ...key('c', { alt: true }), key: 'ç' }, ['alt', 'C'])).toBe(true);
  });
});
