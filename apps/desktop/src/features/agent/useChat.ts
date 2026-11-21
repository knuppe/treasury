/**
 * Estado do chat em andamento: os turnos enviados nesta sessão da tela (mensagem do
 * usuário + resposta em streaming), o início do job (`ai.chat_start`, com a confirmação de
 * custo), a consulta periódica (`ai.chat_poll`) e o cancelamento.
 */
import { useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useReducer, useRef } from 'react';

import { currentMemberId } from '@/app/viewAs';
import { call } from '@/api/client';
import type { AttachmentInfo } from '@/api/generated/AttachmentInfo';
import type { ChatPoll } from '@/api/generated/ChatPoll';
import type { ChatStart } from '@/api/generated/ChatStart';
import type { ChatStartArgs } from '@/api/generated/ChatStartArgs';
import type { CostEstimate } from '@/api/generated/CostEstimate';
import { aiKeys } from '@/api/hooks/ai';
import { t } from '@/i18n';

import { INITIAL_STREAM, reduceEvents, type StreamState } from './chatStream';

/** Intervalo entre consultas ao job (curto nos testes). */
export const POLL_MS = import.meta.env.MODE === 'test' ? 10 : 400;

export interface LiveTurn {
  id: string;
  message: string;
  /** Anexos enviados com a mensagem (metadados, para o balão). */
  attachments: AttachmentInfo[];
  /** Conversa (com histórico) ou `null` (sem histórico / ainda sem id). */
  conversationId: string | null;
  /** Quantas mensagens a conversa tinha ao enviar (para não repetir as já gravadas). */
  baseCount: number;
  jobId: string | null;
  stream: StreamState;
  /** Falha ao iniciar ou ao consultar o job. */
  failure: unknown;
}

export interface PendingConfirm {
  args: ChatStartArgs;
  estimate: CostEstimate;
  baseCount: number;
  attachments: AttachmentInfo[];
}

interface ChatState {
  turns: LiveTurn[];
  confirm: PendingConfirm | null;
}

type Action =
  | { type: 'add'; turn: LiveTurn }
  | { type: 'started'; id: string; jobId: string; conversationId: string | null }
  | { type: 'poll'; id: string; poll: ChatPoll }
  | { type: 'fail'; id: string; error: unknown }
  | { type: 'confirm'; confirm: PendingConfirm | null }
  | { type: 'drop'; id: string };

function reducer(s: ChatState, a: Action): ChatState {
  const patch = (id: string, f: (t: LiveTurn) => LiveTurn) => ({
    ...s,
    turns: s.turns.map((t) => (t.id === id ? f(t) : t)),
  });
  switch (a.type) {
    case 'add':
      return { ...s, turns: [...s.turns, a.turn] };
    case 'started':
      return patch(a.id, (t) => ({
        ...t,
        jobId: a.jobId,
        conversationId: a.conversationId ?? t.conversationId,
      }));
    case 'poll':
      return patch(a.id, (t) => ({
        ...t,
        conversationId: a.poll.conversation_id ?? t.conversationId,
        stream: reduceEvents(t.stream, a.poll.events, a.poll.status),
      }));
    case 'fail':
      return patch(a.id, (t) => ({
        ...t,
        failure: a.error,
        stream: { ...t.stream, status: t.stream.status === 'running' ? 'failed' : t.stream.status },
      }));
    case 'confirm':
      return { ...s, confirm: a.confirm };
    case 'drop':
      return { ...s, turns: s.turns.filter((t) => t.id !== a.id) };
  }
}

let turnSeq = 0;

