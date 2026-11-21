/**
 * Estado do painel do agente companheiro (Zustand). Persistem só preferências de
 * interface (aberto, largura, enviar o contexto da tela); a conversa atual e o rascunho
 * ficam em memória e somem ao bloquear o cofre.
 */
import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

export const COMPANION_STORAGE_KEY = 'treasury.companion';
export const COMPANION_MIN_WIDTH = 320;
export const COMPANION_MAX_WIDTH = 720;
export const COMPANION_DEFAULT_WIDTH = 400;

export interface CompanionState {
  open: boolean;
  width: number;
  /** Enviar o contexto da tela junto com as mensagens. */
  shareContext: boolean;
  /** Conversa em andamento no painel (memória). */
  conversationId: string | null;
  /** Texto a colocar no campo da mensagem (pedido de outra parte do app). */
  draft: string | null;
  /** Arquivos a anexar na mensagem (ex.: "Analisar com IA" da importação). Só memória. */
  files: File[] | null;
  setOpen: (open: boolean) => void;
  toggle: () => void;
  setWidth: (width: number) => void;
  setShareContext: (share: boolean) => void;
  setConversation: (id: string | null) => void;
  /** Abre o painel com um texto pronto no campo (o envio fica com o usuário). */
  ask: (draft: string) => void;
  takeDraft: () => string | null;
  /** Abre o painel com arquivos anexados (e um texto) esperando o usuário enviar. */
  attach: (files: File[], draft?: string) => void;
  takeFiles: () => File[] | null;
}

export const clampWidth = (w: number) =>
  Math.round(Math.min(COMPANION_MAX_WIDTH, Math.max(COMPANION_MIN_WIDTH, w)));

export const useCompanion = create<CompanionState>()(
  persist(
    (set, get) => ({
      open: false,
      width: COMPANION_DEFAULT_WIDTH,
      shareContext: true,
      conversationId: null,
      draft: null,
      files: null,
      setOpen: (open) => set({ open }),
      toggle: () => set((s) => ({ open: !s.open })),
      setWidth: (width) => set({ width: clampWidth(width) }),
      setShareContext: (shareContext) => set({ shareContext }),
      setConversation: (conversationId) => set({ conversationId }),
      ask: (draft) => set({ open: true, draft }),
      attach: (files, draft) => set({ open: true, files, ...(draft != null ? { draft } : {}) }),
      takeFiles: () => {
        const f = get().files;
        if (f != null) set({ files: null });
        return f;
      },
      takeDraft: () => {
        const d = get().draft;
        if (d != null) set({ draft: null });
        return d;
      },
    }),
    {
      name: COMPANION_STORAGE_KEY,
      version: 1,
      storage: createJSONStorage(() => window.localStorage),
      partialize: (s) => ({ open: s.open, width: s.width, shareContext: s.shareContext }),
      merge: (persisted, current) => {
        const p = (persisted ?? {}) as Partial<CompanionState>;
        return {
          ...current,
          open: typeof p.open === 'boolean' ? p.open : current.open,
          width: typeof p.width === 'number' ? clampWidth(p.width) : current.width,
          shareContext: typeof p.shareContext === 'boolean' ? p.shareContext : current.shareContext,
        };
      },
    },
  ),
);
