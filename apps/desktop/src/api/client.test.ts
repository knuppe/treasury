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

describe('transport selection', () => {
  it('without Tauri and without VITE_TREASURY_API it uses the mock', async () => {
    expect(transportKind()).toBe('mock');
    const status = await call<VaultStatus>('vault.status');
    expect(status.state).toBe('no_vault');
  });

  it('in Tauri it uses invoke', () => {
    setTauri(true);
    expect(transportKind()).toBe('tauri');
  });

  it('outside Tauri, with VITE_TREASURY_API, it uses HTTP', () => {
    vi.stubEnv('VITE_TREASURY_API', 'http://127.0.0.1:7420');
    expect(transportKind()).toBe('http');
  });

  it('VITE_TREASURY_MOCK=1 forces the mock', () => {
    setTauri(true);
    vi.stubEnv('VITE_TREASURY_API', 'http://127.0.0.1:7420');
    vi.stubEnv('VITE_TREASURY_MOCK', '1');
    expect(transportKind()).toBe('mock');
  });
});

describe('Tauri transport', () => {
  beforeEach(() => setTauri(true));

  it("calls invoke('api_call', { cmd, args })", async () => {
    invoke.mockResolvedValue({ state: 'locked' });
    const result = await call('vault.unlock', { password: 'x' });
    expect(invoke).toHaveBeenCalledWith('api_call', {
      cmd: 'vault.unlock',
      args: { password: 'x' },
    });
    expect(result).toEqual({ state: 'locked' });
  });

  it('sends null args when there are no arguments', async () => {
    invoke.mockResolvedValue(null);
    await call('vault.touch');
    expect(invoke).toHaveBeenCalledWith('api_call', { cmd: 'vault.touch', args: null });
  });

  it('converts the rejected object into ApiError', async () => {
    invoke.mockRejectedValue({ code: 'wrong_password', message: 'Senha incorreta.' });
    const error = await call('vault.unlock', { password: 'x' }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ code: 'wrong_password', message: 'Senha incorreta.' });
  });

  it('transport errors as text become ApiError("transport")', async () => {
    invoke.mockRejectedValue('command api_call not found');
    await expect(call('vault.status')).rejects.toMatchObject({
      code: 'transport',
      message: 'command api_call not found',
    });
  });
});

describe('HTTP transport', () => {
  beforeEach(() => {
    vi.stubEnv('VITE_TREASURY_API', 'http://127.0.0.1:7420/');
    vi.stubEnv('VITE_TREASURY_TOKEN', 'token-de-teste');
  });

  it('POSTs to /api with { cmd, args } and a Bearer token', async () => {
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

  it('throws ApiError with the envelope error', async () => {
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

  it('network failure becomes ApiError("network")', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));
    await expect(call('vault.status')).rejects.toMatchObject({ code: 'network' });
  });

  it('a non-JSON response becomes ApiError("bad_response")', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('<html>', { status: 502 })));
    await expect(call('vault.status')).rejects.toMatchObject({ code: 'bad_response' });
  });
});

describe('vault mock', () => {
  it('creates, locks and unlocks', async () => {
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

  it('wrong password rejects with wrong_password', async () => {
    resetMock('locked');
    await expect(call('vault.unlock', { password: 'errada' })).rejects.toMatchObject({
      code: 'wrong_password',
    });
    await expect(call('vault.unlock', { password: MOCK_PASSWORD })).resolves.toMatchObject({
      state: 'unlocked',
    });
  });

  it('unknown command rejects with unknown_command', async () => {
    await expect(call('nada.disso')).rejects.toMatchObject({ code: 'unknown_command' });
  });
});
