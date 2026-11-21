import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useMemo } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { renderApp } from '@/test/renderApp';
import { useUi } from '@/state/ui';

import {
  groupCommands,
  normalizeSearch,
  registerCommands,
  useRegisterCommands,
  type AppCommand,
} from './registry';

async function openBar() {
  const user = userEvent.setup();
  renderApp('unlocked');
  await screen.findByRole('navigation', { name: 'Telas' });
  await user.keyboard('{Control>}k{/Control}');
  const dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });
  return { user, dialog };
}

describe('command bar', () => {
  it('opens with Ctrl+K and navigates to a screen typed without accents', async () => {
    const { user, dialog } = await openBar();
    const input = within(dialog).getByRole('combobox');
    expect(document.activeElement).toBe(input);

    await user.type(input, 'orcamento');
    await user.keyboard('{Enter}');

    expect(await screen.findByRole('heading', { level: 1, name: /Orçamento/ })).toBeTruthy();
    expect(window.location.hash).toBe('#/orcamento');
    expect(screen.queryByRole('dialog', { name: 'Barra de comando' })).toBeNull();
  });

  it('opens from the top bar trigger and navigates with the arrows', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });
    await user.click(screen.getByRole('button', { name: /Buscar ou executar/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });

    await user.type(within(dialog).getByRole('combobox'), 'carteira');
    // The screen comes before its tabs ("Carteira › Bens"…).
    expect(within(dialog).getAllByRole('option')[0]!.textContent).toMatch(/Carteira/);
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('heading', { level: 1, name: /Carteira/ })).toBeTruthy();
  });

  it('toggles privacy mode and switches the theme', async () => {
    const { user, dialog } = await openBar();
    await user.type(within(dialog).getByRole('combobox'), 'privacidade');
    await user.keyboard('{Enter}');
    expect(useUi.getState().privacy).toBe(true);

    await user.keyboard('{Control>}k{/Control}');
    const again = await screen.findByRole('dialog', { name: 'Barra de comando' });
    await user.type(within(again).getByRole('combobox'), 'lo-fi');
    await user.keyboard('{Enter}');
    expect(useUi.getState().theme).toBe('lofi');
    expect(document.documentElement.dataset.theme).toBe('lofi');
  });

  it('locks the vault', async () => {
    const { user, dialog } = await openBar();
    await user.type(within(dialog).getByRole('combobox'), 'bloquear');
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('button', { name: 'Desbloquear' })).toBeTruthy();
  });

  it('"Perguntar ao agente" opens the conversation with the typed question', async () => {
    const { user, dialog } = await openBar();
    await user.type(within(dialog).getByRole('combobox'), 'quanto gastei com mercado');
    const agent = within(dialog).getByRole('option', { name: /Perguntar ao agente/ });
    expect(agent.getAttribute('aria-disabled')).not.toBe('true');
    expect(agent.textContent).toContain('“quanto gastei com mercado”');
    await user.click(agent);
    const box = await screen.findByRole('textbox', { name: /mensagem|pergunta/i });
    expect((box as HTMLTextAreaElement).value).toBe('quanto gastei com mercado');
    expect(window.location.hash).not.toContain('pergunta=');
  });

  it('search ignores stray letters and puts the best group first', async () => {
    const { user, dialog } = await openBar();
    await user.type(within(dialog).getByRole('combobox'), 'tema');
    const options = within(dialog).getAllByRole('option');
    expect(options[0]!.textContent).toMatch(/^Tema: Escuro/);
    // Screens unrelated to "tema" do not show up by fuzzy match.
    expect(within(dialog).queryByRole('option', { name: /Dívidas/ })).toBeNull();
    // "Ajustes" shows up for having the keyword "tema" (the screen and the Appearance section).
    expect(within(dialog).getAllByRole('option', { name: /Ajustes/ }).length).toBeGreaterThan(0);
  });

  it('Escape closes the bar', async () => {
    const { user } = await openBar();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog', { name: 'Barra de comando' })).toBeNull();
  });

  it('screens register their own commands while mounted', async () => {
    const perform = vi.fn();
    const { user, dialog } = await openBar();
    const unregister = registerCommands([
      { id: 'teste.exportar', title: 'Exportar CSV de teste', group: 'Relatórios', perform },
    ]);
    await user.type(within(dialog).getByRole('combobox'), 'exportar csv de teste');
    await user.keyboard('{Enter}');
    expect(perform).toHaveBeenCalledOnce();
    unregister();
  });
});

describe('command registry', () => {
  it('sorts known groups first and the rest alphabetically', () => {
    const noop = () => {};
    const groups = groupCommands([
      { id: 'z', title: 'Z', group: 'Relatórios', perform: noop },
      { id: 'a', title: 'A', group: 'Agente', perform: noop },
      { id: 'n', title: 'N', group: 'Navegação', perform: noop },
      { id: 'c', title: 'C', group: 'Carteira', perform: noop },
    ]);
    expect(groups.map((g) => g.group)).toEqual(['Navegação', 'Agente', 'Carteira', 'Relatórios']);
  });

  it('normalizes searches without accents', () => {
    expect(normalizeSearch('Lançamentos ORÇAMENTO')).toBe('lancamentos orcamento');
  });

  it('useRegisterCommands removes the commands on unmount', async () => {
    const { renderHook } = await import('@testing-library/react');
    const { useCommandRegistry } = await import('./registry');
    const { unmount } = renderHook(() => {
      const commands = useMemo<AppCommand[]>(
        () => [{ id: 'temp', title: 'Temporário', group: 'Teste', perform: () => {} }],
        [],
      );
      useRegisterCommands(commands);
    });
    expect(useCommandRegistry.getState().commands.has('temp')).toBe(true);
    unmount();
    expect(useCommandRegistry.getState().commands.has('temp')).toBe(false);
  });
});
