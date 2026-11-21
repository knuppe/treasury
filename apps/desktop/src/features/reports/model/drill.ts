/**
 * Drill-down: o filtro da tela 04 · Lançamentos para cada pedaço de gráfico ou linha.
 * Funções puras; a tela navega com `txnLink(filtro)`.
 */
import type { Basis } from '@/api/generated/Basis';
import type { TxnFilter } from '@/api/generated/TxnFilter';
import type { TxnKind } from '@/api/generated/TxnKind';
import { monthBounds } from '@/lib/months';

import type { Period } from './period';

export interface DrillScope {
  period: Period;
  basis?: Basis | null;
  memberIds?: readonly string[] | null;
  accountIds?: readonly string[] | null;
}

function base(scope: DrillScope, kind: TxnKind | null): TxnFilter {
  const f: TxnFilter = { date_from: scope.period.from, date_to: scope.period.to };
  if (scope.basis) f.basis = scope.basis;
  if (kind) f.kinds = [kind];
  if (scope.memberIds?.length) f.member_ids = [...scope.memberIds];
  if (scope.accountIds?.length) f.account_ids = [...scope.accountIds];
  return f;
}

/**
 * Lançamentos de uma ou mais categorias (com subcategorias). `null` na lista = sem
 * categoria; lista vazia = todas.
 */
export function categoryDrill(
  scope: DrillScope,
  categoryIds: readonly (string | null)[],
  kind: TxnKind = 'expense',
): TxnFilter {
  const f = base(scope, kind);
  const ids = categoryIds.filter((c): c is string => c !== null);
  if (categoryIds.length > 0 && ids.length === 0) f.uncategorized = true;
  else if (ids.length > 0) f.category_ids = ids;
  return f;
}

/** Um dia do calendário. */
export function dayDrill(date: string, basis?: Basis | null): TxnFilter {
  return categoryDrill({ period: { from: date, to: date }, basis }, []);
}

/** Um mês (barras da Tendência, linha de receitas). */
export function monthDrill(
  month: string,
  basis: Basis | null | undefined,
  categoryIds: readonly (string | null)[] = [],
  kind: TxnKind = 'expense',
): TxnFilter {
  return categoryDrill({ period: monthBounds(month), basis }, categoryIds, kind);
}

/** Gasto de um titular numa categoria (barras agrupadas). */
export function memberDrill(
  scope: DrillScope,
  memberId: string | null,
  categoryId: string | null | undefined,
): TxnFilter {
  const f = categoryDrill(scope, categoryId === undefined ? [] : [categoryId]);
  if (memberId) f.member_ids = [memberId];
  return f;
}

/** Lançamentos de uma recorrência (pela descrição, na conta dela). */
export function recurrenceDrill(
  description: string,
  accountId: string,
  period?: Period,
): TxnFilter {
  const f: TxnFilter = { text: description, account_ids: [accountId] };
  if (period) {
    f.date_from = period.from;
    f.date_to = period.to;
  }
  return f;
}

/** Fatura de um cartão num mês de vencimento (base caixa = vencimento da fatura). */
export function cardMonthDrill(accountId: string, month: string): TxnFilter {
  const { from, to } = monthBounds(month);
  return { account_ids: [accountId], date_from: from, date_to: to, basis: 'cash' };
}

/** Lançamentos sem categoria do período. */
export function uncategorizedDrill(period: Period, basis?: Basis | null): TxnFilter {
  const f: TxnFilter = { date_from: period.from, date_to: period.to, uncategorized: true };
  if (basis) f.basis = basis;
  return f;
}

/** Um vencimento do painel: fatura (pelo id) ou previsto (conta e data). */
export function upcomingDrill(item: {
  kind: 'planned_txn' | 'card_due';
  ref_id: string;
  account_id: string;
  date: string;
}): TxnFilter {
  if (item.kind === 'card_due') return { statement_id: item.ref_id };
  return {
    account_ids: [item.account_id],
    date_from: item.date,
    date_to: item.date,
    statuses: ['planned'],
  };
}
