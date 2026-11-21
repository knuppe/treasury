/**
 * 02 · Família › Linha da vida (docs/life-expectancy.md, seção 6.3): o quadro com as barras
 * de todos os membros, só com a função ligada. Pede o que falta (data de nascimento e sexo, com
 * "prefiro não informar") e, em "Ver como", mostra só a barra desse membro.
 */
import { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router';

import type { Member } from '@/api/generated/Member';
import type { MemberSex } from '@/api/generated/MemberSex';
import type { MissingLifeData } from '@/api/generated/MissingLifeData';
import { useMembers, useUpdateMember } from '@/api/hooks/household';
import { useFamilyOutlook, useLifeStages, useLongevitySettings } from '@/api/hooks/longevity';
import { useViewAs } from '@/app/viewAs';
import { Button, DateInput, Field, InstrumentCard, Select, toast } from '@/design/components';
import { InlineError, QueryState } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { FamilyLifeBars } from './FamilyLifeBars';
import { LifeMethod } from './LifeBar';
import { listText } from './model';
import { sexOptions } from './sex';
import { useLifeView } from './useLifeView';

const CARD_ID = 'linha-da-vida';

function MissingRow({ item, member }: { item: MissingLifeData; member: Member }) {
  const save = useUpdateMember();
  const [birth, setBirth] = useState<string | null>(member.profile.birth_date);
  const [sex, setSex] = useState<MemberSex | ''>(member.profile.sex ?? '');
  const needBirth = item.missing.includes('birth_date');
  const needSex = item.missing.includes('sex');
  const items = [
    ...(needBirth ? [t('longevidade-falta-nascimento')] : []),
    ...(needSex ? [t('longevidade-falta-sexo')] : []),
  ];
  const changed = (needBirth && birth) || (needSex && sex);
  return (
    <li className="flex flex-wrap items-end gap-3 py-2">
      <span className="min-w-48 flex-1 self-center text-[13px] text-fg">
        {t('longevidade-falta-de', { nome: item.name, itens: listText(items) })}
      </span>
      {needBirth ? (
        <Field label={t('longevidade-falta-nascimento-de', { nome: item.name })} className="w-40">
          <DateInput value={birth} onValueChange={setBirth} />
        </Field>
      ) : null}
      {needSex ? (
        <Field label={t('longevidade-falta-sexo-de', { nome: item.name })} className="w-48">
          <Select
            value={sex}
            onValueChange={(v) => setSex(v as MemberSex | '')}
            options={sexOptions()}
            placeholder={t('familia-sexo-nao-informado')}
          />
        </Field>
      ) : null}
      <Button
        size="sm"
        variant="secondary"
        disabled={!changed}
        loading={save.isPending}
        onClick={() =>
          save.mutate(
            {
              id: member.id,
              profile: { ...member.profile, birth_date: birth, sex: sex || member.profile.sex },
            },
            {
              onSuccess: () =>
                toast({
                  tone: 'success',
                  title: t('longevidade-falta-salvo', { nome: member.name }),
                }),
            },
          )
        }
      >
        {t('comum-salvar')}
      </Button>
      <InlineError error={save.error} />
    </li>
  );
}

export function FamilyLifeCard() {
  const settings = useLongevitySettings().data;
  const enabled = settings?.enabled === true;
  const family = useFamilyOutlook(enabled);
  const stages = useLifeStages().data;
  const members = useMembers().data ?? [];
  const { memberId, member } = useViewAs();
  const view = useLifeView();
  const [params] = useSearchParams();
  // A barra ⌘K leva até aqui (`/familia?ver=linha-da-vida`).
  const jump = params.get('ver') === 'linha-da-vida';
  const ready = enabled && family.data != null;
  useEffect(() => {
    if (jump && ready) document.getElementById(CARD_ID)?.scrollIntoView?.({ block: 'start' });
  }, [jump, ready]);
  if (!enabled) return null;
  const f = family.data;
  const hidden = new Set(settings?.hidden_members ?? []);
  const shown = (f?.members ?? []).filter(
    (o) => !hidden.has(o.member_id) && (!memberId || o.member_id === memberId),
  );
  const missing = (f?.missing ?? []).filter((m) => !memberId || m.member_id === memberId);
  return (
    <InstrumentCard id={CARD_ID} index="02.L" title={t('longevidade-titulo')}>
      <QueryState loading={family.isPending} error={family.error}>
        {f ? (
          <div className="flex flex-col gap-4">
            <p className="max-w-[80ch] text-[12.5px] text-dim">{t('longevidade-familia-intro')}</p>
            {missing.length ? (
              <section
                aria-label={t('longevidade-falta-titulo')}
                className="rounded-card border border-accent/40 bg-accent-soft/40 px-4 py-2"
              >
                <h3 className="pt-1 text-[13.5px] font-medium text-fg">
                  {t('longevidade-falta-titulo')}
                </h3>
                <p className="text-[12px] text-dim">{t('longevidade-falta-dica')}</p>
                <ul className="flex flex-col divide-y divide-line/60">
                  {missing.map((m) => {
                    const full = members.find((x) => x.id === m.member_id);
                    return full ? <MissingRow key={m.member_id} item={m} member={full} /> : null;
                  })}
                </ul>
              </section>
            ) : null}
            {shown.length ? (
              <FamilyLifeBars
                family={f}
                members={shown}
                tone={view.tone}
                place={view.place(f.country)}
                stages={stages}
                advanced={settings?.show_advanced_later_life}
                showCouple={!memberId}
              />
            ) : member && hidden.has(member.id) ? (
              <p className="text-[13px] text-dim">
                {t('longevidade-escondida', { nome: member.name })}
              </p>
            ) : f.members.length === 0 ? (
              <p className="text-[13px] text-dim">{t('longevidade-nenhum')}</p>
            ) : null}
            <LifeMethod source={f.source} />
          </div>
        ) : null}
      </QueryState>
    </InstrumentCard>
  );
}
