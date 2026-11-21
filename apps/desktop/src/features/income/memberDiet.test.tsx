/**
 * 02 · Família against the mock: food allergies and restrictions in the member's profile
 * (docs/gtin-inflation.md §7.2), saved with `food.diet_set`.
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

describe('02 · Família › allergies and restrictions', () => {
  it("marks Glúten on Ana's profile and saves to food.diet_set", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/familia');
    await screen.findByText('Studio Norte (PJ)');
    // The order of the cards follows the members: Ana first.
    await user.click(screen.getAllByRole('button', { name: 'Cadastro' })[0]!);
    const dialog = await screen.findByRole('dialog', { name: /Cadastro de Ana/ });
    within(dialog).getByRole('group', { name: 'Alergias e restrições alimentares' });
    expect(within(dialog).getByText(/Fica só no seu cofre/)).toBeTruthy();

    await user.click(within(dialog).getByRole('checkbox', { name: 'Glúten' }));
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    const diets = (await mockCall('food.diet_list', {})) as MemberDiet[];
    expect(diets).toEqual([{ member_id: 'mem-ana', member_name: 'Ana', restrictions: ['gluten'] }]);

    // Reopened, the profile shows what was saved.
    await user.click(screen.getAllByRole('button', { name: 'Cadastro' })[0]!);
    const again = await screen.findByRole('dialog', { name: /Cadastro de Ana/ });
    await waitFor(() =>
      expect(
        within(again).getByRole<HTMLInputElement>('checkbox', { name: 'Glúten' }).checked,
      ).toBe(true),
    );
  });

  it('the mock rejects an unknown restriction', async () => {
    await expect(
      mockCall('food.diet_set', { member_id: 'mem-ana', restrictions: ['bacon'] }),
    ).rejects.toMatchObject({ code: 'invalid' });
  });
});
