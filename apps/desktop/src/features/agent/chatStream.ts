/**
 * Redutor do streaming do chat: os eventos de `ai.chat_poll` (na ordem de `seq`) viram
 * as partes de uma resposta do agente — texto, raciocínio, chamadas de ferramenta com o
 * resultado, propostas, pedidos de confirmação de MCPs extras, perguntas do agente, trocas de motor — mais o motor atual, o uso
 * somado e o estado final. Puro e idempotente: eventos repetidos (`seq` já visto) são
 * ignorados.
 *
 * `replyView` organiza as partes como a conversa mostra: raciocínio e passos
 * intermediários (texto seguido de ferramenta) recolhidos, ferramentas numa lista
 * compacta, a resposta final em destaque. É a mesma regra do núcleo (`ai::activity`), que
 * grava a mensagem já separada.
 */
import type { AskAnswer } from '@/api/generated/AskAnswer';
import type { AskQuestion } from '@/api/generated/AskQuestion';
import type { EngineKind } from '@/api/generated/EngineKind';
import type { JobEvent } from '@/api/generated/JobEvent';
import type { JobStatus } from '@/api/generated/JobStatus';
import type { Provider } from '@/api/generated/Provider';

export interface StreamEngine {
  engineId: string;
  name: string;
  kind: EngineKind;
  provider: Provider;
  model: string | null;
}

export type StreamPart =
  | { type: 'text'; text: string; at: number | null }
  | { type: 'reasoning'; text: string; at: number | null }
  | {
      type: 'tool';
      name: string;
      summary: string;
      result: { ok: boolean; summary: string } | null;
    }
  | { type: 'proposal'; proposalId: string; tool: string; command: string }
  /** Pedido de confirmação de uma ferramenta de MCP extra; `allowed` chega com a decisão. */
  | {
      type: 'external';
      requestId: string;
      server: string;
      tool: string;
      args: string;
      allowed: boolean | null;
      /** Permitido no resto da conversa (as próximas chamadas do servidor passam direto). */
      remembered: boolean;
    }
  /**
   * Pergunta do agente à pessoa (`ask_user`). `answer`: `undefined` enquanto espera; `null`
   * quando ficou sem resposta (tempo esgotado ou conversa cancelada).
   */
  | {
      type: 'question';
      requestId: string;
      question: AskQuestion;
      answer: AskAnswer | null | undefined;
    }
  | { type: 'fallback'; engineId: string; engineName: string; reason: string }
  | { type: 'engine'; engine: StreamEngine };

export interface StreamUsage {
  inputTokens: number;
  outputTokens: number;
  costMicros: number;
  currency: string;
  /** Alguma chamada sem preço cadastrado (custo parcial). */
  unpriced: boolean;
  calls: number;
}

export interface StreamState {
  lastSeq: number;
  /** Instante (ms) do primeiro e do último evento da resposta. */
  startedAt: number | null;
  lastAt: number | null;
  status: JobStatus;
  engine: StreamEngine | null;
  parts: StreamPart[];
  usage: StreamUsage | null;
  /** Última mensagem de progresso (some quando chega texto). */
  progress: string | null;
  error: { code: string; message: string } | null;
  conversationId: string | null;
  messageId: string | null;
  alertId: string | null;
}

export const INITIAL_STREAM: StreamState = {
  lastSeq: 0,
  startedAt: null,
  lastAt: null,
  status: 'running',
  engine: null,
  parts: [],
  usage: null,
  progress: null,
  error: null,
  conversationId: null,
  messageId: null,
  alertId: null,
};

function millis(at: string): number | null {
  const t = Date.parse(at);
  return Number.isFinite(t) ? t : null;
}

function applyEvent(s: StreamState, e: JobEvent): StreamState {
  const ev = e.event;
  const parts = s.parts;
  const at = millis(e.at);
  switch (ev.type) {
    case 'engine': {
      const engine: StreamEngine = {
        engineId: ev.engine_id,
        name: ev.engine_name,
        kind: ev.kind,
        provider: ev.provider,
        model: ev.model,
      };
      // O primeiro motor fica só no indicador; trocas (fallback) aparecem no fio.
      const next = s.engine ? [...parts, { type: 'engine' as const, engine }] : parts;
      return { ...s, engine, parts: next };
    }
    case 'text':
    case 'reasoning': {
      const last = parts[parts.length - 1];
      const next =
        last?.type === ev.type
          ? [...parts.slice(0, -1), { ...last, text: last.text + ev.delta }]
          : [...parts, { type: ev.type, text: ev.delta, at }];
      return { ...s, parts: next, progress: null };
    }
    case 'tool_call':
      return {
        ...s,
        parts: [...parts, { type: 'tool', name: ev.name, summary: ev.summary, result: null }],
      };
    case 'tool_result': {
      // Liga ao último chamado aberto da mesma ferramenta.
      let idx = -1;
      for (let i = parts.length - 1; i >= 0; i--) {
        const p = parts[i]!;
        if (p.type === 'tool' && p.name === ev.name && p.result == null) {
          idx = i;
          break;
        }
      }
      const result = { ok: ev.ok, summary: ev.summary };
      if (idx < 0) {
        return {
          ...s,
          parts: [...parts, { type: 'tool', name: ev.name, summary: '', result }],
        };
      }
      const next = parts.slice();
      next[idx] = { ...(parts[idx] as Extract<StreamPart, { type: 'tool' }>), result };
      return { ...s, parts: next };
    }
    case 'proposal':
      return {
        ...s,
        parts: [
          ...parts,
          { type: 'proposal', proposalId: ev.proposal_id, tool: ev.tool, command: ev.command },
        ],
      };
    case 'fallback':
      return {
        ...s,
        parts: [
          ...parts,
          {
            type: 'fallback',
            engineId: ev.engine_id,
            engineName: ev.engine_name,
            reason: ev.reason,
          },
        ],
      };
    case 'external_approval':
      return {
        ...s,
        parts: [
          ...parts,
          {
            type: 'external',
            requestId: ev.request_id,
            server: ev.server,
            tool: ev.tool,
            args: ev.args,
            allowed: null,
            remembered: false,
          },
        ],
      };
    case 'external_decision':
      return {
        ...s,
        parts: parts.map((p) =>
          p.type === 'external' && p.requestId === ev.request_id
            ? { ...p, allowed: ev.allowed, remembered: ev.remembered }
            : p,
        ),
      };
    case 'user_question':
      return {
        ...s,
        parts: [
          ...parts,
          { type: 'question', requestId: ev.request_id, question: ev.question, answer: undefined },
        ],
      };
    case 'user_answer':
      return {
        ...s,
        parts: parts.map((p) =>
          p.type === 'question' && p.requestId === ev.request_id ? { ...p, answer: ev.answer } : p,
        ),
      };
    case 'usage': {
      const u = s.usage ?? {
        inputTokens: 0,
        outputTokens: 0,
        costMicros: 0,
        currency: ev.currency,
        unpriced: false,
        calls: 0,
      };
      return {
        ...s,
        usage: {
          inputTokens: u.inputTokens + ev.input_tokens,
          outputTokens: u.outputTokens + ev.output_tokens,
          costMicros: u.costMicros + ev.cost_micros,
          currency: ev.currency || u.currency,
          unpriced: u.unpriced || !ev.priced,
          calls: u.calls + 1,
        },
      };
    }
    case 'progress':
      return { ...s, progress: ev.message };
    case 'error':
      return { ...s, error: { code: ev.code, message: ev.message } };
    case 'done':
      return {
        ...s,
        status: ev.status,
        progress: null,
        conversationId: ev.conversation_id ?? s.conversationId,
        messageId: ev.message_id,
        alertId: ev.alert_id,
      };
  }
}

