/**
 * Primeira tela (docs/i18n-design.md): numa instalação nova, o idioma vem antes dos termos, já
 * marcado com o palpite do sistema; marcar outro mostra a tela nele; "Continuar" grava e os
 * termos aparecem no idioma escolhido. Em Ajustes, a troca vale na hora.
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

describe('Idioma', () => {
  it('pergunta o idioma antes dos termos, com o palpite do sistema', async () => {
    const user = userEvent.setup();
    vi.spyOn(navigator, 'language', 'get').mockReturnValue('en-US');
    renderApp('empty');
    seedLocaleUnchosen();
    seedTermsPending();
    expect(await screen.findByRole('heading', { name: 'Choose your language' })).toBeTruthy();
    const picker = screen.getByRole('combobox', { name: 'Language' });
    expect(picker.textContent).toBe('English');
    // Cada idioma na própria língua, com a bandeira do país da variante.
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

  it('troca em Ajustes na hora, sem perder a tela', async () => {
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
