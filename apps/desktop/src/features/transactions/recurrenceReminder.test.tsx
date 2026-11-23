/**
 * Recurring expense without a due date reminder (a cleaner paid every week): the bell on the
 * recurrence turns the reminder off and on, and the row says so.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { call } from '@/api/client';
import type { Recurrence } from '@/api/generated/Recurrence';
import { renderApp } from '@/test/renderApp';

describe('recurrence reminder', () => {
  it('the bell turns the due date reminder off and back on', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=recorrencias');
    const grid = await screen.findByRole('grid', { name: 'Recorrências' });
    const first = (await call<Recurrence[]>('recurrences.list', {}))[0]!;
    expect(first.remind).toBe(true);
    const bell = within(grid).getByRole('button', {
      name: `Avisa o vencimento de ${first.description} (clique para não avisar)`,
    });
    await user.click(bell);
    await waitFor(async () =>
      expect((await call<Recurrence>('recurrences.get', { id: first.id })).remind).toBe(false),
    );
    expect(await within(grid).findByText('sem aviso')).toBeTruthy();
    await user.click(
      within(grid).getByRole('button', {
        name: `Não avisa o vencimento de ${first.description} (clique para avisar)`,
      }),
    );
    await waitFor(async () =>
      expect((await call<Recurrence>('recurrences.get', { id: first.id })).remind).toBe(true),
    );
  });
});
