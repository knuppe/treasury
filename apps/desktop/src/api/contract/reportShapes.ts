/**
 * Contrato das respostas que as telas 01 · Painel, 05 · Orçamento e Relatórios leem,
 * escrito à mão a partir dos DTOs Rust (`reports`, `budget`, `goals`…). O mesmo arquivo é
 * usado pelo teste do mock (Vitest) e por `scripts/contract-reports.mjs` contra o núcleo
 * real (Node com remoção de tipos), então não importa nada e só usa sintaxe apagável.
 *
 * Forma: `'string' | 'number' | 'boolean' | 'object'`, com `|null` para anuláveis;
 * `{ $array: forma }`; `{ $enum: [...] }`; `{ $nullable: forma }`; objeto = chaves exatas
 * (faltando ou sobrando é divergência); `{ $partial: {...} }` confere só as chaves dadas.
 */

export type Shape =
  | 'string'
  | 'number'
  | 'boolean'
  | 'object'
  | 'unknown'
  | 'string|null'
  | 'number|null'
  | 'boolean|null'
  | { $array: Shape }
  | { $enum: readonly string[] }
  | { $nullable: Shape }
  | { $partial: { [key: string]: Shape } }
  | { [key: string]: Shape };

const basis: Shape = { $enum: ['accrual', 'cash'] };
const kind: Shape = { $enum: ['income', 'expense'] };
const strings: Shape = { $array: 'string' };
const numbers: Shape = { $array: 'number' };

export const monthFlow: Shape = {
  month: 'string',
  income: 'number',
  expense: 'number',
  net: 'number',
  savings_rate: 'number|null',
  missing_fx: strings,
  estimated: 'boolean',
};

export const accountBalanceItem: Shape = {
  account_id: 'string',
  name: 'string',
  kind: 'string',
  currency: 'string',
  balance: 'number',
  balance_base: 'number|null',
  fx_estimated: 'boolean',
};

export const netWorth: Shape = {
  as_of: 'string',
  base_currency: 'string',
  assets: 'number',
  liabilities: 'number',
  net: 'number',
  investments: 'number',
  physical: 'number',
  debts: 'number',
  accounts: { $array: accountBalanceItem },
  missing_fx: strings,
  estimated: 'boolean',
};

export const netWorthPoint: Shape = {
  month: 'string',
  as_of: 'string',
  assets: 'number',
  liabilities: 'number',
  net: 'number',
  investments: 'number',
  physical: 'number',
  debts: 'number',
  missing_fx: strings,
  estimated: 'boolean',
};

export const groupAmount: Shape = {
  key: 'string|null',
  label: 'string',
  parent_key: 'string|null',
  path: 'string|null',
  amount: 'number',
  share: 'number',
  count: 'number',
  color: 'string|null',
};

export const upcomingItem: Shape = {
  date: 'string',
  kind: { $enum: ['planned_txn', 'card_due'] },
  description: 'string',
  amount: 'number',
  currency: 'string',
  account_id: 'string',
  account_name: 'string',
  ref_id: 'string',
};

export const dashboard: Shape = {
  month: 'string',
  basis,
  net_worth: netWorth,
  flow: monthFlow,
  flow_history: { $array: monthFlow },
  savings_rate_12m: 'number|null',
  top_categories: { $array: groupAmount },
  upcoming: { $array: upcomingItem },
  budget_planned: 'number|null',
  budget_actual: 'number',
  uncategorized_count: 'number',
  currency: 'string',
};

export const spendingSummary: Shape = {
  from: 'string',
  to: 'string',
  basis,
  kind,
  currency: 'string',
  total: 'number',
  count: 'number',
  groups: { $array: groupAmount },
  missing_fx: strings,
  estimated: 'boolean',
};

const sankeyKind: Shape = {
  $enum: [
    'income',
    'hub',
    'expense',
    'subcategory',
    'savings',
    'deficit',
    'investment',
    'redemption',
    'debt',
  ],
};

const sankeyTotals: Shape = {
  from: 'string',
  to: 'string',
  income: 'number',
  expense: 'number',
  debt_payments: 'number',
  investments: 'number',
  redemptions: 'number',
  leftover: 'number',
  savings_rate: 'number|null',
};

