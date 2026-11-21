/**
 * 02 · Família contra o mock: alergias e restrições alimentares no cadastro do membro
 * (docs/gtin-inflation.md §7.2), gravadas com `food.diet_set`.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { MemberDiet } from '@/api/generated/MemberDiet';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

describe('02 · Família › alergias e restrições', () => {
  it('marca Glúten no cadastro da Ana e grava em food.diet_set', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    await screen.findByText('Studio Norte (PJ)');
    // A ordem dos cartões segue os membros: Ana primeiro.
    await user.click(screen.getAllByRole('button', { name: 'Cadastro' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: /Cadastro de Ana/ });
    within(dialog).getByRole('group', { name: 'Alergias e restrições alimentares' });
    expect(within(dialog).getByText(/Fica só no seu cofre/)).toBeTruthy();

    await user.click(within(dialog).getByRole('checkbox', { name: 'Glúten' }));
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    const diets = (await mockCall('food.diet_list', {})) as MemberDiet[];
    expect(diets).toEqual([{ member_id: 'mem-ana', member_name: 'Ana', restrictions: ['gluten'] }]);

    // Reaberto, o cadastro mostra o que ficou gravado.
    await user.click(screen.getAllByRole('button', { name: 'Cadastro' })[0]!);
    const again = await screen.findByRole('dialog', { name: /Cadastro de Ana/ });
    await waitFor(() =>
      expect(
        within(again).getByRole<HTMLInputElement>('checkbox', { name: 'Glúten' }).checked,
      ).toBe(true),
    );
  });

  it('o mock recusa restrição desconhecida', async () => {
    await expect(
      mockCall('food.diet_set', { member_id: 'mem-ana', restrictions: ['bacon'] }),
    ).rejects.toMatchObject({ code: 'invalid' });
  });
});
