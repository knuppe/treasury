/**
 * Desbloqueios que dependem do app (Tauri): Touch ID / senha do Mac antes do Keychain e a
 * chave física FIDO2. Os segredos ficam no Rust; aqui só vão rótulos, PINs da chave física
 * e a senha nova (na recuperação). Fora do app, nada disso existe.
 */
import { ApiError, isTauri } from '@/api/client';
import type { Fido2Slot } from '@/api/generated/Fido2Slot';
import type { VaultStatus } from '@/api/vault';
import { t } from '@/i18n';

export interface BiometricStatus {
  available: boolean;
  /** `touch_id` (biometria) ou `password` (senha do Mac). */
  kind: 'touch_id' | 'password' | null;
}

async function invokeApp<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new ApiError('unavailable', t('cofre-so-no-app'));
  const { invoke } = await import('@tauri-apps/api/core');
  try {
    return await invoke<T>(cmd, args);
  } catch (reason) {
    const r = reason as { code?: string; message?: string } | string;
    if (typeof r === 'string') throw new ApiError('transport', r);
    throw new ApiError(r.code ?? 'transport', r.message ?? 'Falha no app.');
  }
}

export async function biometricStatus(): Promise<BiometricStatus> {
  if (!isTauri()) return { available: false, kind: null };
  try {
    return await invokeApp<BiometricStatus>('biometric_status');
  } catch {
    return { available: false, kind: null };
  }
}

export const unlockBiometric = () => invokeApp<VaultStatus>('unlock_biometric');

export async function fido2Present(): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    return await invokeApp<boolean>('fido2_present');
  } catch {
    return false;
  }
}

export const fido2Register = (label: string, pin?: string) =>
  invokeApp<Fido2Slot>('fido2_register', { label, pin: pin || null });

export const fido2Unlock = (slotId: string, newPassword: string, pin?: string) =>
  invokeApp<VaultStatus>('fido2_unlock', { slotId, newPassword, pin: pin || null });
