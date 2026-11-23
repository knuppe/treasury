/**
 * Mock do agente (`ai.*`) e da auditoria do MCP (`mcp.audit_list`).
 *
 * O chat simula um job que chega aos poucos: cada `ai.chat_poll` revela alguns eventos
 * (motor → raciocínio → passo → ferramenta → resultado → proposta → resposta → uso → fim),
 * e a proposta criada entra de verdade na fila de aprovação do mock. A mensagem gravada
 * separa resposta, raciocínio e ferramentas como o núcleo (`ai::activity`). A memória de
 * procedimentos do agente (`ai.procedures.*`) também mora aqui. Nada sai para a rede.
 *
 * MCPs extras (`ai.mcp_extra.*`): lista em memória, segredos guardados só como "cadastrado"
 * e a dica dos 4 últimos caracteres. Uma pergunta sobre o celular, com um servidor ligado
 * que pede confirmação, para o job no pedido (`external_approval`) até `ai.mcp_extra.decide`.
 * Uma mensagem com "pergunte" (ou "ask me") faz o agente perguntar (`user_question`) e esperar
 * `ai.question.answer`; "pergunte várias" pede escolha múltipla, sem "Outro".
 *
 * Ids com prefixo `mock-ai-` para não colidir com os outros mocks.
 */
import type { AiSettings } from '@/api/generated/AiSettings';
import type { AttachmentInfo } from '@/api/generated/AttachmentInfo';
import type { EngineCaps } from '@/api/generated/EngineCaps';
import type { HistoryStats } from '@/api/generated/HistoryStats';
import type { AiSettingsPatch } from '@/api/generated/AiSettingsPatch';
import type { AiTask } from '@/api/generated/AiTask';
import type { AskAnswer } from '@/api/generated/AskAnswer';
import type { AskQuestion } from '@/api/generated/AskQuestion';
import type { Alert } from '@/api/generated/Alert';
import type { AlertStatus } from '@/api/generated/AlertStatus';
import type { AuditEntry } from '@/api/generated/AuditEntry';
import type { ChangeItem } from '@/api/generated/ChangeItem';
import type { ChatPoll } from '@/api/generated/ChatPoll';
import type { ChatStart } from '@/api/generated/ChatStart';
import type { CliDetection } from '@/api/generated/CliDetection';
import type { Conversation } from '@/api/generated/Conversation';
import type { ConversationDetail } from '@/api/generated/ConversationDetail';
import type { Engine } from '@/api/generated/Engine';
import type { EngineInput } from '@/api/generated/EngineInput';
import type { EngineKind } from '@/api/generated/EngineKind';
import type { EngineTest } from '@/api/generated/EngineTest';
import type { JobEvent } from '@/api/generated/JobEvent';
import type { JobEventKind } from '@/api/generated/JobEventKind';
import type { JobLink } from '../generated/JobLink';
import type { JobInfo } from '@/api/generated/JobInfo';
import type { JobStatus } from '@/api/generated/JobStatus';
import type { McpServer } from '@/api/generated/McpServer';
import type { McpServerInput } from '@/api/generated/McpServerInput';
import type { McpTestResult } from '@/api/generated/McpTestResult';
import type { McpVar } from '@/api/generated/McpVar';
import type { McpVarInput } from '@/api/generated/McpVarInput';
import type { MemoryItem } from '@/api/generated/MemoryItem';
import type { Message } from '@/api/generated/Message';
import type { ModelPrice } from '@/api/generated/ModelPrice';
import type { Procedure } from '@/api/generated/Procedure';
import type { ProcedureTopic } from '@/api/generated/ProcedureTopic';
import type { Proposal } from '@/api/generated/Proposal';
import type { ProposalDetail } from '@/api/generated/ProposalDetail';
import type { ProposalLevel } from '@/api/generated/ProposalLevel';
import type { Provider } from '@/api/generated/Provider';
import type { ProviderInfo } from '@/api/generated/ProviderInfo';
import type { ReviewResult } from '@/api/generated/ReviewResult';
import type { Routine } from '@/api/generated/Routine';
import type { RoutineStatus } from '@/api/generated/RoutineStatus';
import type { SpendLimit } from '@/api/generated/SpendLimit';
import type { Suggestion } from '@/api/generated/Suggestion';
import type { TaskRoute } from '@/api/generated/TaskRoute';
import type { UsageGroup } from '@/api/generated/UsageGroup';
import type { UsageSummary } from '@/api/generated/UsageSummary';
import type { UsageTotals } from '@/api/generated/UsageTotals';
import {
  extraMockHandler,
  mockFail,
  onMockReset,
  registerMockHandlers,
  requireUnlocked,
  type MockHandler,
} from '@/api/mockRegistry';

import { ensureSeeded } from './ledger/boot';
import { cat } from './ledger/fixture';
import { rows } from './ledger/store';

// --- Utilitários ------------------------------------------------------------------------

type Args = Record<string, unknown>;

function str(a: Args, name: string): string {
  const v = a[name];
  if (typeof v !== 'string' || v === '')
    mockFail('invalid', `argumentos inválidos: missing field \`${name}\``);
  return v;
}

function optStr(a: Args, name: string): string | undefined {
  const v = a[name];
  return typeof v === 'string' && v !== '' ? v : undefined;
}

function optNum(a: Args, name: string): number | undefined {
  const v = a[name];
  return typeof v === 'number' && Number.isFinite(v) ? v : undefined;
}

let seq = 0;
const newId = (kind: string) => `mock-ai-${kind}-n${++seq}`;
const now = () => new Date().toISOString();
/** Instante relativo a agora (minutos atrás), para os dados semeados. */
const ago = (minutes: number) => new Date(Date.now() - minutes * 60_000).toISOString();

const TASKS: readonly AiTask[] = ['chat', 'categorize', 'pdf', 'monthly_close', 'alerts', 'study'];
const TASK_LABEL: Record<AiTask, string> = {
  chat: 'Chat com o agente',
  categorize: 'Categorização',
  pdf: 'Leitura de PDF',
  monthly_close: 'Fechamento mensal',
  alerts: 'Alertas',
  study: 'Estudo de ativos',
};

// --- Providers (espelho de `ai::engine::providers`) ----------------------------------

const PROVIDERS: ProviderInfo[] = (
  [
    ['anthropic', 'api', 'Anthropic (Claude)', 'https://api.anthropic.com', 'claude-sonnet-4-5'],
    ['openai', 'api', 'OpenAI', 'https://api.openai.com/v1', 'gpt-5-mini'],
    [
      'gemini',
      'api',
      'Google Gemini',
      'https://generativelanguage.googleapis.com/v1beta',
      'gemini-2.5-flash',
    ],
    ['xai', 'api', 'xAI (Grok)', 'https://api.x.ai/v1', 'grok-4'],
    ['mistral', 'api', 'Mistral', 'https://api.mistral.ai/v1', 'mistral-large-latest'],
    [
      'openrouter',
      'api',
      'OpenRouter',
      'https://openrouter.ai/api/v1',
      'anthropic/claude-sonnet-4.5',
    ],
    ['groq', 'api', 'Groq', 'https://api.groq.com/openai/v1', 'llama-3.3-70b-versatile'],
    ['openai_compatible', 'api', 'Endpoint compatível com OpenAI', null, null],
    ['claude_code', 'cli', 'Claude Code (CLI)', null, null],
    ['codex', 'cli', 'OpenAI Codex (CLI)', null, null],
    ['gemini_cli', 'cli', 'Gemini CLI', null, null],
    ['grok_cli', 'cli', 'Grok (CLI)', null, null],
    ['opencode_cli', 'cli', 'OpenCode (CLI)', null, null],
    ['custom_cli', 'cli', 'CLI personalizada', null, null],
    ['ollama', 'local', 'Ollama', 'http://127.0.0.1:11434/v1', 'qwen3:4b'],
    ['lm_studio', 'local', 'LM Studio', 'http://127.0.0.1:1234/v1', 'qwen/qwen3-4b'],
  ] as const
).map(([provider, kind, label, endpoint, model]) => {
  const templates: Partial<Record<Provider, [string, string]>> = {
    claude_code: [
      'claude',
      'claude -p {prompt} [--model {model}] --mcp-config {mcp_config} --strict-mcp-config --allowedTools "mcp__treasury__*" --output-format stream-json --verbose',
    ],
    codex: [
      'codex',
      'codex exec --sandbox read-only --skip-git-repo-check --cd {workdir} [--model {model}] -c mcp_servers.treasury={mcp_config} {prompt}',
    ],
    gemini_cli: ['gemini', 'gemini [-m {model}] --allowed-mcp-server-names treasury -p {prompt}'],
    grok_cli: ['grok', 'grok [--model {model}] -p {prompt}'],
    opencode_cli: [
      'opencode',
      'opencode run --standalone --format json [--model {model}] {prompt}',
    ],
  };
  const t = templates[provider];
  return {
    provider,
    kind,
    label,
    default_endpoint: endpoint,
    default_model: model,
    default_template: t?.[1] ?? null,
    binary: t?.[0] ?? null,
    needs_key: kind === 'api' && provider !== 'openai_compatible',
  };
});

function providerInfo(p: Provider): ProviderInfo {
  const info = PROVIDERS.find((x) => x.provider === p);
  if (!info) mockFail('invalid', `provider desconhecido: ${p}`);
  return info;
}

// --- Estado -----------------------------------------------------------------------------

interface StoredEngine {
  id: string;
  name: string;
  provider: Provider;
  model: string | null;
  endpoint: string | null;
  command_template: string | null;
  enabled: boolean;
  max_output_tokens: number | null;
  created_at: string;
  updated_at: string;
}

interface StoredProposal {
  proposal: Proposal;
  items: ChangeItem[];
  /** Linhas tiradas do livro-caixa na primeira leitura (o banco do mock é semeado depois). */
  fill?: () => ChangeItem[];
}

interface MockJob {
  id: string;
  task: AiTask;
  kind: 'chat' | 'routine';
  createdAt: string;
  /** Roteiro completo; cada poll revela `perPoll` eventos. */
  script: JobEventKind[];
  revealed: JobEvent[];
  /** Próximo índice do roteiro a revelar. */
  cursor: number;
  /** Como o job termina depois do roteiro. */
  outcome: 'done' | 'failed';
  status: JobStatus;
  conversationId: string | null;
  engineName: string | null;
  /** Do que se trata (lista de tarefas em segundo plano). */
  title?: string;
  finishedAt?: string;
  /** Onde abrir o resultado (ex.: o estudo com IA). */
  link?: JobLink;
  /** Efeitos ao revelar o evento de índice i (ex.: criar a proposta). */
  effects: Map<number, () => void>;
  /** Ao terminar bem: grava a resposta e devolve o id da mensagem. */
  onDone: (() => string | undefined) | null;
  /** Pedido de confirmação de ferramenta externa: o roteiro para depois de `at` até a resposta. */
  gate?: { at: number; requestId: string; allowed: boolean | null; remember?: boolean };
  /** Pergunta à pessoa (`ask_user`): o roteiro para depois de `at` até a resposta. */
  ask?: AskGate;
}

interface AskGate {
  at: number;
  requestId: string;
  question: AskQuestion;
  /** `undefined` enquanto espera; `null` sem resposta (cancelada). */
  answer: AskAnswer | null | undefined;
}

