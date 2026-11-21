/**
 * Os textos que citam uma tela pelo número ("07 · Dívidas") precisam acompanhar o menu: quando
 * uma tela entra no meio da lista, os números das seguintes mudam. Confere todos os catálogos.
 */
import { describe, expect, it } from 'vitest';

import { SCREENS } from './screens';

const SOURCES = import.meta.glob<string>('../../../../locales/*/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});

const LOCALES = ['pt-BR', 'pt-PT', 'en', 'es'];

/** Título de cada tela no idioma (`casca-tela-<id>`); o que faltar vem do pt-BR. */
function titles(locale: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const code of ['pt-BR', locale]) {
    const src = SOURCES[`../../../../locales/${code}/casca.ftl`] ?? '';
    for (const s of SCREENS) {
      const m = new RegExp(`^casca-tela-${s.id} = (.+)$`, 'm').exec(src);
      if (m) out.set(s.id, m[1]!.trim());
    }
  }
  return out;
}

describe('números das telas nos textos', () => {
  it.each(LOCALES)('%s: "NN · Tela" bate com o menu', (locale) => {
    const names = titles(locale);
    const wrong: string[] = [];
    for (const [path, src] of Object.entries(SOURCES)) {
      if (!path.includes(`/locales/${locale}/`)) continue;
      for (const line of src.split('\n')) {
        if (line.startsWith('#')) continue;
        for (const s of SCREENS) {
          const title = names.get(s.id);
          if (!title) continue;
          const escaped = title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
          for (const m of line.matchAll(new RegExp(`(\\d{2}) · ${escaped}(?![\\p{L}])`, 'gu'))) {
            if (m[1] !== s.index) wrong.push(`${path.split('/locales/')[1]}: ${m[0]} → ${s.index}`);
          }
        }
      }
    }
    expect(wrong).toEqual([]);
  });
});
