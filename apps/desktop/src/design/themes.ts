/**
 * Temas de exibição (docs/design.md). Escuro é o padrão. Clássicos: Escuro, Claro e Mono
 * (tons de cinza, um só matiz); especiais: Meia-noite, Papel e Vidro; divertidos: Scanner
 * (preto e vermelho com o scanner na barra superior), Órbita (o buraco negro), Neon (neon e
 * chuva), Lo-fi (papel washi, índigo e folhas ao vento) e Fósforo (verde fósforo sobre
 * preto e a chuva de código).
 * Os nomes e as descrições não citam filmes, séries, personagens nem marcas de terceiros
 * (docs/design.md); os ids antigos (`kitt`, `interstellar`, `bladerunner`, `matrix`) ficam
 * porque são o valor salvo nos ajustes.
 * As cores ficam em tokens.css, presas a `data-theme`; o scanner, em scanner.css; a arte
 * animada da Órbita, do Neon e do Lo-fi, em cinema.css; a peça que se mexe na barra
 * superior desses três enquanto o agente trabalha, em activity.css; a chuva de código, o
 * chip de atividade e o decode do Fósforo, em phosphor.css.
 */
import { t } from '@/i18n';

export type ThemeFamily = 'classic' | 'special' | 'fun';

export interface ThemeInfo {
  readonly id: string;
  readonly label: string;
  /** Uma linha para o seletor de tema e a barra de comando. */
  readonly description: string;
  readonly scheme: 'dark' | 'light';
  readonly family: ThemeFamily;
  /** Um só matiz (Mono): gráficos com até 4 séries, por passos de luminosidade. */
  readonly singleHue: boolean;
}

export const THEMES = [
  {
    id: 'dark',
    get label() {
      return t('casca-tema-dark');
    },
    get description() {
      return t('casca-tema-dark-dica');
    },
    scheme: 'dark',
    family: 'classic',
    singleHue: false,
  },
  {
    id: 'light',
    get label() {
      return t('casca-tema-light');
    },
    get description() {
      return t('casca-tema-light-dica');
    },
    scheme: 'light',
    family: 'classic',
    singleHue: false,
  },
  {
    id: 'mono',
    label: 'Mono',
    get description() {
      return t('casca-tema-mono-dica');
    },
    scheme: 'dark',
    family: 'classic',
    singleHue: true,
  },
  {
    id: 'midnight',
    get label() {
      return t('casca-tema-midnight');
    },
    get description() {
      return t('casca-tema-midnight-dica');
    },
    scheme: 'dark',
    family: 'special',
    singleHue: false,
  },
  {
    id: 'paper',
    get label() {
      return t('casca-tema-paper');
    },
    get description() {
      return t('casca-tema-paper-dica');
    },
    scheme: 'light',
    family: 'special',
    singleHue: false,
  },
  {
    id: 'glass',
    get label() {
      return t('casca-tema-glass');
    },
    get description() {
      return t('casca-tema-glass-dica');
    },
    scheme: 'dark',
    family: 'special',
    singleHue: false,
  },
  {
    id: 'kitt',
    get label() {
      return t('casca-tema-scanner');
    },
    get description() {
      return t('casca-tema-scanner-dica');
    },
    scheme: 'dark',
    family: 'fun',
    singleHue: false,
  },
  {
    id: 'interstellar',
    get label() {
      return t('casca-tema-orbit');
    },
    get description() {
      return t('casca-tema-orbit-dica');
    },
    scheme: 'dark',
    family: 'fun',
    singleHue: false,
  },
  {
    id: 'bladerunner',
    get label() {
      return t('casca-tema-neon');
    },
    get description() {
      return t('casca-tema-neon-dica');
    },
    scheme: 'dark',
    family: 'fun',
    singleHue: false,
  },
  {
    id: 'lofi',
    label: 'Lo-fi',
    get description() {
      return t('casca-tema-lofi-dica');
    },
    scheme: 'light',
    family: 'fun',
    singleHue: false,
  },
  {
    id: 'matrix',
    get label() {
      return t('casca-tema-phosphor');
    },
    get description() {
      return t('casca-tema-phosphor-dica');
    },
    scheme: 'dark',
    family: 'fun',
    singleHue: false,
  },
] as const satisfies readonly ThemeInfo[];

export type ThemeId = (typeof THEMES)[number]['id'];

export const DEFAULT_THEME: ThemeId = 'dark';

export const THEME_FAMILIES: readonly { id: ThemeFamily; label: string }[] = [
  {
    id: 'classic',
    get label() {
      return t('casca-temas-classicos');
    },
  },
  {
    id: 'special',
    get label() {
      return t('casca-temas-especiais');
    },
  },
  {
    id: 'fun',
    get label() {
      return t('casca-temas-divertidos');
    },
  },
];

export function isThemeId(value: unknown): value is ThemeId {
  return THEMES.some((t) => t.id === value);
}

export function themeInfo(id: ThemeId): ThemeInfo {
  return THEMES.find((t) => t.id === id) ?? THEMES[0];
}

export function themeLabel(id: ThemeId): string {
  return THEMES.find((t) => t.id === id)?.label ?? id;
}

/** Alternância claro/escuro: de um tema claro vai ao Escuro; de um escuro, ao Claro. */
export function toggledTheme(id: ThemeId): ThemeId {
  return themeInfo(id).scheme === 'light' ? 'dark' : 'light';
}

/** Ícone que representa o tema (barra superior, barra de comando). */
export function themeIcon(id: ThemeId): 'sun' | 'moon' | 'contrast' {
  const info = themeInfo(id);
  if (info.singleHue) return 'contrast';
  return info.scheme === 'light' ? 'sun' : 'moon';
}

export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export interface ApplyThemeOptions {
  /** Esmaece de um tema para o outro (ignorado com movimento reduzido). */
  animate?: boolean;
}

const SWITCH_MS = 320;

/** Tema a caminho em cada raiz (a View Transition aplica o atributo no quadro seguinte). */
const pending = new WeakMap<HTMLElement, ThemeId>();

/** Aplica o tema no <html> (os tokens de tokens.css seguem o atributo). */
export function applyTheme(
  id: ThemeId,
  root: HTMLElement = document.documentElement,
  options: ApplyThemeOptions = {},
): void {
  if ((pending.get(root) ?? root.dataset.theme) === id) return;
  const set = () => {
    pending.delete(root);
    root.dataset.theme = id;
  };
  if (!options.animate || prefersReducedMotion()) {
    set();
    return;
  }
  const doc = root.ownerDocument;
  if (typeof doc.startViewTransition === 'function') {
    pending.set(root, id);
    doc.startViewTransition(set);
    return;
  }
  root.classList.add('theme-switching');
  set();
  window.setTimeout(() => root.classList.remove('theme-switching'), SWITCH_MS);
}
