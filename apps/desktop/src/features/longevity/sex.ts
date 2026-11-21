/** Sexo do membro (opcional): homem, mulher ou "prefiro não informar" (tábua de ambos). */
import type { MemberSex } from '@/api/generated/MemberSex';
import { t } from '@/i18n';

export const SEXES: readonly MemberSex[] = ['female', 'male', 'unspecified'];

export function sexLabel(s: MemberSex): string {
  switch (s) {
    case 'male':
      return t('familia-sexo-homem');
    case 'female':
      return t('familia-sexo-mulher');
    case 'unspecified':
      return t('familia-sexo-nao-informar');
  }
}

export const sexOptions = () => SEXES.map((value) => ({ value, label: sexLabel(value) }));