interface StoredConversation {
  conversation: Conversation;
  messages: Message[];
}

interface AiMockState {
  engines: StoredEngine[];
  secrets: Map<string, { hint: string; updated_at: string }>;
  routes: Map<AiTask, string[]>;
  limits: Map<Provider, SpendLimit>;
  prices: ModelPrice[];
  usage: {
    at: string;
    engine_id: string;
    task: AiTask;
    input: number;
    output: number;
    cost: number;
    priced: boolean;
    ok: boolean;
  }[];
  settings: AiSettings;
  proposals: StoredProposal[];
  memory: MemoryItem[];
  procedures: Procedure[];
  alerts: Alert[];
  conversations: StoredConversation[];
  jobs: Map<string, MockJob>;
  routinesLastRun: Partial<Record<Routine, string>>;
  audit: AuditEntry[];
}

/** Eventos revelados por consulta (o chat chega aos poucos). */
const PER_POLL = 2;

function engineRecord(
  id: string,
  provider: Provider,
  name: string,
  model: string | null,
  minutes: number,
): StoredEngine {
  return {
    id,
    name,
    provider,
    model,
    endpoint: null,
    command_template: null,
    enabled: true,
    max_output_tokens: null,
    created_at: ago(minutes),
    updated_at: ago(minutes),
  };
}

const ENGINE_CLAUDE = 'mock-ai-engine-claude';
const ENGINE_OLLAMA = 'mock-ai-engine-ollama';
const ENGINE_CLI = 'mock-ai-engine-claude-code';

function txnItem(
  seqNo: number,
  rowId: string,
  description: string,
  amount: number,
  fromCat: string | null,
  toCat: string,
): ChangeItem {
  const base = {
    id: rowId,
    account_id: 'mock-acc-nubank',
    date: '2026-10-08',
    description,
    amount,
    currency: 'BRL',
    status: 'cleared',
    created_at: '2026-10-08T12:00:00Z',
  };
  return {
    seq: seqNo,
    table_name: 'txn',
    row_id: rowId,
    op: 'update',
    before: { ...base, category_id: fromCat, updated_at: '2026-10-08T12:00:00Z' },
    after: { ...base, category_id: toCat, updated_at: '2026-10-14T09:12:00Z' },
  };
}

/** Linha provisória: só conta na lista até a proposta ser lida (`fill`). */
function placeholderTxn(seqNo: number): ChangeItem {
  return {
    seq: seqNo,
    table_name: 'txn',
    row_id: `pending-${seqNo}`,
    op: 'update',
    before: null,
    after: null,
  };
}

/**
 * Categorizar lançamentos sem categoria do livro-caixa, como o núcleo grava a alteração:
 * só o id, a categoria e a data técnica (sem descrição nem valor).
 */
function uncategorizedItems(categoryId: string, count: number): ChangeItem[] {
  ensureSeeded();
  const open = rows('txn').filter(
    (t) => t.category_id === null && t.kind === 'expense' && !t.transfer_id,
  );
  const cards = open.filter((t) => t.description.startsWith('COMPRA'));
  return (cards.length >= count ? cards : open).slice(0, count).map((t, i) => ({
    seq: i + 1,
    table_name: 'txn',
    row_id: t.id,
    op: 'update',
    before: { id: t.id, category_id: null, updated_at: t.updated_at },
    after: { id: t.id, category_id: categoryId, updated_at: '2026-10-14T09:12:00Z' },
  }));
}

function proposalRecord(
  id: string,
  command: string,
  level: ProposalLevel,
  reason: string,
  items: ChangeItem[],
  minutes: number,
  client = 'agente interno · Claude (API)',
): StoredProposal {
  const summary: Record<string, Record<string, number>> = {};
  for (const it of items) {
    const ops = (summary[it.table_name] ??= {});
    ops[it.op] = (ops[it.op] ?? 0) + 1;
  }
  return {
    proposal: {
      id,
      change_set_id: `${id}-cs`,
      client,
      command,
      level,
      status: 'pending',
      reason,
      created_at: ago(minutes),
      reviewed_at: null,
      error: null,
      note: null,
      item_count: items.length,
      summary,
    },
    items,
  };
}

function seedProposals(): StoredProposal[] {
  const approved = proposalRecord(
    'mock-ai-proposal-old',
    'ai.remember',
    'reversible',
    'Memória do gestor',
    [
      {
        seq: 1,
        table_name: 'ai_memory',
        row_id: 'mock-ai-memory-3',
        op: 'insert',
        before: null,
        after: { id: 'mock-ai-memory-3', kind: 'note', content: 'Ana prefere relatórios mensais' },
      },
    ],
    60 * 24 * 3,
  );
  approved.proposal.status = 'approved';
  approved.proposal.reviewed_at = ago(60 * 24 * 3 - 5);
  return [
    {
      ...proposalRecord(
        'mock-ai-proposal-categorize',
        'transactions.set_category',
        'reversible',
        'Categorizar 2 compras no cartão como Mercado',
        // Contagem para a lista; as linhas de verdade vêm de `uncategorizedItems`.
        [placeholderTxn(1), placeholderTxn(2)],
        42,
      ),
      fill: () => uncategorizedItems(cat('alimentacao', 'mercado'), 2),
    },
    proposalRecord(
      'mock-ai-proposal-rule',
      'rules.create',
      'reversible',
      'Regra: “PAO DE ACUCAR” → Mercado',
      [
        {
          seq: 1,
          table_name: 'rule',
          row_id: 'mock-rule-ai-1',
          op: 'insert',
          before: null,
          after: {
            id: 'mock-rule-ai-1',
            name: 'Pão de Açúcar → Mercado',
            priority: 100,
            active: 1,
            match_all: 1,
            // Colunas JSON chegam do núcleo como texto.
            conditions: JSON.stringify([
              { field: 'description', op: 'contains', value: 'PAO DE ACUCAR' },
            ]),
            actions: JSON.stringify({ category_id: cat('alimentacao', 'mercado') }),
            created_at: '2026-10-14T09:12:00Z',
            updated_at: '2026-10-14T09:12:00Z',
          },
        },
      ],
      40,
    ),
    proposalRecord(
      'mock-ai-proposal-trade',
      'portfolio.create_trade',
      'sensitive',
      'Compra de 100 PETR4 lida na nota de corretagem',
      [
        {
          seq: 1,
          table_name: 'trade',
          row_id: 'mock-trade-ai-1',
          op: 'insert',
          before: null,
          after: {
            id: 'mock-trade-ai-1',
            asset_id: 'ast-petr4',
            account_id: 'acc-xp',
            date: '2026-10-10',
            side: 'buy',
            quantity: '100',
            price: '38.50',
            fees: 490,
            taxes: 0,
            currency: 'BRL',
            day_trade: 0,
            note_number: '88412',
            created_at: '2026-10-14T09:15:00Z',
            updated_at: '2026-10-14T09:15:00Z',
          },
        },
      ],
      30,
    ),
    approved,
  ];
}

/** Espelho de `alertas-tipo-*` do núcleo (o mock fala pt-BR). */
const KIND_LABEL: Record<string, string> = {
  budget_over: 'Orçamento estourado',
  due_soon: 'Vencimento',
  rebalance: 'Rebalanceamento',
  monthly_close: 'Fechamento mensal',
};

function seedAlerts(): Alert[] {
  const alert = (
    id: string,
    kind: string,
    severity: Alert['severity'],
    title: string,
    body: string,
    status: AlertStatus,
    minutes: number,
    data: Alert['data'] = {},
    target: Alert['target'] = null,
  ): Alert => ({
    id,
    kind,
    severity,
    title,
    body,
    status,
    source: 'rotina de alertas',
    ref_table: null,
    ref_id: null,
    data,
    created_at: ago(minutes),
    updated_at: ago(minutes),
    seen_at: status === 'new' ? null : ago(minutes - 1),
    resolved_at: status === 'resolved' ? ago(minutes - 2) : null,
    kind_label: KIND_LABEL[kind] ?? kind,
    target,
  });
  return [
    alert(
      'mock-ai-alert-budget',
      'budget_over',
      'warning',
      'Mercado passou de 90% do orçamento',
      'Faltam 17 dias para fechar o mês e o gasto com **Mercado** já está em 92% do previsto.\n\n_Análise baseada nos seus dados, não é aconselhamento financeiro profissional._',
      'new',
      20,
      { sources: ['get_budget_status'] },
      { screen: 'budget', id: 'cat-mercado', account_id: null, month: null, date: null },
    ),
    alert(
      'mock-ai-alert-due',
      'due_soon',
      'critical',
      'Fatura do Nubank vence em 2 dias',
      'Fatura de outubro: vencimento em 16/10.',
      'new',
      90,
    ),
    alert(
      'mock-ai-alert-rebalance',
      'rebalance',
      'info',
      'Carteira 6 p.p. acima do alvo em ações',
      'Ações estão em 46% (alvo 40%).',
      'seen',
      60 * 26,
      {},
      { screen: 'allocation', id: null, account_id: null, month: null, date: null },
    ),
    alert(
      'mock-ai-alert-close',
      'monthly_close',
      'info',
      'Fechamento de setembro',
      'Taxa de poupança de 21%; 4 recomendações.',
      'resolved',
      60 * 24 * 12,
    ),
  ];
}

function seedConversation(): StoredConversation {
  const id = 'mock-ai-conv-1';
  const messages: Message[] = [
    {
      id: 'mock-ai-msg-1',
      role: 'user',
      content: 'Quanto gastamos com restaurantes em setembro?',
      engine_id: null,
      engine_name: null,
      tool_calls: [],
      proposal_ids: [],
      reasoning: null,
      reasoning_ms: null,
      created_at: ago(60 * 24),
      attachments: [],
    },
    {
      id: 'mock-ai-msg-2',
      role: 'assistant',
      content:
        'Em setembro foram R$ 1.284,90 em Restaurantes, 12% acima da média dos últimos 6 meses.',
      engine_id: ENGINE_CLAUDE,
      engine_name: 'Claude (API)',
      tool_calls: [
        {
          name: 'spending_summary',
          args: '{"from":"2026-09-01","to":"2026-09-30","group_by":"category"}',
          summary: 'Restaurantes · 2026-09 · por mês',
          ok: true,
        },
      ],
      proposal_ids: [],
      reasoning: 'Vou somar as despesas de Restaurantes em setembro e comparar com a média.',
      reasoning_ms: 4_200,
      created_at: ago(60 * 24 - 1),
      attachments: [],
    },
  ];
  return {
    conversation: {
      id,
      title: 'Restaurantes em setembro',
      task: 'chat',
      last_engine_id: ENGINE_CLAUDE,
      created_at: ago(60 * 24),
      updated_at: ago(60 * 24 - 1),
      message_count: messages.length,
      pinned: false,
    },
    messages,
  };
}

