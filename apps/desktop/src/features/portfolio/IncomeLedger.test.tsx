import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import type { IncomeLedger } from '@/api/generated/IncomeLedger';
import { mockCall } from '@/api/mock';
import { extraMockHandler, registerMockHandlers } from '@/api/mockRegistry';
import { renderApp } from '@/test/renderApp';

import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
});
afterEach(() => {
  vi.useRealTimers();
});

describe('income ledger', () => {
  it('shows what the sources say was received and applies the fixes in one go', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime.bind(vi) });
    renderApp('unlocked', {}, '#/carteira?aba=proventos');
    const before = (await mockCall('portfolio.income_ledger', {})) as IncomeLedger;
    const official = before.entries.filter((e) => e.state === 'missing' && e.official);
    expect(official.length).toBeGreaterThan(0);
    expect(before.auto).toBe(false);
    const summary = await screen.findByRole('region', { name: 'Proventos pelas fontes oficiais' });
    expect(within(summary).getByText('Recebido pelas fontes')).toBeTruthy();
    await user.click(within(summary).getByRole('button', { name: 'Revisar e corrigir' }));
    const dialog = await screen.findByRole('dialog', {
      name: 'Corrigir os proventos pelos avisos oficiais',
    });
    const table = await within(dialog).findByRole('table', { name: 'Correções propostas' });
    const missing = before.entries.filter((e) => e.state === 'missing');
    expect(within(table).getAllByText('Lançar').length).toBe(missing.length);
    // Without an official notice: listed, but unchecked (check the statement first).
    expect(within(table).getAllByText(/sem aviso oficial/).length).toBe(
      missing.length - official.length,
    );
    await user.click(within(dialog).getByRole('button', { name: /^Aplicar/ }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());

    const after = (await mockCall('portfolio.income_ledger', {})) as IncomeLedger;
    expect(after.auto).toBe(true);
    expect(after.entries.filter((e) => e.state === 'missing' && e.official)).toHaveLength(0);
    expect(after.entries.filter((e) => e.state === 'missing' && !e.official).length).toBe(
      missing.length - official.length,
    );
    expect(await screen.findByText(/Proventos oficiais entram sozinhos desde/)).toBeTruthy();
  });

  it('offers to record the credit of a distribution received without one', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime.bind(vi) });
    // The mock ledger never leaves a credit pending: one recorded distribution is marked here.
    const original = extraMockHandler('portfolio.income_ledger')!;
    let credited: string[] | null = null;
    registerMockHandlers({
      'portfolio.income_ledger': (a) => {
        const all = original({ ...a, only_problems: false }) as IncomeLedger;
        const target = all.entries.find((e) => e.recorded && e.state === 'matches')!;
        const l = original(a) as IncomeLedger;
        const pending = credited == null;
        const marked = { ...target, cash_pending: pending };
        return {
          ...l,
          entries: l.entries.some((e) => e.key === target.key)
            ? l.entries.map((e) => (e.key === target.key ? marked : e))
            : pending
              ? [marked, ...l.entries]
              : l.entries,
          counts: { ...l.counts, cash_pending: pending ? 1 : 0 },
        };
      },
      'portfolio.income_cash': (a) => {
        credited = a.ids as string[];
        return { created: credited.length, change_set_id: null };
      },
    });
    try {
      renderApp('unlocked', {}, '#/carteira?aba=proventos');
      const summary = await screen.findByRole('region', {
        name: 'Proventos pelas fontes oficiais',
      });
      expect(
        await within(summary).findByText('1 provento recebido sem o crédito na conta'),
      ).toBeTruthy();
      await user.click(within(summary).getByRole('button', { name: 'Lançar o crédito' }));
      await waitFor(() => expect(credited).toHaveLength(1));
      const ledger = (await mockCall('portfolio.income_ledger', {})) as IncomeLedger;
      expect(ledger.counts.cash_pending).toBe(0);
      await waitFor(() =>
        expect(screen.queryByText('1 provento recebido sem o crédito na conta')).toBeNull(),
      );
    } finally {
      registerMockHandlers({ 'portfolio.income_ledger': original });
    }
  });
});
