/**
 * Export Markdown in the app (WKWebView): a Blob link downloads nothing there, so the button asks
 * where to save (system dialog) and the core writes the file.
 */
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

const invoke = vi.hoisted(() => vi.fn());
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  invoke.mockReset();
});

describe('Developer › Export Markdown', () => {
  it('in the app, saves through the system dialog and the core', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/desenvolvedor');
    await act(async () => {
      await mockCall('settings.set', { developer_tools: true });
      await queryClient.invalidateQueries();
    });
    await screen.findByRole('group', { name: 'Quadro de feedback' });
    // Only now it "becomes" the app: the dialog and the core call go through invoke.
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    const calls: { cmd: string; args: Record<string, unknown> }[] = [];
    invoke.mockImplementation(async (cmd: string, a: Record<string, unknown>) => {
      if (cmd === 'plugin:dialog|save') return '/Users/teste/Documents/feedback.md';
      if (cmd === 'api_call') {
        const c = a as { cmd: string; args: Record<string, unknown> };
        calls.push(c);
        return mockCall(c.cmd, c.args);
      }
      return null;
    });
    const createUrl = vi.spyOn(URL, 'createObjectURL');
    await user.click(screen.getByRole('button', { name: 'Exportar Markdown' }));
    await waitFor(() => expect(calls.some((c) => c.cmd === 'devfeedback.export')).toBe(true));
    const exp = calls.find((c) => c.cmd === 'devfeedback.export')!;
    expect(exp.args.path).toBe('/Users/teste/Documents/feedback.md');
    expect(String(exp.args.text)).toContain('Falta ferramenta para mover uma compra');
    expect(createUrl).not.toHaveBeenCalled();
    const toast = await screen.findByText(/feedbacks exportados/);
    expect(within(toast.closest('li') ?? document.body).getByText(/feedback\.md/)).toBeTruthy();
  });
});
