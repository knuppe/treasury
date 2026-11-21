/**
 * Nível de experiência na interface (docs/profile.md): o perfil decide o que aparece por
 * padrão; o resto fica a um clique ("Mais telas", "Ver painel completo"). Enquanto o perfil
 * carrega, vale o avançado (nada some de repente para quem já usa o app).
 */
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import { useProfile } from '@/api/hooks/profile';
import { t } from '@/i18n';

import { useViewAs } from './viewAs';

export const LEVEL_RANK: Record<ProfileLevel, number> = {
  iniciante: 0,
  intermediario: 1,
  avancado: 2,
};

export const LEVEL_LABEL: Record<ProfileLevel, string> = {
  get iniciante() {
    return t('casca-nivel-iniciante');
  },
  get intermediario() {
    return t('casca-nivel-intermediario');
  },
  get avancado() {
    return t('casca-nivel-avancado');
  },
};

export const LEVEL_DESCRIPTION: Record<ProfileLevel, string> = {
  get iniciante() {
    return t('casca-nivel-iniciante-dica');
  },
  get intermediario() {
    return t('casca-nivel-intermediario-dica');
  },
  get avancado() {
    return t('casca-nivel-avancado-dica');
  },
};

export function atLeast(level: ProfileLevel, min: ProfileLevel | undefined): boolean {
  return !min || LEVEL_RANK[level] >= LEVEL_RANK[min];
}

/** Nível atual: o do membro em vista ("Ver como") ou o da família (avançado enquanto carrega). */
export function useLevel(): ProfileLevel {
  const { memberId } = useViewAs();
  return useProfile(memberId).data?.level ?? 'avancado';
}
