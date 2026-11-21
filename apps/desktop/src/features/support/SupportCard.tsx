/**
 * Lembrete de apoio no fim da Visão geral (docs/support.md), no estilo das percepções. Só
 * aparece depois de um evento de valor e quando as regras do núcleo deixam (nunca modal, nunca
 * ao destravar, nunca com alerta grave, "Ver como" ou cofre de demonstração). "Já apoio" e
 * "Não mostrar mais" sempre à vista.
 */
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router';

import type { SupportChoice } from '@/api/generated/SupportChoice';
import { useProfile } from '@/api/hooks/profile';
import {
  markSupportSeen,
  useSupportInfo,
  useSupportPrompt,
  useSupportRespond,
} from '@/api/hooks/support';
import { Button, Icon } from '@/design/components';
import { t } from '@/i18n';

import { askText, thanksText } from './texts';

export function SupportCard({ viewAs }: { viewAs: boolean }) {
  const prompt = useSupportPrompt(viewAs);
  const info = useSupportInfo();
  const level = useProfile().data?.level;
  const respond = useSupportRespond();
  const navigate = useNavigate();
  const variant = viewAs ? null : prompt.data?.variant;
  const counted = useRef(false);

  useEffect(() => {
    if (variant && !counted.current) {
      counted.current = true;
      void markSupportSeen();
    }
  }, [variant]);

  if (!variant) return null;
  const answer = (choice: SupportChoice) => respond.mutate({ choice });
  const how = () => {
    answer('how');
    void navigate('/ajustes?secao=sobre');
  };
  return (
    <section
      aria-label={t('apoio-titulo')}
      className="flex flex-col gap-2 rounded-card border border-line bg-surface px-4 py-3.5"
    >
      <div className="flex items-center gap-2">
        <Icon name="info" size={14} className="text-accent" />
        <h2 className="text-[13px] text-dim">{t('apoio-titulo')}</h2>
      </div>
      <p className="text-[13.5px] text-fg">
        {variant === 'thanks' ? thanksText() : askText(level)}
      </p>
      {variant === 'ask' ? (
        <p className="text-[12.5px] text-dim">— {info.data?.channels.signature ?? 'Knuppe'}</p>
      ) : null}
      <div className="flex flex-wrap gap-2 pt-0.5">
        <Button size="sm" variant="secondary" onClick={how}>
          {t('apoio-como-apoiar')}
        </Button>
        {variant === 'ask' ? (
          <>
            <Button size="sm" variant="ghost" onClick={() => answer('supporter')}>
              {t('apoio-ja-apoio')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => answer('later')}>
              {t('apoio-agora-nao')}
            </Button>
          </>
        ) : (
          <Button size="sm" variant="ghost" onClick={() => answer('supporter')}>
            {t('apoio-agora-nao')}
          </Button>
        )}
        <Button size="sm" variant="ghost" onClick={() => answer('never')}>
          {t('apoio-nao-mostrar-mais')}
        </Button>
      </div>
    </section>
  );
}
