/** Detalhes de plataforma usados na interface (atalhos de teclado). */
import { t } from '@/i18n';

export function isMac(): boolean {
  if (typeof navigator === 'undefined') return false;
  const platform =
    (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform ??
    navigator.platform ??
    '';
  return /mac|iphone|ipad/i.test(platform) || /Macintosh/.test(navigator.userAgent);
}

/** Tecla modificadora principal: ⌘ no macOS, Ctrl no Linux. Aceitamos as duas. */
export function hasModKey(event: { metaKey: boolean; ctrlKey: boolean }): boolean {
  return event.metaKey || event.ctrlKey;
}

const MAC_SYMBOLS: Record<string, string> = {
  mod: '⌘',
  shift: '⇧',
  alt: '⌥',
  ctrl: '⌃',
  enter: '↩',
  esc: 'Esc',
  backspace: '⌫',
  delete: '⌦',
};

const OTHER_NAMES: Record<string, string> = {
  mod: 'Ctrl',
  shift: 'Shift',
  alt: 'Alt',
  ctrl: 'Ctrl',
  enter: 'Enter',
  esc: 'Esc',
  backspace: 'Backspace',
  delete: 'Del',
};

/** Barra de espaço: o nome da tecla no idioma do app. */
const isSpace = (k: string) => k.toLowerCase() === 'espaço' || k.toLowerCase() === 'space';

/** `['mod', 'shift', 'H']` → `⌘⇧H` no macOS, `Ctrl+Shift+H` no Linux. */
export function formatShortcut(keys: readonly string[], mac: boolean = isMac()): string {
  const name = (k: string, names: Record<string, string>) =>
    isSpace(k)
      ? t('componentes-tecla-espaco').toUpperCase()
      : (names[k.toLowerCase()] ?? k.toUpperCase());
  if (mac) return keys.map((k) => name(k, MAC_SYMBOLS)).join('');
  return keys.map((k) => name(k, OTHER_NAMES)).join('+');
}

const MODIFIERS = new Set(['mod', 'shift', 'alt', 'ctrl']);

const ARIA_NAMES: Record<string, string> = {
  shift: 'Shift',
  alt: 'Alt',
  ctrl: 'Control',
  enter: 'Enter',
  esc: 'Escape',
  backspace: 'Backspace',
  delete: 'Delete',
  espaço: 'Space',
  space: 'Space',
};

/** `['mod', 'C']` → `Meta+C Control+C` (o formato de `aria-keyshortcuts`). */
export function ariaShortcut(keys: readonly string[]): string {
  const name = (k: string) => ARIA_NAMES[k.toLowerCase()] ?? (k.length === 1 ? k.toUpperCase() : k);
  const mods = keys.some((k) => k.toLowerCase() === 'mod') ? ['Meta', 'Control'] : [''];
  return mods
    .map((m) => keys.map((k) => (k.toLowerCase() === 'mod' ? m : name(k))).join('+'))
    .join(' ');
}

/** Nome do atalho → `KeyboardEvent.key` (o resto compara a própria tecla, sem caixa). */
const EVENT_KEYS: Record<string, string> = {
  enter: 'Enter',
  esc: 'Escape',
  backspace: 'Backspace',
  delete: 'Delete',
  espaço: ' ',
  space: ' ',
};

/**
 * A tecla corresponde ao atalho mostrado na dica (`['mod', 'C']`, `['mod', 'Backspace']`,
 * `['enter']`)? Uma fonte só para a dica e para a tecla. `mod` aceita ⌘ ou Ctrl (como
 * `hasModKey`); os demais modificadores têm de bater exatamente (sem `shift` no atalho,
 * Shift não pode estar apertado). Letras comparam pelo código físico também, porque ⌥ e
 * Shift mudam o `key`.
 */
export function matchesShortcut(
  event: {
    key: string;
    code?: string;
    metaKey: boolean;
    ctrlKey: boolean;
    shiftKey: boolean;
    altKey: boolean;
  },
  keys: readonly string[],
): boolean {
  const mods = new Set(keys.map((k) => k.toLowerCase()).filter((k) => MODIFIERS.has(k)));
  const main = keys.filter((k) => !MODIFIERS.has(k.toLowerCase()));
  if (main.length !== 1) return false;
  const modOk = mods.has('mod')
    ? hasModKey(event)
    : mods.has('ctrl')
      ? event.ctrlKey && !event.metaKey
      : !hasModKey(event);
  if (!modOk) return false;
  if (event.shiftKey !== mods.has('shift') || event.altKey !== mods.has('alt')) return false;
  const name = main[0]!;
  const want = EVENT_KEYS[name.toLowerCase()] ?? name;
  if (event.key.toLowerCase() === want.toLowerCase()) return true;
  return /^[a-z]$/i.test(want) && event.code === `Key${want.toUpperCase()}`;
}
