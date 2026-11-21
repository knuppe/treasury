/**
 * Mock dos comandos extras do cofre: trocar a senha mestra, kit de recuperação e
 * desbloqueio pelo keychain. A senha é conferida pelo cofre do `mock.ts`.
 */
import type { Fido2Slot } from '@/api/generated/Fido2Slot';
import type { RecoveryKit } from '@/api/generated/RecoveryKit';
import type { RecoveryShares } from '@/api/generated/RecoveryShares';
import type { RecoveryStatus } from '@/api/generated/RecoveryStatus';
import type { VaultStatus } from '@/api/generated/VaultStatus';

import { MOCK_DB_PATH, MOCK_RECOVERY_CODE } from '../mockFixtures';
import {
  onMockReset,
  registerMockHandlers,
  requireUnlocked,
  setVaultStatusPatch,
} from '../mockRegistry';
import { fail, invalid, str } from './ledger/errors';
import { setMasterPassword, vaultStatus, verifyMasterPassword } from './ledger/vaultBridge';

let keychainEnabled = false;
let pin: string | null = null;
let pinFailures = 0;
let sharesAt: string | null = null;
let kitAt: string | null = null;
let fido2: Fido2Slot[] = [];
/** Versão mais nova do app que já abriu o cofre (testes do aviso de versão). */
let newerAppVersion: string | null = null;
onMockReset(() => {
  keychainEnabled = false;
  pin = null;
  pinFailures = 0;
  sharesAt = null;
  kitAt = null;
  fido2 = [];
  newerAppVersion = null;
});

export function setMockNewerAppVersion(v: string | null): void {
  newerAppVersion = v;
}

setVaultStatusPatch(() => ({
  keychain_enabled: keychainEnabled,
  pin_enabled: pin !== null,
  fido2_keys: fido2.length,
  newer_app_version: newerAppVersion,
}));

/** Partes fixas do mock: quaisquer duas destas recuperam o cofre. */
export const MOCK_SHARES = [
  'TRSP-AEBA-QCQM-MOCK-PART-UM00',
  'TRSP-AECA-QCQM-MOCK-PART-DOIS',
  'TRSP-AEDA-QCQM-MOCK-PART-TRES',
];

async function status(): Promise<VaultStatus> {
  return {
    ...(await vaultStatus()),
    keychain_enabled: keychainEnabled,
    pin_enabled: pin !== null,
    fido2_keys: fido2.length,
  };
}

async function openWith(newPassword: string): Promise<VaultStatus> {
  if ([...newPassword].length < 10)
    fail('weak_password', 'a senha mestra precisa de pelo menos 10 caracteres');
  await setMasterPassword(newPassword);
  return status();
}

registerMockHandlers({
  'vault.change_password': async (args) => {
    requireUnlocked();
    const current = str(args, 'current');
    const next = str(args, 'new');
    if ([...next].length < 10)
      fail('weak_password', 'a senha mestra precisa de pelo menos 10 caracteres');
    await verifyMasterPassword(current);
    await setMasterPassword(next);
    return status();
  },
  'vault.recovery_kit': async (args): Promise<RecoveryKit> => {
    requireUnlocked();
    await verifyMasterPassword(str(args, 'password'));
    kitAt = new Date().toISOString();
    return {
      code: MOCK_RECOVERY_CODE,
      created_at: kitAt,
      db_path: MOCK_DB_PATH,
    };
  },
  'vault.keychain_enable': async (args) => {
    requireUnlocked();
    const s = await vaultStatus();
    if (!s.keychain_available) invalid('keychain indisponível neste sistema');
    await verifyMasterPassword(str(args, 'password'));
    keychainEnabled = true;
    return status();
  },
  'vault.keychain_disable': async () => {
    keychainEnabled = false;
    return status();
  },
  'vault.pin_enable': async (args) => {
    requireUnlocked();
    await verifyMasterPassword(str(args, 'password'));
    const p = str(args, 'pin');
    if (!/^\d{6,12}$/.test(p)) invalid('o PIN precisa ter de 6 a 12 dígitos');
    if (/^(\d)\1+$/.test(p) || '0123456789'.includes(p) || '9876543210'.includes(p))
      invalid('PIN fácil demais (repetido ou em sequência): escolha outro');
    pin = p;
    pinFailures = 0;
    return status();
  },
  'vault.pin_disable': async () => {
    pin = null;
    return status();
  },
  'vault.unlock_pin': async (args) => {
    if (pin === null) invalid('o PIN não está ligado neste computador');
    if (str(args, 'pin') !== pin) {
      pinFailures += 1;
      if (pinFailures >= 5) {
        pin = null;
        fail('wrong_pin', 'PIN errado 5 vezes: o PIN foi desligado. Entre com a senha mestra.');
      }
      fail(
        'wrong_pin',
        `PIN incorreto. Restam ${5 - pinFailures} tentativa(s) antes de exigir a senha mestra.`,
      );
    }
    pinFailures = 0;
    const { mockUnlockVault } = await import('../mock');
    mockUnlockVault();
    return status();
  },
  'vault.recovery_status': (): RecoveryStatus => {
    requireUnlocked();
    return {
      shares_created_at: sharesAt,
      fido2: [...fido2],
      kit_created_at: kitAt,
      kit_outdated: false,
    };
  },
  'vault.recovery_shares_create': async (args): Promise<RecoveryShares> => {
    requireUnlocked();
    await verifyMasterPassword(str(args, 'password'));
    sharesAt = new Date().toISOString();
    const removed = args.rotate === true ? fido2.length : 0;
    if (args.rotate === true) fido2 = [];
    return { shares: [...MOCK_SHARES], created_at: sharesAt, fido2_removed: removed };
  },
  'vault.unlock_shares': async (args) => {
    const shares = Array.isArray(args.shares) ? (args.shares as string[]) : [];
    const valid = new Set(shares.filter((s) => MOCK_SHARES.includes(s.trim())));
    if (valid.size < 2) invalid('as partes não reconstroem a chave (alguma foi digitada errado?)');
    return openWith(str(args, 'new_password'));
  },
  'vault.fido2_slots': () => [...fido2],
  'vault.fido2_remove': async (args) => {
    requireUnlocked();
    await verifyMasterPassword(str(args, 'password'));
    fido2 = fido2.filter((f) => f.id !== str(args, 'id'));
    return null;
  },
});
