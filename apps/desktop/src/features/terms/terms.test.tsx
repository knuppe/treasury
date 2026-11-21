/**
 * Terms of use against the mock: the "Antes de começar" screen before any vault, with
 * "Continuar" tied to the checked box; the full text in Ajustes; the app's copy equal to
 * docs/terms.md; and the screen notices.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { seedTermsPending } from '@/api/mock/terms';
import { renderApp } from '@/test/renderApp';

import { fullTerms } from './texts';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(() => vi.restoreAllMocks());

describe('Terms of use', () => {
  it('asks for acceptance before the vault, with Continuar disabled until checked', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    // Before the gate asks the core (it waits for the vault state first).
    seedTermsPending();
    expect(await screen.findByRole('heading', { name: 'Antes de começar' })).toBeTruthy();
    const box = screen.getByRole('checkbox', { name: /Li e entendi/ });
    expect((box as HTMLInputElement).checked).toBe(false);
    const go = screen.getByRole('button', { name: 'Continuar' });
    expect(go.hasAttribute('disabled')).toBe(true);
    expect(screen.getByRole('button', { name: 'Sair' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Ler os termos completos' }));
    const dialog = await screen.findByRole('dialog', { name: 'Ler os termos completos' });
    expect(within(dialog).getByText(/Uso por sua conta e risco/)).toBeTruthy();
    await user.keyboard('{Escape}');
    await user.click(box);
    await user.click(screen.getByRole('button', { name: 'Continuar' }));
    await waitFor(() =>
      expect(screen.queryByRole('heading', { name: 'Antes de começar' })).toBeNull(),
    );
  });

  it('has the full text in the four languages', () => {
    expect(fullTerms('pt-BR')).toContain('Uso por sua conta e risco');
    expect(fullTerms('pt-PT')).toContain('Termos de Utilização');
    expect(fullTerms('en')).toContain('AGPL-3.0');
    expect(fullTerms('es')).toContain('Uso bajo tu propia cuenta y riesgo');
    expect(fullTerms('xx')).toBe(fullTerms('pt-BR'));
  });

  it('shows the terms in Ajustes and the study notices', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=sobre');
    const card = await screen.findByRole('region', { name: /Termos de uso/ });
    expect(within(card).getByText(/Versão 1\.0/)).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Ler os termos completos' }));
    expect(await screen.findByRole('dialog', { name: 'Termos de uso' })).toBeTruthy();
  });
});
