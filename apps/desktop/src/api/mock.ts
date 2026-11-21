/**
 * Núcleo falso, em memória, para desenvolvimento no navegador e para testes.
 *
 * Implementa os comandos do cofre com as mesmas formas de resposta e de erro
 * do núcleo Rust (`{ code, message }` em pt-BR). Nada é gravado em disco nem no
 * navegador: recarregar a página volta ao estado inicial.
 *
 * Estado inicial em desenvolvimento: `VITE_TREASURY_MOCK_SEED=empty|locked|unlocked`
 * (padrão `empty`). Nos cofres semeados, a senha é `MOCK_PASSWORD`.
 */
import type { ApiErrorPayload } from './client';
import { MOCK_DB_PATH, MOCK_PASSWORD, MOCK_RECOVERY_CODE } from './mockFixtures';
import {
  extraMockHandler,
  runMockResetters,
  setMockCreatedCountry,
  setMockHomeCountry,
  setUnlockedProbe,
  vaultStatusPatch,
} from './mockRegistry';
import type { VaultStatus } from './vault';
import { COUNTRIES } from './mock/countryList';

export { MOCK_DB_PATH, MOCK_PASSWORD, MOCK_RECOVERY_CODE };

export type MockSeed = 'empty' | 'locked' | 'unlocked';

export interface MockOptions {
  autoLockMinutes?: number;
  householdName?: string;
  password?: string;
  keychainAvailable?: boolean;
  /** Atraso simulado (ms). Padrão: 0 nos testes; ~0,4 s para criar/abrir (Argon2id). */
  latencyMs?: number;
  /**
   * País da família (padrão: Brasil). A família de exemplo nasce nele: país do cofre, moeda
   * base e as contas que eram em reais na moeda do país (`US` → USD, `PT` → EUR).
   */
  country?: string;
}

interface MockVault {
  password: string;
  householdName: string;
  members: string[];
  recoveryCode: string;
}

/** Um cofre da lista (docs/vaults.md); `vault` nulo: registrado, ainda sem criar. */
interface MockVaultEntry {
  id: string;
  name: string;
  purpose: string | null;
  path: string;
  vault: MockVault | null;
  /** Cofre de demonstração (`vaults.new_demo`). */
  demo?: boolean;
  /** Demonstração que abre sem senha (`vault.unlock_demo`). */
  noPassword?: boolean;
}

/** Chave fixa das demonstrações sem senha (no núcleo, `demo::DEMO_PASSWORD`). */
const DEMO_PASSWORD = 'treasury-demonstracao-sem-senha-dados-ficticios';

interface MockState {
  vault: MockVault | null;
  /** Cofres conhecidos; o ativo guarda o `vault` acima. */
  vaults: MockVaultEntry[];
  activeId: string;
  unlocked: boolean;
  autoLockMinutes: number;
  keychainAvailable: boolean;
  latencyMs: number | null;
  /** Cofre que não abre com a senha certa (docs/backup-recovery.md). */
  damaged?: boolean;
}

const isTest = import.meta.env.MODE === 'test';

function seedState(seed: MockSeed, options: MockOptions = {}): MockState {
  const vault: MockVault | null =
    seed === 'empty'
      ? null
      : {
          password: options.password ?? MOCK_PASSWORD,
          householdName: options.householdName ?? 'Família Exemplo',
          members: ['Ana', 'Bruno'],
          recoveryCode: MOCK_RECOVERY_CODE,
        };
  return {
    vault,
    vaults: [{ id: 'principal', name: 'Principal', purpose: null, path: MOCK_DB_PATH, vault }],
    activeId: 'principal',
    unlocked: seed === 'unlocked',
    autoLockMinutes: options.autoLockMinutes ?? 15,
    keychainAvailable: options.keychainAvailable ?? false,
    latencyMs: options.latencyMs ?? null,
  };
}

function initialSeed(): MockSeed {
  const seed = import.meta.env.VITE_TREASURY_MOCK_SEED;
  return seed === 'locked' || seed === 'unlocked' ? seed : seed === 'damaged' ? 'locked' : 'empty';
}

let state: MockState = seedState(isTest ? 'empty' : initialSeed());
// `VITE_TREASURY_MOCK_SEED=damaged`: cofre fechado que não abre, para ver a recuperação.
if (!isTest && import.meta.env.VITE_TREASURY_MOCK_SEED === 'damaged') state.damaged = true;
let log: string[] = [];

setUnlockedProbe(() => state.unlocked);
// Mocks dos módulos de domínio (src/api/mock/*.ts) se registram ao serem carregados.
import.meta.glob('./mock/*.ts', { eager: true });

