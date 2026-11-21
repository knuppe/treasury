/**
 * Peças de anexos da caixa de mensagem: botão de anexar (seletor de arquivos), área que aceita
 * arrastar e soltar, cartões dos anexos (com remover), aviso de anexos que o motor não lê e a
 * lista de anexos de uma mensagem já enviada.
 */
import { useRef, useState, type DragEvent, type ReactNode } from 'react';

import type { AttachmentInfo } from '@/api/generated/AttachmentInfo';
import type { Engine } from '@/api/generated/Engine';
import type { EngineCaps } from '@/api/generated/EngineCaps';
import type { ImageMode } from '@/api/generated/ImageMode';
import { Icon, Private, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t } from '@/i18n';

import { ACCEPT, type AttachmentsDraft, type DraftAttachment } from './useAttachments';

export { formatBytes };

/** Espelho de `Attachment::readable_by` no núcleo. */
export function readableBy(a: AttachmentInfo, caps: EngineCaps, mode: ImageMode = 'both'): boolean {
  switch (a.kind) {
    case 'image':
      if (a.text_chars === 0) return caps.images;
      // Com texto do OCR: "só a imagem" precisa de visão; o resto vai pelo texto.
      return mode === 'image' ? caps.images : true;
    case 'pdf':
      return caps.pdf || a.text_chars > 0;
    default:
      return true;
  }
}

/**
 * Anexos que nenhum dos motores candidatos lê (o escolhido, ou os da rota): o núcleo pula
 * motores que não leem e, se nenhum servir, só envia com `skip_unsupported`.
 */
export function unreadable(
  items: readonly AttachmentInfo[],
  candidates: readonly Engine[],
  mode: ImageMode = 'both',
) {
  if (candidates.length === 0) return [];
  return items.filter((a) => !candidates.some((e) => readableBy(a, e.caps, mode)));
}

function detail(a: AttachmentInfo): string {
  const bits = [formatBytes(a.size)];
  if (a.width && a.height) bits.push(`${a.width}×${a.height}`);
  if (a.pages) bits.push(t('agente-anexo-paginas', { n: a.pages }));
  if (a.kind !== 'image' && a.text_chars > 0)
    bits.push(t('agente-anexo-caracteres', { n: a.text_chars }));
  if (a.kind === 'image' && a.text_chars > 0) bits.push(t('agente-anexo-texto-lido'));
  if (a.kind === 'pdf' && a.text_chars === 0) bits.push(t('agente-anexo-escaneado'));
  if (a.truncated) bits.push(t('agente-anexo-cortado'));
  return bits.join(' · ');
}

/** Arrastar e soltar arquivos sobre a caixa de mensagem. */
export function useFileDrop(onFiles: (files: File[]) => void, disabled?: boolean) {
  const [over, setOver] = useState(false);
  const depth = useRef(0);
  const hasFiles = (e: DragEvent) => Array.from(e.dataTransfer?.types ?? []).includes('Files');
  return {
    over,
    props: {
      onDragEnter: (e: DragEvent) => {
        if (disabled || !hasFiles(e)) return;
        e.preventDefault();
        depth.current += 1;
        setOver(true);
      },
      onDragOver: (e: DragEvent) => {
        if (disabled || !hasFiles(e)) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = 'copy';
      },
      onDragLeave: (e: DragEvent) => {
        if (disabled || !hasFiles(e)) return;
        depth.current = Math.max(0, depth.current - 1);
        if (depth.current === 0) setOver(false);
      },
      onDrop: (e: DragEvent) => {
        if (disabled || !hasFiles(e)) return;
        e.preventDefault();
        depth.current = 0;
        setOver(false);
        const files = Array.from(e.dataTransfer.files ?? []);
        if (files.length) onFiles(files);
      },
    },
  };
}

