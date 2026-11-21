/**
 * Contexto da tela para o agente companheiro: cada tela descreve, de forma compacta e
 * estruturada, o que está mostrando (período, filtros, itens selecionados, números
 * visíveis) com `usePageContext`. O painel do agente lê o contexto da tela aberta e o envia
 * junto com cada mensagem (`ai.chat_start { page_context }`), se o usuário deixar.
 *
 * O núcleo põe esse objeto num bloco `<contexto_da_tela>` que o agente trata como dado,
 * nunca como instrução. Valores monetários vão em centavos com a moeda, como nas
 * ferramentas. Nada daqui vai para o localStorage.
 */
import { useEffect, useId } from 'react';
import { create } from 'zustand';

/** Um número visível na tela: dinheiro em centavos (com moeda) ou outro valor. */
export type PageFigure = { amount: number; currency: string } | number | string | boolean | null;

export interface PageContext {
  /** Id da tela (`contas`, `lancamentos`…). */
  screen: string;
  /** Título visível (`03 · Contas`). */
  title: string;
  /** Aba ou visão dentro da tela. */
  view?: string;
  /** Período mostrado (datas `YYYY-MM-DD` ou mês `YYYY-MM`). */
  period?: { from?: string; to?: string; month?: string; basis?: string };
  /** Filtros aplicados. */
  filters?: Record<string, unknown>;
  /** Entidades selecionadas ou abertas (ids e nomes). */
  selection?: Record<string, unknown>;
  /** Números principais visíveis. */
  figures?: Record<string, PageFigure>;
  /** Linhas visíveis mais relevantes (cortadas pelo painel). */
  items?: Record<string, unknown>[];
  /** Resumo por grupo de gasto (Orçamento: para onde vai a renda do mês). */
  groups?: Record<string, unknown>[];
}

interface Entry {
  id: string;
  ctx: PageContext;
}

interface PageContextState {
  stack: Entry[];
  set: (id: string, ctx: PageContext) => void;
  remove: (id: string) => void;
}

export const usePageContextStore = create<PageContextState>()((set) => ({
  stack: [],
  set: (id, ctx) =>
    set((s) => {
      const i = s.stack.findIndex((e) => e.id === id);
      if (i < 0) return { stack: [...s.stack, { id, ctx }] };
      const stack = s.stack.slice();
      stack[i] = { id, ctx };
      return { stack };
    }),
  remove: (id) =>
    set((s) =>
      s.stack.some((e) => e.id === id) ? { stack: s.stack.filter((e) => e.id !== id) } : s,
    ),
}));

/**
 * Registra o contexto desta tela enquanto ela estiver montada. Passe `null` enquanto os
 * dados carregam. O último registrado (a tela ou painel mais interno) vale.
 */
export function usePageContext(ctx: PageContext | null | undefined): void {
  const id = useId();
  const key = ctx ? JSON.stringify(ctx) : '';
  useEffect(() => {
    const store = usePageContextStore.getState();
    if (!key) store.remove(id);
    else store.set(id, JSON.parse(key) as PageContext);
  }, [id, key]);
  useEffect(() => () => usePageContextStore.getState().remove(id), [id]);
}

/** Contexto registrado pela tela aberta (ou `null`). */
export function useCurrentPageContext(): PageContext | null {
  return usePageContextStore((s) => s.stack[s.stack.length - 1]?.ctx ?? null);
}

/** Dinheiro em centavos para o contexto. */
export function money(amount: number | null | undefined, currency: string): PageFigure {
  return amount == null ? null : { amount, currency };
}

/** Tamanho máximo enviado (o núcleo recusa acima de 8.000 caracteres). */
export const MAX_CONTEXT_CHARS = 7_000;
/** Máximo de linhas visíveis enviadas. */
export const MAX_CONTEXT_ITEMS = 15;

/**
 * Corta o contexto para caber no limite: primeiro as linhas, depois os filtros. Devolve
 * `null` se nem o essencial couber.
 */
export function fitContext<T extends object>(ctx: T): T | null {
  const c = { ...ctx } as T & { items?: unknown[]; filters?: unknown };
  if (Array.isArray(c.items) && c.items.length > MAX_CONTEXT_ITEMS) {
    c.items = c.items.slice(0, MAX_CONTEXT_ITEMS);
  }
  const size = () => JSON.stringify(c).length;
  while (size() > MAX_CONTEXT_CHARS && Array.isArray(c.items) && c.items.length > 0) {
    c.items = c.items.slice(0, Math.floor(c.items.length / 2));
    if (c.items.length === 0) delete c.items;
  }
  if (size() > MAX_CONTEXT_CHARS) delete c.filters;
  return size() > MAX_CONTEXT_CHARS ? null : c;
}
