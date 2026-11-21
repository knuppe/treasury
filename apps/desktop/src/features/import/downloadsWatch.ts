/**
 * Pasta Downloads (Ajustes › Preferências, ajuste `watch_downloads`): o app avisa com
 * `downloads://file` quando chega um arquivo que o Treasury importa; aqui ficam os avisos
 * pendentes (um cartão só para vários arquivos), os dispensados nesta sessão e a leitura do
 * arquivo aceito (`downloads_read`, só arquivos diretamente dentro da pasta acompanhada).
 *
 * Fora do app (navegador de desenvolvimento, testes) não há acompanhamento: use
 * `simulateDownloadOffer` para fazer o papel do evento.
 */
import { create } from 'zustand';

import { ApiError, isTauri } from '@/api/client';
import type { FileKind } from '@/api/generated/FileKind';
import { listenApp, type Unlisten } from '@/features/agent/tauriBridge';
import { t } from '@/i18n';

/** O que o app manda em `downloads://file`. */
export interface DownloadOffer {
  path: string;
  name: string;
  size: number;
  kind: FileKind;
  label: string;
  format?: string | null;
}

interface DownloadOffersState {
  offers: DownloadOffer[];
  /** Caminhos já tratados (importados ou dispensados) nesta sessão do app. */
  handled: Set<string>;
  add: (offer: DownloadOffer) => void;
  /** Tira do cartão e não oferece de novo nesta sessão. */
  settle: (paths: string[]) => void;
}

export const useDownloadOffers = create<DownloadOffersState>()((set) => ({
  offers: [],
  handled: new Set(),
  add: (offer) =>
    set((s) =>
      s.handled.has(offer.path) || s.offers.some((o) => o.path === offer.path)
        ? s
        : { offers: [...s.offers, offer] },
    ),
  settle: (paths) =>
    set((s) => {
      const handled = new Set(s.handled);
      for (const p of paths) handled.add(p);
      return { handled, offers: s.offers.filter((o) => !paths.includes(o.path)) };
    }),
}));

/** Assina o evento do app. Devolve a função de cancelar (no-op fora do Tauri). */
export function listenDownloadOffers(): Unlisten {
  return listenApp<DownloadOffer>('downloads://file', (o) => {
    if (o.kind !== 'unknown') useDownloadOffers.getState().add(o);
  });
}

/** Conteúdo dos arquivos simulados (fora do app). */
const simulated = new Map<string, string>();

/** Faz o papel do evento `downloads://file` fora do app (testes e desenvolvimento). */
export function simulateDownloadOffer(offer: DownloadOffer, contentBase64: string): void {
  simulated.set(offer.path, contentBase64);
  if (offer.kind !== 'unknown') useDownloadOffers.getState().add(offer);
}

/** Esquece avisos e arquivos simulados (testes). */
export function resetDownloadOffers(): void {
  simulated.clear();
  simulatedTrash.length = 0;
  useDownloadOffers.setState({ offers: [], handled: new Set() });
}

function base64ToBytes(b64: string): Uint8Array<ArrayBuffer> {
  const bin = atob(b64);
  const out = new Uint8Array(new ArrayBuffer(bin.length));
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
  return out;
}

/** Tipo pelo nome (o arquivo lido da pasta não traz tipo; o agente precisa dele nas imagens). */
const MIME: Record<string, string> = {
  png: 'image/png',
  jpg: 'image/jpeg',
  jpeg: 'image/jpeg',
  heic: 'image/heic',
  heif: 'image/heif',
  webp: 'image/webp',
  gif: 'image/gif',
  pdf: 'application/pdf',
  csv: 'text/csv',
  txt: 'text/plain',
};

export const mimeOf = (name: string) => MIME[name.toLowerCase().split('.').pop() ?? ''] ?? '';

/** Arquivos que foram para a Lixeira (fora do app). */
export const simulatedTrash: string[] = [];

/**
 * Manda para a Lixeira um arquivo da pasta Downloads já importado (`downloads_trash`; o
 * app confere o ajuste "apagar depois de importar" e a pasta acompanhada).
 */
export async function trashOfferedFile(path: string): Promise<void> {
  if (!isTauri()) {
    simulated.delete(path);
    simulatedTrash.push(path);
    return;
  }
  const { invoke } = await import('@tauri-apps/api/core');
  try {
    await invoke('downloads_trash', { path });
  } catch (reason) {
    const r = reason as { code?: string; message?: string } | string;
    if (typeof r === 'string') throw new ApiError('transport', r);
    throw new ApiError(r.code ?? 'transport', r.message ?? t('importacao-erro-apagar-arquivo'));
  }
}

/** Lê o arquivo aceito e devolve como `File` (para a janela de importação). */
export async function readOfferedFile(offer: DownloadOffer): Promise<File> {
  let content: string;
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    try {
      const r = await invoke<{ content_base64: string }>('downloads_read', { path: offer.path });
      content = r.content_base64;
    } catch (reason) {
      const r = reason as { code?: string; message?: string } | string;
      if (typeof r === 'string') throw new ApiError('transport', r);
      throw new ApiError(r.code ?? 'transport', r.message ?? t('importacao-erro-ler-arquivo'));
    }
  } else {
    const sim = simulated.get(offer.path);
    if (sim == null)
      throw new ApiError('not_found', t('importacao-erro-nao-encontrado', { nome: offer.name }));
    content = sim;
  }
  return new File([base64ToBytes(content)], offer.name, { type: mimeOf(offer.name) });
}
