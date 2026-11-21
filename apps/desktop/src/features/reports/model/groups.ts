/**
 * Grupos de gasto das categorias (essenciais, dívidas, futuro, não essencial): ordem fixa,
 * nomes e dicas do catálogo `grupos.ftl` e a situação de cada grupo contra a meta. Usado pelo
 * Orçamento ("Para onde vai a renda"), pela figura 12 dos Relatórios e por Ajustes › Categorias.
 */
import type { GroupTargets } from '@/api/generated/GroupTargets';
import type { SpendGroup } from '@/api/generated/SpendGroup';
import type { SpendGroupAmount } from '@/api/generated/SpendGroupAmount';
import type { SpendGroupMonth } from '@/api/generated/SpendGroupMonth';
import { t } from '@/i18n';

/** Sempre nesta ordem (a mesma do núcleo): também decide as cores. */
export const SPEND_GROUPS = [
  'essential',
  'debt',
  'future',
  'non_essential',
] as const satisfies readonly SpendGroup[];

export const groupName = (g: SpendGroup) => t(`grupos-${g}`);
export const groupTip = (g: SpendGroup) => t(`grupos-${g}-dica`);

/** Opções de um seletor de grupo, na ordem fixa. */
export const groupOptions = () => SPEND_GROUPS.map((g) => ({ value: g, label: groupName(g) }));

export const isSpendGroup = (v: unknown): v is SpendGroup =>
  typeof v === 'string' && (SPEND_GROUPS as readonly string[]).includes(v);

/**
 * Situação contra a meta. Futuro é um piso (bom quando chega na meta); os outros são tetos.
 * Sem meta ou sem renda no período: `none`.
 */
export type GroupStatus = 'over' | 'within' | 'below' | 'reached' | 'none';

export function groupStatus(
  group: SpendGroup,
  share: number | null,
  target: number | null,
): GroupStatus {
  if (share == null || target == null) return 'none';
  // Compara em décimos de ponto percentual, como a tela mostra.
  const s = Math.round(share * 1000);
  const m = Math.round(target * 1000);
  if (group === 'future') return s >= m ? 'reached' : 'below';
  return s > m ? 'over' : 'within';
}

/** A situação pede atenção (acima do teto ou abaixo do piso)? */
export const offTarget = (s: GroupStatus) => s === 'over' || s === 'below';

export interface GroupRow {
  group: SpendGroup;
  amount: number;
  share: number | null;
  target: number | null;
  status: GroupStatus;
  top: SpendGroupAmount['top'];
}

/** Linhas dos quatro grupos de um mês (ou do total), com as metas e a situação. */
export function groupRows(month: SpendGroupMonth, targets: GroupTargets): GroupRow[] {
  return SPEND_GROUPS.map((group) => {
    const g = month.groups.find((x) => x.group === group);
    const share = g?.share_of_income ?? null;
    const target = targets[group];
    return {
      group,
      amount: g?.amount ?? 0,
      share,
      target,
      status: groupStatus(group, share, target),
      top: g?.top ?? [],
    };
  });
}

/** Soma das metas definidas (fração da renda), ou `null` se nenhuma tem meta. */
export function targetsSum(targets: GroupTargets): number | null {
  const values = SPEND_GROUPS.map((g) => targets[g]).filter((v): v is number => v != null);
  return values.length ? values.reduce((s, v) => s + v, 0) : null;
}

/** O guia 50/30/20 (Dívidas sem meta: dívida acima do mínimo sai do Futuro). */
export const DEFAULT_TARGETS: GroupTargets = {
  essential: 0.5,
  debt: null,
  future: 0.2,
  non_essential: 0.3,
};
