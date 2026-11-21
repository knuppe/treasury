/**
 * Registro de comandos extras do núcleo falso.
 *
 * Cada módulo de domínio pode ter seu mock em `src/api/mock/<modulo>.ts`, que
 * chama `registerMockHandlers` ao ser importado; o `mock.ts` carrega todos os
 * arquivos dessa pasta automaticamente. Fica separado do `mock.ts` para evitar
 * importação circular.
 */
import type { ApiErrorPayload } from './client';

export type MockHandler = (args: Record<string, unknown>) => unknown;

const handlers: Record<string, MockHandler> = {};
const resetters: (() => void)[] = [];
let unlockedProbe: () => boolean = () => true;

export function registerMockHandlers(map: Record<string, MockHandler>): void {
  Object.assign(handlers, map);
}

/** Chamado a cada `resetMock`, para o módulo voltar ao estado inicial. */
export function onMockReset(reset: () => void): void {
  resetters.push(reset);
}

export function extraMockHandler(cmd: string): MockHandler | undefined {
  return handlers[cmd];
}

export function runMockResetters(): void {
  for (const r of resetters) r();
}

export function setUnlockedProbe(probe: () => boolean): void {
  unlockedProbe = probe;
}

/** O cofre do mock está aberto? Comandos de domínio devem falhar com `vault_locked` se não. */
export function mockIsUnlocked(): boolean {
  return unlockedProbe();
}

/** Rejeita como o núcleo: com o objeto `{ code, message }`. */
export function mockFail(code: string, message: string, details?: unknown): never {
  const payload: ApiErrorPayload = { code, message, ...(details ? { details } : {}) };
  // eslint-disable-next-line @typescript-eslint/only-throw-error
  throw payload;
}

export function requireUnlocked(): void {
  if (!mockIsUnlocked()) mockFail('vault_locked', 'O cofre está bloqueado.');
}

/** Campos do estado do cofre que outros mocks controlam (PIN, keychain, chaves físicas). */
let statusPatch: () => Record<string, unknown> = () => ({});

export function setVaultStatusPatch(fn: () => Record<string, unknown>): void {
  statusPatch = fn;
}

export function vaultStatusPatch(): Record<string, unknown> {
  return statusPatch();
}

/** País escolhido na criação do cofre (o mock de `country.*` aplica na primeira consulta). */
let createdCountry: string | null = null;

/**
 * País da família de exemplo: a semente do livro-caixa nasce na moeda dele (as contas em reais
 * da família fictícia passam para a moeda do país), como um cofre novo do núcleo nasce na moeda
 * do país escolhido. Muda com `resetMock(…, { country })` e com a criação do cofre.
 */
let homeCountry = 'BR';

export function mockHomeCountry(): string {
  return homeCountry;
}

export function setMockHomeCountry(code: string): void {
  homeCountry = code.trim().toUpperCase() || 'BR';
}

export function setMockCreatedCountry(code: string | null): void {
  createdCountry = code;
}

export function takeMockCreatedCountry(): string | null {
  const c = createdCountry;
  createdCountry = null;
  return c;
}
