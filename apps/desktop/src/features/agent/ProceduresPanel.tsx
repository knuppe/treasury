/**
 * Memória de procedimentos do agente (Agente › Memória): o que ele aprendeu ao buscar
 * informações — onde baixar o extrato de uma conta, que menu leva à fatura, que formato o
 * arquivo tem, de onde veio o dado de um ativo. O agente grava e atualiza sozinho (sem
 * proposta, com histórico e desfazer); aqui o usuário busca, vê de onde cada nota veio e
 * quantas vezes funcionou, edita e apaga.
 */
import { useState } from 'react';
import { Link } from 'react-router';

import type { Procedure } from '@/api/generated/Procedure';
import type { ProcedureTopic } from '@/api/generated/ProcedureTopic';
import { useDeleteProcedure, useProcedures, useSaveProcedure } from '@/api/hooks/ai';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  InstrumentCard,
  Private,
  Select,
  toast,
  useFieldControl,
} from '@/design/components';
import {
  ConfirmButton,
  ErrorBlock,
  LoadingBlock,
  errorMessage,
} from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDate, formatDateTime } from '@/lib/format';

import { PROCEDURE_TOPIC_LABEL } from './format';
import { Pill } from './ui';

const ALL = '__all__';
const TOPICS = Object.keys(PROCEDURE_TOPIC_LABEL) as ProcedureTopic[];

function StepsInput({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const a11y = useFieldControl({});
  return (
    <textarea
      id={a11y.id}
      aria-describedby={a11y['aria-describedby']}
      rows={5}
      value={value}
      maxLength={2000}
      onChange={(e) => onChange(e.target.value)}
      className="w-full resize-y rounded-control border border-line-strong bg-bg px-2.5 py-2 text-[13px] text-fg outline-none focus:border-accent"
    />
  );
}

/** Cadastro ou edição de uma nota (o agente costuma gravar sozinho). */
function ProcedureDialog({
  item,
  open,
  onOpenChange,
}: {
  item: Procedure | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const save = useSaveProcedure();
  const [topic, setTopic] = useState<ProcedureTopic>(item?.topic ?? 'statement');
  const [title, setTitle] = useState(item?.title ?? '');
  const [site, setSite] = useState(item?.site ?? '');
  const [format, setFormat] = useState(item?.file_format ?? '');
  const [steps, setSteps] = useState(item?.steps ?? '');
  const submit = () =>
    save.mutate(
      {
        ...(item ? { id: item.id } : {}),
        topic,
        title: title.trim(),
        steps: steps.trim(),
        site: site.trim() || null,
        file_format: format.trim() || null,
      },
      {
        onSuccess: (r) => {
          onOpenChange(false);
          toast({
            tone: 'success',
            title: item ? t('agente-proc-atualizado') : t('agente-proc-gravado'),
            description:
              r.redactions > 0 ? t('agente-proc-ocultados', { n: r.redactions }) : undefined,
          });
        },
        onError: (e) =>
          toast({ tone: 'error', title: t('agente-nao-gravou'), description: errorMessage(e) }),
      },
    );
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="md"
      title={item ? t('agente-proc-editar') : t('agente-proc-novo')}
      description={t('agente-proc-dialogo-texto')}
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="primary"
            loading={save.isPending}
            disabled={!title.trim() || !steps.trim()}
            onClick={submit}
          >
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <form
        className="flex flex-col gap-3"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-[200px_minmax(0,1fr)]">
          <Field label={t('agente-assunto')}>
            <Select
              value={topic}
              onValueChange={(v) => setTopic(v as ProcedureTopic)}
              options={TOPICS.map((x) => ({ value: x, label: PROCEDURE_TOPIC_LABEL[x] }))}
            />
          </Field>
          <Field label={t('agente-titulo')}>
            <Input value={title} maxLength={120} onChange={(e) => setTitle(e.target.value)} />
          </Field>
        </div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-[minmax(0,1fr)_140px]">
          <Field label={t('agente-site')} hint={t('agente-site-dica')}>
            <Input value={site} onChange={(e) => setSite(e.target.value)} />
          </Field>
          <Field label={t('agente-formato')}>
            <Input
              value={format}
              maxLength={20}
              placeholder="OFX"
              onChange={(e) => setFormat(e.target.value)}
            />
          </Field>
        </div>
        <Field label={t('agente-passos')} hint={t('agente-passos-dica')}>
          <StepsInput value={steps} onChange={setSteps} />
        </Field>
      </form>
    </Dialog>
  );
}

function origin(p: Procedure): string {
  // O cliente às vezes já começa com "agente" ("agente (nome da CLI)"): sem repetir.
  const data = formatDate(p.created_at.slice(0, 10));
  if (p.created_by === 'user') return t('agente-proc-por-voce', { data });
  if (!p.client) return t('agente-proc-pelo-agente', { data });
  return /^agente\b/i.test(p.client)
    ? t('agente-proc-pelo-cliente', { cliente: p.client, data })
    : t('agente-proc-pelo-agente-cliente', { cliente: p.client, data });
}

function ProcedureRow({ p }: { p: Procedure }) {
  const del = useDeleteProcedure();
  const [editing, setEditing] = useState(false);
  const links = [p.institution_name, p.account_name, p.asset_name].filter(Boolean).join(' · ');
  const conversation = p.conversation_id
    ? `/agente?aba=conversa&conversa=${encodeURIComponent(p.conversation_id)}`
    : null;
  return (
    <li
      aria-label={t('agente-proc-rotulo', { titulo: p.title })}
      className="flex flex-col gap-1.5 px-4 py-3"
    >
      <div className="flex flex-wrap items-center gap-2">
        <Pill tone="accent">{PROCEDURE_TOPIC_LABEL[p.topic]}</Pill>
        <Private>
          <span className="text-[13.5px] font-medium text-fg">{p.title}</span>
        </Private>
        <span className="flex-1" />
        {p.worked > 0 ? (
          <Pill tone="positive" title={t('agente-proc-funcionou-dica')}>
            {t('agente-proc-funcionou', { n: p.worked })}
          </Pill>
        ) : null}
        {p.failed > 0 ? (
          <Pill tone="negative" title={t('agente-proc-falhou-dica')}>
            {t('agente-proc-falhou', { n: p.failed })}
          </Pill>
        ) : null}
      </div>
      {p.site || p.file_format || links ? (
        <p className="flex flex-wrap gap-x-3 text-[12px] text-dim">
          {p.site ? (
            <span className="inline-flex items-center gap-1">
              <Icon name="lock" size={10} />
              {p.site}
            </span>
          ) : null}
          {p.file_format ? (
            <span>{t('agente-proc-arquivo', { formato: p.file_format })}</span>
          ) : null}
          {links ? (
            <Private>
              <span>{links}</span>
            </Private>
          ) : null}
        </p>
      ) : null}
      <Private>
        <p data-selectable className="text-[12.5px] leading-relaxed whitespace-pre-wrap text-fg">
          {p.steps}
        </p>
      </Private>
      {p.last_note ? (
        <Private>
          <p className="text-[12px] text-warning">
            {t('agente-proc-observacao', { nota: p.last_note })}
          </p>
        </Private>
      ) : null}
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11.5px] text-dim">
        <span>{origin(p)}</span>
        {conversation ? (
          <Link className="text-accent underline-offset-2 hover:underline" to={conversation}>
            {p.conversation_title
              ? t('agente-proc-conversa', { titulo: p.conversation_title })
              : t('agente-proc-ver-conversa')}
          </Link>
        ) : null}
        {p.updated_at !== p.created_at ? (
          <span>{t('agente-proc-atualizado-em', { quando: formatDateTime(p.updated_at) })}</span>
        ) : null}
        {p.last_used_at ? (
          <span>
            {p.last_outcome
              ? t('agente-proc-ultimo-uso-resultado', {
                  quando: formatDateTime(p.last_used_at),
                  resultado: p.last_outcome === 'worked' ? 'worked' : 'failed',
                })
              : t('agente-proc-ultimo-uso', { quando: formatDateTime(p.last_used_at) })}
          </span>
        ) : null}
        <span className="flex-1" />
        <Button size="sm" variant="ghost" onClick={() => setEditing(true)}>
          {t('comum-editar')}
        </Button>
        <ConfirmButton
          label={t('comum-apagar')}
          title={t('agente-proc-apagar-titulo')}
          description={t('agente-proc-apagar-texto')}
          confirmLabel={t('comum-apagar')}
          pending={del.isPending}
          onConfirm={() =>
            del.mutateAsync(p.id).then(
              () => toast({ tone: 'success', title: t('agente-proc-apagado') }),
              (e: unknown) =>
                toast({
                  tone: 'error',
                  title: t('agente-nao-apagou'),
                  description: errorMessage(e),
                }),
            )
          }
        />
      </div>
      {editing ? <ProcedureDialog item={p} open onOpenChange={setEditing} /> : null}
    </li>
  );
}

