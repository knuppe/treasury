/** Cashback no mock (V158): mesmas regras do núcleo (`cards/tests_rewards.rs`). */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { RewardsView } from '@/api/generated/RewardsView';
import { mockCall, resetMock } from '@/api/mock';

import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday('2026-12-01');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock: cashback', () => {
  it('previsto, recebido e o que não veio', async () => {
    const card = await call<AccountView>('accounts.create', {
      name: 'Cartão Teste',
      kind: 'credit_card',
      opening_date: '2026-01-01',
      card: { closing_day: 3, due_day: 10 },
    });
    await call('cards.set_reward_program', {
      account_id: card.id,
      kind: 'cashback',
      base_rate: '0.01',
      payout_days: 20,
      starts_on: '2026-01-01',
    });
    const add = (date: string, amount: number, description: string, kind = 'expense') =>
      call('transactions.create', { account_id: card.id, date, amount, description, kind });
    await add('2026-09-10', -100_000, 'Loja');
    await add('2026-09-15', -5_000, 'ANUIDADE 1/12');
    await add('2026-10-10', -50_000, 'Mercado');
    await add('2026-10-08', 1_000, 'CASHBACK', 'income');
    const v = await call<RewardsView>('cards.rewards', { account_id: card.id });
    expect(v.statements.map((s) => [s.reference_month, s.expected, s.received, s.missing])).toEqual(
      [
        ['2026-11', 500, 0, true],
        ['2026-10', 1_000, 1_000, false],
      ],
    );
  });
});
