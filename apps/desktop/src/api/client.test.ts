import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ApiError, call, transportKind } from './client';
import { MOCK_PASSWORD, resetMock } from './mock';
import type { VaultStatus } from './vault';

const invoke = vi.hoisted(() => vi.fn());
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

function setTauri(on: boolean) {
  const w = window as unknown as Record<string, unknown>;
  if (on) w.__TAURI_INTERNALS__ = {};
  else delete w.__TAURI_INTERNALS__;
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

beforeEach(() => {
  setTauri(false);
  invoke.mockReset();
  resetMock('empty');
});

describe('seleção de transporte', () => {
  it('sem Tauri e sem VITE_TREASURY_API usa o mock', async () => {
    expect(transportKind()).toBe('mock');
    const status = await call<VaultStatus>('vault.status');
    expect(status.state).toBe('no_vault');
  });

  it('no Tauri usa invoke', () => {
    setTauri(true);
    expect(transportKind()).toBe('tauri');
  });

  it('fora do Tauri, com VITE_TREASURY_API, usa HTTP', () => {
    vi.stubEnv('VITE_TREASURY_API', 'http://127.0.0.1:7420');
    expect(transportKind()).toBe('http');
  });

  it('VITE_TREASURY_MOCK=1 força o mock', () => {
    setTauri(true);
    vi.stubEnv('VITE_TREASURY_API', 'http://127.0.0.1:7420');
    vi.stubEnv('VITE_TREASURY_MOCK', '1');
    expect(transportKind()).toBe('mock');
  });
});

describe('transporte Tauri', () => {
  beforeEach(() => setTauri(true));

  it("chama invoke('api_call', { cmd, args })", async () => {
    invoke.mockResolvedValue({ state: 'locked' });
    const result = await call('vault.unlock', { password: 'x' });
    expect(invoke).toHaveBeenCalledWith('api_call', {
      cmd: 'vault.unlock',
      args: { password: 'x' },
    });
    expect(result).toEqual({ state: 'locked' });
  });

  it('manda args null quando não há argumentos', async () => {
    invoke.mockResolvedValue(null);
    await call('vault.touch');
    expect(invoke).toHaveBeenCalledWith('api_call', { cmd: 'vault.touch', args: null });
  });

  it('converte o objeto rejeitado em ApiError', async () => {
    invoke.mockRejectedValue({ code: 'wrong_password', message: 'Senha incorreta.' });
    const error = await call('vault.unlock', { password: 'x' }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ code: 'wrong_password', message: 'Senha incorreta.' });
  });

  it('erros de transporte em texto viram ApiError("transport")', async () => {
    invoke.mockRejectedValue('command api_call not found');
    await expect(call('vault.status')).rejects.toMatchObject({
      code: 'transport',
      message: 'command api_call not found',
    });
  });
});

describe('transporte HTTP', () => {
  beforeEach(() => {
    vi.stubEnv('VITE_TREASURY_API', 'http://127.0.0.1:7420/');
    vi.stubEnv('VITE_TREASURY_TOKEN', 'token-de-teste');
  });

  it('faz POST em /api com { cmd, args } e Bearer token', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ ok: true, data: { a: 1 } }));
    vi.stubGlobal('fetch', fetchMock);

    const data = await call('vault.status');

    expect(data).toEqual({ a: 1 });
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('http://127.0.0.1:7420/api');
    expect(init.method).toBe('POST');
    expect(init.headers).toMatchObject({
      'Content-Type': 'application/json',
      Authorization: 'Bearer token-de-teste',
    });
    expect(JSON.parse(init.body as string)).toEqual({ cmd: 'vault.status', args: null });
  });

  it('lança ApiError com o erro do envelope', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          jsonResponse(
            { ok: false, error: { code: 'vault_locked', message: 'O cofre está bloqueado.' } },
            423,
          ),
        ),
    );
    await expect(call('vault.touch')).rejects.toMatchObject({
      name: 'ApiError',
      code: 'vault_locked',
      message: 'O cofre está bloqueado.',
    });
  });

  it('falha de rede vira ApiError("network")', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));
    await expect(call('vault.status')).rejects.toMatchObject({ code: 'network' });
  });

  it('resposta que não é JSON vira ApiError("bad_response")', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('<html>', { status: 502 })));
    await expect(call('vault.status')).rejects.toMatchObject({ code: 'bad_response' });
  });
});

describe('mock do cofre', () => {
  it('cria, bloqueia e desbloqueia', async () => {
    const created = await call<VaultStatus>('vault.create', {
      password: 'uma senha longa o bastante',
      household_name: 'Família Teste',
      members: ['Ana', ' '],
    });
    expect(created).toMatchObject({ state: 'unlocked', household_name: 'Família Teste' });
    expect((await call<VaultStatus>('vault.lock')).state).toBe('locked');
    await expect(call('vault.touch')).rejects.toMatchObject({ code: 'vault_locked' });
    const unlocked = await call<VaultStatus>('vault.unlock', {
      password: 'uma senha longa o bastante',
    });
    expect(unlocked.state).toBe('unlocked');
    await expect(call('vault.touch')).resolves.toBeNull();
  });

  it('senha errada rejeita com wrong_password', async () => {
    resetMock('locked');
    await expect(call('vault.unlock', { password: 'errada' })).rejects.toMatchObject({
      code: 'wrong_password',
    });
    await expect(call('vault.unlock', { password: MOCK_PASSWORD })).resolves.toMatchObject({
      state: 'unlocked',
    });
  });

  it('comando desconhecido rejeita com unknown_command', async () => {
    await expect(call('nada.disso')).rejects.toMatchObject({ code: 'unknown_command' });
  });
});
