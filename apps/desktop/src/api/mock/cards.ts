/** Mock de `cards.*`: faturas, pagamento, parcelamentos e compromissos futuros. */
import type { InstallmentPlan } from '@/api/generated/InstallmentPlan';
import type { MonthAmount } from '@/api/generated/MonthAmount';
import type { StatementView } from '@/api/generated/StatementView';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import {
  addMonths,
  brDate,
  firstOfMonth,
  monthOf,
  nowUtc,
  parseMonth,
  parseYmd,
  today,
} from './ledger/dates';
import {
  baseCurrency,
  checkCategory,
  convert,
  currencyAndKind,
  deleteRows,
  ensureStatement,
  insertTxn,
  replaceTags,
  requireCard,
  requireMember,
  sortTxns,
  statementView,
  txnRow,
  txnView,
  txnViews,
} from './ledger/domain';
import { cleanOpt, conflict, invalid, notFound, str, text } from './ledger/errors';
import {
  addLimitChange,
  cardOverview,
  deleteLimitChange,
  endAnnualFee,
  setAnnualFee,
  updateStatement,
} from './ledger/cardOverview';
import { endRewardProgram, rewardsView, setRewardProgram } from './ledger/rewards';
import { cardValueReport } from './ledger/cardValue';
import {
  addLoyaltyEntry,
  createLoyalty,
  deleteLoyalty,
  deleteLoyaltyEntry,
  loyaltyEntries,
  loyaltyList,
  loyaltyView,
  transferLoyalty,
  updateLoyalty,
} from './ledger/loyalty';
import {
  createPlastic,
  deletePlastic,
  listPlastics,
  plasticSubtotals,
  plasticView,
  updatePlastic,
} from './ledger/plastics';
import { type PlanRow, del, find, newId, patch, put, rows, write } from './ledger/store';
import { createTransfer, short } from './transactions';

function statement(id: string) {
  return find('card_statement', id) ?? notFound(`fatura ${id}`);
}

function listStatements(args: Record<string, unknown>): StatementView[] {
  for (const m of [args.from_month, args.to_month]) if (typeof m === 'string') parseMonth(m);
  const sortOf = (accountId: string) => find('account', accountId)?.sort_order ?? 0;
  return rows('card_statement')
    .filter((s) => !args.account_id || s.account_id === args.account_id)
    .filter((s) => typeof args.from_month !== 'string' || s.reference_month >= args.from_month)
    .filter((s) => typeof args.to_month !== 'string' || s.reference_month <= args.to_month)
    .sort(
      (a, b) =>
        a.reference_month.localeCompare(b.reference_month) ||
        sortOf(a.account_id) - sortOf(b.account_id),
    )
    .map((s) => statementView(s));
}

function planView(p: PlanRow, T = today()): InstallmentPlan {
  const parts = rows('txn').filter((t) => t.installment_plan_id === p.id);
  return {
    id: p.id,
    account_id: p.account_id,
    description: p.description,
    total_amount: p.total_amount,
    currency: p.currency,
    installments: p.installments,
    purchase_date: p.purchase_date,
    category_id: p.category_id,
    member_id: p.member_id,
    paid_installments: parts.filter((t) => t.date <= T).length,
    remaining_amount: parts.filter((t) => t.date > T).reduce((s, t) => s - t.amount, 0),
  };
}

function listPlans(accountId?: string | null): InstallmentPlan[] {
  return rows('installment_plan')
    .filter((p) => !accountId || p.account_id === accountId)
    .sort((a, b) => b.purchase_date.localeCompare(a.purchase_date) || b.id.localeCompare(a.id))
    .map((p) => planView(p));
}

function createInstallments(args: Record<string, unknown>): string {
  const accountId = str(args, 'account_id');
  requireCard(accountId);
  const { currency } = currencyAndKind(accountId);
  const n = Number(args.installments);
  if (!(Number.isInteger(n) && n >= 1 && n <= 120))
    invalid('o número de parcelas deve ficar entre 1 e 120');
  const total = Number(args.total_amount);
  if (!(total > 0)) invalid('informe o valor total da compra (positivo)');
  if (total < n) invalid('o valor total é menor que o número de parcelas');
  const purchase = str(args, 'purchase_date');
  parseYmd(purchase);
  const description = text(args.description).trim();
  if (!description) invalid('informe a descrição da compra');
  const categoryId = cleanOpt(args.category_id);
  checkCategory(categoryId, 'expense');
  const memberId = cleanOpt(args.member_id);
  if (memberId) requireMember(memberId);
  const now = nowUtc();
  const T = today();
  const plan = put('installment_plan', {
    id: newId(),
    account_id: accountId,
    description,
    total_amount: total,
    currency,
    installments: n,
    purchase_date: purchase,
    category_id: categoryId,
    member_id: memberId,
    created_at: now,
    updated_at: now,
  });
  const base = Math.floor(total / n);
  const rest = total - base * n;
  for (let i = 1; i <= n; i++) {
    const date = addMonths(purchase, i - 1);
    const t = insertTxn({
      account_id: accountId,
      date,
      amount: -(base + (i === 1 ? rest : 0)),
      currency,
      kind: 'expense',
      description: `${description} ${i}/${n}`,
      status: date > T ? 'planned' : 'cleared',
      category_id: categoryId,
      member_id: memberId,
      installment_plan_id: plan.id,
      installment_number: i,
    });
    if (Array.isArray(args.tags)) replaceTags(t.id, args.tags.map(String));
  }
  return plan.id;
}

