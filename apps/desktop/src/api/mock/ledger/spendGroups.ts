/**
 * Grupos de gasto do mock (`reports.spend_groups` e as metas `budget.group_targets`), como o
 * núcleo: despesas pelo grupo da categoria (sem categoria = não essencial), Futuro também com
 * os aportes (perna de investimento que sai da conta para a corretora) e as transferências para
 * contas de metas; resgates não abatem. Transferências comuns, faturas e ajustes ficam fora.
 */
import type { Basis } from '@/api/generated/Basis';
import type { GroupTargets } from '@/api/generated/GroupTargets';
import type { SpendGroup } from '@/api/generated/SpendGroup';
import type { SpendGroupAmount } from '@/api/generated/SpendGroupAmount';
import type { SpendGroupCategory } from '@/api/generated/SpendGroupCategory';
import type { SpendGroupMonth } from '@/api/generated/SpendGroupMonth';
import type { SpendGroupsReport } from '@/api/generated/SpendGroupsReport';
import { monthBounds, monthsBetween } from '@/lib/months';

import { flowKind, flowParts, Fx, soleOwner } from './engine';
import { BASE, category } from './fixture';
import { invalid } from './errors';
import { goalsList } from './goalsState';
import { getLedger, type LedgerTxn } from './ledger';
import { investmentLegs } from './reports';

export const SPEND_GROUP_ORDER: readonly SpendGroup[] = [
  'essential',
  'debt',
  'future',
  'non_essential',
];

/** O guia 50/30/20: dívida acima do mínimo sai do Futuro, por isso Dívidas fica sem meta. */
export const DEFAULT_GROUP_TARGETS: GroupTargets = {
  essential: 0.5,
  debt: null,
  future: 0.2,
  non_essential: 0.3,
};

let targets: GroupTargets = { ...DEFAULT_GROUP_TARGETS };

export function resetGroupTargets(): void {
  targets = { ...DEFAULT_GROUP_TARGETS };
}

export function groupTargets(): GroupTargets {
  return { ...targets };
}

/** Como `budget::set_group_targets`: cada meta é uma fração de 0 a 1, ou `null` (sem meta). */
export function setGroupTargets(input: Partial<Record<SpendGroup, unknown>>): GroupTargets {
  const next = { ...targets };
  for (const g of SPEND_GROUP_ORDER) {
    if (!(g in input)) continue;
    const v = input[g];
    if (v === null || v === undefined) next[g] = null;
    else if (typeof v !== 'number' || !Number.isFinite(v) || v < 0 || v > 1)
      invalid(`meta de ${g} deve ficar entre 0 e 1 (fração da renda)`);
    else next[g] = Math.round(v * 10_000) / 10_000;
  }
  targets = next;
  return groupTargets();
}

const NO_CATEGORY = 'Sem categoria';
const CONTRIBUTIONS = 'Aportes';
const GOAL_TRANSFERS = 'Transferências para metas';

interface Hit {
  month: string;
  group: SpendGroup;
  /** Categoria (ou a chave do item sem categoria). */
  key: string;
  categoryId: string | null;
  label: string;
  amount: number;
}

/** Entradas em contas das metas ativas vindas de contas fora delas (a outra perna fica fora). */
function goalTransfers(from: string, to: string): LedgerTxn[] {
  const goalAccounts = new Set(goalsList(false).flatMap((g) => g.account_ids));
  if (!goalAccounts.size) return [];
  const { txns } = getLedger();
  const otherLeg = (t: LedgerTxn) =>
    txns.find(
      (o) =>
        o.id !== t.id &&
        o.kind === 'transfer' &&
        (t.transfer_id
          ? o.transfer_id === t.transfer_id
          : Boolean(t.recurrence_id) &&
            o.recurrence_id === t.recurrence_id &&
            o.date === t.date &&
            o.amount === -t.amount),
    );
  return txns.filter((t) => {
    if (t.kind !== 'transfer' || t.status === 'planned' || t.amount <= 0) return false;
    if (t.date < from || t.date > to || !goalAccounts.has(t.account_id)) return false;
    const other = otherLeg(t);
    return !other || !goalAccounts.has(other.account_id);
  });
}

