/**
 * DESENVOLVEDOR: o feedback que o agente registrou para o desenvolvedor do Treasury (bugs,
 * melhorias, ferramentas que faltaram, dados confusos, lacunas da documentação). Por padrão
 * um quadro por status, na ordem da fila (a manual, depois a prioridade), com o item inteiro
 * no painel lateral; a lista fica a um clique. Aqui o usuário arrasta entre colunas, reordena,
 * edita, apaga e copia em Markdown (pronto para uma issue ou para um agente de código).
 *
 * Só aparece no menu com o modo desenvolvedor ligado (Ajustes › Desenvolvedor).
 */
import { useState } from 'react';
import { Link } from 'react-router';

import { screenMeta } from '@/app/screens';
import { useDeveloperMode } from '@/app/useVisibleScreens';
import type { DevFeedback } from '@/api/generated/DevFeedback';
import type { DevFeedbackArea } from '@/api/generated/DevFeedbackArea';
import type { DevFeedbackKind } from '@/api/generated/DevFeedbackKind';
import type { DevFeedbackStatus } from '@/api/generated/DevFeedbackStatus';
import {
  useDeleteDevFeedback,
  useExportDevFeedback,
  useDevFeedback,
  useSetDevFeedbackStatus,
  useUpdateDevFeedback,
} from '@/api/hooks/devfeedback';
import { isTauri } from '@/api/client';
import { useAppSettings } from '@/api/hooks/settings';
import { useUndoToast } from '@/api/hooks/undoToast';
import { chooseSavePath } from '@/lib/saveDialog';
import {
  Button,
  Chip,
  ChipGroup,
  Dialog,
  DialogClose,
  EmptyState,
  Field,
  Input,
  InstrumentCard,
  Markdown,
  ScreenHeader,
  Select,
  toast,
  useFieldControl,
} from '@/design/components';
import { cx } from '@/design/cx';
import { Pill } from '@/features/agent/ui';
import {
  ConfirmButton,
  ErrorBlock,
  LoadingBlock,
  errorMessage,
} from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { copyText as copy, DevFeedbackBoard, FeedbackPanel, STATUS_TONE } from './DevFeedbackBoard';
import {
  allMarkdown,
  AREA_LABEL,
  AREAS,
  authorLabel,
  taskMarkdown,
  KIND_LABEL,
  KINDS,
  priorityLabel,
  STATUS_LABEL,
  STATUSES,
} from './format';

const meta = screenMeta('desenvolvedor');

const OPEN = '__open__';
const ALL = '__all__';

const VIEW_KEY = 'treasury.devfeedback.view';
type View = 'quadro' | 'lista';

function savedView(): View {
  try {
    return localStorage.getItem(VIEW_KEY) === 'lista' ? 'lista' : 'quadro';
  } catch {
    return 'quadro';
  }
}

