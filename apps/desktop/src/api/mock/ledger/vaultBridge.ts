/**
 * Ponte com o cofre do `mock.ts` (sem mexer nele): confere a senha mestra pelo
 * próprio `vault.unlock` (que, com o cofre aberto, só valida a senha) e troca a
 * senha pelo caminho do kit de recuperação, que o mock aceita com o código fixo.
 */
import type { VaultStatus } from '@/api/generated/VaultStatus';

import { MOCK_RECOVERY_CODE } from '../../mockFixtures';
import { wrongPassword } from './errors';

async function vaultCall<T>(cmd: string, args?: unknown): Promise<T> {
  const { mockCall } = await import('../../mock');
  return (await mockCall(cmd, args)) as T;
}

export async function verifyMasterPassword(password: string): Promise<void> {
  try {
    await vaultCall('vault.unlock', { password });
  } catch (e) {
    if ((e as { code?: string }).code === 'wrong_password') wrongPassword();
    throw e;
  }
}

export async function setMasterPassword(password: string): Promise<VaultStatus> {
  return vaultCall<VaultStatus>('vault.unlock_recovery', {
    code: MOCK_RECOVERY_CODE,
    new_password: password,
  });
}

export async function vaultStatus(): Promise<VaultStatus> {
  return vaultCall<VaultStatus>('vault.status');
}
