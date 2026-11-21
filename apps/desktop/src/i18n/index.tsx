/**
 * Idiomas da interface (docs/i18n-design.md): os catálogos Fluent de `locales/` na raiz do
 * repositório (os mesmos do núcleo), o idioma da instalação (lido e gravado pelo núcleo no
 * `config.toml`) e o `t()` tipado pelas chaves do pt-BR (`keys.ts`, gerado).
 *
 * O pt-BR vai junto no pacote (é a origem e a reserva); os outros idiomas carregam só quando
 * escolhidos. Reserva: pt-PT → pt-BR; en → pt-BR; es → en → pt-BR. `xx` é o pseudo-idioma do
 * modo desenvolvedor (textos alongados e acentuados, para achar texto fixo e botão que estoura).
 */
import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';
import { useQueryClient } from '@tanstack/react-query';
import { Fragment, useEffect, useRef, useState, type ReactNode } from 'react';
import { create } from 'zustand';

import { call } from '@/api/client';
import type { LocaleState } from '@/api/generated/LocaleState';
import { setFormatLocale } from '@/lib/format';

import type { MessageKey, Messages } from './keys';

export type AppLocale = 'pt-BR' | 'en' | 'es' | 'pt-PT' | 'xx';

/** Idiomas do seletor (o pseudo-idioma só no modo desenvolvedor). */
export const LOCALES: readonly AppLocale[] = ['pt-BR', 'en', 'es', 'pt-PT'];

const PT_BR_SOURCES = import.meta.glob<string>('../../../../locales/pt-BR/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
});

const LAZY_SOURCES = import.meta.glob<string>(
  ['../../../../locales/*/*.ftl', '!../../../../locales/pt-BR/*.ftl'],
  { query: '?raw', import: 'default' },
);

function bundleOf(code: string, sources: readonly string[]): FluentBundle {
  const b = new FluentBundle(code, { useIsolating: false });
  for (const s of sources) b.addResource(new FluentResource(s));
  return b;
}

const PT_BR = bundleOf('pt-BR', Object.values(PT_BR_SOURCES));

const CHAIN: Record<AppLocale, readonly AppLocale[]> = {
  'pt-BR': ['pt-BR'],
  'pt-PT': ['pt-PT', 'pt-BR'],
  en: ['en', 'pt-BR'],
  es: ['es', 'en', 'pt-BR'],
  xx: ['pt-BR'],
};

const loaded = new Map<string, FluentBundle>([['pt-BR', PT_BR]]);

async function load(code: AppLocale): Promise<FluentBundle> {
  if (code === 'xx') return PT_BR;
  const hit = loaded.get(code);
  if (hit) return hit;
  const files = Object.entries(LAZY_SOURCES).filter(([p]) => p.includes(`/locales/${code}/`));
  const b = bundleOf(code, await Promise.all(files.map(([, f]) => f())));
  loaded.set(code, b);
  return b;
}

interface I18nState {
  locale: AppLocale;
  bundles: readonly FluentBundle[];
  /** O idioma já foi escolhido nesta instalação (sem isso, a primeira tela pergunta). */
  chosen: boolean;
}

export const useI18n = create<I18nState>()(() => ({
  locale: 'pt-BR',
  bundles: [PT_BR],
  chosen: true,
}));

/** Pseudo-idioma: letras acentuadas e ~35% mais longo, entre colchetes. */
export function pseudo(s: string): string {
  const map: Record<string, string> = {
    a: 'å',
    e: 'é',
    i: 'î',
    o: 'ö',
    u: 'ü',
    c: 'ç',
    n: 'ñ',
    s: 'š',
    t: 'ţ',
    y: 'ý',
    A: 'Å',
    E: 'É',
    I: 'Î',
    O: 'Ö',
    U: 'Ü',
    C: 'Ç',
    N: 'Ñ',
    S: 'Š',
    T: 'Ţ',
    Y: 'Ý',
  };
  const body = [...s].map((c) => map[c] ?? c).join('');
  return `[${body} ${'~'.repeat(Math.max(1, Math.ceil(s.length * 0.35)))}]`;
}

