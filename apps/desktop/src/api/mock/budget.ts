/** Mock de `budget.*` sobre o livro-caixa fictício (`./ledger/budgetState.ts`). */
import type { BudgetLineInput } from '@/api/generated/BudgetLineInput';

import {
  budgetCopy,
  budgetGet,
  budgetMonths,
  budgetSetLines,
  budgetSuggest,
} from './ledger/budgetState';
import { basisArg, defaultBasis, monthArg, optNum, register } from './ledger/common';
import { groupTargets, resetGroupTargets, setGroupTargets } from './ledger/spendGroups';
import { onMockReset } from '../mockRegistry';

onMockReset(resetGroupTargets);

register({
  'budget.get': (a) => budgetGet(monthArg(a, 'month'), basisArg(a)),
  'budget.set_lines': (a) => {
    const month = monthArg(a, 'month');
    budgetSetLines(month, (Array.isArray(a.lines) ? a.lines : []) as BudgetLineInput[]);
    return budgetGet(month, defaultBasis());
  },
  'budget.copy': (a) => {
    const to = monthArg(a, 'to_month');
    budgetCopy(monthArg(a, 'from_month'), to, a.overwrite === true);
    return budgetGet(to, defaultBasis());
  },
  'budget.suggest': (a) =>
    budgetSuggest(monthArg(a, 'month'), optNum(a, 'months_back') ?? 3, basisArg(a)),
  'budget.list_months': () => budgetMonths(),
  'budget.group_targets': () => groupTargets(),
  'budget.set_group_targets': (a) => setGroupTargets(a),
});