function seedState(): AiMockState {
  return {
    engines: [
      engineRecord(ENGINE_CLAUDE, 'anthropic', 'Claude (API)', 'claude-sonnet-4-5', 600),
      engineRecord(ENGINE_OLLAMA, 'ollama', 'Ollama local', 'llama3.2', 500),
      engineRecord(ENGINE_CLI, 'claude_code', 'Claude Code', null, 400),
    ],
    secrets: new Map([[ENGINE_CLAUDE, { hint: '…a1b2', updated_at: ago(600) }]]),
    routes: new Map<AiTask, string[]>([
      ['chat', [ENGINE_CLAUDE, ENGINE_OLLAMA]],
      ['categorize', [ENGINE_OLLAMA]],
    ]),
    limits: new Map<Provider, SpendLimit>([
      [
        'anthropic',
        {
          provider: 'anthropic',
          daily_micros: 1_000_000,
          monthly_micros: 10_000_000,
          confirm_above_micros: 100_000,
        },
      ],
    ]),
    prices: [
      price('anthropic', 'claude-sonnet', 3_000_000, 15_000_000, 300_000),
      price('anthropic', 'claude-haiku', 1_000_000, 5_000_000, 100_000),
      price('openai', 'gpt-5-mini', 250_000, 2_000_000, 25_000),
      price('gemini', 'gemini-2.5-flash', 300_000, 2_500_000, null),
    ],
    usage: seedUsage(),
    settings: {
      autonomous: false,
      mcp_allow_writes: false,
      keep_history: true,
      attachment_retention: 'none',
      history_retention_days: 90,
      routines: { review: true, monthly_close: true, alerts: true },
      cli_timeout_minutes: 15,
    },
    proposals: seedProposals(),
    memory: [
      memory('mock-ai-memory-1', 'profile', 'Perfil moderado, horizonte de 10 anos', 'user', 6000),
      memory('mock-ai-memory-2', 'family_rule', 'O aluguel vence dia 5', 'user', 5000),
      memory('mock-ai-memory-3', 'note', 'Ana prefere relatórios mensais', 'ai', 60 * 24 * 3),
    ],
    procedures: seedProcedures(),
    alerts: seedAlerts(),
    conversations: [seedConversation()],
    jobs: new Map(),
    routinesLastRun: { alerts: ago(15), review: ago(120) },
    audit: seedAudit(),
  };
}

function price(
  provider: Provider,
  model: string,
  input: number,
  output: number,
  cached: number | null,
): ModelPrice {
  return {
    provider,
    model,
    input_micros_per_mtok: input,
    output_micros_per_mtok: output,
    cached_micros_per_mtok: cached,
    source: 'default',
    updated_at: '2026-10-01T00:00:00Z',
  };
}

function memory(
  id: string,
  kind: MemoryItem['kind'],
  content: string,
  by: string,
  minutes: number,
): MemoryItem {
  return { id, kind, content, created_by: by, created_at: ago(minutes), updated_at: ago(minutes) };
}

function seedUsage(): AiMockState['usage'] {
  const out: AiMockState['usage'] = [];
  // Seis meses de uso, crescendo devagar; o mês atual mais leve.
  for (let m = 5; m >= 0; m--) {
    const d = new Date();
    d.setDate(5);
    d.setMonth(d.getMonth() - m);
    const at = d.toISOString();
    const calls = 6 + (5 - m) * 3;
    for (let i = 0; i < calls; i++) {
      out.push({
        at,
        engine_id: ENGINE_CLAUDE,
        task: i % 3 === 0 ? 'monthly_close' : 'chat',
        input: 4200,
        output: 650,
        cost: 22_350,
        priced: true,
        ok: true,
      });
    }
    out.push({
      at,
      engine_id: ENGINE_OLLAMA,
      task: 'categorize',
      input: 9000,
      output: 1200,
      cost: 0,
      priced: true,
      ok: m !== 2,
    });
  }
  return out;
}

function seedAudit(): AuditEntry[] {
  const entry = (
    i: number,
    client: string,
    tool: string,
    params: Record<string, unknown>,
    ok: boolean,
    minutes: number,
    error: string | null = null,
  ): AuditEntry => ({
    id: `mock-ai-audit-${i}`,
    created_at: ago(minutes),
    client,
    tool,
    params,
    ok,
    error,
    duration_ms: 12 + i * 7,
  });
  return [
    entry(1, 'agente interno · Claude (API)', 'spending_summary', { month: '2026-09' }, true, 5),
    entry(
      2,
      'agente interno · Claude (API)',
      'query_transactions',
      { text: 'PAO DE ACUCAR' },
      true,
      4,
    ),
    entry(
      3,
      'cli-teste via treasury mcp',
      'categorize_transactions',
      { ids: ['…2 itens'], category_id: 'cat-mercado' },
      true,
      3,
    ),
    entry(
      4,
      'cli-teste via treasury mcp',
      'generate_darf',
      { month: '2026-09' },
      false,
      2,
      'forbidden: tax.generate_darf não pode ser feito pela IA',
    ),
  ];
}

let state: AiMockState = seedState();
let mockLastCleanup: string | null = null;

/** Resposta de exemplo com markdown (mensagem com "markdown"), para conferir o desenho. */
const MARKDOWN_SAMPLE = [
  'Propus a categoria **Transporte**; aprove na [fila](/agente?aba=aprovacao).',
  '',
  '## Outros ajustes',
  '',
  '1. **Arquivar a conta** antiga do *Itaú* (saldo `R$ 0,00`).',
  '2. Revisar o cartão:',
  '   - fechamento dia 3',
  '   - vencimento dia 10',
  '',
  '| Ativo | Qtd | Valor |',
  '|:--|--:|--:|',
  '| PETR4 | 100 | R$ 3.050,00 |',
  '| HGLG11 | 12 | R$ 1.898,40 |',
  '',
  '> Recomendações são análises dos seus dados, não aconselhamento.',
  '',
  '```',
  'regra: UBER* → Transporte',
  '```',
].join('\n');

/** Anexos preparados e ainda não enviados (`ai.attachments.prepare`). */
const staged = new Map<string, AttachmentInfo>();
/** Companheiro: contexto da tela recebido em cada `ai.chat_start` (para os testes). */
let pageContexts: unknown[] = [];
/** Argumentos de cada `ai.chat_start` (para os testes). */
let chatStarts: Args[] = [];
/** Sugestões fixas para testes (`null` = calculadas do mock). */
let suggestionsOverride: Suggestion[] | null = null;
onMockReset(() => {
  seq = 0;
  state = seedState();
  pageContexts = [];
  chatStarts = [];
  staged.clear();
  suggestionsOverride = null;
  mcpServers = [];
  cliMcpsAllowed = [];
});

/** Para testes: argumentos de cada `ai.chat_start`, na ordem. */
export function mockChatStarts(): Args[] {
  return chatStarts.map((a) => ({ ...a }));
}

/** Para testes: contextos de tela enviados com as mensagens, na ordem. */
export function mockPageContexts(): unknown[] {
  return [...pageContexts];
}

/** Para testes: fixa as sugestões do companheiro (`null` volta ao cálculo do mock). */
export function mockSetSuggestions(list: Suggestion[] | null): void {
  suggestionsOverride = list;
}

const ONBOARDING_PROMPT_MOCK =
  'Quero cadastrar onde a família tem dinheiro. Pergunte onde temos contas e proponha o cadastro.';

/** Espelho simples de `ai::suggestions` (núcleo): a tela aberta primeiro. */
function suggestions(a: Args): Suggestion[] {
  const screen = optStr(a, 'screen') ?? '';
  const limit = optNum(a, 'limit') ?? 6;
  let list: Suggestion[];
  if (suggestionsOverride) {
    list = [...suggestionsOverride];
  } else {
    list = [];
    const accounts = extraMockHandler('accounts.list')?.({});
    if (Array.isArray(accounts) && accounts.length === 0) {
      list.push({
        id: 'onboarding',
        kind: 'onboarding',
        severity: 'info',
        title: 'Cadastrar as contas da família',
        detail:
          'Comece pela conta que você mais usa (banco, cartão ou corretora). Se tiver um assistente de IA configurado, ele pode propor o cadastro das outras e dizer como trazer o extrato de cada uma.',
        prompt: ONBOARDING_PROMPT_MOCK,
        route: '/contas',
        screens: ['painel', 'contas', 'lancamentos', 'carteira'],
        count: 0,
      });
    }
    const pending = state.proposals.filter((p) => p.proposal.status === 'pending').length;
    if (pending > 0) {
      list.push({
        id: 'pending_proposals',
        kind: 'pending_proposals',
        severity: 'info',
        title: `${pending} proposta${pending === 1 ? '' : 's'} aguardando aprovação`,
        detail: 'Nada muda até você aprovar; cada uma mostra o diff exato.',
        prompt: 'Resuma as propostas pendentes e diga quais parecem seguras para aprovar.',
        route: '/agente?aba=aprovacao',
        screens: ['agente'],
        count: pending,
      });
    }
  }
  const rank = (s: Suggestion) =>
    s.kind === 'onboarding' || s.screens.includes(screen) ? 0 : s.severity === 'warning' ? 1 : 2;
  return list
    .map((s, i) => ({ s, i }))
    .sort((x, y) => rank(x.s) - rank(y.s) || x.i - y.i)
    .map((x) => x.s)
    .slice(0, limit);
}

// --- Motores ----------------------------------------------------------------------------

/** Espelho simplificado de `attachments::caps_for` e `effort::supported`. */
function capsOf(provider: Provider, model: string | null): EngineCaps {
  const m = (model ?? '').toLowerCase();
  const images =
    ['anthropic', 'gemini', 'claude_code', 'codex'].includes(provider) ||
    (provider === 'openai' && /gpt-4o|gpt-4\.1|gpt-5|o3|o4/.test(m)) ||
    (provider === 'xai' && m.includes('grok-4')) ||
    (['ollama', 'lm_studio'].includes(provider) && /vision|llava|vl/.test(m));
  const pdf = images && ['anthropic', 'gemini', 'openai'].includes(provider);
  const effort =
    ['anthropic', 'gemini', 'openrouter', 'claude_code', 'grok_cli', 'codex'].includes(provider) ||
    (provider === 'openai' && /^(o1|o3|o4|gpt-5)/.test(m)) ||
    (provider === 'xai' && m.includes('grok-3-mini'));
  return { images, pdf, effort };
}

function toEngine(e: StoredEngine): Engine {
  const info = providerInfo(e.provider);
  const secret = state.secrets.get(e.id);
  return {
    ...e,
    kind: info.kind,
    provider_label: info.label,
    effective_endpoint: e.endpoint ?? info.default_endpoint,
    effective_template: e.command_template ?? info.default_template,
    secret: {
      set: Boolean(secret),
      hint: secret?.hint ?? null,
      needs_reentry: false,
      updated_at: secret?.updated_at ?? null,
    },
    needs_key: info.needs_key,
    vision: null,
    caps: capsOf(e.provider, e.model),
  };
}

function findEngine(id: string): StoredEngine {
  const e = state.engines.find((x) => x.id === id);
  if (!e) mockFail('not_found', `motor ${id} não encontrado`);
  return e;
}

function readiness(e: Engine): string | null {
  if (!e.enabled) return 'motor desligado';
  if (e.needs_key && !e.secret.set) return 'sem API key cadastrada';
  if (e.kind === 'cli' && !e.effective_template) return 'sem template de comando';
  if (e.kind !== 'cli' && !e.model) return 'sem modelo definido';
  return null;
}

function validateEndpoint(url: string): void {
  const lower = url.toLowerCase();
  if (lower.startsWith('https://')) return;
  if (/^http:\/\/(localhost|127\.0\.0\.1|\[::1\])(:\d+)?(\/|$)/.test(lower)) return;
  mockFail('invalid', 'use https:// (http só para servidores neste computador)');
}

