import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('Dashboard › What Treasury noticed', () => {
  it('shows the insights without AI, opens the inflation groups and dismisses', async () => {
    const user = userEvent.setup();
    localStorage.removeItem('treasury.insights.dismissed');
    renderApp('unlocked', {}, '#/painel');
    const card = await screen.findByRole(
      'region',
      { name: 'O que o Treasury percebeu' },
      { timeout: 5_000 },
    );
    expect(within(card).getByText(/Sua inflação em 12 meses: 5,6%/)).toBeTruthy();
    // The attention insights (fees and interest) live in "Para resolver".
    expect(within(card).queryByText(/em tarifas e juros em 12 meses/)).toBeNull();
    const todo = screen.getByRole('region', { name: 'Para resolver' });
    expect(await within(todo).findByText(/em tarifas e juros em 12 meses/)).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Ver por grupo' }));
    const table = within(card).getByRole('table', { name: 'Inflação pessoal por grupo' });
    expect(within(table).getByText('Alimentação e bebidas')).toBeTruthy();
    const before = within(card).getAllByRole('button', { name: 'Dispensar' }).length;
    await user.click(within(card).getAllByRole('button', { name: 'Dispensar' })[0]!);
    await waitFor(() =>
      expect(within(card).getAllByRole('button', { name: 'Dispensar' }).length).toBe(before - 1),
    );
    expect(within(card).queryByText(/Sua inflação em 12 meses/)).toBeNull();
    expect(localStorage.getItem('treasury.insights.dismissed')).toContain('inflation:2026-09');
  });
});
