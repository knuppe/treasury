/**
 * Mock da atualização das contas (`accounts::freshness`): dias desde o último lançamento e o
 * status pelos limites do tipo de conta, com os mesmos padrões do núcleo. A regra do mês
 * passado sem lançamento fica só no núcleo.
 */
import type { AccountKind } from '@/api/generated/AccountKind';
import type { BalanceStatus } from '@/api/generated/BalanceStatus';
import type { StalenessLimits } from '@/api/generated/StalenessLimits';
import type { StalenessSetting } from '@/api/generated/StalenessSetting';

import { onMockReset } from '../../mockRegistry';
import { diffDays } from './dates';
import { invalid } from './errors';

const KINDS: AccountKind[] = [
  'checking',
  'savings',
  'cash',
  'credit_card',
  'benefit',
  'foreign',
  'brokerage',
  'retirement',
  'fgts',
  'other',
];

export function defaultLimits(kind: AccountKind): StalenessLimits {
  switch (kind) {
    case 'credit_card':
      return { aging_days: 35, incomplete_days: 50 };
    case 'benefit':
      return { aging_days: 20, incomplete_days: 45 };
    case 'brokerage':
    case 'retirement':
    case 'fgts':
      return { aging_days: 45, incomplete_days: 90 };
    default:
      return { aging_days: 7, incomplete_days: 20 };
  }
}

const custom = new Map<AccountKind, StalenessLimits>();
onMockReset(() => custom.clear());

const limitsOf = (kind: AccountKind) => custom.get(kind) ?? defaultLimits(kind);

export function stalenessSettings(): StalenessSetting[] {
  return KINDS.map((kind) => {
    const d = defaultLimits(kind);
    const own = custom.get(kind);
    return {
      kind,
      limits: own ?? d,
      default_limits: d,
      custom:
        own != null &&
        (own.aging_days !== d.aging_days || own.incomplete_days !== d.incomplete_days),
    };
  });
}

export function setStalenessLimits(args: Record<string, unknown>): StalenessSetting[] {
  const kind = args.kind as AccountKind;
  const l = args.limits as StalenessLimits | null | undefined;
  if (!l) {
    custom.delete(kind);
  } else {
    if (l.aging_days <= 0 || l.incomplete_days <= l.aging_days || l.incomplete_days > 3650) {
      invalid(
        'Os dias para "possivelmente incompleto" precisam ser maiores que os de "atualizado há", e os dois maiores que zero.',
      );
    }
    custom.set(kind, { aging_days: l.aging_days, incomplete_days: l.incomplete_days });
  }
  return stalenessSettings();
}

/** Dias desde a novidade mais recente (sem lançamento, a abertura) e o status da conta. */
export function freshnessOf(
  kind: AccountKind,
  openingDate: string,
  last: string | null,
  todayIso: string,
  archived: boolean,
): { staleness_days: number; balance_status: BalanceStatus | null } {
  const anchor = last ?? openingDate;
  const days = Math.max(0, diffDays(todayIso, anchor));
  const l = limitsOf(kind);
  const status: BalanceStatus =
    days > l.incomplete_days ? 'possibly_incomplete' : days > l.aging_days ? 'aging' : 'fresh';
  return { staleness_days: days, balance_status: archived ? null : status };
}
