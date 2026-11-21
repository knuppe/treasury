/**
 * Hooks TanStack Query para os comandos do cofre.
 *
 * Senhas só passam pelas mutações, que têm `gcTime: 0`: as variáveis (com a
 * senha) saem do cache assim que a mutação termina e ninguém a observa.
 */
import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query';

import type { BackupFile } from '@/api/generated/BackupFile';
import type { RestoreFileArgs } from '@/api/generated/RestoreFileArgs';
import type { RestoreFileResult } from '@/api/generated/RestoreFileResult';
import { useAgentSession } from '@/state/agentSession';

import { call } from './client';
import {
  vaultApi,
  type CreateVaultArgs,
  type UnlockRecoveryArgs,
  type UnlockVaultArgs,
  type VaultStatus,
} from './vault';

export const vaultKeys = {
  all: ['vault'] as const,
  status: ['vault', 'status'] as const,
};

/** Estado do cofre. Revalida ao voltar o foco para a janela (o núcleo pode ter bloqueado). */
export function useVaultStatus() {
  return useQuery({
    queryKey: vaultKeys.status,
    queryFn: vaultApi.status,
    staleTime: Infinity,
    refetchOnWindowFocus: true,
  });
}

/**
 * Avisa quando o cofre passa de bloqueado a aberto, venha o desbloqueio de onde vier
 * (senha, PIN, Touch ID, partes, chave física). Criar um cofre novo não conta.
 */
export function watchVaultUnlock(
  client: QueryClient,
  onUnlock: (status: VaultStatus) => void,
): () => void {
  const first = client.getQueryData<VaultStatus>(vaultKeys.status);
  let last = first?.state;
  let lastVault = first?.vault_id;
  return client.getQueryCache().subscribe((event) => {
    if (event.type !== 'updated') return;
    const key = event.query.queryKey as readonly unknown[];
    if (key.length !== 2 || key[0] !== vaultKeys.status[0] || key[1] !== vaultKeys.status[1]) {
      return;
    }
    const status = event.query.state.data as VaultStatus | undefined;
    const next = status?.state;
    if (!status || next === undefined) return;
    const prev = last;
    const prevVault = lastVault;
    last = next;
    lastVault = status.vault_id;
    // Cofre aberto agora, ou outro cofre já aberto (a demonstração criada com um cofre aberto).
    const opened = prev === 'locked' && next === 'unlocked';
    const switched = next === 'unlocked' && prevVault != null && prevVault !== status.vault_id;
    if (opened || switched) onUnlock(status);
  });
}

function setStatus(client: QueryClient, status: VaultStatus) {
  client.setQueryData(vaultKeys.status, status);
}

/**
 * Esquece todo dado em memória que não seja o estado do cofre: consultas de
 * dados financeiros e mutações (com seus argumentos) já feitas.
 */
export function clearSensitiveCache(client: QueryClient): void {
  // O estado do cofre e o aceite dos termos de uso (fora do cofre) ficam.
  const sensitive = (key: readonly unknown[]) => key[0] !== vaultKeys.all[0] && key[0] !== 'terms';
  client.cancelQueries({ predicate: (q) => sensitive(q.queryKey) }).catch(() => {});
  client.removeQueries({ predicate: (q) => sensitive(q.queryKey) });
  client.getMutationCache().clear();
  // Liberações e estado da conversa com o agente valem só na sessão desbloqueada.
  useAgentSession.getState().reset();
}

export function useCreateVault() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: CreateVaultArgs) => vaultApi.create(args),
    gcTime: 0,
    onSuccess: (status) => setStatus(client, status),
  });
}

export function useUnlockVault() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: UnlockVaultArgs) => vaultApi.unlock(args),
    gcTime: 0,
    onSuccess: (status) => setStatus(client, status),
  });
}

/** Demonstração sem senha: abre pela chave fixa do app. */
export function useUnlockDemo() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => vaultApi.unlockDemo(),
    gcTime: 0,
    onSuccess: (status) => setStatus(client, status),
  });
}

export function useUnlockRecovery() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: UnlockRecoveryArgs) => vaultApi.unlockRecovery(args),
    gcTime: 0,
    onSuccess: (status) => setStatus(client, status),
  });
}

/**
 * Bloqueia o cofre e limpa o cache do TanStack Query. Mesmo se o núcleo falhar,
 * a interface limpa o cache e volta para a tela de desbloqueio.
 */
export function useLockVault() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => vaultApi.lock(),
    gcTime: 0,
    onSettled: (status) => {
      clearSensitiveCache(client);
      const previous = client.getQueryData<VaultStatus>(vaultKeys.status);
      if (status) setStatus(client, status);
      else if (previous) setStatus(client, { ...previous, state: 'locked' });
    },
  });
}

/** Avisa o núcleo que há atividade (não precisa de cache nem de estado). */
export function touchVault(): void {
  vaultApi.touch().catch(() => {
    // Cofre já bloqueado ou núcleo indisponível: o próximo status mostra.
  });
}

/** Backups da pasta do cofre (e os importados), mesmo com o cofre fechado ou quebrado. */
export function useBackupFiles() {
  return useQuery({
    queryKey: ['backup', 'scan'] as const,
    queryFn: () => call<BackupFile[]>('backup.scan'),
    gcTime: 0,
  });
}

/**
 * Restaura um arquivo da pasta de backups (docs/backup-recovery.md). Com o cofre fechado, o
 * cofre que não abre vai para o lado (nunca é apagado) e o backup entra no lugar.
 */
export function useRestoreBackupFile() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (args: RestoreFileArgs) => call<RestoreFileResult>('backup.restore_file', args),
    gcTime: 0,
    onSuccess: (r) => setStatus(client, r.status),
  });
}

/** Traz um backup de fora (pen drive, HD, nuvem) para a pasta `importados/`. */
export function useImportBackupFile() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (source: string) => call<BackupFile>('backup.import_file', { source }),
    onSuccess: () => void client.invalidateQueries({ queryKey: ['backup'] }),
  });
}

/** Copia um backup da pasta para o caminho escolhido no diálogo "Salvar como". */
export function useExportBackupFile() {
  return useMutation({
    mutationFn: (args: { path: string; dest: string }) => call<null>('backup.export_file', args),
  });
}
