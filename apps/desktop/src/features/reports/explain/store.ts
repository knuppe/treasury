/** Painel de explicação ("medido, nunca inventado"): qual medida está aberta. */
import { create } from 'zustand';

import type { ExplainArgs } from '@/api/generated/ExplainArgs';

export interface ExplainTarget {
  /** Rótulo da medida ("Despesas de outubro de 2026"). */
  title: string;
  args: ExplainArgs;
  /** Link para os lançamentos da medida (opcional). */
  link?: string;
}

interface ExplainState {
  target: ExplainTarget | null;
  open: (t: ExplainTarget) => void;
  close: () => void;
}

export const useExplainStore = create<ExplainState>()((set) => ({
  target: null,
  open: (target) => set({ target }),
  close: () => set({ target: null }),
}));

export const openExplain = (t: ExplainTarget) => useExplainStore.getState().open(t);
