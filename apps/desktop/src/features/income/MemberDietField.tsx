/**
 * Alergias e restrições alimentares do membro (docs/gtin-inflation.md §7.2), no cadastro em
 * 02 · Família. Opcional; o núcleo cruza com o Open Food Facts e avisa em 06 · Consumo › Saúde.
 */
import { useState } from 'react';

import { useMemberDiets, useSetMemberDiet } from '@/api/hooks/diet';
import { cx } from '@/design/cx';
import { Checkbox } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

/** As restrições que o núcleo aceita, na ordem em que aparecem. */
const RESTRICTIONS = [
  'gluten',
  'milk',
  'eggs',
  'peanuts',
  'nuts',
  'soybeans',
  'fish',
  'crustaceans',
  'sesame',
  'vegan',
  'vegetarian',
] as const;
type Restriction = (typeof RESTRICTIONS)[number];

const LABEL: Record<Restriction, () => string> = {
  gluten: () => t('familia-dieta-gluten'),
  milk: () => t('familia-dieta-milk'),
  eggs: () => t('familia-dieta-eggs'),
  peanuts: () => t('familia-dieta-peanuts'),
  nuts: () => t('familia-dieta-nuts'),
  soybeans: () => t('familia-dieta-soybeans'),
  fish: () => t('familia-dieta-fish'),
  crustaceans: () => t('familia-dieta-crustaceans'),
  sesame: () => t('familia-dieta-sesame'),
  vegan: () => t('familia-dieta-vegan'),
  vegetarian: () => t('familia-dieta-vegetarian'),
};

/**
 * Estado da escolha do membro: o que está gravado até alguém mexer, e `save()` grava só se mudou
 * (para o botão Salvar do cadastro).
 */
export function useMemberDietDraft(memberId: string) {
  const saved = useMemberDiets().data?.find((d) => d.member_id === memberId)?.restrictions ?? [];
  const mutation = useSetMemberDiet();
  const [draft, setDraft] = useState<string[] | null>(null);
  const value = draft ?? saved;
  const changed =
    draft !== null && (draft.length !== saved.length || draft.some((r) => !saved.includes(r)));
  const toggle = (r: string, on: boolean) =>
    setDraft(on ? [...new Set([...value, r])] : value.filter((x) => x !== r));
  const save = async () => {
    if (changed) await mutation.mutateAsync({ member_id: memberId, restrictions: draft });
  };
  return { value, toggle, save, mutation };
}

export function MemberDietField({
  value,
  onToggle,
}: {
  value: readonly string[];
  onToggle: (restriction: string, on: boolean) => void;
}) {
  return (
    <fieldset className="flex flex-col gap-1.5">
      <legend className="label-caps mb-1.5">{t('familia-dieta-titulo')}</legend>
      <div className="flex flex-wrap gap-1.5">
        {RESTRICTIONS.map((r) => {
          const on = value.includes(r);
          return (
            <Checkbox
              key={r}
              checked={on}
              onChange={(v) => onToggle(r, v)}
              label={LABEL[r]()}
              className={cx(
                'rounded-control border px-2 py-1 text-[12.5px]',
                on ? 'border-accent/50 bg-accent/10' : 'border-line hover:border-line-strong',
              )}
            />
          );
        })}
      </div>
      <p className="text-[12px] leading-snug text-dim">{t('familia-dieta-dica')}</p>
    </fieldset>
  );
}
