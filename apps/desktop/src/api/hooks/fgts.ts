/** Hooks de `fgts.*` (docs/brazil/fgts.md): contas vinculadas, extrato da Caixa, conferência e mais. */
import type { FgtsAccountPatch } from '../generated/FgtsAccountPatch';
import type { FgtsAccountView } from '../generated/FgtsAccountView';
import type { FgtsAnniversary } from '../generated/FgtsAnniversary';
import type { FgtsCheck } from '../generated/FgtsCheck';
import type { FgtsCompare } from '../generated/FgtsCompare';
import type { FgtsEntryView } from '../generated/FgtsEntryView';
import type { FgtsHousingInput } from '../generated/FgtsHousingInput';
import type { FgtsHousingResult } from '../generated/FgtsHousingResult';
import type { FgtsModalityInput } from '../generated/FgtsModalityInput';
import type { FgtsStatementArgs } from '../generated/FgtsStatementArgs';
import type { FgtsStatementImport } from '../generated/FgtsStatementImport';
import type { FgtsStatementPreview } from '../generated/FgtsStatementPreview';
import type { FgtsWithdrawalInput } from '../generated/FgtsWithdrawalInput';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useFgtsAccounts = (enabled = true) =>
  useCore<FgtsAccountView[]>('fgts', 'list', {}, { enabled });

export const useFgtsEntries = (accountId: string) =>
  useCore<FgtsEntryView[]>('fgts', 'entries', { account_id: accountId });

export const useFgtsCheck = (accountId: string) =>
  useCore<FgtsCheck>('fgts', 'check', { account_id: accountId });

export const useFgtsCompare = (accountId: string) =>
  useCore<FgtsCompare>('fgts', 'yield_compare', { account_id: accountId });

export const useFgtsAnniversary = (memberId: string | null | undefined) =>
  useCore<FgtsAnniversary>('fgts', 'anniversary_simulate', memberId ? { member_id: memberId } : {});

/** Prévia do extrato da Caixa (sem gravar). */
export const previewFgtsStatement = (args: FgtsStatementArgs) =>
  call<FgtsStatementPreview>('fgts.statement_preview', args);

export const useImportFgtsStatement = () =>
  useCoreMutation<FgtsStatementArgs, FgtsStatementImport>('fgts.statement_import', {
    affects: ['ledger', 'accounts'],
  });

export const useFgtsWithdrawal = () =>
  useCoreMutation<FgtsWithdrawalInput, { txn_id: string }>('fgts.record_withdrawal', {
    affects: ['ledger', 'accounts'],
  });

export const useFgtsHousing = () =>
  useCoreMutation<FgtsHousingInput, FgtsHousingResult>('fgts.use_for_housing', {
    affects: ['ledger', 'debt'],
  });

export const useFgtsModality = () =>
  useCoreMutation<FgtsModalityInput, FgtsAnniversary>('fgts.set_modality', {
    affects: ['ledger'],
  });

export const useUpdateFgtsAccount = () =>
  useCoreMutation<FgtsAccountPatch, FgtsAccountView | null>('fgts.update_account', {
    affects: ['ledger'],
  });