function setSecret(engineId: string, value: string) {
  const v = value.trim();
  if (!v) mockFail('invalid', 'a chave está vazia');
  state.secrets.set(engineId, { hint: `…${v.slice(-4)}`, updated_at: now() });
}

function createEngine(a: EngineInput): Engine {
  const info = providerInfo(a.provider);
  const endpoint = a.endpoint?.trim() || null;
  if (endpoint) validateEndpoint(endpoint);
  const template = a.command_template?.trim() || null;
  if (template && !template.includes('{prompt}'))
    mockFail('invalid', 'o template precisa do marcador {prompt}');
  const rec: StoredEngine = {
    id: newId('engine'),
    name: a.name?.trim() || info.label,
    provider: a.provider,
    model: a.model?.trim() || info.default_model,
    endpoint,
    command_template: template,
    enabled: a.enabled ?? true,
    max_output_tokens: a.max_output_tokens ?? null,
    created_at: now(),
    updated_at: now(),
  };
  state.engines.push(rec);
  if (a.api_key) setSecret(rec.id, a.api_key);
  return toEngine(rec);
}

function updateEngine(a: Args): Engine {
  const e = findEngine(str(a, 'id'));
  if (typeof a.name === 'string') e.name = a.name.trim() || e.name;
  if ('model' in a) e.model = (a.model as string | null)?.trim() || null;
  if ('endpoint' in a) {
    const ep = (a.endpoint as string | null)?.trim() || null;
    if (ep) validateEndpoint(ep);
    e.endpoint = ep;
  }
  if ('command_template' in a) e.command_template = (a.command_template as string | null) || null;
  if (typeof a.enabled === 'boolean') e.enabled = a.enabled;
  if ('max_output_tokens' in a)
    e.max_output_tokens = (a.max_output_tokens as number | null) ?? null;
  e.updated_at = now();
  return toEngine(e);
}

// --- Uso --------------------------------------------------------------------------------

function totals(rows: AiMockState['usage']): UsageTotals {
  return {
    calls: rows.length,
    input_tokens: rows.reduce((s, r) => s + r.input, 0),
    output_tokens: rows.reduce((s, r) => s + r.output, 0),
    cost_micros: rows.reduce((s, r) => s + r.cost, 0),
    unpriced_calls: rows.filter((r) => !r.priced).length,
    errors: rows.filter((r) => !r.ok).length,
  };
}

function groupBy(
  rows: AiMockState['usage'],
  key: (r: AiMockState['usage'][number]) => string,
  label: (k: string) => string,
  kind: (k: string) => EngineKind | null,
): UsageGroup[] {
  const map = new Map<string, AiMockState['usage']>();
  for (const r of rows) map.set(key(r), [...(map.get(key(r)) ?? []), r]);
  return [...map.entries()]
    .map(([k, rs]) => ({ key: k, label: label(k), kind: kind(k), totals: totals(rs) }))
    .sort((a, b) => b.totals.cost_micros - a.totals.cost_micros || a.key.localeCompare(b.key));
}

function localDay(d: Date): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

function usageSummary(a: Args): UsageSummary {
  const today = localDay(new Date());
  const from = optStr(a, 'from') ?? `${today.slice(0, 7)}-01`;
  const to = optStr(a, 'to') ?? today;
  if (from > to) mockFail('invalid', '`from` não pode ser depois de `to`');
  const rows = state.usage.filter((r) => {
    const d = localDay(new Date(r.at));
    return d >= from && d <= to;
  });
  const eng = (id: string) => state.engines.find((e) => e.id === id);
  const provOf = (r: AiMockState['usage'][number]) => eng(r.engine_id)?.provider ?? 'openai';
  const monthRows = state.usage.filter(
    (r) => localDay(new Date(r.at)).slice(0, 7) === today.slice(0, 7),
  );
  const dayRows = state.usage.filter((r) => localDay(new Date(r.at)) === today);
  const apiProviders = [
    ...new Set([
      ...state.engines
        .filter((e) => providerInfo(e.provider).kind === 'api')
        .map((e) => e.provider),
      ...state.limits.keys(),
    ]),
  ];
  return {
    from,
    to,
    currency: 'USD',
    totals: totals(rows),
    by_provider: groupBy(
      rows,
      provOf,
      (k) => providerInfo(k as Provider).label,
      (k) => providerInfo(k as Provider).kind,
    ),
    by_engine: groupBy(
      rows,
      (r) => r.engine_id,
      (k) => eng(k)?.name ?? k,
      (k) => {
        const e = eng(k);
        return e ? providerInfo(e.provider).kind : null;
      },
    ),
    by_task: groupBy(
      rows,
      (r) => r.task,
      (k) => TASK_LABEL[k as AiTask] ?? k,
      () => null,
    ),
    by_month: groupBy(
      rows,
      (r) => localDay(new Date(r.at)).slice(0, 7),
      (k) => k,
      () => null,
    ).sort((x, y) => x.key.localeCompare(y.key)),
    spend: apiProviders.map((p) => ({
      provider: p,
      label: providerInfo(p).label,
      today_micros: dayRows.filter((r) => provOf(r) === p).reduce((s, r) => s + r.cost, 0),
      month_micros: monthRows.filter((r) => provOf(r) === p).reduce((s, r) => s + r.cost, 0),
      limit: state.limits.get(p) ?? {
        provider: p,
        daily_micros: null,
        monthly_micros: null,
        confirm_above_micros: null,
      },
    })),
  };
}

// --- Chat (job simulado) ----------------------------------------------------------------

function routeCandidates(task: AiTask, explicit?: string): Engine[] {
  if (explicit) return [toEngine(findEngine(explicit))];
  let ids = state.routes.get(task) ?? [];
  if (ids.length === 0 && task !== 'chat') ids = state.routes.get('chat') ?? [];
  return ids
    .map((id) => state.engines.find((e) => e.id === id))
    .filter((e): e is StoredEngine => Boolean(e?.enabled))
    .map(toEngine);
}

function addProposalFromChat(): string {
  const id = newId('proposal');
  const rec = proposalRecord(
    id,
    'transactions.set_category',
    'reversible',
    'Categorizar “UBER *TRIP” como Transporte',
    [txnItem(1, newId('txn'), 'UBER *TRIP', -3_270, null, 'cat-transporte')],
    0,
  );
  state.proposals.unshift(rec);
  return id;
}

function chatScript(
  engine: Engine,
  message: string,
): {
  script: JobEventKind[];
  effects: Map<number, () => void>;
  gate?: MockJob['gate'];
  ask?: AskGate;
} {
  const external = externalScript(engine, message);
  if (external) return external;
  const asked = questionScript(engine, message);
  if (asked) return asked;
  const effects = new Map<number, () => void>();
  const script: JobEventKind[] = [
    {
      type: 'engine',
      engine_id: engine.id,
      engine_name: engine.name,
      kind: engine.kind,
      provider: engine.provider,
      model: engine.model,
    },
    { type: 'reasoning', delta: 'O usuário quer categorizar o que falta; ' },
    { type: 'reasoning', delta: 'começo pelos lançamentos sem categoria.' },
    { type: 'text', delta: 'Vou olhar os lançamentos ' },
    { type: 'text', delta: 'sem categoria deste mês.' },
    { type: 'tool_call', name: 'query_transactions', summary: 'sem categoria · 2026-10' },
    { type: 'tool_result', name: 'query_transactions', ok: true, summary: '1 lançamento' },
    { type: 'text', delta: '\n\nEncontrei 1 lançamento (UBER *TRIP). ' },
    {
      type: 'tool_call',
      name: 'categorize_transactions',
      summary: '1 lançamento → Transporte',
    },
    {
      type: 'proposal',
      proposal_id: 'pending',
      tool: 'categorize_transactions',
      command: 'transactions.set_category',
    },
    {
      type: 'tool_result',
      name: 'categorize_transactions',
      ok: true,
      summary: 'proposta criada (aguarda aprovação)',
    },
    {
      type: 'text',
      delta: /markdown/i.test(message)
        ? MARKDOWN_SAMPLE
        : 'Propus a categoria Transporte; aprove na fila.',
    },
    {
      type: 'usage',
      engine_id: engine.id,
      input_tokens: 3800 + message.length,
      output_tokens: 420,
      cost_micros: engine.kind === 'api' ? 17_700 : 0,
      currency: 'USD',
      priced: true,
    },
  ];
  // A proposta só nasce quando o evento é revelado (como no núcleo).
  const at = script.findIndex((e) => e.type === 'proposal');
  effects.set(at, () => {
    const id = addProposalFromChat();
    const ev = script[at] as Extract<JobEventKind, { type: 'proposal' }>;
    ev.proposal_id = id;
  });
  return { script, effects };
}

// --- MCPs extras (`ai.mcp_extra.*`) ----------------------------------------------------

/** Servidor cadastrado; dos segredos fica só a dica (o valor nunca fica no mock). */
interface StoredMcp {
  server: McpServer;
}

let mcpServers: StoredMcp[] = [];
/** MCPs das CLIs liberados para o agente (Ajustes › IA). */
let cliMcpsAllowed: string[] = [];
const MCP_NAME = /^[a-z][a-z0-9_]{1,23}$/;

function mcpVars(input: readonly McpVarInput[], previous: readonly McpVar[]): McpVar[] {
  return input.map((v) => {
    const key = v.key.trim();
    if (!key) mockFail('invalid', 'variável inválida ou repetida: (vazia)');
    if (!v.secret) return { key, value: v.value ?? '', secret: false, set: false, hint: null };
    const value = v.value?.trim() ?? '';
    if (value) return { key, value: null, secret: true, set: true, hint: `…${value.slice(-4)}` };
    const old = previous.find((p) => p.key === key && p.secret && p.set);
    if (!old) mockFail('invalid', `informe o valor secreto de ${key}`);
    return { ...old };
  });
}

function saveMcp(a: Args): McpServer {
  const i = a as unknown as McpServerInput;
  if (!MCP_NAME.test(i.name) || i.name === 'treasury')
    mockFail(
      'invalid',
      'o nome do MCP precisa ter de 2 a 24 letras minúsculas, números ou _, começar por letra e não ser "treasury"',
    );
  if (!i.label?.trim()) mockFail('invalid', 'informe um rótulo de até 60 caracteres');
  if (i.transport === 'stdio' && !i.command?.trim())
    mockFail('invalid', 'informe o comando que inicia o servidor MCP');
  if (
    i.transport === 'http' &&
    !/^(https:\/\/|http:\/\/(localhost|127\.0\.0\.1|\[::1\]))/.test(i.url?.trim() ?? '')
  )
    mockFail(
      'invalid',
      'o endereço precisa começar com https:// (ou http://localhost para um servidor local)',
    );
  const prev = i.id ? mcpServers.find((m) => m.server.id === i.id) : undefined;
  if (i.id && !prev) mockFail('not_found', 'MCP extra não encontrado');
  if (mcpServers.some((m) => m.server.name === i.name && m.server.id !== i.id))
    mockFail('invalid', 'já existe um MCP com esse nome neste cofre');
  const stdio = i.transport === 'stdio';
  const server: McpServer = {
    id: prev?.server.id ?? newId('mcpx'),
    name: i.name,
    label: i.label.trim(),
    transport: i.transport,
    command: stdio ? (i.command?.trim() ?? null) : null,
    args: stdio ? i.args : [],
    url: stdio ? null : (i.url?.trim() ?? null),
    env: stdio ? mcpVars(i.env, prev?.server.env ?? []) : [],
    headers: stdio ? [] : mcpVars(i.headers, prev?.server.headers ?? []),
    enabled: i.enabled,
    auto_approve: i.auto_approve,
    created_at: prev?.server.created_at ?? now(),
    updated_at: now(),
  };
  if (prev) prev.server = server;
  else mcpServers.push({ server });
  return structuredClone(server);
}

