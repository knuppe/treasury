/**
 * Tarefas em segundo plano (barra superior): ícone com o número de tarefas em andamento, do
 * agente (conversas, rotinas como o fechamento do mês e os alertas) e do núcleo (coletas de
 * índices, cotações e fundamentos, e-mail, download da atualização; docs/background-tasks.md).
 * Com uma coleta rodando, um chip ao lado mostra o nome e a barra de progresso. O clique abre
 * a lista com o que cada uma está fazendo agora, há quanto tempo, o progresso e os botões de
 * abrir e de cancelar; abaixo, as que terminaram nos últimos 30 minutos.
 */
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { createPortal } from 'react-dom';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router';

import type { JobInfo } from '@/api/generated/JobInfo';
import type { JobLink } from '@/api/generated/JobLink';
import type { JobStatus } from '@/api/generated/JobStatus';
import type { TaskInfo } from '@/api/generated/TaskInfo';
import { useCancelJob, useJobs } from '@/api/hooks/ai';
import { useTasks } from '@/api/hooks/tasks';
import { Button, Icon, Tooltip, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { TASK_LABEL } from '@/features/agent/format';
import { errorText } from '@/features/transactions/ui/errors';
import { t } from '@/i18n';
import { useActivity, type ActivityStatus } from '@/state/activity';

import { SystemTaskChip, SystemTaskRow, taskTitle } from './SystemTasks';

const statusLabel = (s: JobStatus): string =>
  s === 'running'
    ? t('casca-tarefa-andamento')
    : s === 'done'
      ? t('casca-tarefa-concluida')
      : s === 'failed'
        ? t('casca-tarefa-falhou')
        : t('casca-tarefa-cancelada');

/** "agora", "há 42 s", "há 3 min", "há 1 h 05". */
export function since(iso: string, now: number): string {
  const s = Math.max(0, Math.round((now - Date.parse(iso)) / 1000));
  if (s < 5) return t('casca-agora');
  if (s < 60) return t('casca-ha-segundos', { s });
  const m = Math.floor(s / 60);
  if (m < 60) return t('casca-ha-minutos', { m });
  return t('casca-ha-horas', { h: Math.floor(m / 60), m: String(m % 60).padStart(2, '0') });
}

/** Caminho do resultado de um job (o estudo com IA aberto na aba do ativo). */
export function jobLinkHref(l: JobLink): string | null {
  switch (l.screen) {
    case 'asset_study':
      return l.asset_id
        ? `/ativos?ativo=${encodeURIComponent(l.asset_id)}&secao=estudo${l.id ? `&estudo=${encodeURIComponent(l.id)}` : ''}`
        : null;
  }
}

/**
 * Estudo com IA que terminou (mesmo com a aba do ativo fechada): o histórico é atualizado e
 * um aviso oferece abrir o estudo. Cada job avisa uma vez.
 */
/** Estudos que terminaram desde a última leitura (e os marca como vistos). */
export function newlyDoneStudies(seen: Set<string>, jobs: readonly JobInfo[]): JobInfo[] {
  const out = jobs.filter((j) => j.task === 'study' && j.status === 'done' && !seen.has(j.job_id));
  for (const j of out) seen.add(j.job_id);
  return out;
}

function useStudyDoneNotice(jobs: readonly JobInfo[]) {
  const client = useQueryClient();
  const navigate = useNavigate();
  const seen = useRef<Set<string> | null>(null);
  useEffect(() => {
    const done = jobs.filter((j) => j.task === 'study' && j.status === 'done');
    // Na primeira leitura, os que já tinham terminado não avisam de novo.
    if (seen.current == null) {
      seen.current = new Set(done.map((j) => j.job_id));
      return;
    }
    for (const j of newlyDoneStudies(seen.current, done)) {
      void client.invalidateQueries({
        predicate: (q) =>
          q.queryKey[0] === 'assets' &&
          typeof q.queryKey[1] === 'string' &&
          q.queryKey[1].startsWith('study'),
      });
      const href = j.link ? jobLinkHref(j.link) : null;
      toast({
        tone: 'success',
        title: t('casca-estudo-pronto'),
        ...(href
          ? { action: { label: t('casca-abrir'), onClick: () => void navigate(href) } }
          : {}),
      });
    }
  }, [jobs, client, navigate]);
}

function title(j: JobInfo): string {
  return j.title ?? TASK_LABEL[j.task];
}

function TaskRow({ job, now, onOpen }: { job: JobInfo; now: number; onOpen?: () => void }) {
  const cancel = useCancelJob();
  const running = job.status === 'running';
  const meta = [
    job.kind === 'routine' ? t('casca-rotina') : TASK_LABEL[job.task],
    job.engine_name,
    running ? since(job.created_at, now) : null,
  ]
    .filter(Boolean)
    .join(' · ');
  return (
    <li className="flex flex-col gap-1 rounded-control px-2 py-2 hover:bg-surface-2">
      <div className="flex items-start gap-2">
        <span
          aria-hidden
          className={cx(
            'mt-1.5 size-1.5 shrink-0 rounded-full',
            running
              ? 'animate-pulse bg-accent'
              : job.status === 'failed'
                ? 'bg-negative'
                : 'bg-line-strong',
          )}
        />
        <span className="flex min-w-0 flex-1 flex-col">
          <span className="truncate text-[13px] text-fg" title={title(job)}>
            {title(job)}
          </span>
          <span className="truncate text-[11.5px] text-dim">
            {running
              ? meta
              : `${statusLabel(job.status)}${job.finished_at ? ` ${since(job.finished_at, now)}` : ''}`}
          </span>
          {running && job.activity ? (
            <span className="truncate text-[12px] text-fg/80" title={job.activity}>
              {job.cancelling ? t('casca-cancelando') : job.activity}
            </span>
          ) : null}
        </span>
      </div>
      {running || onOpen ? (
        <div className="flex justify-end gap-1">
          {onOpen ? (
            <Button size="sm" variant="ghost" onClick={onOpen}>
              {t('casca-abrir')}
            </Button>
          ) : null}
          {running ? (
            <Button
              size="sm"
              variant="ghost"
              aria-label={t('casca-tarefa-cancelar', { tarefa: title(job) })}
              loading={cancel.isPending}
              disabled={job.cancelling}
              onClick={() =>
                cancel.mutate(job.job_id, {
                  onSuccess: () =>
                    toast({ title: t('casca-tarefa-cancelada-toast'), description: title(job) }),
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('casca-tarefa-nao-cancelou'),
                      description: errorText(e),
                    }),
                })
              }
            >
              {job.cancelling ? t('casca-cancelando') : t('comum-cancelar')}
            </Button>
          ) : null}
        </div>
      ) : null}
    </li>
  );
}

