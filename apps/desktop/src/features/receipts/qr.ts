/**
 * Lê o QR do cupom fiscal numa foto, no próprio computador (a imagem não sai dele).
 *
 * - No Mac, primeiro pelo Apple Vision (comando `read_qr_native`): o mesmo leitor da câmera do
 *   sistema, que aguenta sombra, inclinação, papel amassado, HEIC e a rotação EXIF do iPhone.
 * - Depois (e nos outros sistemas) pelo jsQR em [`decodePixels`], com a iluminação nivelada e
 *   limiar local, em três tamanhos (fotos grandes do celular são reduzidas antes).
 */
import { isTauri } from '@/api/client';

import { decodePixels } from './qrDecode';

export async function bitmapOf(
  file: Blob,
): Promise<{ img: CanvasImageSource; w: number; h: number }> {
  if (typeof createImageBitmap === 'function') {
    // A rotação EXIF da foto do celular entra já na leitura.
    const b = await createImageBitmap(file, { imageOrientation: 'from-image' });
    return { img: b, w: b.width, h: b.height };
  }
  const url = URL.createObjectURL(file);
  try {
    const img = new Image();
    img.src = url;
    await img.decode();
    return { img, w: img.naturalWidth, h: img.naturalHeight };
  } finally {
    URL.revokeObjectURL(url);
  }
}

export async function base64Of(file: Blob): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  let bin = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(bin);
}

/** Apple Vision pelo app (só no Mac; nos outros sistemas devolve `null`). */
async function readNative(file: Blob): Promise<string | null> {
  if (!isTauri()) return null;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<string | null>('read_qr_native', {
      contentBase64: await base64Of(file),
    });
  } catch {
    return null;
  }
}

/** O texto do QR (o link da consulta da NFC-e), ou `null` se não achar. */
export async function readQr(file: Blob): Promise<string | null> {
  const native = await readNative(file);
  if (native) return native;
  const { img, w, h } = await bitmapOf(file);
  // Do menor para o maior: o QR costuma ser grande na foto e o tamanho menor é mais rápido.
  let last = 0;
  for (const max of [1024, 1600, 2400]) {
    const scale = Math.min(1, max / Math.max(w, h));
    const cw = Math.max(1, Math.round(w * scale));
    const ch = Math.max(1, Math.round(h * scale));
    if (cw === last) break;
    last = cw;
    const canvas = document.createElement('canvas');
    canvas.width = cw;
    canvas.height = ch;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) return null;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(img, 0, 0, cw, ch);
    const code = decodePixels(ctx.getImageData(0, 0, cw, ch).data, cw, ch);
    if (code) return code;
  }
  return null;
}
