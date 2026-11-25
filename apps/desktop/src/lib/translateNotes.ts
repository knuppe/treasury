/**
 * Translating the release notes (Settings › Updates). The notes are written in English, so
 * nothing is offered when the interface is in English. On macOS 26 or later with the language
 * downloaded, the app translates on the device (Apple's Translation framework, through the app
 * shell). Anywhere else, the fallback is the browser translator, which only opens when the
 * person clicks it: the notes are public, and nothing about the vault goes along.
 */
import { isTauri } from '@/api/client';
import type { AppLocale } from '@/i18n';

/** Target language for the on-device translator, or `null` (English needs no translation). */
export function nativeTarget(locale: AppLocale): string | null {
  return locale === 'pt-BR' || locale === 'pt-PT' || locale === 'es' ? locale : null;
}

/** The browser translator's code for the interface language. */
const WEB_TARGET: Partial<Record<AppLocale, string>> = {
  'pt-BR': 'pt',
  'pt-PT': 'pt-PT',
  es: 'es',
};

/** About the most the browser translator takes at once. */
export const WEB_LIMIT = 5000;

/** Cuts long notes at the last paragraph (or line) break that fits. */
export function clip(text: string, limit = WEB_LIMIT): { text: string; clipped: boolean } {
  if (text.length <= limit) return { text, clipped: false };
  const head = text.slice(0, limit);
  const cut = Math.max(head.lastIndexOf('\n\n'), head.lastIndexOf('\n'));
  return { text: (cut > limit / 2 ? head.slice(0, cut) : head).trimEnd(), clipped: true };
}

/** The browser translator's address with the notes, or `null` when English needs none. */
export function webTranslateUrl(
  text: string,
  locale: AppLocale,
): { url: string; clipped: boolean } | null {
  const tl = WEB_TARGET[locale];
  if (!tl) return null;
  const c = clip(text);
  const q = new URLSearchParams({ sl: 'en', tl, text: c.text, op: 'translate' });
  return { url: `https://translate.google.com/?${q.toString()}`, clipped: c.clipped };
}

async function native<T>(cmd: string, args: Record<string, unknown>): Promise<T | null> {
  if (!isTauri()) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

/** Whether this computer translates into `target` on the device (false outside the app). */
export async function canTranslateOnDevice(target: string): Promise<boolean> {
  try {
    return (await native<boolean>('translation_status', { target })) === true;
  } catch {
    return false;
  }
}

/** Translates on the device; throws when the system could not. */
export async function translateOnDevice(text: string, target: string): Promise<string> {
  const out = await native<string>('translate_text', { text, target });
  if (out === null) throw new Error('unavailable');
  return out;
}
