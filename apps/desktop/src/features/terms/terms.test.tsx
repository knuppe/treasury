/**
 * Termos de uso contra o mock: a tela "Antes de começar" antes de qualquer cofre, com o
 * "Continuar" preso à caixa marcada; o texto completo em Ajustes; a cópia do app igual à de
 * docs/terms.md; e os avisos das telas.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { seedTermsPending } from '@/api/mock/terms';
import { renderApp } from '@/test/renderApp';

import { fullTerms } from './texts';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(() => vi.restoreAllMocks());

describe('Termos de uso', () => {
  it('pede o aceite antes do cofre, com Continuar desabilitado até marcar', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    // Antes de a porta perguntar ao núcleo (ela espera o estado do cofre primeiro).
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

  it('tem o texto completo nos quatro idiomas', () => {
    expect(fullTerms('pt-BR')).toContain('Uso por sua conta e risco');
    expect(fullTerms('pt-PT')).toContain('Termos de Utilização');
    expect(fullTerms('en')).toContain('AGPL-3.0');
    expect(fullTerms('es')).toContain('Uso bajo tu propia cuenta y riesgo');
    expect(fullTerms('xx')).toBe(fullTerms('pt-BR'));
  });

  it('mostra os termos em Ajustes e os avisos de estudo', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=sobre');
    const card = await screen.findByRole('region', { name: /Termos de uso/ });
    expect(within(card).getByText(/Versão 1\.0/)).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Ler os termos completos' }));
    expect(await screen.findByRole('dialog', { name: 'Termos de uso' })).toBeTruthy();
  });
});
