/** Hooks de `receipts.*` (docs/receipts.md): cupons fiscais lidos e o que a família compra. */
import type { ForeignReceipt } from '../generated/ForeignReceipt';
import type { PhotoArgs } from '../generated/PhotoArgs';
import type { PhotoReading } from '../generated/PhotoReading';
import type { Receipt } from '../generated/Receipt';
import type { ReceiptArgs } from '../generated/ReceiptArgs';
import type { ReceiptListArgs } from '../generated/ReceiptListArgs';
import type { ReceiptPreview } from '../generated/ReceiptPreview';
import type { ReceiptSummary } from '../generated/ReceiptSummary';
import type { ReceiptUpdate } from '../generated/ReceiptUpdate';
import { useCore, useCoreMutation } from './coreQuery';

export const useReceipts = (args: ReceiptListArgs) => useCore<Receipt[]>('receipts', 'list', args);

export const useReceipt = (id: string | null) =>
  useCore<Receipt>('receipts', 'get', { id }, { enabled: Boolean(id) });

export const useReceiptSummary = (args: ReceiptListArgs) =>
  useCore<ReceiptSummary>('receipts', 'summary', args);

/** Prévia (não grava): lê o link do QR, o PDF ou o texto. */
/** Nota de outro país pelo QR (Argentina, AFIP): data, CUIT, valor e moeda, sem itens. */
export const useForeignReceipt = () =>
  useCoreMutation<{ url: string }, ForeignReceipt>('receipts.foreign_qr', { affects: [] });

/**
 * QR de uma nota que não é do Brasil e que o app lê: o link da AFIP (Argentina), o link do
 * VeriFactu (Espanha, AEAT) e o texto do QR das faturas de Portugal ("A:…*B:…*…").
 */
export function isForeignReceiptLink(url: string): boolean {
  const u = url.trim();
  return (
    /^https?:\/\/([^/?#]+\.)?(afip|arca)\.gob\.ar([/?#]|$)/i.test(u) ||
    /^https?:\/\/([^/?#]+\.)?(agenciatributaria\.gob\.es|aeat\.es)([/?#]|$)/i.test(u) ||
    (u.startsWith('A:') && u.includes('*F:') && u.includes('*O:'))
  );
}

export const usePreviewReceipt = () =>
  useCoreMutation<ReceiptArgs, ReceiptPreview>('receipts.preview', { affects: [] });

export const useImportReceipt = () =>
  useCoreMutation<ReceiptArgs, Receipt>('receipts.import', {
    affects: ['receipts', 'ledger'],
  });

/** Ajusta o cupom gravado (categoria dos itens, membro, lançamento) sem tocar no documento. */
export const useUpdateReceipt = () =>
  useCoreMutation<ReceiptUpdate, Receipt>('receipts.update', {
    affects: ['receipts', 'food', 'ledger'],
  });

export const useDeleteReceipt = () =>
  useCoreMutation<{ id: string }, null>('receipts.delete', { affects: ['receipts'] });

/** Cupom pela foto, sem QR (docs/ocr-receipts.md): OCR no computador e, se pedido, a IA. */
export const useReadPhoto = () =>
  useCoreMutation<PhotoArgs, PhotoReading>('receipts.read_photo', { affects: [] });
