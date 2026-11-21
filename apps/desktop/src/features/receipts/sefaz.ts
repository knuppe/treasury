/**
 * Consulta da NFC-e numa janela do app (docs/ocr-receipts.md §4): para as SEFAZ com captcha, a
 * pessoa resolve o captcha na janela e o app recebe o texto da página do cupom (evento
 * `sefaz://page`). Só no app (fora do Tauri não faz nada).
 */
import { isTauri } from '@/api/client';
import { listenApp, type Unlisten } from '@/features/agent/tauriBridge';

export const canOpenSefaz = () => isTauri();

/** Abre a janela da consulta no link do QR (só https em .gov.br). */
export async function openSefaz(url: string): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('sefaz_open', { url });
}

/** Lê a página aberta agora (quando ela não foi reconhecida sozinha). */
export async function captureSefaz(): Promise<boolean> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<boolean>('sefaz_capture');
}

export const onSefazPage = (handler: (text: string) => void): Unlisten =>
  listenApp<string>('sefaz://page', handler);
