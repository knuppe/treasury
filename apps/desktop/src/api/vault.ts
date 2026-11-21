/**
 * Comandos do cofre (senha mestra, bloqueio, recuperação).
 *
 * Tipos declarados à mão por enquanto; quando o ts-rs gerar os tipos do Rust
 * em `src/api/generated/`, estes passam a ser reexportados de lá.
 */
import { call } from './client';

export type VaultState = 'no_vault' | 'locked' | 'unlocked';

export interface VaultStatus {
  state: VaultState;
  /** Caminho do arquivo do banco (ex.: `~/.treasury/data/treasury.db`). */
  db_path: string;
  /** Minutos de inatividade até o bloqueio automático (0 = desligado). */
  auto_lock_minutes: number;
  keychain_available: boolean;
  keychain_enabled: boolean;
  /** PIN ligado neste computador. */
  pin_enabled: boolean;
  /** Chaves físicas FIDO2 cadastradas para recuperação. */
  fido2_keys: number;
  household_name: string | null;
  /** Cofre ativo (docs/vaults.md). */
  vault_id: string;
  vault_name: string;
  /** Cofres na lista (1: só o principal). */
  vault_count: number;
  /** Cofre de demonstração (dados fictícios do assistente; docs/demo.md). */
  demo: boolean;
  /** O cofre ativo é uma demonstração que abre sem senha (`vault.unlock_demo`). */
  demo_no_password: boolean;
  /** Uma versão mais nova do app já abriu este cofre (sugira atualizar). */
  newer_app_version: string | null;
}

export interface CreateVaultArgs {
  password: string;
  household_name: string;
  /** Membros da família. */
  members: string[];
  /** País onde a família mora (ISO 3166-1 alfa-2; padrão: Brasil). */
  country?: string;
}

export interface UnlockVaultArgs {
  password: string;
}

export interface UnlockRecoveryArgs {
  /** Código do kit de recuperação. */
  code: string;
  new_password: string;
}

/** Códigos de erro do cofre que a interface trata de forma específica. */
export const VAULT_ERRORS = {
  wrongPassword: 'wrong_password',
  /** A senha abre os backups, mas não o cofre (docs/backup-recovery.md). */
  vaultDamaged: 'vault_damaged',
  wrongRecoveryCode: 'wrong_recovery_code',
  vaultExists: 'vault_exists',
  noVault: 'no_vault',
  locked: 'vault_locked',
  weakPassword: 'weak_password',
  invalidArgs: 'invalid',
} as const;

export const vaultApi = {
  status: () => call<VaultStatus>('vault.status'),
  create: (args: CreateVaultArgs) => call<VaultStatus>('vault.create', args),
  unlock: (args: UnlockVaultArgs) => call<VaultStatus>('vault.unlock', args),
  /** Demonstração sem senha: abre pela chave fixa do app (docs/demo.md). */
  unlockDemo: () => call<VaultStatus>('vault.unlock_demo', {}),
  unlockRecovery: (args: UnlockRecoveryArgs) => call<VaultStatus>('vault.unlock_recovery', args),
  lock: () => call<VaultStatus>('vault.lock'),
  /** Avisa o núcleo que há atividade, para adiar o bloqueio automático. */
  touch: () => call<null>('vault.touch'),
};
