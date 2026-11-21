import { describe, expect, it } from 'vitest';

import type { ChangeItem } from '@/api/generated/ChangeItem';
import type { Engine } from '@/api/generated/Engine';
import type { JobEvent } from '@/api/generated/JobEvent';
import type { JobEventKind } from '@/api/generated/JobEventKind';
import type { Message } from '@/api/generated/Message';
import { readChartTheme } from '@/charts/theme';

import { chatTimeline } from './ChatPanel';
import { INITIAL_STREAM, reduceEvents, replyView, streamText } from './chatStream';
import { diffItem, diffItems, formatDiffValue, isMoneyField, visibleFields } from './diff';
import { engineProblem, nextEngine, routeEngines } from './engines';
import { costInputValue, formatCost, formatTokens, parseCost, summaryText } from './format';
import { moveId } from './settings/RoutesPanel';
import { periodFrom } from './settings/UsagePanel';
import { usageBarsOption, usageMonths } from './settings/usageChart';
import type { LiveTurn } from './useChat';

const ev = (seq: number, event: JobEventKind): JobEvent => ({
  seq,
  at: '2026-10-14T12:00:00Z',
  event,
});

describe('custos em micro-dólares', () => {
  it('formata em pt-BR com 2 casas e 4 abaixo de dez centavos', () => {
    expect(formatCost(0)).toBe('USD 0,00');
    expect(formatCost(1_234_560)).toBe('USD 1,23');
    expect(formatCost(4_200)).toBe('USD 0,0042');
    expect(formatCost(12_345_000_000)).toBe('USD 12.345,00');
    expect(formatCost(-500_000)).toBe('−USD 0,50');
    expect(formatCost(17_700)).toBe('USD 0,0177');
  });

  it('lê valores digitados e devolve micro-dólares', () => {
    expect(parseCost('1,50')).toBe(1_500_000);
    expect(parseCost('0.25')).toBe(250_000);
    expect(parseCost('US$ 2')).toBe(2_000_000);
    expect(parseCost('1.234,5')).toBe(1_234_500_000);
    expect(parseCost('abc')).toBeNull();
    expect(parseCost('')).toBeNull();
    expect(costInputValue(1_500_000)).toBe('1,5');
    expect(costInputValue(null)).toBe('');
  });

  it('tokens compactos', () => {
    expect(formatTokens(950)).toBe('950');
    expect(formatTokens(12_300)).toBe('12,3 mil');
    expect(formatTokens(1_200_000)).toBe('1,2 mi');
  });

  it('resume o change_set', () => {
    expect(summaryText({ txn: { update: 2 }, rule: { insert: 1 } })).toBe(
      'altera 2 lançamentos · cria 1 regra',
    );
  });
});

describe('diff exato das propostas', () => {
  const update: ChangeItem = {
    seq: 2,
    table_name: 'txn',
    row_id: 't1',
    op: 'update',
    before: { id: 't1', amount: -1899, category_id: null, memo: 'a', updated_at: 'x' },
    after: { id: 't1', amount: -1899, category_id: 'cat-mercado', memo: 'a', updated_at: 'y' },
  };
  const insert: ChangeItem = {
    seq: 1,
    table_name: 'rule',
    row_id: 'r1',
    op: 'insert',
    before: null,
    after: { id: 'r1', conditions: [{ field: 'description' }], created_at: 'z' },
  };

  it('marca o que muda e recolhe campos iguais e técnicos', () => {
    const row = diffItem(update);
    expect(row.tableLabel).toBe('Lançamento');
    expect(row.opLabel).toBe('Alteração');
    expect(row.changedCount).toBe(1);
    expect(row.fields.find((f) => f.field === 'category_id')).toMatchObject({
      before: null,
      after: 'cat-mercado',
      change: 'changed',
    });
    expect(visibleFields(row).map((f) => f.field)).toEqual(['id', 'category_id']);
    expect(visibleFields(row, true).map((f) => f.field)).toEqual([
      'id',
      'amount',
      'category_id',
      'memo',
      'updated_at',
    ]);
  });

  it('inclusão mostra todos os campos como novos, na ordem de seq', () => {
    const rows = diffItems([update, insert]);
    expect(rows.map((r) => r.table)).toEqual(['rule', 'txn']);
    expect(rows[0]!.fields.every((f) => f.change === 'added')).toBe(true);
    expect(visibleFields(rows[0]!).map((f) => f.field)).toEqual(['id', 'conditions']);
  });

  it('valores brutos, sem perder nada', () => {
    expect(formatDiffValue(null)).toBe('∅');
    expect(formatDiffValue('PAO')).toBe('"PAO"');
    expect(formatDiffValue(-1899)).toBe('-1899');
    expect(formatDiffValue([{ a: 1 }])).toBe('[{"a":1}]');
    expect(isMoneyField('amount', -1899)).toBe(true);
    expect(isMoneyField('fees_amount', 490)).toBe(true);
    expect(isMoneyField('quantity', 100)).toBe(false);
  });
});

