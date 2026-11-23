/**
 * Mock de `attention.*` ("Para resolver"). Só **compõe** o que os outros mocks já devolvem
 * (alertas, percepções, sugestões e contas), como o núcleo faz em `attention/mod.rs`; nenhuma
 * regra de dinheiro é recalculada aqui. Diferenças assumidas: os vencimentos vêm dos alertas
 * `due_soon` do mock, e não dos previstos de agora; o próximo passo é um exemplo fixo; e o
 * fechamento do mês só mede os extratos e as pendências.
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { Alert } from '@/api/generated/Alert';
import type { Attention } from '@/api/generated/Attention';
import type { AttentionItem } from '@/api/generated/AttentionItem';
import type { CloseStep } from '@/api/generated/CloseStep';
import type { DataCoverage } from '@/api/generated/DataCoverage';
import type { Insights } from '@/api/generated/Insights';
import type { MonthClose } from '@/api/generated/MonthClose';
import type { NextStep } from '@/api/generated/NextStep';
import type { Suggestion } from '@/api/generated/Suggestion';

import { extraMockHandler, mockFail, onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { today } from './ledger/dates';

const EVERYDAY = new Set(['checking', 'savings', 'cash', 'credit_card', 'benefit', 'foreign']);
/** Tipos que a lista mostra pela regra de agora; o alerta gravado fica de fora. */
const LIVE_ALERT_KINDS = new Set(['budget_over', 'stale_accounts']);
const TODO_KINDS = new Set([
  'onboarding',
  'uncategorized',
  'budget_over',
  'asset_no_price',
  'b3_import',
  'reserve_liquidity',
  'pending_proposals',
]);
const ORDER = { critical: 0, warning: 1, info: 2 } as const;
const SOURCE = { due: 0, alert: 1, data: 2, todo: 3, insight: 4 } as const;

let dismissed = new Set<string>();
onMockReset(() => {
  dismissed = new Set();
});

function invoke<T>(cmd: string, args: Record<string, unknown> = {}): T {
  return extraMockHandler(cmd)!(args) as T;
}

function coverage(): DataCoverage {
  const everyday = invoke<AccountView[]>('accounts.list').filter(
    (a) => a.include_in_net_worth && EVERYDAY.has(a.kind),
  );
  const stale = everyday
    .filter((a) => a.balance_status === 'possibly_incomplete')
    .map((a) => ({
      account_id: a.id,
      name: a.name,
      last_txn_date: a.last_txn_date,
      staleness_days: a.staleness_days,
    }));
  const news = everyday.map((a) =>
    [a.last_txn_date, a.last_import_at]
      .filter((d): d is string => !!d)
      .sort()
      .at(-1),
  );
  return {
    status: everyday.length === 0 ? 'empty' : stale.length ? 'partial' : 'complete',
    accounts: everyday.length,
    stale,
    complete_until: news.length && news.every((d) => d !== undefined) ? news.sort()[0]! : null,
  };
}

function item(base: Pick<AttentionItem, 'id' | 'source' | 'kind' | 'severity' | 'title' | 'body'>) {
  return {
    action: null,
    route: null,
    target: null,
    alert_id: null,
    count: 1,
    date: null,
    ...base,
  } satisfies AttentionItem;
}

function list(): Attention {
  const cov = coverage();
  const items: AttentionItem[] = [];
  for (const a of invoke<Alert[]>('ai.alerts.list', { limit: 200 })) {
    if (LIVE_ALERT_KINDS.has(a.kind)) continue;
    items.push({
      ...item({
        id: `alert:${a.id}`,
        source: a.kind === 'due_soon' ? 'due' : 'alert',
        kind: a.kind,
        severity: a.severity,
        title: a.title,
        body: a.body,
      }),
      target: a.target,
      alert_id: a.id,
      date: a.created_at.slice(0, 10),
    });
  }
  if (cov.stale.length) {
    items.push({
      ...item({
        id: 'data:stale_accounts',
        source: 'data',
        kind: 'stale_accounts',
        severity: 'warning',
        title:
          cov.stale.length === 1
            ? '1 conta sem extrato recente. Importar?'
            : `${cov.stale.length} contas sem extrato recente. Importar?`,
        body: `O saldo pode estar incompleto: ${cov.stale.map((s) => s.name).join('; ')}.`,
      }),
      target: {
        screen: 'import',
        id: null,
        account_id: cov.stale[0]!.account_id,
        month: null,
        date: null,
      },
      count: cov.stale.length,
    });
  }
  for (const s of invoke<Suggestion[]>('ai.suggestions', { limit: 20 })) {
    if (!TODO_KINDS.has(s.kind)) continue;
    items.push({
      ...item({
        id: `todo:${s.kind}`,
        source: 'todo',
        kind: s.kind,
        severity: s.severity,
        title: s.title,
        body: s.detail,
      }),
      route: s.route,
      count: Math.max(s.count, 1),
    });
  }
  for (const i of invoke<Insights>('reports.insights').items) {
    if (i.tone !== 'warning' || i.kind === 'personal_inflation') continue;
    items.push({
      ...item({
        id: `insight:${i.id}`,
        source: 'insight',
        kind: i.kind,
        severity: 'warning',
        title: i.title,
        body: i.body,
      }),
      action: i.action,
      route: i.screen ? `/${i.screen}` : null,
    });
  }
  const open = items
    .filter((i) => !dismissed.has(i.id))
    .map((i, n) => ({ i, n }))
    .sort(
      (a, b) =>
        ORDER[a.i.severity] - ORDER[b.i.severity] ||
        SOURCE[a.i.source] - SOURCE[b.i.source] ||
        a.n - b.n,
    )
    .map((x) => x.i);
  return {
    items: open,
    counts: {
      total: open.length,
      urgent: open.filter((i) => i.severity !== 'info').length,
    },
    coverage: cov,
  };
}