export function useChat(onConversation: (id: string) => void) {
  const [state, dispatch] = useReducer(reducer, { turns: [], confirm: null });
  const client = useQueryClient();
  const onConversationRef = useRef(onConversation);
  useEffect(() => {
    onConversationRef.current = onConversation;
  }, [onConversation]);

  const active = state.turns.find((t) => t.jobId && t.stream.status === 'running' && !t.failure);
  const activeId = active?.id;
  const activeJob = active?.jobId;

  // Consulta o job em andamento até terminar.
  useEffect(() => {
    if (!activeId || !activeJob) return;
    let stop = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let after = 0;
    const tick = async () => {
      try {
        const poll = await call<ChatPoll>('ai.chat_poll', { job_id: activeJob, after_seq: after });
        if (stop) return;
        after = poll.next_seq;
        dispatch({ type: 'poll', id: activeId, poll });
        if (poll.events.some((e) => e.event.type === 'proposal')) {
          void client.invalidateQueries({ queryKey: ['ai', 'proposals'] });
          void client.invalidateQueries({ queryKey: aiKeys.pendingCount });
        }
        if (poll.status !== 'running') {
          void client.invalidateQueries({ queryKey: aiKeys.conversations });
          if (poll.conversation_id) {
            void client.invalidateQueries({ queryKey: aiKeys.conversation(poll.conversation_id) });
          }
          void client.invalidateQueries({ queryKey: ['ai', 'usage'] });
          void client.invalidateQueries({ queryKey: aiKeys.jobs });
          return;
        }
      } catch (error) {
        if (!stop) dispatch({ type: 'fail', id: activeId, error });
        return;
      }
      timer = setTimeout(() => void tick(), POLL_MS);
    };
    void tick();
    return () => {
      stop = true;
      if (timer) clearTimeout(timer);
    };
  }, [activeId, activeJob, client]);

  const start = useCallback(
    async (args: ChatStartArgs, baseCount: number, attachments: AttachmentInfo[] = []) => {
      const id = `turn-${++turnSeq}`;
      dispatch({
        type: 'add',
        turn: {
          id,
          message: args.message,
          attachments,
          conversationId: args.conversation_id ?? null,
          baseCount,
          jobId: null,
          stream: INITIAL_STREAM,
          failure: null,
        },
      });
      try {
        // "Ver como" um membro: o agente fala com ele, no nível e no tom dele.
        const memberId = currentMemberId(client);
        const res = await call<ChatStart>(
          'ai.chat_start',
          memberId ? { ...args, member_id: memberId } : args,
        );
        if (res.status === 'confirm_required' && res.estimate) {
          dispatch({ type: 'drop', id });
          dispatch({
            type: 'confirm',
            confirm: { args, estimate: res.estimate, baseCount, attachments },
          });
          return;
        }
        if (!res.job_id) throw new Error(t('agente-erro-sem-job'));
        dispatch({ type: 'started', id, jobId: res.job_id, conversationId: res.conversation_id });
        // A lista de tarefas em segundo plano (barra superior) mostra o job na hora.
        void client.invalidateQueries({ queryKey: aiKeys.jobs });
        if (res.conversation_id) {
          void client.invalidateQueries({ queryKey: aiKeys.conversations });
          onConversationRef.current(res.conversation_id);
        }
      } catch (error) {
        dispatch({ type: 'fail', id, error });
      }
    },
    [client],
  );

  const send = useCallback(
    (args: ChatStartArgs, baseCount: number, attachments?: AttachmentInfo[]) =>
      start(args, baseCount, attachments),
    [start],
  );

  const confirmCost = useCallback(() => {
    const c = state.confirm;
    if (!c) return;
    dispatch({ type: 'confirm', confirm: null });
    void start({ ...c.args, confirm_cost: true }, c.baseCount, c.attachments);
  }, [start, state.confirm]);

  const dismissConfirm = useCallback(() => {
    // Desistiu do envio: os anexos preparados saem da memória do núcleo.
    const ids = state.confirm?.args.attachment_ids ?? [];
    if (ids.length) void call('ai.attachments.discard', { ids }).catch(() => {});
    dispatch({ type: 'confirm', confirm: null });
  }, [state.confirm]);

  const cancel = useCallback(async () => {
    if (!activeJob) return;
    try {
      await call<null>('ai.chat_cancel', { job_id: activeJob });
    } catch (error) {
      if (activeId) dispatch({ type: 'fail', id: activeId, error });
    }
  }, [activeJob, activeId]);

  return {
    turns: state.turns,
    confirm: state.confirm,
    running: Boolean(active) || state.turns.some((t) => !t.jobId && !t.failure),
    send,
    confirmCost,
    dismissConfirm,
    cancel,
  };
}
