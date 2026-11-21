/** Limit, statement dates and annual fee in the mock (V156): same rules as the core. */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { CardOverview } from '@/api/generated/CardOverview';
import type { StatementView } from '@/api/generated/StatementView';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall, resetMock } from '@/api/mock';

import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday('2026-10-20');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock: limit and annual fee', () => {
  it('limit usage, best day, history and annual fee check', async () => {
    const card = await call<AccountView>('accounts.create', {
      name: 'Cartão Teste',
      kind: 'credit_card',
      opening_date: '2026-01-01',
      card: { closing_day: 3, due_day: 10, credit_limit: 100_000 },
    });
    const buy = (date: string, amount: number, description: string) =>
      call('transactions.create', { account_id: card.id, date, amount: -amount, description });
    await buy('2026-10-10', 30_000, 'Mercado');
    await buy('2026-10-15', 5_000, 'ANUIDADE 10/12');
    await buy('2026-10-16', 8_000, 'ANUIDADE 11/12');
    await call('cards.set_annual_fee', {
      account_id: card.id,
      amount: 60_000,
      installments: 12,
      anniversary_month: 12,
      waiver_kind: 'spend',
      waiver_threshold: 100_000,
      starts_on: '2026-01-01',
    });
    const o = await call<CardOverview>('cards.overview', { account_id: card.id });
    expect(o.used).toBe(43_000);
    expect(o.utilization_bp).toBe(4_300);
    expect([o.best_purchase_date, o.best_purchase_due_date]).toEqual(['2026-11-04', '2026-12-10']);
    expect(o.limit_history.map((c) => [c.date, c.credit_limit])).toEqual([['2026-01-01', 100_000]]);
    const fee = o.annual_fee!;
    expect(fee.paid_12m).toBe(13_000);
    expect(fee.charges.map((c) => c.issue)).toEqual([null, 'amount_differs']);
    expect([fee.waiver.current, fee.waiver.remaining]).toEqual([30_000, 70_000]);
    expect(fee.next_renewal).toBe('2026-12-01');

    const after = await call<CardOverview>('cards.add_limit_change', {
      account_id: card.id,
      date: '2026-10-18',
      credit_limit: 200_000,
    });
    expect(after.credit_limit).toBe(200_000);

    const [st] = await call<StatementView[]>('cards.list_statements', { account_id: card.id });
    const edited = await call<StatementView>('cards.update_statement', {
      id: st!.id,
      closing_date: '2026-11-04',
      due_date: '2026-11-11',
      informed_minimum: 6_000,
    });
    expect([edited.due_date, edited.dates_source, edited.informed_minimum]).toEqual([
      '2026-11-11',
      'manual',
      6_000,
    ]);
  });

  it('revolving credit and purchase abroad', async () => {
    const bank = await call<AccountView>('accounts.create', {
      name: 'Conta Teste',
      kind: 'checking',
      opening_date: '2026-01-01',
    });
    const card = await call<AccountView>('accounts.create', {
      name: 'Cartão Teste',
      kind: 'credit_card',
      opening_date: '2026-01-01',
      card: { closing_day: 3, due_day: 10, credit_limit: 500_000 },
    });
    const abroad = await call<TxnView>('transactions.create', {
      account_id: card.id,
      date: '2026-09-15',
      amount: -100_000,
      description: 'LOJA EXTERIOR',
      orig_amount: 20_000,
      orig_currency: 'usd',
    });
    expect([abroad.orig_amount, abroad.orig_currency]).toEqual([-20_000, 'USD']);
    const [oct] = await call<StatementView[]>('cards.list_statements', { account_id: card.id });
    await call('cards.pay_statement', {
      statement_id: oct!.id,
      from_account_id: bank.id,
      amount: 30_000,
      date: '2026-10-10',
    });
    await call('transactions.create', {
      account_id: card.id,
      date: '2026-10-12',
      amount: -20_000,
      description: 'JUROS DO ROTATIVO',
    });
    const o = await call<CardOverview>('cards.overview', { account_id: card.id });
    expect(o.revolving).toMatchObject({ unpaid: 70_000, charges: 20_000, over_cap: false });
  });
});