type ArgsOf<K extends MessageKey> = [Messages[K]] extends [never] ? [] : [Messages[K]];

/** Texto da chave no idioma atual (fora do React também). Sem a chave: a própria chave. */
export function t<K extends MessageKey>(key: K, ...rest: ArgsOf<K>): string {
  const { bundles, locale } = useI18n.getState();
  const args = rest[0] as Record<string, FluentVariable> | undefined;
  for (const b of bundles) {
    const m = b.getMessage(key);
    if (m?.value) {
      const s = b.formatPattern(m.value, args ?? null);
      return locale === 'xx' ? pseudo(s) : s;
    }
  }
  return key;
}

/** `t` para componentes: redesenha quando o idioma muda. */
export function useT(): typeof t {
  useI18n((s) => s.bundles);
  return t;
}

export function useLocale(): AppLocale {
  return useI18n((s) => s.locale);
}

/** Passa a usar o idioma (catálogos, formatação, `<html lang>`), sem gravar. */
export async function applyLocale(code: AppLocale): Promise<void> {
  const bundles = await Promise.all(CHAIN[code].map(load));
  useI18n.setState({ locale: code, bundles });
  const months = t('comum-meses').split(',');
  const short = t('comum-meses-curtos').split(',');
  setFormatLocale(code, code === 'xx' ? undefined : months, code === 'xx' ? undefined : short);
  if (typeof document !== 'undefined') {
    document.documentElement.lang = code === 'xx' ? 'pt-BR' : code;
  }
}

/** Troca o idioma da instalação: grava no núcleo (config.toml) e redesenha. */
export async function setLocale(code: AppLocale): Promise<void> {
  if (code !== 'xx') await call<LocaleState>('system.set_locale', { locale: code });
  await applyLocale(code);
  useI18n.setState({ chosen: true });
}

/** Palpite pelo idioma do sistema (`navigator.language`), para a primeira tela. */
export function guessLocale(lang: string | undefined = globalThis.navigator?.language): AppLocale {
  const l = (lang ?? '').toLowerCase();
  if (l === 'pt-pt') return 'pt-PT';
  if (l.startsWith('pt') || !l) return 'pt-BR';
  if (l.startsWith('es')) return 'es';
  return 'en';
}

/**
 * Lê o idioma da instalação no núcleo antes de mostrar o app (em teste, mostra na hora: o
 * idioma é pt-BR e a tela não pode esperar o mock).
 */
export function I18nProvider({ children }: { children: ReactNode }) {
  const immediate = import.meta.env.MODE === 'test';
  const [ready, setReady] = useState(immediate);
  const locale = useLocale();
  useEffect(() => {
    let alive = true;
    call<LocaleState>('system.locale')
      .then(async (s) => {
        // Sem idioma escolhido, a primeira tela pergunta, já com o palpite do sistema.
        const code = s.chosen ? s.locale : guessLocale();
        if (code !== 'pt-BR') await applyLocale(code);
        useI18n.setState({ chosen: s.chosen });
      })
      .catch(() => undefined)
      .finally(() => {
        if (alive) setReady(true);
      });
    return () => {
      alive = false;
    };
  }, []);
  // Textos montados pelo núcleo (insights, saúde, alertas…) vêm no idioma do pedido: ao trocar
  // o idioma, as consultas em cache são refeitas.
  const client = useQueryClient();
  const shown = useRef(locale);
  useEffect(() => {
    if (shown.current === locale) return;
    shown.current = locale;
    void client.invalidateQueries();
  }, [locale, client]);
  // Trocar o idioma remonta o app: todo texto sai de novo do catálogo.
  return ready ? <Fragment key={locale}>{children}</Fragment> : null;
}
