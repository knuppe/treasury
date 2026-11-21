/** "Vale a pena?" no mock: mesmas regras do núcleo (`cards/tests_value.rs`). */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { CardValueReport } from '@/api/generated/CardValueReport';
import { mockCall, resetMock } from '@/api/mock';

import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday('2026-10-01');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock: vale a pena', () => {
  it('custo e retorno do cartão', async () => {
    const card = await call<AccountView>('accounts.create', {
      name: 'Cartão Teste',
      kind: 'credit_card',
      opening_date: '2026-01-01',
      card: { closing_day: 3, due_day: 10 },
    });
    await call('cards.set_reward_program', {
      account_id: card.id,
      kind: 'cashback',
      base_rate: '0.02',
      starts_on: '2026-01-01',
    });
    const add = (date: string, amount: number, description: string, kind = 'expense') =>
      call('transactions.create', { account_id: card.id, date, amount, description, kind });
    await add('2026-05-10', -300_000, 'Loja');
    await add('2026-05-11', -10_000, 'ANUIDADE 5/12');
    await add('2026-06-12', -4_000, 'JUROS ROTATIVO');
    await add('2026-06-13', -1_000, 'IOF COMPRA INTERNACIONAL');
    await add('2026-06-20', 6_000, 'CASHBACK', 'income');
    const r = await call<CardValueReport>('cards.value');
    const v = r.cards.find((c) => c.account_id === card.id)!;
    expect([v.spend, v.annual_fee, v.charges, v.iof, v.cashback, v.net]).toEqual([
      300_000, 10_000, 4_000, 1_000, 6_000, -9_000,
    ]);
    expect(r.scenarios.find((s) => s.account_id === card.id)?.reward).toBeGreaterThan(6_000);
  });
});
