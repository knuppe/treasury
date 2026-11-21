/**
 * Ajustes › Perfil (docs/profile.md): nível, linguagem, o questionário de boas-vindas e a linha
 * da vida (docs/life-expectancy.md).
 */
import { useNavigate } from 'react-router';

import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import type { ProfileTone } from '@/api/generated/ProfileTone';
import { useProfile, useProfileQuestions, useSetProfile } from '@/api/hooks/profile';
import { LEVEL_DESCRIPTION, LEVEL_LABEL } from '@/app/profile';
import { useViewAs } from '@/app/viewAs';
import { Button, InstrumentCard, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { LongevitySettingsCard } from '@/features/longevity/LongevitySettingsCard';
import { InlineError, QueryState, Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { useUi, type HelpTips } from '@/state/ui';

import { tRich } from './richText';

const LEVELS: ProfileLevel[] = ['iniciante', 'intermediario', 'avancado'];
const tones = (): { value: ProfileTone; label: string }[] => [
  { value: 'simples', label: t('ajustes-perfil-tom-simples') },
  { value: 'equilibrado', label: t('ajustes-perfil-tom-equilibrado') },
  { value: 'tecnico', label: t('ajustes-perfil-tom-tecnico') },
];
const helpTips = (): { value: HelpTips; label: string }[] => [
  { value: 'auto', label: t('ajustes-perfil-dicas-nivel') },
  { value: 'on', label: t('ajustes-perfil-dicas-sempre') },
  { value: 'off', label: t('ajustes-perfil-dicas-nunca') },
];
/** De onde veio o nível atual (`source` do perfil). */
const SOURCE = {
  get questionario() {
    return t('ajustes-perfil-origem-questionario');
  },
  get pulou() {
    return t('ajustes-perfil-origem-pulou');
  },
  get escolha() {
    return t('ajustes-perfil-origem-escolha');
  },
  get padrao() {
    return t('ajustes-perfil-origem-padrao');
  },
  get familia() {
    return t('ajustes-perfil-origem-familia');
  },
};

export function ProfileSection() {
  const { memberId, member } = useViewAs();
  const profile = useProfile(memberId);
  const mine = memberId ? { member_id: memberId } : {};
  const questions = useProfileQuestions();
  const save = useSetProfile();
  const navigate = useNavigate();
  const helpTipsValue = useUi((s) => s.helpTips);
  const setHelpTips = useUi((s) => s.setHelpTips);
  return (
    <QueryState
      loading={profile.isPending}
      error={profile.error}
      label={t('ajustes-perfil-carregando')}
    >
      {profile.data ? (
        <div className="flex flex-col gap-5">
          <InstrumentCard
            index="12.P"
            title={
              member
                ? t('ajustes-perfil-nivel-de', { nome: member.name })
                : t('ajustes-perfil-nivel')
            }
          >
            <div className="flex flex-col gap-3">
              <p className="text-[12.5px] leading-snug text-dim">
                {`${t('ajustes-perfil-texto')} ${
                  profile.data.updated_at
                    ? t('ajustes-perfil-atual-em', {
                        nivel: LEVEL_LABEL[profile.data.level],
                        origem: SOURCE[profile.data.source],
                        data: formatDateTime(profile.data.updated_at),
                      })
                    : t('ajustes-perfil-atual', {
                        nivel: LEVEL_LABEL[profile.data.level],
                        origem: SOURCE[profile.data.source],
                      })
                }`}
              </p>
              <div
                role="radiogroup"
                aria-label={t('ajustes-perfil-nivel')}
                className="flex flex-col gap-2"
              >
                {LEVELS.map((l) => {
                  const on = profile.data.level === l;
                  return (
                    <button
                      key={l}
                      type="button"
                      role="radio"
                      aria-checked={on}
                      disabled={save.isPending}
                      onClick={() =>
                        !on &&
                        save.mutate(
                          { level: l, ...mine },
                          {
                            onSuccess: () =>
                              toast({
                                tone: 'success',
                                title: t('ajustes-perfil-modo', { nivel: LEVEL_LABEL[l] }),
                              }),
                          },
                        )
                      }
                      className={cx(
                        'flex flex-col gap-0.5 rounded-control border px-3.5 py-2.5 text-left transition-colors focus-visible:outline-2 focus-visible:outline-accent',
                        on ? 'border-accent bg-accent-soft' : 'border-line hover:bg-surface-2',
                      )}
                    >
                      <span className={cx('text-[13.5px]', on ? 'text-accent' : 'text-fg')}>
                        {LEVEL_LABEL[l]}
                      </span>
                      <span className="text-[12px] leading-snug text-dim">
                        {LEVEL_DESCRIPTION[l]}
                      </span>
                    </button>
                  );
                })}
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <span className="label-caps">{t('ajustes-perfil-linguagem')}</span>
                <Segmented
                  label={t('ajustes-perfil-linguagem')}
                  value={profile.data.tone}
                  onChange={(tone) => save.mutate({ tone, ...mine })}
                  options={tones()}
                />
                <span className="text-[12px] text-dim">{t('ajustes-perfil-linguagem-dica')}</span>
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <span className="label-caps">{t('ajustes-perfil-explicacoes')}</span>
                <Segmented
                  label={t('ajustes-perfil-explicacoes')}
                  value={helpTipsValue}
                  onChange={setHelpTips}
                  options={helpTips()}
                />
                <span className="text-[12px] text-dim">
                  {tRich('ajustes-perfil-explicacoes-dica', {
                    icone: <span aria-hidden>{'?'}</span>,
                  })}
                </span>
              </div>
              <InlineError error={save.error} />
              <div>
                <Button variant="secondary" onClick={() => void navigate('/boas-vindas?refazer=1')}>
                  {t('perfil-refazer')}
                </Button>
              </div>
            </div>
          </InstrumentCard>
          {Object.keys(profile.data.answers).length && questions.data ? (
            <InstrumentCard index="12.P2" title={t('ajustes-perfil-respostas')}>
              <dl className="flex flex-col gap-2 text-[12.5px]">
                {questions.data
                  .filter((q) => profile.data.answers[q.id]?.length)
                  .map((q) => (
                    <div key={q.id}>
                      <dt className="text-dim">{q.title}</dt>
                      <dd className="text-fg">
                        {profile.data.answers[q.id]!.map(
                          (a) => q.options.find((o) => o.id === a)?.label ?? a,
                        ).join('; ')}
                      </dd>
                    </div>
                  ))}
              </dl>
            </InstrumentCard>
          ) : null}
          <LongevitySettingsCard />
        </div>
      ) : null}
    </QueryState>
  );
}