/** Baixa por link de Blob: só no navegador (modo de demonstração); o WebKit do app ignora. */
function download(text: string, filename: string) {
  const url = URL.createObjectURL(new Blob([text], { type: 'text/markdown' }));
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Exportar Markdown: no app, pergunta onde salvar e o núcleo grava o arquivo. */
function useExportMarkdown() {
  const exp = useExportDevFeedback();
  const run = async (text: string, fileName: string, n: number) => {
    if (!isTauri()) {
      download(text, fileName);
      return;
    }
    try {
      const path = await chooseSavePath(t('desenvolvedor-exportar-salvar'), fileName, [
        { name: 'Markdown', extensions: ['md'] },
      ]);
      if (!path) return;
      const r = await exp.mutateAsync({ path, text });
      toast({
        tone: 'success',
        title: t('desenvolvedor-exportados', { n }),
        description: r.path,
      });
    } catch (e) {
      toast({
        tone: 'error',
        title: t('desenvolvedor-erro-exportar'),
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };
  return { run, pending: exp.isPending };
}

function BodyInput({
  value,
  onChange,
  rows = 8,
  max = 4000,
}: {
  value: string;
  onChange: (v: string) => void;
  rows?: number;
  max?: number;
}) {
  const a11y = useFieldControl({});
  return (
    <textarea
      id={a11y.id}
      aria-describedby={a11y['aria-describedby']}
      rows={rows}
      value={value}
      maxLength={max}
      onChange={(e) => onChange(e.target.value)}
      className="w-full resize-y rounded-control border border-line-strong bg-bg px-2.5 py-2 text-[13px] text-fg outline-none focus:border-accent"
    />
  );
}

function EditDialog({
  item,
  onOpenChange,
}: {
  item: DevFeedback;
  onOpenChange: (o: boolean) => void;
}) {
  const save = useUpdateDevFeedback();
  const [title, setTitle] = useState(item.title);
  const [body, setBody] = useState(item.body);
  const [area, setArea] = useState<DevFeedbackArea>(item.area);
  const [kind, setKind] = useState<DevFeedbackKind>(item.kind);
  const [importance, setImportance] = useState(String(item.importance));
  const [proposal, setProposal] = useState(item.proposal ?? '');
  const [acceptance, setAcceptance] = useState(item.acceptance_criteria ?? '');
  const [pointers, setPointers] = useState(item.code_pointers.join('\n'));
  const [assignee, setAssignee] = useState(item.assignee ?? '');
  const [labels, setLabels] = useState(item.labels.join(', '));
  const imp = Number(importance);
  const valid = title.trim() && body.trim() && Number.isInteger(imp) && imp >= 1 && imp <= 1000;
  const submit = () =>
    save.mutate(
      {
        id: item.id,
        title: title.trim(),
        body: body.trim(),
        area,
        kind,
        importance: imp,
        proposal: proposal.trim(),
        acceptance_criteria: acceptance.trim(),
        code_pointers: pointers
          .split('\n')
          .map((p) => p.trim())
          .filter(Boolean),
        assignee: assignee.trim(),
        labels: labels
          .split(',')
          .map((l) => l.trim())
          .filter(Boolean),
      },
      {
        onSuccess: () => {
          onOpenChange(false);
          toast({ tone: 'success', title: t('desenvolvedor-atualizado') });
        },
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('desenvolvedor-erro-gravar'),
            description: errorMessage(e),
          }),
      },
    );
  return (
    <Dialog
      open
      onOpenChange={onOpenChange}
      size="lg"
      title={t('desenvolvedor-editar-titulo')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" loading={save.isPending} disabled={!valid} onClick={submit}>
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <form
        className="flex flex-col gap-3"
        onSubmit={(e) => {
          e.preventDefault();
          if (valid) submit();
        }}
      >
        <Field label={t('desenvolvedor-titulo')}>
          <Input value={title} maxLength={160} onChange={(e) => setTitle(e.target.value)} />
        </Field>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-[1fr_1fr_120px]">
          <Field label={t('desenvolvedor-area')}>
            <Select
              value={area}
              onValueChange={(v) => setArea(v as DevFeedbackArea)}
              options={AREAS.map((a) => ({ value: a, label: AREA_LABEL[a] }))}
            />
          </Field>
          <Field label={t('desenvolvedor-tipo')}>
            <Select
              value={kind}
              onValueChange={(v) => setKind(v as DevFeedbackKind)}
              options={KINDS.map((k) => ({ value: k, label: KIND_LABEL[k] }))}
            />
          </Field>
          <Field label={t('desenvolvedor-importancia')}>
            <Input
              inputMode="numeric"
              value={importance}
              onChange={(e) => setImportance(e.target.value.replace(/\D/g, ''))}
            />
          </Field>
        </div>
        <Field label={t('desenvolvedor-secao-problema')}>
          <BodyInput value={body} onChange={setBody} />
        </Field>
        <Field label={t('desenvolvedor-secao-proposta')}>
          <BodyInput value={proposal} onChange={setProposal} rows={4} max={2000} />
        </Field>
        <Field label={t('desenvolvedor-secao-aceite')}>
          <BodyInput value={acceptance} onChange={setAcceptance} rows={3} max={2000} />
        </Field>
        <Field label={t('desenvolvedor-secao-onde')} hint={t('desenvolvedor-onde-dica')}>
          <BodyInput value={pointers} onChange={setPointers} rows={3} max={6000} />
        </Field>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <Field label={t('desenvolvedor-responsavel')}>
            <Input value={assignee} maxLength={40} onChange={(e) => setAssignee(e.target.value)} />
          </Field>
          <Field label={t('desenvolvedor-etiquetas')} hint={t('desenvolvedor-etiquetas-dica')}>
            <Input value={labels} onChange={(e) => setLabels(e.target.value)} />
          </Field>
        </div>
      </form>
    </Dialog>
  );
}

/** Apagar (com confirmação e "Desfazer" no aviso). */
function DeleteButton({ f, onDone }: { f: DevFeedback; onDone?: () => void }) {
  const del = useDeleteDevFeedback();
  const undoToast = useUndoToast();
  return (
    <ConfirmButton
      label={t('comum-apagar')}
      title={t('desenvolvedor-apagar-titulo')}
      description={t('desenvolvedor-apagar-texto')}
      confirmLabel={t('comum-apagar')}
      pending={del.isPending}
      onConfirm={() =>
        del.mutateAsync(f.id).then(
          (r) => {
            onDone?.();
            undoToast({
              title: t('desenvolvedor-apagado'),
              description: f.title,
              changeSetId: r.change_set_id,
            });
          },
          (e: unknown) =>
            toast({
              tone: 'error',
              title: t('desenvolvedor-erro-apagar'),
              description: errorMessage(e),
            }),
        )
      }
    />
  );
}

/** Próximos status oferecidos para cada status. */
const next = (): Record<DevFeedbackStatus, { status: DevFeedbackStatus; label: string }[]> => ({
  aberto: [
    { status: 'em_andamento', label: t('desenvolvedor-em-andamento') },
    { status: 'resolvido', label: t('desenvolvedor-resolvido') },
    { status: 'descartado', label: t('desenvolvedor-descartar') },
  ],
  em_andamento: [
    { status: 'resolvido', label: t('desenvolvedor-resolvido') },
    { status: 'aberto', label: t('desenvolvedor-voltar-aberto') },
    { status: 'descartado', label: t('desenvolvedor-descartar') },
  ],
  resolvido: [{ status: 'aberto', label: t('desenvolvedor-reabrir') }],
  descartado: [{ status: 'aberto', label: t('desenvolvedor-reabrir') }],
});

function FeedbackRow({ f, onOpen }: { f: DevFeedback; onOpen: () => void }) {
  const setStatus = useSetDevFeedbackStatus();
  const [editing, setEditing] = useState(false);
  const closed = f.status === 'resolvido' || f.status === 'descartado';
  return (
    <li
      aria-label={t('desenvolvedor-feedback-rotulo', { titulo: f.title })}
      className="flex gap-4 px-4 py-4"
    >
      <div
        className={cx(
          'flex h-12 w-12 shrink-0 flex-col items-center justify-center rounded-control border',
          closed ? 'border-line text-dim' : 'border-accent/50 bg-accent-soft text-accent',
        )}
        title={t('desenvolvedor-importancia-dica')}
      >
        <span className="mt-0.5 font-mono text-[8.5px] tracking-[0.08em] uppercase">
          {t('desenvolvedor-prioridade-curta')}
        </span>
        <span className="num text-[18px] leading-none font-semibold">{f.importance}</span>
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        <div className="flex flex-wrap items-start gap-x-3 gap-y-1.5">
          <button
            type="button"
            onClick={onOpen}
            className={cx(
              'min-w-0 flex-1 basis-[260px] text-left text-[14px] font-medium hover:underline',
              closed ? 'text-dim line-through decoration-line-strong' : 'text-fg',
            )}
          >
            {f.title}
          </button>
          <span className="flex flex-wrap gap-1">
            <Pill tone={STATUS_TONE[f.status]}>{STATUS_LABEL[f.status]}</Pill>
            <Pill tone="accent">{AREA_LABEL[f.area]}</Pill>
            <Pill>{KIND_LABEL[f.kind]}</Pill>
          </span>
        </div>
        <div data-selectable className="text-[12.5px]">
          <Markdown text={f.body} />
        </div>
        <p className="flex flex-wrap gap-x-3 gap-y-1 text-[11.5px] text-dim">
          <span>{t('desenvolvedor-n-reforcos', { n: f.bumps })}</span>
          {f.last_bumped_at ? (
            <span>{t('desenvolvedor-ultimo', { data: formatDateTime(f.last_bumped_at) })}</span>
          ) : null}
        </p>
        {f.notes.length > 0 ? (
          <ol
            aria-label={t('desenvolvedor-reforcos')}
            className="flex flex-col gap-1.5 border-l border-line-strong pl-3"
          >
            {f.notes.map((n) => (
              <li key={n.id} className="relative text-[12px]">
                <span
                  aria-hidden
                  className="absolute top-[7px] -left-[15.5px] size-1.5 rounded-full bg-accent"
                />
                <span className="num text-dim">{formatDateTime(n.created_at)}</span>
                <span className="text-dim"> · {authorLabel(n.client)}</span>
                {n.note ? (
                  <span data-selectable className="text-fg">
                    : {n.note}
                  </span>
                ) : (
                  <span className="text-dim"> {t('desenvolvedor-sem-nota')}</span>
                )}
              </li>
            ))}
          </ol>
        ) : null}
        <div className="flex flex-wrap items-center gap-1">
          {next()[f.status].map((n) => (
            <Button
              key={n.status}
              size="sm"
              variant="secondary"
              disabled={setStatus.isPending}
              onClick={() =>
                setStatus.mutate(
                  { id: f.id, status: n.status },
                  {
                    onSuccess: () =>
                      toast({
                        tone: 'success',
                        title: t('desenvolvedor-status-mudou', { status: n.status }),
                      }),
                    onError: (e) =>
                      toast({
                        tone: 'error',
                        title: t('desenvolvedor-erro-status'),
                        description: errorMessage(e),
                      }),
                  },
                )
              }
            >
              {n.label}
            </Button>
          ))}
          <span className="flex-1" />
          <Button
            size="sm"
            variant="ghost"
            onClick={() => copy(taskMarkdown(f), t('desenvolvedor-md-copiado'))}
          >
            {t('desenvolvedor-copiar-md')}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setEditing(true)}>
            {t('comum-editar')}
          </Button>
          <DeleteButton f={f} />
        </div>
      </div>
      {editing ? <EditDialog item={f} onOpenChange={setEditing} /> : null}
    </li>
  );
}

/** Minúsculas e sem acentos, para comparar textos na busca. */
function fold(s: string): string {
  return s
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
}

/** Filtros rápidos como chips: "Todas" e um por valor. */
function ChipFilter<T extends string>({
  label,
  all,
  values,
  names,
  value,
  onChange,
}: {
  label: string;
  all: string;
  values: readonly T[];
  names: Record<T, string>;
  value: T | null;
  onChange: (v: T | null) => void;
}) {
  return (
    <ChipGroup label={label}>
      <Chip pressed={value === null} onClick={() => onChange(null)}>
        {all}
      </Chip>
      {values.map((v) => (
        <Chip key={v} pressed={value === v} onClick={() => onChange(value === v ? null : v)}>
          {names[v]}
        </Chip>
      ))}
    </ChipGroup>
  );
}

/** O primeiro da fila: o topo de Em andamento ou, sem nada lá, o de Aberto. */
function nextUp(items: DevFeedback[]): DevFeedback | undefined {
  return items.find((f) => f.status === 'em_andamento') ?? items.find((f) => f.status === 'aberto');
}

export default function DevFeedbackScreen() {
  const settings = useAppSettings();
  const dev = useDeveloperMode();
  const [view, setView] = useState<View>(savedView);
  const [status, setStatus] = useState<string>(OPEN);
  const [area, setArea] = useState<DevFeedbackArea | null>(null);
  const [kind, setKind] = useState<DevFeedbackKind | null>(null);
  const [openId, setOpenId] = useState<string | null>(null);
  const [editing, setEditing] = useState<DevFeedback | null>(null);
  // Tudo de uma vez (o quadro mostra os quatro status); filtros e busca na própria tela.
  const list = useDevFeedback({}, dev);
  const [query, setQuery] = useState('');
  const all = list.data ?? [];
  const pick = (v: View) => {
    setView(v);
    try {
      localStorage.setItem(VIEW_KEY, v);
    } catch {
      // Sem armazenamento (janela privada): a escolha vale só nesta visita.
    }
  };
  // Busca na própria tela (sem ida ao núcleo): título, texto, reforços e quem registrou,
  // sem diferença de acento ou maiúscula; todas as palavras precisam aparecer.
  const words = fold(query).split(/\s+/).filter(Boolean);
  const matched = all.filter((f) => {
    if (area && f.area !== area) return false;
    if (kind && f.kind !== kind) return false;
    if (words.length === 0) return true;
    const hay = fold(
      [
        f.title,
        f.body,
        f.proposal ?? '',
        f.acceptance_criteria ?? '',
        f.created_by,
        ...f.labels,
        ...f.notes.map((n) => `${n.note ?? ''} ${n.client}`),
      ].join(' '),
    );
    return words.every((w) => hay.includes(w));
  });
  const items =
    view === 'quadro'
      ? matched
      : matched.filter((f) =>
          status === OPEN
            ? f.status === 'aberto' || f.status === 'em_andamento'
            : status === ALL || f.status === status,
        );
  const exporter = useExportMarkdown();
  const filtered = (view === 'lista' && status !== OPEN) || !!area || !!kind || words.length > 0;
  const stamp = new Date().toISOString().slice(0, 10);
  const opened = openId ? all.find((f) => f.id === openId) : undefined;
  const top = nextUp(matched);

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={t('desenvolvedor-descricao')}
        meta={
          dev && list.data
            ? items.length !== all.length
              ? t('desenvolvedor-n-de-feedbacks', { n: items.length, total: all.length })
              : t('desenvolvedor-n-feedbacks', { n: items.length })
            : undefined
        }
        actions={
          dev ? (
            <>
              <Button
                size="sm"
                variant="secondary"
                disabled={items.length === 0}
                onClick={() =>
                  copy(allMarkdown(items), t('desenvolvedor-n-copiados', { n: items.length }))
                }
              >
                {t('desenvolvedor-copiar-tudo')}
              </Button>
              <Button
                size="sm"
                variant="secondary"
                disabled={items.length === 0}
                loading={exporter.pending}
                onClick={() =>
                  void exporter.run(
                    allMarkdown(items),
                    `treasury-feedback-${stamp}.md`,
                    items.length,
                  )
                }
              >
                {t('desenvolvedor-exportar-md')}
              </Button>
            </>
          ) : undefined
        }
      />
      {settings.isLoading ? (
        <LoadingBlock />
      ) : !dev ? (
        <EmptyState
          kicker={t('desenvolvedor-desligado')}
          title={t('desenvolvedor-desligado-titulo')}
          description={t('desenvolvedor-desligado-texto')}
          actions={
            <Link
              to="/ajustes"
              className="text-[13px] text-accent underline-offset-2 hover:underline"
            >
              {t('desenvolvedor-abrir-ajustes')}
            </Link>
          }
        />
      ) : (
        <InstrumentCard
          index={`${meta.index}.A`}
          title={t('desenvolvedor-card')}
          padded={false}
          footer={<p className="text-[12px] text-dim">{t('desenvolvedor-card-rodape')}</p>}
        >
          {top ? (
            <div
              role="region"
              aria-label={t('desenvolvedor-proximo')}
              className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b border-line bg-accent-soft/40 px-4 py-2.5"
            >
              <span className="label-caps text-[11px] text-accent">
                {t('desenvolvedor-proximo')}
              </span>
              <button
                type="button"
                onClick={() => setOpenId(top.id)}
                className="min-w-0 flex-1 truncate text-left text-[13.5px] font-medium text-fg hover:underline"
              >
                {top.title}
              </button>
              <span className="num text-[12px] text-dim">
                {priorityLabel(top)} · {STATUS_LABEL[top.status]}
              </span>
            </div>
          ) : null}
          <div className="flex flex-col gap-3 border-b border-line px-4 py-3">
            <div className="flex flex-wrap items-end gap-3">
              <Field label={t('desenvolvedor-buscar')} className="min-w-[240px] flex-1">
                <Input
                  type="search"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder={t('desenvolvedor-buscar-dica')}
                  aria-label={t('desenvolvedor-buscar')}
                />
              </Field>
              {view === 'lista' ? (
                <Field label={t('desenvolvedor-filtro-status')} className="w-[200px]">
                  <Select
                    value={status}
                    onValueChange={setStatus}
                    options={[
                      { value: OPEN, label: t('desenvolvedor-abertos') },
                      { value: ALL, label: t('desenvolvedor-todos') },
                      ...STATUSES.map((s) => ({ value: s, label: STATUS_LABEL[s] })),
                    ]}
                  />
                </Field>
              ) : null}
              <ChipGroup label={t('desenvolvedor-visao')}>
                <Chip pressed={view === 'quadro'} onClick={() => pick('quadro')}>
                  {t('desenvolvedor-visao-quadro')}
                </Chip>
                <Chip pressed={view === 'lista'} onClick={() => pick('lista')}>
                  {t('desenvolvedor-visao-lista')}
                </Chip>
              </ChipGroup>
            </div>
            <ChipFilter
              label={t('desenvolvedor-area')}
              all={t('desenvolvedor-todas-areas')}
              values={AREAS}
              names={AREA_LABEL}
              value={area}
              onChange={setArea}
            />
            <ChipFilter
              label={t('desenvolvedor-tipo')}
              all={t('desenvolvedor-todos-tipos')}
              values={KINDS}
              names={KIND_LABEL}
              value={kind}
              onChange={setKind}
            />
          </div>
          {list.isLoading ? (
            <LoadingBlock />
          ) : list.error ? (
            <ErrorBlock error={list.error} className="m-3" />
          ) : view === 'quadro' ? (
            all.length === 0 ? (
              <p className="px-4 py-6 text-center text-[13px] text-dim">
                {t('desenvolvedor-nenhum')}
              </p>
            ) : (
              <DevFeedbackBoard items={items} nextId={top?.id} onOpen={(f) => setOpenId(f.id)} />
            )
          ) : items.length === 0 ? (
            <p className="px-4 py-6 text-center text-[13px] text-dim">
              {filtered ? t('desenvolvedor-nenhum-filtro') : t('desenvolvedor-nenhum')}
            </p>
          ) : (
            <ul aria-label={t('desenvolvedor-lista')} className="divide-y divide-line">
              {items.map((f) => (
                <FeedbackRow key={f.id} f={f} onOpen={() => setOpenId(f.id)} />
              ))}
            </ul>
          )}
        </InstrumentCard>
      )}
      {opened ? (
        <FeedbackPanel
          f={opened}
          column={all.filter((x) => x.status === opened.status)}
          onClose={() => setOpenId(null)}
          onOpenRelated={setOpenId}
          onEdit={() => setEditing(opened)}
          deleteButton={<DeleteButton f={opened} onDone={() => setOpenId(null)} />}
        />
      ) : null}
      {editing ? <EditDialog item={editing} onOpenChange={() => setEditing(null)} /> : null}
    </div>
  );
}
