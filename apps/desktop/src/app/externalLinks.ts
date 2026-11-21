/**
 * Links para fora do app (`http(s)` de outro endereço: documentação, painéis das fontes de
 * dados, sites das instituições, links das respostas do agente) abrem no navegador do
 * sistema (`open_external`), nunca na janela do Treasury. Quem trata o clique antes e chama
 * `preventDefault` fica como está. Fora do app (navegador de desenvolvimento, testes), o
 * link segue o comportamento normal.
 */
import { isTauri } from '@/api/client';
import { toast } from '@/design/components';
import { t } from '@/i18n';

/** Endereço `http(s)` de fora do app, ou `null`. */
export function externalUrl(href: string | null, base: string): string | null {
  if (!href) return null;
  let url: URL;
  try {
    url = new URL(href, base);
  } catch {
    return null;
  }
  if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
  if (url.origin === new URL(base).origin) return null;
  return url.href;
}

export async function openExternal(url: string): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('open_external', { url });
}

function onClick(e: MouseEvent) {
  // Clique normal ou do botão do meio (auxclick); o direito abre o menu de contexto.
  if (e.defaultPrevented || e.button !== (e.type === 'click' ? 0 : 1)) return;
  const a = e.target instanceof Element ? e.target.closest('a[href]') : null;
  const url = externalUrl(a?.getAttribute('href') ?? null, window.location.href);
  if (!url) return;
  e.preventDefault();
  openExternal(url).catch((err: unknown) =>
    toast({
      tone: 'error',
      title: t('casca-navegador-falhou'),
      description: err instanceof Error ? err.message : String(err),
    }),
  );
}

/** Liga o desvio dos links externos (uma vez, na janela principal). */
export function installExternalLinks(): () => void {
  if (!isTauri()) return () => undefined;
  // Na fase de subida, depois dos handlers do React: um `preventDefault` deles vale.
  document.addEventListener('click', onClick);
  document.addEventListener('auxclick', onClick);
  return () => {
    document.removeEventListener('click', onClick);
    document.removeEventListener('auxclick', onClick);
  };
}
