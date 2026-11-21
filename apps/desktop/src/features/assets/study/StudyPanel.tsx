/**
 * Ativo › Estudo (IA) (docs/agent-enrichment.md §7 e §8): aviso "Gerado por IA: pode conter
 * erros" sempre visível no topo, aceite da primeira vez (por cofre, sem marcação prévia),
 * "Contexto para a IA" salvo por ativo, a pergunta do momento, opções da posição (desligadas
 * por padrão), "O que vai para …" com o pacote exato, gerar (job acompanhado até o fim) e o
 * histórico dos 20 últimos estudos. Cada estudo aberto leva o selo com a data e o motor, os
 * números conferidos com os dados do ativo e "Copiar" com o aviso completo no fim.
 */
import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router';
import { useQueryClient } from '@tanstack/react-query';

import type { NumberCheck } from '@/api/generated/NumberCheck';
import type { Study } from '@/api/generated/Study';
import type { StudyArgs } from '@/api/generated/StudyArgs';
import type { StudySummary } from '@/api/generated/StudySummary';
import {
  useStudy,
  useStudyAccept,
  useStudyContext,
  useStudyDelete,
  useStudyJob,
  useStudyPackage,
  useStudyStart,
  useStudyState,
} from '@/api/hooks/assetStudy';
import { Button, Icon, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate, formatDateTime } from '@/lib/format';
import { useCompanion } from '@/state/companion';

import { ErrorBlock, LoadingBlock } from '../../portfolio/components/States';

const MAX_CONTEXT = 2000;

function day(iso: string): string {
  try {
    return formatDate(iso.slice(0, 10));
  } catch {
    return iso;
  }
}

const CONF_LABEL = (c: string | null) =>
  c === 'high'
    ? t('estudo-confianca-alta')
    : c === 'medium'
      ? t('estudo-confianca-media')
      : c === 'low'
        ? t('estudo-confianca-baixa')
        : t('estudo-confianca-sem');

/** Aviso obrigatório, sempre no topo (sem botão de fechar). */
export function StudyDisclaimer() {
  return (
    <div
      role="note"
      aria-label={t('estudo-aviso-titulo')}
      className="flex flex-col gap-1 rounded-sm border border-warning/50 bg-warning/5 px-3 py-2.5"
    >
      <span className="flex items-center gap-2 text-[13px] font-medium text-warning">
        <Icon name="alert" size={13} /> {t('estudo-aviso-titulo')}
      </span>
      <p className="text-[12.5px] leading-snug text-fg">{t('estudo-aviso-texto')}</p>
    </div>
  );
}

function Accept() {
  const [ok, setOk] = useState(false);
  const accept = useStudyAccept();
  return (
    <div className="flex flex-col gap-2 rounded-sm border border-line p-3">
      <label className="flex items-start gap-2 text-[13px]">
        <input type="checkbox" checked={ok} onChange={(e) => setOk(e.target.checked)} />
        <span>{t('estudo-aceite')}</span>
      </label>
      <Button
        size="sm"
        className="w-fit"
        disabled={!ok}
        loading={accept.isPending}
        onClick={() => accept.mutate({})}
      >
        {t('estudo-aceite-continuar')}
      </Button>
    </div>
  );
}

function ContextField({ assetId, initial }: { assetId: string; initial: string }) {
  const [text, setText] = useState(initial);
  const save = useStudyContext();
  const changed = text.trim() !== initial.trim();
  return (
    <div className="flex flex-col gap-1">
      <label htmlFor="estudo-contexto" className="label-caps">
        {t('estudo-contexto')}
      </label>
      <textarea
        id="estudo-contexto"
        value={text}
        maxLength={MAX_CONTEXT}
        rows={3}
        placeholder={t('estudo-contexto-dica')}
        onChange={(e) => setText(e.target.value)}
        className="rounded-sm border border-line bg-transparent p-2 text-[13px]"
      />
      <span className="flex items-center gap-2 text-[11px] text-dim">
        {t('estudo-contexto-limite', { n: text.length, max: MAX_CONTEXT })}
        <span className="grow" />
        <Button
          size="sm"
          variant="ghost"
          disabled={!changed}
          loading={save.isPending}
          onClick={() =>
            save.mutate(
              { asset_id: assetId, text },
              {
                onSuccess: () => toast({ tone: 'success', title: t('estudo-contexto-salvo') }),
                onError: (e: Error) =>
                  toast({ tone: 'error', title: t('estudo-nao-salvou'), description: e.message }),
              },
            )
          }
        >
          {t('estudo-contexto-salvar')}
        </Button>
      </span>
    </div>
  );
}