describe('redutor do streaming do chat', () => {
  const script: JobEvent[] = [
    ev(1, {
      type: 'engine',
      engine_id: 'e1',
      engine_name: 'Claude',
      kind: 'api',
      provider: 'anthropic',
      model: 'claude-sonnet-4-5',
    }),
    ev(2, { type: 'text', delta: 'Olá, ' }),
    ev(3, { type: 'text', delta: 'vou olhar.' }),
    ev(4, { type: 'tool_call', name: 'query_transactions', summary: 'mês 10' }),
    ev(5, { type: 'tool_result', name: 'query_transactions', ok: true, summary: '3 itens' }),
    ev(6, {
      type: 'proposal',
      proposal_id: 'p1',
      tool: 'categorize',
      command: 'transactions.set_category',
    }),
    ev(7, { type: 'fallback', engine_id: 'e2', engine_name: 'Ollama', reason: 'sem cota' }),
    ev(8, {
      type: 'usage',
      engine_id: 'e1',
      input_tokens: 100,
      output_tokens: 20,
      cost_micros: 1500,
      currency: 'USD',
      priced: true,
    }),
    ev(9, {
      type: 'usage',
      engine_id: 'e1',
      input_tokens: 50,
      output_tokens: 10,
      cost_micros: 0,
      currency: 'USD',
      priced: false,
    }),
    ev(10, {
      type: 'done',
      status: 'done',
      conversation_id: 'c1',
      message_id: 'm1',
      alert_id: null,
    }),
  ];

  it('monta as partes na ordem, juntando o texto e ligando o resultado à chamada', () => {
    const s = reduceEvents(INITIAL_STREAM, script, 'done');
    expect(s.engine?.name).toBe('Claude');
    expect(streamText(s)).toBe('Olá, vou olhar.');
    expect(s.parts.map((p) => p.type)).toEqual(['text', 'tool', 'proposal', 'fallback']);
    expect(s.parts[1]).toMatchObject({
      name: 'query_transactions',
      summary: 'mês 10',
      result: { ok: true },
    });
    expect(s.usage).toMatchObject({
      inputTokens: 150,
      outputTokens: 30,
      costMicros: 1500,
      unpriced: true,
      calls: 2,
    });
    expect(s).toMatchObject({ status: 'done', conversationId: 'c1', messageId: 'm1', lastSeq: 10 });
  });

  it('é idempotente e aceita lotes parciais fora de ordem', () => {
    let s = reduceEvents(INITIAL_STREAM, [script[2]!, script[0]!, script[1]!], 'running');
    s = reduceEvents(s, script.slice(0, 5), 'running');
    expect(streamText(s)).toBe('Olá, vou olhar.');
    expect(s.parts.filter((p) => p.type === 'tool')).toHaveLength(1);
    expect(s.status).toBe('running');
  });

  it('troca de motor aparece no fio; erro e cancelamento ficam no estado', () => {
    const s = reduceEvents(
      INITIAL_STREAM,
      [
        script[0]!,
        ev(2, {
          type: 'engine',
          engine_id: 'e2',
          engine_name: 'Ollama',
          kind: 'local',
          provider: 'ollama',
          model: 'llama3.2',
        }),
        ev(3, { type: 'error', code: 'timeout', message: 'demorou' }),
      ],
      'cancelled',
    );
    expect(s.engine?.name).toBe('Ollama');
    expect(s.parts).toHaveLength(1);
    expect(s.error?.message).toBe('demorou');
    expect(s.status).toBe('cancelled');
  });
});

