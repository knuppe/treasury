import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { MOCK_PASSWORD } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

describe('vários cofres', () => {
  it('cria um cofre novo a partir do bloqueio e volta ao principal', async () => {
    const user = userEvent.setup();
    renderApp('locked');
    await screen.findByRole('button', { name: 'Desbloquear' });
    await user.click(screen.getByText('Outros cofres'));
    await user.click(screen.getByRole('button', { name: 'Novo cofre' }));
    await user.type(screen.getByLabelText('Nome do cofre'), 'Empresa');
    await user.type(screen.getByLabelText('Finalidade (opcional)'), 'PJ');
    await user.click(screen.getByRole('button', { name: 'Criar cofre' }));

    // O cofre novo fica pronto para criar, com o nome dele no topo.
    expect(await screen.findByText('00 · Novo cofre · Empresa')).toBeTruthy();
    await user.click(screen.getByText('Outros cofres (1)'));
    const list = screen.getByRole('list', { name: 'Cofres' });
    expect(within(list).getByText(/ainda não criado/)).toBeTruthy();
    await user.click(within(list).getByRole('button', { name: 'Abrir' }));

    // De volta ao principal: bloqueado, com a senha dele.
    expect(await screen.findByText('00 · Cofre bloqueado · Principal')).toBeTruthy();
    await user.type(screen.getByLabelText('Senha mestra'), MOCK_PASSWORD);
    await user.click(screen.getByRole('button', { name: 'Desbloquear' }));
    expect(await screen.findByRole('navigation', { name: 'Telas' })).toBeTruthy();
  });

  it('Ajustes › Cofres: renomeia e não tira da lista o cofre em uso', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=cofres');
    const list = await screen.findByRole('list', { name: 'Cofres' });
    await user.click(within(list).getByRole('button', { name: 'Editar' }));
    const name = within(list).getByLabelText('Nome');
    await user.clear(name);
    await user.type(name, 'Família');
    await user.click(within(list).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(within(list).getByText('Família')).toBeTruthy());
    expect(within(list).queryByRole('button', { name: 'Tirar da lista' })).toBeNull();
  });

  it('menu de contexto: cria uma cópia e remove com confirmação', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=cofres');
    const list = await screen.findByRole('list', { name: 'Cofres' });
    await user.click(within(list).getByRole('button', { name: 'Ações de Principal' }));
    let menu = await screen.findByRole('menu', { name: 'Ações de Principal' });
    // O cofre em uso não sai.
    const remove = within(menu).getByRole('menuitem', { name: /Remover/ });
    expect(remove.getAttribute('aria-disabled')).toBe('true');
    await user.click(within(menu).getByRole('menuitem', { name: 'Criar uma cópia' }));
    expect(await within(list).findByText('Principal (cópia)')).toBeTruthy();

    // Botão direito na cópia: remover pergunta o que fazer com o arquivo.
    const copy = within(list).getByText('Principal (cópia)').closest('li')!;
    fireEvent.contextMenu(copy, { clientX: 40, clientY: 40 });
    menu = await screen.findByRole('menu', { name: 'Ações de Principal (cópia)' });
    await user.click(within(menu).getByRole('menuitem', { name: /Remover/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Remover Principal (cópia)?' });
    expect(within(dialog).getByRole('radio', { name: /Só tirar da lista/ })).toHaveProperty(
      'checked',
      true,
    );
    await user.click(
      within(dialog).getByRole('radio', { name: /mandar o arquivo para a Lixeira/ }),
    );
    await user.click(
      within(dialog).getByRole('button', { name: 'Remover e mandar para a Lixeira' }),
    );
    await waitFor(() => expect(within(list).queryByText('Principal (cópia)')).toBeNull());
    expect(await screen.findByText('O arquivo foi para a Lixeira.')).toBeTruthy();
  });
});
