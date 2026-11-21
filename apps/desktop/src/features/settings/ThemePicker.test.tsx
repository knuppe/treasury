import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { THEMES } from '@/design/themes';
import { useUi, UI_STORAGE_KEY } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

describe('Settings › Preferences › theme', () => {
  it('shows a thumbnail of each theme and switches, saves and applies the theme', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    const group = await screen.findByRole('radiogroup', { name: 'Tema' });
    const options = within(group).getAllByRole('radio');
    expect(options.map((o) => o.getAttribute('aria-label'))).toEqual(THEMES.map((t) => t.label));
    // Each thumbnail applies its own theme only inside itself.
    for (const t of THEMES) {
      expect(
        group.querySelector(`[data-theme-option='${t.id}'] [data-theme='${t.id}']`),
      ).not.toBeNull();
    }
    expect(within(group).getByRole('radio', { name: 'Escuro' }).getAttribute('aria-checked')).toBe(
      'true',
    );

    await user.click(within(group).getByRole('radio', { name: 'Vidro' }));
    expect(useUi.getState().theme).toBe('glass');
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe('glass'));
    const saved = JSON.parse(localStorage.getItem(UI_STORAGE_KEY)!) as { state: { theme: string } };
    expect(saved.state.theme).toBe('glass');
    expect(within(group).getByRole('radio', { name: 'Vidro' }).getAttribute('aria-checked')).toBe(
      'true',
    );

    // Arrows walk through the themes, like in a radio group.
    await user.keyboard('{ArrowRight}');
    expect(useUi.getState().theme).toBe('kitt');
    expect(document.activeElement?.getAttribute('aria-label')).toBe('Scanner');
    await user.keyboard('{ArrowLeft}{ArrowLeft}');
    expect(useUi.getState().theme).toBe('paper');
  });

  it('the command bar has a command for each theme', async () => {
    const user = userEvent.setup();
    renderApp('unlocked');
    await screen.findByRole('navigation', { name: 'Telas' });
    await user.keyboard('{Control>}k{/Control}');
    const dialog = await screen.findByRole('dialog', { name: 'Barra de comando' });
    await user.type(within(dialog).getByRole('combobox'), 'tema');
    for (const t of THEMES) {
      expect(
        within(dialog).getByRole('option', { name: new RegExp(`Tema: ${t.label}`) }),
      ).toBeTruthy();
    }
    await user.clear(within(dialog).getByRole('combobox'));
    await user.type(within(dialog).getByRole('combobox'), 'vidro');
    // The order of the results changes as the data loads: picks the command by name.
    await user.click(await within(dialog).findByRole('option', { name: /Tema: Vidro/ }));
    await waitFor(() => expect(useUi.getState().theme).toBe('glass'));
  });
});
