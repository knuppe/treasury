/** Detected installments in the mock: same rules as the core (`installments/tests.rs`). */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { DetectedInstallments } from '@/api/generated/DetectedInstallments';
import { mockCall, resetMock } from '@/api/mock';

import { parseInstallment } from '../installments';
import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday('2026-09-20');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock: detected installments', () => {
  it('reads the common patterns', () => {
    expect(parseInstallment('Lj Exemplo Centro - Parcela 1/2')).toMatchObject({
      number: 1,
      total: 2,
      base: 'Lj Exemplo Centro',
    });
    expect(parseInstallment('PREST.EMPREST 003/005')).toMatchObject({
      number: 3,
      total: 5,
      loan: true,
      base: 'PREST.EMPREST',
    });
    expect(parseInstallment('Parcela 5/3')).toBeNull();
  });

  it('loan on the account: projection and ignore', async () => {
    const acc = await call<AccountView>('accounts.create', {
      name: 'Conta Teste',
      kind: 'checking',
      opening_date: '2026-01-01',
    });
    await call('transactions.create', {
      account_id: acc.id,
      date: '2026-09-05',
      amount: -50_000,
      description: 'PREST.EMPREST 003/005',
    });
    const r = await call<DetectedInstallments>('installments.detected');
    const s = r.series.find((x) => x.account_id === acc.id)!;
    expect([s.kind, s.number, s.remaining, s.remaining_amount]).toEqual(['loan', 3, 2, 100_000]);
    expect(s.projected.map((p) => p.date)).toEqual(['2026-10-05', '2026-11-05']);
    const after = await call<DetectedInstallments>('installments.ignore', { key: s.key });
    expect(after.series.some((x) => x.key === s.key)).toBe(false);
  });
});
