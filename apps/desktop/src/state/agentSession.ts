/**
 * Estado do agente que só vale na sessão desbloqueada (memória, nunca o localStorage):
 * quais blocos de raciocínio e de ferramentas o usuário abriu em cada mensagem, em quais
 * conversas ele liberou a leitura do computador, o esforço de raciocínio e o motor escolhidos
 * em cada conversa. Some ao bloquear o cofre (`clearSensitiveCache`).
 */
import { create } from 'zustand';

import type { Effort } from '@/api/generated/Effort';
import type { ImageMode } from '@/api/generated/ImageMode';

/** Chave da conversa ainda sem id (antes da primeira resposta). */
export const NEW_CONVERSATION = '__nova__';

interface AgentSessionState {
  /** `<mensagem>:<bloco>` → aberto. */
  expanded: Record<string, boolean>;
  /** Conversa → leitura do computador liberada pelo usuário. */
  computer: Record<string, boolean>;
  /** Conversa → esforço de raciocínio (ausente = padrão do motor). */
  effort: Record<string, Effort>;
  /** Conversa → motor escolhido no chat (ausente = automático, pela rota). */
  engine: Record<string, string>;
  /** Conversa → o que vai de imagens com texto lido pelo OCR (ausente = a sugestão). */
  imageMode: Record<string, ImageMode>;
  setExpanded: (key: string, open: boolean) => void;
  /** Leva o estado de uma resposta ao vivo para a mensagem gravada. */
  alias: (from: string, to: string) => void;
  setComputer: (conversation: string | null, on: boolean) => void;
  setEffort: (conversation: string | null, effort: Effort) => void;
  setImageMode: (conversation: string | null, mode: ImageMode) => void;
  /** `null` volta ao automático. */
  setEngine: (conversation: string | null, engineId: string | null) => void;
  /** A conversa nova ganhou id: a liberação do computador e o esforço vão junto. */
  carryComputer: (to: string) => void;
  reset: () => void;
}

function moveNew<T>(map: Record<string, T>, to: string): Record<string, T> | null {
  if (!(NEW_CONVERSATION in map)) return null;
  const { [NEW_CONVERSATION]: value, ...rest } = map;
  return { ...rest, [to]: value };
}

export const useAgentSession = create<AgentSessionState>()((set) => ({
  expanded: {},
  computer: {},
  effort: {},
  engine: {},
  imageMode: {},
  setExpanded: (key, open) => set((s) => ({ expanded: { ...s.expanded, [key]: open } })),
  alias: (from, to) =>
    set((s) => {
      const moved: Record<string, boolean> = {};
      for (const [k, v] of Object.entries(s.expanded)) {
        if (k.startsWith(`${from}:`)) moved[`${to}${k.slice(from.length)}`] = v;
      }
      return Object.keys(moved).length ? { expanded: { ...s.expanded, ...moved } } : s;
    }),
  setComputer: (conversation, on) =>
    set((s) => ({ computer: { ...s.computer, [conversation ?? NEW_CONVERSATION]: on } })),
  setEffort: (conversation, effort) =>
    set((s) => ({ effort: { ...s.effort, [conversation ?? NEW_CONVERSATION]: effort } })),
  setImageMode: (conversation, mode) =>
    set((s) => ({ imageMode: { ...s.imageMode, [conversation ?? NEW_CONVERSATION]: mode } })),
  setEngine: (conversation, engineId) =>
    set((s) => {
      const key = conversation ?? NEW_CONVERSATION;
      const { [key]: _old, ...rest } = s.engine;
      return { engine: engineId ? { ...rest, [key]: engineId } : rest };
    }),
  carryComputer: (to) =>
    set((s) => {
      const computer = s.computer[NEW_CONVERSATION] ? moveNew(s.computer, to) : null;
      const effort = moveNew(s.effort, to);
      const engine = moveNew(s.engine, to);
      const imageMode = moveNew(s.imageMode, to);
      if (!computer && !effort && !engine && !imageMode) return s;
      return {
        ...(computer ? { computer } : {}),
        ...(effort ? { effort } : {}),
        ...(engine ? { engine } : {}),
        ...(imageMode ? { imageMode } : {}),
      };
    }),
  reset: () => set({ expanded: {}, computer: {}, effort: {}, engine: {}, imageMode: {} }),
}));

/** A leitura do computador está liberada nesta conversa? */
export function useComputerAccess(conversation: string | null): boolean {
  return useAgentSession((s) => Boolean(s.computer[conversation ?? NEW_CONVERSATION]));
}

/** Esforço escolhido nesta conversa (`default` = o do motor). */
export function useConversationEffort(conversation: string | null): Effort {
  return useAgentSession((s) => s.effort[conversation ?? NEW_CONVERSATION] ?? 'default');
}

/** Motor escolhido nesta conversa (`null` = automático, pela rota). */
export function useConversationEngine(conversation: string | null): string | null {
  return useAgentSession((s) => s.engine[conversation ?? NEW_CONVERSATION] ?? null);
}

/** Imagens com texto do OCR nesta conversa: texto, imagem ou os dois (`null` = a sugestão). */
export function useConversationImageMode(conversation: string | null): ImageMode | null {
  return useAgentSession((s) => s.imageMode[conversation ?? NEW_CONVERSATION] ?? null);
}