export function AttachButton({
  onFiles,
  disabled,
  compact,
}: {
  onFiles: (files: File[]) => void;
  disabled?: boolean;
  compact?: boolean;
}) {
  const input = useRef<HTMLInputElement>(null);
  return (
    <Tooltip content={t('agente-anexar-dica')}>
      <button
        type="button"
        aria-label={t('agente-anexar-arquivos')}
        disabled={disabled}
        onClick={() => input.current?.click()}
        className="inline-flex h-7 shrink-0 items-center gap-1.5 rounded-control border border-line px-2 text-[11px] whitespace-nowrap text-dim transition-colors hover:text-fg disabled:opacity-50"
      >
        <Icon name="clip" size={12} />
        {compact ? null : t('agente-anexar')}
        <input
          ref={input}
          type="file"
          multiple
          accept={ACCEPT}
          hidden
          tabIndex={-1}
          aria-hidden
          data-testid="attach-input"
          onChange={(e) => {
            const files = Array.from(e.target.files ?? []);
            e.target.value = '';
            if (files.length) onFiles(files);
          }}
        />
      </button>
    </Tooltip>
  );
}

function Chip({
  name,
  sub,
  preview,
  tone,
  busy,
  onRemove,
  title,
}: {
  name: string;
  sub: ReactNode;
  preview?: string;
  tone?: 'warning' | 'negative';
  busy?: boolean;
  onRemove?: () => void;
  title?: string;
}) {
  return (
    <li
      title={title}
      className={cx(
        'flex max-w-[260px] min-w-0 items-center gap-2 rounded-control border bg-bg py-1 pr-1 pl-1.5',
        tone === 'negative'
          ? 'border-negative/60'
          : tone === 'warning'
            ? 'border-warning/60'
            : 'border-line',
      )}
    >
      {preview ? (
        <img src={preview} alt="" className="size-8 shrink-0 rounded-[3px] object-cover" />
      ) : (
        <span className="grid size-8 shrink-0 place-items-center rounded-[3px] bg-surface text-dim">
          <Icon name={busy ? 'refresh' : 'clip'} size={13} className={busy ? 'animate-spin' : ''} />
        </span>
      )}
      <span className="flex min-w-0 flex-col">
        <span className="truncate text-[12px] text-fg">{name}</span>
        <span
          className={cx(
            'truncate text-[10.5px]',
            tone === 'negative'
              ? 'text-negative'
              : tone === 'warning'
                ? 'text-warning'
                : 'text-dim',
          )}
        >
          {sub}
        </span>
      </span>
      {onRemove ? (
        <button
          type="button"
          aria-label={t('agente-remover', { nome: name })}
          onClick={onRemove}
          className="ml-auto grid size-6 shrink-0 place-items-center rounded-control text-dim hover:bg-surface hover:text-fg"
        >
          <Icon name="x" size={11} />
        </button>
      ) : null}
    </li>
  );
}

/** Cartões dos anexos ainda não enviados. */
export function AttachmentTray({
  draft,
  unsupported,
}: {
  draft: AttachmentsDraft;
  unsupported: ReadonlySet<string>;
}) {
  if (draft.items.length === 0) return null;
  return (
    <Private>
      <ul aria-label={t('agente-anexos-mensagem')} className="flex flex-wrap gap-1.5">
        {draft.items.map((it: DraftAttachment) => {
          const info = it.info;
          const warn = info ? unsupported.has(info.id) : false;
          return (
            <Chip
              key={it.key}
              name={it.name}
              preview={it.preview}
              busy={it.status === 'preparing'}
              tone={it.status === 'error' ? 'negative' : warn ? 'warning' : undefined}
              title={info?.warnings.join('\n') || it.error}
              sub={
                it.status === 'preparing'
                  ? t('agente-anexo-preparando')
                  : it.status === 'error'
                    ? (it.error ?? t('agente-falhou'))
                    : warn
                      ? t('agente-anexo-nao-lido')
                      : info
                        ? detail(info)
                        : ''
              }
              onRemove={() => draft.remove(it.key)}
            />
          );
        })}
      </ul>
    </Private>
  );
}

