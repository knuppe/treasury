import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { call } from '@/api/client';
import type { LearnedRule } from '@/api/generated/LearnedRule';
import type { Rule } from '@/api/generated/Rule';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('Lançamentos › Regras sugeridas', () => {
  it('oferece a regra, cria só quando a pessoa pede e some depois', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=regras');
    const card = await screen.findByRole(
      'region',
      { name: 'Regras sugeridas' },
      { timeout: 15_000 },
    );
    const [offer] = await call<LearnedRule[]>('rules.learned', {});
    expect(offer).toBeTruthy();
    expect(within(card).getByText(offer!.key)).toBeTruthy();
    expect(within(card).getByText(offer!.category_path)).toBeTruthy();
    const before = (await call<Rule[]>('rules.list', {})).length;
    await user.click(within(card).getByRole('button', { name: 'Criar regra' }));
    await waitFor(async () =>
      expect((await call<Rule[]>('rules.list', {})).length).toBe(before + 1),
    );
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Regras sugeridas' })).toBeNull(),
    );
  });

  it('não sugerir tira a oferta sem criar regra', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos?aba=regras');
    const card = await screen.findByRole(
      'region',
      { name: 'Regras sugeridas' },
      { timeout: 15_000 },
    );
    const before = (await call<Rule[]>('rules.list', {})).length;
    await user.click(within(card).getByRole('button', { name: 'Não sugerir' }));
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Regras sugeridas' })).toBeNull(),
    );
    expect((await call<Rule[]>('rules.list', {})).length).toBe(before);
    expect(await call<LearnedRule[]>('rules.learned', {})).toEqual([]);
  });
});
