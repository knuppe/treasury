/**
 * Settings, welcome and demo vault in English (docs/i18n-design.md): groups, sections,
 * search and section texts come from the `ajustes`, `perfil` and `demo` catalogs.
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

afterEach(async () => {
  await applyLocale('pt-BR');
});

describe('Settings in English', () => {
  it('overview, search and a section in the chosen language', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes');
    const overview = await screen.findByRole('region', { name: 'Settings overview' });
    for (const g of ['You and your family', 'Security and data', 'Integrations']) {
      expect(within(overview).getByRole('heading', { name: g })).toBeTruthy();
    }
    const nav = screen.getByRole('navigation', { name: 'Settings sections' });
    await user.type(within(nav).getByRole('searchbox', { name: 'Search settings' }), 'theme');
    await user.keyboard('{Enter}');
    const section = await screen.findByRole('region', { name: 'Appearance and use' });
    expect(within(section).getByRole('radiogroup', { name: 'Theme' })).toBeTruthy();
    expect(within(section).getByText('Privacy mode')).toBeTruthy();

    await user.click(within(section).getByRole('button', { name: 'Settings' }));
    await user.click(
      within(await screen.findByRole('region', { name: 'Settings overview' })).getByRole('button', {
        name: /^Backups:/,
      }),
    );
    expect(await screen.findByRole('region', { name: 'Backup health' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Back up now' })).toBeTruthy();
  });

  it('welcome and demo wizard', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/boas-vindas?refazer=1');
    expect(await screen.findByRole('heading', { name: 'Retake the questionnaire' })).toBeTruthy();
    expect(screen.getByRole('button', { name: /I want to tailor the app to me/ })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Create a demo vault' }));
    const dialog = await screen.findByRole('dialog', { name: 'Demo vault' });
    expect(within(dialog).getByRole('radio', { name: /Family in financial trouble/ })).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Create and open' })).toBeTruthy();
    expect(within(dialog).getByText(/opens without a password/)).toBeTruthy();
  });
});
