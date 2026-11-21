/**
 * Trocar o idioma em Ajustes com o cofre aberto (docs/i18n-design.md › Escolha do idioma ›
 * Depois): o app oferece traduzir as categorias padrão; as renomeadas ficam como estão.
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

/** Escolhe o idioma no seletor (o rótulo sai no idioma atual). */
async function pickLanguage(user: ReturnType<typeof userEvent.setup>, name: string) {
  await user.click(await screen.findByRole('combobox', { name: /^(Idioma|Language)$/ }));
  await user.click(await screen.findByRole('option', { name }));
}

const names = async () =>
  ((await mockCall('categories.list', {})) as Category[]).map((c) => c.name);

describe('Idioma e categorias padrão', () => {
  it('oferece traduzir as categorias padrão e deixa as renomeadas', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    const padaria = ((await mockCall('categories.list', {})) as Category[]).find(
      (c) => c.name === 'Padaria',
    )!;
    await mockCall('categories.update', { id: padaria.id, name: 'Padoca' });

    // Em inglês: a oferta sai no idioma novo.
    await pickLanguage(user, 'English');
    expect(await screen.findByText('Translate the default categories too?')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Translate' }));
    await waitFor(async () => expect(await names()).toContain('Groceries'));
    expect(await names()).toContain('Padoca');
    expect(await screen.findByText(/categories translated$/)).toBeTruthy();

    // De volta ao pt-BR: a pergunta em pt-BR, e as padrão voltam ao nome de origem.
    await pickLanguage(user, 'Português (Brasil)');
    expect(await screen.findByText('Traduzir também as categorias padrão?')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Traduzir' }));
    await waitFor(async () => expect(await names()).toContain('Mercado'));
    expect(await names()).not.toContain('Groceries');
    expect(await names()).toContain('Padoca');
  });

  it('em espanhol também; depois de traduzir, não sobra nada a oferecer', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=preferencias');
    await pickLanguage(user, 'Español');
    expect(
      await screen.findByText('¿Traducir también las categorías predeterminadas?'),
    ).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Traducir' }));
    await waitFor(async () => expect(await names()).toContain('Supermercado'));
    // Já em espanhol: a mesma troca de novo não pergunta nada.
    expect(await mockCall('categories.untranslated_defaults', {})).toBe(0);
  });
});
