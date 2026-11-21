/**
 * Quadro do feedback: uma coluna por status (Aberto, Em andamento, Resolvido, Descartado),
 * cards compactos na ordem da fila (a manual primeiro, depois a prioridade) e o painel lateral
 * com o item inteiro em seções. Arrastar um card muda o status ou o lugar na coluna; cada
 * movimento é um conjunto de alterações, com "Desfazer" no aviso.
 */
import * as RDialog from '@radix-ui/react-dialog';
import { useState, type DragEvent, type ReactNode } from 'react';

import type { DevFeedback } from '@/api/generated/DevFeedback';
import type { DevFeedbackStatus } from '@/api/generated/DevFeedbackStatus';
import {
  useSetDevFeedbackPriority,
  useSetDevFeedbackStatus,
  type DevFeedbackWritten,
} from '@/api/hooks/devfeedback';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, Icon, IconButton, Markdown, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { Pill, type PillTone } from '@/features/agent/ui';
import { errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDate, formatDateTime } from '@/lib/format';

import {
  AREA_LABEL,
  authorLabel,
  taskMarkdown,
  KIND_LABEL,
  linkLabel,
  priorityLabel,
  sections,
  STATUS_LABEL,
  STATUSES,
} from './format';

export const STATUS_TONE: Record<DevFeedbackStatus, PillTone> = {
  aberto: 'warning',
  em_andamento: 'accent',
  resolvido: 'positive',
  descartado: 'dim',
};

export function copyText(text: string, title: string) {
  void navigator.clipboard?.writeText(text).then(
    () => toast({ tone: 'success', title }),
    () => toast({ tone: 'error', title: t('desenvolvedor-erro-copiar') }),
  );
}

/** Data curta do card: o último reforço ou, sem reforço, o registro. */
function cardDate(f: DevFeedback): string {
  return formatDate((f.last_bumped_at ?? f.created_at).slice(0, 10));
}

const DRAG_TYPE = 'application/x-treasury-feedback';

/** Mover (status e/ou lugar) com o aviso de desfazer. */
function useMove() {
  const setStatus = useSetDevFeedbackStatus();
  const setPriority = useSetDevFeedbackPriority();
  const undoToast = useUndoToast();
  const done = (r: DevFeedbackWritten, title: string) =>
    undoToast({ title, description: r.title, changeSetId: r.change_set_id });
  const fail = (e: unknown) =>
    toast({ tone: 'error', title: t('desenvolvedor-erro-status'), description: errorMessage(e) });
  return {
    pending: setStatus.isPending || setPriority.isPending,
    /** Para a coluna `status`, no lugar `position` (1 = topo; sem ele, pela prioridade). */
    move: (f: DevFeedback, status: DevFeedbackStatus, position?: number) => {
      if (status === f.status) {
        if (position === undefined || position === f.position) return;
        setPriority.mutate(
          { id: f.id, position },
          { onSuccess: (r) => done(r, t('desenvolvedor-reordenado')), onError: fail },
        );
        return;
      }
      setStatus.mutate(
        { id: f.id, status, ...(position !== undefined ? { position } : {}) },
        {
          onSuccess: (r) => done(r, t('desenvolvedor-status-mudou', { status })),
          onError: fail,
        },
      );
    },
  };
}