/** Tarefas do agente e do núcleo em andamento (o contador, o scanner e a peça dos temas). */
const endStatus = (s: JobStatus | TaskInfo['status']): ActivityStatus =>
  s === 'failed' ? 'failed' : s === 'cancelled' ? 'cancelled' : 'done';

/**
 * Ponte para a atividade do app (state/activity.ts): os jobs do agente e as tarefas do núcleo
 * em andamento entram nela, e saem com a situação em que terminaram.
 */
export function ActivityBridge() {
  const { data: jobs = [] } = useJobs({ poll: true });
  const { data: tasks = [] } = useTasks();
  const start = useActivity((s) => s.start);
  const end = useActivity((s) => s.end);
  const seen = useRef(new Set<string>());
  useEffect(() => {
    const running = new Set<string>();
    const finished = new Map<string, ActivityStatus>();
    for (const j of jobs) {
      const id = `job:${j.job_id}`;
      if (j.status === 'running') {
        running.add(id);
        start(id, title(j));
      } else finished.set(id, endStatus(j.status));
    }
    for (const x of tasks) {
      const id = `task:${x.id}`;
      if (x.status === 'running') {
        running.add(id);
        start(id, taskTitle(x));
      } else finished.set(id, endStatus(x.status));
    }
    for (const id of seen.current) if (!running.has(id)) end(id, finished.get(id) ?? 'done');
    seen.current = running;
  }, [jobs, tasks, start, end]);
  // Ao bloquear o cofre, nada fica "rodando".
  useEffect(() => () => seen.current.forEach((id) => end(id, 'cancelled')), [end]);
  return null;
}

/** Hora em que terminou (ou começou, se ainda roda), para misturar as duas listas. */
const jobAt = (j: JobInfo) => j.finished_at ?? j.created_at;
const taskAt = (x: TaskInfo) => x.finished_at ?? x.started_at;

