/**
 * O que a caixa de mensagem do agente (tela 10 e painel lateral) tem em comum além do texto:
 * anexos (seletor, arrastar e soltar, colar), o aviso de anexos que nenhum motor candidato lê
 * e o esforço da conversa. `extraArgs()` devolve os campos de `ai.chat_start` que saem
 * daqui; `sent()` limpa os cartões depois do envio.
 */
import type { ClipboardEvent } from 'react';

import type { ChatStartArgs } from '@/api/generated/ChatStartArgs';
import type { Engine } from '@/api/generated/Engine';
import type { ImageMode } from '@/api/generated/ImageMode';
import {
  useAgentSession,
  useConversationEffort,
  useConversationImageMode,
} from '@/state/agentSession';

import { unreadable, useFileDrop, type ImageChoice } from './Attachments';
import { filesFromClipboard, useAttachments } from './useAttachments';

export function useComposer({
  conversationId,
  candidates,
  running,
}: {
  conversationId: string | null;
  /** Motores que podem responder: o escolhido ou os prontos da rota de chat. */
  candidates: readonly Engine[];
  running: boolean;
}) {
  const attachments = useAttachments();
  const drop = useFileDrop(attachments.add, running);
  const effort = useConversationEffort(conversationId);
  // Imagens com texto lido pelo OCR: a pessoa escolhe o que vai (lembrado na conversa). A
  // sugestão segue a confiança do OCR: alta, só o texto (a imagem fica no computador); baixa,
  // os dois. Sem motor que leia imagens, só o texto.
  const chosen = useConversationImageMode(conversationId);
  const setImageMode = useAgentSession((s) => s.setImageMode);
  const ocrImages = attachments.ready.filter(
    (a) => a.kind === 'image' && a.text_chars > 0 && a.ocr_confidence != null,
  );
  const confidence = ocrImages.length
    ? ocrImages.reduce((n, a) => n + (a.ocr_confidence ?? 0), 0) / ocrImages.length
    : 0;
  const vision = candidates.length === 0 || candidates.some((e) => e.caps.images);
  const suggested: ImageMode = confidence >= 0.9 ? 'text' : 'both';
  const imageMode: ImageMode = vision ? (chosen ?? suggested) : 'text';
  const imageChoice: ImageChoice = {
    visible: ocrImages.length > 0,
    mode: imageMode,
    setMode: (m) => setImageMode(conversationId, m),
    vision,
    confidence,
  };
  const unsupported = unreadable(attachments.ready, candidates, imageMode);

  const onPaste = (e: ClipboardEvent<HTMLTextAreaElement>) => {
    const files = filesFromClipboard(e.clipboardData);
    if (files.length === 0) return;
    e.preventDefault();
    attachments.add(files);
  };

  const canSend = (text: string) =>
    !running && !attachments.busy && (text.trim().length > 0 || attachments.ready.length > 0);

  const extraArgs = (): Partial<ChatStartArgs> => ({
    ...(attachments.ids.length ? { attachment_ids: attachments.ids } : {}),
    ...(unsupported.length ? { skip_unsupported: true } : {}),
    ...(effort !== 'default' ? { effort } : {}),
    ...(ocrImages.length ? { image_mode: imageMode } : {}),
  });

  return {
    attachments,
    drop,
    unsupported,
    unsupportedIds: new Set(unsupported.map((a) => a.id)),
    imageChoice,
    onPaste,
    canSend,
    extraArgs,
    /** Metadados para o balão da mensagem enviada. */
    sentAttachments: () => attachments.ready,
    sent: attachments.clear,
  };
}
