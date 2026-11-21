/**
 * Hooks do perfil (`profile.*`, docs/profile.md): nível de experiência e tom, que decidem o
 * que a interface mostra por padrão. Sem perfil salvo, o núcleo devolve o padrão (avançado
 * para cofres com dados; questionário pendente para um cofre vazio).
 */
import type { ProfileLevel } from '../generated/ProfileLevel';
import type { ProfileQuestion } from '../generated/ProfileQuestion';
import type { ProfileTone } from '../generated/ProfileTone';
import type { UserProfile } from '../generated/UserProfile';
import { useCore, useCoreMutation } from './coreQuery';

/** Perfil da família ou, com `memberId`, de um membro (sem questionário próprio: o da família). */
export const useProfile = (memberId?: string | null) =>
  useCore<UserProfile>('profile', 'get', memberId ? { member_id: memberId } : {});

export const useProfileQuestions = () =>
  useCore<ProfileQuestion[]>('profile', 'questions', {}, { staleTime: Infinity });

export const useAnswerProfile = () =>
  useCoreMutation<{ answers: Record<string, string[]>; member_id?: string }, UserProfile>(
    'profile.answer',
    {
      affects: ['profile'],
    },
  );

export const useSkipProfile = () =>
  useCoreMutation<{ member_id?: string }, UserProfile>('profile.skip', { affects: ['profile'] });

export const useSetProfile = () =>
  useCoreMutation<{ level?: ProfileLevel; tone?: ProfileTone; member_id?: string }, UserProfile>(
    'profile.set',
    {
      affects: ['profile'],
    },
  );