export function BackgroundTasks() {
  const { data: jobs = [] } = useJobs({ poll: true });
  const { data: tasks = [] } = useTasks();
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [now, setNow] = useState(() => Date.now());
  const button = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; right: number } | null>(null);
  const running = jobs.filter((j) => j.status === 'running');
  const runningTasks = tasks.filter((x) => x.status === 'running');
  // As 5 que terminaram por último, do agente e do núcleo juntas.
  const recent = [
    ...jobs.filter((j) => j.status !== 'running').map((j) => ({ at: jobAt(j), job: j })),
    ...tasks.filter((x) => x.status !== 'running').map((x) => ({ at: taskAt(x), task: x })),
  ]
    .sort((a, b) => b.at.localeCompare(a.at))
    .slice(0, 5);
  useStudyDoneNotice(jobs);

  useLayoutEffect(() => {
    if (!open) return;
    const place = () => {
      const r = button.current?.getBoundingClientRect();
      if (r) setPos({ top: r.bottom + 6, right: Math.max(8, window.innerWidth - r.right) });
    };
    place();
    window.addEventListener('resize', place);
    return () => window.removeEventListener('resize', place);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    // Relógio do "há N s" enquanto a lista está aberta.
    const t = setInterval(() => setNow(Date.now()), 1000);
    const onDown = (e: MouseEvent) => {
      const el = e.target as Node;
      if (!button.current?.contains(el) && !panel.current?.contains(el)) setOpen(false);
    };
    // Esc fecha mesmo com o foco ainda no botão.
    const onEsc = (e: globalThis.KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    document.addEventListener('mousedown', onDown);
    document.addEventListener('keydown', onEsc);
    return () => {
      clearInterval(t);
      document.removeEventListener('mousedown', onDown);
      document.removeEventListener('keydown', onEsc);
    };
  }, [open]);

  const close = () => {
    setOpen(false);
    button.current?.focus();
  };
  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      close();
    }
  };
  const openConversation = (j: JobInfo) => {
    const href = j.link ? jobLinkHref(j.link) : null;
    if (href)
      return () => {
        close();
        void navigate(href);
      };
    return openChat(j);
  };
  const openChat = (j: JobInfo) =>
    j.conversation_id
      ? () => {
          close();
          void navigate(`/agente?aba=conversa&conversa=${encodeURIComponent(j.conversation_id!)}`);
        }
      : undefined;

  const n = running.length + runningTasks.length;
  const ago = (iso: string) => since(iso, now);
  const label = n === 0 ? t('casca-tarefas') : t('casca-tarefas-andamento', { n });
  return (
    <>
      <SystemTaskChip
        tasks={runningTasks}
        open={open}
        onToggle={() => {
          setNow(Date.now());
          setOpen((o) => !o);
        }}
      />
      <Tooltip content={n === 0 ? t('casca-tarefas-nenhuma') : t('casca-tarefas-n', { n })}>
        <button
          ref={button}
          type="button"
          aria-label={label}
          aria-haspopup="dialog"
          aria-expanded={open}
          onClick={() => {
            setNow(Date.now());
            setOpen((o) => !o);
          }}
          className={cx(
            'relative inline-flex size-8 items-center justify-center rounded-control transition-colors duration-150 hover:bg-surface-2 hover:text-fg',
            n > 0 ? 'text-accent' : 'text-dim',
            open && 'bg-accent-soft text-accent hover:bg-accent-soft hover:text-accent',
          )}
        >
          <Icon name="activity" />
          {n > 0 ? (
            <span
              aria-hidden
              className="num absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-accent px-1 text-[10px] leading-none text-accent-ink"
            >
              {n}
            </span>
          ) : null}
        </button>
      </Tooltip>
      {open
        ? createPortal(
            <div
              ref={panel}
              role="dialog"
              aria-label={t('casca-tarefas')}
              onKeyDown={onKey}
              style={
                pos
                  ? { position: 'fixed', top: pos.top, right: pos.right }
                  : { position: 'fixed', top: 0, right: 0, opacity: 0 }
              }
              className="z-[85] flex max-h-[calc(100vh-64px)] w-80 animate-pop-in flex-col overflow-y-auto rounded-card border border-panel pop-surface p-1.5"
            >
              <p className="label-caps px-2 pt-1 pb-1 text-[10px]">{t('casca-tarefa-andamento')}</p>
              {n === 0 ? (
                <p className="px-2 pb-2 text-[12.5px] text-dim">{t('casca-tarefas-vazio')}</p>
              ) : (
                <ul aria-label={t('casca-tarefas-em-andamento')} className="flex flex-col">
                  {runningTasks.map((x) => (
                    <SystemTaskRow key={x.id} task={x} since={ago} />
                  ))}
                  {running.map((j) => (
                    <TaskRow key={j.job_id} job={j} now={now} onOpen={openConversation(j)} />
                  ))}
                </ul>
              )}
              {recent.length > 0 ? (
                <>
                  <p className="label-caps border-t border-line px-2 pt-2 pb-1 text-[10px]">
                    {t('casca-tarefas-ultimos')}
                  </p>
                  <ul aria-label={t('casca-tarefas-recentes')} className="flex flex-col">
                    {recent.map((r) =>
                      'task' in r ? (
                        <SystemTaskRow key={r.task.id} task={r.task} since={ago} />
                      ) : (
                        <TaskRow
                          key={r.job.job_id}
                          job={r.job}
                          now={now}
                          onOpen={openConversation(r.job)}
                        />
                      ),
                    )}
                  </ul>
                </>
              ) : null}
            </div>,
            document.body,
          )
        : null}
    </>
  );
}