function FeedbackCard({
  f,
  top,
  onOpen,
  onDropBefore,
}: {
  f: DevFeedback;
  /** O próximo da fila (topo de Em andamento ou, sem nada lá, de Aberto). */
  top: boolean;
  onOpen: () => void;
  onDropBefore: (id: string) => void;
}) {
  const [over, setOver] = useState(false);
  const closed = f.status === 'resolvido' || f.status === 'descartado';
  return (
    <li
      aria-label={t('desenvolvedor-feedback-rotulo', { titulo: f.title })}
      draggable
      onDragStart={(e) => {
        e.dataTransfer.setData(DRAG_TYPE, f.id);
        e.dataTransfer.setData('text/plain', f.title);
        e.dataTransfer.effectAllowed = 'move';
      }}
      onDragOver={(e) => {
        if (!e.dataTransfer.types.includes(DRAG_TYPE)) return;
        e.preventDefault();
        e.stopPropagation();
        setOver(true);
      }}
      onDragLeave={() => setOver(false)}
      onDrop={(e) => {
        const id = e.dataTransfer.getData(DRAG_TYPE);
        setOver(false);
        if (!id || id === f.id) return;
        e.preventDefault();
        e.stopPropagation();
        onDropBefore(id);
      }}
      className={cx(
        'rounded-control border bg-bg transition-colors',
        over ? 'border-accent' : top ? 'border-accent/60' : 'border-line',
      )}
    >
      <button
        type="button"
        onClick={onOpen}
        className="flex w-full cursor-grab flex-col gap-2 px-3 py-2.5 text-left active:cursor-grabbing"
      >
        {top ? (
          <span className="label-caps text-[10px] text-accent">{t('desenvolvedor-proximo')}</span>
        ) : null}
        <span
          className={cx(
            'line-clamp-3 text-[13px] leading-snug font-medium',
            closed ? 'text-dim line-through decoration-line-strong' : 'text-fg',
          )}
        >
          {f.title}
        </span>
        <span className="flex flex-wrap items-center gap-1">
          <Pill tone="accent">{AREA_LABEL[f.area]}</Pill>
          <Pill>{KIND_LABEL[f.kind]}</Pill>
          {f.assignee ? <Pill>{f.assignee}</Pill> : null}
        </span>
        <span className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11.5px] text-dim">
          <span
            className={cx('num font-semibold', closed ? 'text-dim' : 'text-fg')}
            title={t('desenvolvedor-importancia-dica')}
          >
            {priorityLabel(f)}
          </span>
          {f.bumps > 0 ? <span>{t('desenvolvedor-n-reforcos', { n: f.bumps })}</span> : null}
          <span className="num ml-auto">{cardDate(f)}</span>
        </span>
      </button>
    </li>
  );
}

function Column({
  status,
  items,
  nextId,
  onOpen,
  onMove,
}: {
  status: DevFeedbackStatus;
  items: DevFeedback[];
  nextId: string | undefined;
  onOpen: (f: DevFeedback) => void;
  onMove: (id: string, status: DevFeedbackStatus, position?: number) => void;
}) {
  const [over, setOver] = useState(false);
  const label = STATUS_LABEL[status];
  const onDragOver = (e: DragEvent) => {
    if (!e.dataTransfer.types.includes(DRAG_TYPE)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setOver(true);
  };
  return (
    <section
      aria-label={t('desenvolvedor-coluna', { status: label, n: items.length })}
      onDragOver={onDragOver}
      onDragLeave={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setOver(false);
      }}
      onDrop={(e) => {
        const id = e.dataTransfer.getData(DRAG_TYPE);
        setOver(false);
        if (!id) return;
        e.preventDefault();
        // Solto no fim da coluna: na mesma coluna vai para o fim; noutra, entra pela prioridade.
        const mine = items.some((f) => f.id === id);
        onMove(id, status, mine ? items.length : undefined);
      }}
      className={cx(
        'flex min-h-[220px] min-w-[240px] flex-1 flex-col rounded-card border bg-panel/40',
        over ? 'border-accent' : 'border-line',
      )}
    >
      <header className="flex items-center gap-2 border-b border-line px-3 py-2">
        <Pill tone={STATUS_TONE[status]}>{label}</Pill>
        <span className="num text-[12px] text-dim">{items.length}</span>
      </header>
      <ol
        aria-label={label}
        className="flex max-h-[68vh] min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-2"
      >
        {items.map((f, i) => (
          <FeedbackCard
            key={f.id}
            f={f}
            top={f.id === nextId}
            onOpen={() => onOpen(f)}
            onDropBefore={(id) => onMove(id, status, i + 1)}
          />
        ))}
        {items.length === 0 ? (
          <li className="px-1 py-6 text-center text-[12px] text-dim">
            {t('desenvolvedor-coluna-vazia')}
          </li>
        ) : null}
      </ol>
    </section>
  );
}

export function DevFeedbackBoard({
  items,
  nextId,
  onOpen,
}: {
  items: DevFeedback[];
  nextId: string | undefined;
  onOpen: (f: DevFeedback) => void;
}) {
  const { move } = useMove();
  const byStatus = (s: DevFeedbackStatus) => items.filter((f) => f.status === s);
  const onMove = (id: string, status: DevFeedbackStatus, position?: number) => {
    const f = items.find((x) => x.id === id);
    if (f) move(f, status, position);
  };
  return (
    <div
      role="group"
      aria-label={t('desenvolvedor-quadro')}
      className="flex gap-3 overflow-x-auto p-3"
    >
      {STATUSES.map((s) => (
        <Column
          key={s}
          status={s}
          items={byStatus(s)}
          nextId={nextId}
          onOpen={onOpen}
          onMove={onMove}
        />
      ))}
    </div>
  );
}

