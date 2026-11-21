/**
 * Cupom pela foto, sem QR (docs/ocr-receipts.md): o texto é lido no computador, o núcleo monta os
 * itens e confere as contas, e a compra abre já preenchida para revisar e gravar. Quando a conta
 * não fecha, a pessoa pode pedir à IA (a foto vai para o motor escolhido em Ajustes › IA).
 */
import { useEffect, useState } from 'react';

import type { PhotoReading } from '@/api/generated/PhotoReading';
import { useReadPhoto } from '@/api/hooks/receipts';
import { PurchaseDialog } from '@/features/food/PurchaseDialog';

import { photoArgs } from './photo';

/** Lê a foto e abre a compra preenchida; `start(file)` dispara, `dialog` vai na tela. */
export function usePhotoReceipt() {
  const read = useReadPhoto();
  const [current, setCurrent] = useState<{
    file: File;
    url: string;
    reading: PhotoReading;
    round: number;
  }>();
  // A URL da foto vive só enquanto o diálogo está aberto.
  const url = current?.url;
  useEffect(
    () => () => {
      if (url) URL.revokeObjectURL(url);
    },
    [url],
  );
  const start = async (file: File, useAi = false) => {
    const args = await photoArgs(file, useAi);
    read.mutate(args, {
      onSuccess: (reading) =>
        setCurrent((c) => ({
          file,
          url: typeof URL.createObjectURL === 'function' ? URL.createObjectURL(file) : '',
          reading,
          round: (c?.round ?? 0) + 1,
        })),
    });
  };

  const dialog = current ? (
    <PurchaseDialog
      key={current.round}
      purchase={null}
      draft={{
        reading: current.reading,
        ...(current.url ? { imageUrl: current.url } : {}),
        onAskAi: () => void start(current.file, true),
        askingAi: read.isPending,
      }}
      onClose={() => setCurrent(undefined)}
    />
  ) : null;
  return { start, dialog, pending: read.isPending, error: read.error };
}
