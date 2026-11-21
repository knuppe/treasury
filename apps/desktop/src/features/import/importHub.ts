/** Janela única de importação (aberta de qualquer tela). */
import { create } from 'zustand';

interface ImportHubState {
  open: boolean;
  setOpen: (open: boolean) => void;
  /** Arquivos para a janela carregar ao abrir (ex.: os que chegaram em Downloads). */
  pending: File[];
  /**
   * De onde veio cada arquivo da pasta Downloads (nome → caminho), para mandar à Lixeira
   * depois de importado com sucesso quando o ajuste estiver ligado.
   */
  origins: Record<string, string>;
  /** Abre a janela já com estes arquivos. */
  openWith: (files: File[], origins?: Record<string, string>) => void;
  /** A janela pega os arquivos pendentes (e a fila esvazia). */
  takePending: () => File[];
  /** Esquece a origem (arquivo já tratado). */
  forget: (names: string[]) => void;
}

export const useImportHub = create<ImportHubState>()((set, get) => ({
  open: false,
  setOpen: (open) => set({ open }),
  pending: [],
  origins: {},
  openWith: (files, origins = {}) =>
    set((s) => ({
      open: true,
      pending: [...s.pending, ...files],
      origins: { ...s.origins, ...origins },
    })),
  takePending: () => {
    const files = get().pending;
    if (files.length) set({ pending: [] });
    return files;
  },
  forget: (names) =>
    set((s) => {
      const origins = { ...s.origins };
      for (const n of names) delete origins[n];
      return { origins };
    }),
}));
