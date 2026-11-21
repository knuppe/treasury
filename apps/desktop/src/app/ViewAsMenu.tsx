/**
 * Barra lateral, abaixo do nome da família: "Ver como" a família ou um membro
 * (docs/members.md). Só com 2+ membros.
 */
import { Select } from '@/design/components';
import { t } from '@/i18n';

import { useViewAs } from './viewAs';

const FAMILY = '__familia__';

export function ViewAsMenu() {
  const { memberId, members, setViewAs } = useViewAs();
  if (members.length < 2) return null;
  return (
    <div className="mt-2.5 flex flex-col gap-1">
      <span className="label-caps text-[10px]" aria-hidden>
        {t('casca-ver-como')}
      </span>
      <Select
        aria-label={t('casca-ver-como')}
        className="h-8 w-full"
        value={memberId ?? FAMILY}
        onValueChange={(v) => setViewAs(v === FAMILY ? null : v)}
        options={[
          { value: FAMILY, label: t('casca-familia-toda') },
          ...members.map((m) => ({ value: m.id, label: m.name })),
        ]}
      />
    </div>
  );
}
