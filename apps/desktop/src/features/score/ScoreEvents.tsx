/**
 * Eventos qualitativos do score (§8.2 da metodologia): recuperação judicial, fraude,
 * litígio, desastre ambiental, governança, regulação. Só os aceitos mexem na nota (teto ou
 * penalidade que decaem com o tempo); os fatos relevantes da CVM aparecem como sugestões
 * e só viram evento quando o usuário aplica.
 */
import { useState, type FormEvent } from 'react';

import type { AssetScore } from '@/api/generated/AssetScore';
import type { AssetScoreArgs } from '@/api/generated/AssetScoreArgs';
import type { AssetScoreEvent } from '@/api/generated/AssetScoreEvent';
import type { AssetScoreEventStatus } from '@/api/generated/AssetScoreEventStatus';
import type { AssetScoreEventSuggestion } from '@/api/generated/AssetScoreEventSuggestion';
import {
  useAddScoreEvent,
  useDeleteScoreEvent,
  useScoreMethodology,
  useSetScoreEventStatus,
} from '@/api/hooks/score';
import {
  Button,
  DateInput,
  Dialog,
  DialogClose,
  Field,
  Input,
  InstrumentCard,
  Select,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { formatNumber } from '@/lib/format';
import { FormError } from '@/vault/FormError';
import { t } from '@/i18n';

import { safeDate } from '../portfolio/labels';
import { safeLink } from '../research/DocumentsPanel';

const STATUS = {
  pending: { label: 'score-evento-aguardando', tone: 'text-warn' },
  accepted: { label: 'score-evento-aceito', tone: 'text-neg' },
  rejected: { label: 'score-evento-rejeitado', tone: 'text-dim' },
  resolved: { label: 'score-evento-resolvido', tone: 'text-pos' },
} as const satisfies Record<AssetScoreEventStatus, { label: string; tone: string }>;

function effect(e: AssetScoreEvent): string | null {
  const parts: string[] = [];
  if (e.effective_cap != null) {
    parts.push(t('score-teto', { nota: formatNumber(e.effective_cap, 0) }));
  }
  if (e.effective_penalty != null && e.effective_penalty > 0) {
    parts.push(
      t('score-penalidade', {
        n: e.effective_penalty,
        pontos: `−${formatNumber(e.effective_penalty, 1)}`,
      }),
    );
  }
  return parts.length ? parts.join(' · ') : null;
}

function Evidence({ url }: { url: string | null }) {
  const href = url ? safeLink(url) : null;
  return href ? (
    <a
      href={href}
      target="_blank"
      rel="noreferrer noopener"
      className="text-accent underline-offset-2 hover:underline"
    >
      {t('score-evidencia')}
    </a>
  ) : null;
}

function EventRow({ e }: { e: AssetScoreEvent }) {
  const setStatus = useSetScoreEventStatus();
  const del = useDeleteScoreEvent();
  const st = STATUS[e.status];
  const fx = effect(e);
  const act = (status: AssetScoreEventStatus) => setStatus.mutate({ id: e.id, status });
  return (
    <li className="flex flex-col gap-1.5 rounded-control border border-line px-3 py-2 text-[12.5px]">
      <span className="flex flex-wrap items-center gap-2">
        <span className="label-caps text-fg">{e.kind_label}</span>
        <span className="num text-dim">{safeDate(e.event_date)}</span>
        <span className={cx('label-caps', st.tone)}>{t(st.label)}</span>
        {fx && e.status === 'accepted' ? (
          <span className="num text-dim">{t('score-evento-hoje', { efeito: fx })}</span>
        ) : null}
        <Evidence url={e.evidence_url} />
      </span>
      <span className="text-fg">{e.title}</span>
      {e.note ? <span className="text-dim">{e.note}</span> : null}
      <span className="flex flex-wrap gap-2">
        {e.status === 'pending' ? (
          <>
            <Button size="sm" onClick={() => act('accepted')} loading={setStatus.isPending}>
              {t('score-aceitar')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => act('rejected')}>
              {t('score-rejeitar')}
            </Button>
          </>
        ) : null}
        {e.status === 'accepted' ? (
          <Button size="sm" variant="ghost" onClick={() => act('resolved')}>
            {t('score-marcar-resolvido')}
          </Button>
        ) : null}
        {e.status === 'rejected' || e.status === 'resolved' ? (
          <Button size="sm" variant="ghost" onClick={() => act('accepted')}>
            {t('score-voltar-valer')}
          </Button>
        ) : null}
        <Button
          size="sm"
          variant="ghost"
          onClick={() => del.mutate({ id: e.id })}
          aria-label={t('score-apagar-evento', { titulo: e.title })}
        >
          {t('comum-apagar')}
        </Button>
      </span>
    </li>
  );
}

function SuggestionRow({ s, subject }: { s: AssetScoreEventSuggestion; subject: AssetScoreArgs }) {
  const add = useAddScoreEvent();
  const apply = () =>
    add.mutate(
      {
        ...subject,
        kind: s.kind,
        title: s.title,
        event_date: s.event_date,
        evidence_url: s.evidence_url,
        source: 'cvm_ipe',
        accepted: true,
      },
      { onSuccess: () => toast({ tone: 'success', title: t('score-evento-aplicado') }) },
    );
  return (
    <li className="flex flex-col gap-1 rounded-control border border-dashed border-line px-3 py-2 text-[12.5px]">
      <span className="flex flex-wrap items-center gap-2 text-dim">
        <span className="label-caps text-fg">{s.kind_label}</span>
        <span className="num">{safeDate(s.event_date)}</span>
        <span>· {s.source}</span>
        <span>{t('score-casou', { termo: s.matched })}</span>
        <Evidence url={s.evidence_url} />
      </span>
      <span className="text-fg">{s.title}</span>
      <span>
        <Button size="sm" onClick={apply} loading={add.isPending}>
          {t('score-aplicar')}
        </Button>
      </span>
    </li>
  );
}

function EventForm({ subject, onDone }: { subject: AssetScoreArgs; onDone: () => void }) {
  const methodology = useScoreMethodology();
  const kinds = (methodology.data?.event_kinds ?? []).map(([value, label]) => ({ value, label }));
  const add = useAddScoreEvent();
  const [kind, setKind] = useState('outro');
  const [title, setTitle] = useState('');
  const [date, setDate] = useState<string | null>(null);
  const [link, setLink] = useState('');
  const [note, setNote] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (!title.trim() || !date) return;
    add.mutate(
      {
        ...subject,
        kind,
        title: title.trim(),
        event_date: date,
        ...(link.trim() ? { evidence_url: link.trim() } : {}),
        ...(note.trim() ? { note: note.trim() } : {}),
      },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title: t('score-evento-registrado'),
            description: t('score-nota-refeita'),
          });
          onDone();
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      noValidate
      className="flex flex-col gap-4"
      aria-label={t('score-novo-evento')}
    >
      <Field label={t('score-evento-tipo')} hint={t('score-evento-tipo-dica')}>
        <Select value={kind} onValueChange={setKind} options={kinds} />
      </Field>
      <Field
        label={t('score-evento-o-que')}
        error={submitted && !title.trim() ? t('score-evento-descreva') : undefined}
      >
        <Input value={title} maxLength={300} onChange={(e) => setTitle(e.target.value)} />
      </Field>
      <Field
        label={t('score-data')}
        error={submitted && !date ? t('score-informe-data') : undefined}
      >
        <DateInput value={date} onValueChange={setDate} />
      </Field>
      <Field label={t('score-evidencia-link')} hint={t('score-evidencia-dica')}>
        <Input
          value={link}
          placeholder="https://"
          onChange={(e) => setLink(e.target.value)}
          inputMode="url"
        />
      </Field>
      <Field label={t('score-observacao')} hint={t('score-opcional')}>
        <Input value={note} onChange={(e) => setNote(e.target.value)} />
      </Field>
      <FormError error={add.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={add.isPending}>
          {t('score-registrar-aplicar')}
        </Button>
      </div>
    </form>
  );
}