/** Recomeça o mock (usado pelos testes e pelo catálogo de componentes). */
export function resetMock(seed: MockSeed = 'empty', options: MockOptions = {}): void {
  state = seedState(seed, options);
  log = [];
  runMockResetters();
  const country = options.country?.trim().toUpperCase() || 'BR';
  setMockHomeCountry(country);
  setMockCreatedCountry(country === 'BR' ? null : country);
}

/** Comandos recebidos desde o último `resetMock` (sem argumentos: podem ter senhas). */
export function mockCallLog(): readonly string[] {
  return log;
}

function activeEntry(): MockVaultEntry {
  return state.vaults.find((v) => v.id === state.activeId) ?? state.vaults[0]!;
}

function status(): VaultStatus {
  const active = activeEntry();
  return {
    state: !state.vault ? 'no_vault' : state.unlocked ? 'unlocked' : 'locked',
    db_path: active.path,
    vault_id: active.id,
    vault_name: active.name,
    vault_count: state.vaults.length,
    auto_lock_minutes: state.autoLockMinutes,
    keychain_available: state.keychainAvailable,
    keychain_enabled: false,
    pin_enabled: false,
    fido2_keys: 0,
    household_name: state.vault?.householdName ?? null,
    demo: state.unlocked && Boolean(active.demo),
    demo_no_password: Boolean(active.noPassword),
    newer_app_version: null,
    ...(vaultStatusPatch() as Partial<VaultStatus>),
  };
}

function fail(code: string, message: string, details?: unknown): never {
  const payload: ApiErrorPayload = { code, message, ...(details ? { details } : {}) };
  // Como o núcleo via Tauri: rejeita com o objeto de erro, não com um Error.
  // eslint-disable-next-line @typescript-eslint/only-throw-error
  throw payload;
}

function record(args: unknown): Record<string, unknown> {
  return typeof args === 'object' && args !== null ? (args as Record<string, unknown>) : {};
}

function stringArg(args: Record<string, unknown>, name: string): string {
  const value = args[name];
  if (typeof value !== 'string') fail('invalid', `Argumento ausente ou inválido: ${name}.`);
  return value;
}

function checkPassword(password: string): void {
  if (password.length < 8) {
    fail('weak_password', 'A senha mestra precisa ter pelo menos 8 caracteres.');
  }
}

const normalizeCode = (code: string) => code.toUpperCase().replace(/[^A-Z0-9]/g, '');

const SLOW_COMMANDS = new Set(['vault.create', 'vault.unlock', 'vault.unlock_recovery']);

async function delay(cmd: string): Promise<void> {
  const ms = state.latencyMs ?? (isTest ? 0 : SLOW_COMMANDS.has(cmd) ? 420 : 40);
  if (ms > 0) await new Promise((resolve) => setTimeout(resolve, ms));
  else await Promise.resolve();
}

