/**
 * Cupom pela foto, sem QR (docs/ocr-receipts.md). No Mac, o Apple Vision lê as linhas aqui
 * (comando `read_text_native`); nos outros sistemas, o núcleo usa o PaddleOCR embutido. O núcleo
 * monta os itens e confere as contas (`receipts.read_photo`).
 */
import type { OcrLine } from '@/api/generated/OcrLine';
import type { PhotoArgs } from '@/api/generated/PhotoArgs';
import { isTauri } from '@/api/client';

import { base64Of, bitmapOf, readQr } from './qr';

/** Linhas do Apple Vision (só no Mac; nos outros sistemas, `null`). */
export async function readNative(content: string): Promise<OcrLine[] | null> {
  if (!isTauri()) return null;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<OcrLine[] | null>('read_text_native', { contentBase64: content });
  } catch {
    return null;
  }
}

/** Os argumentos de `receipts.read_photo` para a foto. */
export async function photoArgs(file: File, useAi = false): Promise<PhotoArgs> {
  const content = await base64Of(file);
  const lines = await readNative(content);
  // Tamanho já com a rotação EXIF (as caixas do Vision vêm na foto girada).
  const size = await bitmapOf(file).catch(() => null);
  return {
    image_base64: content,
    filename: file.name,
    use_ai: useAi,
    ...(lines ? { lines } : {}),
    ...(size ? { width: size.w, height: size.h } : {}),
  };
}

/**
 * Códigos de pagamento da foto: no Mac, o Apple Vision lê o QR do Pix e o código de barras do
 * boleto (intercalado 2 de 5); nos outros sistemas, só o QR (leitor em JavaScript). O núcleo
 * confere o DV e o CRC (`payments.read`).
 */
export async function readPaymentCodes(file: File): Promise<string[]> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const codes = await invoke<string[]>('read_payment_codes_native', {
        contentBase64: await base64Of(file),
      });
      if (codes.length) return codes;
    } catch {
      // Sem o comando (fora do Mac): o leitor de QR abaixo.
    }
  }
  const qr = await readQr(file).catch(() => null);
  return qr ? [qr] : [];
}
