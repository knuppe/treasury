/** Hooks de `backup.*`: saúde, lista, criar, verificar, ajustes e restaurar. */
import { useMutation, useQueryClient } from '@tanstack/react-query';

import type { BackupHealth } from '../generated/BackupHealth';
import type { BackupRecord } from '../generated/BackupRecord';
import type { BackupSettings } from '../generated/BackupSettings';
import type { RestoreArgs } from '../generated/RestoreArgs';
import type { VaultStatus } from '../generated/VaultStatus';
import { call, type ApiError } from '../client';
import { clearSensitiveCache, vaultKeys } from '../vaultQueries';
import { useCore, useCoreMutation } from './coreQuery';

export const useBackupHealth = () => useCore<BackupHealth>('backup', 'health');

export const useBackups = () => useCore<BackupRecord[]>('backup', 'list');

export const useBackupSettings = () => useCore<BackupSettings>('backup', 'get_settings');

export const useCreateBackup = () =>
  useCoreMutation<Record<string, never>, BackupRecord>('backup.create', { affects: ['backup'] });

export const useVerifyBackup = () =>
  useCoreMutation<{ id: string; backup_password?: string }, BackupRecord>('backup.verify', {
    affects: ['backup'],
    sensitive: true,
  });

export const useSaveBackupSettings = () =>
  useCoreMutation<BackupSettings, BackupSettings>('backup.set_settings', { affects: ['backup'] });

export const useSetBackupPassword = () =>
  useCoreMutation<{ password: string }, null>('backup.set_password', {
    affects: ['backup'],
    sensitive: true,
  });

/**
 * Restaura um backup: o núcleo troca o banco e reabre a sessão. Todo dado em
 * cache fica velho, então a interface esquece tudo e guarda o novo estado do cofre.
 */
export function useRestoreBackup() {
  const client = useQueryClient();
  return useMutation<VaultStatus, ApiError, RestoreArgs>({
    mutationFn: (args) => call<VaultStatus>('backup.restore', args),
    gcTime: 0,
    onSuccess: (status) => {
      clearSensitiveCache(client);
      client.setQueryData(vaultKeys.status, status);
    },
  });
}
