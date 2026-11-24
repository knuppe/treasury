import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import type { IncomeLedger } from '@/api/generated/IncomeLedger';
import { mockCall } from '@/api/mock';
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
});