function Section({
  title,
  children,
  empty,
}: {
  title: string;
  children?: ReactNode;
  empty?: boolean;
}) {
  return (
    <section aria-label={title} className="flex flex-col gap-1.5">
      <h3 className="label-caps text-[11px] text-dim">{title}</h3>
      {empty ? (
        <p className="text-[12.5px] text-dim">{t('desenvolvedor-secao-vazia')}</p>
      ) : (
        children
      )}
    </section>
  );
}

/** Próximos status oferecidos no painel para cada status. */
const NEXT: Record<DevFeedbackStatus, DevFeedbackStatus[]> = {
  aberto: ['em_andamento', 'resolvido', 'descartado'],
  em_andamento: ['resolvido', 'aberto', 'descartado'],
  resolvido: ['aberto'],
  descartado: ['aberto'],
};

/**
 * Painel lateral: o item inteiro em seções fixas (problema, proposta, critério de aceite,
 * onde mexer, reforços, relacionados) e a origem discreta no fim.
 */
export function FeedbackPanel({
  f,
  column,
  onClose,
  onOpenRelated,
  onEdit,
  deleteButton,
}: {
  f: DevFeedback;
  /** Os itens da coluna do feedback, na ordem do quadro (para subir e descer). */
  column: DevFeedback[];
  onClose: () => void;
  onOpenRelated: (id: string) => void;
  onEdit: () => void;
  deleteButton: ReactNode;
}) {
  const { move, pending } = useMove();
  const s = sections(f);
  const at = column.findIndex((x) => x.id === f.id);
  const timeline = f.notes;
  return (
    <RDialog.Root open onOpenChange={(o) => (o ? null : onClose())}>
      <RDialog.Portal>
        <RDialog.Overlay className="fixed inset-0 z-40 bg-overlay/40" />
        <RDialog.Content
          aria-describedby={undefined}
          className="fixed top-0 right-0 bottom-0 z-50 flex w-[min(96vw,560px)] animate-fade-in flex-col border-l border-panel pop-surface focus:outline-none"
        >
          <div className="flex h-10 shrink-0 items-center gap-2 border-b border-line pr-1.5 pl-3.5">
            <Pill tone={STATUS_TONE[f.status]}>{STATUS_LABEL[f.status]}</Pill>
            <span className="num text-[12px] font-semibold text-fg">{priorityLabel(f)}</span>
            <span className="flex-1" />
            <RDialog.Close asChild>
              <IconButton
                size="sm"
                label={t('comum-fechar')}
                icon={<Icon name="x" />}
                tooltip={false}
              />
            </RDialog.Close>
          </div>
          <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto p-4">
            <div className="flex flex-col gap-2">
              <RDialog.Title
                data-selectable
                className="text-[16px] leading-snug font-semibold text-fg"
              >
                {f.title}
              </RDialog.Title>
              <span className="flex flex-wrap gap-1">
                <Pill tone="accent">{AREA_LABEL[f.area]}</Pill>
                <Pill>{KIND_LABEL[f.kind]}</Pill>
                {f.user_impact ? (
                  <Pill tone={f.user_impact === 'alto' ? 'negative' : 'dim'}>
                    {t('desenvolvedor-impacto', { impacto: f.user_impact })}
                  </Pill>
                ) : null}
                {f.confirmed_by_user ? (
                  <Pill tone="positive">{t('desenvolvedor-confirmado')}</Pill>
                ) : null}
                {f.assignee ? <Pill>{f.assignee}</Pill> : null}
                {f.labels.map((l) => (
                  <Pill key={l}>#{l}</Pill>
                ))}
              </span>
            </div>
            <div className="flex flex-wrap items-center gap-1">
              {NEXT[f.status].map((st) => (
                <Button
                  key={st}
                  size="sm"
                  variant="secondary"
                  disabled={pending}
                  onClick={() => move(f, st)}
                >
                  {t('desenvolvedor-mover-para', { status: st })}
                </Button>
              ))}
              <Button
                size="sm"
                variant="ghost"
                disabled={pending || at <= 0}
                onClick={() => move(f, f.status, at)}
              >
                {t('desenvolvedor-subir')}
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={pending || at < 0 || at >= column.length - 1}
                onClick={() => move(f, f.status, at + 2)}
              >
                {t('desenvolvedor-descer')}
              </Button>
              <span className="flex-1" />
              <Button
                size="sm"
                variant="primary"
                onClick={() => copyText(taskMarkdown(f), t('desenvolvedor-md-copiado'))}
              >
                {t('desenvolvedor-copiar-md')}
              </Button>
            </div>

            <Section title={t('desenvolvedor-secao-problema')}>
              <div data-selectable className="text-[13px]">
                <Markdown text={s.problem} />
              </div>
              {f.expected || f.actual ? (
                <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[12.5px]">
                  {f.expected ? (
                    <>
                      <dt className="text-dim">{t('desenvolvedor-cc-esperado')}</dt>
                      <dd data-selectable>{f.expected}</dd>
                    </>
                  ) : null}
                  {f.actual ? (
                    <>
                      <dt className="text-dim">{t('desenvolvedor-cc-obtido')}</dt>
                      <dd data-selectable>{f.actual}</dd>
                    </>
                  ) : null}
                </dl>
              ) : null}
              {f.repro_steps ? (
                <div data-selectable className="text-[12.5px]">
                  <Markdown text={f.repro_steps} />
                </div>
              ) : null}
            </Section>

            <Section
              title={t('desenvolvedor-secao-proposta')}
              empty={!s.proposal && s.instructions.length === 0}
            >
              {s.proposal ? (
                <div data-selectable className="text-[13px]">
                  <Markdown text={s.proposal} />
                </div>
              ) : null}
              {s.instructions.length > 0 ? (
                <ul className="flex flex-col gap-1 text-[12.5px]">
                  {s.instructions.map((i) => (
                    <li key={i} data-selectable className="border-l-2 border-accent/60 pl-2">
                      {i}
                    </li>
                  ))}
                </ul>
              ) : null}
            </Section>

            <Section title={t('desenvolvedor-secao-aceite')} empty={!s.acceptance}>
              <div data-selectable className="text-[13px]">
                <Markdown text={s.acceptance ?? ''} />
              </div>
            </Section>

            <Section title={t('desenvolvedor-secao-onde')} empty={s.pointers.length === 0}>
              <ul className="flex flex-col gap-1">
                {s.pointers.map((p) => (
                  <li key={p}>
                    <button
                      type="button"
                      title={t('desenvolvedor-copiar-caminho')}
                      onClick={() => copyText(p, t('desenvolvedor-caminho-copiado'))}
                      className="font-mono text-[12px] text-accent underline-offset-2 hover:underline"
                    >
                      {p}
                    </button>
                  </li>
                ))}
              </ul>
            </Section>

            <Section title={t('desenvolvedor-reforcos')} empty={timeline.length === 0}>
              <ol
                aria-label={t('desenvolvedor-reforcos')}
                className="flex flex-col gap-2 border-l border-line-strong pl-3"
              >
                {timeline.map((n) => (
                  <li key={n.id} className="relative text-[12px]">
                    <span
                      aria-hidden
                      className={cx(
                        'absolute top-[6px] -left-[15.5px] size-1.5 rounded-full',
                        n.kind === 'reforco' ? 'bg-accent' : 'bg-line-strong',
                      )}
                    />
                    <span className="num text-dim">{formatDateTime(n.created_at)}</span>
                    <span className="text-dim">
                      {' · '}
                      {t('desenvolvedor-nota-tipo', { kind: n.kind })}
                    </span>
                    {n.note ? (
                      <p data-selectable className="mt-0.5 text-fg">
                        {n.note}
                      </p>
                    ) : (
                      <span className="text-dim"> {t('desenvolvedor-sem-nota')}</span>
                    )}
                  </li>
                ))}
              </ol>
            </Section>

            <Section title={t('desenvolvedor-secao-relacionados')} empty={f.links.length === 0}>
              <ul className="flex flex-col gap-1 text-[12.5px]">
                {f.links.map((l) => (
                  <li key={`${l.kind}-${l.related_id}`} className="flex flex-wrap gap-x-2">
                    <span className="text-dim">{linkLabel(l)}</span>
                    <button
                      type="button"
                      onClick={() => onOpenRelated(l.related_id)}
                      className="text-left text-accent underline-offset-2 hover:underline"
                    >
                      {l.related_title}
                    </button>
                    <span className="text-dim">· {STATUS_LABEL[l.related_status]}</span>
                  </li>
                ))}
              </ul>
            </Section>

            <div className="mt-auto flex flex-wrap items-center gap-2 border-t border-line pt-3">
              <p className="min-w-0 flex-1 text-[11px] text-dim">
                {t('desenvolvedor-origem', {
                  autor: authorLabel(f.created_by),
                  data: formatDateTime(f.created_at),
                })}
                {f.app_version ? ` · ${t('desenvolvedor-versao', { versao: f.app_version })}` : ''}
              </p>
              <Button size="sm" variant="ghost" onClick={onEdit}>
                {t('comum-editar')}
              </Button>
              {deleteButton}
            </div>
          </div>
        </RDialog.Content>
      </RDialog.Portal>
    </RDialog.Root>
  );
}