/** Aplica um lote de eventos (de um `chat_poll`) ao estado. */
export function reduceEvents(
  state: StreamState,
  events: readonly JobEvent[],
  status?: JobStatus,
): StreamState {
  let s = state;
  for (const e of [...events].sort((a, b) => a.seq - b.seq)) {
    if (e.seq <= s.lastSeq) continue;
    const at = e.event.type === 'done' ? null : millis(e.at);
    s = {
      ...applyEvent(s, e),
      lastSeq: e.seq,
      startedAt: s.startedAt ?? at,
      lastAt: at ?? s.lastAt,
    };
  }
  // O job pode ter terminado sem um `done` (ex.: cancelado antes de começar).
  if (status && status !== 'running' && s.status === 'running') s = { ...s, status };
  return s;
}

/** Todo o texto streamado, passos intermediários incluídos (testes). */
export function streamText(s: StreamState): string {
  return s.parts
    .filter((p): p is Extract<StreamPart, { type: 'text' }> => p.type === 'text')
    .map((p) => p.text)
    .join('');
}

export type ToolPart = Extract<StreamPart, { type: 'tool' }>;

/** Uma resposta como a conversa mostra. */
export interface ReplyView {
  /** Raciocínio e passos intermediários, na ordem. */
  reasoning: string[];
  /** Tempo até a resposta começar (ms), quando houve raciocínio ou ferramentas. */
  reasoningMs: number | null;
  tools: ToolPart[];
  /** Resposta final (ao vivo: o texto depois da última ferramenta). */
  answer: string;
  /** Propostas, pedidos de confirmação, perguntas e trocas de motor, na ordem (ficam à vista). */
  notices: StreamPart[];
}

/**
 * Separa resposta, raciocínio e ferramentas. Texto seguido de ferramenta é passo
 * intermediário; o que vem depois da última ferramenta é a resposta (se não vier nada, o
 * último passo vira a resposta); uma troca de motor descarta o texto do motor que falhou.
 * Enquanto o job roda, o texto em andamento aparece como resposta e pode virar passo
 * quando chegar uma ferramenta.
 */
export function replyView(s: StreamState, running = false): ReplyView {
  const steps: { kind: 'reasoning' | 'narration'; text: string }[] = [];
  const tools: ToolPart[] = [];
  const notices: StreamPart[] = [];
  let pending: { text: string; at: number | null } | null = null;
  for (const p of s.parts) {
    switch (p.type) {
      case 'reasoning':
        if (p.text.trim()) steps.push({ kind: 'reasoning', text: p.text.trim() });
        break;
      case 'text':
        pending = pending
          ? { text: pending.text + p.text, at: pending.at ?? p.at }
          : { text: p.text, at: p.text.trim() ? p.at : null };
        break;
      case 'tool':
        if (pending?.text.trim()) steps.push({ kind: 'narration', text: pending.text.trim() });
        pending = null;
        tools.push(p);
        break;
      case 'fallback':
        pending = null;
        notices.push(p);
        break;
      default:
        notices.push(p);
    }
  }
  let answer = pending?.text.trim() ?? '';
  let answerAt = answer ? (pending?.at ?? null) : null;
  if (!answer && !running) {
    const i = steps.map((x) => x.kind).lastIndexOf('narration');
    if (i >= 0) {
      answer = steps[i]!.text;
      steps.splice(i, 1);
      answerAt = null;
    }
  }
  const reasoning = steps.map((x) => x.text);
  const busy = reasoning.length > 0 || tools.length > 0;
  const end = answerAt ?? s.lastAt;
  const reasoningMs =
    busy && s.startedAt != null && end != null ? Math.max(0, end - s.startedAt) : null;
  return { reasoning, reasoningMs, tools, answer, notices };
}
