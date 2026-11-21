/**
 * Languages: the generated keys match the pt-BR catalogs, `t()` follows the language and falls back to the
 * fallback, formatting follows and the pseudo-language lengthens the texts.
 */
import { afterEach, describe, expect, it } from 'vitest';

import { formatMoney, formatMonthLong } from '@/lib/format';

import { applyLocale, pseudo, t } from '.';
import { KEYS } from './keys';

const SOURCES = import.meta.glob<string>('../../../../locales/pt-BR/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});

afterEach(() => applyLocale('pt-BR'));

describe('i18n', () => {
  it('keys.ts is up to date with the pt-BR catalogs (run npm run i18n:keys)', () => {
    const ids = Object.values(SOURCES)
      .flatMap((src) => src.split('\n'))
      .filter((l) => /^[a-zA-Z]/.test(l))
      .map((l) => l.split('=')[0]!.trim())
      .sort();
    expect([...KEYS]).toEqual(ids);
  });

  it('language switch, with formatting and fallback', async () => {
    expect(t('comum-salvar')).toBe('Salvar');
    await applyLocale('en');
    expect(t('comum-salvar')).toBe('Save');
    expect(formatMonthLong('2026-10')).toBe('October 2026');
    expect(document.documentElement.lang).toBe('en');
    await applyLocale('es');
    expect(t('comum-cancelar')).toBe('Cancelar');
    expect(formatMonthLong('2026-10')).toBe('octubre de 2026');
    // Partial pt-PT: what is missing comes entirely from pt-BR.
    await applyLocale('pt-PT');
    expect(t('comum-salvar')).toBe('Guardar');
    expect(t('comum-fechar')).toBe('Fechar');
    expect(formatMoney(123456, 'EUR')).toBe('1234,56 €');
    await applyLocale('xx');
    expect(t('comum-salvar')).toMatch(/^\[Šålvår ~+\]$/);
    expect(pseudo('abc')).toBe('[åbç ~~]');
  });
});
