import { afterEach, describe, expect, it, vi } from 'vitest';

import { captureProtected, copySecret, copyText, updateCapture } from './osGuard';

const w = window as unknown as Record<string, unknown>;

function mockClipboard() {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
  return writeText;
}

function mockTauri(invoke: (cmd: string, args: unknown) => Promise<unknown>) {
  w.__TAURI_INTERNALS__ = {};
  vi.doMock('@tauri-apps/api/core', () => ({ invoke }));
}

afterEach(() => {
  delete w.__TAURI_INTERNALS__;
  Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true });
  vi.doUnmock('@tauri-apps/api/core');
  vi.restoreAllMocks();
});

describe('cópia de segredos', () => {
  it('no app, usa a cópia oculta do sistema e não a do navegador', async () => {
    const write = mockClipboard();
    const invoke = vi.fn().mockResolvedValue(null);
    mockTauri(invoke);
    await copySecret('parte-inventada');
    expect(invoke).toHaveBeenCalledWith('secret_copy', { text: 'parte-inventada' });
    expect(write).not.toHaveBeenCalled();
  });

  it('sem a cópia oculta (Windows, Linux), cai na cópia normal', async () => {
    const write = mockClipboard();
    mockTauri(vi.fn().mockRejectedValue('unsupported'));
    await copySecret('parte-inventada');
    expect(write).toHaveBeenCalledWith('parte-inventada');
  });

  it('sem área de transferência, falha para a tela avisar', async () => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true });
    await expect(copyText('x')).rejects.toThrow();
  });
});

describe('capturas de tela', () => {
  it('protege só com o modo privacidade, salvo se o ajuste liberar', () => {
    updateCapture({ privacy: false, allowed: false });
    expect(captureProtected()).toBe(false);
    updateCapture({ privacy: true });
    expect(captureProtected()).toBe(true);
    updateCapture({ allowed: true });
    expect(captureProtected()).toBe(false);
    // Cofre bloqueado: o ajuste deixa de valer e a privacidade volta a proteger.
    updateCapture({ allowed: false });
    expect(captureProtected()).toBe(true);
    updateCapture({ privacy: false });
    expect(captureProtected()).toBe(false);
  });
});