function emptyMonth(month: string): SpendGroupMonth {
  return {
    month,
    income: 0,
    spending: 0,
    groups: SPEND_GROUP_ORDER.map((group) => ({
      group,
      amount: 0,
      share_of_income: null,
      top: [],
    })),
  };
}

function summarize(month: string, income: number, hits: readonly Hit[]): SpendGroupMonth {
  const out = emptyMonth(month);
  out.income = income;
  out.groups = SPEND_GROUP_ORDER.map((group): SpendGroupAmount => {
    const mine = hits.filter((h) => h.group === group);
    const byKey = new Map<string, SpendGroupCategory>();
    for (const h of mine) {
      const cur = byKey.get(h.key);
      if (cur) cur.amount += h.amount;
      else byKey.set(h.key, { category_id: h.categoryId, label: h.label, amount: h.amount });
    }
    const amount = Math.max(
      0,
      mine.reduce((s, h) => s + h.amount, 0),
    );
    return {
      group,
      amount,
      share_of_income: income > 0 ? amount / income : null,
      top: [...byKey.values()]
        .filter((c) => c.amount > 0)
        .sort((a, b) => b.amount - a.amount || a.label.localeCompare(b.label))
        .slice(0, 3),
    };
  });
  out.spending = out.groups.reduce((s, g) => s + g.amount, 0);
  return out;
}

export function spendGroups(
  fromMonth: string,
  toMonth: string,
  basis: Basis,
  memberId: string | null,
): SpendGroupsReport {
  const months = monthsBetween(fromMonth, toMonth);
  if (!months.length) invalid('o primeiro mês deve vir antes do último');
  const from = monthBounds(months[0]!).from;
  const to = monthBounds(months[months.length - 1]!).to;
  const fx = new Fx();
  const income = new Map<string, number>();
  const hits: Hit[] = [];
  const ownedBy = (t: LedgerTxn) =>
    !memberId || (t.member_id ?? soleOwner(t.account_id)) === memberId;

  for (const p of flowParts({ from, to, basis, member_ids: memberId ? [memberId] : null })) {
    const c = fx.convert(p.amount, p.txn.currency, p.txn.date);
    if (!c) continue;
    const month = p.eff.slice(0, 7);
    if (flowKind(p.txn) === 'income') {
      income.set(month, (income.get(month) ?? 0) + c.amount);
      continue;
    }
    const cat = category(p.category_id);
    hits.push({
      month,
      group: cat?.spend_group ?? 'non_essential',
      key: p.category_id ?? '\u0000sem-categoria',
      categoryId: p.category_id,
      label: cat?.path ?? NO_CATEGORY,
      amount: -c.amount,
    });
  }
  for (const leg of investmentLegs(from, to)) {
    if (!leg.out || !ownedBy(leg.txn)) continue;
    const c = fx.convert(leg.txn.amount, leg.txn.currency, leg.txn.date);
    if (!c) continue;
    hits.push({
      month: leg.txn.date.slice(0, 7),
      group: 'future',
      key: '\u0000aportes',
      categoryId: null,
      label: CONTRIBUTIONS,
      amount: -c.amount,
    });
  }
  for (const t of goalTransfers(from, to)) {
    if (!ownedBy(t)) continue;
    const c = fx.convert(t.amount, t.currency, t.date);
    if (!c) continue;
    hits.push({
      month: t.date.slice(0, 7),
      group: 'future',
      key: '\u0000metas',
      categoryId: null,
      label: GOAL_TRANSFERS,
      amount: c.amount,
    });
  }

  const totalIncome = [...income.values()].reduce((s, v) => s + v, 0);
  return {
    currency: BASE,
    months: months.map((m) =>
      summarize(
        m,
        income.get(m) ?? 0,
        hits.filter((h) => h.month === m),
      ),
    ),
    total: summarize('total', totalIncome, hits),
    targets: groupTargets(),
  };
}
