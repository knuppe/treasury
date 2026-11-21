/**
 * First screen (docs/i18n-design.md): on a fresh install, the language comes before the terms, already
 * selected with the system's guess; selecting another shows the screen in it; "Continuar" saves and the
 * terms appear in the chosen language. In Settings, the switch applies immediately.
 */
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { call } from '@/api/client';
import type { LocaleState } from '@/api/generated/LocaleState';
import { seedLocaleUnchosen } from '@/api/mock/locale';
import { seedTermsPending } from '@/api/mock/terms';
import { renderApp } from '@/test/renderApp';

import { applyLocale, useI18n } from './index';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  vi.restoreAllMocks();
  await applyLocale('pt-BR');
  useI18n.setState({ chosen: true });
});

describe('Language', () => {
  it("asks for the language before the terms, with the system's guess", async () => {
    const user = userEvent.setup();
    vi.spyOn(navigator, 'language', 'get').mockReturnValue('en-US');
    renderApp('empty');
    seedLocaleUnchosen();
    seedTermsPending();
    expect(await screen.findByRole('heading', { name: 'Choose your language' })).toBeTruthy();
    const picker = screen.getByRole('combobox', { name: 'Language' });
    expect(picker.textContent).toBe('English');
    // Each language in its own tongue, with the flag of the variant's country.
    expect(picker.querySelector('img')?.getAttribute('src')).toMatch(/svg/);
    await user.click(picker);
    expect(await screen.findByRole('option', { name: 'Português (Brasil)' })).toBeTruthy();
    expect(screen.getByRole('option', { name: 'Português (Portugal)' })).toBeTruthy();
    expect(screen.queryByRole('option', { name: /Pseudo/ })).toBeNull();
    await user.click(screen.getByRole('option', { name: 'Español' }));
    expect(await screen.findByRole('heading', { name: 'Elige el idioma' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Continuar' }));
    expect(await screen.findByRole('heading', { name: 'Antes de empezar' })).toBeTruthy();
    const saved = await call<LocaleState>('system.locale');
    expect(saved).toMatchObject({ locale: 'es', chosen: true });
    expect(document.documentElement.lang).toBe('es');
  });

  it('switches in Settings immediately, without losing the screen', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    await user.click(await screen.findByRole('combobox', { name: 'Idioma' }));
    await user.click(await screen.findByRole('option', { name: 'English' }));
    await waitFor(() => expect(useI18n.getState().locale).toBe('en'));
    const picker = await screen.findByRole('combobox', { name: 'Language' });
    expect(picker.textContent).toBe('English');
    expect(screen.getByRole('link', { name: /Settings/ })).toBeTruthy();
    const saved = await call<LocaleState>('system.locale');
    expect(saved).toMatchObject({ locale: 'en', chosen: true });
  });
});
