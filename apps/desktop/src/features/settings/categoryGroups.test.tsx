/** Ajustes › Categorias: spend group on the row, in bulk and in the filter (pt-BR, mock). */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import type { Category } from '@/api/generated/Category';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

const get = (id: string) => mockCall('categories.get', { id }) as Promise<Category>;

async function openCategories() {
  renderApp('unlocked', {}, '#/ajustes?secao=categorias');
  return screen.findByRole('tree', { name: 'Categorias de despesa' });
}

describe('Ajustes › Categorias: spend groups', () => {
  it('changes the group on the row: the category and the subcategories that followed its group', async () => {
    const user = userEvent.setup();
    const tree = await openCategories();
    const select = within(tree).getByRole('combobox', { name: 'Grupo de Transporte' });
    expect(select.textContent).toContain('Essenciais');
    // Aplicativos is an exception (non-essential inside Transporte) and stays as it is.
    expect(
      within(tree).getByRole('combobox', { name: 'Grupo de Aplicativos' }).textContent,
    ).toContain('Não essencial');
    await user.click(select);
    await user.click(await screen.findByRole('option', { name: 'Dívidas' }));
    expect(
      await screen.findByText('Transporte e 6 subcategorias agora estão em Dívidas'),
    ).toBeTruthy();
    const [root, fuel, apps] = await Promise.all([
      get('cat-transporte'),
      get('cat-transporte-combustivel'),
      get('cat-transporte-aplicativos'),
    ]);
    expect(root.spend_group).toBe('debt');
    // Debt still counts as mandatory spending (emergency fund).
    expect(root.is_essential).toBe(true);
    expect(fuel.spend_group).toBe('debt');
    expect(apps.spend_group).toBe('non_essential');
    await waitFor(() =>
      expect(
        within(tree).getByRole('combobox', { name: 'Grupo de Combustível' }).textContent,
      ).toContain('Dívidas'),
    );
  });

  it('changes the group of several in bulk and filters the tree by group', async () => {
    const user = userEvent.setup();
    const tree = await openCategories();
    // Marking the parent takes the subcategories along.
    await user.click(within(tree).getByRole('checkbox', { name: 'Selecionar Pets' }));
    await user.click(within(tree).getByRole('checkbox', { name: 'Selecionar Streaming' }));
    const bar = screen.getByRole('region', { name: 'Categorias selecionadas' });
    expect(within(bar).getByText('4 categorias selecionadas')).toBeTruthy();
    await user.click(within(bar).getByRole('combobox', { name: 'Mudar o grupo das selecionadas' }));
    await user.click(await screen.findByRole('option', { name: 'Essenciais' }));
    expect(await screen.findByText('4 categorias agora estão em Essenciais')).toBeTruthy();
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Categorias selecionadas' })).toBeNull(),
    );
    for (const id of ['cat-pets', 'cat-pets-racao', 'cat-pets-veterinario']) {
      const c = await get(id);
      expect(c.spend_group).toBe('essential');
      expect(c.is_essential).toBe(true);
    }
    expect((await get('cat-assinaturas-streaming')).spend_group).toBe('essential');
    expect((await get('cat-assinaturas-software')).spend_group).toBe('non_essential');

    // Filter: only what is in Dívidas (the parent shows up as context for the subcategory).
    await user.click(
      within(screen.getByRole('radiogroup', { name: 'Filtrar por grupo' })).getByRole('radio', {
        name: 'Dívidas',
      }),
    );
    await waitFor(() => expect(within(tree).queryByText('Moradia')).toBeNull());
    expect(within(tree).getByText('Juros e multas')).toBeTruthy();
    expect(within(tree).getByText('Impostos e taxas')).toBeTruthy();
    expect(within(tree).queryByText('Tarifas bancárias')).toBeNull();
  });

  it("a new subcategory is born in the parent category's group", async () => {
    const user = userEvent.setup();
    const tree = await openCategories();
    await user.click(within(tree).getByRole('button', { name: 'Nova subcategoria em Moradia' }));
    const dialog = await screen.findByRole('dialog', { name: /Nova subcategoria de Moradia/ });
    expect(within(dialog).getByRole('combobox', { name: 'Grupo' }).textContent).toContain(
      'Essenciais',
    );
    await user.type(within(dialog).getByLabelText('Nome'), 'Jardinagem');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    expect(await within(tree).findByText('Jardinagem')).toBeTruthy();
    const created = ((await mockCall('categories.list', { kind: 'expense' })) as Category[]).find(
      (c) => c.name === 'Jardinagem',
    );
    expect(created?.spend_group).toBe('essential');
    expect(created?.is_essential).toBe(true);
  });
});
