/** Hooks de `retirement.*` (docs/international.md): planos de aposentadoria do país e o limite do ano. */
import type { RetirementAccountPlan } from '../generated/RetirementAccountPlan';
import type { RetirementAccountStatus } from '../generated/RetirementAccountStatus';
import { useCore } from './coreQuery';

export const useRetirementPlans = () => useCore<RetirementAccountPlan[]>('retirement', 'plans', {});

export const useRetirementStatus = (accountId: string | null) =>
  useCore<RetirementAccountStatus>(
    'retirement',
    'status',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );
