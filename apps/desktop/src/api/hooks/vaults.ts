/**
 * Vários cofres (`vaults.*`, docs/vaults.md). A lista vale com o cofre bloqueado e mora sob a
 * chave `['vault', …]`, que o bloqueio não apaga. Trocar de cofre fecha o atual: o estado novo
 * volta do núcleo e todo dado em memória do cofre anterior é descartado.
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { call } from '../client';
import type { DemoProfile } from '../generated/DemoProfile';
import type { DemoVaultResult } from '../generated/DemoVaultResult';
import type { VaultList } from '../generated/VaultList';
import type { VaultStatus } from '../vault';
import { clearSensitiveCache, vaultKeys } from '../vaultQueries';

export const vaultListKey = ['vault', 'list'] as const;

export const useVaults = () =>
  useQuery({ queryKey: vaultListKey, queryFn: () => call<VaultList>('vaults.list') });

/** Troca, cria ou abre um cofre: o núcleo devolve o estado do cofre novo. */
function useSwitching<TArgs>(cmd: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: TArgs) => call<VaultStatus>(cmd, args),
    onSuccess: async (status) => {
      clearSensitiveCache(client);
      client.setQueryData(vaultKeys.status, status);
      await client.invalidateQueries({ queryKey: vaultListKey });
    },
  });
}

export const useSelectVault = () => useSwitching<{ id: string }>('vaults.select');
export const useNewVault = () =>
  useSwitching<{ name: string; purpose?: string; path?: string }>('vaults.new');
export const useAddVault = () =>
  useSwitching<{ path: string; name?: string; purpose?: string }>('vaults.add');

/**
 * Cofre de demonstração (docs/demo.md): cria um cofre novo e separado, sem senha (abre pela
 * chave fixa do app), e o preenche com o histórico do assistente. O cofre atual fecha (com backup).
 */
export const useNewDemoVault = () => {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: {
      name: string;
      household_name: string;
      /** Sem senha (o padrão), o cofre abre pela chave fixa do app. */
      password?: string;
      profile: DemoProfile;
    }) => call<DemoVaultResult>('vaults.new_demo', args),
    onSuccess: async (r) => {
      clearSensitiveCache(client);
      client.setQueryData(vaultKeys.status, r.status);
      await client.invalidateQueries({ queryKey: vaultListKey });
    },
  });
};

function useListEdit<TArgs>(cmd: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: TArgs) => call<VaultList>(cmd, args),
    onSuccess: async (list) => {
      client.setQueryData(vaultListKey, list);
      await client.invalidateQueries({ queryKey: vaultKeys.status });
    },
  });
}

export const useUpdateVault = () =>
  useListEdit<{ id: string; name?: string; purpose?: string | null }>('vaults.update');
/** Tira da lista; com `delete_file`, o arquivo vai para a Lixeira (o cofre em uso, nunca). */
export const useRemoveVault = () =>
  useListEdit<{ id: string; delete_file?: boolean }>('vaults.remove');
/** Cópia de um cofre (mesma senha), como "… (cópia)" na lista. */
export const useCopyVault = () => useListEdit<{ id: string; name?: string }>('vaults.copy');

/** Escolhe um arquivo .db pelo diálogo do sistema (só no app; no navegador, `null`). */
export async function pickVaultFile(): Promise<string | null> {
  const { isTauri } = await import('../client');
  if (!isTauri()) return null;
  const { invoke } = await import('@tauri-apps/api/core');
  const path = await invoke<string | string[] | null>('plugin:dialog|open', {
    options: {
      title: 'Abrir um cofre do Treasury',
      multiple: false,
      directory: false,
      filters: [{ name: 'Cofre do Treasury', extensions: ['db'] }],
    },
  });
  return typeof path === 'string' ? path : null;
}