describe('resposta separada do raciocínio e das ferramentas', () => {
  const at = (sec: number) => `2026-10-15T12:00:${String(sec).padStart(2, '0')}.000Z`;
  const e = (seq: number, sec: number, event: JobEventKind): JobEvent => ({
    seq,
    at: at(sec),
    event,
  });

  it('texto seguido de ferramenta vira passo; o texto depois da última é a resposta', () => {
    const s = reduceEvents(INITIAL_STREAM, [
      e(1, 0, { type: 'reasoning', delta: 'Preciso das ' }),
      e(2, 1, { type: 'reasoning', delta: 'contas.' }),
      e(3, 2, { type: 'text', delta: 'Vou consultar.' }),
      e(4, 3, { type: 'tool_call', name: 'list_accounts', summary: '{}' }),
      e(5, 4, { type: 'tool_result', name: 'list_accounts', ok: true, summary: '3 contas.' }),
      e(6, 5, { type: 'text', delta: '\n\n' }),
      e(7, 6, { type: 'tool_call', name: 'get_account_balance', summary: '{"id":"a"}' }),
      e(8, 7, { type: 'tool_result', name: 'get_account_balance', ok: false, summary: 'erro' }),
      e(9, 12, { type: 'text', delta: 'Vocês têm 3 contas.' }),
      e(10, 13, {
        type: 'proposal',
        proposal_id: 'p1',
        tool: 'create_rule',
        command: 'rules.create',
      }),
    ]);
    const v = replyView(s);
    expect(v.answer).toBe('Vocês têm 3 contas.');
    expect(v.reasoning).toEqual(['Preciso das contas.', 'Vou consultar.']);
    expect(v.reasoningMs).toBe(12_000);
    expect(v.tools.map((t) => [t.name, t.result?.ok])).toEqual([
      ['list_accounts', true],
      ['get_account_balance', false],
    ]);
    expect(v.notices.map((n) => n.type)).toEqual(['proposal']);
  });

  it('ao vivo, o texto em andamento é a resposta até chegar uma ferramenta', () => {
    let s = reduceEvents(INITIAL_STREAM, [e(1, 0, { type: 'text', delta: 'Vou ver.' })], 'running');
    expect(replyView(s, true).answer).toBe('Vou ver.');
    s = reduceEvents(
      s,
      [e(2, 1, { type: 'tool_call', name: 'get_dashboard', summary: '{}' })],
      'running',
    );
    const live = replyView(s, true);
    expect(live.answer).toBe('');
    expect(live.reasoning).toEqual(['Vou ver.']);
    expect(live.tools[0]!.result).toBeNull();
    // Terminou sem texto depois da ferramenta: o último passo vira a resposta.
    expect(replyView(s).answer).toBe('Vou ver.');
  });

  it('sem raciocínio nem ferramentas não há atividade; fallback descarta o texto parcial', () => {
    const plain = replyView(
      reduceEvents(INITIAL_STREAM, [e(1, 0, { type: 'text', delta: 'Oi!' })]),
    );
    expect(plain).toMatchObject({ answer: 'Oi!', reasoning: [], tools: [], reasoningMs: null });
    const fb = replyView(
      reduceEvents(INITIAL_STREAM, [
        e(1, 0, { type: 'text', delta: 'Pela met' }),
        e(2, 1, { type: 'fallback', engine_id: 'e1', engine_name: 'A', reason: 'rede' }),
        e(3, 2, { type: 'text', delta: 'Completa.' }),
      ]),
    );
    expect(fb.answer).toBe('Completa.');
    expect(fb.notices.map((n) => n.type)).toEqual(['fallback']);
  });
});

