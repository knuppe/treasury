#!/usr/bin/env node
/**
 * Contrato das telas 01 · Painel, Orçamento e Relatórios contra o núcleo real.
 *
 * Cria (uma vez) um cofre com contas, cartão com parcelamento, recorrência, lançamentos com
 * divisão e sem categoria, conta em USD com câmbio e em EUR sem câmbio, orçamento e meta;
 * depois chama todos os comandos que as telas usam, confere a forma de cada resposta com
 * `src/api/contract/reportShapes.ts` (o mesmo contrato que o teste do mock usa) e algumas
 * regras de coerência entre comandos (o que o Sankey recebe sai, o painel bate com o fluxo,
 * a explicação soma o valor, o drill-down da tela de lançamentos acha o mesmo total…).
 *
 * Uso (Node 22.18+, que roda TypeScript sem flags):
 *   treasury-dev serve --home /tmp/tdr --port 7782 --token dev --no-mcp-socket
 *   node apps/desktop/scripts/contract-reports.mjs
 *
 * Variáveis: TREASURY_API (padrão http://127.0.0.1:7782), TREASURY_TOKEN (dev),
 * TREASURY_PASSWORD (senha do cofre criado/aberto pelo script).
 */
import { checkShape, RESPONSES } from '../src/api/contract/reportShapes.ts';

const API = (process.env.TREASURY_API ?? 'http://127.0.0.1:7782').replace(/\/+$/, '');
const TOKEN = process.env.TREASURY_TOKEN ?? 'dev';
const PASSWORD = process.env.TREASURY_PASSWORD ?? 'senha de teste longa 1';
const MARK = 'Contrato · Corrente';

class CoreError extends Error {
  constructor(code, message) {
    super(`${code}: ${message}`);
    this.code = code;
  }
}