function PackagePreview({ args }: { args: StudyArgs }) {
  const q = useStudyPackage(args, true);
  if (q.isPending) return <LoadingBlock label={t('estudo-pacote')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  return (
    <div className="flex flex-col gap-1">
      <span className="text-[12px] text-dim">
        {q.data.engine?.local
          ? t('estudo-pacote-local')
          : t('estudo-pacote-tamanho', { n: q.data.chars })}
      </span>
      <pre
        aria-label={t('estudo-pacote')}
        className="max-h-72 overflow-auto rounded-sm border border-line p-2 font-mono text-[11px] leading-snug whitespace-pre-wrap"
      >
        {JSON.stringify(q.data.package, null, 2)}
      </pre>
    </div>
  );
}

function Generate({ assetId, engineName, provider, onStart }: GenerateProps) {
  const [question, setQuestion] = useState('');
  const [position, setPosition] = useState(false);
  const [values, setValues] = useState(false);
  const [preview, setPreview] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const start = useStudyStart();
  const job = useStudyJob(jobId);
  const running = Boolean(jobId) && (!job.data || job.data.status === 'running');
  // Terminou: o histórico ganha o estudo novo.
  const client = useQueryClient();
  const done = job.data?.status === 'done';
  useEffect(() => {
    if (done) void client.invalidateQueries({ queryKey: ['assets', 'study_state'] });
  }, [done, client]);
  const args: StudyArgs = {
    asset_id: assetId,
    ...(question.trim() ? { question: question.trim() } : {}),
    ...(position ? { include_position: true } : {}),
    ...(values ? { include_values: true } : {}),
  };
  const failed = job.data && job.data.status !== 'running' && job.data.status !== 'done';
  const error = failed
    ? job.data?.events.map((e) => e.event).find((e) => e.type === 'error')
    : null;
  return (
    <div className="flex flex-col gap-3">
      <label className="flex flex-col gap-1">
        <span className="label-caps">{t('estudo-pergunta')}</span>
        <input
          value={question}
          maxLength={500}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder={t('estudo-pergunta-dica')}
          className="h-9 rounded-sm border border-line bg-transparent px-2 text-[13px]"
        />
      </label>
      <div className="flex flex-col gap-1 text-[13px]">
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={position}
            onChange={(e) => setPosition(e.target.checked)}
          />
          {t('estudo-incluir-posicao')}
        </label>
        <label className="flex items-center gap-2">
          <input type="checkbox" checked={values} onChange={(e) => setValues(e.target.checked)} />
          {t('estudo-incluir-valores')}
        </label>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[12px] text-dim">
          {engineName ? t('estudo-motor', { motor: engineName }) : t('estudo-sem-motor')}
        </span>
        <span className="grow" />
        <Button
          size="sm"
          variant="ghost"
          onClick={() => setPreview((p) => !p)}
          aria-expanded={preview}
        >
          {t('estudo-o-que-vai', { provedor: provider ?? t('estudo-o-motor') })}
        </Button>
        <Button
          size="sm"
          disabled={!engineName || running}
          loading={start.isPending || running}
          onClick={() => {
            onStart();
            start.mutate(args, {
              onSuccess: (r) => setJobId(r.job_id),
              onError: (e: Error) =>
                toast({ tone: 'error', title: t('estudo-nao-gerou'), description: e.message }),
            });
          }}
        >
          {running ? t('estudo-gerando') : t('estudo-gerar')}
        </Button>
      </div>
      {preview ? <PackagePreview args={args} /> : null}
      {error && error.type === 'error' ? (
        <p role="alert" className="text-[12.5px] text-negative">
          {t('estudo-nao-gerou')}: {error.message}
        </p>
      ) : null}
      {job.data?.status === 'done' ? (
        <p className="text-[12.5px] text-positive">{t('estudo-pronto')}</p>
      ) : null}
    </div>
  );
}

interface GenerateProps {
  assetId: string;
  engineName: string | null;
  provider: string | null;
  /** Pedido enviado: o painel abre o estudo novo quando ele chegar no histórico. */
  onStart: () => void;
}

const CHECK_TONE: Record<NumberCheck['status'], string> = {
  ok: 'text-positive',
  mismatch: 'text-negative',
  not_found: 'text-warning',
};

function checkLabel(c: NumberCheck): string {
  if (c.status === 'ok') return t('estudo-numero-ok');
  if (c.status === 'mismatch')
    return c.expected != null
      ? t('estudo-numero-diferente', { valor: String(c.expected) })
      : t('estudo-numero-diferente-sem');
  return t('estudo-numero-nao-encontrado');
}

/** Texto do estudo para copiar, com o aviso completo no fim. */
export function studyText(s: Study): string {
  const o = s.output;
  const list = (title: string, items: string[]) =>
    items.length ? `\n${title}\n${items.map((x) => `- ${x}`).join('\n')}\n` : '';
  return [
    `${t('estudo-selo', { data: day(s.created_at), motor: s.engine_name })}`,
    '',
    o.summary,
    o.business ? `\n${o.business}` : '',
    list(
      t('estudo-fortes'),
      o.strengths.map((p) => p.text),
    ),
    list(
      t('estudo-riscos'),
      o.risks.map((p) => p.text),
    ),
    list(
      t('estudo-numeros'),
      o.key_numbers.map((n, i) => {
        const c = s.checks[i];
        return `${n.label}: ${n.value ?? '—'}${n.unit ? ` ${n.unit}` : ''}${c ? ` (${checkLabel(c)})` : ''}`;
      }),
    ),
    o.changes ? `\n${t('estudo-mudou')}\n${o.changes}\n` : '',
    o.context_check ? `\n${t('estudo-seu-contexto')}\n${o.context_check}\n` : '',
    list(
      t('estudo-conferir'),
      o.manual_checks.map((c) => (c.where ? `${c.item} (${c.where})` : c.item)),
    ),
    list(
      t('estudo-fontes'),
      o.sources.map((x) => (x.date ? `${x.name} (${x.date})` : x.name)),
    ),
    '',
    `${t('estudo-aviso-titulo')}. ${t('estudo-aviso-texto')}`,
  ]
    .filter((x) => x !== '')
    .join('\n');
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-1" aria-label={title}>
      <h4 className="label-caps text-fg">{title}</h4>
      {children}
    </section>
  );
}