describe('motores, rotas e uso', () => {
  const engine = (id: string, patch: Partial<Engine> = {}): Engine => ({
    id,
    name: id,
    kind: 'api',
    provider: 'anthropic',
    provider_label: 'Anthropic',
    model: 'm',
    endpoint: null,
    command_template: null,
    effective_endpoint: null,
    effective_template: null,
    enabled: true,
    max_output_tokens: null,
    secret: { set: true, hint: '…abcd', needs_reentry: false, updated_at: null },
    needs_key: true,
    vision: null,
    caps: { images: true, pdf: true, effort: true },
    created_at: '',
    updated_at: '',
    ...patch,
  });

  it('prontidão e o próximo motor da rota', () => {
    const noKey = engine('a', {
      secret: { set: false, hint: null, needs_reentry: false, updated_at: null },
    });
    const off = engine('b', { enabled: false });
    const ok = engine('c');
    expect(engineProblem(noKey)).toBe('sem API key cadastrada');
    expect(engineProblem(engine('d', { kind: 'local', needs_key: false, model: null }))).toBe(
      'sem modelo definido',
    );
    const routes = [{ task: 'chat' as const, label: 'Chat', engine_ids: ['a', 'b', 'c'] }];
    expect(routeEngines(routes, [noKey, off, ok]).map((e) => e.id)).toEqual(['a', 'c']);
    expect(routeEngines(routes, [noKey, off, ok], 'pdf').map((e) => e.id)).toEqual(['a', 'c']);
    expect(nextEngine(routes, [noKey, off, ok])?.id).toBe('c');
    expect(nextEngine(routes, [noKey, off, ok], 'b')?.id).toBe('b');
  });

  it('reordena a rota sem sair dos limites', () => {
    expect(moveId(['a', 'b', 'c'], 2, -1)).toEqual(['a', 'c', 'b']);
    expect(moveId(['a', 'b'], 0, -1)).toEqual(['a', 'b']);
  });

  it('meses do período com zero onde não houve uso, num só eixo', () => {
    expect(periodFrom('2026-10-14', 6)).toBe('2026-05-01');
    expect(periodFrom('2026-02-03', 3)).toBe('2025-12-01');
    const months = usageMonths(
      [
        {
          key: '2026-09',
          label: '2026-09',
          kind: null,
          totals: {
            calls: 3,
            input_tokens: 10,
            output_tokens: 2,
            cost_micros: 5000,
            unpriced_calls: 0,
            errors: 0,
          },
        },
      ],
      '2026-08-01',
      '2026-10-14',
    );
    expect(months.map((m) => [m.key, m.costMicros])).toEqual([
      ['2026-08', 0],
      ['2026-09', 5000],
      ['2026-10', 0],
    ]);
    const option = usageBarsOption(readChartTheme('dark', false), months) as {
      yAxis: unknown;
      series: { type: string; data: number[] }[];
    };
    expect(Array.isArray(option.yAxis)).toBe(false);
    expect(option.series).toHaveLength(1);
    expect(option.series[0]!.data).toEqual([0, 5000, 0]);
  });
});

describe('linha do tempo do chat', () => {
  const msg = (id: string, role: 'user' | 'assistant', proposals: string[] = []) =>
    ({ id, role, content: id, proposal_ids: proposals }) as unknown as Message;
  const turn = (id: string, baseCount: number, extra: Partial<LiveTurn['stream']> = {}) =>
    ({
      id,
      message: id,
      attachments: [],
      conversationId: 'c1',
      baseCount,
      jobId: 'j',
      stream: { ...INITIAL_STREAM, status: 'failed', ...extra },
      failure: null,
    }) as LiveTurn;

  it('põe o turno onde a conversa estava ao enviar, não no fim', () => {
    // Turno antigo (enviado com 2 mensagens gravadas, a resposta dele não foi gravada) e
    // uma troca depois dele: o turno fica logo depois da mensagem do usuário que o abriu.
    const saved = [
      msg('u1', 'user'),
      msg('a1', 'assistant'),
      msg('u2', 'user'),
      msg('u3', 'user'),
      msg('a3', 'assistant'),
    ];
    const order = chatTimeline(saved, [turn('t', 2)]).map((i) =>
      i.kind === 'saved' ? i.message.id : `turn:${i.showUser}`,
    );
    expect(order).toEqual(['u1', 'a1', 'u2', 'turn:false', 'u3', 'a3']);
    // Sem a mensagem do usuário gravada, o turno mostra o balão dele no mesmo lugar.
    const fresh = chatTimeline(saved.slice(0, 2), [turn('t', 2)]).map((i) => i.kind);
    expect(fresh).toEqual(['saved', 'saved', 'turn']);
  });

  it('turno terminado cujas propostas estão numa mensagem gravada sai', () => {
    const saved = [msg('u1', 'user'), msg('a1', 'assistant', ['p1'])];
    const t = turn('t', 0, {
      parts: [{ type: 'proposal', proposalId: 'p1', tool: 'x', command: 'y' }],
    });
    expect(chatTimeline(saved, [t]).filter((i) => i.kind === 'turn')).toHaveLength(0);
    // Em andamento, fica.
    const running = turn('r', 2, { status: 'running' });
    expect(chatTimeline(saved, [running]).at(-1)).toMatchObject({ kind: 'turn' });
  });
});

