import { beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { ChangeSetDetail } from '@/api/generated/ChangeSetDetail';
import type { ChangeSetSummary } from '@/api/generated/ChangeSetSummary';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { RecurrencePreview } from '@/api/generated/RecurrencePreview';
import type { StatementView } from '@/api/generated/StatementView';
import type { TxnPage } from '@/api/generated/TxnPage';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall, resetMock } from '@/api/mock';
import { todayIso } from '@/lib/format';

const call = <T>(cmd: string, args?: unknown) => mockCall(cmd, args) as Promise<T>;

async function accounts() {
  return call<AccountView[]>('accounts.list', {});
}

describe('livro-caixa do mock', () => {
  beforeEach(() => resetMock('unlocked'));

  it('semeia a família de exemplo com seis contas e meses de lançamentos', async () => {
    const list = await accounts();
    expect(new Set(list.map((a) => a.kind))).toEqual(
      new Set(['checking', 'savings', 'cash', 'credit_card', 'foreign', 'brokerage']),
    );
    // Mesmos ids do razão dos relatórios e da carteira.
    expect(list.map((a) => a.id)).toEqual(
      expect.arrayContaining([
        'acc-itau',
        'acc-nucard',
        'acc-wise',
        'acc-xp',
        'acc-inter',
        'acc-avenue',
      ]),
    );
    const usd = list.find((a) => a.currency === 'USD')!;
    expect(usd.balance_base).not.toBeNull();
    const card = list.find((a) => a.kind === 'credit_card')!;
    expect(card.available_credit).not.toBeNull();
    expect(card.balance).toBeLessThanOrEqual(0);
    const page = await call<TxnPage>('transactions.query', { limit: 5 });
    expect(page.total).toBeGreaterThan(150);
    expect(page.items[0]!.date >= page.items[4]!.date).toBe(true);
    const planned = await call<TxnPage>('transactions.query', { statuses: ['planned'], limit: 1 });
    expect(planned.total).toBeGreaterThan(0);
    const uncategorized = await call<TxnPage>('transactions.query', {
      uncategorized: true,
      limit: 1,
    });
    expect(uncategorized.total).toBeGreaterThan(0);
  });

  it('calcula faturas do cartão e paga as fechadas', async () => {
    const card = (await accounts()).find((a) => a.kind === 'credit_card')!;
    const statements = await call<StatementView[]>('cards.list_statements', {
      account_id: card.id,
    });
    expect(statements.length).toBeGreaterThan(4);
    const paid = statements.filter((s) => s.status === 'paid');
    expect(paid.length).toBeGreaterThan(2);
    expect(statements.some((s) => s.informed_total != null)).toBe(true);
  });

  it('cria, divide e desfaz um lançamento', async () => {
    const checking = (await accounts())[0]!;
    const before = checking.balance;
    const t = await call<TxnView>('transactions.create', {
      account_id: checking.id,
      date: todayIso(),
      amount: -10_000,
      description: 'Teste',
    });
    expect((await accounts())[0]!.balance).toBe(before - 10_000);
    await expect(
      call('transactions.update', { id: t.id, splits: [{ amount: -4_000 }, { amount: -5_000 }] }),
    ).rejects.toMatchObject({
      code: 'invalid',
      message: expect.stringContaining('difere do total') as unknown,
    });
    const changes = await call<ChangeSetSummary[]>('changes.list', { limit: 1 });
    expect(changes[0]!.reason).toBe('Lançamento criado: Teste');
    const detail = await call<ChangeSetDetail>('changes.get', { id: changes[0]!.id });
    expect(detail.items[0]!.table_name).toBe('txn');
    await call('changes.undo', { id: changes[0]!.id });
    expect((await accounts())[0]!.balance).toBe(before);
    await expect(call('changes.undo', { id: changes[0]!.id })).rejects.toMatchObject({
      code: 'conflict',
    });
  });

  it('recusa desfazer quando alterações posteriores tocaram nas mesmas linhas', async () => {
    const checking = (await accounts())[0]!;
    const t = await call<TxnView>('transactions.create', {
      account_id: checking.id,
      date: todayIso(),
      amount: -500,
      description: 'Café',
    });
    const [created] = await call<ChangeSetSummary[]>('changes.list', { limit: 1 });
    await call('transactions.update', { id: t.id, description: 'Café da manhã' });
    await expect(call('changes.undo', { id: created!.id })).rejects.toMatchObject({
      code: 'conflict',
    });
  });

  it('descreve recorrências em português', async () => {
    const p = await call<RecurrencePreview>('recurrences.preview', {
      spec: { freq: 'WEEKLY', interval: 2, by_day: ['MO'] },
      start_date: '2026-10-05',
      count: 3,
    });
    expect(p.human).toBe('a cada 2 semanas às segundas');
    expect(p.dates).toEqual(['2026-10-05', '2026-10-19', '2026-11-02']);
    const list = await call<Recurrence[]>('recurrences.list', {});
    expect(list.find((r) => r.description.startsWith('Aluguel'))!.human).toBe('todo mês no dia 10');
  });
});
