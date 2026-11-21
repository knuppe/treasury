/**
 * Registro extensível de comandos da barra de comando (⌘K / Ctrl+K).
 *
 * Qualquer tela pode registrar comandos próprios enquanto estiver montada:
 *
 * ```tsx
 * const commands = useMemo<AppCommand[]>(() => [
 *   { id: 'lancamentos.novo', title: 'Novo lançamento', group: 'Lançamentos',
 *     shortcut: ['mod', 'N'], perform: () => abrirFormulario() },
 * ], []);
 * useRegisterCommands(commands);
 * ```
 */
import { useEffect, type ReactNode } from 'react';
import { create } from 'zustand';

import type { IconName } from '@/design/components';
import { t } from '@/i18n';

export interface AppCommand {
  /** Identificador único e estável (`nav.painel`, `vault.lock`). */
  id: string;
  title: string;
  /** Grupo na barra (`Navegação`, `Cofre`, `Exibição`, `Agente` ou o nome da tela). */
  group: string;
  /** Palavras extras para a busca (acentos são ignorados). */
  keywords?: readonly string[];
  /** Atalho mostrado ao lado, ex.: `['mod', 'shift', 'H']`. */
  shortcut?: readonly string[];
  /** Prefixo técnico à esquerda do título (ex.: o índice da tela, `03`). */
  prefix?: string;
  /** Texto curto à direita (ex.: `Fase 4`, `atual`). */
  hint?: string;
  icon?: IconName;
  disabled?: boolean;
  /** Aparece mesmo quando a busca não casa (ex.: "Perguntar ao agente"). */
  alwaysShow?: boolean;
  /** Só aparece com algo digitado (contas, ativos, categorias e lançamentos do cofre). */
  searchOnly?: boolean;
  /** Título dinâmico em função do texto digitado. */
  render?: (search: string) => ReactNode;
  /** Ordem dentro do grupo (menor primeiro). */
  order?: number;
  /** Recebe o texto digitado na barra (comandos como "Perguntar ao agente" usam). */
  perform: (search: string) => void;
}

/** Ordem dos grupos conhecidos; os demais vêm depois, em ordem alfabética. */
/** Ordem dos grupos da barra de comando (os outros vêm depois), no idioma atual. */
export function groupOrder(): string[] {
  return [
    t('casca-cmd-grupo-navegacao'),
    t('casca-cmd-grupo-cofre'),
    t('casca-cmd-grupo-exibicao'),
    t('casca-cmd-grupo-agente'),
  ];
}

interface CommandRegistryState {
  commands: ReadonlyMap<string, AppCommand>;
  register: (commands: readonly AppCommand[]) => () => void;
}

export const useCommandRegistry = create<CommandRegistryState>()((set) => ({
  commands: new Map(),
  register: (commands) => {
    set((state) => {
      const next = new Map(state.commands);
      for (const command of commands) next.set(command.id, command);
      return { commands: next };
    });
    return () =>
      set((state) => {
        const next = new Map(state.commands);
        for (const command of commands) {
          if (next.get(command.id) === command) next.delete(command.id);
        }
        return { commands: next };
      });
  },
}));

/** Registra comandos fora do React; devolve a função que os remove. */
export function registerCommands(commands: readonly AppCommand[]): () => void {
  return useCommandRegistry.getState().register(commands);
}

/**
 * Registra comandos enquanto o componente estiver montado. Passe uma lista
 * memoizada (`useMemo`) para não registrar de novo a cada renderização.
 */
export function useRegisterCommands(commands: readonly AppCommand[]): void {
  useEffect(() => registerCommands(commands), [commands]);
}

/** Minúsculas sem acentos: "lancamentos" encontra "Lançamentos". */
export function normalizeSearch(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
}

/** Comandos agrupados e ordenados para exibição. */
export function groupCommands(
  commands: Iterable<AppCommand>,
): Array<{ group: string; commands: AppCommand[] }> {
  const groups = new Map<string, AppCommand[]>();
  for (const command of commands) {
    const list = groups.get(command.group) ?? [];
    list.push(command);
    groups.set(command.group, list);
  }
  const order = groupOrder();
  const rank = (g: string) => {
    const i = order.indexOf(g);
    return i === -1 ? order.length : i;
  };
  return [...groups.entries()]
    .sort(([a], [b]) => rank(a) - rank(b) || a.localeCompare(b, 'pt-BR'))
    .map(([group, list]) => ({
      group,
      commands: list.sort(
        (a, b) => (a.order ?? 0) - (b.order ?? 0) || a.title.localeCompare(b.title, 'pt-BR'),
      ),
    }));
}

interface CommandBarState {
  open: boolean;
  setOpen: (open: boolean) => void;
  toggle: () => void;
}

/** Abre e fecha a barra de comando de qualquer lugar. */
export const useCommandBar = create<CommandBarState>()((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
  toggle: () => set((s) => ({ open: !s.open })),
}));
