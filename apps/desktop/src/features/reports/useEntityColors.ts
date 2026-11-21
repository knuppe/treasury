/**
 * Cores das categorias de primeiro nível de despesa, dos titulares e dos cartões, presas à
 * entidade. A janela que decide quem disputa as cores é fixa (24 meses até o mês atual),
 * então trocar o filtro da tela não repinta as séries que continuam visíveis.
 */
import { useMemo } from 'react';

import { useAccounts, useCategories, useMembers, useSpending } from '@/api/hooks/reports';
import { todayIso } from '@/lib/format';
import { monthBounds, monthOf, shiftMonth } from '@/lib/months';
import { useUi } from '@/state/ui';

import { entityColors, type EntityColors } from './model/colors';
import { SPEND_GROUPS } from './model/groups';

export const NO_MEMBER_KEY = '__sem_titular__';

function colorWindow() {
  const end = monthOf(todayIso());
  return { from: monthBounds(shiftMonth(end, -23)).from, to: monthBounds(end).to };
}

export function useCategoryColors(): EntityColors {
  const theme = useUi((s) => s.theme);
  const categories = useCategories({ kind: 'expense' });
  const window = useSpending({ ...colorWindow(), kind: 'expense', group_by: 'parent_category' });
  return useMemo(() => {
    const roots = (categories.data ?? [])
      .filter((c) => !c.parent_id)
      .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name, 'pt-BR'))
      .map((c) => c.id);
    const active = window.data
      ? new Set(window.data.groups.filter((g) => g.amount > 0 && g.key).map((g) => g.key!))
      : null;
    return entityColors(roots, active, theme);
  }, [categories.data, window.data, theme]);
}

export function useMemberColors(): EntityColors {
  const theme = useUi((s) => s.theme);
  const members = useMembers();
  return useMemo(() => {
    const ids = (members.data ?? [])
      .slice()
      .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name, 'pt-BR'))
      .map((m) => m.id);
    return entityColors([...ids, NO_MEMBER_KEY], null, theme);
  }, [members.data, theme]);
}

export function useAccountColors(filter?: (kind: string) => boolean): EntityColors {
  const theme = useUi((s) => s.theme);
  const accounts = useAccounts();
  return useMemo(() => {
    const ids = (accounts.data ?? [])
      .filter((a) => !filter || filter(a.kind))
      .sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name, 'pt-BR'))
      .map((a) => a.id);
    return entityColors(ids, null, theme);
  }, [accounts.data, theme, filter]);
}

/** Cores dos quatro grupos de gasto, presas ao grupo (essenciais, dívidas, futuro, não essencial). */
export function useGroupColors(): EntityColors {
  const theme = useUi((s) => s.theme);
  return useMemo(() => entityColors(SPEND_GROUPS, null, theme), [theme]);
}
