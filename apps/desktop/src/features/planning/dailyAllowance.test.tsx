/**
 * "Quanto posso gastar hoje" (how much can I spend today): with today's amount already used and
 * money left in the month, the highlight is the per-day amount from tomorrow on (report from
 * 2026-10-20, demo vault), and the calculation shows today's spending separately, without
 * discounting it twice.
 */
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router';
import { describe, expect, it, vi } from 'vitest';

import type { DailyAllowance } from '@/api/generated/DailyAllowance';
import type { DataCoverage } from '@/api/generated/DataCoverage';

let data: DailyAllowance;
let coverage: DataCoverage | undefined;
vi.mock('@/api/hooks/planning', () => ({ useDailyAllowance: () => ({ data }) }));
vi.mock('@/api/hooks/attention', () => ({
  useAttention: () => ({ data: coverage ? { coverage } : undefined }),
}));

import { DailyAllowanceCard } from './DailyAllowanceCard';

const base: DailyAllowance = {
  month: '2026-10',
  days_left: 12,
  income: 557_723,
  spent: 362_544,
  spent_today: 28_425,
  fixed_remaining: 0,
  savings_remaining: 0,
  available: 195_179,
  per_day: 18_633,
  today_left: 0,
  per_day_from_tomorrow: 17_743,
  currency: 'BRL',
  status: 'ok',
  explanation: '',
};

describe('Quanto posso gastar hoje', () => {
  it("today's amount used: highlights the per-day amount from tomorrow on", async () => {
    data = base;
    const user = userEvent.setup();
    render(<DailyAllowanceCard />);
    const card = screen.getByRole('region', { name: 'Quanto posso gastar hoje' });
    expect(within(card).getByText('Por dia a partir de amanhã')).toBeTruthy();
    expect(within(card).getByText('R$ 177,43')).toBeTruthy();
    expect(within(card).queryByText('R$ 0,00')).toBeNull();
    // The calculation: spending up to yesterday, what is left at the start of today, today's
    // spending and tomorrow's.
    await user.click(within(card).getByRole('button', { name: 'Como calculei' }));
    expect(within(card).getByText('Gastos até ontem')).toBeTruthy();
    expect(within(card).getByText('−R$ 3.341,19')).toBeTruthy();
    expect(within(card).getByText(/R\$ 2\.236,04/)).toBeTruthy();
    expect(within(card).getByText('Sobra para amanhã em diante')).toBeTruthy();
    expect(within(card).getByText(/Dividido por 11 dias, a partir de amanhã/)).toBeTruthy();
  });

  it('no income in the month: invites to record it instead of a red zero', () => {
    data = {
      ...base,
      income: 0,
      spent: 0,
      spent_today: 0,
      available: 0,
      per_day: 0,
      per_day_from_tomorrow: 0,
      status: 'no_income',
    };
    render(<DailyAllowanceCard />);
    const card = screen.getByRole('region', { name: 'Quanto posso gastar hoje' });
    expect(within(card).getByText(/Anote a renda do mês/)).toBeTruthy();
    expect(within(card).queryByText('R$ 0,00')).toBeNull();
    expect(within(card).queryByText(/todo comprometido/)).toBeNull();
    expect(within(card).queryByRole('button', { name: 'Como calculei' })).toBeNull();
  });

  it("with money left today, the highlight is still today's", () => {
    data = { ...base, spent_today: 5_000, today_left: 13_633, per_day_from_tomorrow: 19_871 };
    render(<DailyAllowanceCard />);
    const card = screen.getByRole('region', { name: 'Quanto posso gastar hoje' });
    expect(within(card).getAllByText('Quanto posso gastar hoje').length).toBeGreaterThan(0);
    expect(within(card).getByText('R$ 136,33')).toBeTruthy();
    expect(within(card).getByText(/por dia a partir de amanhã \(11 dias\)/)).toBeTruthy();
    // With no idle accounts, no coverage warning.
    expect(within(card).queryByRole('status')).toBeNull();
  });

  it('with an account with no recent statement, warns that the number may be incomplete', () => {
    data = base;
    coverage = {
      status: 'partial',
      accounts: 2,
      complete_until: '2026-08-01',
      stale: [
        { account_id: 'c2', name: 'Poupança', last_txn_date: '2026-08-01', staleness_days: 75 },
      ],
    };
    render(
      <MemoryRouter>
        <DailyAllowanceCard />
      </MemoryRouter>,
    );
    const card = screen.getByRole('region', { name: 'Quanto posso gastar hoje' });
    const note = within(card).getByRole('status');
    expect(note.textContent).toContain('1 conta sem extrato recente (Poupança).');
    expect(note.textContent).toContain('Os totais podem estar incompletos.');
    expect(within(note).getByRole('link', { name: 'Importar extrato' }).getAttribute('href')).toBe(
      '/lancamentos?importar=1&conta=c2',
    );
    coverage = undefined;
  });
});