export const sankeyData: Shape = {
  from: 'string',
  to: 'string',
  basis,
  nodes: {
    $array: {
      id: 'string',
      name: 'string',
      kind: sankeyKind,
      category_id: 'string|null',
    },
  },
  links: { $array: { source: 'string', target: 'string', value: 'number' } },
  total_income: 'number',
  total_expense: 'number',
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
  card_bills_paid: 'number',
  depth: 'number',
  totals: sankeyTotals,
  previous: { $nullable: sankeyTotals },
  comparison: {
    $array: {
      id: 'string',
      name: 'string',
      kind: sankeyKind,
      category_id: 'string|null',
      parent_id: 'string|null',
      current: 'number',
      previous: 'number',
      delta: 'number',
      delta_pct: 'number|null',
      share_of_income: 'number|null',
    },
  },
};

const sankeyItemsPeriod: Shape = {
  from: 'string',
  to: 'string',
  value: 'number',
  items_total: 'number',
  items: {
    $array: {
      txn_id: 'string|null',
      node_id: 'string',
      component: 'string|null',
      date: 'string|null',
      due_date: 'string|null',
      label: 'string',
      account: 'string|null',
      category_id: 'string|null',
      group: 'string',
      amount: 'number',
      original_amount: 'number|null',
      original_currency: 'string|null',
      split: 'boolean',
      counted: 'boolean',
      note: 'string|null',
      estimated: 'boolean',
    },
  },
  missing_fx: strings,
  estimated: 'boolean',
  note: 'string|null',
};

export const sankeyItems: Shape = {
  node_ids: strings,
  name: 'string',
  kind: sankeyKind,
  basis,
  currency: 'string',
  current: sankeyItemsPeriod,
  previous: sankeyItemsPeriod,
  groups: {
    $array: {
      group: 'string',
      category_id: 'string|null',
      current: 'number',
      previous: 'number',
      delta: 'number',
      current_count: 'number',
      previous_count: 'number',
    },
  },
};

export const calendarData: Shape = {
  from: 'string',
  to: 'string',
  basis,
  days: { $array: { date: 'string', expense: 'number', income: 'number', count: 'number' } },
  max_expense: 'number',
  total_expense: 'number',
  total_income: 'number',
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
};

const evolutionSeries: Shape = {
  key: 'string|null',
  label: 'string',
  color: 'string|null',
  values: numbers,
};

export const monthlyEvolution: Shape = {
  months: strings,
  basis,
  series: { $array: evolutionSeries },
  totals: numbers,
  moving_avg_3: numbers,
  income: numbers,
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
};

export const inflationReport: Shape = {
  months: strings,
  basis,
  currency: 'string',
  nominal: numbers,
  real: { $array: 'number|null' },
  estimated: { $array: 'boolean' },
  inflation: 'number|null',
  inflation_estimated: 'boolean',
  index: {
    $nullable: { series_id: 'string', name: 'string', source: 'string', last_date: 'string|null' },
  },
  categories: {
    $array: {
      key: 'string|null',
      label: 'string',
      color: 'string|null',
      first: 'number',
      last: 'number',
      change: 'number|null',
      inflation: 'number|null',
      above_inflation: 'number|null',
    },
  },
  compare_from: 'string|null',
  missing_fx: strings,
  fx_estimated: 'boolean',
};

const period: Shape = { from: 'string', to: 'string' };

export const periodComparison: Shape = {
  a: period,
  b: period,
  basis,
  kind,
  rows: {
    $array: {
      key: 'string|null',
      label: 'string',
      a: 'number',
      b: 'number',
      delta: 'number',
      delta_ratio: 'number|null',
      color: 'string|null',
    },
  },
  total_a: 'number',
  total_b: 'number',
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
};

export const memberBreakdown: Shape = {
  from: 'string',
  to: 'string',
  members: { $array: groupAmount },
  categories: { $array: evolutionSeries },
  basis,
  total: 'number',
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
};

export const explanation: Shape = {
  metric: { $enum: ['account_balance', 'net_worth', 'spending', 'income', 'savings_rate'] },
  value: 'number',
  currency: 'string',
  formula: 'string',
  sources: {
    $array: {
      kind: {
        $enum: ['txn', 'account', 'fx_rate', 'opening', 'index', 'portfolio', 'loan', 'physical'],
      },
      id: 'string|null',
      label: 'string',
      date: 'string|null',
      amount: 'number',
      currency: 'string',
      source: 'string|null',
      estimated: 'boolean',
    },
  },
  truncated: 'boolean',
  estimated: 'boolean',
  ratio: 'number|null',
  count: 'number',
  missing_fx: strings,
};

