/**
 * Coletas e downloads do núcleo nas tarefas em segundo plano (docs/background-tasks.md):
 * a linha da lista (etapa, o que está sendo feito, barra de progresso e cancelar) e o chip
 * da barra superior, que aparece só enquanto alguma roda.
 */
import type { TaskInfo } from '@/api/generated/TaskInfo';
import type { TaskKind } from '@/api/generated/TaskKind';
import type { TaskStage } from '@/api/generated/TaskStage';
import { useCancelTask } from '@/api/hooks/tasks';
import { Button, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { formatBytes } from '@/features/research/format';
import { errorText } from '@/features/transactions/ui/errors';
import type { PlainKey as TKey } from '@/features/agent/format';
import { t } from '@/i18n';
import { formatNumber, formatPercent } from '@/lib/format';

const KIND: Record<TaskKind, TKey> = {
  market_sync: 'casca-coleta-market-sync',
  indices_sync: 'casca-coleta-indices-sync',
  research_sync: 'casca-coleta-research-sync',
  mail_sync: 'casca-coleta-mail-sync',
  update_download: 'casca-coleta-update-download',
};

/** Nome curto, para o chip da barra superior. */
const SHORT: Record<TaskKind, TKey> = {
  market_sync: 'casca-coleta-curta-market-sync',
  indices_sync: 'casca-coleta-curta-indices-sync',
  research_sync: 'casca-coleta-curta-research-sync',
  mail_sync: 'casca-coleta-curta-mail-sync',
  update_download: 'casca-coleta-curta-update-download',
};

const STAGE: Record<TaskStage, TKey> = {
  indices: 'casca-etapa-indices',
  prices: 'casca-etapa-prices',
  funds: 'casca-etapa-funds',
  income: 'casca-etapa-income',
  fundamentals: 'casca-etapa-fundamentals',
  scores: 'casca-etapa-scores',
  mail: 'casca-etapa-mail',
  download: 'casca-etapa-download',
};

export const taskTitle = (x: TaskInfo): string => t(KIND[x.kind]);

/** Fração concluída, quando o total é conhecido. */
export function taskRatio(x: TaskInfo): number | null {
  if (x.total == null || x.total <= 0) return null;
  return Math.min(1, x.done / x.total);
}

/** "12 de 48", "3,1 MB de 40,0 MB", "7 feitos" ou nada. */
export function taskCount(x: TaskInfo): string | null {
  if (x.bytes) {
    return x.total != null
      ? t('casca-tarefa-de', { feito: formatBytes(x.done), total: formatBytes(x.total) })
      : formatBytes(x.done);
  }
  if (x.total != null)
    return t('casca-tarefa-de', { feito: formatNumber(x.done), total: formatNumber(x.total) });
  return x.done > 0 ? t('casca-tarefa-feitos', { n: x.done }) : null;
}

/** Barra fina: determinada com o total; sem ele, uma faixa que corre. */
export function TaskBar({ task, className }: { task: TaskInfo; className?: string }) {
  const ratio = taskRatio(task);
  return (
    <div
      role="progressbar"
      aria-label={t('casca-tarefa-progresso', { tarefa: taskTitle(task) })}
      aria-valuemin={0}
      aria-valuemax={100}
      {...(ratio != null
        ? {
            'aria-valuenow': Math.round(ratio * 100),
            'aria-valuetext': formatPercent(ratio, { decimals: 0 }),
          }
        : {})}
      className={cx('relative h-1 overflow-hidden rounded-full bg-surface-2', className)}
    >
      {ratio != null ? (
        <div
          className="h-full rounded-full bg-accent transition-[width] duration-300"
          style={{ width: `${ratio * 100}%` }}
        />
      ) : (
        <div className="task-indeterminate absolute inset-y-0 w-1/3 rounded-full bg-accent" />
      )}
    </div>
  );
}

const STATUS: Record<TaskInfo['status'], TKey> = {
  running: 'casca-tarefa-andamento',
  done: 'casca-tarefa-concluida',
  failed: 'casca-tarefa-falhou',
  cancelled: 'casca-tarefa-cancelada',
};

export function SystemTaskRow({ task, since }: { task: TaskInfo; since: (iso: string) => string }) {
  const cancel = useCancelTask();
  const running = task.status === 'running';
  const title = taskTitle(task);
  const count = taskCount(task);
  const stage = task.stage ? t(STAGE[task.stage]) : null;
  return (
    <li className="flex flex-col gap-1 rounded-control px-2 py-2 hover:bg-surface-2">
      <div className="flex items-start gap-2">
        <span
          aria-hidden
          className={cx(
            'mt-1.5 size-1.5 shrink-0 rounded-full',
            running
              ? 'animate-pulse bg-accent'
              : task.status === 'failed'
                ? 'bg-negative'
                : 'bg-line-strong',
          )}
        />
        <span className="flex min-w-0 flex-1 flex-col gap-0.5">
          <span className="truncate text-[13px] text-fg" title={title}>
            {title}
          </span>
          <span className="truncate text-[11.5px] text-dim">
            {running
              ? [
                  task.origin === 'auto' ? t('casca-tarefa-automatica') : null,
                  since(task.started_at),
                ]
                  .filter(Boolean)
                  .join(' · ')
              : `${t(STATUS[task.status])}${task.finished_at ? ` ${since(task.finished_at)}` : ''}`}
          </span>
          {running ? (
            <>
              <span className="truncate text-[12px] text-fg/80" title={task.detail ?? undefined}>
                {task.cancelling
                  ? t('casca-cancelando')
                  : [stage, task.detail].filter(Boolean).join(': ')}
              </span>
              <TaskBar task={task} className="mt-1" />
              {count ? <span className="num text-[11px] text-dim">{count}</span> : null}
            </>
          ) : task.error ? (
            <span
              className={cx(
                'line-clamp-2 text-[12px]',
                task.status === 'failed' ? 'text-negative' : 'text-dim',
              )}
              title={task.error}
            >
              {task.error}
            </span>
          ) : null}
        </span>
      </div>
      {running && task.cancellable ? (
        <div className="flex justify-end">
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('casca-tarefa-cancelar', { tarefa: title })}
            loading={cancel.isPending}
            disabled={task.cancelling}
            onClick={() =>
              cancel.mutate(task.id, {
                onSuccess: () =>
                  toast({
                    title: t('casca-tarefa-cancelada-toast'),
                    description: t('casca-coleta-cancelada-dica', { tarefa: title }),
                  }),
                onError: (e) =>
                  toast({
                    tone: 'error',
                    title: t('casca-tarefa-nao-cancelou'),
                    description: errorText(e),
                  }),
              })
            }
          >
            {task.cancelling ? t('casca-cancelando') : t('comum-cancelar')}
          </Button>
        </div>
      ) : null}
    </li>
  );
}