/** O mês que terminou por último (`YYYY-MM`), pelo "hoje" do mock. */
function lastMonth(): string {
  const [y, m] = today().split('-').map(Number) as [number, number];
  return m === 1 ? `${y - 1}-12` : `${y}-${String(m - 1).padStart(2, '0')}`;
}

const RUNG_TITLES: [string, string][] = [
  ['know', 'Saber para onde o dinheiro vai'],
  ['balance', 'Gastar menos do que ganha'],
  ['credit', 'Sair do crédito caro'],
  ['reserve', 'Montar a reserva de emergência'],
  ['debts', 'Deixar as dívidas leves'],
  ['invest', 'Investir e proteger o que tem'],
];

/** Exemplo fixo: a família de exemplo está montando a reserva. As regras moram no núcleo. */
function nextStep(): NextStep {
  return {
    rung: 4,
    key: 'reserve',
    title: RUNG_TITLES[3]![1],
    why: 'A reserva cobre 1,4 meses de gastos essenciais. O próximo marco é 3 meses.',
    action:
      'Separe um valor todo mês, numa aplicação que dê para resgatar no mesmo dia, até chegar lá.',
    route: '/orcamento?aba=reserva',
    rungs: RUNG_TITLES.map(([key, title], i) => ({ key, title, done: i < 3 })),
    estimated: coverage().status === 'partial',
  };
}

let closeState: Record<string, { balances: boolean; summary: boolean; closed_at: string | null }> =
  {};
onMockReset(() => {
  closeState = {};
});

function monthClose(): MonthClose {
  const month = lastMonth();
  const st = closeState[month] ?? { balances: false, summary: false, closed_at: null };
  const att = list();
  const stale = att.coverage.stale;
  const open = att.items.filter(
    (i) =>
      ['due', 'data', 'todo'].includes(i.source) &&
      !['stale_accounts', 'uncategorized'].includes(i.kind),
  ).length;
  const step = (
    key: string,
    title: string,
    done: boolean,
    manual: boolean,
    detail: string,
    route: string,
  ): CloseStep => ({ key, title, done, manual, detail: done ? '' : detail, route });
  const steps = [
    step(
      'statements',
      'Trazer os extratos',
      stale.length === 0,
      false,
      `${stale.length} ${stale.length === 1 ? 'conta' : 'contas'} sem extrato recente: ${stale.map((a) => a.name).join(', ')}.`,
      '/lancamentos?importar=1',
    ),
    step(
      'categories',
      'Dar categoria aos lançamentos',
      true,
      false,
      '',
      '/lancamentos?uncategorized=1',
    ),
    step(
      'balances',
      'Conferir os saldos',
      st.balances,
      true,
      'Compare o saldo de cada conta com o do banco e marque quando bater.',
      '/contas',
    ),
    step(
      'review',
      'Resolver as pendências',
      open === 0,
      false,
      open === 1 ? '1 item em Para resolver.' : `${open} itens em Para resolver.`,
      '/painel?resolver=1',
    ),
    step(
      'summary',
      'Ler o resumo do mês',
      st.summary,
      true,
      'Veja nos relatórios para onde o dinheiro foi e marque quando ler.',
      `/relatorios?mes=${month}`,
    ),
  ];
  return {
    month,
    applicable: true,
    steps,
    done: steps.filter((s) => s.done).length,
    closed_at: st.closed_at,
  };
}

registerMockHandlers(
  guarded({
    'attention.list': list,
    'attention.next_step': nextStep,
    'attention.month_close': monthClose,
    'attention.month_close_set': (args) => {
      const month = typeof args.month === 'string' ? args.month : lastMonth();
      const st = (closeState[month] ??= { balances: false, summary: false, closed_at: null });
      const done = args.done === true;
      if (args.step === 'balances') st.balances = done;
      else if (args.step === 'summary') st.summary = done;
      else if (args.step === 'closed') st.closed_at = done ? new Date().toISOString() : null;
      else mockFail('invalid', 'Passo desconhecido do fechamento do mês.');
      return { month, step: args.step, done };
    },
    'attention.coverage': coverage,
    'attention.dismiss': (args) => {
      const id = typeof args.id === 'string' ? args.id : '';
      if (id.startsWith('alert:')) {
        invoke('ai.alerts.set_status', { ids: [id.slice(6)], status: 'resolved' });
      } else if (/^(due|data|todo|insight):/.test(id)) {
        dismissed.add(id);
      } else {
        mockFail('invalid', 'Este item não pode ser dispensado.');
      }
      return { dismissed: id };
    },
  }),
);
