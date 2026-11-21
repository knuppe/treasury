/** Cartões adicionais no mock (V155): mesmas regras do núcleo (`cards/tests_plastics.rs`). */
import { afterAll, beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { CardPlastic } from '@/api/generated/CardPlastic';
import type { Member } from '@/api/generated/Member';
import type { StatementDetail } from '@/api/generated/StatementDetail';
import type { StatementView } from '@/api/generated/StatementView';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall, resetMock } from '@/api/mock';

import { setMockToday } from './ledger';

const call = <T>(cmd: string, args: object = {}) => mockCall(cmd, args) as Promise<T>;

beforeEach(() => {
  setMockToday('2026-10-14');
  resetMock('unlocked');
});
afterAll(() => setMockToday(null));

describe('mock: cartões adicionais', () => {
  it('a compra herda o membro do cartão usado e a fatura soma por cartão', async () => {
    const members = await call<Member[]>('household.list_members');
    const [ana, beto] = [members[0]!, members[1]!];
    const card = await call<AccountView>('accounts.create', {
      name: 'Cartão Teste',
      kind: 'credit_card',
      opening_date: '2026-01-01',
      member_ids: [ana.id],
      card: { closing_day: 3, due_day: 10, last4: '1111' },
    });
    const list = await call<CardPlastic[]>('cards.list_plastics', { account_id: card.id });
    expect(list.map((p) => [p.kind, p.last4])).toEqual([['holder', '1111']]);
    const extra = await call<CardPlastic>('cards.create_plastic', {
      account_id: card.id,
      kind: 'additional',
      last4: '**** 2222',
      member_id: beto.id,
    });
    expect(extra.last4).toBe('2222');
    await expect(
      call('cards.create_plastic', {
        account_id: card.id,
        kind: 'virtual',
        last4: '4111111111111111',
      }),
    ).rejects.toMatchObject({ code: 'invalid' });

    const buy = (description: string, plastic?: string) =>
      call<TxnView>('transactions.create', {
        account_id: card.id,
        date: '2026-10-10',
        amount: -1_000,
        description,
        ...(plastic ? { card_plastic_id: plastic } : {}),
      });
    const mine = await buy('Padaria', list[0]!.id);
    const his = await buy('Farmácia', extra.id);
    expect(mine.member_id).toBe(ana.id);
    expect(his.member_id).toBe(beto.id);
    expect(his.card_last4).toBe('2222');

    const [st] = await call<StatementView[]>('cards.list_statements', { account_id: card.id });
    const detail = await call<StatementDetail>('cards.get_statement', { id: st!.id });
    expect(detail.by_plastic.map((s) => [s.last4, s.total])).toEqual([
      ['1111', 1_000],
      ['2222', 1_000],
    ]);

    // Apagar o adicional: a compra volta para a titular da conta.
    await call('cards.delete_plastic', { id: extra.id });
    const after = await call<TxnView>('transactions.get', { id: his.id });
    expect([after.member_id, after.card_plastic_id]).toEqual([ana.id, null]);
  });
});
