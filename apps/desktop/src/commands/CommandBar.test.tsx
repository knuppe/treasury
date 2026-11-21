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

describe('barra de comando', () => {
  it('abre com Ctrl+K e navega para uma tela digitando sem acento', async () => {
    const { user, dialog } = await openBar();
    const input = within(dialog).getByRole('combobox');
    expect(document.activeElement).toBe(input);

    await user.type(input, 'orcamento');
    await user.keyboard('{Enter}');

    expect(await screen.findByRole('heading', { level: 1, name: /Orçamento/ })).toBeTruthy();
    expect(window.location.hash).toBe('#/orcamento');
    expect(screen.queryByRole('dialog', { name: 'Barra de comando' })).toBeNull();
  });

  it('abre pelo gatilho da barra superior e navega com as setas', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });
    await user.click(screen.getByRole('button', { name: /Buscar ou executar/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });

    await user.type(within(dialog).getByRole('combobox'), 'carteira');
    // A tela vem antes das abas dela ("Carteira › Bens"…).
    expect(within(dialog).getAllByRole('option')[0]!.textContent).toMatch(/Carteira/);
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('heading', { level: 1, name: /Carteira/ })).toBeTruthy();
  });

  it('alterna o modo privacidade e troca o tema', async () => {
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

  it('bloqueia o cofre', async () => {
    const { user, dialog } = await openBar();
    await user.type(within(dialog).getByRole('combobox'), 'bloquear');
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('button', { name: 'Desbloquear' })).toBeTruthy();
  });

  it('"Perguntar ao agente" abre a conversa com a pergunta digitada', async () => {
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

  it('a busca ignora letras soltas e põe o melhor grupo primeiro', async () => {
    const { user, dialog } = await openBar();
    await user.type(within(dialog).getByRole('combobox'), 'tema');
    const options = within(dialog).getAllByRole('option');
    expect(options[0]!.textContent).toMatch(/^Tema: Escuro/);
    // Telas sem relação com "tema" não aparecem por casamento difuso.
    expect(within(dialog).queryByRole('option', { name: /Dívidas/ })).toBeNull();
    // "Ajustes" aparece por ter a palavra-chave "tema" (a tela e a seção Aparência e uso).
    expect(within(dialog).getAllByRole('option', { name: /Ajustes/ }).length).toBeGreaterThan(0);
  });

  it('Escape fecha a barra', async () => {
    const { user } = await openBar();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog', { name: 'Barra de comando' })).toBeNull();
  });

  it('telas registram comandos próprios enquanto montadas', async () => {
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

describe('registro de comandos', () => {
  it('ordena grupos conhecidos primeiro e os demais em ordem alfabética', () => {
    const noop = () => {};
    const groups = groupCommands([
      { id: 'z', title: 'Z', group: 'Relatórios', perform: noop },
      { id: 'a', title: 'A', group: 'Agente', perform: noop },
      { id: 'n', title: 'N', group: 'Navegação', perform: noop },
      { id: 'c', title: 'C', group: 'Carteira', perform: noop },
    ]);
    expect(groups.map((g) => g.group)).toEqual(['Navegação', 'Agente', 'Carteira', 'Relatórios']);
  });

  it('normaliza buscas sem acentos', () => {
    expect(normalizeSearch('Lançamentos ORÇAMENTO')).toBe('lancamentos orcamento');
  });

  it('useRegisterCommands remove os comandos ao desmontar', async () => {
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
