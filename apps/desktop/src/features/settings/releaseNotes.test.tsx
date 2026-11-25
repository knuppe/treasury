/**
 * The release notes' translate button: hidden in English, on the device when the system can,
 * the browser translator otherwise or after a failure, and only on the person's click.
 */
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ReleaseNotes } from '@/features/settings/ReleaseNotes';
import { applyLocale } from '@/i18n';
import type * as TranslateNotes from '@/lib/translateNotes';

const native = vi.hoisted(() => ({
  can: vi.fn<(t: string) => Promise<boolean>>(),
  translate: vi.fn<(text: string, t: string) => Promise<string>>(),
}));
vi.mock('@/lib/translateNotes', async (orig) => ({
  ...(await orig<typeof TranslateNotes>()),
  canTranslateOnDevice: native.can,
  translateOnDevice: native.translate,
}));

const NOTES = 'Fixes card statements.';

beforeEach(async () => {
  await applyLocale('pt-BR');
});
afterEach(() => {
  vi.restoreAllMocks();
  native.can.mockReset();
  native.translate.mockReset();
});

describe('ReleaseNotes', () => {
  it('shows no button when the interface is in English', async () => {
    await applyLocale('en');
    native.can.mockResolvedValue(true);
    render(<ReleaseNotes notes={NOTES} />);
    expect(screen.getByText(NOTES)).toBeTruthy();
    await Promise.resolve();
    expect(screen.queryByRole('button')).toBeNull();
    expect(native.can).not.toHaveBeenCalled();
  });

  it('translates on the device and shows the original again', async () => {
    const user = userEvent.setup();
    native.can.mockResolvedValue(true);
    native.translate.mockResolvedValue('Corrige as faturas do cartão.');
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    render(<ReleaseNotes notes={NOTES} />);
    await user.click(await screen.findByRole('button', { name: 'Traduzir' }));
    expect(await screen.findByText('Corrige as faturas do cartão.')).toBeTruthy();
    expect(native.translate).toHaveBeenCalledWith(NOTES, 'pt-BR');
    expect(screen.getByText('Traduzido neste computador')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Ver original' }));
    expect(screen.getByText(NOTES)).toBeTruthy();
    expect(open).not.toHaveBeenCalled();
  });

  it('falls back to the browser translator, opened only by a click', async () => {
    const user = userEvent.setup();
    native.can.mockResolvedValue(true);
    native.translate.mockRejectedValue(new Error('not now'));
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    render(<ReleaseNotes notes={NOTES} />);
    await user.click(await screen.findByRole('button', { name: 'Traduzir' }));
    const google = await screen.findByRole('button', { name: 'Traduzir no Google Tradutor' });
    expect(screen.getByText('O macOS não conseguiu traduzir agora.')).toBeTruthy();
    expect(open).not.toHaveBeenCalled();
    await user.click(google);
    await waitFor(() => expect(open).toHaveBeenCalledTimes(1));
    const url = new URL(String(open.mock.calls[0]?.[0]));
    expect(url.searchParams.get('tl')).toBe('pt');
    expect(url.searchParams.get('text')).toBe(NOTES);
  });

  it('goes straight to the browser translator where the device cannot translate', async () => {
    native.can.mockResolvedValue(false);
    render(<ReleaseNotes notes={NOTES} />);
    expect(await screen.findByRole('button', { name: 'Traduzir no Google Tradutor' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Traduzir' })).toBeNull();
    expect(native.translate).not.toHaveBeenCalled();
  });
});
