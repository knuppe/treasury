/**
 * Código de barras de produto (EAN-13, EAN-8, UPC-E, ITF-14) numa foto, pelo Apple Vision (comando
 * `read_barcode_native`, só no Mac). A imagem é lida no computador e não sai dele. O núcleo
 * confere o dígito verificador quando o código é gravado.
 */
import { isTauri } from '@/api/client';

import { base64Of } from './qr';

/** O leitor de código de barras pela foto está disponível (no app). */
export const canReadBarcode = () => isTauri();

/** Os dígitos do código de barras da foto, ou `null` se não achar (ou fora do Mac). */
export async function readBarcode(file: Blob): Promise<string | null> {
  if (!isTauri()) return null;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<string | null>('read_barcode_native', {
      contentBase64: await base64Of(file),
    });
  } catch {
    return null;
  }
}