/**
 * Chip da barra superior com a coleta em andamento: nome, quanto falta e a barra fina. O
 * clique abre a lista (com o cancelar). Com várias, mostra a primeira e "+N".
 */
export function SystemTaskChip({
  tasks,
  open,
  onToggle,
}: {
  tasks: readonly TaskInfo[];
  open: boolean;
  onToggle: () => void;
}) {
  const first = tasks[0];
  if (!first) return null;
  const ratio = taskRatio(first);
  const title = taskTitle(first);
  const short = t(SHORT[first.kind]);
  const more = tasks.length - 1;
  const label =
    ratio != null
      ? t('casca-coleta-chip-pct', { tarefa: title, pct: formatPercent(ratio, { decimals: 0 }) })
      : t('casca-coleta-chip', { tarefa: title });
  return (
    <button
      type="button"
      onClick={onToggle}
      aria-expanded={open}
      aria-haspopup="dialog"
      aria-label={more > 0 ? `${label} ${t('casca-coleta-chip-mais', { n: more })}` : label}
      className="mr-0.5 hidden h-8 w-32 flex-col justify-center gap-1 rounded-control border border-line px-2 text-left transition-colors duration-150 hover:border-line-strong md:flex"
    >
      <span className="flex items-baseline gap-1.5 text-[11.5px] leading-none">
        <span className="truncate text-fg">{short}</span>
        <span className="num ml-auto shrink-0 text-dim">
          {ratio != null ? formatPercent(ratio, { decimals: 0 }) : null}
          {more > 0 ? ` +${more}` : null}
        </span>
      </span>
      <TaskBar task={first} />
    </button>
  );
}
