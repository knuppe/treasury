/** Hooks de `settings.*` e `app.info`: ajustes gerais guardados no cofre. */
import { useQueryClient } from '@tanstack/react-query';

import type { AppInfo } from '../generated/AppInfo';
import type { AppSettings } from '../generated/AppSettings';
import type { UpdateSettings } from '../generated/UpdateSettings';
import type { VaultStatus } from '../generated/VaultStatus';
import { vaultKeys } from '../vaultQueries';
import { coreKey, useCore, useCoreMutation } from './coreQuery';

export const useAppSettings = () => useCore<AppSettings>('settings', 'get');

export const useAppInfo = () => useCore<AppInfo>('app', 'info', {}, { staleTime: Infinity });

/**
 * Grava ajustes. O bloqueio automático também vale para o estado do cofre (o
 * núcleo aplica na próxima verificação, em até 30 s): atualiza o cache já.
 */
export function useSaveAppSettings() {
  const client = useQueryClient();
  return useCoreMutation<Partial<AppSettings>, AppSettings>('settings.set', {
    affects: ['settings'],
    onSuccess: (saved) => {
      client.setQueryData<VaultStatus>(vaultKeys.status, (s) =>
        s ? { ...s, auto_lock_minutes: saved.auto_lock_minutes } : s,
      );
    },
  });
}

/** Atualização automática: por instalação, no `config.toml` (vale com o cofre fechado). */
export const useUpdateSettings = () => useCore<UpdateSettings>('system', 'update_settings', {});

export function useSaveUpdateSettings() {
  const client = useQueryClient();
  return useCoreMutation<UpdateSettings, UpdateSettings>('system.set_update_settings', {
    affects: [],
    onSuccess: (saved) => client.setQueryData(coreKey('system', 'update_settings', {}), saved),
  });
}