function futureCommitments(accountId: string | null, months: number): MonthAmount[] {
  const n = Math.min(Math.max(months, 1), 120);
  const first = firstOfMonth(today());
  const keys = Array.from({ length: n }, (_, i) => monthOf(addMonths(first, i)));
  const out = new Map(keys.map((k) => [k, 0]));
  const base = baseCurrency();
  for (const v of listStatements({
    account_id: accountId,
    from_month: keys[0],
    to_month: keys.at(-1),
  })) {
    if (v.remaining <= 0) continue;
    const amount =
      v.currency === base ? v.remaining : convert(v.remaining, v.currency, base, today())?.amount;
    if (amount != null && out.has(v.reference_month))
      out.set(v.reference_month, out.get(v.reference_month)! + amount);
  }
  return [...out.entries()].map(([month, amount]) => ({ month, amount }));
}

registerMockHandlers(
  guarded({
    'cards.list_statements': listStatements,
    'cards.get_statement': (args) => {
      const s = statement(str(args, 'id'));
      const txns = sortTxns(
        rows('txn').filter((t) => t.statement_id === s.id),
        'date_asc',
      );
      return {
        statement: statementView(s),
        transactions: txnViews(txns),
        by_plastic: plasticSubtotals(s.id),
      };
    },
    'cards.list_plastics': (args) => listPlastics(cleanOpt(args.account_id)),
    'cards.overview': (args) => cardOverview(str(args, 'account_id')),
    'cards.rewards': (args) => rewardsView(str(args, 'account_id')),
    'cards.loyalty_list': () => loyaltyList(),
    'cards.value': () => cardValueReport(),
    'cards.loyalty_entries': (args) => loyaltyEntries(str(args, 'program_id')),
    'cards.loyalty_create': (args) => {
      const id = write(`Programa de pontos: ${text(args.name)}`, () => createLoyalty(args)).value;
      return loyaltyView(id);
    },
    'cards.loyalty_update': (args) => {
      write('Programa de pontos alterado', () => updateLoyalty(args));
      return loyaltyView(str(args, 'id'));
    },
    'cards.loyalty_delete': (args) => {
      write('Programa de pontos apagado', () => deleteLoyalty(str(args, 'id')));
      return null;
    },
    'cards.loyalty_add_entry': (args) => {
      write('Pontos', () => addLoyaltyEntry(args));
      return loyaltyView(str(args, 'program_id'));
    },
    'cards.loyalty_transfer': (args) => {
      write('Transferência de pontos', () => transferLoyalty(args));
      return loyaltyList();
    },
    'cards.loyalty_delete_entry': (args) => {
      const rec = write('Movimento de pontos apagado', () => deleteLoyaltyEntry(str(args, 'id')));
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'cards.set_reward_program': (args) => {
      write('Cashback e pontos do cartão', () => setRewardProgram(args));
      return rewardsView(str(args, 'account_id'));
    },
    'cards.end_reward_program': (args) => {
      write('Cartão sem programa de recompensa', () => endRewardProgram(str(args, 'account_id')));
      return rewardsView(str(args, 'account_id'));
    },
    'cards.update_statement': (args) => {
      const id = str(args, 'id');
      write('Fatura alterada', () => updateStatement(args));
      return statementView(statement(id));
    },
    'cards.add_limit_change': (args) => {
      write('Limite do cartão', () => addLimitChange(args));
      return cardOverview(str(args, 'account_id'));
    },
    'cards.delete_limit_change': (args) => {
      write('Limite do cartão', () => deleteLimitChange(str(args, 'id')));
      return null;
    },
    'cards.set_annual_fee': (args) => {
      write('Anuidade do cartão', () => setAnnualFee(args));
      return cardOverview(str(args, 'account_id'));
    },
    'cards.end_annual_fee': (args) => {
      write('Cartão sem anuidade', () => endAnnualFee(str(args, 'account_id')));
      return cardOverview(str(args, 'account_id'));
    },
    'cards.create_plastic': (args) => {
      const name = find('account', str(args, 'account_id'))?.name ?? '';
      const id = write(`Cartão adicional em ${name}`, () => createPlastic(args)).value;
      return plasticView(id);
    },
    'cards.update_plastic': (args) => {
      const id = str(args, 'id');
      write('Alteração no cartão', () => updatePlastic(args));
      return plasticView(id);
    },
    'cards.delete_plastic': (args) => {
      const rec = write('Remoção do cartão', () => deletePlastic(str(args, 'id')));
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'cards.pay_statement': (args) => {
      const s = statement(str(args, 'statement_id'));
      const view = statementView(s);
      const reason = `Pagamento da fatura ${view.account_name} com vencimento em ${brDate(view.due_date)}`;
      const ids = write(reason, () => {
        const card = requireCard(s.account_id);
        const from =
          cleanOpt(args.from_account_id) ??
          card.payment_account_id ??
          invalid('informe a conta que paga a fatura');
        const payer = currencyAndKind(from);
        if (payer.kind === 'credit_card')
          invalid('a fatura deve ser paga a partir de uma conta que não seja cartão');
        if (payer.currency !== view.currency) {
          invalid(
            `a conta pagadora está em ${payer.currency} e o cartão em ${view.currency}; use uma conta na moeda do cartão`,
          );
        }
        const amount =
          typeof args.amount === 'number'
            ? args.amount > 0
              ? args.amount
              : invalid('o valor do pagamento deve ser positivo')
            : view.remaining > 0
              ? view.remaining
              : invalid('esta fatura não tem valor a pagar');
        const date = cleanOpt(args.date) ?? view.due_date;
        parseYmd(date);
        const { y, m } = parseMonth(view.reference_month);
        return createTransfer(
          {
            from_account_id: from,
            to_account_id: s.account_id,
            date,
            amount,
            description: `Pagamento da fatura ${view.account_name} (${String(m).padStart(2, '0')}/${y})`,
            kind: 'card_payment',
            status: date > today() ? 'planned' : 'cleared',
          },
          s.id,
        );
      }).value;
      return {
        statement: statementView(statement(s.id)),
        transfer: { transfer_id: ids.transferId, from: txnView(ids.fromId), to: txnView(ids.toId) },
      };
    },
    'cards.set_informed_total': (args) => {
      const s = statement(str(args, 'id'));
      const informed = typeof args.informed_total === 'number' ? args.informed_total : null;
      write('Total informado da fatura', () => {
        if (s.informed_total !== informed)
          patch('card_statement', s.id, { informed_total: informed, updated_at: nowUtc() });
      });
      return statementView(statement(s.id));
    },
    'cards.move_to_statement': (args) => {
      const txnIds = Array.isArray(args.txn_ids) ? args.txn_ids.map(String) : [];
      const rec = write(`${txnIds.length} lançamento(s) movido(s) de fatura`, () => {
        let target: string;
        if (typeof args.statement_id === 'string') target = statement(args.statement_id).id;
        else if (typeof args.reference_month === 'string' && typeof args.account_id === 'string') {
          target = ensureStatement(args.account_id, args.reference_month);
        } else {
          invalid('informe a fatura de destino (statement_id, ou account_id e reference_month)');
        }
        const accountId = statement(target).account_id;
        let n = 0;
        for (const id of new Set(txnIds)) {
          const row = txnRow(id);
          if (row.account_id !== accountId)
            invalid(`o lançamento "${row.description}" não é deste cartão`);
          if (row.statement_id === target) continue;
          patch('txn', id, { statement_id: target, updated_at: nowUtc() });
          n++;
        }
        return n;
      });
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'cards.create_installments': (args) => {
      const reason = `Compra parcelada em ${String(args.installments)}x: ${short(text(args.description))}`;
      const id = write(reason, () => createInstallments(args)).value;
      return planView(find('installment_plan', id)!);
    },
    'cards.list_installment_plans': (args) => listPlans(cleanOpt(args.account_id)),
    'cards.delete_installment_plan': (args) => {
      const plan =
        find('installment_plan', str(args, 'id')) ?? notFound(`parcelamento ${String(args.id)}`);
      const rec = write(`Parcelamento apagado: ${short(plan.description)}`, () => {
        const parts = rows('txn').filter((t) => t.installment_plan_id === plan.id);
        const reconciled = parts.find((t) => t.status === 'reconciled');
        if (reconciled) {
          conflict(
            `a parcela "${reconciled.description}" de ${brDate(reconciled.date)} está conciliada; desconcilie antes de apagar o parcelamento`,
          );
        }
        const n = deleteRows(parts);
        del('installment_plan', plan.id);
        return n;
      });
      return { updated: rec.value, change_set_id: rec.changeSetId };
    },
    'cards.future_commitments': (args) =>
      futureCommitments(
        cleanOpt(args.account_id),
        typeof args.months === 'number' ? args.months : 12,
      ),
  }),
);