export const budgetLineStatus: Shape = {
  category_id: 'string',
  category_name: 'string',
  category_path: 'string',
  parent_id: 'string|null',
  kind,
  planned: 'number',
  actual: 'number',
  remaining: 'number',
  ratio: 'number|null',
  committed: 'number',
  has_own_line: 'boolean',
  color: 'string|null',
  archived: 'boolean',
};

export const budgetStatus: Shape = {
  month: 'string',
  basis,
  lines: { $array: budgetLineStatus },
  totals: {
    planned_expense: 'number',
    actual_expense: 'number',
    planned_income: 'number',
    actual_income: 'number',
    committed_expense: 'number',
    committed_income: 'number',
    uncategorized_expense: 'number',
  },
  month_elapsed: 'number',
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
};

export const budgetLineInputs: Shape = { $array: { category_id: 'string', amount: 'number' } };

/** Uma fonte da meta e quanto vale hoje (docs/goal-sources.md). */
const goalSourceValue: Shape = {
  kind: 'string',
  ref_id: 'string|null',
  account_id: 'string|null',
  name: 'string',
  value: 'number',
  value_used: 'string',
  share: 'number|null',
  ignored_negative: 'number|null',
  stale: 'boolean',
};

export const goalView: Shape = {
  id: 'string',
  name: 'string',
  kind: {
    $enum: ['emergency', 'travel', 'debt_payoff', 'purchase', 'retirement', 'education', 'other'],
  },
  target_amount: 'number',
  currency: 'string',
  target_date: 'string|null',
  initial_amount: 'number',
  monthly_contribution: 'number|null',
  account_ids: strings,
  archived: 'boolean',
  notes: 'string|null',
  real_return: 'number|null',
  current_amount: 'number',
  progress: 'number',
  remaining: 'number',
  monthly_needed: 'number|null',
  avg_monthly_contribution: 'number',
  projected_date: 'string|null',
  on_track: 'boolean|null',
  missing_fx: strings,
  estimated: 'boolean',
  sources: {
    $array: {
      kind: 'string',
      ref_id: 'string',
      account_id: 'string|null',
      value_mode: 'string|null',
      share: 'number|null',
    },
  },
  breakdown: { $array: goalSourceValue },
  stale_sources: { $array: goalSourceValue },
};

export const emergencyReserve: Shape = {
  avg_essential_monthly: 'number',
  avg_essential_regular_monthly: 'number',
  monthly_cost: 'number',
  annual: {
    items: {
      $array: {
        category_id: 'string',
        category: 'string',
        essential: 'boolean',
        total: 'number',
        months: strings,
        monthly: 'number',
      },
    },
    monthly: 'number',
    essential_monthly: 'number',
    months: strings,
    complete_year: 'boolean',
  },
  include_annual: 'boolean',
  avg_total_expense_monthly: 'number',
  months_analyzed: 'number',
  target_months: 'number',
  target_amount: 'number',
  current_amount: 'number',
  coverage_months: 'number|null',
  goal_id: 'string|null',
  currency: 'string',
  months: strings,
  basis,
  missing_fx: strings,
  estimated: 'boolean',
};

export const portfolioSummary: Shape = {
  as_of: 'string',
  base_currency: 'string',
  deflator: 'string',
  market_value: 'number',
  cost: 'number',
  unrealized: 'number',
  realized: 'number',
  income: 'number',
  total_result: 'number',
  real_result: 'number|null',
  inflation_effect: 'number|null',
  xirr: 'number|null',
  xirr_real: 'number|null',
  by_class: {
    $array: {
      class: 'string',
      label: 'string',
      market_value_base: 'number',
      cost_base: 'number',
      weight: 'number',
      target_pct: 'number|null',
      deviation_pp: 'number|null',
      out_of_band: 'boolean',
      xirr: 'number|null',
    },
  },
  positions: { $array: 'object' },
  missing_prices: strings,
  balance_due: { $array: { asset_id: 'string', name: 'string', as_of: 'string|null' } },
  estimated: 'boolean',
  concentration_alerts: {
    $array: {
      kind: { $enum: ['asset', 'group', 'class'] },
      ref: 'string',
      label: 'string',
      weight: 'number',
      limit: 'number',
      severity: { $enum: ['warning', 'critical'] },
      excess_base: 'number',
      assets: { $array: 'object' },
    },
  },
};

