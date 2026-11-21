/**
 * Boas-vindas (docs/profile.md): escolher entre personalizar o app (as perguntas de múltipla
 * escolha do núcleo, com progresso e botão de voltar) ou ir direto para o modo avançado, sem
 * nenhuma pergunta. No fim, o nível escolhido e o que muda nele. Também usado para refazer o
 * questionário em Ajustes › Perfil. A última pergunta, sem pontos, liga a linha da vida
 * (docs/life-expectancy.md, seção 8): a chave é da família, então ela não aparece no
 * questionário de um membro.
 */
import { useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router';

import type { UserProfile } from '@/api/generated/UserProfile';
import {
  useAnswerProfile,
  useProfile,
  useProfileQuestions,
  useSkipProfile,
} from '@/api/hooks/profile';
import { LEVEL_DESCRIPTION, LEVEL_LABEL } from '@/app/profile';
import { useViewAs } from '@/app/viewAs';
import { Button, Icon } from '@/design/components';
import { cx } from '@/design/cx';
import { DemoQuickDialog } from '@/features/demo/DemoQuick';
import { tRich } from '@/features/settings/richText';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

/** Pergunta do núcleo que liga a linha da vida (sem pontos; "Sim / Não / Decidir depois"). */
const LONGEVITY_QUESTION = 'longevity';

function Welcome({ onStart, redo }: { onStart: () => void; redo: boolean }) {
  const skip = useSkipProfile();
  const { member } = useViewAs();
  const navigate = useNavigate();
  const [demo, setDemo] = useState(false);
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-2">
        <h1 className="text-[24px] font-medium text-fg">
          {member
            ? t('perfil-questionario-de', { nome: member.name })
            : redo
              ? t('perfil-refazer')
              : t('perfil-boas-vindas')}
        </h1>
        <p className="text-[14px] leading-relaxed text-dim">
          {`${t('perfil-boas-vindas-texto')} ${t('perfil-confianca')}`}
        </p>
      </div>
      <div className="grid gap-3 @min-[640px]:grid-cols-2">
        <button
          type="button"
          onClick={onStart}
          className="flex flex-col gap-1.5 rounded-card border border-accent bg-accent-soft/50 px-5 py-4 text-left transition-colors hover:bg-accent-soft focus-visible:outline-2 focus-visible:outline-accent"
        >
          <span className="text-[15px] font-medium text-fg">{t('perfil-personalizar')}</span>
          <span className="text-[12.5px] text-dim">{t('perfil-personalizar-dica')}</span>
        </button>
        <button
          type="button"
          disabled={skip.isPending}
          onClick={() =>
            skip.mutate(member ? { member_id: member.id } : {}, {
              onSuccess: () => void navigate('/painel', { replace: true }),
            })
          }
          className="flex flex-col gap-1.5 rounded-card border border-line px-5 py-4 text-left transition-colors hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-accent"
        >
          <span className="text-[15px] font-medium text-fg">{t('perfil-completo')}</span>
          <span className="text-[12.5px] text-dim">{t('perfil-completo-dica')}</span>
        </button>
      </div>
      {member ? null : (
        <p className="text-[13px] text-dim">
          {tRich('perfil-demo-convite', {
            botao: (
              <button
                type="button"
                onClick={() => setDemo(true)}
                className="text-demo underline-offset-4 hover:underline"
              >
                {t('perfil-demo-criar')}
              </button>
            ),
          })}
        </p>
      )}
      <DemoQuickDialog open={demo} onOpenChange={setDemo} />
      <InlineError error={skip.error} />
    </div>
  );
}

function Result({ profile }: { profile: UserProfile }) {
  const navigate = useNavigate();
  return (
    <div className="flex flex-col gap-5">
      <div className="flex items-center gap-3">
        <span className="flex size-10 items-center justify-center rounded-full bg-positive/15 text-positive">
          <Icon name="check" size={20} />
        </span>
        <h1 className="text-[22px] font-medium text-fg">
          {t('perfil-resultado', { nivel: LEVEL_LABEL[profile.level] })}
        </h1>
      </div>
      <p className="text-[14px] leading-relaxed text-dim">{LEVEL_DESCRIPTION[profile.level]}</p>
      <p className="text-[13px] text-dim">{t('perfil-resultado-dica')}</p>
      {profile.answers[LONGEVITY_QUESTION]?.[0] === 'yes' ? (
        <p className="text-[13px] text-dim">
          {t('longevidade-onboarding-ligada')}{' '}
          <button
            type="button"
            className="text-accent hover:underline"
            onClick={() => void navigate('/familia', { replace: true })}
          >
            {t('longevidade-onboarding-ir')}
          </button>
        </p>
      ) : null}
      <div>
        <Button variant="primary" onClick={() => void navigate('/painel', { replace: true })}>
          {t('perfil-comecar')}
        </Button>
      </div>
    </div>
  );
}