function StudyView({ id, onClose }: { id: string; onClose: () => void }) {
  const q = useStudy(id);
  const del = useStudyDelete();
  const ask = useCompanion((s) => s.ask);
  if (q.isPending) return <LoadingBlock label={t('estudo-abrindo')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const s = q.data;
  const o = s.output;
  const points = (items: { text: string; data: string | null }[]) => (
    <ul className="flex list-disc flex-col gap-1 pl-5 text-[13px]">
      {items.map((p, i) => (
        <li key={i}>
          {p.text}
          {p.data ? <span className="text-[11px] text-dim"> · {p.data}</span> : null}
        </li>
      ))}
    </ul>
  );
  return (
    <article
      className="flex flex-col gap-4 rounded-sm border border-line p-4"
      aria-label={t('estudo-aberto')}
    >
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-sm border border-warning/50 px-1.5 py-px text-[11px] text-warning">
          {t('estudo-selo', { data: day(s.created_at), motor: s.engine_name })}
        </span>
        <span className="text-[12px] text-dim">
          {t('estudo-confianca', { valor: CONF_LABEL(s.confidence) })}
        </span>
        <span className="grow" />
        <Button
          size="sm"
          variant="ghost"
          onClick={() => {
            void navigator.clipboard?.writeText(studyText(s));
            toast({ tone: 'success', title: t('estudo-copiado') });
          }}
        >
          {t('estudo-copiar')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => ask(t('estudo-chat-pedido', { id: s.id, data: day(s.created_at) }))}
        >
          {t('estudo-continuar-chat')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          loading={del.isPending}
          onClick={() =>
            del.mutate(
              { id: s.id },
              {
                onSuccess: () => {
                  toast({ tone: 'success', title: t('estudo-apagado') });
                  onClose();
                },
              },
            )
          }
        >
          {t('estudo-apagar')}
        </Button>
        <Button size="sm" variant="ghost" onClick={onClose}>
          {t('estudo-fechar')}
        </Button>
      </div>
      {s.question ? <p className="text-[12.5px] text-dim">“{s.question}”</p> : null}
      <Section title={t('estudo-resumo')}>
        <p className="text-[13.5px] leading-relaxed">{o.summary}</p>
      </Section>
      {o.business ? (
        <Section title={t('estudo-negocio')}>
          <p className="text-[13px]">{o.business}</p>
        </Section>
      ) : null}
      {o.strengths.length ? (
        <Section title={t('estudo-fortes')}>{points(o.strengths)}</Section>
      ) : null}
      {o.risks.length ? <Section title={t('estudo-riscos')}>{points(o.risks)}</Section> : null}
      {o.key_numbers.length ? (
        <Section title={t('estudo-numeros')}>
          <ul className="flex flex-col gap-1 text-[13px]">
            {o.key_numbers.map((n, i) => {
              const c = s.checks[i];
              return (
                <li key={i} className="flex flex-wrap items-baseline gap-2">
                  <span className="text-dim">{n.label}</span>
                  <span className="num">
                    {n.value ?? '—'}
                    {n.unit ? ` ${n.unit}` : ''}
                  </span>
                  {c ? (
                    <span className={cx('text-[11px]', CHECK_TONE[c.status])}>{checkLabel(c)}</span>
                  ) : null}
                </li>
              );
            })}
          </ul>
        </Section>
      ) : null}
      {o.changes ? (
        <Section title={t('estudo-mudou')}>
          <p className="text-[13px]">{o.changes}</p>
        </Section>
      ) : null}
      {o.context_check ? (
        <Section title={t('estudo-seu-contexto')}>
          <p className="text-[13px]">{o.context_check}</p>
        </Section>
      ) : null}
      {o.manual_checks.length ? (
        <Section title={t('estudo-conferir')}>
          <ul className="flex list-disc flex-col gap-1 pl-5 text-[13px]">
            {o.manual_checks.map((c, i) => (
              <li key={i}>
                {c.item}
                {c.where ? <span className="text-dim"> · {c.where}</span> : null}
              </li>
            ))}
          </ul>
        </Section>
      ) : null}
      {o.sources.length ? (
        <Section title={t('estudo-fontes')}>
          <p className="text-[12.5px] text-dim">
            {o.sources.map((x) => (x.date ? `${x.name} (${x.date})` : x.name)).join(' · ')}
          </p>
        </Section>
      ) : null}
      {o.missing ? (
        <p className="text-[12px] text-dim">{t('estudo-faltou', { texto: o.missing })}</p>
      ) : null}
      <p className="border-t border-line pt-2 text-[11.5px] leading-snug text-dim">
        {t('estudo-aviso-curto')}
      </p>
    </article>
  );
}

function History({
  studies,
  onOpen,
}: {
  studies: readonly StudySummary[];
  onOpen: (id: string) => void;
}) {
  if (studies.length === 0) return <p className="text-[12.5px] text-dim">{t('estudo-nenhum')}</p>;
  return (
    <ul className="flex flex-col divide-y divide-line/60" aria-label={t('estudo-anteriores')}>
      {studies.map((s) => (
        <li key={s.id} className="flex flex-wrap items-center gap-2 py-1.5 text-[13px]">
          <span className="num">{formatDateTime(s.created_at)}</span>
          <span className="text-dim">· {s.engine_name}</span>
          <span className="text-dim">
            · {t('estudo-confianca', { valor: CONF_LABEL(s.confidence) })}
          </span>
          {s.unchecked > 0 ? (
            <span className="text-[12px] text-warning">
              · {t('estudo-nao-conferidos', { n: s.unchecked })}
            </span>
          ) : null}
          <span className="grow" />
          <Button size="sm" variant="ghost" onClick={() => onOpen(s.id)}>
            {t('estudo-abrir')}
          </Button>
        </li>
      ))}
    </ul>
  );
}

export function StudyPanel({ assetId }: { assetId: string }) {
  const q = useStudyState(assetId);
  // `?estudo=<id>` (aviso de estudo pronto, lista de tarefas) abre o estudo direto.
  const [params] = useSearchParams();
  const wanted = params.get('estudo');
  const [open, setOpen] = useState<string | null>(wanted);
  const [lastWanted, setLastWanted] = useState(wanted);
  if (wanted !== lastWanted) {
    setLastWanted(wanted);
    if (wanted) setOpen(wanted);
  }
  // Depois de gerar: abre o primeiro estudo do histórico que não estava lá antes.
  const top = q.data?.studies[0]?.id ?? '';
  const [waitingFrom, setWaitingFrom] = useState<string | null>(null);
  if (waitingFrom != null && top && top !== waitingFrom) {
    setWaitingFrom(null);
    setOpen(top);
  }
  const onStart = useCallback(() => setWaitingFrom(top), [top]);
  if (q.isPending) return <LoadingBlock label={t('estudo-titulo')} />;
  if (q.isError) return <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />;
  const s = q.data;
  return (
    <div className="flex flex-col gap-5">
      <StudyDisclaimer />
      {!s.accepted ? (
        <Accept />
      ) : (
        <>
          <ContextField
            assetId={assetId}
            initial={s.context ?? ''}
            key={s.context_updated_at ?? ''}
          />
          <Generate
            assetId={assetId}
            engineName={s.engine?.name ?? null}
            provider={s.engine?.provider_label ?? null}
            onStart={onStart}
          />
        </>
      )}
      {open ? <StudyView id={open} onClose={() => setOpen(null)} /> : null}
      <section className="flex flex-col gap-2" aria-label={t('estudo-anteriores')}>
        <h3 className="label-caps">{t('estudo-anteriores')}</h3>
        <History studies={s.studies} onOpen={setOpen} />
      </section>
    </div>
  );
}
