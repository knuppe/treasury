/** Utilitários comuns dos handlers do mock (argumentos, erros, base padrão). */
import type { Basis } from '@/api/generated/Basis';
import {
  extraMockHandler,
  mockFail,
  registerMockHandlers,
  requireUnlocked,
  type MockHandler,
} from '@/api/mockRegistry';

import { ensureSeeded } from './boot';
import { MockError } from './budgetState';

/** Base padrão: o ajuste `budget_basis` (se outro mock tiver `settings.get`) ou competência. */
export function defaultBasis(): Basis {
  const get = extraMockHandler('settings.get');
  if (get) {
    try {
      const s = get({}) as { budget_basis?: Basis } | null;
      if (s?.budget_basis === 'cash' || s?.budget_basis === 'accrual') return s.budget_basis;
    } catch {
      // Sem ajustes no mock: competência.
    }
  }
  return 'accrual';
}

export function str(args: Record<string, unknown>, name: string): string {
  const v = args[name];
  if (typeof v !== 'string' || v === '')
    mockFail('invalid', `argumentos inválidos: missing field \`${name}\``);
  return v;
}

export function optStr(args: Record<string, unknown>, name: string): string | undefined {
  const v = args[name];
  return typeof v === 'string' && v !== '' ? v : undefined;
}

export function optNum(args: Record<string, unknown>, name: string): number | undefined {
  const v = args[name];
  return typeof v === 'number' && Number.isFinite(v) ? v : undefined;
}

export function basisArg(args: Record<string, unknown>): Basis {
  const b = args.basis;
  return b === 'cash' || b === 'accrual' ? b : defaultBasis();
}

/** `YYYY-MM` (aceita `YYYY-MM-DD`). */
export function monthArg(args: Record<string, unknown>, name: string, fallback?: string): string {
  const v = optStr(args, name) ?? fallback;
  if (!v || !/^\d{4}-\d{2}/.test(v)) mockFail('invalid', `mês inválido: ${v ?? ''}`);
  return v.slice(0, 7);
}

/** Cofre aberto + erros do domínio no formato do núcleo. */
function guard(h: MockHandler): MockHandler {
  return (args) => {
    requireUnlocked();
    // Banco único do mock semeado (e listas vivas de `fixture.ts` em dia).
    ensureSeeded();
    try {
      return h(args);
    } catch (e) {
      if (e instanceof MockError) mockFail(e.code, e.message);
      throw e;
    }
  };
}

export function register(map: Record<string, MockHandler>): void {
  registerMockHandlers(Object.fromEntries(Object.entries(map).map(([k, h]) => [k, guard(h)])));
}

/**
 * Registra só os comandos que nenhum outro mock registrou. Se o mock do dono do módulo
 * carregar depois, ele sobrescreve este; se carregou antes, este não entra.
 */
export function registerFallbacks(map: Record<string, MockHandler>): void {
  const missing = Object.entries(map).filter(([k]) => !extraMockHandler(k));
  register(Object.fromEntries(missing));
}
