import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { renderApp } from '@/test/renderApp';

describe('Ajustes › Fontes de dados › brapi.dev', () => {
  it('cadastra o token, testa, mostra o plano, o consumo e muda o limite', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=fontes');
    const card = await screen.findByRole('region', { name: /brapi\.dev/i }, { timeout: 15_000 });
    await user.type(within(card).getByLabelText('Token da brapi.dev'), 'token-de-teste-123');
    await user.click(within(card).getByRole('button', { name: 'Salvar token' }));
    const usage = await within(card).findByRole('region', { name: 'Uso no mês' });
    expect(within(usage).getByText(/ainda não lido/)).toBeTruthy();
    await user.click(within(usage).getByRole('button', { name: 'Testar token' }));
    expect(await screen.findByText('Token aceito: plano Gratuito')).toBeTruthy();
    expect(
      within(usage).getByText(/1\.248 de 12\.000 pedidos \(cota do plano: 15\.000\)/),
    ).toBeTruthy();
    expect(
      within(usage).getByText(/conferência de indicadores e proventos é dos planos pagos/),
    ).toBeTruthy();
  });
});
