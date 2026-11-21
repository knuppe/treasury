/** Mock de `transactions.suggest`: agrupa a história do mock pela descrição sem acento. */
import type { TxnPage } from '../generated/TxnPage';
import type { TxnSuggestion } from '../generated/TxnSuggestion';
import { registerMockHandlers } from '../mockRegistry';

const fold = (s: string) =>
  s
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .trim();

registerMockHandlers({
  'transactions.suggest': async (args): Promise<TxnSuggestion[]> => {
    const text = fold(typeof args.text === 'string' ? args.text : '');
    if (text.length < 2) return [];
    const { mockCall } = await import('../mock');
    const page = (await mockCall('transactions.query', { limit: 2000 })) as TxnPage;
    const words = text.split(' ');
    const groups = new Map<string, TxnSuggestion>();
    for (const t of page.items) {
      if (t.kind !== 'expense' && t.kind !== 'income') continue;
      const key = fold(t.description);
      const kw = key.split(' ');
      if (!words.every((w) => kw.some((k) => k.startsWith(w)))) continue;
      const g = groups.get(key);
      if (g) g.count += 1;
      else
        groups.set(key, {
          description: t.description,
          payee: t.payee,
          category_id: t.category_id,
          category_path: t.category_path,
          account_id: t.account_id,
          kind: t.kind,
          last_amount: t.amount,
          last_date: t.date,
          count: 1,
        });
    }
    return [...groups.values()].sort((a, b) => b.count - a.count).slice(0, 6);
  },
});
