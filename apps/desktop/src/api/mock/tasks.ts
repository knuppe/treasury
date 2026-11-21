/**
 * Mock de `tasks.*` (docs/background-tasks.md): o registro de tarefas em segundo plano,
 * em memória. Nada roda sozinho aqui; testes e prints criam e avançam tarefas com
 * `mockStartTask`, `mockTaskProgress` e `mockFinishTask`.
 */
import type { TaskInfo } from '@/api/generated/TaskInfo';
import type { TaskStatus } from '@/api/generated/TaskStatus';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { invalid, notFound } from './ledger/errors';

const tasks = new Map<string, TaskInfo>();
let seq = 0;

onMockReset(() => {
  tasks.clear();
  seq = 0;
});

export function mockStartTask(p: Partial<TaskInfo> & Pick<TaskInfo, 'kind'>): TaskInfo {
  seq += 1;
  const task: TaskInfo = {
    id: `task-${seq}`,
    origin: 'auto',
    status: 'running',
    stage: null,
    detail: null,
    done: 0,
    total: null,
    bytes: false,
    cancellable: true,
    cancelling: false,
    started_at: new Date().toISOString(),
    finished_at: null,
    error: null,
    ...p,
  };
  tasks.set(task.id, task);
  return structuredClone(task);
}

export function mockTaskProgress(id: string, patch: Partial<TaskInfo>): void {
  const task = tasks.get(id);
  if (task) Object.assign(task, patch);
}

export function mockFinishTask(id: string, status: TaskStatus, error: string | null = null): void {
  const task = tasks.get(id);
  if (!task) return;
  Object.assign(task, {
    status,
    error,
    cancelling: false,
    finished_at: new Date().toISOString(),
  });
}

registerMockHandlers({
  'tasks.list': (): TaskInfo[] =>
    [...tasks.values()]
      .map((x) => structuredClone(x))
      .sort(
        (a, b) =>
          Number(a.status !== 'running') - Number(b.status !== 'running') ||
          b.started_at.localeCompare(a.started_at),
      ),
  // No mock, o pedido de cancelar já é atendido (no núcleo, na próxima fronteira).
  'tasks.cancel': (a) => {
    const id = (a as { id?: string }).id ?? '';
    const task = tasks.get(id);
    if (!task) notFound('tarefa não encontrada (pode já ter terminado)');
    if (task.status !== 'running') return structuredClone(task);
    if (!task.cancellable) invalid('essa tarefa não pode ser cancelada');
    mockFinishTask(id, 'cancelled');
    return structuredClone(tasks.get(id)!);
  },
});
