/**
 * Anexos da mensagem ao agente, antes do envio: cada arquivo vai ao núcleo
 * (`ai.attachments.prepare`, em base64), que confere o tipo pelos bytes, extrai o texto e o
 * guarda só na memória até o envio; aqui ficam os cartões. Imagens grandes são reduzidas
 * (lado de 2.048 px, JPEG) antes de sair do navegador. Remover um cartão descarta o anexo no
 * núcleo; o envio leva os ids e o núcleo os consome.
 */
import { useCallback, useEffect, useRef, useState } from 'react';

import { call } from '@/api/client';
import type { AttachmentInfo } from '@/api/generated/AttachmentInfo';
import { readNative } from '@/features/receipts/photo';
import { t } from '@/i18n';

export const MAX_FILES = 10;
export const MAX_FILE_BYTES = 20 * 1024 * 1024;
const MAX_IMAGE_EDGE = 2048;
const MAX_IMAGE_BYTES = 3_750_000;

/** Tipos aceitos no seletor de arquivos (o núcleo confere de novo pelos bytes). */
export const ACCEPT =
  'image/png,image/jpeg,image/webp,image/gif,image/heic,image/heif,.heic,.heif,application/pdf,.txt,.md,.csv,.tsv,.json,.ofx,.qfx,.xml,.xlsx,.xls,.ods,.docx,.odt';

export interface DraftAttachment {
  key: string;
  name: string;
  size: number;
  status: 'preparing' | 'ready' | 'error';
  info?: AttachmentInfo;
  error?: string;
  /** Miniatura (object URL) de imagens. */
  preview?: string;
}

function toBase64(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let bin = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(bin);
}

/** Reduz imagens grandes (onde o navegador permite); devolve o arquivo original se não. */
async function shrinkImage(file: File): Promise<Blob> {
  // HEIC (fotos do iPhone): o motor de IA não lê; o WebKit decodifica e vira JPEG.
  const heic = /^image\/hei[cf]$/.test(file.type) || /\.hei[cf]$/i.test(file.name);
  if ((!file.type.startsWith('image/') && !heic) || file.type === 'image/gif') return file;
  if (typeof createImageBitmap !== 'function' || typeof document === 'undefined') return file;
  try {
    const bmp = await createImageBitmap(file);
    const edge = Math.max(bmp.width, bmp.height);
    if (!heic && edge <= MAX_IMAGE_EDGE && file.size <= MAX_IMAGE_BYTES) return file;
    const k = Math.min(1, MAX_IMAGE_EDGE / edge);
    const canvas = document.createElement('canvas');
    canvas.width = Math.round(bmp.width * k);
    canvas.height = Math.round(bmp.height * k);
    canvas.getContext('2d')?.drawImage(bmp, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, 'image/jpeg', 0.85));
    return blob ?? file;
  } catch {
    return file;
  }
}

function errorText(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) return String(e.message);
  return String(e);
}

let seq = 0;

export function useAttachments() {
  const [items, setItems] = useState<DraftAttachment[]>([]);
  const itemsRef = useRef(items);
  useEffect(() => {
    itemsRef.current = items;
  }, [items]);
  /** Cartões removidos antes de o núcleo terminar de preparar. */
  const dropped = useRef(new Set<string>());

  const patch = (key: string, p: Partial<DraftAttachment>) =>
    setItems((xs) => xs.map((x) => (x.key === key ? { ...x, ...p } : x)));

  const add = useCallback((files: Iterable<File>) => {
    const list = [...files];
    const room = MAX_FILES - itemsRef.current.length;
    const take = list.slice(0, Math.max(0, room));
    const extra = list.length - take.length;
    const fresh: DraftAttachment[] = take.map((f) => ({
      key: `a${(seq += 1)}`,
      name: f.name || t('agente-anexo-colado'),
      size: f.size,
      status: 'preparing',
      ...(f.type.startsWith('image/') && typeof URL.createObjectURL === 'function'
        ? { preview: URL.createObjectURL(f) }
        : {}),
    }));
    const rejected: DraftAttachment[] =
      extra > 0
        ? [
            {
              key: `a${(seq += 1)}`,
              name: t('agente-anexos-a-mais', { n: extra }),
              size: 0,
              status: 'error',
              error: t('agente-anexos-maximo', { max: MAX_FILES }),
            },
          ]
        : [];
    setItems((xs) => [...xs, ...fresh, ...rejected]);
    take.forEach((f, i) => {
      const key = fresh[i]!.key;
      void (async () => {
        if (f.size > MAX_FILE_BYTES) {
          patch(key, { status: 'error', error: t('agente-anexo-grande') });
          return;
        }
        try {
          const blob = await shrinkImage(f);
          const data = toBase64(await blob.arrayBuffer());
          // Imagem: no Mac, o Apple Vision lê o texto aqui; fora dele, o núcleo usa o OCR
          // embutido. O texto vai ao agente se a pessoa quiser (texto, imagem ou os dois).
          const lines = blob.type.startsWith('image/') ? await readNative(data) : null;
          const info = await call<AttachmentInfo>('ai.attachments.prepare', {
            name: fresh[i]!.name,
            mime: blob.type || f.type || undefined,
            data,
            ...(lines?.length ? { ocr_lines: lines } : {}),
          });
          if (dropped.current.delete(key)) {
            void call('ai.attachments.discard', { ids: [info.id] }).catch(() => {});
            return;
          }
          patch(key, { status: 'ready', info });
        } catch (e) {
          patch(key, { status: 'error', error: errorText(e) });
        }
      })();
    });
  }, []);

  const remove = useCallback((key: string) => {
    const it = itemsRef.current.find((x) => x.key === key);
    if (it?.preview) URL.revokeObjectURL?.(it.preview);
    if (it?.info) void call('ai.attachments.discard', { ids: [it.info.id] }).catch(() => {});
    else if (it?.status === 'preparing') dropped.current.add(key);
    setItems((xs) => xs.filter((x) => x.key !== key));
  }, []);

  /** Depois do envio: o núcleo já consumiu os anexos. */
  const clear = useCallback(() => {
    for (const it of itemsRef.current) if (it.preview) URL.revokeObjectURL?.(it.preview);
    setItems([]);
  }, []);

  // Sai da tela com anexos não enviados: descarta no núcleo.
  useEffect(
    () => () => {
      const ids = itemsRef.current.flatMap((x) => (x.info ? [x.info.id] : []));
      if (ids.length) void call('ai.attachments.discard', { ids }).catch(() => {});
    },
    [],
  );

  const ready = items.filter((x) => x.status === 'ready' && x.info).map((x) => x.info!);
  return {
    items,
    ready,
    ids: ready.map((x) => x.id),
    busy: items.some((x) => x.status === 'preparing'),
    add,
    remove,
    clear,
  };
}

export type AttachmentsDraft = ReturnType<typeof useAttachments>;

/** Arquivos colados (⌘V / Ctrl+V): imagens de captura de tela e arquivos copiados. */
export function filesFromClipboard(data: DataTransfer | null): File[] {
  if (!data) return [];
  const out: File[] = [];
  for (const it of Array.from(data.items ?? [])) {
    if (it.kind === 'file') {
      const f = it.getAsFile();
      if (f) out.push(f);
    }
  }
  if (out.length === 0 && data.files?.length) out.push(...Array.from(data.files));
  return out;
}
