import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { Member } from '@/api/generated/Member';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

describe('06 · Consumption › Health', () => {
  it('with the source off, explains and turns on Open Food Facts; then fetches the data', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=saude');
    expect(await screen.findByRole('tab', { name: 'Saúde', selected: true })).toBeTruthy();
    const card = await screen.findByRole('region', { name: /Saúde da cesta/ });
    expect(within(card).getByText(/Só o código de barras sai do computador/)).toBeTruthy();
    expect(screen.getByText(/Não é orientação médica nem nutricional/)).toBeTruthy();
    const credit = screen.getByRole('link', { name: 'Dados: Open Food Facts (ODbL)' });
    expect(credit.getAttribute('href')).toBe('https://world.openfoodfacts.org');

    await user.click(within(card).getByRole('button', { name: 'Ligar o Open Food Facts' }));
    const sync = await within(card).findByRole('button', { name: 'Buscar dados de 7 produtos' });
    await user.click(sync);
    expect(
      await screen.findByText(/Nos últimos 3 meses, 31% do seu gasto em comida com dados/),
    ).toBeTruthy();
  });

  it('shows the headline, the "alto em" and the member\'s allergy alert', async () => {
    renderApp('unlocked', {}, '#/painel');
    await screen.findByRole('heading', { level: 1 });
    await mockCall('sources.set', { id: 'open_food_facts', enabled: true });
    await mockCall('food.open_data_sync', {});
    const member = ((await mockCall('household.list_members', {})) as Member[])[0]!;
    await mockCall('food.diet_set', { member_id: member.id, restrictions: ['gluten', 'sesame'] });
    window.location.hash = '#/consumo?aba=saude';

    expect(
      await screen.findByText(
        'Nos últimos 3 meses, 31% do seu gasto em comida com dados foi com ultraprocessados (era 38% há um ano).',
      ),
    ).toBeTruthy();
    const high = await screen.findByRole('list', { name: 'Alto em' });
    expect(within(high).getByText('Biscoito Recheado Exemplo 130g')).toBeTruthy();
    expect(within(high).getAllByText('Alto em açúcar adicionado').length).toBeGreaterThan(0);

    const alerts = await screen.findByRole('list', { name: 'Alergias e restrições' });
    expect(
      within(alerts).getByText(
        new RegExp(`${member.name}: Biscoito Recheado Exemplo 130g \\(Glúten\\)`),
      ),
    ).toBeTruthy();

    // Sesame only appears as a trace ("pode conter") in the sandwich bread.
    const sesame = within(alerts)
      .getByText(/Pão de Forma Exemplo 500g \(Gergelim\)/)
      .closest('li')!;
    expect(within(sesame).getByText(/pode conter/)).toBeTruthy();
  });

  it('Products: merges the suggested pair after confirming', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/consumo?aba=produtos');
    const card = await screen.findByRole('region', { name: /Talvez sejam o mesmo produto/ });
    await user.click(within(card).getByRole('button', { name: 'Ver' }));
    await user.click(
      within(card).getByRole('button', {
        name: 'Juntar IOG NAT EXEMPLO 170G em Iogurte Natural Exemplo 170g',
      }),
    );
    const dialog = await screen.findByRole('dialog', { name: 'Juntar os produtos?' });
    await user.click(within(dialog).getByRole('button', { name: 'Juntar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    await waitFor(() => expect(within(card).queryByText(/IOG NAT EXEMPLO 170G/)).toBeNull());
    expect(within(card).getByText('1 par de produtos com nomes parecidos.')).toBeTruthy();
  });
});