export const recurrence: Shape = {
  id: 'string',
  description: 'string',
  account_id: 'string',
  account_name: 'string',
  amount: 'number',
  currency: 'string',
  kind: 'string',
  category_id: 'string|null',
  member_id: 'string|null',
  to_account_id: 'string|null',
  rrule: 'string',
  spec: { $partial: { freq: { $enum: ['DAILY', 'WEEKLY', 'MONTHLY', 'YEARLY'] } } },
  human: 'string',
  start_date: 'string',
  end_date: 'string|null',
  active: 'boolean',
  next_dates: strings,
  monthly_equivalent: 'number',
};

export const monthAmounts: Shape = { $array: { month: 'string', amount: 'number' } };

const spendGroup: Shape = { $enum: ['essential', 'debt', 'future', 'non_essential'] };

export const category: Shape = {
  id: 'string',
  parent_id: 'string|null',
  name: 'string',
  kind,
  icon: 'string|null',
  color: 'string|null',
  is_essential: 'boolean',
  spend_group: { $nullable: spendGroup },
  archived: 'boolean',
  sort_order: 'number',
  path: 'string',
};

export const member: Shape = {
  id: 'string',
  name: 'string',
  color: 'string|null',
  is_primary: 'boolean',
  archived: 'boolean',
  sort_order: 'number',
  profile: {
    birth_date: 'string|null',
    relationship: 'string|null',
    irpf_dependent: 'boolean',
    occupation: 'string|null',
    notes: 'string|null',
    sex: 'string|null',
  },
};

/** Só os campos de conta que estas telas leem. */
export const accountView: Shape = {
  $partial: {
    id: 'string',
    name: 'string',
    kind: 'string',
    currency: 'string',
    archived: 'boolean',
    sort_order: 'number',
  },
};

/** Comando → forma da resposta. */
const healthStatus: Shape = { $enum: ['good', 'ok', 'bad', 'unknown'] };
const healthModel: Shape = { $enum: ['isf', 'ratios', 'off'] };
const healthBand: Shape = { key: 'string', label: 'string', min: 'number', max: 'number' };

export const healthReport: Shape = {
  model: healthModel,
  enabled: 'boolean',
  as_of: 'string',
  period: { $nullable: { from: 'string', to: 'string' } },
  months_analyzed: 'number',
  score: 'number|null',
  band: { $nullable: healthBand },
  bands: { $array: healthBand },
  reference: {
    $nullable: { label: 'string', short_label: 'string', score: 'number', source_url: 'string' },
  },
  dimensions: {
    $array: {
      key: 'string',
      label: 'string',
      description: 'string',
      weight: 'number',
      score: 'number|null',
      status: healthStatus,
      metrics: strings,
    },
  },
  metrics: {
    $array: {
      key: 'string',
      label: 'string',
      dimension: 'string|null',
      value: 'number|null',
      unit: { $enum: ['percent', 'months', 'ratio', 'count'] },
      value_text: 'string|null',
      target: 'string',
      target_value: 'number|null',
      status: healthStatus,
      points: 'number|null',
      weight: 'number',
      explanation: 'string',
      computation: 'string',
      suggestions: strings,
      amounts: { $array: { label: 'string', amount: 'number', currency: 'string' } },
    },
  },
  summary: 'string',
  notes: strings,
  currency: 'string',
  missing_fx: strings,
  estimated: 'boolean',
  stale_inputs: {
    $array: {
      account_id: 'string',
      name: 'string',
      kind: 'string',
      last_txn_date: 'string|null',
      staleness_days: 'number|null',
      missed_month: 'string|null',
      negative: 'boolean',
      balance: 'number',
      currency: 'string',
    },
  },
};

export const healthSettingsView: Shape = {
  model: healthModel,
  models: {
    $array: {
      id: healthModel,
      label: 'string',
      description: 'string',
      source_url: 'string|null',
    },
  },
};

export const groupTargets: Shape = {
  essential: 'number|null',
  debt: 'number|null',
  future: 'number|null',
  non_essential: 'number|null',
};

