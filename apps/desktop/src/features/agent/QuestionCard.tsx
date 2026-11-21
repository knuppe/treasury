/**
 * Pergunta do agente à pessoa (`ask_user`), no fio da conversa: as opções que o agente
 * definiu (escolha única ou múltipla), a recomendada já marcada e, quando o agente deixa,
 * "Outro" com texto livre. O agente para até a resposta (`ai.question.answer`). Depois do
 * envio (ou do evento `user_answer`, que também chega quando o tempo acaba), o cartão mostra
 * só um resumo do que foi respondido.
 */
import { Fragment, useId, useRef, useState, type ReactNode } from 'react';

import type { AskAnswer } from '@/api/generated/AskAnswer';
import { useAnswerQuestion } from '@/api/hooks/ai';
import { Button, Icon, Input, Private } from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';

import type { StreamPart } from './chatStream';
import { Pill } from './ui';

type QuestionPart = Extract<StreamPart, { type: 'question' }>;

/** O que foi respondido, numa linha (o texto livre fica sob o modo privado). */
function answerText(a: AskAnswer): ReactNode {
  const items: ReactNode[] = a.selected.map((label) => label);
  if (a.other) items.push(<Private>{a.other}</Private>);
  return items.map((item, i) => (
    <Fragment key={i}>
      {i > 0 ? ', ' : null}
      {item}
    </Fragment>
  ));
}

export function QuestionCard({
  part,
  running,
}: {
  part: QuestionPart;
  /** O job ainda roda (sem ele, uma pergunta sem resposta não aceita mais envio). */
  running: boolean;
}) {
  const q = part.question;
  const multi = q.multi_select;
  const recommended =
    q.recommended != null && q.recommended < q.options.length ? q.recommended : null;
  const send = useAnswerQuestion();
  const [selected, setSelected] = useState<string[]>(
    recommended != null ? [q.options[recommended]!.label] : [],
  );
  const [otherOn, setOtherOn] = useState(false);
  const [otherText, setOtherText] = useState('');
  const [sent, setSent] = useState<AskAnswer | null>(null);
  const [error, setError] = useState<string | null>(null);
  const otherRef = useRef<HTMLInputElement>(null);
  const id = useId();
  // `undefined`: ainda à espera; `null`: ficou sem resposta.
  const final: AskAnswer | null | undefined =
    part.answer !== undefined ? part.answer : (sent ?? (running ? undefined : null));

  const other = otherText.trim();
  const valid = multi
    ? (selected.length > 0 || otherOn) && (!otherOn || other !== '')
    : otherOn
      ? other !== ''
      : selected.length === 1;

  const choose = (label: string, on: boolean) => {
    if (multi) {
      setSelected((cur) =>
        on ? (cur.includes(label) ? cur : [...cur, label]) : cur.filter((x) => x !== label),
      );
    } else if (on) {
      setSelected([label]);
      setOtherOn(false);
    }
  };
  const chooseOther = (on: boolean, focus = true) => {
    setOtherOn(on);
    if (on && !multi) setSelected([]);
    if (on && focus) otherRef.current?.focus();
  };
  const submit = () => {
    if (!valid || send.isPending) return;
    setError(null);
    const answer: AskAnswer = { selected, other: otherOn ? other : null };
    send.mutate(
      { request_id: part.requestId, selected, ...(otherOn ? { other } : {}) },
      {
        onSuccess: () => setSent(answer),
        onError: (e) => setError(errorMessage(e)),
      },
    );
  };

  const inputType = multi ? 'checkbox' : 'radio';
  const name = `${id}-opcao`;
  return (
    <div
      role="group"
      aria-label={t('agente-pergunta-rotulo')}
      className="flex flex-col gap-2 rounded-control border border-l-2 border-line border-l-accent bg-surface-2 px-3 py-2.5 text-[12.5px]"
    >
      {final !== undefined ? (
        <>
          <p className="flex items-start gap-2 text-fg">
            <Icon name="info" size={12} className="mt-0.5 shrink-0 text-accent" />
            <span>{q.question}</span>
          </p>
          <p role="status" className="text-[12px] text-dim">
            {final
              ? tRich('agente-pergunta-respondeu', {
                  resposta: <span className="text-fg">{answerText(final)}</span>,
                })
              : t('agente-pergunta-sem-resposta')}
          </p>
        </>
      ) : (
        <form
          className="flex flex-col gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          <fieldset className="flex min-w-0 flex-col gap-1.5" aria-describedby={`${id}-dica`}>
            <legend className="mb-1 flex items-start gap-2 text-fg">
              <Icon name="info" size={12} className="mt-0.5 shrink-0 text-accent" />
              <span>{q.question}</span>
            </legend>
            <p id={`${id}-dica`} className="text-[11.5px] text-dim">
              {multi ? t('agente-pergunta-varias') : t('agente-pergunta-uma')}
            </p>
            {q.options.map((o, i) => {
              const descId = o.description ? `${id}-desc-${i}` : undefined;
              return (
                <label
                  key={o.label}
                  className="flex cursor-pointer items-start gap-2 rounded-control px-1.5 py-1 hover:bg-surface"
                >
                  <input
                    type={inputType}
                    name={name}
                    className="mt-0.5 size-3.5 shrink-0 accent-[var(--accent)]"
                    checked={selected.includes(o.label)}
                    aria-describedby={descId}
                    disabled={!running}
                    onChange={(e) => choose(o.label, e.target.checked)}
                  />
                  <span className="flex min-w-0 flex-col gap-0.5">
                    <span className="flex flex-wrap items-center gap-1.5 text-fg">
                      {o.label}
                      {i === recommended ? (
                        <Pill tone="accent">{t('agente-pergunta-recomendado')}</Pill>
                      ) : null}
                    </span>
                    {o.description ? (
                      <span id={descId} className="text-[12px] leading-snug text-dim">
                        {o.description}
                      </span>
                    ) : null}
                  </span>
                </label>
              );
            })}
            {q.allow_other ? (
              <div className="flex flex-col gap-1 px-1.5 py-1">
                <label className="flex cursor-pointer items-center gap-2 text-fg">
                  <input
                    type={inputType}
                    name={name}
                    className="size-3.5 shrink-0 accent-[var(--accent)]"
                    checked={otherOn}
                    disabled={!running}
                    onChange={(e) => chooseOther(e.target.checked)}
                  />
                  {t('agente-pergunta-outro')}
                </label>
                <Input
                  ref={otherRef}
                  wrapperClassName="ml-5.5 h-8 w-auto"
                  aria-label={t('agente-pergunta-outro-texto')}
                  placeholder={t('agente-pergunta-outro-dica')}
                  value={otherText}
                  maxLength={500}
                  disabled={!running}
                  onChange={(e) => {
                    setOtherText(e.target.value);
                    if (e.target.value.trim() && !otherOn) chooseOther(true, false);
                  }}
                />
              </div>
            ) : null}
          </fieldset>
          {running ? (
            <div className="flex flex-wrap items-center gap-2">
              <Button
                type="submit"
                size="sm"
                variant="primary"
                loading={send.isPending}
                disabled={!valid || send.isPending}
              >
                {t('agente-pergunta-enviar')}
              </Button>
              <span className="text-[12px] text-dim">{t('agente-pergunta-aguardando')}</span>
            </div>
          ) : null}
        </form>
      )}
      {error ? (
        <p role="alert" className="text-[12px] text-negative">
          {`${t('agente-pergunta-nao-enviou')}: ${error}`}
        </p>
      ) : null}
    </div>
  );
}
