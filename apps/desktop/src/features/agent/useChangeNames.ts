/**
 * Nomes das entidades citadas num conjunto de alterações, a partir das listas que o app
 * já guarda em cache (mesmas chaves das telas): categorias, contas, membros,
 * instituições, ativos, contratos, etiquetas, regras, metas, recorrências e bens. Só busca
 * as listas que as alterações citam; lançamentos citados só pelo id vêm um a um
 * (`transactions.get`, até 60).
 */
import { useQueries, useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';

import { call } from '@/api/client';
import type { AccountView } from '@/api/generated/AccountView';
import type { Category } from '@/api/generated/Category';
import type { ChangeItem } from '@/api/generated/ChangeItem';
import type { GoalView } from '@/api/generated/GoalView';
import type { Institution } from '@/api/generated/Institution';
import type { LoanView } from '@/api/generated/LoanView';
import type { Member } from '@/api/generated/Member';
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { Recurrence } from '@/api/generated/Recurrence';
import type { Rule } from '@/api/generated/Rule';
import type { TagSummary } from '@/api/generated/TagSummary';
import type { TxnView } from '@/api/generated/TxnView';
import { useAssets } from '@/api/hooks/assets';
import { coreKey, useCore } from '@/api/hooks/coreQuery';
import { goalKeys } from '@/api/hooks/goals';
import { baseCurrency } from '@/lib/format';

import {
  namesFromItems,
  referencedIds,
  type NameBook,
  type RefKind,
  type TxnBrief,
} from './changeText';

const MAX_TXN_LOOKUPS = 60;

/** Estável fora do componente: o TanStack só refaz a lista quando os dados mudam. */
const txnResults = (results: { data?: TxnView }[]) => results.map((r) => r.data);

export function useChangeNames(items: readonly ChangeItem[]): NameBook {
  const refs = useMemo(() => referencedIds(items), [items]);
  const kinds = useMemo(() => new Set(refs.map((r) => r.kind)), [refs]);
  const tables = useMemo(() => new Set(items.map((i) => i.table_name)), [items]);
  const wants = (kind: RefKind, table = kind as string) => kinds.has(kind) || tables.has(table);

  const categories = useCore<Category[]>(
    'categories',
    'list',
    { include_archived: true },
    { enabled: wants('category') },
  );
  const accounts = useCore<AccountView[]>(
    'accounts',
    'list',
    { include_archived: true },
    { enabled: wants('account') },
  );
  const members = useCore<Member[]>(
    'household',
    'list_members',
    { include_archived: true },
    { enabled: wants('member') },
  );
  const institutions = useCore<Institution[]>('household', 'list_institutions', undefined, {
    enabled: wants('institution'),
  });
  const assets = useAssets({ include_archived: true }, { enabled: wants('asset') });
  const loans = useCore<LoanView[]>(
    'debt',
    'list',
    { include_archived: true },
    { enabled: wants('loan') },
  );
  const tags = useCore<TagSummary[]>('transactions', 'list_tags', undefined, {
    enabled: wants('tag'),
  });
  const rules = useCore<Rule[]>('rules', 'list', undefined, { enabled: wants('rule') });
  const goals = useQuery({
    queryKey: goalKeys.list({}),
    queryFn: () => call<GoalView[]>('goals.list', {}),
    enabled: wants('goal'),
  });
  const recurrences = useCore<Recurrence[]>(
    'recurrences',
    'list',
    {},
    {
      enabled: wants('recurrence'),
    },
  );
  const physical = useCore<PhysicalAssetView[]>(
    'physical',
    'list',
    {},
    {
      enabled: wants('physical', 'physical_asset'),
    },
  );

  // Lançamentos citados só pelo id (alteração de categoria, partes, etiquetas). Os que a
  // própria lista já descreve (inclusões e exclusões trazem a linha inteira) não precisam.
  const txnIds = useMemo(() => {
    const described = new Set(
      items
        .filter((i) => i.table_name === 'txn' && (i.after ?? i.before)?.description !== undefined)
        .map((i) => i.row_id),
    );
    return refs
      .filter((r) => r.kind === 'txn' && !described.has(r.id))
      .map((r) => r.id)
      .slice(0, MAX_TXN_LOOKUPS);
  }, [items, refs]);
  const txnData = useQueries({
    queries: txnIds.map((id) => ({
      queryKey: coreKey('transactions', 'get', { id }),
      queryFn: () => call<TxnView>('transactions.get', { id }),
      retry: false,
      staleTime: 60_000,
    })),
    combine: txnResults,
  });

  return useMemo<NameBook>(() => {
    const names = namesFromItems(items);
    const put = (kind: RefKind, id: string, label: string | null | undefined) => {
      if (label) names.set(`${kind}:${id}`, label);
    };
    for (const c of categories.data ?? []) put('category', c.id, c.path);
    for (const a of accounts.data ?? []) put('account', a.id, a.name);
    for (const m of members.data ?? []) put('member', m.id, m.name);
    for (const i of institutions.data ?? []) put('institution', i.id, i.name);
    for (const a of assets.data ?? []) put('asset', a.id, a.ticker ? a.ticker : a.name);
    for (const l of loans.data ?? []) put('loan', l.id, l.name);
    for (const t of tags.data ?? []) put('tag', t.id, `#${t.name}`);
    for (const r of rules.data ?? []) put('rule', r.id, r.name);
    for (const g of goals.data ?? []) put('goal', g.id, g.name);
    for (const r of recurrences.data ?? []) put('recurrence', r.id, r.description);
    for (const p of physical.data ?? []) put('physical', p.id, p.name);

    const briefs = new Map<string, TxnBrief>();
    for (const it of items) {
      const row = it.after ?? it.before;
      if (it.table_name !== 'txn' || !row || typeof row.description !== 'string') continue;
      briefs.set(it.row_id, {
        description: row.description,
        date: typeof row.date === 'string' ? row.date : '',
        amount: typeof row.amount === 'number' ? row.amount : 0,
        currency: typeof row.currency === 'string' ? row.currency : baseCurrency(),
      });
    }
    for (const t of txnData) {
      if (t)
        briefs.set(t.id, {
          description: t.description,
          date: t.date,
          amount: t.amount,
          currency: t.currency,
        });
    }
    return {
      name: (kind, id) => names.get(`${kind}:${id}`),
      txn: (id) => briefs.get(id),
    };
  }, [
    items,
    categories.data,
    accounts.data,
    members.data,
    institutions.data,
    assets.data,
    loans.data,
    tags.data,
    rules.data,
    goals.data,
    recurrences.data,
    physical.data,
    txnData,
  ]);
}
