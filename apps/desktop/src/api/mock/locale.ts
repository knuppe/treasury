/**
 * Mock de `system.locale` e `system.set_locale` (docs/i18n-design.md): o idioma da instalação,
 * que no núcleo fica no `config.toml` e vale sem cofre.
 */
import type { Locale } from '@/api/generated/Locale';
import type { LocaleState } from '@/api/generated/LocaleState';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { fail } from './ledger/errors';

const NAMES: Record<Locale, string> = {
  'pt-BR': 'Português (Brasil)',
  en: 'English',
  es: 'Español',
  'pt-PT': 'Português (Portugal)',
};

let locale: Locale = 'pt-BR';
let chosen = true;

onMockReset(() => {
  locale = 'pt-BR';
  chosen = true;
});

/** Idioma atual da instalação (o núcleo usa para os nomes das categorias padrão). */
export function mockLocale(): Locale {
  return locale;
}

/** Testes: instalação nova, que ainda não escolheu o idioma. */
export function seedLocaleUnchosen(): void {
  chosen = false;
}

const state = (): LocaleState => ({
  locale,
  chosen,
  available: (Object.keys(NAMES) as Locale[]).map((code) => ({ code, name: NAMES[code] })),
});

function parse(code: string): Locale | null {
  const c = code.trim().toLowerCase();
  if (c === 'pt-pt') return 'pt-PT';
  if (c.startsWith('pt')) return 'pt-BR';
  if (c.startsWith('en')) return 'en';
  if (c.startsWith('es')) return 'es';
  return null;
}

registerMockHandlers({
  'system.locale': () => state(),
  'system.set_locale': (args) => {
    const l = parse(typeof args.locale === 'string' ? args.locale : '');
    if (!l) fail('invalid', 'idioma desconhecido');
    locale = l;
    chosen = true;
    return state();
  },
});
