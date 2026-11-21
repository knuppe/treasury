/**
 * Textos dos termos de uso (docs/terms.md): os curtos estão em `locales/<idioma>/termos.ftl`; o
 * texto completo, em Markdown por idioma, em `locales/conteudo/<idioma>/termos.md`. Os dois
 * seguem o idioma do app (a tela "Antes de começar" vem logo depois da escolha do idioma).
 */
import type { AppLocale } from '@/i18n';

const FULL = import.meta.glob<string>('../../../../../locales/conteudo/*/termos.md', {
  query: '?raw',
  import: 'default',
  eager: true,
});

function fullOf(code: string): string | undefined {
  return Object.entries(FULL).find(([p]) => p.includes(`/conteudo/${code}/`))?.[1];
}

/** Texto completo no idioma (espanhol sem tradução cai para o inglês; o resto, para pt-BR). */
export function fullTerms(locale: AppLocale): string {
  return fullOf(locale) ?? (locale === 'es' ? fullOf('en') : undefined) ?? fullOf('pt-BR') ?? '';
}

export const AGPL_URL = 'https://www.gnu.org/licenses/agpl-3.0.html';

/** Os pontos da tela "Antes de começar": o começo em negrito e o resto. */
export const POINTS = [
  ['termos-ponto-dados', 'termos-ponto-dados-texto'],
  ['termos-ponto-senha', 'termos-ponto-senha-texto'],
  ['termos-ponto-impostos', 'termos-ponto-impostos-texto'],
  ['termos-ponto-investimentos', 'termos-ponto-investimentos-texto'],
  ['termos-ponto-erros', 'termos-ponto-erros-texto'],
  ['termos-ponto-nuvem', 'termos-ponto-nuvem-texto'],
  ['termos-ponto-risco', 'termos-ponto-risco-texto'],
] as const;

export const SIGNATURE = 'Knuppe';