async function call(cmd, args = {}) {
  const res = await fetch(`${API}/api`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${TOKEN}` },
    body: JSON.stringify({ cmd, args }),
  });
  const body = await res.json();
  if (!body.ok)
    throw new CoreError(body.error?.code ?? 'erro', body.error?.message ?? `HTTP ${res.status}`);
  return body.data;
}

// ── Datas ────────────────────────────────────────────────────────

const pad = (n) => String(n).padStart(2, '0');
const now = new Date();
const TODAY = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
const MONTH = TODAY.slice(0, 7);
function shiftMonth(m, n) {
  const [y, mm] = m.split('-').map(Number);
  const i = y * 12 + (mm - 1) + n;
  return `${Math.floor(i / 12)}-${pad((i % 12) + 1)}`;
}
function monthBounds(m) {
  const [y, mm] = m.split('-').map(Number);
  return { from: `${m}-01`, to: `${m}-${pad(new Date(Date.UTC(y, mm, 0)).getUTCDate())}` };
}
const day = (m, d) => `${m}-${pad(Math.min(d, Number(monthBounds(m).to.slice(8))))}`;
function addDays(iso, n) {
  const [y, m, d] = iso.split('-').map(Number);
  const t = new Date(Date.UTC(y, m - 1, d + n));
  return `${t.getUTCFullYear()}-${pad(t.getUTCMonth() + 1)}-${pad(t.getUTCDate())}`;
}

// ── Preparação ───────────────────────────────────────────────────

async function openVault() {
  const status = await call('vault.status');
  if (status.state === 'no_vault') {
    await call('vault.create', {
      password: PASSWORD,
      household_name: 'Família Contrato',
      members: ['Ana', 'Bruno'],
    });
  } else if (status.state === 'locked') {
    await call('vault.unlock', { password: PASSWORD });
  }
}

async function seed() {
  const accounts = await call('accounts.list', { include_archived: true });
  const existing = accounts.find((a) => a.name === MARK);
  const categories = await call('categories.list', { include_archived: true });
  const byPath = new Map(categories.map((c) => [c.path, c.id]));
  const cat = (path) => {
    const id = byPath.get(path);
    if (!id) throw new Error(`categoria padrão ausente: ${path}`);
    return id;
  };
  const members = await call('household.list_members');
  const [ana, bruno] = members;
  const first = shiftMonth(MONTH, -3);
  if (existing) {
    const find = (n) => accounts.find((a) => a.name === n).id;
    return {
      checking: existing.id,
      savings: find('Contrato · Poupança'),
      card: find('Contrato · Cartão'),
      usd: find('Contrato · USD'),
      cat,
      ana,
      bruno,
      first,
      created: false,
    };
  }
  const open = monthBounds(first).from;
  const checking = (
    await call('accounts.create', {
      name: MARK,
      kind: 'checking',
      opening_balance: 500_000,
      opening_date: open,
      member_ids: [ana.id, bruno.id],
    })
  ).id;
  const savings = (
    await call('accounts.create', {
      name: 'Contrato · Poupança',
      kind: 'savings',
      opening_balance: 1_000_000,
      opening_date: open,
      member_ids: [ana.id],
    })
  ).id;
  const card = (
    await call('accounts.create', {
      name: 'Contrato · Cartão',
      kind: 'credit_card',
      opening_date: open,
      member_ids: [bruno.id],
      card: {
        credit_limit: 1_500_000,
        closing_day: 3,
        due_day: 10,
        payment_account_id: checking,
        network: 'Visa',
        last4: '1234',
      },
    })
  ).id;
  const usd = (
    await call('accounts.create', {
      name: 'Contrato · USD',
      kind: 'foreign',
      currency: 'USD',
      opening_balance: 50_000,
      opening_date: open,
    })
  ).id;
  await call('accounts.create', {
    name: 'Contrato · EUR',
    kind: 'foreign',
    currency: 'EUR',
    opening_balance: 20_000,
    opening_date: open,
  });
  await call('fx.set', { base: 'USD', quote: 'BRL', date: open, rate: '5.40' });

  const txn = (input) =>
    input.date <= TODAY ? call('transactions.create', input) : Promise.resolve(null);
  for (let i = -3; i <= 0; i++) {
    const m = shiftMonth(MONTH, i);
    await txn({
      account_id: checking,
      date: day(m, 5),
      amount: 1_200_000,
      kind: 'income',
      description: 'Salário',
      category_id: cat('Salário'),
      member_id: ana.id,
    });
    await txn({
      account_id: checking,
      date: day(m, 10),
      amount: -300_000,
      kind: 'expense',
      description: 'Aluguel',
      category_id: cat('Moradia › Aluguel'),
      member_id: ana.id,
    });
    await txn({
      account_id: card,
      date: day(m, 2),
      amount: -42_000,
      kind: 'expense',
      description: 'Mercado',
      category_id: cat('Alimentação › Mercado'),
      member_id: bruno.id,
    });
    await txn({
      account_id: card,
      date: day(m, 4),
      amount: -18_000,
      kind: 'expense',
      description: 'Restaurante',
      category_id: cat('Alimentação › Restaurantes'),
      member_id: ana.id,
    });
    await txn({
      account_id: card,
      date: day(m, 1),
      amount: -50_000,
      kind: 'expense',
      description: 'Mercado + ração',
      member_id: bruno.id,
      splits: [
        { category_id: cat('Alimentação › Mercado'), amount: -38_000 },
        { category_id: cat('Pets › Ração'), amount: -12_000 },
      ],
    });
    await txn({
      account_id: usd,
      date: day(m, 3),
      amount: -2_000,
      kind: 'expense',
      description: 'Assinatura em dólar',
      category_id: cat('Assinaturas › Software e apps'),
      member_id: bruno.id,
    });
    await txn({
      account_id: checking,
      date: day(m, 2),
      amount: -9_000,
      kind: 'expense',
      description: 'Cinema',
      category_id: cat('Lazer › Cinema e shows'),
      member_id: ana.id,
    });
    if (day(m, 6) <= TODAY) {
      await call('transactions.create_transfer', {
        from_account_id: checking,
        to_account_id: savings,
        date: day(m, 6),
        amount: 150_000,
        description: 'Reserva',
      });
    }
  }
  await txn({
    account_id: checking,
    date: TODAY,
    amount: -7_700,
    kind: 'expense',
    description: 'PIX sem categoria',
  });
  await call('transactions.create', {
    account_id: checking,
    date: day(shiftMonth(MONTH, 1), 10),
    amount: -300_000,
    kind: 'expense',
    status: 'planned',
    description: 'Aluguel (previsto)',
    category_id: cat('Moradia › Aluguel'),
  });
  await call('recurrences.create', {
    description: 'Streaming',
    account_id: card,
    amount: -5_590,
    kind: 'expense',
    category_id: cat('Assinaturas › Streaming'),
    spec: { freq: 'MONTHLY', by_month_day: 17 },
    start_date: day(first, 17),
  });
  await call('cards.create_installments', {
    account_id: card,
    description: 'Notebook',
    total_amount: 450_000,
    installments: 10,
    purchase_date: addDays(TODAY, -40),
    category_id: cat('Compras › Eletrônicos'),
    member_id: bruno.id,
  });
  await call('budget.set_lines', {
    month: MONTH,
    lines: [
      { category_id: cat('Moradia'), amount: 400_000 },
      { category_id: cat('Alimentação › Mercado'), amount: 150_000 },
      { category_id: cat('Lazer'), amount: 30_000 },
      { category_id: cat('Salário'), amount: 1_200_000 },
    ],
  });
  await call('goals.create', {
    name: 'Reserva de emergência',
    kind: 'emergency',
    target_amount: 3_000_000,
    target_date: day(shiftMonth(MONTH, 24), 1),
    account_ids: [savings],
  });
  return { checking, savings, card, usd, cat, ana, bruno, first, created: true };
}

// ── Verificações ─────────────────────────────────────────────────

const results = [];
let failed = 0;

async function check(cmd, args, extra) {
  const label = `${cmd} ${JSON.stringify(args)}`;
  try {
    const data = await call(cmd, args);
    const issues = RESPONSES[cmd]
      ? checkShape(data, RESPONSES[cmd])
      : [`sem forma no contrato para ${cmd}`];
    if (extra) issues.push(...(extra(data) ?? []));
    results.push({ label, issues });
    if (issues.length) failed++;
    return data;
  } catch (e) {
    results.push({ label, issues: [String(e.message ?? e)] });
    failed++;
    return undefined;
  }
}

const expectEq = (what, a, b) => (a === b ? [] : [`${what}: ${a} ≠ ${b}`]);

async function main() {
  await openVault();
  const s = await seed();
  const m = monthBounds(MONTH);
  const prev = monthBounds(shiftMonth(MONTH, -1));
  const cat = s.cat;

  const dash = await check('reports.dashboard', {});
  await check('reports.dashboard', { month: shiftMonth(MONTH, -1) });
  const nw = await check('reports.net_worth', { as_of: TODAY }, (d) => [
    ...expectEq('patrimônio líquido = ativos − passivos', d.net, d.assets - d.liabilities),
    ...(d.missing_fx.includes('EUR') ? [] : ['EUR deveria aparecer em missing_fx']),
  ]);
  await check('reports.net_worth_history', { months: 24, end_month: MONTH }, (d) =>
    d.length && d[d.length - 1].net !== nw?.net
      ? [`último mês do histórico (${d[d.length - 1].net}) ≠ patrimônio de hoje (${nw?.net})`]
      : [],
  );
  const flows = await check(
    'reports.month_flow',
    { from_month: shiftMonth(MONTH, -11), to_month: MONTH },
    (d) =>
      dash
        ? expectEq(
            'painel.flow.expense = month_flow (último)',
            dash.flow.expense,
            d[d.length - 1].expense,
          )
        : [],
  );
  await check('reports.month_flow', {
    from_month: shiftMonth(MONTH, -11),
    to_month: MONTH,
    basis: 'cash',
  });
  const spendPrev = await check('reports.spending', { ...prev, group_by: 'category' }, (d) =>
    expectEq(
      'Σ grupos = total',
      d.groups.reduce((a, g) => a + g.amount, 0),
      d.total,
    ),
  );
  await check('reports.spending', {
    ...prev,
    group_by: 'parent_category',
    member_ids: [s.bruno.id],
  });
  await check('reports.spending', { ...prev, group_by: 'member' });
  await check('reports.spending', { ...prev, group_by: 'account', kind: 'income' });
  await check('reports.spending', { ...prev, group_by: 'month', account_ids: [s.card] });
  await check('reports.spending', { ...prev, basis: 'cash', group_by: 'parent_category' });
  const flow = await check('reports.sankey', prev, (d) => {
    const into = d.links.filter((l) => l.target === 'hub').reduce((a, l) => a + l.value, 0);
    const out = d.links.filter((l) => l.source === 'hub').reduce((a, l) => a + l.value, 0);
    return [
      ...expectEq('Sankey: entra no orçamento = sai', into, out),
      ...expectEq('Sankey: despesas = gastos', d.total_expense, spendPrev?.total),
    ];
  });
  // O analítico de cada linha do comparativo soma, ao centavo, as duas células.
  for (const row of flow?.comparison ?? []) {
    await check('reports.sankey_items', { ...prev, node_ids: [row.id] }, (d) => [
      ...expectEq(`analítico de ${row.name} (atual)`, d.current.items_total, row.current),
      ...expectEq(`analítico de ${row.name} (anterior)`, d.previous.items_total, row.previous),
    ]);
  }
  await check('reports.calendar', { from: monthBounds(shiftMonth(MONTH, -11)).from, to: TODAY });
  await check('reports.calendar', prev, (d) =>
    expectEq(
      'Σ dias = gastos',
      d.days.reduce((a, x) => a + x.expense, 0),
      spendPrev?.total,
    ),
  );
  await check(
    'reports.monthly_evolution',
    { months: 12, end_month: shiftMonth(MONTH, -1), top_n: 99 },
    (d) =>
      expectEq(
        'evolução: total do último mês = gastos',
        d.totals[d.totals.length - 1],
        spendPrev?.total,
      ),
  );
  await check('reports.compare', {
    a: monthBounds(shiftMonth(MONTH, -2)),
    b: prev,
    kind: 'expense',
  });
  await check('reports.by_member', prev, (d) =>
    expectEq('titulares: total = gastos', d.total, spendPrev?.total),
  );
  await check('reports.upcoming', { days: 60 }, (d) =>
    d.some((u) => u.kind === 'card_due') ? [] : ['esperava ao menos uma fatura a vencer'],
  );
  await check(
    'reports.explain',
    { metric: 'spending', params: { ...prev }, limit: 10_000 },
    (d) => [
      ...expectEq('explicação = gastos', d.value, spendPrev?.total),
      ...expectEq(
        'Σ fontes = valor',
        d.sources.reduce((a, x) => a + x.amount, 0),
        d.value,
      ),
    ],
  );
  await check('reports.explain', {
    metric: 'spending',
    params: { ...prev, category_id: cat('Moradia'), member_id: s.ana.id },
  });
  await check('reports.explain', { metric: 'income', params: { ...prev, basis: 'cash' } });
  await check('reports.explain', {
    metric: 'savings_rate',
    params: { month: shiftMonth(MONTH, -1) },
  });
  await check('reports.explain', { metric: 'net_worth', params: { as_of: TODAY } }, (d) =>
    expectEq(
      'Σ fontes do patrimônio = valor',
      d.sources.reduce((a, x) => a + x.amount, 0),
      d.value,
    ),
  );
  await check('reports.explain', { metric: 'account_balance', params: { account_id: s.checking } });

  // Drill-down: o filtro que a tela de lançamentos recebe acha o mesmo total.
  const moradia = spendPrev?.groups.find((g) => g.key === cat('Moradia › Aluguel'));
  try {
    const page = await call('transactions.query', {
      date_from: prev.from,
      date_to: prev.to,
      category_ids: [cat('Moradia')],
      kinds: ['expense'],
      statuses: ['cleared', 'reconciled'],
      limit: 2000,
    });
    const sum = -page.items.reduce((a, t) => a + t.amount, 0);
    const issues = expectEq(
      'drill-down Moradia (lançamentos) = gasto do relatório',
      sum,
      moradia?.amount,
    );
    results.push({ label: 'transactions.query (drill-down de categoria)', issues });
    if (issues.length) failed++;
  } catch (e) {
    results.push({
      label: 'transactions.query (drill-down de categoria)',
      issues: [String(e.message)],
    });
    failed++;
  }

  const budget = await check('budget.get', { month: MONTH }, (d) =>
    dash
      ? expectEq('orçamento: realizado = painel', d.totals.actual_expense, dash.budget_actual)
      : [],
  );
  await check('budget.get', { month: MONTH, basis: 'cash' });
  await check(
    'budget.set_lines',
    { month: MONTH, lines: [{ category_id: cat('Pets'), amount: 20_000 }] },
    (d) =>
      d.lines.some((l) => l.category_id === cat('Pets') && l.has_own_line)
        ? []
        : ['linha de Pets não gravada'],
  );
  await check('budget.set_lines', {
    month: MONTH,
    lines: [{ category_id: cat('Pets'), amount: 0 }],
  });
  await check(
    'budget.copy',
    { from_month: MONTH, to_month: shiftMonth(MONTH, 1), overwrite: true },
    (d) =>
      budget
        ? expectEq('cópia: previsto igual', d.totals.planned_expense, budget.totals.planned_expense)
        : [],
  );
  await check('budget.suggest', { month: MONTH, months_back: 3 }, (d) =>
    d.every((l) => l.amount % 1000 === 0) ? [] : ['sugestão fora do múltiplo de R$ 10'],
  );
  await check('budget.list_months', {});
  const targets = await check('budget.group_targets', {});
  if (targets) {
    await check('budget.set_group_targets', { ...targets, debt: 0.1 }, (d) =>
      expectEq('meta de dívidas gravada', d.debt, 0.1),
    );
    await call('budget.set_group_targets', targets);
  }
  await check('reports.spend_groups', { from: shiftMonth(MONTH, -2), to: MONTH }, (d) => [
    ...expectEq('três meses', d.months.length, 3),
    ...expectEq(
      'grupos na ordem',
      d.total.groups.map((g) => g.group).join(','),
      'essential,debt,future,non_essential',
    ),
  ]);

  const goals = await check('goals.list', { include_archived: true });
  const reserve = goals?.find((g) => g.kind === 'emergency');
  if (reserve) await check('goals.get', { id: reserve.id });
  const tmp = await check('goals.create', {
    name: 'Meta do contrato',
    kind: 'travel',
    target_amount: 500_000,
    monthly_contribution: 50_000,
  });
  if (tmp) {
    await check('goals.update', {
      id: tmp.id,
      target_date: day(shiftMonth(MONTH, 6), 1),
      monthly_contribution: null,
      archived: true,
    });
    await call('goals.delete', { id: tmp.id });
  }
  await check('goals.emergency_reserve', { target_months: 6, months_back: 3 }, (d) =>
    reserve ? expectEq('reserva usa a meta de emergência', d.goal_id, reserve.id) : [],
  );

  await check('portfolio.summary', {});
  await check('recurrences.list', {}, (d) =>
    d.some((r) => r.description === 'Streaming') ? [] : ['recorrência não listada'],
  );
  await check('cards.future_commitments', { account_id: s.card, months: 12 }, (d) =>
    d.length === 12 && d.reduce((a, x) => a + x.amount, 0) > 0
      ? []
      : ['esperava 12 meses com parcelas'],
  );
  await check('categories.list', {});
  await check('household.list_members', {});
  await check('accounts.list', {});

  for (const r of results) {
    console.log(`${r.issues.length ? '✗' : '✓'} ${r.label}`);
    for (const i of r.issues) console.log(`    ${i}`);
  }
  console.log(
    `\n${results.length - failed}/${results.length} verificações de acordo${s.created ? ' (dados criados agora)' : ''}.`,
  );
  process.exit(failed ? 1 : 0);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});
