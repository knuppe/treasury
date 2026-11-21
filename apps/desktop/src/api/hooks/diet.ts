/**
 * Hooks de `food.diet_*` (docs/gtin-inflation.md §7.2): alergias e restrições alimentares de
 * cada membro, editadas no cadastro em 02 · Família e usadas nos avisos de 06 · Consumo › Saúde.
 */
import type { MemberDiet } from '../generated/MemberDiet';
import type { MemberDietInput } from '../generated/MemberDietInput';
import { useCore, useCoreMutation } from './coreQuery';

/** Só os membros com alguma restrição marcada. */
export const useMemberDiets = () => useCore<MemberDiet[]>('food', 'diet_list');

/** Grava a lista inteira de um membro (lista vazia limpa). */
export const useSetMemberDiet = () =>
  useCoreMutation<MemberDietInput, MemberDiet[]>('food.diet_set', { affects: ['food'] });
