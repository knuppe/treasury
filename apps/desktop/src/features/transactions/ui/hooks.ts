import { useEffect, useEffectEvent } from 'react';

import { t } from '@/i18n';

/** Alvo de digitação: atalhos de uma tecla não disparam aí. */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
}

/**
 * Atalho de uma tecla (sem modificadores) enquanto o componente está montado,
 * ignorado quando o foco está num campo ou há um diálogo aberto.
 */
export function useKeyShortcut(key: string, handler: () => void, enabled = true): void {
  const run = useEffectEvent((e: KeyboardEvent) => {
    if (!enabled || e.metaKey || e.ctrlKey || e.altKey || e.isComposing) return;
    if (e.key !== key || isTypingTarget(e.target)) return;
    if (document.querySelector('[role="dialog"][data-state="open"], [role="alertdialog"]')) return;
    e.preventDefault();
    handler();
  });
  useEffect(() => {
    const listener = (e: KeyboardEvent) => run(e);
    window.addEventListener('keydown', listener);
    return () => window.removeEventListener('keydown', listener);
  }, []);
}

/** Lê um arquivo como base64 (sem o prefixo `data:`). */
export async function fileToBase64(file: Blob): Promise<string> {
  const buffer =
    typeof file.arrayBuffer === 'function'
      ? await file.arrayBuffer()
      : await new Promise<ArrayBuffer>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(reader.result as ArrayBuffer);
          reader.onerror = () => reject(new Error(t('controles-erro-ler-arquivo')));
          reader.readAsArrayBuffer(file);
        });
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary);
}

/** Baixa um conteúdo base64 como arquivo (anexos). */
export function downloadBase64(content: string, filename: string, mime: string): void {
  const bin = atob(content);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  const url = URL.createObjectURL(new Blob([bytes], { type: mime }));
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Tamanho de arquivo (`512 B`, `1,5 KB`, `2 MB`), com o número no idioma do app. */
export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return t('controles-tamanho-kb', { n: n / 1024 });
  return t('controles-tamanho-mb', { n: n / 1024 / 1024 });
}