/** Aviso antes do envio: algum anexo não é lido por nenhum motor candidato. */
export function UnsupportedNotice({
  items,
  engines,
}: {
  items: readonly AttachmentInfo[];
  engines: readonly Pick<Engine, 'name'>[];
}) {
  if (items.length === 0) return null;
  const tipo = items.some((a) => a.kind === 'image') ? 'imagem' : 'pdf';
  return (
    <p role="alert" className="flex items-start gap-1.5 text-[12px] text-warning">
      <Icon name="alert" size={12} className="mt-0.5 shrink-0" />
      <span>
        {engines.length === 1
          ? t('agente-anexo-motor-nao-le', { motor: engines[0]!.name, tipo })
          : t('agente-anexo-rota-nao-le', { tipo })}
      </span>
    </p>
  );
}

/** Anexos de uma mensagem enviada (gravada ou ao vivo). */
export function MessageAttachments({ items }: { items: readonly AttachmentInfo[] }) {
  if (items.length === 0) return null;
  return (
    <ul aria-label={t('agente-anexos')} className="mt-1.5 flex flex-wrap gap-1.5">
      {items.map((a) => (
        <Chip
          key={a.id}
          name={a.name}
          sub={a.stored ? detail(a) : t('agente-anexo-nao-guardado', { detalhe: detail(a) })}
          title={a.stored ? t('agente-anexo-guardado-dica') : t('agente-anexo-nao-guardado-dica')}
        />
      ))}
    </ul>
  );
}

/** Contorno de "solte para anexar" sobre a caixa de mensagem. */
export function DropOverlay({ over }: { over: boolean }) {
  if (!over) return null;
  return (
    <div className="pointer-events-none absolute inset-1 z-10 grid place-items-center rounded-card border-2 border-dashed border-accent bg-accent-soft/80 text-[13px] text-accent">
      {t('agente-solte')}
    </div>
  );
}

/** O que vai de imagens com texto lido pelo OCR: sugestão e escolha da conversa. */
export interface ImageChoice {
  /** Há imagem com texto do OCR entre os anexos. */
  visible: boolean;
  mode: ImageMode;
  setMode: (m: ImageMode) => void;
  /** Algum motor candidato lê imagens. */
  vision: boolean;
  /** Confiança média do OCR (0–1). */
  confidence: number;
}

/**
 * Perguntar o que mandar ao agente: o texto lido no computador (a imagem não sai dele), a
 * imagem ou os dois. Sem motor que leia imagens, só o texto.
 */
export function ImageModeChoice({ choice }: { choice: ImageChoice }) {
  if (!choice.visible) return null;
  const pct = Math.round(choice.confidence * 100);
  if (!choice.vision)
    return (
      <p className="flex items-start gap-1.5 text-[12px] text-dim">
        <Icon name="info" size={12} className="mt-0.5 shrink-0" />
        <span>{t('agente-imagem-so-texto', { confianca: pct })}</span>
      </p>
    );
  const options: { value: ImageMode; label: string }[] = [
    { value: 'text', label: t('agente-imagem-modo-texto') },
    { value: 'image', label: t('agente-imagem-modo-imagem') },
    { value: 'both', label: t('agente-imagem-modo-ambos') },
  ];
  return (
    <div
      role="radiogroup"
      aria-label={t('agente-imagem-modo-rotulo')}
      className="flex flex-wrap items-center gap-1.5 text-[12px]"
    >
      <span className="text-dim">{t('agente-imagem-modo-pergunta', { confianca: pct })}</span>
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          role="radio"
          aria-checked={choice.mode === o.value}
          onClick={() => choice.setMode(o.value)}
          className={cx(
            'h-6 rounded-control border px-2 transition-colors',
            choice.mode === o.value
              ? 'border-accent/60 bg-accent-soft text-accent'
              : 'border-line text-dim hover:text-fg',
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
