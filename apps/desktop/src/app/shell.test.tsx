import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { setMockNewerAppVersion } from '@/api/mock/vaultExtras';
import { THEMES } from '@/design/themes';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

// Contas mostra a previsão de saldo (gráfico): jsdom não tem canvas.
vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('shell', () => {
  it('avisa na barra superior quando um app mais novo já abriu o cofre', async () => {
    const { queryClient } = renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });
    expect(screen.queryByText('Atualize o app')).toBeNull();
    setMockNewerAppVersion('9.9.9');
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    expect(await screen.findByText('Atualize o app')).toBeTruthy();
  });

  it('recolhe o menu lateral (botão e ⌘\\) e mantém as telas navegáveis', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    const nav = await screen.findByRole('navigation', { name: 'Telas' });
    expect(within(nav).getByText('Contas')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Recolher menu lateral' }));
    expect(useUi.getState().sidebarCollapsed).toBe(true);
    const collapsed = screen.getByRole('navigation', { name: 'Telas' });
    // Só os números aparecem; o nome fica no rótulo acessível (e na dica).
    expect(within(collapsed).queryByText('Contas')).toBeNull();
    await user.click(within(collapsed).getByRole('link', { name: 'Contas' }));
    expect(await screen.findByRole('heading', { level: 1, name: /Contas/ })).toBeTruthy();

    await user.keyboard('{Control>}\\{/Control}');
    expect(useUi.getState().sidebarCollapsed).toBe(false);
    expect(screen.getByRole('button', { name: 'Recolher menu lateral' })).toBeTruthy();
  });

  it('⌘0 abre o Mercado no Brasil e não abre onde o país não tem a tela', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/contas');
    await screen.findByRole('heading', { level: 1, name: /Contas/ });
    await mockCall('country.set', { country: 'US' });
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    const nav = screen.getByRole('navigation', { name: 'Telas' });
    await waitFor(() => expect(within(nav).queryByRole('link', { name: /Mercado/ })).toBeNull());
    await user.keyboard('{Control>}0{/Control}');
    expect(screen.getByRole('heading', { level: 1, name: /Contas/ })).toBeTruthy();
    // Tela do menu continua abrindo pelo número.
    await user.keyboard('{Control>}4{/Control}');
    expect(await screen.findByRole('heading', { level: 1, name: /Lançamentos/ })).toBeTruthy();

    await mockCall('country.set', { country: 'BR' });
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    await user.keyboard('{Control>}0{/Control}');
    expect(await screen.findByRole('heading', { level: 1, name: /Mercado/ })).toBeTruthy();
  }, 60_000);

  it('o seletor de tema da barra superior lista todos os temas e aplica o escolhido', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });

    await user.click(screen.getByRole('button', { name: /Escolher tema \(atual: Escuro\)/ }));
    const menu = screen.getByRole('menu', { name: 'Temas' });
    const items = within(menu).getAllByRole('menuitemradio');
    expect(items).toHaveLength(THEMES.length);
    // A descrição fica cortada na linha; a dica mostra o texto inteiro.
    const kitt = THEMES.find((t) => t.id === 'kitt')!;
    await user.hover(within(menu).getByRole('menuitemradio', { name: /Scanner/ }));
    // Pode haver outra dica aberta (o item focado ao abrir o menu): procura a do item.
    await waitFor(() =>
      expect(
        screen.getAllByRole('tooltip').some((t) => t.textContent?.includes(kitt.description)),
      ).toBe(true),
    );
    expect(
      within(menu)
        .getByRole('menuitemradio', { name: /Escuro/ })
        .getAttribute('aria-checked'),
    ).toBe('true');

    await user.click(within(menu).getByRole('menuitemradio', { name: /^Mono/ }));
    expect(useUi.getState().theme).toBe('mono');
    expect(screen.queryByRole('menu', { name: 'Temas' })).toBeNull();

    // Teclado: ↓ abre, setas percorrem, Enter escolhe.
    screen.getByRole('button', { name: /Escolher tema \(atual: Mono\)/ }).focus();
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('menu', { name: 'Temas' })).toBeTruthy();
    await user.keyboard('{ArrowDown}{Enter}');
    expect(useUi.getState().theme).toBe('midnight');
  });
});
