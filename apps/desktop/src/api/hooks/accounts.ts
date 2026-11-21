/** Hooks de `accounts.*` e `fx.*`: contas, saldos, histórico, conciliação e câmbio. */
import type { AccountInput } from '../generated/AccountInput';
import type { AccountPatch } from '../generated/AccountPatch';
import type { AccountView } from '../generated/AccountView';
import type { BalanceHistoryArgs } from '../generated/BalanceHistoryArgs';
import type { CapitalBalanceInput } from '../generated/CapitalBalanceInput';
import type { CapitalShareInput } from '../generated/CapitalShareInput';
import type { CapitalShareView } from '../generated/CapitalShareView';
import type { BalanceSync } from '../generated/BalanceSync';
import type { ResolveSyncArgs } from '../generated/ResolveSyncArgs';
import type { SyncBalanceArgs } from '../generated/SyncBalanceArgs';
import type { SetStalenessArgs } from '../generated/SetStalenessArgs';
import type { StalenessSetting } from '../generated/StalenessSetting';
import type { BalanceSyncHints } from '../generated/BalanceSyncHints';
import type { BalancePoint } from '../generated/BalancePoint';
import type { OfxMatch } from '../generated/OfxMatch';
import type { OfxMatchArgs } from '../generated/OfxMatchArgs';
import type { ReconcileArgs } from '../generated/ReconcileArgs';
import type { ReconcileResult } from '../generated/ReconcileResult';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useAccounts = (includeArchived = false) =>
  useCore<AccountView[]>('accounts', 'list', { include_archived: includeArchived });

export const useAccount = (id: string | null | undefined) =>
  useCore<AccountView>('accounts', 'get', { id }, { enabled: Boolean(id) });

export const useBalanceHistory = (args: BalanceHistoryArgs | null) =>
  useCore<BalancePoint[]>('accounts', 'balance_history', args, { enabled: Boolean(args) });

export const useCreateAccount = () =>
  useCoreMutation<AccountInput, AccountView>('accounts.create', { affects: ['accounts'] });

export const useUpdateAccount = () =>
  useCoreMutation<AccountPatch, AccountView>('accounts.update', { affects: ['accounts'] });

export const useDeleteAccount = () =>
  useCoreMutation<{ id: string }, null>('accounts.delete', { affects: ['accounts'] });

/** Confere (e, com `apply`, marca como conciliado). Só invalida quando grava. */
export const useReconcile = () =>
  useCoreMutation<ReconcileArgs, ReconcileResult>('accounts.reconcile', { affects: ['ledger'] });

/** Conta do OFX pelo banco, agência e número (ou pelo final do cartão). */
export const matchOfx = (args: OfxMatchArgs) => call<OfxMatch>('accounts.match_ofx', args);

/** Marcos de sincronia de saldo (saldo do banco × calculado) de uma conta. */
export const useBalanceSyncs = (accountId: string | null | undefined) =>
  useCore<BalanceSync[]>(
    'accounts',
    'list_syncs',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );

export const useSyncHints = (id: string | null | undefined) =>
  useCore<BalanceSyncHints>('accounts', 'sync_hints', { id }, { enabled: Boolean(id) });

/** Lança a diferença para o saldo do banco como ajuste e grava o marco. */
export const useSyncBalance = () =>
  useCoreMutation<SyncBalanceArgs, BalanceSync>('accounts.sync_balance', { affects: ['ledger'] });

export const useResolveSync = () =>
  useCoreMutation<ResolveSyncArgs, BalanceSync>('accounts.resolve_sync', { affects: ['accounts'] });

/** Cotas capital da cooperativa ligadas à conta (`null` se não ativadas). */
export const useCapitalShare = (accountId: string | null | undefined) =>
  useCore<CapitalShareView | null>(
    'accounts',
    'capital_share',
    { account_id: accountId },
    { enabled: Boolean(accountId) },
  );

export const useSetCapitalShare = () =>
  useCoreMutation<CapitalShareInput, CapitalShareView | null>('accounts.capital_share_set', {
    affects: ['accounts'],
  });

export const useAddCapitalBalance = () =>
  useCoreMutation<CapitalBalanceInput, CapitalShareView | null>(
    'accounts.capital_share_add_balance',
    { affects: ['accounts'] },
  );

export const useDeleteCapitalBalance = () =>
  useCoreMutation<{ id: string }, null>('accounts.capital_share_delete_balance', {
    affects: ['accounts'],
  });

export const useRemoveCapitalShare = () =>
  useCoreMutation<{ account_id: string }, null>('accounts.capital_share_remove', {
    affects: ['accounts'],
  });

/** Tira os aportes das despesas (viram aporte, como numa corretora). */
export const useMarkCapitalContributions = () =>
  useCoreMutation<{ account_id: string }, { updated: number }>(
    'accounts.capital_share_mark_contributions',
    { affects: ['ledger'] },
  );

/** Limites de atualização das contas por tipo (docs/account-freshness.md). */
export const useStalenessLimits = () =>
  useCore<StalenessSetting[]>('accounts', 'staleness_limits', {});

export const useSetStalenessLimits = () =>
  useCoreMutation<SetStalenessArgs, StalenessSetting[]>('accounts.set_staleness_limits', {
    affects: 'all',
  });