const handlers: Record<string, (args: Record<string, unknown>) => unknown> = {
  'vault.status': () => status(),

  'vault.create': (args) => {
    if (state.vault) fail('vault_exists', 'Já existe um cofre neste computador.');
    const password = stringArg(args, 'password');
    const householdName = stringArg(args, 'household_name').trim();
    const members = Array.isArray(args.members)
      ? args.members.filter((m): m is string => typeof m === 'string' && m.trim() !== '')
      : [];
    if (!householdName) fail('invalid', 'Informe o nome da família.');
    if (members.length === 0) fail('invalid', 'Informe pelo menos um membro.');
    checkPassword(password);
    const country = typeof args.country === 'string' ? args.country.trim().toUpperCase() : 'BR';
    if (!COUNTRIES.some((c) => c[0] === country)) fail('invalid', `país fora da lista: ${country}`);
    state.vault = {
      password,
      householdName,
      members: members.map((m) => m.trim()),
      recoveryCode: MOCK_RECOVERY_CODE,
    };
    activeEntry().vault = state.vault;
    // Como no núcleo, o cofre nasce na moeda do país escolhido (a família de exemplo também).
    setMockHomeCountry(country);
    setMockCreatedCountry(country === 'BR' ? null : country);
    state.unlocked = true;
    return status();
  },

  'vault.unlock_demo': () => {
    const active = activeEntry();
    if (!state.vault) fail('no_vault', 'Nenhum cofre encontrado neste computador.');
    if (!active.noPassword)
      fail('invalid', 'este cofre não é uma demonstração sem senha: use a senha');
    state.unlocked = true;
    return status();
  },
  'vault.unlock': (args) => {
    if (!state.vault) fail('no_vault', 'Nenhum cofre encontrado neste computador.');
    const password = stringArg(args, 'password');
    if (password !== state.vault.password) fail('wrong_password', 'Senha incorreta.');
    if (state.damaged && !state.unlocked)
      fail(
        'vault_damaged',
        'Essa senha abre os seus backups, mas não abre o cofre. O cofre pode estar danificado (ou a senha mudou depois desses backups). Recupere de um backup.',
      );
    state.unlocked = true;
    return status();
  },

  'vault.unlock_recovery': (args) => {
    if (!state.vault) fail('no_vault', 'Nenhum cofre encontrado neste computador.');
    const code = stringArg(args, 'code');
    const newPassword = stringArg(args, 'new_password');
    if (normalizeCode(code) !== normalizeCode(state.vault.recoveryCode)) {
      fail('wrong_recovery_code', 'Código de recuperação inválido.');
    }
    checkPassword(newPassword);
    state.vault.password = newPassword;
    state.unlocked = true;
    return status();
  },

  'vault.lock': () => {
    state.unlocked = false;
    return status();
  },

  // Vários cofres (como `vaults.*` do núcleo): os dados de domínio do mock são os mesmos
  // em todos; só a lista, o cofre ativo e a senha de cada um mudam.
  'vaults.list': () => vaultList(),
  'vaults.select': (args) => {
    selectVault(stringArg(args, 'id'));
    return status();
  },
  'vaults.new': (args) => {
    const name = vaultName(args.name);
    const id = vaultId(name);
    const path =
      typeof args.path === 'string' && args.path.trim()
        ? args.path.trim()
        : MOCK_DB_PATH.replace(/[^/]+$/, `${id}.db`);
    if (state.vaults.some((v) => v.path === path))
      fail('conflict', 'Este caminho já está na lista de cofres.');
    state.vaults.push({ id, name, purpose: purpose(args.purpose), path, vault: null });
    selectVault(id);
    return status();
  },
  // Cofre de demonstração: no mock os dados de domínio são os da família de exemplo.
  'vaults.new_demo': (args) => {
    const name = vaultName(args.name);
    const household = typeof args.household_name === 'string' ? args.household_name.trim() : '';
    if (!household) fail('invalid', 'informe o nome da família');
    const typed = typeof args.password === 'string' ? args.password : '';
    if (typed) checkPassword(typed);
    const password = typed || DEMO_PASSWORD;
    const profile = record(args.profile);
    const months = typeof profile.months === 'number' ? profile.months : 0;
    if (months < 3 || months > 36) fail('invalid', 'o histórico vai de 3 a 36 meses');
    const adults = Array.isArray(profile.adults) ? (profile.adults as { name: string }[]) : [];
    if (adults.length < 1 || adults.length > 2) fail('invalid', 'informe um ou dois adultos');
    const children = Array.isArray(profile.children) ? (profile.children as string[]) : [];
    const country =
      typeof profile.country === 'string' && profile.country.trim()
        ? profile.country.trim().toUpperCase()
        : 'BR';
    if (!COUNTRIES.some((c) => c[0] === country)) fail('invalid', `país fora da lista: ${country}`);
    const vault: MockVault = {
      password,
      householdName: household,
      members: [...adults.map((a) => a.name), ...children],
      recoveryCode: MOCK_RECOVERY_CODE,
    };
    const entry = {
      name,
      purpose: 'Demonstração: dados fictícios gerados pelo assistente',
      vault,
      demo: true,
      ...(typed ? {} : { noPassword: true }),
    };
    // Como no núcleo: o cofre escolhido ainda sem arquivo vira a demonstração.
    const active = activeEntry();
    if (!active.vault) {
      Object.assign(active, entry);
      state.vault = vault;
    } else {
      const id = vaultId(name);
      state.vaults.push({ id, path: MOCK_DB_PATH.replace(/[^/]+$/, `${id}.db`), ...entry });
      selectVault(id);
    }
    setMockHomeCountry(country);
    setMockCreatedCountry(country === 'BR' ? null : country);
    state.unlocked = true;
    const today = new Date().toISOString().slice(0, 10);
    const from = new Date();
    from.setMonth(from.getMonth() - (months - 1), 1);
    return {
      status: status(),
      summary: {
        from: from.toISOString().slice(0, 10),
        to: today,
        accounts: 5,
        transactions: months * 48,
        assets: 9,
        loans: profile.car_installment || profile.housing === 'mortgage' ? 1 : 0,
      },
    };
  },
  'vaults.add': (args) => {
    const path = stringArg(args, 'path').trim();
    if (!path.startsWith('/')) fail('invalid', 'Informe o caminho completo do arquivo.');
    if (state.vaults.some((v) => v.path === path))
      fail('conflict', 'Este arquivo já está na lista de cofres.');
    const stem =
      path
        .split('/')
        .pop()!
        .replace(/\.[^.]+$/, '') || 'Cofre';
    const name = vaultName(typeof args.name === 'string' ? args.name : stem);
    const id = vaultId(name);
    state.vaults.push({
      id,
      name,
      purpose: purpose(args.purpose),
      path,
      vault: {
        password: MOCK_PASSWORD,
        householdName: name,
        members: ['Ana'],
        recoveryCode: MOCK_RECOVERY_CODE,
      },
    });
    if (args.select !== false) selectVault(id);
    return status();
  },
  'vaults.update': (args) => {
    const v =
      state.vaults.find((x) => x.id === args.id) ?? fail('not_found', 'Cofre não encontrado.');
    if (typeof args.name === 'string') v.name = vaultName(args.name);
    if ('purpose' in args) v.purpose = purpose(args.purpose);
    return vaultList();
  },
  // Com `delete_file`, no núcleo o arquivo vai para a Lixeira; no mock só sai da lista.
  'vaults.remove': (args) => {
    if (args.id === state.activeId)
      fail('invalid', 'Este é o cofre em uso: troque de cofre antes de tirá-lo da lista.');
    const before = state.vaults.length;
    state.vaults = state.vaults.filter((v) => v.id !== args.id);
    if (state.vaults.length === before) fail('not_found', 'Cofre não encontrado.');
    return vaultList();
  },
  'vaults.copy': (args) => {
    const src =
      state.vaults.find((x) => x.id === args.id) ?? fail('not_found', 'Cofre não encontrado.');
    if (!src.vault) fail('invalid', 'este cofre ainda não foi criado');
    const name = vaultName(
      typeof args.name === 'string' && args.name.trim()
        ? args.name
        : `${src.name.slice(0, 52)} (cópia)`,
    );
    const id = vaultId(name);
    state.vaults.push({
      id,
      name,
      purpose: src.purpose,
      path: MOCK_DB_PATH.replace(/[^/]+$/, `${id}.db`),
      vault: { ...src.vault },
      ...(src.demo ? { demo: true } : {}),
      ...(src.noPassword ? { noPassword: true } : {}),
    });
    return vaultList();
  },

  'vault.touch': () => {
    if (!state.unlocked) fail('vault_locked', 'O cofre está bloqueado.');
    return null;
  },
};

