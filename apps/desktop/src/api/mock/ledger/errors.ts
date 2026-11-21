/**
 * Erros do mock com o mesmo `code` e o mesmo texto do núcleo (`crate::error::Error`):
 * `invalid` usa a mensagem pura, `not_found` vira "não encontrado: …" e `conflict`
 * vira "conflito: …".
 */
import { mockFail, requireUnlocked } from '../../mockRegistry';

export type CoreErrorCode =
  | 'invalid'
  | 'not_found'
  | 'conflict'
  | 'wrong_password'
  | 'wrong_pin'
  | 'wrong_recovery_code'
  | 'schema_too_new'
  | 'no_engine'
  | 'weak_password'
  | 'forbidden'
  | 'unknown_command'
  | 'other';

export function fail(code: CoreErrorCode, message: string): never {
  return mockFail(code, message);
}

export function invalid(message: string): never {
  return mockFail('invalid', message);
}

export function notFound(what: string): never {
  return mockFail('not_found', `não encontrado: ${what}`);
}

export function conflict(message: string): never {
  return mockFail('conflict', `conflito: ${message}`);
}

export function wrongPassword(): never {
  return mockFail('wrong_password', 'senha incorreta');
}

export { requireUnlocked };

/** Lê os argumentos como o `parse_args` do núcleo (campos obrigatórios). */
export function str(args: Record<string, unknown>, name: string): string {
  const v = args[name];
  if (typeof v !== 'string') invalid(`argumentos inválidos: missing field \`${name}\``);
  return v;
}

export function optStr(args: Record<string, unknown>, name: string): string | undefined {
  const v = args[name];
  if (v === undefined || v === null) return undefined;
  if (typeof v !== 'string') invalid(`argumentos inválidos: invalid type for \`${name}\``);
  return v;
}

export function num(args: Record<string, unknown>, name: string): number {
  const v = args[name];
  if (typeof v !== 'number' || !Number.isInteger(v)) {
    invalid(`argumentos inválidos: missing field \`${name}\``);
  }
  return v;
}

export function optNum(args: Record<string, unknown>, name: string): number | undefined {
  const v = args[name];
  if (v === undefined || v === null) return undefined;
  if (typeof v !== 'number' || !Number.isInteger(v)) {
    invalid(`argumentos inválidos: invalid type for \`${name}\``);
  }
  return v;
}

export function optBool(args: Record<string, unknown>, name: string): boolean | undefined {
  const v = args[name];
  if (v === undefined || v === null) return undefined;
  if (typeof v !== 'boolean') invalid(`argumentos inválidos: invalid type for \`${name}\``);
  return v;
}

/** `Some("  x ")` → `"x"`; vazio vira `null` (como `clean_opt`). */
export function cleanOpt(v: unknown): string | null {
  if (typeof v !== 'string') return null;
  const t = v.trim();
  return t === '' ? null : t;
}

/** Nome obrigatório (como `clean_name`). */
export function cleanName(v: unknown, what: string): string {
  const t = typeof v === 'string' ? v.trim() : '';
  if (!t) invalid(`informe o nome ${what}`);
  if ([...t].length > 120) invalid(`o nome ${what} pode ter no máximo 120 caracteres`);
  return t;
}

/** Patch anulável: ausente = não mexe (`undefined`), `null` = limpar. */
export function has(args: Record<string, unknown>, name: string): boolean {
  return Object.prototype.hasOwnProperty.call(args, name) && args[name] !== undefined;
}

/** Texto de um argumento qualquer (sem `[object Object]`). */
export function text(v: unknown): string {
  return typeof v === 'string'
    ? v
    : typeof v === 'number' || typeof v === 'boolean'
      ? String(v)
      : '';
}
