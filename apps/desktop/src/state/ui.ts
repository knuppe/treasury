/**
 * Estado de interface (Zustand), persistido em localStorage.
 *
 * Só preferências de UI vivem aqui (tema, modo privacidade, menu lateral). Dados financeiros
 * nunca vão para o localStorage: eles ficam no cofre cifrado e no cache em
 * memória do TanStack Query, que é limpo ao bloquear.
 */
import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

import { applyTheme, DEFAULT_THEME, isThemeId, toggledTheme, type ThemeId } from '@/design/themes';

export const UI_STORAGE_KEY = 'treasury.ui';

export interface UiState {
  theme: ThemeId;
  /** Modo privacidade: todos os valores viram `•••••`. */
  privacy: boolean;
  /** Liga o modo privacidade sempre que o cofre é desbloqueado (padrão: sim). */
  privacyOnUnlock: boolean;
  /** Menu lateral recolhido: só os números das telas, mais área de trabalho. */
  sidebarCollapsed: boolean;
  /** Painel completo mesmo nos níveis Iniciante e Intermediário ("Ver painel completo"). */
  fullDashboard: boolean;
  /** Lista de conversas aberta ao lado do chat na tela Agente (padrão: recolhida). */
  agentSessionsOpen: boolean;
  /** "Ver como" um membro da família (docs/members.md), no cofre em que foi escolhido. */
  viewAs: { vaultId: string; memberId: string } | null;
  /**
   * Ícones `?` com explicação dos termos: `auto` segue o nível do perfil (Iniciante e
   * Intermediário), `on` mostra sempre, `off` nunca.
   */
  helpTips: HelpTips;
  setTheme: (theme: ThemeId) => void;
  /** Alterna claro/escuro: de um tema claro vai ao Escuro; de um escuro, ao Claro. */
  toggleTheme: () => void;
  setPrivacy: (privacy: boolean) => void;
  togglePrivacy: () => void;
  setPrivacyOnUnlock: (on: boolean) => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;
  setFullDashboard: (full: boolean) => void;
  setAgentSessionsOpen: (open: boolean) => void;
  setViewAs: (viewAs: { vaultId: string; memberId: string } | null) => void;
  setHelpTips: (helpTips: HelpTips) => void;
}

export type HelpTips = 'auto' | 'on' | 'off';
const HELP_TIPS: readonly HelpTips[] = ['auto', 'on', 'off'];

export const UI_DEFAULTS = {
  theme: DEFAULT_THEME,
  privacy: false,
  privacyOnUnlock: true,
  sidebarCollapsed: false,
  fullDashboard: false,
  agentSessionsOpen: false,
  viewAs: null,
  helpTips: 'auto',
} as const;

type PersistedUi = Pick<
  UiState,
  | 'theme'
  | 'privacy'
  | 'privacyOnUnlock'
  | 'sidebarCollapsed'
  | 'fullDashboard'
  | 'agentSessionsOpen'
  | 'viewAs'
  | 'helpTips'
>;

export const useUi = create<UiState>()(
  persist(
    (set) => ({
      ...UI_DEFAULTS,
      setTheme: (theme) => set({ theme }),
      toggleTheme: () => set((s) => ({ theme: toggledTheme(s.theme) })),
      setPrivacy: (privacy) => set({ privacy }),
      togglePrivacy: () => set((s) => ({ privacy: !s.privacy })),
      setPrivacyOnUnlock: (privacyOnUnlock) => set({ privacyOnUnlock }),
      setSidebarCollapsed: (sidebarCollapsed) => set({ sidebarCollapsed }),
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      setFullDashboard: (fullDashboard) => set({ fullDashboard }),
      setAgentSessionsOpen: (agentSessionsOpen) => set({ agentSessionsOpen }),
      setViewAs: (viewAs) => set({ viewAs }),
      setHelpTips: (helpTips) => set({ helpTips }),
    }),
    {
      name: UI_STORAGE_KEY,
      version: 1,
      storage: createJSONStorage(() => window.localStorage),
      partialize: (s): PersistedUi => ({
        theme: s.theme,
        privacy: s.privacy,
        privacyOnUnlock: s.privacyOnUnlock,
        sidebarCollapsed: s.sidebarCollapsed,
        fullDashboard: s.fullDashboard,
        agentSessionsOpen: s.agentSessionsOpen,
        viewAs: s.viewAs,
        helpTips: s.helpTips,
      }),
      merge: (persisted, current) => {
        const p = (persisted ?? {}) as Partial<PersistedUi>;
        const bool = (v: unknown, fallback: boolean) => (typeof v === 'boolean' ? v : fallback);
        return {
          ...current,
          // Temas que saíram (De Volta para o Futuro, Windows 95) deram lugar ao Lo-fi.
          theme: isThemeId(p.theme)
            ? p.theme
            : p.theme === 'bttf' || p.theme === 'win95'
              ? 'lofi'
              : current.theme,
          privacy: bool(p.privacy, current.privacy),
          privacyOnUnlock: bool(p.privacyOnUnlock, current.privacyOnUnlock),
          sidebarCollapsed: bool(p.sidebarCollapsed, current.sidebarCollapsed),
          fullDashboard: bool(p.fullDashboard, current.fullDashboard),
          agentSessionsOpen: bool(p.agentSessionsOpen, current.agentSessionsOpen),
          viewAs:
            p.viewAs &&
            typeof p.viewAs.vaultId === 'string' &&
            typeof p.viewAs.memberId === 'string'
              ? p.viewAs
              : null,
          helpTips: HELP_TIPS.includes(p.helpTips as HelpTips) ? p.helpTips! : current.helpTips,
        };
      },
    },
  ),
);

/**
 * Mantém `data-theme` do <html> em sincronia com a preferência salva. A primeira
 * aplicação é imediata; as trocas seguintes esmaecem (sem movimento reduzido).
 */
export function syncThemeWithDocument(): () => void {
  applyTheme(useUi.getState().theme);
  return useUi.subscribe((state, prev) => {
    if (state.theme !== prev.theme) applyTheme(state.theme, undefined, { animate: true });
  });
}

export const usePrivacy = () => useUi((s) => s.privacy);

/**
 * Chamado quando um cofre abre: liga a privacidade se o ajuste pedir. O cofre de demonstração
 * abre sempre com ela desligada (dados fictícios, feitos para ver).
 */
export function applyPrivacyOnUnlock(status?: { demo: boolean }): void {
  const { privacyOnUnlock, setPrivacy } = useUi.getState();
  if (status?.demo) setPrivacy(false);
  else if (privacyOnUnlock) setPrivacy(true);
}
