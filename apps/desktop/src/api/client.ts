/**
 * Cliente da API do núcleo Rust.
 *
 * Uma única função, `call(cmd, args)`, com três transportes:
 *
 * 1. **Tauri** (o app de verdade): `invoke('api_call', { cmd, args })`. Em caso
 *    de erro, o núcleo rejeita com o próprio objeto `{ code, message, details? }`.
 * 2. **HTTP** (desenvolvimento no navegador contra um núcleo rodando à parte):
 *    `POST ${VITE_TREASURY_API}/api` com `{ cmd, args }` e
 *    `Authorization: Bearer ${VITE_TREASURY_TOKEN}`; a resposta é
 *    `{ ok: true, data }` ou `{ ok: false, error: { code, message, details? } }`.
 * 3. **Mock em memória** (`mock.ts`), quando não há Tauri nem `VITE_TREASURY_API`,
 *    ou com `VITE_TREASURY_MOCK=1`. Serve para desenvolvimento e testes.
 *
 * As mensagens de erro já vêm do núcleo no idioma da instalação.
 *
 * Os comandos demorados que a interface dispara (importação, cupons, e-mail, cenários,
 * cotações) entram na atividade do app (state/activity.ts) enquanto rodam.
 */
import { t } from '@/i18n';
import { startActivity } from '@/state/activity';

export interface ApiErrorPayload {
  code: string;
  message: string;
  details?: unknown;
}

/** Erro devolvido pelo núcleo (ou pelo transporte), com `code` estável. */
export class ApiError extends Error {
  readonly code: string;
  readonly details?: unknown;

  constructor(code: string, message: string, details?: unknown) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.details = details;
  }
}

/** Códigos de erro produzidos pelo próprio cliente (não pelo núcleo). */
export const CLIENT_ERROR = {
  network: 'network',
  badResponse: 'bad_response',
  transport: 'transport',
} as const;

export function isApiError(value: unknown): value is ApiError {
  return value instanceof ApiError;
}

export function isErrorPayload(value: unknown): value is ApiErrorPayload {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as ApiErrorPayload).code === 'string' &&
    typeof (value as ApiErrorPayload).message === 'string'
  );
}

export type TransportKind = 'tauri' | 'http' | 'mock';

function env(name: 'VITE_TREASURY_API' | 'VITE_TREASURY_TOKEN' | 'VITE_TREASURY_MOCK') {
  const value = import.meta.env[name];
  return value === '' ? undefined : value;
}

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** Qual transporte `call` vai usar agora. */
export function transportKind(): TransportKind {
  if (env('VITE_TREASURY_MOCK') === '1') return 'mock';
  if (isTauri()) return 'tauri';
  if (env('VITE_TREASURY_API')) return 'http';
  return 'mock';
}

function toApiError(reason: unknown): ApiError {
  if (reason instanceof ApiError) return reason;
  if (isErrorPayload(reason)) return new ApiError(reason.code, reason.message, reason.details);
  if (typeof reason === 'string') return new ApiError(CLIENT_ERROR.transport, reason);
  if (reason instanceof Error) return new ApiError(CLIENT_ERROR.transport, reason.message);
  return new ApiError(CLIENT_ERROR.transport, t('componentes-nucleo-falha'));
}

async function callTauri<T>(cmd: string, args: unknown): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  try {
    return await invoke<T>('api_call', { cmd, args: args ?? null });
  } catch (reason) {
    throw toApiError(reason);
  }
}

async function callHttp<T>(cmd: string, args: unknown): Promise<T> {
  const base = (env('VITE_TREASURY_API') ?? '').replace(/\/+$/, '');
  const token = env('VITE_TREASURY_TOKEN');
  let response: Response;
  try {
    response = await fetch(`${base}/api`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify({ cmd, args: args ?? null }),
      credentials: 'omit',
      cache: 'no-store',
    });
  } catch {
    throw new ApiError(CLIENT_ERROR.network, t('componentes-nucleo-sem-conexao'));
  }

  let body: unknown;
  try {
    body = await response.json();
  } catch {
    throw new ApiError(
      CLIENT_ERROR.badResponse,
      t('componentes-nucleo-resposta-invalida', { status: String(response.status) }),
    );
  }

  if (typeof body === 'object' && body !== null && 'ok' in body) {
    const envelope = body as { ok: boolean; data?: unknown; error?: unknown };
    if (envelope.ok) return envelope.data as T;
    if (isErrorPayload(envelope.error)) throw toApiError(envelope.error);
  }
  throw new ApiError(
    CLIENT_ERROR.badResponse,
    t('componentes-nucleo-resposta-inesperada', { status: String(response.status) }),
  );
}

async function callMock<T>(cmd: string, args: unknown): Promise<T> {
  const { mockCall } = await import('./mock');
  try {
    return (await mockCall(cmd, args)) as T;
  } catch (reason) {
    throw toApiError(reason);
  }
}

/** Comandos que contam como atividade do app, com o rótulo que o chip e a lista mostram. */
const importacao = () => t('casca-atividade-importacao');
const cupom = () => t('casca-atividade-cupom');
const email = () => t('casca-atividade-email');
const cenario = () => t('casca-atividade-cenario');

export const ACTIVITY_COMMANDS: Readonly<Record<string, () => string>> = {
  'import.detect': importacao,
  'import.preview': importacao,
  'import.analyze_image': importacao,
  'import.commit': importacao,
  'receipts.preview': cupom,
  'receipts.read_photo': cupom,
  'receipts.import': cupom,
  'mail.sync_now': email,
  'mail.apply_part': email,
  'scenarios.simulate': cenario,
  'pension.simulate': cenario,
  'market.sync': () => t('casca-atividade-cotacoes'),
};

/** Um comando rápido não chega a acender nada (sem piscar a cada tecla num cenário). */
export const ACTIVITY_GRACE_MS = 250;

let activitySeq = 0;

/**
 * Chama um comando do núcleo. Rejeita sempre com `ApiError`.
 *
 * ```ts
 * const status = await call<VaultStatus>('vault.status');
 * ```
 */
export function call<T>(cmd: string, args?: unknown): Promise<T> {
  const label = ACTIVITY_COMMANDS[cmd];
  if (!label) return send<T>(cmd, args);
  const id = `cmd:${cmd}:${++activitySeq}`;
  let stop: ReturnType<typeof startActivity> | null = null;
  const timer = setTimeout(() => {
    stop = startActivity(id, label());
  }, ACTIVITY_GRACE_MS);
  const finish = (status: 'done' | 'failed') => {
    clearTimeout(timer);
    stop?.(status);
  };
  return send<T>(cmd, args).then(
    (value) => {
      finish('done');
      return value;
    },
    (reason: unknown) => {
      finish('failed');
      throw reason;
    },
  );
}

function send<T>(cmd: string, args?: unknown): Promise<T> {
  switch (transportKind()) {
    case 'tauri':
      return callTauri<T>(cmd, args);
    case 'http':
      return callHttp<T>(cmd, args);
    case 'mock':
      return callMock<T>(cmd, args);
  }
}
