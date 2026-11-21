/**
 * Ações rápidas da barra ⌘K que abrem uma janela em qualquer tela (o cofre de demonstração);
 * a janela fica no casco do app (`QuickActionsHost`).
 */
import { create } from 'zustand';

interface QuickActionsState {
  demo: boolean;
  openDemo: () => void;
  setDemo: (open: boolean) => void;
}

export const useQuickActions = create<QuickActionsState>()((set) => ({
  demo: false,
  openDemo: () => set({ demo: true }),
  setDemo: (demo) => set({ demo }),
}));
