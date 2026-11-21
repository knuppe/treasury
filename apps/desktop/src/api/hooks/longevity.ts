/**
 * Hooks da linha da vida (`longevity.*`, docs/life-expectancy.md): os ajustes da família, as
 * fases e a perspectiva de cada membro (médias da população, calculadas no próprio computador).
 * Desligada por padrão.
 */
import type { FamilyOutlook } from '../generated/FamilyOutlook';
import type { LongevitySettings } from '../generated/LongevitySettings';
import type { MemberOutlook } from '../generated/MemberOutlook';
import type { StageInfo } from '../generated/StageInfo';
import { useCore, useCoreMutation } from './coreQuery';

export const useLongevitySettings = () => useCore<LongevitySettings>('longevity', 'settings');

/** A função está ligada (`false` enquanto carrega). */
export const useLongevityEnabled = (): boolean => useLongevitySettings().data?.enabled === true;

export const useSetLongevitySettings = () =>
  useCoreMutation<LongevitySettings, LongevitySettings>('longevity.set_settings', {
    affects: ['longevity'],
  });

export const useLifeStages = () =>
  useCore<StageInfo[]>('longevity', 'stages', {}, { staleTime: Infinity });

/** A família toda (com a função desligada: `enabled: false` e listas vazias). */
export const useFamilyOutlook = (enabled = true) =>
  useCore<FamilyOutlook>('longevity', 'family', {}, { enabled });

/** Um membro (erro `invalid` sem data de nascimento). */
export const useMemberOutlook = (memberId: string | null | undefined, enabled = true) =>
  useCore<MemberOutlook>(
    'longevity',
    'outlook',
    { member_id: memberId ?? '' },
    { enabled: enabled && Boolean(memberId), retry: false },
  );