export function ProceduresPanel() {
  const [query, setQuery] = useState('');
  const [topic, setTopic] = useState<string>(ALL);
  const [adding, setAdding] = useState(false);
  const args = {
    ...(query.trim() ? { query: query.trim() } : {}),
    ...(topic !== ALL ? { topic: topic as ProcedureTopic } : {}),
    limit: 200,
  };
  const list = useProcedures(args);
  const items = list.data ?? [];
  const filtered = Boolean(args.query || args.topic);
  return (
    <InstrumentCard
      index="02"
      title={t('agente-procs-titulo')}
      padded={false}
      actions={
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setAdding(true)}
        >
          {t('agente-adicionar')}
        </Button>
      }
      footer={<p className="text-[12px] text-dim">{t('agente-procs-rodape')}</p>}
    >
      <div className="flex flex-wrap items-end gap-3 border-b border-line px-4 py-3">
        <Field label={t('agente-buscar')} className="min-w-[200px] flex-1">
          <Input
            value={query}
            placeholder={t('agente-procs-placeholder')}
            onChange={(e) => setQuery(e.target.value)}
          />
        </Field>
        <Field label={t('agente-assunto')} className="w-[220px]">
          <Select
            value={topic}
            onValueChange={setTopic}
            options={[
              { value: ALL, label: t('agente-todos-assuntos') },
              ...TOPICS.map((x) => ({ value: x, label: PROCEDURE_TOPIC_LABEL[x] })),
            ]}
          />
        </Field>
      </div>
      {list.isLoading ? (
        <LoadingBlock />
      ) : list.error ? (
        <ErrorBlock error={list.error} className="m-3" />
      ) : items.length === 0 ? (
        <p className="px-4 py-6 text-center text-[13px] text-dim">
          {filtered ? t('agente-procs-filtro-vazio') : t('agente-procs-vazio')}
        </p>
      ) : (
        <ul aria-label={t('agente-procs')} className="divide-y divide-line">
          {items.map((p) => (
            <ProcedureRow key={p.id} p={p} />
          ))}
        </ul>
      )}
      {adding ? <ProcedureDialog item={null} open onOpenChange={setAdding} /> : null}
    </InstrumentCard>
  );
}
