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

describe('copying secrets', () => {
  it('in the app, uses the system concealed copy and not the browser one', async () => {
    const write = mockClipboard();
    const invoke = vi.fn().mockResolvedValue(null);
    mockTauri(invoke);
    await copySecret('parte-inventada');
    expect(invoke).toHaveBeenCalledWith('secret_copy', { text: 'parte-inventada' });
    expect(write).not.toHaveBeenCalled();
  });

  it('without the concealed copy (Windows, Linux), falls back to the normal copy', async () => {
    const write = mockClipboard();
    mockTauri(vi.fn().mockRejectedValue('unsupported'));
    await copySecret('parte-inventada');
    expect(write).toHaveBeenCalledWith('parte-inventada');
  });

  it('without a clipboard, it fails so the screen warns', async () => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true });
    await expect(copyText('x')).rejects.toThrow();
  });
});

describe('screenshots', () => {
  it('protects only with privacy mode, unless the setting allows it', () => {
    updateCapture({ privacy: false, allowed: false });
    expect(captureProtected()).toBe(false);
    updateCapture({ privacy: true });
    expect(captureProtected()).toBe(true);
    updateCapture({ allowed: true });
    expect(captureProtected()).toBe(false);
    // Locked vault: the setting stops applying and privacy protects again.
    updateCapture({ allowed: false });
    expect(captureProtected()).toBe(true);
    updateCapture({ privacy: false });
    expect(captureProtected()).toBe(false);
  });
});