describe('pedido de confirmação de MCP extra no streaming', () => {
  it('vira um aviso à vista e recebe a decisão pelo request_id', () => {
    const waiting = reduceEvents(INITIAL_STREAM, [
      ev(1, { type: 'tool_call', name: 'celular__abrir_app', summary: '{"app": "Agenda"}' }),
      ev(2, {
        type: 'external_approval',
        request_id: 'r1',
        server: 'Meu celular',
        tool: 'abrir_app',
        args: '{"app": "Agenda"}',
      }),
    ]);
    const view = replyView(waiting, true);
    expect(view.tools.map((x) => x.name)).toEqual(['celular__abrir_app']);
    expect(view.notices).toEqual([
      {
        type: 'external',
        requestId: 'r1',
        server: 'Meu celular',
        tool: 'abrir_app',
        args: '{"app": "Agenda"}',
        allowed: null,
        remembered: false,
      },
    ]);
    const decided = reduceEvents(waiting, [
      ev(3, { type: 'external_decision', request_id: 'outro', allowed: true, remembered: true }),
      ev(4, { type: 'external_decision', request_id: 'r1', allowed: true, remembered: true }),
    ]);
    expect(decided.parts[1]).toMatchObject({ type: 'external', allowed: true, remembered: true });
  });
});

describe('pergunta do agente no streaming', () => {
  const question = {
    question: 'Qual conta devo usar?',
    options: [
      { label: 'Conta corrente', description: null },
      { label: 'Poupança', description: 'Reserva' },
    ],
    recommended: 0,
    multi_select: false,
    allow_other: true,
  };

  it('vira um aviso à vista à espera e recebe a resposta pelo request_id', () => {
    const waiting = reduceEvents(INITIAL_STREAM, [
      ev(1, { type: 'tool_call', name: 'ask_user', summary: 'Qual conta devo usar?' }),
      ev(2, { type: 'user_question', request_id: 'q1', question }),
    ]);
    const view = replyView(waiting, true);
    expect(view.tools.map((x) => x.name)).toEqual(['ask_user']);
    expect(view.notices).toEqual([
      { type: 'question', requestId: 'q1', question, answer: undefined },
    ]);
    const answered = reduceEvents(waiting, [
      ev(3, {
        type: 'user_answer',
        request_id: 'outra',
        answer: { selected: ['Poupança'], other: null },
      }),
      ev(4, {
        type: 'user_answer',
        request_id: 'q1',
        answer: { selected: [], other: 'Conta da família' },
      }),
    ]);
    expect(answered.parts[1]).toMatchObject({
      type: 'question',
      answer: { selected: [], other: 'Conta da família' },
    });
  });

  it('sem resposta (tempo esgotado) fica null, diferente de à espera', () => {
    const s = reduceEvents(INITIAL_STREAM, [
      ev(1, { type: 'user_question', request_id: 'q1', question }),
      ev(2, { type: 'user_answer', request_id: 'q1', answer: null }),
    ]);
    expect(s.parts[0]).toMatchObject({ type: 'question', answer: null });
  });
});
