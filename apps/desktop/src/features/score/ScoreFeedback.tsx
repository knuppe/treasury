/**
 * "Discordo" do score: veredito, faixa esperada (opcional) e motivo vão para
 * `score.feedback.add` (ajudam a calibrar a metodologia); abaixo, o que já foi registrado
 * para este ativo.
 */
import { useState, type FormEvent } from 'react';

import type { AssetScore } from '@/api/generated/AssetScore';
import type { AssetScoreArgs } from '@/api/generated/AssetScoreArgs';
import { useAddScoreFeedback, useScoreFeedback } from '@/api/hooks/score';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Select,
  toast,
  useFieldControl,
} from '@/design/components';
import { formatDateTime } from '@/lib/format';
import { FormError } from '@/vault/FormError';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';

import { Segmented } from '../portfolio/components/Segmented';
import { BAND, BAND_ORDER, bandKey, formatScore } from './format';
import { BandChip } from './parts';

function when(instant: string): string {
  try {
    return formatDateTime(instant);
  } catch {
    return instant;
  }
}

type Verdict = 'discordo' | 'concordo';

const verdicts = () => [
  { value: 'discordo' as const, label: t('score-discordo') },
  { value: 'concordo' as const, label: t('score-concordo') },
];

const NO_BAND = 'nenhuma';
const bandOptions = () => [
  { value: NO_BAND, label: t('score-sem-opiniao') },
  ...BAND_ORDER.map((k) => ({ value: k, label: BAND[k].label })),
];

function ReasonInput({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const a11y = useFieldControl({});
  return (
    <textarea
      id={a11y.id}
      aria-describedby={a11y['aria-describedby']}
      aria-invalid={a11y['aria-invalid']}
      rows={4}
      value={value}
      maxLength={2000}
      onChange={(e) => onChange(e.target.value)}
      className="w-full resize-y rounded-control border border-line-strong bg-bg px-2.5 py-2 text-[13px] text-fg outline-none focus:border-accent"
    />
  );
}

function FeedbackForm({ subject, onDone }: { subject: AssetScoreArgs; onDone: () => void }) {
  const add = useAddScoreFeedback();
  const [verdict, setVerdict] = useState<Verdict>('discordo');
  const [band, setBand] = useState<string>(NO_BAND);
  const [reason, setReason] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (!reason.trim()) return;
    add.mutate(
      {
        ...subject,
        verdict,
        ...(band !== NO_BAND ? { expected_band: band } : {}),
        reason: reason.trim(),
      },
      {
        onSuccess: () => {
          toast({
            tone: 'success',
            title: t('score-opiniao-registrada'),
            description: t('score-opiniao-registrada-dica'),
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
      aria-label={t('score-opiniao-form')}
    >
      <div className="flex flex-col gap-1.5">
        <span className="label-caps">{t('score-veredito')}</span>
        <Segmented
          label={t('score-veredito')}
          value={verdict}
          onChange={setVerdict}
          options={verdicts()}
        />
      </div>
      <Field label={t('score-faixa-esperada')} hint={t('score-opcional')}>
        <Select value={band} onValueChange={setBand} options={bandOptions()} />
      </Field>
      <Field
        label={t('score-motivo')}
        hint={t('score-motivo-dica')}
        error={submitted && !reason.trim() ? t('score-motivo-erro') : undefined}
      >
        <ReasonInput value={reason} onChange={setReason} />
      </Field>
      <FormError error={add.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={add.isPending}>
          {t('score-registrar-opiniao')}
        </Button>
      </div>
    </form>
  );
}

export function ScoreFeedbackSection({
  subject,
  score,
}: {
  subject: AssetScoreArgs;
  score: AssetScore;
}) {
  const [open, setOpen] = useState(false);
  const list = useScoreFeedback(subject);
  const items = list.data ?? [];
  const name = score.ticker ?? score.name;
  return (
    <section aria-label={t('score-sua-opiniao-sobre')} className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h3 className="label-caps text-fg">{t('score-sua-opiniao')}</h3>
          <p className="text-[12.5px] text-dim">{t('score-sua-opiniao-dica')}</p>
        </div>
        <Button onClick={() => setOpen(true)}>{t('score-discordo')}</Button>
      </div>
      {items.length > 0 ? (
        <ul aria-label={t('score-opinioes-registradas')} className="flex flex-col gap-2">
          {items.map((f) => (
            <li
              key={f.id}
              className="flex flex-col gap-1 rounded-control border border-line px-3 py-2 text-[12.5px]"
            >
              <span className="flex flex-wrap items-center gap-2 text-dim">
                <span className="label-caps text-fg">
                  {f.verdict === 'concordo' ? t('score-concordo') : t('score-discordo')}
                </span>
                <span className="num">{when(f.created_at)}</span>
                <span>
                  {t('score-opiniao-nota', {
                    nota: formatScore(f.score),
                    faixa: BAND[bandKey(f.band)].label,
                    versao: f.methodology_version,
                  })}
                </span>
                {f.expected_band ? (
                  <span className="flex items-center gap-1">
                    {withSlots(t('score-esperava', { faixa: slot(0) }), [
                      <BandChip key="faixa" band={f.expected_band} />,
                    ])}
                  </span>
                ) : null}
              </span>
              <span className="text-fg">{f.reason}</span>
            </li>
          ))}
        </ul>
      ) : null}
      <Dialog
        open={open}
        onOpenChange={setOpen}
        size="md"
        index="07"
        title={t('score-discordo-titulo', { nome: name })}
        description={t('score-discordo-descricao', {
          nota: formatScore(score.score),
          faixa: score.band_label,
          versao: score.methodology_version,
        })}
      >
        {open ? <FeedbackForm subject={subject} onDone={() => setOpen(false)} /> : null}
      </Dialog>
    </section>
  );
}