export function Onboarding({ redo = false }: { redo?: boolean }) {
  const questions = useProfileQuestions();
  const answer = useAnswerProfile();
  const { memberId } = useViewAs();
  const [step, setStep] = useState(-1);
  const [answers, setAnswers] = useState<Record<string, string[]>>({});
  const [done, setDone] = useState<UserProfile | null>(null);
  // A pergunta da linha da vida é só da família.
  const qs = (questions.data ?? []).filter((q) => !memberId || q.id !== LONGEVITY_QUESTION);

  if (done) return <Result profile={done} />;
  if (step < 0) return <Welcome redo={redo} onStart={() => setStep(0)} />;

  return (
    <QueryState loading={questions.isPending} error={questions.error}>
      {qs.length
        ? (() => {
            const q = qs[step]!;
            const picked = answers[q.id] ?? [];
            const last = step === qs.length - 1;
            const toggle = (id: string) =>
              setAnswers((a) => {
                const cur = a[q.id] ?? [];
                const next = q.multiple
                  ? cur.includes(id)
                    ? cur.filter((x) => x !== id)
                    : [...cur, id]
                  : [id];
                return { ...a, [q.id]: next };
              });
            const next = () => {
              if (!last) return setStep(step + 1);
              const filled = Object.fromEntries(
                Object.entries(answers).filter(([, v]) => v.length > 0),
              );
              answer.mutate(
                { answers: filled, ...(memberId ? { member_id: memberId } : {}) },
                { onSuccess: setDone },
              );
            };
            return (
              <div className="flex flex-col gap-5">
                <div className="flex flex-col gap-1.5">
                  <span className="text-[12.5px] text-dim">
                    {t('perfil-pergunta', { n: step + 1, total: qs.length })}
                  </span>
                  <div
                    role="progressbar"
                    aria-label={t('perfil-progresso')}
                    aria-valuemin={0}
                    aria-valuemax={qs.length}
                    aria-valuenow={step + 1}
                    className="h-1.5 w-full overflow-hidden rounded-full bg-surface-2"
                  >
                    <div
                      className="h-full rounded-full bg-accent transition-[width]"
                      style={{ width: `${((step + 1) / qs.length) * 100}%` }}
                    />
                  </div>
                </div>
                <fieldset className="flex flex-col gap-3">
                  <legend className="mb-1 text-[19px] font-medium text-fg">{q.title}</legend>
                  {q.hint ? <p className="text-[13px] text-dim">{q.hint}</p> : null}
                  <div
                    role={q.multiple ? 'group' : 'radiogroup'}
                    aria-label={q.title}
                    className="flex flex-col gap-2"
                  >
                    {q.options.map((o) => {
                      const on = picked.includes(o.id);
                      return (
                        <button
                          key={o.id}
                          type="button"
                          role={q.multiple ? 'checkbox' : 'radio'}
                          aria-checked={on}
                          onClick={() => toggle(o.id)}
                          className={cx(
                            'flex items-center gap-3 rounded-control border px-4 py-3 text-left text-[14px] transition-colors focus-visible:outline-2 focus-visible:outline-accent',
                            on
                              ? 'border-accent bg-accent-soft text-fg'
                              : 'border-line hover:bg-surface-2',
                          )}
                        >
                          <span
                            aria-hidden
                            className={cx(
                              'flex size-4 shrink-0 items-center justify-center border',
                              q.multiple ? 'rounded-[4px]' : 'rounded-full',
                              on ? 'border-accent bg-accent text-accent-ink' : 'border-line-strong',
                            )}
                          >
                            {on ? <Icon name="check" size={11} /> : null}
                          </span>
                          {o.label}
                        </button>
                      );
                    })}
                  </div>
                </fieldset>
                <p className="text-[12px] text-dim">{t('perfil-confianca')}</p>
                <InlineError error={answer.error} />
                <div className="flex items-center justify-between gap-3">
                  <Button variant="ghost" onClick={() => setStep(step - 1)}>
                    {t('comum-voltar')}
                  </Button>
                  <div className="flex gap-2">
                    {!picked.length ? (
                      <Button variant="ghost" onClick={next} loading={last && answer.isPending}>
                        {t('perfil-pular')}
                      </Button>
                    ) : null}
                    <Button
                      variant="primary"
                      disabled={!picked.length}
                      loading={last && answer.isPending}
                      onClick={next}
                    >
                      {last ? t('perfil-concluir') : t('comum-continuar')}
                    </Button>
                  </div>
                </div>
              </div>
            );
          })()
        : null}
    </QueryState>
  );
}

/** Tela das boas-vindas (rota `/boas-vindas`; `?refazer=1` em Ajustes › Perfil). */
export default function BoasVindas() {
  const profile = useProfile();
  const [params] = useSearchParams();
  const redo = params.has('refazer');
  return (
    <div className="mx-auto flex w-full max-w-[640px] flex-col gap-6 py-10">
      <QueryState loading={profile.isPending} error={profile.error}>
        <Onboarding redo={redo || profile.data?.onboarding_pending === false} />
      </QueryState>
    </div>
  );
}
