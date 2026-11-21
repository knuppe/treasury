/** Points and miles in the mock (V159): same rules as the core (`cards/tests_loyalty.rs`). */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { LoyaltyProgram } from '@/api/generated/LoyaltyProgram';
import { mockCall, resetMock } from '@/api/mock';

import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday('2026-10-01');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock: points and miles', () => {
  it('balance by batches, expiry and transfer with bonus', async () => {
    const bank = await call<LoyaltyProgram>('cards.loyalty_create', {
      name: 'Banco Pontos',
      value_per_1000: 2_500,
    });
    const air = await call<LoyaltyProgram>('cards.loyalty_create', {
      name: 'Aérea Milhas',
      kind: 'airline',
    });
    const add = (date: string, points: number, kind: string, expires_on?: string) =>
      call('cards.loyalty_add_entry', { program_id: bank.id, date, points, kind, expires_on });
    await add('2026-01-10', 10_000, 'earn', '2026-12-31');
    await add('2026-02-10', 5_000, 'earn', '2026-11-30');
    await add('2026-03-10', 3_000, 'earn', '2026-06-30');
    await add('2026-04-01', 4_000, 'redeem');
    let list = await call<LoyaltyProgram[]>('cards.loyalty_list');
    const b = list.find((p) => p.id === bank.id)!;
    expect([b.balance, b.next_expiry, b.next_expiry_points, b.estimated_value]).toEqual([
      14_000,
      '2026-11-30',
      4_000,
      35_000,
    ]);
    list = await call<LoyaltyProgram[]>('cards.loyalty_transfer', {
      from_program_id: bank.id,
      to_program_id: air.id,
      date: '2026-10-01',
      points_out: 5_000,
      points_in: 10_000,
    });
    expect(list.map((p) => [p.name, p.balance])).toEqual([
      ['Aérea Milhas', 10_000],
      ['Banco Pontos', 9_000],
    ]);
  });
});
