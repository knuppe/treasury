import { describe, expect, it } from 'vitest';

import { clip, nativeTarget, webTranslateUrl } from '@/lib/translateNotes';

describe('release notes translation', () => {
  it('offers nothing in English and maps each other interface language', () => {
    expect(nativeTarget('en')).toBeNull();
    expect(webTranslateUrl('Hello', 'en')).toBeNull();
    expect(nativeTarget('pt-BR')).toBe('pt-BR');
    expect(nativeTarget('pt-PT')).toBe('pt-PT');
    expect(nativeTarget('es')).toBe('es');
    const tl = (l: 'pt-BR' | 'pt-PT' | 'es') =>
      new URL(webTranslateUrl('Hello', l)?.url ?? '').searchParams.get('tl');
    expect([tl('pt-BR'), tl('pt-PT'), tl('es')]).toEqual(['pt', 'pt-PT', 'es']);
  });

  it('sends the notes from English, keeping line breaks and special characters', () => {
    const notes = '## New\n- Close accounts & cards: 100% done?';
    const web = webTranslateUrl(notes, 'pt-BR');
    const url = new URL(web?.url ?? '');
    expect(url.origin).toBe('https://translate.google.com');
    expect(url.searchParams.get('sl')).toBe('en');
    expect(url.searchParams.get('text')).toBe(notes);
    expect(web?.clipped).toBe(false);
  });

  it('cuts long notes at a paragraph break under the limit', () => {
    const para = 'x'.repeat(3000);
    const c = clip(`${para}\n\n${para}`);
    expect(c).toEqual({ text: para, clipped: true });
    // No break in the second half: a hard cut at the limit.
    expect(clip('y'.repeat(6000)).text).toHaveLength(5000);
  });
});
