/**
 * Switching the language in Ajustes with the vault open (docs/i18n-design.md › Escolha do idioma ›
 * Depois): the app offers to translate the default categories; renamed ones stay as they are.
 */
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import type { Category } from '@/api/generated/Category';
import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  await applyLocale('pt-BR');
});

/** Picks the language in the selector (the label comes out in the current language). */
async function pickLanguage(user: ReturnType<typeof userEvent.setup>, name: string) {
  await user.click(await screen.findByRole('combobox', { name: /^(Idioma|Language)$/ }));
  await user.click(await screen.findByRole('option', { name }));
}

const names = async () =>
  ((await mockCall('categories.list', {})) as Category[]).map((c) => c.name);

describe('Language and default categories', () => {
  it('offers to translate the default categories and leaves the renamed ones', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    const padaria = ((await mockCall('categories.list', {})) as Category[]).find(
      (c) => c.name === 'Padaria',
    )!;
    await mockCall('categories.update', { id: padaria.id, name: 'Padoca' });

    // In English: the offer comes out in the new language.
    await pickLanguage(user, 'English');
    expect(await screen.findByText('Translate the default categories too?')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Translate' }));
    await waitFor(async () => expect(await names()).toContain('Groceries'));
    expect(await names()).toContain('Padoca');
    expect(await screen.findByText(/categories translated$/)).toBeTruthy();

    // Back to pt-BR: the question in pt-BR, and the defaults go back to their original name.
    await pickLanguage(user, 'Português (Brasil)');
    expect(await screen.findByText('Traduzir também as categorias padrão?')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Traduzir' }));
    await waitFor(async () => expect(await names()).toContain('Mercado'));
    expect(await names()).not.toContain('Groceries');
    expect(await names()).toContain('Padoca');
  });

  it('in Spanish too; after translating, nothing is left to offer', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    await pickLanguage(user, 'Español');
    expect(
      await screen.findByText('¿Traducir también las categorías predeterminadas?'),
    ).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Traducir' }));
    await waitFor(async () => expect(await names()).toContain('Supermercado'));
    // Already in Spanish: the same switch again asks nothing.
    expect(await mockCall('categories.untranslated_defaults', {})).toBe(0);
  });
});
