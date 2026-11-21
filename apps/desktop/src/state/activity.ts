/**
 * Atividade do app: a fonte única do que está rodando agora (docs/design.md, "Atividade").
 * Os temas reagem só a este estado: o scanner do tema Scanner e as peças da Órbita, do
 * Neon e do Lo-fi ligam com alguma tarefa; no Fósforo, a chuva acelera, o chip
 * `▮ PROCESSANDO_` aparece e o card pisca ao terminar.
 *
 * Quem registra:
 * - as tarefas do agente (jobs) e do núcleo (coletas, e-mail, atualização), pela ponte em
 *   app/BackgroundTasks.tsx;
 * - os comandos demorados chamados pela interface (importação, cupons, cenários, cotações),
 *   direto em `call()` (api/client.ts, `ACTIVITY_COMMANDS`);
 * - qualquer outra parte do app, com `start(id, rótulo)` / `end(id, situação)` ou o hook
 *   `useActivityTask`.
 *
 * Só memória: nada persiste.
 */
import { useEffect, useRef, type RefObject } from 'react';
import { create } from 'zustand';

export type ActivityStatus = 'done' | 'failed' | 'cancelled';

export interface ActivityTask {
  id: string;
  label: string;
  startedAt: number;
}

export interface ActivityEnd {
  id: string;
  label: string;
  status: ActivityStatus;
  at: number;
}

interface ActivityState {
  tasks: Record<string, ActivityTask>;
  /** A última que terminou (o flash e o `▮ FALHA_` do Fósforo). */
  last: ActivityEnd | null;
  start: (id: string, label: string) => void;
  end: (id: string, status?: ActivityStatus) => void;
}

export const useActivity = create<ActivityState>()((set, get) => ({
  tasks: {},
  last: null,
  start: (id, label) => {
    const known = get().tasks[id];
    if (known && known.label === label) return;
    set((s) => ({
      tasks: { ...s.tasks, [id]: { id, label, startedAt: known?.startedAt ?? Date.now() } },
    }));
  },
  end: (id, status = 'done') => {
    const task = get().tasks[id];
    if (!task) return;
    set((s) => {
      const tasks = { ...s.tasks };
      delete tasks[id];
      return { tasks, last: { id, label: task.label, status, at: Date.now() } };
    });
  },
}));

/** Começa uma tarefa fora do React (ex.: `call()`); devolve a função que a encerra. */
export function startActivity(id: string, label: string): (status?: ActivityStatus) => void {
  useActivity.getState().start(id, label);
  return (status) => useActivity.getState().end(id, status);
}

/** Tarefas em andamento, da mais antiga para a mais nova. */
export function activeTasks(tasks: Record<string, ActivityTask>): ActivityTask[] {
  return Object.values(tasks).sort((a, b) => a.startedAt - b.startedAt);
}

export function useActiveTasks(): ActivityTask[] {
  const tasks = useActivity((s) => s.tasks);
  return activeTasks(tasks);
}

/** Alguma tarefa em andamento. */
export function useBusy(): boolean {
  return useActivity((s) => Object.keys(s.tasks).length > 0);
}

const FLASH_MS = 300;

/**
 * Registra uma tarefa enquanto `running` for verdadeiro e, ao terminar, pisca a borda do
 * elemento de `ref` (classes `activity-flash-ok` e `activity-flash-error`; cada tema decide
 * se mostra, hoje só o Fósforo). `failed` diz como terminou.
 */
export function useActivityTask(
  id: string,
  label: string,
  running: boolean,
  failed = false,
): RefObject<HTMLElement | null> {
  const ref = useRef<HTMLElement | null>(null);
  const was = useRef(false);
  const start = useActivity((s) => s.start);
  const end = useActivity((s) => s.end);
  useEffect(() => {
    if (running) {
      start(id, label);
      was.current = true;
      return;
    }
    if (!was.current) return;
    was.current = false;
    end(id, failed ? 'failed' : 'done');
    flash(ref.current, failed);
  }, [id, label, running, failed, start, end]);
  // Saiu da tela no meio: a tarefa não fica presa como "rodando".
  useEffect(() => () => end(id, 'cancelled'), [id, end]);
  return ref;
}

/**
 * Só o flash: para o que já entra na atividade por outro caminho (os comandos de
 * `ACTIVITY_COMMANDS` em api/client.ts). Pisca o elemento de `ref` quando `running` volta a
 * falso.
 */
export function useSettleFlash<T extends HTMLElement>(
  running: boolean,
  failed = false,
): RefObject<T | null> {
  const ref = useRef<T | null>(null);
  const was = useRef(running);
  useEffect(() => {
    if (was.current && !running) flash(ref.current, failed);
    was.current = running;
  }, [running, failed]);
  return ref;
}

/** Pisca a borda do elemento (verde ou vermelha no Fósforo). */
export function flash(el: Element | null | undefined, failed = false): void {
  if (!el) return;
  const cls = failed ? 'activity-flash-error' : 'activity-flash-ok';
  el.classList.remove('activity-flash-ok', 'activity-flash-error');
  // Força o reinício da animação quando a mesma classe volta logo em seguida.
  void (el as HTMLElement).offsetWidth;
  el.classList.add(cls);
  window.setTimeout(() => el.classList.remove(cls), FLASH_MS + 50);
}
