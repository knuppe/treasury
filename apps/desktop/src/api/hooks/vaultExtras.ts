/**
 * Hooks dos comandos extras do cofre: trocar a senha mestra, kit de recuperação e
 * desbloqueio pelo keychain. Senhas e o código de recuperação nunca vão para o
 * cache (`gcTime: 0`, resultado só no estado do componente).
 */
import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query';

import type { ChangePasswordArgs } from '../generated/ChangePasswordArgs';
import type { Fido2Slot } from '../generated/Fido2Slot';
import type { RecoveryShares } from '../generated/RecoveryShares';
import type { RecoveryStatus } from '../generated/RecoveryStatus';
import type { RecoveryKit } from '../generated/RecoveryKit';
import type { VaultStatus } from '../generated/VaultStatus';
import { call, type ApiError } from '../client';
import { vaultKeys } from '../vaultQueries';

function useVaultMutation<TArgs>(cmd: string, after?: (client: QueryClient) => void) {
  const client = useQueryClient();
  return useMutation<VaultStatus, ApiError, TArgs>({
    mutationFn: (args) => call<VaultStatus>(cmd, args ?? {}),
    gcTime: 0,
    onSuccess: (status) => {
      client.setQueryData(vaultKeys.status, status);
      after?.(client);
    },
  });
}

/** O kit de recuperação muda de situação: a tela de Segurança e o lembrete do Painel. */
const refreshRecovery = (client: QueryClient) => {
  void client.invalidateQueries({ queryKey: ['vault', 'recovery_status'] });
  void client.invalidateQueries({ queryKey: ['reports', 'insights'] });
};

/** Trocar a senha invalida o kit antigo (o código é a chave do banco). */
export const useChangePassword = () =>
  useVaultMutation<ChangePasswordArgs>('vault.change_password', refreshRecovery);

export const useKeychainEnable = () =>
  useVaultMutation<{ password: string }>('vault.keychain_enable');

export const useKeychainDisable = () =>
  useVaultMutation<Record<string, never>>('vault.keychain_disable');

export function useRecoveryKit() {
  const client = useQueryClient();
  return useMutation<RecoveryKit, ApiError, { password: string }>({
    mutationFn: (args) => call<RecoveryKit>('vault.recovery_kit', args),
    gcTime: 0,
    onSuccess: () => refreshRecovery(client),
  });
}

// --- PIN, recuperação dividida e chaves físicas ---------------------------------------

export const usePinEnable = () =>
  useVaultMutation<{ password: string; pin: string }>('vault.pin_enable');

export const usePinDisable = () => useVaultMutation<Record<string, never>>('vault.pin_disable');

/** Desbloqueio pelo PIN (tela do cofre bloqueado). */
export const useUnlockPin = () => useVaultMutation<{ pin: string }>('vault.unlock_pin');

export const useUnlockShares = () =>
  useVaultMutation<{ shares: string[]; new_password: string }>('vault.unlock_shares');

export const recoveryKeys = {
  status: ['vault', 'recovery_status'] as const,
  fido2: ['vault', 'fido2_slots'] as const,
};

export function useRecoveryStatus() {
  return useQuery({
    queryKey: recoveryKeys.status,
    queryFn: () => call<RecoveryStatus>('vault.recovery_status'),
  });
}

/** Chaves físicas cadastradas (públicas: nada aqui abre o cofre sem a chave). */
export function useFido2Slots(enabled = true) {
  return useQuery({
    queryKey: recoveryKeys.fido2,
    queryFn: () => call<Fido2Slot[]>('vault.fido2_slots'),
    enabled,
  });
}

/** Gera as 3 partes; o resultado fica só no estado do componente. */
export function useCreateShares() {
  const client = useQueryClient();
  return useMutation<RecoveryShares, ApiError, { password: string; rotate?: boolean }>({
    mutationFn: (args) => call<RecoveryShares>('vault.recovery_shares_create', args),
    gcTime: 0,
    onSuccess: () => void client.invalidateQueries({ queryKey: ['vault'] }),
  });
}

export function useFido2Remove() {
  const client = useQueryClient();
  return useMutation<null, ApiError, { id: string; password: string }>({
    mutationFn: (args) => call<null>('vault.fido2_remove', args),
    gcTime: 0,
    onSuccess: () => void client.invalidateQueries({ queryKey: ['vault'] }),
  });
}
