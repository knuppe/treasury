import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { renderApp } from '@/test/renderApp';

const BEGINNER = [
  /Nada seguro/,
  /nunca controlei/,
  /Saber para onde o dinheiro está indo/,
  /Não sei o que é/,
  /Benefícios do governo/,
  /Não entendo e fico perdido/,
  /Bem simples/,
];

describe('finances per family member', () => {
  it('board per member, "Ver como" and a member questionnaire', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/painel');
    const card = await screen.findByRole('list', { name: 'Membros' });
    const bruno = within(card).getByText('Bruno').closest('li')!;
    expect(within(bruno).getByText(/Avançado \(da família\)/)).toBeTruthy();

    // Bruno's questionnaire: his level, not the family's.
    await user.click(within(bruno).getByRole('button', { name: 'Fazer o questionário' }));
    expect(await screen.findByText('Questionário de Bruno')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: /Quero personalizar/ }));
    for (const [i, option] of BEGINNER.entries()) {
      await screen.findByText(`Pergunta ${i + 1} de 7`);
      await user.click(screen.getByRole(i === 2 ? 'checkbox' : 'radio', { name: option }));
      await user.click(screen.getByRole('button', { name: i === 6 ? 'Concluir' : 'Continuar' }));
    }
    await user.click(await screen.findByRole('button', { name: 'Começar' }));

    // Viewing as Bruno: simple dashboard and the description of his finances.
    expect(await screen.findByRole('region', { name: 'Resumo do mês' })).toBeTruthy();
    expect(screen.getByText(/As finanças de Bruno/)).toBeTruthy();
    expect(screen.queryByRole('list', { name: 'Membros' })).toBeNull();

    // Back to the family: full dashboard (the family level did not change) and the per-member board.
    await user.click(screen.getByRole('combobox', { name: 'Ver como' }));
    await user.click(await screen.findByRole('option', { name: 'Família toda' }));
    await waitFor(() => expect(screen.queryByText(/As finanças de Bruno/)).toBeNull());
    expect(screen.queryByRole('region', { name: 'Resumo do mês' })).toBeNull();
    const again = await screen.findByRole('list', { name: 'Membros' });
    expect(
      within(within(again).getByText('Bruno').closest('li')!).getByText('Iniciante'),
    ).toBeTruthy();
  });
});