function findMcp(id: string): StoredMcp {
  const m = mcpServers.find((x) => x.server.id === id);
  if (!m) mockFail('not_found', 'MCP extra não encontrado');
  return m;
}

function testMcp(a: Args): McpTestResult {
  const m = findMcp(str(a, 'id'));
  return {
    server_info: `${m.server.name}-mock 0.1.0`,
    tools: [
      { name: 'abrir_app', description: 'Abre um aplicativo no aparelho (exemplo)' },
      { name: 'ler_notificacoes', description: 'Lista as notificações recentes (exemplo)' },
    ],
  };
}

/** Pedido ao "celular" com um MCP extra que pede confirmação: o job espera a resposta. */
function externalScript(
  engine: Engine,
  message: string,
): { script: JobEventKind[]; effects: Map<number, () => void>; gate: MockJob['gate'] } | null {
  if (!/celular|telefone|phone|m[óo]vil|tel[ée]fono/i.test(message)) return null;
  const server = mcpServers.map((m) => m.server).find((s) => s.enabled && !s.auto_approve);
  if (!server) return null;
  const requestId = newId('mcpx-req');
  const args = '{"app": "Agenda"}';
  const script: JobEventKind[] = [
    {
      type: 'engine',
      engine_id: engine.id,
      engine_name: engine.name,
      kind: engine.kind,
      provider: engine.provider,
      model: engine.model,
    },
    { type: 'text', delta: 'Vou abrir a agenda no aparelho.' },
    { type: 'tool_call', name: `${server.name}-abrir_app`, summary: args },
    {
      type: 'external_approval',
      request_id: requestId,
      server: server.label,
      tool: 'abrir_app',
      args,
    },
    { type: 'external_decision', request_id: requestId, allowed: false, remembered: false },
    { type: 'tool_result', name: `${server.name}-abrir_app`, ok: false, summary: 'recusada' },
    { type: 'text', delta: '' },
    {
      type: 'usage',
      engine_id: engine.id,
      input_tokens: 2100 + message.length,
      output_tokens: 120,
      cost_micros: engine.kind === 'api' ? 6_900 : 0,
      currency: 'USD',
      priced: true,
    },
  ];
  const gate: NonNullable<MockJob['gate']> = { at: 3, requestId, allowed: null };
  const effects = new Map<number, () => void>([
    [
      4,
      () => {
        const ev = script[4] as Extract<JobEventKind, { type: 'external_decision' }>;
        ev.allowed = gate.allowed === true;
        ev.remembered = gate.allowed === true && gate.remember === true;
      },
    ],
    [
      5,
      () => {
        const ev = script[5] as Extract<JobEventKind, { type: 'tool_result' }>;
        ev.ok = gate.allowed === true;
        ev.summary = gate.allowed ? server.label : 'recusada';
      },
    ],
    [
      6,
      () => {
        (script[6] as Extract<JobEventKind, { type: 'text' }>).delta = gate.allowed
          ? '\n\nAbri a agenda no aparelho (simulado).'
          : '\n\nTudo bem, não usei o aparelho.';
      },
    ],
  ]);
  return { script, effects, gate };
}

/** O agente pergunta qual conta usar e espera a resposta (`ai.question.answer`). */
function questionScript(
  engine: Engine,
  message: string,
): { script: JobEventKind[]; effects: Map<number, () => void>; ask: AskGate } | null {
  if (!/pergunte|ask me|preg[uú]ntame/i.test(message)) return null;
  const multi = /v[áa]rias|several|varias/i.test(message);
  const question: AskQuestion = multi
    ? {
        question: 'Quais contas devo incluir no resumo?',
        options: [
          { label: 'Conta corrente', description: 'Movimento do dia a dia' },
          { label: 'Poupança', description: null },
          { label: 'Cartão', description: 'Fatura aberta' },
        ],
        recommended: null,
        multi_select: true,
        allow_other: false,
      }
    : {
        question: 'Qual conta devo usar para o aluguel?',
        options: [
          { label: 'Conta corrente', description: 'Onde cai o salário' },
          { label: 'Poupança', description: 'Reserva da família' },
          { label: 'Cartão', description: null },
        ],
        recommended: 0,
        multi_select: false,
        allow_other: true,
      };
  const requestId = newId('ask-req');
  const script: JobEventKind[] = [
    {
      type: 'engine',
      engine_id: engine.id,
      engine_name: engine.name,
      kind: engine.kind,
      provider: engine.provider,
      model: engine.model,
    },
    { type: 'tool_call', name: 'ask_user', summary: question.question },
    { type: 'user_question', request_id: requestId, question },
    { type: 'user_answer', request_id: requestId, answer: null },
    { type: 'tool_result', name: 'ask_user', ok: true, summary: 'respondida' },
    { type: 'text', delta: '' },
    {
      type: 'usage',
      engine_id: engine.id,
      input_tokens: 1800 + message.length,
      output_tokens: 90,
      cost_micros: engine.kind === 'api' ? 5_400 : 0,
      currency: 'USD',
      priced: true,
    },
  ];
  const ask: AskGate = { at: 2, requestId, question, answer: undefined };
  const chosen = () => {
    const a = ask.answer;
    return a ? [...a.selected, ...(a.other ? [a.other] : [])].join(', ') : '';
  };
  const effects = new Map<number, () => void>([
    [
      3,
      () => {
        (script[3] as Extract<JobEventKind, { type: 'user_answer' }>).answer = ask.answer ?? null;
      },
    ],
    [
      5,
      () => {
        (script[5] as Extract<JobEventKind, { type: 'text' }>).delta = ask.answer
          ? `Combinado: vou usar ${chosen()}.`
          : 'Sem resposta; não escolhi nada.';
      },
    ],
  ]);
  return { script, effects, ask };
}

/** Espelho de `ai::ask::check`: rótulos que existem, uma só sem múltipla, "Outro" se permitido. */
function answerQuestion(a: Args): null {
  const requestId = str(a, 'request_id');
  const job = [...state.jobs.values()].find((j) => j.ask?.requestId === requestId);
  if (!job?.ask || job.ask.answer !== undefined || job.status !== 'running')
    mockFail('not_found', 'essa pergunta já foi respondida ou expirou');
  const q = job.ask.question;
  const raw = Array.isArray(a.selected) ? (a.selected as unknown[]) : [];
  const selected = [...new Set(raw.map((x) => String(x).trim()).filter(Boolean))];
  const unknown = selected.find((l) => !q.options.some((o) => o.label === l));
  if (unknown) mockFail('invalid', `opção desconhecida: ${unknown}`);
  const other = optStr(a, 'other')?.trim() || null;
  if (other && !q.allow_other) mockFail('invalid', 'esta pergunta não aceita "Outro"');
  if (selected.length + (other ? 1 : 0) === 0) mockFail('invalid', 'escolha uma opção');
  if (!q.multi_select && selected.length + (other ? 1 : 0) > 1)
    mockFail('invalid', 'esta pergunta aceita uma opção só');
  job.ask.answer = { selected, other };
  return null;
}

function decideExternal(a: Args): null {
  const requestId = str(a, 'request_id');
  const job = [...state.jobs.values()].find((j) => j.gate?.requestId === requestId);
  if (!job?.gate || job.gate.allowed != null || job.status !== 'running')
    mockFail('not_found', 'esse pedido de confirmação já foi respondido ou expirou');
  job.gate.allowed = a.allow === true;
  job.gate.remember = a.scope === 'conversation';
  return null;
}

/**
 * Espelho de `ai::activity::split`: texto seguido de ferramenta é passo intermediário (vai
 * para o raciocínio); o que vem depois da última ferramenta é a resposta.
 */
function splitReply(events: JobEventKind[]): {
  answer: string;
  reasoning: string;
  tools: Message['tool_calls'];
} {
  const steps: string[] = [];
  const tools: Message['tool_calls'] = [];
  let text = '';
  let thinking = '';
  const flush = () => {
    if (thinking.trim()) steps.push(thinking.trim());
    thinking = '';
  };
  for (const e of events) {
    if (e.type !== 'reasoning') flush();
    if (e.type === 'reasoning') thinking += e.delta;
    else if (e.type === 'text') text += e.delta;
    else if (e.type === 'tool_call') {
      if (text.trim()) steps.push(text.trim());
      text = '';
      tools.push({ name: e.name, args: e.summary, summary: '', ok: false });
    } else if (e.type === 'tool_result') {
      const open = [...tools].reverse().find((t) => t.name === e.name && !t.summary && !t.ok);
      if (open) {
        open.ok = e.ok;
        open.summary = e.summary;
      } else tools.push({ name: e.name, args: '', summary: e.summary, ok: e.ok });
    } else if (e.type === 'fallback') text = '';
  }
  flush();
  let answer = text.trim();
  if (!answer && steps.length) answer = steps.pop()!;
  return { answer, reasoning: steps.join('\n\n'), tools };
}

// --- Memória de procedimentos do agente (`ai.procedures.*`) -----------------------------

function procedure(
  id: string,
  topic: ProcedureTopic,
  title: string,
  steps: string,
  extra: Partial<Procedure>,
  minutes: number,
): Procedure {
  return {
    id,
    topic,
    title,
    steps,
    site: null,
    file_format: null,
    institution_id: null,
    institution_name: null,
    account_id: null,
    account_name: null,
    asset_id: null,
    asset_name: null,
    created_by: 'ai',
    client: 'agente (CLI de teste)',
    conversation_id: null,
    conversation_title: null,
    created_at: ago(minutes),
    updated_at: ago(minutes),
    worked: 0,
    failed: 0,
    last_used_at: null,
    last_outcome: null,
    last_note: null,
    ...extra,
  };
}

function seedProcedures(): Procedure[] {
  return [
    procedure(
      'mock-ai-proc-itau',
      'statement',
      'Extrato OFX da conta corrente Itaú',
      'Entrar no internet banking › Conta corrente › Extrato › período de 90 dias › Exportar › OFX. O arquivo vem com o saldo final.',
      {
        site: 'www.itau.com.br',
        file_format: 'OFX',
        institution_name: 'Itaú',
        account_name: 'Itaú Corrente',
        conversation_id: 'mock-ai-conv-1',
        conversation_title: 'Restaurantes em setembro',
        worked: 3,
        last_used_at: ago(60 * 30),
        last_outcome: 'worked',
      },
      60 * 24 * 20,
    ),
    procedure(
      'mock-ai-proc-b3',
      'investments',
      'Posição e negociações na Área do Investidor B3',
      'investidor.b3.com.br › Extratos › Negociação › filtrar o mês › baixar Excel. Movimentação traz proventos.',
      {
        site: 'www.investidor.b3.com.br',
        file_format: 'XLSX',
        worked: 1,
        failed: 1,
        last_used_at: ago(60 * 24 * 2),
        last_outcome: 'failed',
        last_note: 'O botão de baixar agora fica no menu ⋯ do relatório.',
      },
      60 * 24 * 45,
    ),
  ];
}