function vaultName(name: unknown): string {
  const n = typeof name === 'string' ? name.trim() : '';
  if (!n) fail('invalid', 'Dê um nome ao cofre.');
  return n;
}

const purpose = (p: unknown) => (typeof p === 'string' && p.trim() ? p.trim() : null);

function vaultId(name: string): string {
  const base =
    name
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, '')
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '') || 'cofre';
  let id = base;
  for (let n = 2; state.vaults.some((v) => v.id === id); n++) id = `${base}-${n}`;
  return id;
}

function selectVault(id: string) {
  const next = state.vaults.find((v) => v.id === id) ?? fail('not_found', 'Cofre não encontrado.');
  if (next.id === state.activeId) return;
  state.unlocked = false;
  state.activeId = next.id;
  state.vault = next.vault;
}

function vaultList() {
  return {
    active_id: state.activeId,
    data_dir: MOCK_DB_PATH.replace(/\/[^/]+$/, ''),
    vaults: state.vaults.map((v) => ({
      id: v.id,
      name: v.name,
      purpose: v.purpose,
      path: v.path,
      exists: Boolean(v.vault),
      active: v.id === state.activeId,
      bytes: v.vault ? 4_718_592 : null,
      demo: Boolean(v.noPassword),
    })),
  };
}

/** O cofre do mock está aberto? */
export function isMockUnlocked(): boolean {
  return state.unlocked;
}

/** Deixa o cofre do mock danificado: a senha certa não abre (docs/backup-recovery.md). */
export function seedVaultDamaged(): void {
  state.damaged = true;
  state.unlocked = false;
}

/**
 * Restaura o cofre do mock de um backup, com o cofre fechado: a senha tem de ser a do backup
 * (no mock, a do cofre). Devolve onde ficou o cofre que não abria.
 */
export function mockRestoreLockedVault(password: string): string {
  if (!state.vault) fail('no_vault', 'Nenhum cofre encontrado neste computador.');
  if (password !== state.vault.password) fail('wrong_password', 'Senha incorreta.');
  state.damaged = false;
  state.unlocked = true;
  return `${MOCK_DB_PATH}.danificado-20261022-101500`;
}

/** Desbloqueia o cofre do mock sem senha (PIN, Touch ID, partes, chave física simulados). */
export function mockUnlockVault(): void {
  if (!state.vault) fail('no_vault', 'Nenhum cofre encontrado neste computador.');
  state.unlocked = true;
}

/** Executa um comando no núcleo falso. Rejeita com `{ code, message }`. */
export async function mockCall(cmd: string, args: unknown): Promise<unknown> {
  log.push(cmd);
  await delay(cmd);
  const handler = handlers[cmd] ?? extraMockHandler(cmd);
  if (!handler) fail('unknown_command', `Comando desconhecido: ${cmd}.`);
  return handler(record(args));
}