export function ScoreEventsSection({
  subject,
  score,
  index,
}: {
  subject: AssetScoreArgs;
  score: AssetScore;
  index: string;
}) {
  const [open, setOpen] = useState(false);
  const events = score.events ?? [];
  const suggestions = score.event_suggestions ?? [];
  return (
    <InstrumentCard index={index} title={t('score-eventos-titulo')} headingLevel={3}>
      <div className="flex flex-col gap-3">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <p className="max-w-[60ch] text-[12.5px] text-dim">{t('score-eventos-ajuda')}</p>
          <Button onClick={() => setOpen(true)}>{t('score-novo-evento')}</Button>
        </div>
        {events.length > 0 ? (
          <ul aria-label={t('score-eventos-registrados')} className="flex flex-col gap-2">
            {events.map((e) => (
              <EventRow key={e.id} e={e} />
            ))}
          </ul>
        ) : (
          <p className="text-[12.5px] text-dim">{t('score-nenhum-evento')}</p>
        )}
        {suggestions.length > 0 ? (
          <div className="flex flex-col gap-2">
            <h4 className="label-caps text-fg">{t('score-sugestoes-cvm')}</h4>
            <ul aria-label={t('score-sugestoes')} className="flex flex-col gap-2">
              {suggestions.map((s) => (
                <SuggestionRow key={s.evidence_url} s={s} subject={subject} />
              ))}
            </ul>
          </div>
        ) : null}
      </div>
      <Dialog
        open={open}
        onOpenChange={setOpen}
        size="md"
        index="08"
        title={t('score-novo-evento-de', { nome: score.ticker ?? score.name })}
        description={t('score-novo-evento-descricao')}
      >
        {open ? <EventForm subject={subject} onDone={() => setOpen(false)} /> : null}
      </Dialog>
    </InstrumentCard>
  );
}
