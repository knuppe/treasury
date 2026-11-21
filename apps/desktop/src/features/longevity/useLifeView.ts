/** Como quem vê lê a linha da vida: o tom do perfil (do membro em "Ver como") e o país. */
import type { ProfileTone } from '@/api/generated/ProfileTone';
import { useCountryInfo } from '@/api/hooks/country';
import { useProfile } from '@/api/hooks/profile';
import { useViewAs } from '@/app/viewAs';

import type { Place } from './model';

export function useLifeView(): { tone: ProfileTone; place: (code: string) => Place } {
  const { memberId } = useViewAs();
  const tone = useProfile(memberId).data?.tone ?? 'equilibrado';
  const info = useCountryInfo();
  return {
    tone,
    place: (code) => ({ code, name: info && info.code === code ? info.name : code }),
  };
}
