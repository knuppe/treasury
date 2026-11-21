/**
 * Background tasks in the top bar: counter of the agent tasks in progress,
 * list of what each one does, and cancel.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { JobInfo } from '@/api/generated/JobInfo';
import type { TaskInfo } from '@/api/generated/TaskInfo';
import { mockCall } from '@/api/mock';
import { mockFinishTask, mockStartTask, mockTaskProgress } from '@/api/mock/tasks';
import { renderApp } from '@/test/renderApp';

import { since } from './BackgroundTasks';
import { taskCount, taskRatio } from './SystemTasks';

describe('Background tasks', () => {
  it('counts the agent tasks, shows what they do and cancels', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/painel');
    const idle = await screen.findByRole('button', { name: 'Tarefas em segundo plano' });
    await user.click(idle);
    expect(
      within(screen.getByRole('dialog', { name: 'Tarefas em segundo plano' })).getByText(
        /Nada rodando agora/,
      ),
    ).toBeTruthy();
    await user.keyboard('{Escape}');

    const started = (await mockCall('ai.chat_start', {
      message: 'Categorize os lançamentos de outubro',
    })) as { job_id: string };
    await queryClient.invalidateQueries();
    const busy = await screen.findByRole('button', {
      name: 'Tarefas em segundo plano (1 em andamento)',
    });
    await user.click(busy);
    const panel = screen.getByRole('dialog', { name: 'Tarefas em segundo plano' });
    const list = within(panel).getByRole('list', { name: 'Tarefas em andamento' });
    expect(within(list).getByText('Categorize os lançamentos de outubro')).toBeTruthy();
    await user.click(
      within(list).getByRole('button', { name: 'Cancelar Categorize os lançamentos de outubro' }),
    );
    expect(await screen.findByText('Tarefa cancelada')).toBeTruthy();
    await waitFor(async () => {
      const jobs = (await mockCall('ai.jobs.list', {})) as JobInfo[];
      expect(jobs.find((j) => j.job_id === started.job_id)?.status).toBe('cancelled');
    });
    const recent = await within(
      screen.getByRole('dialog', { name: 'Tarefas em segundo plano' }),
    ).findByRole('list', { name: 'Tarefas recentes' });
    expect(within(recent).getByText(/^Cancelada/)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Tarefas em segundo plano' })).toBeTruthy();
  });

  it('shows the collection at the top with the bar and the step, and cancels keeping what came in', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('button', { name: 'Tarefas em segundo plano' });
    const task = mockStartTask({
      kind: 'research_sync',
      stage: 'fundamentals',
      detail: 'CVM DFP 2025',
      done: 12,
    });
    await queryClient.invalidateQueries();
    // Chip at the top: with no total, the bar is indeterminate and there is no percentage.
    const chip = await screen.findByRole('button', {
      name: 'Fundamentos e score do mercado em andamento',
    });
    expect(within(chip).getByRole('progressbar').getAttribute('aria-valuenow')).toBeNull();
    mockTaskProgress(task.id, { stage: 'scores', detail: null, done: 30, total: 120 });
    await queryClient.invalidateQueries();
    const withPct = await screen.findByRole('button', {
      name: 'Fundamentos e score do mercado em andamento, 25%',
    });
    expect(
      screen.getByRole('button', { name: 'Tarefas em segundo plano (1 em andamento)' }),
    ).toBeTruthy();

    await user.click(withPct);
    const panel = screen.getByRole('dialog', { name: 'Tarefas em segundo plano' });
    const list = within(panel).getByRole('list', { name: 'Tarefas em andamento' });
    expect(within(list).getByText('Score do mercado')).toBeTruthy();
    expect(within(list).getByText('30 de 120')).toBeTruthy();
    expect(
      within(list)
        .getByRole('progressbar', { name: 'Progresso: Fundamentos e score do mercado' })
        .getAttribute('aria-valuenow'),
    ).toBe('25');
    await user.click(
      within(list).getByRole('button', { name: 'Cancelar Fundamentos e score do mercado' }),
    );
    expect(await screen.findByText(/o que já tinha vindo ficou gravado/)).toBeTruthy();
    await waitFor(async () => {
      const tasks = (await mockCall('tasks.list', {})) as TaskInfo[];
      expect(tasks.find((x) => x.id === task.id)?.status).toBe('cancelled');
    });
    // Finished: leaves the top and goes to the recent ones.
    await waitFor(() =>
      expect(
        screen.queryByRole('button', { name: /Fundamentos e score do mercado em andamento/ }),
      ).toBeNull(),
    );
    const recent = await within(
      screen.getByRole('dialog', { name: 'Tarefas em segundo plano' }),
    ).findByRole('list', { name: 'Tarefas recentes' });
    expect(within(recent).getByText('Fundamentos e score do mercado')).toBeTruthy();
  });

  it('a failure shows the reason and the update download has no cancel', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('button', { name: 'Tarefas em segundo plano' });
    const failed = mockStartTask({ kind: 'market_sync' });
    mockFinishTask(failed.id, 'failed', 'Yahoo: tempo esgotado');
    mockStartTask({
      kind: 'update_download',
      stage: 'download',
      bytes: true,
      done: 2_500_000,
      total: 8_000_000,
      cancellable: false,
    });
    await queryClient.invalidateQueries();
    await user.click(
      await screen.findByRole('button', { name: 'Tarefas em segundo plano (1 em andamento)' }),
    );
    const panel = screen.getByRole('dialog', { name: 'Tarefas em segundo plano' });
    const list = within(panel).getByRole('list', { name: 'Tarefas em andamento' });
    expect(within(list).getByText('2,5 MB de 8 MB')).toBeTruthy();
    expect(within(list).queryByRole('button', { name: /^Cancelar/ })).toBeNull();
    const recent = within(panel).getByRole('list', { name: 'Tarefas recentes' });
    expect(within(recent).getByText('Yahoo: tempo esgotado')).toBeTruthy();
  });

  it('counts progress only with a known total', () => {
    const base = mockStartTask({ kind: 'market_sync' });
    expect(taskRatio({ ...base, done: 3 })).toBeNull();
    expect(taskCount({ ...base, done: 3 })).toBe('3 concluídos');
    expect(taskCount({ ...base, done: 0 })).toBeNull();
    expect(taskRatio({ ...base, done: 3, total: 4 })).toBe(0.75);
    expect(taskCount({ ...base, done: 3, total: 4 })).toBe('3 de 4');
  });

  it('writes how long ago', () => {
    const t = Date.parse('2026-10-17T12:00:00Z');
    expect(since('2026-10-17T12:00:00Z', t + 2_000)).toBe('agora');
    expect(since('2026-10-17T12:00:00Z', t + 42_000)).toBe('há 42 s');
    expect(since('2026-10-17T12:00:00Z', t + 3 * 60_000)).toBe('há 3 min');
    expect(since('2026-10-17T12:00:00Z', t + 65 * 60_000)).toBe('há 1 h 05');
  });
});