function normalized(t: string): string {
  return t
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
}

function listProcedures(a: Args): Procedure[] {
  const words = normalized(optStr(a, 'query') ?? '')
    .split(/\s+/)
    .filter(Boolean);
  const topic = optStr(a, 'topic');
  return state.procedures
    .filter((p) => !topic || p.topic === topic)
    .filter((p) => {
      const hay = normalized(
        [p.title, p.steps, p.site, p.institution_name, p.account_name, p.asset_name]
          .filter(Boolean)
          .join(' '),
      );
      return words.every((w) => hay.includes(w));
    })
    .sort((x, y) => y.updated_at.localeCompare(x.updated_at))
    .slice(0, optNum(a, 'limit') ?? 50);
}

function findProcedure(id: string): Procedure {
  const p = state.procedures.find((x) => x.id === id);
  if (!p) mockFail('not_found', 'procedimento não encontrado');
  return p;
}

function saveProcedure(a: Args) {
  const id = optStr(a, 'id');
  const existing = id ? findProcedure(id) : undefined;
  const text = (name: string, max: number) => {
    const v = a[name];
    if (v === undefined) return undefined;
    if (typeof v !== 'string' || !v.trim()) mockFail('invalid', `informe \`${name}\``);
    if (v.trim().length > max)
      mockFail('invalid', `\`${name}\` pode ter no máximo ${max} caracteres`);
    return v.trim();
  };
  const title = text('title', 120) ?? existing?.title;
  const steps = text('steps', 2000) ?? existing?.steps;
  const topic = (optStr(a, 'topic') as ProcedureTopic | undefined) ?? existing?.topic;
  if (!topic) mockFail('invalid', 'informe `topic`');
  if (!title) mockFail('invalid', 'informe `title`');
  if (!steps) mockFail('invalid', 'informe `steps`');
  const opt = (name: 'site' | 'file_format') =>
    name in a ? (a[name] as string | null)?.trim() || null : (existing?.[name] ?? null);
  let site = opt('site');
  if (site) {
    site = site
      .replace(/^https?:\/\//, '')
      .split(/[/?#]/)[0]!
      .toLowerCase();
    if (!/^[a-z0-9.-]+\.[a-z]{2,}$/.test(site))
      mockFail('invalid', `site inválido: ${site} (informe o domínio, ex.: itau.com.br)`);
  }
  const fields = {
    topic,
    title,
    steps,
    site,
    file_format: opt('file_format')?.toUpperCase() ?? null,
    updated_at: now(),
  };
  if (existing) {
    Object.assign(existing, fields);
    return { id: existing.id, created: false, redactions: 0, message: 'Nota atualizada.' };
  }
  const rec = procedure(
    newId('proc'),
    topic,
    title,
    steps,
    { ...fields, created_by: 'user', client: null },
    0,
  );
  state.procedures.unshift(rec);
  return { id: rec.id, created: true, redactions: 0, message: 'Gravado.' };
}

function estimateMicros(message: string): number {
  return 30_000 + message.length * 50;
}

function chatStart(a: Args): ChatStart {
  chatStarts.push({ ...a });
  const typed = str(a, 'message').trim();
  const attIds = Array.isArray(a.attachment_ids) ? (a.attachment_ids as string[]) : [];
  const atts: AttachmentInfo[] = attIds.map((id) => {
    const x = staged.get(id);
    if (!x) mockFail('not_found', 'anexo não encontrado (prepare de novo)');
    return x;
  });
  if (!typed && atts.length === 0) mockFail('invalid', 'escreva a mensagem');
  const message = typed || '(só anexos)';
  if (a.page_context !== undefined) {
    const ctx = a.page_context;
    if (ctx === null || typeof ctx !== 'object' || Array.isArray(ctx))
      mockFail('invalid', 'o contexto da tela precisa ser um objeto JSON');
    if (JSON.stringify(ctx).length > 8000)
      mockFail('invalid', 'contexto da tela grande demais (máx. 8000 caracteres)');
    pageContexts.push(structuredClone(ctx));
  }
  const task = (optStr(a, 'task') as AiTask | undefined) ?? 'chat';
  const convId = optStr(a, 'conversation_id');
  const keep = state.settings.keep_history;
  const existing =
    convId && keep ? state.conversations.find((c) => c.conversation.id === convId) : undefined;
  if (convId && keep && !existing) mockFail('not_found', 'conversa não encontrada');
  const cands = routeCandidates(task, optStr(a, 'engine_id'));
  if (cands.length === 0)
    mockFail(
      'no_engine',
      'nenhum motor de IA configurado para esta tarefa; cadastre um em Ajustes › IA e agente',
    );
  const ready = cands.filter((e) => readiness(e) == null);
  const first = ready[0];
  if (a.confirm_cost !== true && first?.kind === 'api') {
    const th = state.limits.get(first.provider)?.confirm_above_micros;
    const est = estimateMicros(message);
    if (th != null && est > th) {
      return {
        status: 'confirm_required',
        job_id: null,
        conversation_id: existing?.conversation.id ?? null,
        estimate: {
          engine_id: first.id,
          engine_name: first.name,
          cost_micros: est,
          threshold_micros: th,
          currency: 'USD',
        },
      };
    }
  }
  let conv = existing;
  if (keep && !conv) {
    conv = {
      conversation: {
        id: newId('conv'),
        title: message.slice(0, 60),
        task,
        last_engine_id: null,
        created_at: now(),
        updated_at: now(),
        message_count: 0,
        pinned: false,
      },
      messages: [],
    };
    state.conversations.unshift(conv);
  }
  // Página do ativo (docs/asset-page.md): a conversa fica ligada a ele.
  const page = a.page_context as { screen?: string; selection?: { asset_id?: string } } | undefined;
  if (conv && page?.screen === 'ativo' && page.selection?.asset_id && !conv.conversation.asset_id)
    conv.conversation.asset_id = page.selection.asset_id;
  if (conv) {
    conv.messages.push({
      id: newId('msg'),
      role: 'user',
      content: typed,
      engine_id: null,
      engine_name: null,
      tool_calls: [],
      proposal_ids: [],
      reasoning: null,
      reasoning_ms: null,
      created_at: now(),
      attachments: atts.map((x) => ({
        ...x,
        stored: state.settings.attachment_retention !== 'none',
      })),
    });
  }
  const job: MockJob = {
    id: newId('job'),
    task,
    kind: 'chat',
    createdAt: now(),
    script: [],
    revealed: [],
    cursor: 0,
    outcome: 'done',
    status: 'running',
    conversationId: conv?.conversation.id ?? null,
    engineName: null,
    title: message.replace(/\s+/g, ' ').trim().slice(0, 120),
    effects: new Map(),
    onDone: null,
  };
  if (!first) {
    // Nenhum motor pronto: fallback por todos e erro.
    job.script = [
      ...cands.map((e): JobEventKind => ({
        type: 'fallback',
        engine_id: e.id,
        engine_name: e.name,
        reason: readiness(e) ?? 'falhou',
      })),
      {
        type: 'error',
        code: 'no_engine',
        message: 'Nenhum motor da rota está pronto (chave, modelo ou motor desligado).',
      },
    ];
    job.outcome = 'failed';
  } else {
    const skipped = cands.slice(0, cands.indexOf(first));
    const { script, effects, gate, ask } = chatScript(first, message);
    const pre: JobEventKind[] = skipped.map((e) => ({
      type: 'fallback',
      engine_id: e.id,
      engine_name: e.name,
      reason: readiness(e) ?? 'falhou',
    }));
    job.script = [...pre, ...script];
    for (const [i, fx] of effects) job.effects.set(i + pre.length, fx);
    if (gate) {
      // O mesmo objeto que os efeitos leem (a resposta chega por `ai.mcp_extra.decide`).
      gate.at += pre.length;
      job.gate = gate;
    }
    if (ask) {
      // Idem: a resposta chega por `ai.question.answer`.
      ask.at += pre.length;
      job.ask = ask;
    }
    job.engineName = first.name;
    job.onDone = () => {
      const msgId = newId('msg');
      if (conv) {
        const ev = job.revealed.map((r) => r.event);
        const parts = splitReply(ev);
        conv.messages.push({
          id: msgId,
          role: 'assistant',
          content: parts.answer,
          engine_id: first.id,
          engine_name: first.name,
          tool_calls: parts.tools,
          proposal_ids: ev
            .filter((e): e is Extract<JobEventKind, { type: 'proposal' }> => e.type === 'proposal')
            .map((e) => e.proposal_id),
          reasoning: parts.reasoning || null,
          reasoning_ms: parts.reasoning || parts.tools.length ? 6_000 : null,
          created_at: now(),
          attachments: [],
        });
        conv.conversation.updated_at = now();
        conv.conversation.last_engine_id = first.id;
        conv.conversation.message_count = conv.messages.length;
      }
      state.usage.push({
        at: now(),
        engine_id: first.id,
        task,
        input: 3800 + message.length,
        output: 420,
        cost: first.kind === 'api' ? 17_700 : 0,
        priced: true,
        ok: true,
      });
      return msgId;
    };
  }
  state.jobs.set(job.id, job);
  return {
    status: 'started',
    job_id: job.id,
    conversation_id: job.conversationId,
    estimate: null,
  };
}

function pushEvent(job: MockJob, event: JobEventKind) {
  job.revealed.push({ seq: job.revealed.length + 1, at: now(), event });
}

function finish(job: MockJob, status: JobStatus) {
  let messageId: string | null = null;
  if (status === 'done' && job.onDone) messageId = job.onDone() ?? null;
  job.status = status;
  job.finishedAt = now();
  pushEvent(job, {
    type: 'done',
    status,
    conversation_id: job.conversationId,
    message_id: messageId,
    alert_id: null,
  });
}

function chatPoll(a: Args): ChatPoll {
  const job = state.jobs.get(str(a, 'job_id'));
  if (!job) mockFail('not_found', 'job (encerrado há mais de 30 minutos?) não encontrado');
  const after = optNum(a, 'after_seq') ?? 0;
  if (job.status === 'running') {
    const end = Math.min(job.cursor + PER_POLL, job.script.length);
    while (job.cursor < end) {
      // Pedido de confirmação em aberto: nada sai até a resposta (`ai.mcp_extra.decide`).
      if (job.gate && job.cursor > job.gate.at && job.gate.allowed == null) break;
      // Pergunta em aberto: idem, até `ai.question.answer`.
      if (job.ask && job.cursor > job.ask.at && job.ask.answer === undefined) break;
      // O efeito pode ajustar o evento (id da proposta) antes de ele sair.
      job.effects.get(job.cursor)?.();
      pushEvent(job, { ...job.script[job.cursor]! });
      job.cursor += 1;
    }
    if (job.cursor >= job.script.length) finish(job, job.outcome);
  }
  const events = job.revealed.filter((e) => e.seq > after);
  return {
    job_id: job.id,
    status: job.status,
    events,
    next_seq: events[events.length - 1]?.seq ?? after,
    conversation_id: job.conversationId,
  };
}

function chatCancel(a: Args): null {
  const job = state.jobs.get(str(a, 'job_id'));
  if (!job) mockFail('not_found', 'job não encontrado');
  if (job.status === 'running') {
    job.onDone = null;
    // Como no núcleo: a pergunta em aberto fecha sem resposta.
    if (job.ask && job.ask.answer === undefined && job.cursor > job.ask.at) {
      job.ask.answer = null;
      pushEvent(job, { type: 'user_answer', request_id: job.ask.requestId, answer: null });
    }
    finish(job, 'cancelled');
  }
  return null;
}

// --- Fila de aprovação ------------------------------------------------------------------

function findProposal(id: string): StoredProposal {
  const p = state.proposals.find((x) => x.proposal.id === id);
  if (!p) mockFail('not_found', `proposta ${id} não encontrada`);
  return p;
}

function review(a: Args, status: 'approved' | 'rejected'): ReviewResult[] {
  const ids = a.ids;
  if (!Array.isArray(ids) || ids.length === 0)
    mockFail('invalid', 'selecione ao menos uma proposta');
  const found = (ids as string[]).map(findProposal);
  for (const p of found) {
    if (p.proposal.status !== 'pending')
      mockFail('conflict', `a proposta ${p.proposal.id} não está pendente`);
  }
  return found.map((p) => {
    p.proposal.status = status;
    p.proposal.reviewed_at = now();
    p.proposal.note = optStr(a, 'note') ?? null;
    if (status === 'approved' && p.proposal.command === 'ai.remember') {
      const after = p.items[0]?.after as { content?: string; kind?: MemoryItem['kind'] } | null;
      if (after?.content)
        state.memory.push(memory(newId('memory'), after.kind ?? 'note', after.content, 'ai', 0));
    }
    return { id: p.proposal.id, status, error: null };
  });
}

// --- Alertas e rotinas ------------------------------------------------------------------

function routinesStatus(): RoutineStatus[] {
  const labels: Record<Routine, string> = {
    review: 'Revisão contínua',
    monthly_close: 'Fechamento mensal',
    alerts: 'Alertas',
  };
  return (['review', 'monthly_close', 'alerts'] as const).map((r) => ({
    routine: r,
    label: labels[r],
    enabled: state.settings.routines[r],
    last_run: state.routinesLastRun[r] ?? null,
    running: [...state.jobs.values()].some(
      (j) => j.kind === 'routine' && j.task === routineTask(r) && j.status === 'running',
    ),
    engine_routed: (state.routes.get(routineTask(r)) ?? []).length > 0,
  }));
}

function routineTask(r: Routine): AiTask {
  return r === 'review' ? 'categorize' : r === 'monthly_close' ? 'monthly_close' : 'alerts';
}

const ROUTINE_TITLE: Partial<Record<string, string>> = {
  monthly_close: 'Fechamento mensal',
  alerts: 'Alertas',
  categorize: 'Categorização',
};

/** Etapa atual do job, como o núcleo: a última ferramenta, progresso ou proposta. */
function jobActivity(job: MockJob): string | null {
  for (let i = job.revealed.length - 1; i >= 0; i--) {
    const e = job.revealed[i]!.event;
    if (e.type === 'tool_call') return e.summary || `Usando ${e.name}`;
    if (e.type === 'progress') return e.message;
    if (e.type === 'proposal') return 'Proposta aguardando aprovação';
    if (e.type === 'text') return 'Escrevendo a resposta';
  }
  return null;
}

function runRoutine(a: Args): { job_id: string } {
  const routine = str(a, 'routine') as Routine;
  const job: MockJob = {
    id: newId('job'),
    task: routineTask(routine),
    kind: 'routine',
    createdAt: now(),
    script: [
      { type: 'progress', message: 'Lendo os dados do mês' },
      { type: 'progress', message: 'Calculando' },
    ],
    revealed: [],
    cursor: 0,
    outcome: 'done',
    status: 'running',
    conversationId: null,
    engineName: null,
    title: ROUTINE_TITLE[routine] ?? routine,
    effects: new Map(),
    onDone: () => {
      state.routinesLastRun[routine] = now();
      return undefined;
    },
  };
  state.jobs.set(job.id, job);
  // Rotinas do mock terminam na hora (a tela só acompanha o estado).
  for (const ev of job.script) pushEvent(job, ev);
  job.cursor = job.script.length;
  finish(job, 'done');
  return { job_id: job.id };
}

/** Job de rotina já terminado (outros mocks, como o estudo com IA, acompanham pelo poll). */
export function mockFinishedJob(
  title: string,
  task: AiTask,
  onDone: () => JobLink | undefined,
): string {
  const job: MockJob = {
    id: newId('job'),
    task,
    kind: 'routine',
    createdAt: now(),
    script: [{ type: 'progress', message: title }],
    revealed: [],
    cursor: 0,
    outcome: 'done',
    status: 'running',
    conversationId: null,
    engineName: null,
    title,
    effects: new Map(),
    onDone: () => {
      job.link = onDone();
      return undefined;
    },
  };
  state.jobs.set(job.id, job);
  for (const ev of job.script) pushEvent(job, ev);
  job.cursor = job.script.length;
  finish(job, 'done');
  return job.id;
}

// --- Registro ---------------------------------------------------------------------------

function guard(h: MockHandler): MockHandler {
  return (args) => {
    requireUnlocked();
    return h(args ?? {});
  };
}

function register(map: Record<string, MockHandler>, fallbackOnly = false) {
  const entries = Object.entries(map).filter(([k]) => !fallbackOnly || !extraMockHandler(k));
  registerMockHandlers(Object.fromEntries(entries.map(([k, h]) => [k, guard(h)])));
}

function itemsOf(p: StoredProposal): ChangeItem[] {
  if (p.fill) {
    p.items = p.fill();
    p.fill = undefined;
  }
  return p.items;
}

const detail = (p: StoredProposal): ProposalDetail => ({ ...p.proposal, items: itemsOf(p) });

register({
  // Fila de aprovação
  'ai.list_proposals': (a) => {
    const status = optStr(a, 'status');
    const limit = optNum(a, 'limit') ?? 100;
    return state.proposals
      .filter((p) => !status || p.proposal.status === status)
      .sort((x, y) => y.proposal.created_at.localeCompare(x.proposal.created_at))
      .slice(0, limit)
      .map((p) => p.proposal);
  },
  'ai.get_proposal': (a) => detail(findProposal(str(a, 'id'))),
  'ai.pending_count': () => state.proposals.filter((p) => p.proposal.status === 'pending').length,
  'ai.approve': (a) => review(a, 'approved'),
  'ai.reject': (a) => review(a, 'rejected'),
  // Memória
  'ai.memory': () => [...state.memory],
  'ai.remember': (a) => {
    const content = str(a, 'content').trim();
    const item = memory(newId('memory'), str(a, 'kind') as MemoryItem['kind'], content, 'user', 0);
    state.memory.push(item);
    return { id: item.id };
  },
  'ai.forget': (a) => {
    const id = str(a, 'id');
    if (!state.memory.some((m) => m.id === id)) mockFail('not_found', 'nota não encontrada');
    state.memory = state.memory.filter((m) => m.id !== id);
    return null;
  },
  'ai.procedures.list': (a) => listProcedures(a),
  'ai.procedures.get': (a) => findProcedure(str(a, 'id')),
  'ai.procedures.save': (a) => saveProcedure(a),
  'ai.procedures.delete': (a) => {
    const id = str(a, 'id');
    findProcedure(id);
    state.procedures = state.procedures.filter((p) => p.id !== id);
    return null;
  },
  // Motores
  'ai.engines.providers': () => PROVIDERS,
  'ai.engines.list': () => state.engines.map(toEngine),
  'ai.engines.get': (a) => toEngine(findEngine(str(a, 'id'))),
  'ai.engines.create': (a) => createEngine(a as unknown as EngineInput),
  'ai.engines.update': updateEngine,
  'ai.engines.delete': (a) => {
    const id = findEngine(str(a, 'id')).id;
    state.engines = state.engines.filter((e) => e.id !== id);
    state.secrets.delete(id);
    for (const [t, ids] of state.routes)
      state.routes.set(
        t,
        ids.filter((x) => x !== id),
      );
    return null;
  },
  'ai.set_secret': (a) => {
    const id = findEngine(str(a, 'engine_id')).id;
    setSecret(id, str(a, 'value'));
    return toEngine(findEngine(id)).secret;
  },
  'ai.delete_secret': (a) => {
    state.secrets.delete(str(a, 'engine_id'));
    return { set: false, hint: null, needs_reentry: false, updated_at: null };
  },
  'ai.detect_clis': (): CliDetection[] => [
    {
      provider: 'claude_code',
      label: 'Claude Code (CLI)',
      binary: 'claude',
      path: '/usr/local/bin/claude',
      version: '2.1.0',
      logged_in: true,
      error: null,
    },
    {
      provider: 'codex',
      label: 'OpenAI Codex (CLI)',
      binary: 'codex',
      path: null,
      version: null,
      logged_in: null,
      error: null,
    },
    {
      provider: 'gemini_cli',
      label: 'Gemini CLI',
      binary: 'gemini',
      path: '/opt/homebrew/bin/gemini',
      version: '0.9.1',
      logged_in: null,
      error: null,
    },
    {
      provider: 'grok_cli',
      label: 'Grok (CLI)',
      binary: 'grok',
      path: null,
      version: null,
      logged_in: null,
      error: null,
    },
    {
      provider: 'opencode_cli',
      label: 'OpenCode (CLI)',
      binary: 'opencode',
      path: '/opt/homebrew/bin/opencode',
      version: 'opencode v2.0.32',
      logged_in: true,
      error: null,
    },
  ],
  'ai.test_engine': (a): EngineTest => {
    const e = toEngine(findEngine(str(a, 'engine_id')));
    const problem = readiness(e);
    return {
      engine_id: e.id,
      ok: problem == null,
      latency_ms: problem == null ? 640 : 0,
      reply: problem == null ? 'OK' : null,
      error: problem,
      input_tokens: problem == null ? 12 : 0,
      output_tokens: problem == null ? 2 : 0,
    };
  },
  // Roteamento, limites, preços e uso
  'ai.routes.list': (): TaskRoute[] =>
    TASKS.map((t) => ({ task: t, label: TASK_LABEL[t], engine_ids: state.routes.get(t) ?? [] })),
  'ai.routes.set': (a): TaskRoute => {
    const task = str(a, 'task') as AiTask;
    const ids = (a.engine_ids as string[] | undefined) ?? [];
    if (new Set(ids).size !== ids.length)
      mockFail('invalid', 'o mesmo motor aparece duas vezes na lista');
    ids.forEach(findEngine);
    state.routes.set(task, [...ids]);
    return { task, label: TASK_LABEL[task], engine_ids: [...ids] };
  },
  'ai.limits.list': () =>
    [...state.limits.values()].sort((x, y) => (x.provider ?? '').localeCompare(y.provider ?? '')),
  'ai.limits.set': (a): SpendLimit => {
    const provider = str(a, 'provider') as Provider;
    if (providerInfo(provider).kind !== 'api')
      mockFail(
        'invalid',
        'limites de gasto valem para providers de API (CLIs e modelos locais não têm custo no app)',
      );
    const v = (k: string) => (typeof a[k] === 'number' ? a[k] : null);
    for (const k of ['daily_micros', 'monthly_micros', 'confirm_above_micros']) {
      if ((v(k) ?? 0) < 0) mockFail('invalid', 'limites não podem ser negativos');
    }
    const limit: SpendLimit = {
      provider,
      daily_micros: v('daily_micros'),
      monthly_micros: v('monthly_micros'),
      confirm_above_micros: v('confirm_above_micros'),
    };
    state.limits.set(provider, limit);
    return limit;
  },
  'ai.prices.list': () =>
    [...state.prices].sort(
      (x, y) => x.provider.localeCompare(y.provider) || x.model.localeCompare(y.model),
    ),
  'ai.prices.set': (a): ModelPrice => {
    const provider = str(a, 'provider') as Provider;
    const model = str(a, 'model').trim();
    const input = optNum(a, 'input_micros_per_mtok') ?? -1;
    const output = optNum(a, 'output_micros_per_mtok') ?? -1;
    if (input < 0 || output < 0) mockFail('invalid', 'preços não podem ser negativos');
    const p: ModelPrice = {
      ...price(provider, model, input, output, optNum(a, 'cached_micros_per_mtok') ?? null),
      source: 'user',
      updated_at: now(),
    };
    state.prices = [
      ...state.prices.filter((x) => !(x.provider === provider && x.model === model)),
      p,
    ];
    return p;
  },
  'ai.prices.delete': (a) => {
    const provider = str(a, 'provider');
    const model = str(a, 'model');
    state.prices = state.prices.filter((x) => !(x.provider === provider && x.model === model));
    return null;
  },
  'ai.usage_summary': usageSummary,
  // Ajustes
  'ai.settings.get': () => structuredClone(state.settings),
  'ai.settings.set': (a) => {
    const p = a as AiSettingsPatch;
    state.settings = {
      ...state.settings,
      ...(typeof p.autonomous === 'boolean' ? { autonomous: p.autonomous } : {}),
      ...(typeof p.mcp_allow_writes === 'boolean' ? { mcp_allow_writes: p.mcp_allow_writes } : {}),
      ...(typeof p.keep_history === 'boolean' ? { keep_history: p.keep_history } : {}),
      ...(p.attachment_retention ? { attachment_retention: p.attachment_retention } : {}),
      ...(typeof p.history_retention_days === 'number'
        ? { history_retention_days: p.history_retention_days }
        : {}),
      ...(typeof p.cli_timeout_minutes === 'number'
        ? { cli_timeout_minutes: p.cli_timeout_minutes }
        : {}),
      routines: { ...state.settings.routines, ...(p.routines ?? {}) },
    };
    return structuredClone(state.settings);
  },
  // Conversas e jobs
  'ai.suggestions': suggestions,
  'ai.chat_start': chatStart,
  'ai.chat_poll': chatPoll,
  'ai.chat_cancel': chatCancel,
  'ai.jobs.list': (): JobInfo[] =>
    [...state.jobs.values()]
      .map((j) => ({
        job_id: j.id,
        kind: j.kind,
        task: j.task,
        status: j.status,
        created_at: j.createdAt,
        conversation_id: j.conversationId,
        engine_name: j.engineName,
        title: j.title ?? null,
        activity: jobActivity(j),
        finished_at: j.finishedAt ?? null,
        cancelling: false,
        link: j.link ?? null,
      }))
      .sort((x, y) => y.created_at.localeCompare(x.created_at)),
  'ai.conversations.list': (a) =>
    state.conversations
      .filter((c) => !optStr(a, 'asset_id') || c.conversation.asset_id === optStr(a, 'asset_id'))
      // Cópias, como o núcleo serializado: o cache não pode ver a fixação mudar sozinho.
      .map((c) => ({ ...c.conversation }))
      .sort((x, y) => y.updated_at.localeCompare(x.updated_at)),
  'ai.conversations.get': (a): ConversationDetail => {
    const c = state.conversations.find((x) => x.conversation.id === str(a, 'id'));
    if (!c) mockFail('not_found', 'conversa não encontrada');
    return { ...c.conversation, messages: [...c.messages] };
  },
  // O núcleo monta o Markdown/PDF sem segredos e grava no caminho; o mock só confere.
  'ai.conversations.export': (a) => {
    const c = state.conversations.find((x) => x.conversation.id === str(a, 'id'));
    if (!c) mockFail('not_found', 'conversa não encontrada');
    const format = str(a, 'format');
    if (format !== 'markdown' && format !== 'pdf') mockFail('invalid', 'formato inválido');
    return { path: str(a, 'path'), bytes: 1024, pages: format === 'pdf' ? 1 : 0, redacted: 0 };
  },
  'ai.attachments.prepare': (a): AttachmentInfo => {
    const name = str(a, 'name');
    const data = str(a, 'data');
    const mime = optStr(a, 'mime') ?? '';
    const ext = name.toLowerCase().split('.').pop() ?? '';
    const size = Math.floor((data.length * 3) / 4);
    if (size > 20 * 1024 * 1024) mockFail('invalid', `${name}: arquivo grande demais (máx. 20 MB)`);
    const kind =
      mime.startsWith('image/') || ['png', 'jpg', 'jpeg', 'webp', 'gif'].includes(ext)
        ? 'image'
        : mime === 'application/pdf' || ext === 'pdf'
          ? 'pdf'
          : ['txt', 'md', 'csv', 'json', 'ofx', 'xml', 'tsv', 'xlsx', 'docx'].includes(ext) ||
              mime.startsWith('text/')
            ? 'text'
            : mockFail('unsupported', `${name}: tipo de arquivo não suportado`);
    const info: AttachmentInfo = {
      id: newId('att'),
      name,
      mime: mime || 'application/octet-stream',
      kind,
      size,
      sha256: `mock-${size}`,
      width: kind === 'image' ? 800 : null,
      height: kind === 'image' ? 600 : null,
      pages: kind === 'pdf' ? 2 : null,
      text_chars: kind === 'image' ? 0 : Math.min(size, 60_000),
      truncated: false,
      needs_native: kind === 'image',
      stored: false,
      warnings: [],
    };
    // Imagem com "comprovante" no nome: o OCR local leu o texto (mock).
    if (kind === 'image' && name.toLowerCase().includes('comprovante')) {
      info.text_chars = 240;
      info.needs_native = false;
      info.ocr_confidence = 0.93;
    }
    staged.set(info.id, info);
    return info;
  },
  'ai.attachments.discard': (a) => {
    for (const id of (a.ids as string[] | undefined) ?? []) staged.delete(id);
    return null;
  },
  'ai.conversations.pin': (a) => {
    const c = state.conversations.find((x) => x.conversation.id === str(a, 'id'));
    if (!c) mockFail('not_found', 'conversa não encontrada');
    c.conversation.pinned = a.pinned === true;
    return null;
  },
  'ai.history.stats': (): HistoryStats => {
    const messages = state.conversations.reduce((n, c) => n + c.messages.length, 0);
    return {
      conversations: state.conversations.length,
      pinned: state.conversations.filter((c) => c.conversation.pinned).length,
      messages,
      attachments: state.conversations.reduce(
        (n, c) => n + c.messages.reduce((k, m) => k + m.attachments.length, 0),
        0,
      ),
      stored_attachments: 0,
      attachment_bytes: 0,
      history_bytes: messages * 400,
      db_bytes: 8_400_000,
      free_bytes: 120_000,
      last_cleanup: mockLastCleanup,
    };
  },
  'ai.history.cleanup': () => {
    mockLastCleanup = new Date().toISOString();
    const days = state.settings.history_retention_days;
    const before = state.conversations.length;
    if (days > 0) {
      const limit = Date.now() - days * 86_400_000;
      state.conversations = state.conversations.filter(
        (c) => c.conversation.pinned || Date.parse(c.conversation.updated_at) >= limit,
      );
    }
    return {
      conversations_deleted: before - state.conversations.length,
      messages_deleted: 0,
      attachments_expired: 0,
      vacuumed: false,
      db_bytes_before: 8_400_000,
      db_bytes_after: 8_400_000,
    };
  },
  'ai.conversations.delete': (a) => {
    const id = str(a, 'id');
    state.conversations = state.conversations.filter((c) => c.conversation.id !== id);
    return null;
  },
  'ai.conversations.clear': () => {
    const deleted = state.conversations.length;
    state.conversations = [];
    return { deleted };
  },
  // Alertas e rotinas
  'ai.alerts.list': (a) => {
    const status = optStr(a, 'status');
    const kind = optStr(a, 'kind');
    return state.alerts
      .filter((x) => (status ? x.status === status : x.status !== 'resolved'))
      .filter((x) => !kind || x.kind === kind)
      .slice(0, optNum(a, 'limit') ?? 50);
  },
  'ai.alerts.count': () => ({
    new: state.alerts.filter((x) => x.status === 'new').length,
    seen: state.alerts.filter((x) => x.status === 'seen').length,
  }),
  'ai.alerts.set_status': (a) => {
    const ids = (a.ids as string[] | undefined) ?? [];
    const status = str(a, 'status') as AlertStatus;
    const changed = state.alerts.filter((x) => ids.includes(x.id));
    for (const x of changed) {
      x.status = status;
      x.updated_at = now();
      if (status !== 'new') x.seen_at ??= now();
      x.resolved_at = status === 'resolved' ? now() : null;
    }
    return changed.map((x) => ({ ...x }));
  },
  'ai.alerts.delete': (a) => {
    const ids = (a.ids as string[] | undefined) ?? [];
    const before = state.alerts.length;
    state.alerts = state.alerts.filter((x) => !ids.includes(x.id));
    return { deleted: before - state.alerts.length };
  },
  'ai.routines.status': routinesStatus,
  'ai.routines.run': runRoutine,
  // MCPs que as CLIs do usuário carregam (modo de demonstração: dois de exemplo)
  'ai.cli_mcps.list': () =>
    ['github', 'navegador'].map((name) => ({
      name,
      clis: name === 'github' ? ['claude_code', 'grok_cli'] : ['grok_cli'],
      allowed: cliMcpsAllowed.includes(name),
    })),
  'ai.cli_mcps.set': (a) => {
    cliMcpsAllowed = [...new Set((a.allowed as string[] | undefined) ?? [])].sort();
    return cliMcpsAllowed;
  },
  // MCPs extras deste cofre
  'ai.mcp_extra.list': () => mcpServers.map((m) => structuredClone(m.server)),
  'ai.mcp_extra.save': saveMcp,
  'ai.mcp_extra.delete': (a) => {
    const id = findMcp(str(a, 'id')).server.id;
    mcpServers = mcpServers.filter((m) => m.server.id !== id);
    return null;
  },
  'ai.mcp_extra.test': testMcp,
  'ai.mcp_extra.decide': decideExternal,
  'ai.question.answer': answerQuestion,
});

// Comandos de outros módulos que as telas do agente leem: só entram se o mock do dono
// do módulo não estiver registrado (e são sobrescritos se ele carregar depois).
register(
  {
    'mcp.audit_list': (a) => {
      const limit = Math.min(optNum(a, 'limit') ?? 100, 500);
      const offset = optNum(a, 'offset') ?? 0;
      return state.audit
        .filter((e) => !optStr(a, 'client') || e.client === a.client)
        .filter((e) => !optStr(a, 'tool') || e.tool === a.tool)
        .filter((e) => a.only_errors !== true || !e.ok)
        .sort((x, y) => y.created_at.localeCompare(x.created_at))
        .slice(offset, offset + limit);
    },
    'changes.undo': () => null,
  },
  true,
);
