import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { setMockNewerAppVersion } from '@/api/mock/vaultExtras';
import { THEMES } from '@/design/themes';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

// Accounts shows the balance forecast (chart): jsdom has no canvas.
vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('shell', () => {
  it('warns in the top bar when a newer app has already opened the vault', async () => {
    const { queryClient } = renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });
    expect(screen.queryByText('Atualize o app')).toBeNull();
    setMockNewerAppVersion('9.9.9');
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    expect(await screen.findByText('Atualize o app')).toBeTruthy();
  });

  it('collapses the side menu (button and ⌘\\) and keeps the screens navigable', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    const nav = await screen.findByRole('navigation', { name: 'Telas' });
    expect(within(nav).getByText('Contas')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Recolher menu lateral' }));
    expect(useUi.getState().sidebarCollapsed).toBe(true);
    const collapsed = screen.getByRole('navigation', { name: 'Telas' });
    // Only the numbers show; the name stays in the accessible label (and in the tooltip).
    expect(within(collapsed).queryByText('Contas')).toBeNull();
    await user.click(within(collapsed).getByRole('link', { name: 'Contas' }));
    expect(await screen.findByRole('heading', { level: 1, name: /Contas/ })).toBeTruthy();

    await user.keyboard('{Control>}\\{/Control}');
    expect(useUi.getState().sidebarCollapsed).toBe(false);
    expect(screen.getByRole('button', { name: 'Recolher menu lateral' })).toBeTruthy();
  });

  it('⌘0 opens Market in Brazil and does not open where the country has no such screen', async () => {
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
    // A menu screen still opens by its number.
    await user.keyboard('{Control>}4{/Control}');
    expect(await screen.findByRole('heading', { level: 1, name: /Lançamentos/ })).toBeTruthy();

    await mockCall('country.set', { country: 'BR' });
    await act(async () => {
      await queryClient.invalidateQueries();
    });
    await user.keyboard('{Control>}0{/Control}');
    expect(await screen.findByRole('heading', { level: 1, name: /Mercado/ })).toBeTruthy();
  }, 60_000);

  it('the top bar theme picker lists all themes and applies the chosen one', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });

    await user.click(screen.getByRole('button', { name: /Escolher tema \(atual: Escuro\)/ }));
    const menu = screen.getByRole('menu', { name: 'Temas' });
    const items = within(menu).getAllByRole('menuitemradio');
    expect(items).toHaveLength(THEMES.length);
    // The description is cut off on the line; the tooltip shows the whole text.
    const kitt = THEMES.find((t) => t.id === 'kitt')!;
    await user.hover(within(menu).getByRole('menuitemradio', { name: /Scanner/ }));
    // Another tooltip may be open (the item focused when the menu opened): look for the item's own.
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

    // Keyboard: ↓ opens, arrows move through, Enter chooses.
    screen.getByRole('button', { name: /Escolher tema \(atual: Mono\)/ }).focus();
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('menu', { name: 'Temas' })).toBeTruthy();
    await user.keyboard('{ArrowDown}{Enter}');
    expect(useUi.getState().theme).toBe('midnight');
  });
});