const spendGroupMonth: Shape = {
  month: 'string',
  income: 'number',
  spending: 'number',
  groups: {
    $array: {
      group: spendGroup,
      amount: 'number',
      share_of_income: 'number|null',
      top: { $array: { category_id: 'string|null', label: 'string', amount: 'number' } },
    },
  },
};

export const spendGroupsReport: Shape = {
  currency: 'string',
  months: { $array: spendGroupMonth },
  total: spendGroupMonth,
  targets: groupTargets,
};

export const RESPONSES: { [cmd: string]: Shape } = {
  'reports.dashboard': dashboard,
  'reports.net_worth': netWorth,
  'reports.net_worth_history': { $array: netWorthPoint },
  'reports.month_flow': { $array: monthFlow },
  'reports.spending': spendingSummary,
  'reports.sankey': sankeyData,
  'reports.sankey_items': sankeyItems,
  'reports.calendar': calendarData,
  'reports.monthly_evolution': monthlyEvolution,
  'reports.compare': periodComparison,
  'reports.by_member': memberBreakdown,
  'reports.upcoming': { $array: upcomingItem },
  'reports.explain': explanation,
  'reports.inflation': inflationReport,
  'reports.spend_groups': spendGroupsReport,
  'budget.get': budgetStatus,
  'budget.set_lines': budgetStatus,
  'budget.copy': budgetStatus,
  'budget.suggest': budgetLineInputs,
  'budget.list_months': strings,
  'budget.group_targets': groupTargets,
  'budget.set_group_targets': groupTargets,
  'goals.list': { $array: goalView },
  'goals.get': goalView,
  'goals.create': goalView,
  'goals.update': goalView,
  'goals.emergency_reserve': emergencyReserve,
  'portfolio.summary': portfolioSummary,
  'recurrences.list': { $array: recurrence },
  'cards.future_commitments': monthAmounts,
  'categories.list': { $array: category },
  'household.list_members': { $array: member },
  'accounts.list': { $array: accountView },
  'health.score': healthReport,
  'health.get_settings': healthSettingsView,
};

function typeName(v: unknown): string {
  if (v === null) return 'null';
  if (Array.isArray(v)) return 'array';
  return typeof v;
}

/**
 * Confere `value` contra `shape`. Devolve as divergências (`caminho: esperado X, veio Y`,
 * `caminho: chave a mais`); vazio = de acordo.
 */
export function checkShape(value: unknown, shape: Shape, path = '$'): string[] {
  if (typeof shape === 'string') {
    if (shape === 'unknown') return [];
    const [t, nullable] = shape.split('|');
    if (value === null) return nullable === 'null' ? [] : [`${path}: esperado ${t}, veio null`];
    const actual = typeName(value);
    if (t === 'object')
      return actual === 'object' ? [] : [`${path}: esperado objeto, veio ${actual}`];
    return actual === t ? [] : [`${path}: esperado ${shape}, veio ${actual}`];
  }
  if ('$nullable' in shape) {
    return value === null ? [] : checkShape(value, shape.$nullable, path);
  }
  if ('$enum' in shape) {
    const options = shape.$enum as readonly string[];
    return typeof value === 'string' && options.includes(value)
      ? []
      : [`${path}: esperado um de [${options.join(', ')}], veio ${JSON.stringify(value)}`];
  }
  if ('$array' in shape) {
    if (!Array.isArray(value)) return [`${path}: esperado lista, veio ${typeName(value)}`];
    // Confere até 50 itens (o suficiente para achar divergências sem custar caro).
    return value.slice(0, 50).flatMap((v, i) => checkShape(v, shape.$array, `${path}[${i}]`));
  }
  const partial = '$partial' in shape;
  const fields = (partial ? shape.$partial : shape) as { [key: string]: Shape };
  if (typeName(value) !== 'object') return [`${path}: esperado objeto, veio ${typeName(value)}`];
  const obj = value as { [key: string]: unknown };
  const out: string[] = [];
  for (const [k, s] of Object.entries(fields)) {
    if (!(k in obj)) out.push(`${path}.${k}: faltando`);
    else out.push(...checkShape(obj[k], s, `${path}.${k}`));
  }
  if (!partial) {
    for (const k of Object.keys(obj)) if (!(k in fields)) out.push(`${path}.${k}: chave a mais`);
  }
  return out;
}
