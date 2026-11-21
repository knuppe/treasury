/** Hooks de `cards.*`: faturas, pagamento, parcelamentos, compromissos futuros e plásticos. */
import type { AnnualFeeInput } from '../generated/AnnualFeeInput';
import type { BulkResult } from '../generated/BulkResult';
import type { CardOverview } from '../generated/CardOverview';
import type { CardValueReport } from '../generated/CardValueReport';
import type { CardPlastic } from '../generated/CardPlastic';
import type { CardPlasticInput } from '../generated/CardPlasticInput';
import type { CardPlasticPatch } from '../generated/CardPlasticPatch';
import type { FutureCommitmentsArgs } from '../generated/FutureCommitmentsArgs';
import type { InstallmentInput } from '../generated/InstallmentInput';
import type { InstallmentPlan } from '../generated/InstallmentPlan';
import type { LimitChangeInput } from '../generated/LimitChangeInput';
import type { LoyaltyEntry } from '../generated/LoyaltyEntry';
import type { LoyaltyEntryInput } from '../generated/LoyaltyEntryInput';
import type { LoyaltyProgram } from '../generated/LoyaltyProgram';
import type { LoyaltyProgramInput } from '../generated/LoyaltyProgramInput';
import type { LoyaltyProgramPatch } from '../generated/LoyaltyProgramPatch';
import type { LoyaltyTransferInput } from '../generated/LoyaltyTransferInput';
import type { ListStatementsArgs } from '../generated/ListStatementsArgs';
import type { MonthAmount } from '../generated/MonthAmount';
import type { MoveToStatementArgs } from '../generated/MoveToStatementArgs';
import type { PayStatementArgs } from '../generated/PayStatementArgs';
import type { PayStatementResult } from '../generated/PayStatementResult';
import type { StatementDetail } from '../generated/StatementDetail';
import type { RewardProgramInput } from '../generated/RewardProgramInput';
import type { RewardsView } from '../generated/RewardsView';
import type { StatementPatch } from '../generated/StatementPatch';
import type { StatementView } from '../generated/StatementView';
import { useCore, useCoreMutation } from './coreQuery';

export const useStatements = (args: ListStatementsArgs, enabled = true) =>
  useCore<StatementView[]>('cards', 'list_statements', args, { enabled });

export const useStatement = (id: string | null) =>
  useCore<StatementDetail>('cards', 'get_statement', { id }, { enabled: Boolean(id) });

export const useInstallmentPlans = (accountId?: string) =>
  useCore<InstallmentPlan[]>('cards', 'list_installment_plans', { account_id: accountId });

export const useFutureCommitments = (args: FutureCommitmentsArgs) =>
  useCore<MonthAmount[]>('cards', 'future_commitments', args);

export const usePayStatement = () =>
  useCoreMutation<PayStatementArgs, PayStatementResult>('cards.pay_statement', {
    affects: ['ledger'],
  });

export const useSetInformedTotal = () =>
  useCoreMutation<{ id: string; informed_total: number | null }, StatementView>(
    'cards.set_informed_total',
    { affects: ['ledger'] },
  );

export const useMoveToStatement = () =>
  useCoreMutation<MoveToStatementArgs, BulkResult>('cards.move_to_statement', {
    affects: ['ledger'],
  });

export const useCreateInstallments = () =>
  useCoreMutation<InstallmentInput, InstallmentPlan>('cards.create_installments', {
    affects: ['ledger'],
  });

export const useDeleteInstallmentPlan = () =>
  useCoreMutation<{ id: string }, BulkResult>('cards.delete_installment_plan', {
    affects: ['ledger'],
  });

export const useCardPlastics = (accountId: string | null | undefined) =>
  useCore<CardPlastic[]>(
    'cards',
    'list_plastics',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );

export const useCreatePlastic = () =>
  useCoreMutation<CardPlasticInput, CardPlastic>('cards.create_plastic', { affects: ['ledger'] });

export const useUpdatePlastic = () =>
  useCoreMutation<CardPlasticPatch, CardPlastic>('cards.update_plastic', { affects: ['ledger'] });

export const useDeletePlastic = () =>
  useCoreMutation<{ id: string }, BulkResult>('cards.delete_plastic', { affects: ['ledger'] });

export const useCardOverview = (accountId: string | null | undefined) =>
  useCore<CardOverview>(
    'cards',
    'overview',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );

export const useUpdateStatement = () =>
  useCoreMutation<StatementPatch, StatementView>('cards.update_statement', { affects: ['ledger'] });

export const useAddLimitChange = () =>
  useCoreMutation<LimitChangeInput, CardOverview>('cards.add_limit_change', {
    affects: ['ledger', 'accounts'],
  });

export const useDeleteLimitChange = () =>
  useCoreMutation<{ id: string }, null>('cards.delete_limit_change', {
    affects: ['ledger', 'accounts'],
  });

export const useSetAnnualFee = () =>
  useCoreMutation<AnnualFeeInput, CardOverview>('cards.set_annual_fee', { affects: ['ledger'] });

export const useEndAnnualFee = () =>
  useCoreMutation<{ account_id: string }, CardOverview>('cards.end_annual_fee', {
    affects: ['ledger'],
  });

export const useCardRewards = (accountId: string | null | undefined) =>
  useCore<RewardsView>(
    'cards',
    'rewards',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );

export const useSetRewardProgram = () =>
  useCoreMutation<RewardProgramInput, RewardsView>('cards.set_reward_program', {
    affects: ['ledger'],
  });

export const useEndRewardProgram = () =>
  useCoreMutation<{ account_id: string }, RewardsView>('cards.end_reward_program', {
    affects: ['ledger'],
  });

export const useLoyaltyPrograms = () => useCore<LoyaltyProgram[]>('cards', 'loyalty_list', {});

export const useLoyaltyEntries = (programId: string | null) =>
  useCore<LoyaltyEntry[]>(
    'cards',
    'loyalty_entries',
    { program_id: programId },
    { enabled: Boolean(programId) },
  );

export const useCreateLoyalty = () =>
  useCoreMutation<LoyaltyProgramInput, LoyaltyProgram>('cards.loyalty_create', {
    affects: ['ledger'],
  });

export const useUpdateLoyalty = () =>
  useCoreMutation<LoyaltyProgramPatch, LoyaltyProgram>('cards.loyalty_update', {
    affects: ['ledger'],
  });

export const useDeleteLoyalty = () =>
  useCoreMutation<{ id: string }, null>('cards.loyalty_delete', { affects: ['ledger'] });

export const useAddLoyaltyEntry = () =>
  useCoreMutation<LoyaltyEntryInput, LoyaltyProgram>('cards.loyalty_add_entry', {
    affects: ['ledger'],
  });

export const useLoyaltyTransfer = () =>
  useCoreMutation<LoyaltyTransferInput, LoyaltyProgram[]>('cards.loyalty_transfer', {
    affects: ['ledger'],
  });

export const useDeleteLoyaltyEntry = () =>
  useCoreMutation<{ id: string }, BulkResult>('cards.loyalty_delete_entry', {
    affects: ['ledger'],
  });

export const useCardValue = () => useCore<CardValueReport>('cards', 'value', {});
