/**
 * Conversa com o agente: lista de conversas, fio com as mensagens gravadas e a resposta em
 * streaming, indicador permanente do motor que recebe os dados, raciocínio e ferramentas
 * usadas recolhidos (`ReplyActivity`), a resposta final em destaque, propostas criadas
 * com link para a fila, pedidos de confirmação de MCPs extras, perguntas do agente (formulário), trocas de motor, uso e custo, cancelamento, confirmação de custo e
 * a liberação, por conversa, da leitura do computador (só CLIs).
 */
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
  type Ref,
} from 'react';
import { Link, useSearchParams } from 'react-router';

import { isApiError } from '@/api/client';
import type { Engine } from '@/api/generated/Engine';
import type { Message } from '@/api/generated/Message';
import {
  useAiSettings,
  useConversation,
  useConversations,
  useEngines,
  useRoutes,
} from '@/api/hooks/ai';
import {
  Button,
  Dialog,
  Icon,
  InstrumentCard,
  LiveIndicator,
  Markdown,
  Private,
  Select,
  Tooltip,
} from '@/design/components';
import { cx } from '@/design/cx';
import { ErrorBlock, LoadingBlock, Notice } from '@/features/portfolio/components/States';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';
import { useAgentSession, useComputerAccess, useConversationEngine } from '@/state/agentSession';
import { flash } from '@/state/activity';
import { useUi } from '@/state/ui';

import { replyView, type StreamEngine, type StreamPart, type StreamState } from './chatStream';
import {
  AttachButton,
  AttachmentTray,
  DropOverlay,
  EffortSelect,
  MessageAttachments,
  ImageModeChoice,
  UnsupportedNotice,
  useComposer,
} from './composer';
import { ComputerAccessSwitch } from './ComputerAccess';
import { ExportConversation } from './ExportConversation';
import { ExternalApprovalCard } from './ExternalApproval';
import { ClearHistoryButton, ConversationRows } from './ConversationHistory';
import { engineProblem, nextEngine, routeEngines } from './engines';
import {
  ENGINE_KIND_DESTINATION,
  ENGINE_KIND_LABEL,
  JOB_STATUS_LABEL,
  formatCost,
  formatTokens,
} from './format';
import { useFreshText } from './freshText';
import { QuestionCard } from './QuestionCard';
import { ReplyActivity, type ActivityTool } from './ReplyActivity';
import { Pill } from './ui';
import { useChat, type LiveTurn } from './useChat';

const AUTO = '__auto__';

export interface ChatLinks {
  settings: string;
  proposal: (id: string) => string;
}

/**
 * Como desenhar propostas dentro do fio. A tela 10 usa o link para a fila; o painel do
 * agente companheiro aprova ali mesmo.
 */
export interface ChatRender {
  proposal?: (id: string, label: string) => ReactNode;
}

export const ChatRenderContext = createContext<ChatRender>({});

// --- Indicador do motor -----------------------------------------------------------------

export function EngineIndicator({
  engine,
  live,
  compact,
}: {
  engine: Pick<StreamEngine, 'name' | 'kind' | 'model'> | null;
  live: boolean;
  /** Uma linha só (cabeçalho do painel): o destino dos dados fica na dica. */
  compact?: boolean;
}) {
  if (compact) {
    const destination = engine ? ENGINE_KIND_DESTINATION[engine.kind] : null;
    return (
      <Tooltip
        side="bottom"
        align="start"
        content={
          engine ? (
            <span className="flex flex-col gap-0.5">
              <span>
                {t(live ? 'agente-motor-em-uso-nome' : 'agente-proximo-motor-nome', {
                  motor: engine.model ? `${engine.name} · ${engine.model}` : engine.name,
                })}
              </span>
              <span className="text-dim">{destination}</span>
            </span>
          ) : (
            t('agente-nenhum-motor-dica')
          )
        }
      >
        <div
          role="status"
          aria-label={t('agente-motor-recebe')}
          tabIndex={0}
          className="flex min-w-0 items-center gap-1.5 rounded-sm text-[11.5px] text-dim outline-none focus-visible:text-fg"
        >
          {live ? (
            <LiveIndicator label={t('agente-respondendo')} tone="accent" className="text-[10px]" />
          ) : (
            <span
              aria-hidden
              className={cx(
                'inline-block size-1.5 shrink-0 rounded-full',
                engine ? 'bg-positive' : 'bg-warning',
              )}
            />
          )}
          {engine ? (
            <>
              <span className="truncate text-fg">{engine.name}</span>
              <span className="shrink-0">{`· ${ENGINE_KIND_LABEL[engine.kind]}`}</span>
              <span className="sr-only">{destination}</span>
            </>
          ) : (
            <span className="truncate">{t('agente-nenhum-motor-pronto')}</span>
          )}
        </div>
      </Tooltip>
    );
  }
  return (
    <div
      role="status"
      aria-label={t('agente-motor-recebe')}
      className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b border-line bg-surface-2 px-4 py-2"
    >
      <span className="label-caps text-[10px]">
        {live ? t('agente-motor-em-uso') : t('agente-proximo-motor')}
      </span>
      {engine ? (
        <>
          <span className="text-[13px] font-medium text-fg">{engine.name}</span>
          {engine.model ? (
            <span className="font-mono text-[11.5px] text-dim">{engine.model}</span>
          ) : null}
          <Pill
            tone={
              engine.kind === 'local' ? 'positive' : engine.kind === 'cli' ? 'accent' : 'warning'
            }
          >
            {ENGINE_KIND_LABEL[engine.kind]}
          </Pill>
          <span className="text-[12px] text-dim">{ENGINE_KIND_DESTINATION[engine.kind]}</span>
          {live ? <LiveIndicator label={t('agente-recebendo')} tone="accent" /> : null}
        </>
      ) : (
        <span className="text-[12.5px] text-dim">{t('agente-nenhum-motor-pronto')}</span>
      )}
    </div>
  );
}

// --- Partes da resposta -----------------------------------------------------------------

function ProposalLink({ id, label, links }: { id: string; label: string; links: ChatLinks }) {
  const render = useContext(ChatRenderContext);
  if (render.proposal) return <>{render.proposal(id, label)}</>;
  return (
    <div className="flex items-center gap-2 rounded-control border border-l-2 border-line border-l-accent bg-[var(--accent-faint)] px-2.5 py-1.5 text-[12.5px]">
      <Icon name="check" size={12} className="text-accent" />
      <span className="text-fg">{t('agente-proposta-criada', { nome: label })}</span>
      <span className="text-dim">{t('agente-aguarda-aprovacao')}</span>
      <span className="flex-1" />
      <Link className="text-accent underline-offset-2 hover:underline" to={links.proposal(id)}>
        {t('agente-revisar-fila')}
      </Link>
    </div>
  );
}

/** Resposta final, em destaque. */
function Answer({ text, running = false }: { text: string; running?: boolean }) {
  // No Fósforo, o texto que acabou de chegar entra em verde mais claro (freshText.ts).
  const ref = useFreshText<HTMLDivElement>(text, running);
  if (!text) return null;
  return (
    <Private>
      <div ref={ref}>
        <Markdown text={text} />
      </div>
    </Private>
  );
}

/** Propostas, pedidos de confirmação, perguntas e trocas de motor: ficam à vista, depois da resposta. */
function Part({
  part,
  links,
  running = false,
}: {
  part: StreamPart;
  links: ChatLinks;
  running?: boolean;
}) {
  switch (part.type) {
    case 'text':
    case 'reasoning':
    case 'tool':
      return null;
    case 'proposal':
      return <ProposalLink id={part.proposalId} label={part.tool} links={links} />;
    case 'external':
      return <ExternalApprovalCard part={part} running={running} />;
    case 'question':
      return <QuestionCard part={part} running={running} />;
    case 'fallback':
      return (
        <p className="flex items-center gap-2 text-[12px] text-warning" role="note">
          <Icon name="alert" size={12} />
          {t('agente-fallback', { motor: part.engineName, motivo: part.reason })}
        </p>
      );
    case 'engine':
      return (
        <p className="text-[12px] text-dim" role="note">
          {tRich('agente-agora-com', {
            motor: <span className="text-fg">{part.engine.name}</span>,
            tipo: ENGINE_KIND_LABEL[part.engine.kind],
          })}
        </p>
      );
  }
}

function UsageLine({ stream }: { stream: StreamState }) {
  const u = stream.usage;
  if (!u) return null;
  return (
    <p
      className="num flex flex-wrap gap-x-3 text-[11px] text-dim"
      aria-label={t('agente-uso-resposta')}
    >
      <span>{t('agente-tokens-entrada', { n: formatTokens(u.inputTokens) })}</span>
      <span>{t('agente-tokens-saida', { n: formatTokens(u.outputTokens) })}</span>
      <span>
        {tRich(u.unpriced ? 'agente-custo-parcial' : 'agente-custo', {
          valor: <Private>{formatCost(u.costMicros, u.currency)}</Private>,
        })}
      </span>
    </p>
  );
}

/**
 * Uma mensagem do fio. A sua fica à direita, num balão; a do agente ocupa a largura, sem
 * moldura, abaixo de um rótulo discreto, para a resposta ler como texto e não como cartão.
 */
export function Bubble({
  role,
  children,
  meta,
  boxRef,
}: {
  role: 'user' | 'assistant';
  children: ReactNode;
  meta?: ReactNode;
  /** O corpo da resposta (o flash ao terminar, state/activity.ts). */
  boxRef?: Ref<HTMLDivElement>;
}) {
  if (role === 'user') {
    return (
      <div className="flex flex-col items-end gap-1 pl-8">
        <span className="sr-only">{t('agente-voce')}</span>
        <div className="flex max-w-full flex-col gap-2 rounded-card rounded-br-sm border border-line bg-surface-2 px-3.5 py-2.5">
          {children}
        </div>
        {meta}
      </div>
    );
  }
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <span className="flex items-center gap-1.5">
        <span
          aria-hidden
          className="inline-flex size-5 items-center justify-center rounded-full bg-accent-soft text-accent"
        >
          <Icon name="agent" size={11} />
        </span>
        <span className="label-caps text-[10px]">{t('casca-tela-agente')}</span>
      </span>
      <div
        ref={boxRef}
        className="flex max-w-[min(100%,720px)] min-w-0 flex-col gap-2.5 rounded-control pl-6.5"
      >
        {children}
      </div>
      {meta ? <div className="pl-6.5">{meta}</div> : null}
    </div>
  );
}

function savedTools(m: Message): ActivityTool[] {
  return m.tool_calls.map((t) => ({
    name: t.name,
    args: t.args,
    result: { ok: t.ok, summary: t.summary },
  }));
}

export function SavedMessage({ m, links }: { m: Message; links: ChatLinks }) {
  if (m.role === 'user') {
    return (
      <Bubble
        role="user"
        meta={<span className="num text-[10.5px] text-dim">{formatDateTime(m.created_at)}</span>}
      >
        <Private>
          {m.content ? (
            <p data-selectable className="text-[13.5px] whitespace-pre-wrap text-fg">
              {m.content}
            </p>
          ) : null}
          <MessageAttachments items={m.attachments ?? []} />
        </Private>
      </Bubble>
    );
  }
  return (
    <Bubble
      role="assistant"
      meta={
        <span className="num text-[10.5px] text-dim">
          {`${m.engine_name ?? t('agente-motor-generico')} · ${formatDateTime(m.created_at)}`}
        </span>
      }
    >
      <ReplyActivity
        id={m.id}
        reasoning={m.reasoning ? [m.reasoning] : []}
        reasoningMs={m.reasoning_ms}
        tools={savedTools(m)}
        running={false}
        answering
      />
      <Answer text={m.content} />
      {m.proposal_ids.map((id) => (
        <ProposalLink key={id} id={id} label={t('agente-alteracao')} links={links} />
      ))}
    </Bubble>
  );
}

export function NoEngineHint({ links }: { links: ChatLinks }) {
  return (
    <span className="flex flex-col gap-1">
      <span>
        {tRich('agente-sem-motor-dica', {
          link: (
            <Link className="text-accent underline-offset-2 hover:underline" to={links.settings}>
              {t('agente-configurar-motores')}
            </Link>
          ),
        })}
      </span>
      <span>{t('groq-sem-motor-dica')}</span>
    </span>
  );
}

export function isNoEngineError(error: unknown): boolean {
  return isApiError(error) && error.code === 'no_engine';
}

export type TimelineItem =
  { kind: 'saved'; message: Message } | { kind: 'turn'; turn: LiveTurn; showUser: boolean };

/**
 * Mensagens gravadas e turnos ao vivo na ordem da conversa: cada turno entra logo depois da
 * mensagem que o abriu (ou onde a conversa estava ao enviar), não no fim da lista. Um turno
 * que já terminou e cujas propostas já estão numa mensagem gravada sai (a gravada vale).
 */
export function chatTimeline(saved: Message[], turns: LiveTurn[]): TimelineItem[] {
  const savedProposals = new Set(saved.flatMap((m) => m.proposal_ids));
  const live = turns.filter((x) => {
    if (x.stream.status === 'running' || x.failure) return true;
    const ids = x.stream.parts.flatMap((p) => (p.type === 'proposal' ? [p.proposalId] : []));
    return !(ids.length > 0 && ids.every((id) => savedProposals.has(id)));
  });
  const at = (x: LiveTurn) => {
    const userSaved = Boolean(x.conversationId) && saved.length > x.baseCount;
    return { x, userSaved, index: Math.min(saved.length, x.baseCount + (userSaved ? 1 : 0)) };
  };
  const placed = live.map(at);
  const out: TimelineItem[] = [];
  for (let i = 0; i <= saved.length; i++) {
    for (const p of placed) {
      if (p.index === i) out.push({ kind: 'turn', turn: p.x, showUser: !p.userSaved });
    }
    const m = saved[i];
    if (m) out.push({ kind: 'saved', message: m });
  }
  return out;
}

export function LiveTurnView({
  turn,
  showUser,
  links,
}: {
  turn: LiveTurn;
  showUser: boolean;
  links: ChatLinks;
}) {
  const s = turn.stream;
  const running = s.status === 'running' && !turn.failure;
  const view = replyView(s, running);
  // Quem abriu o raciocínio ou as ferramentas ao vivo continua vendo na mensagem gravada.
  const alias = useAgentSession((st) => st.alias);
  useEffect(() => {
    if (s.messageId) alias(turn.id, s.messageId);
  }, [alias, s.messageId, turn.id]);
  // Terminou: a borda pisca (verde, ou vermelha com erro) nos temas que mostram (Fósforo).
  const box = useRef<HTMLDivElement>(null);
  const was = useRef(s.status);
  useEffect(() => {
    if (was.current === 'running' && s.status !== 'running') {
      flash(box.current, s.status === 'failed');
    }
    was.current = s.status;
  }, [s.status]);
  return (
    <>
      {showUser ? (
        <Bubble role="user">
          <Private>
            {turn.message ? (
              <p data-selectable className="text-[13.5px] whitespace-pre-wrap text-fg">
                {turn.message}
              </p>
            ) : null}
            <MessageAttachments items={turn.attachments} />
          </Private>
        </Bubble>
      ) : null}
      {turn.failure ? (
        <ErrorBlock
          error={turn.failure}
          title={
            isNoEngineError(turn.failure) ? t('agente-sem-motor-titulo') : t('agente-nao-respondeu')
          }
          className="max-w-[720px]"
        />
      ) : null}
      {turn.failure && isNoEngineError(turn.failure) ? (
        <p className="text-[12.5px] text-dim">
          <NoEngineHint links={links} />
        </p>
      ) : null}
      {!turn.failure ? (
        <Bubble
          role="assistant"
          boxRef={box}
          meta={
            <span className="num text-[10.5px] text-dim">
              {`${s.engine?.name ?? t('agente-aguardando-motor')} · ${JOB_STATUS_LABEL[s.status]}`}
            </span>
          }
        >
          <ReplyActivity
            id={turn.id}
            reasoning={view.reasoning}
            reasoningMs={view.reasoningMs}
            tools={view.tools.map((x) => ({ name: x.name, args: x.summary, result: x.result }))}
            running={running}
            answering={Boolean(view.answer)}
          />
          <Answer text={view.answer} running={running} />
          {view.notices.map((p, i) => (
            <Part key={i} part={p} links={links} running={running} />
          ))}
          {running && s.progress ? <p className="text-[12px] text-dim">{s.progress}</p> : null}
          {s.error ? (
            <p role="alert" className="text-[12.5px] text-negative">
              {s.error.message}
            </p>
          ) : null}
          {s.status === 'cancelled' ? (
            <p className="text-[12px] text-dim">{t('agente-resposta-cancelada')}</p>
          ) : null}
          <UsageLine stream={s} />
        </Bubble>
      ) : null}
    </>
  );
}

// --- Lista de conversas -----------------------------------------------------------------

function ConversationList({
  selected,
  onSelect,
}: {
  selected: string | null;
  onSelect: (id: string | null) => void;
}) {
  const list = useConversations();
  return (
    <InstrumentCard
      index="01"
      title={t('agente-conversas')}
      padded={false}
      className="h-full min-h-0"
      actions={
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="plus" size={12} />}
          onClick={() => onSelect(null)}
        >
          {t('agente-nova')}
        </Button>
      }
      footer={
        (list.data?.length ?? 0) > 0 ? (
          <ClearHistoryButton onCleared={() => onSelect(null)} />
        ) : null
      }
    >
      {list.isLoading ? (
        <LoadingBlock />
      ) : list.error ? (
        <ErrorBlock error={list.error} className="m-3" />
      ) : (list.data ?? []).length === 0 ? (
        <p className="px-4 py-6 text-[12.5px] text-dim">{t('agente-nenhuma-conversa')}</p>
      ) : (
        <ConversationRows
          items={list.data ?? []}
          selected={selected}
          onSelect={onSelect}
          className="min-h-0 overflow-auto p-1.5"
        />
      )}
    </InstrumentCard>
  );
}

// --- Painel -----------------------------------------------------------------------------

export function CostDialog({
  estimate,
  onConfirm,
  onCancel,
}: {
  estimate: {
    engine_name: string;
    cost_micros: number;
    threshold_micros: number;
    currency: string;
  } | null;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <Dialog
      open={estimate != null}
      onOpenChange={(o) => {
        if (!o) onCancel();
      }}
      size="sm"
      title={t('agente-confirmar-custo')}
      description={
        estimate ? (
          <>
            {tRich(
              'agente-custo-texto',
              {
                motor: estimate.engine_name,
                custo: formatCost(estimate.cost_micros, estimate.currency),
                limite: formatCost(estimate.threshold_micros, estimate.currency),
              },
              (content) => (
                <strong className="text-fg">{content}</strong>
              ),
            )}
          </>
        ) : undefined
      }
      footer={
        <>
          <Button variant="ghost" onClick={onCancel}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="primary" onClick={onConfirm}>
            {t('agente-enviar-mesmo-assim')}
          </Button>
        </>
      }
    />
  );
}

export function ChatPanel({ links }: { links: ChatLinks }) {
  const [params, setParams] = useSearchParams();
  const sessionsOpen = useUi((s) => s.agentSessionsOpen);
  const setSessionsOpen = useUi((s) => s.setAgentSessionsOpen);
  // `?conversa=<id>` abre uma conversa (ex.: vinda do painel do agente).
  const [conversationId, setConversationId] = useState<string | null>(() => params.get('conversa'));
  const wanted = params.get('conversa');
  const [lastWanted, setLastWanted] = useState(wanted);
  if (wanted !== lastWanted) {
    setLastWanted(wanted);
    if (wanted) setConversationId(wanted);
  }
  const [draft, setDraft] = useState(() => params.get('pergunta') ?? '');
  // A pergunta vinda da barra de comando entra no campo uma vez e sai da URL.
  const asked = params.get('pergunta');
  const [lastAsked, setLastAsked] = useState(asked);
  if (asked !== lastAsked) {
    setLastAsked(asked);
    if (asked != null) setDraft(asked);
  }
  useEffect(() => {
    if (asked == null) return;
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        next.delete('pergunta');
        return next;
      },
      { replace: true },
    );
  }, [asked, setParams]);
  // Motor escolhido nesta conversa (sessão): o mesmo do painel ⌘J.
  const chosenEngine = useConversationEngine(conversationId);
  const setEngine = useAgentSession((s) => s.setEngine);
  const setEngineChoice = (v: string) => setEngine(conversationId, v === AUTO ? null : v);
  const engines = useEngines();
  const routes = useRoutes();
  const settings = useAiSettings();
  const detail = useConversation(conversationId);
  const carryComputer = useAgentSession((s) => s.carryComputer);
  const chat = useChat((id) => {
    setConversationId(id);
    carryComputer(id);
  });
  const computerAccess = useComputerAccess(conversationId);

  const engineList = engines.data ?? [];
  // A escolha vale enquanto o motor existe e está ligado (apagado ou desligado depois, a
  // conversa volta ao automático em vez de mandar um motor que não existe mais).
  const explicit =
    chosenEngine && engineList.some((e) => e.id === chosenEngine && e.enabled)
      ? chosenEngine
      : null;
  const engineChoice = explicit ?? AUTO;
  const upcoming = nextEngine(routes.data ?? [], engineList, explicit);
  const routeEmpty =
    engines.isSuccess && routes.isSuccess && routeEngines(routes.data, engineList).length === 0;

  const saved = conversationId ? (detail.data?.messages ?? []) : [];
  const savedIds = new Set(saved.map((m) => m.id));
  const turns = chat.turns.filter(
    (x) =>
      x.conversationId === conversationId &&
      !(x.stream.messageId && savedIds.has(x.stream.messageId)),
  );
  const liveEngine = [...turns].reverse().find((x) => x.stream.engine)?.stream.engine ?? null;
  const running = chat.running;
  const candidates = explicit
    ? engineList.filter((e) => e.id === explicit)
    : routeEngines(routes.data ?? [], engineList).filter((e) => engineProblem(e) == null);
  const composer = useComposer({ conversationId, candidates, running });

  const send = (text = draft) => {
    const message = text.trim();
    if (!composer.canSend(text)) return;
    if (text === draft) setDraft('');
    const attachments = composer.sentAttachments();
    void chat.send(
      {
        message,
        ...(conversationId && settings.data?.keep_history !== false
          ? { conversation_id: conversationId }
          : {}),
        ...(explicit ? { engine_id: explicit } : {}),
        ...(computerAccess ? { computer_access: true } : {}),
        ...composer.extraArgs(),
      },
      saved.length,
      attachments,
    );
    composer.sent();
  };

  const onKey = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
      e.preventDefault();
      send();
    }
  };

  const engineOptions = [
    { value: AUTO, label: t('agente-automatico') },
    ...engineList.map((e: Engine) => ({
      value: e.id,
      label: e.name,
      hint: engineProblem(e) ?? ENGINE_KIND_LABEL[e.kind],
      disabled: !e.enabled,
    })),
  ];

  const title = detail.data?.title ?? null;
  return (
    // O chat é o principal da tela: ocupa a altura da janela, com as conversas num menu
    // lateral recolhível (fechado por padrão; o app lembra a escolha).
    <div className="flex h-[calc(100dvh-268px)] min-h-[480px] flex-col gap-3 @min-[790px]:flex-row">
      {sessionsOpen ? (
        <div className="flex max-h-[40%] shrink-0 flex-col @min-[790px]:max-h-none @min-[790px]:w-[260px]">
          <ConversationList
            selected={conversationId}
            onSelect={(id) => {
              setConversationId(id);
            }}
          />
        </div>
      ) : null}
      <section
        aria-label={t('agente-conversa-com')}
        className="flex min-h-0 min-w-0 flex-1 flex-col rounded-card border border-line bg-surface"
      >
        <div className="flex items-center gap-2 border-b border-line px-2 py-1.5">
          <Tooltip
            content={sessionsOpen ? t('agente-conversas-recolher') : t('agente-conversas-abrir')}
          >
            <Button
              size="sm"
              variant="ghost"
              aria-expanded={sessionsOpen}
              aria-label={
                sessionsOpen ? t('agente-conversas-recolher') : t('agente-conversas-abrir')
              }
              icon={<Icon name={sessionsOpen ? 'chevronLeft' : 'history'} size={14} />}
              onClick={() => setSessionsOpen(!sessionsOpen)}
            >
              {sessionsOpen ? null : t('agente-conversas')}
            </Button>
          </Tooltip>
          {/* Com a lista aberta, o "Nova" fica só no cabeçalho dela. */}
          {sessionsOpen ? null : (
            <Button
              size="sm"
              variant="ghost"
              icon={<Icon name="plus" size={12} />}
              onClick={() => setConversationId(null)}
              disabled={conversationId == null}
            >
              {t('agente-nova')}
            </Button>
          )}
          <span
            className="min-w-0 flex-1 truncate text-[12.5px] text-dim"
            title={title ?? undefined}
          >
            {title ?? t('agente-conversa-nova-titulo')}
          </span>
          {conversationId && saved.length > 0 && title ? (
            <ExportConversation id={conversationId} title={title} />
          ) : null}
        </div>
        <EngineIndicator
          engine={running && liveEngine ? liveEngine : upcoming}
          live={Boolean(running && liveEngine)}
        />
        <div
          role="log"
          aria-label={t('agente-mensagens-conversa')}
          className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4"
          aria-live="polite"
        >
          {routeEmpty ? (
            <Notice tone="warning" title={t('agente-sem-motor-titulo')}>
              <NoEngineHint links={links} />
            </Notice>
          ) : null}
          {settings.data && !settings.data.keep_history ? (
            <p className="text-[12px] text-dim">{t('agente-historico-desligado')}</p>
          ) : null}
          {conversationId && detail.isLoading ? <LoadingBlock /> : null}
          {conversationId && detail.error ? <ErrorBlock error={detail.error} /> : null}
          {chatTimeline(saved, turns).map((item) =>
            item.kind === 'saved' ? (
              <SavedMessage key={item.message.id} m={item.message} links={links} />
            ) : (
              <LiveTurnView
                key={item.turn.id}
                turn={item.turn}
                showUser={item.showUser}
                links={links}
              />
            ),
          )}
          {saved.length === 0 && turns.length === 0 && !routeEmpty ? (
            <div className="flex flex-1 flex-col items-center justify-center gap-2 py-10 text-center">
              <Icon name="agent" size={22} className="text-accent" />
              <p className="text-[13.5px] text-fg">{t('agente-vazio-titulo')}</p>
              <p className="max-w-md text-[12.5px] text-dim">
                {t(
                  settings.data?.autonomous ? 'agente-vazio-texto-autonomo' : 'agente-vazio-texto',
                )}
              </p>
            </div>
          ) : null}
        </div>
        <form
          className="relative flex flex-col gap-2 border-t border-line p-3"
          onSubmit={(e) => {
            e.preventDefault();
            send();
          }}
          {...composer.drop.props}
        >
          <DropOverlay over={composer.drop.over} />
          <AttachmentTray draft={composer.attachments} unsupported={composer.unsupportedIds} />
          <UnsupportedNotice items={composer.unsupported} engines={candidates} />
          <ImageModeChoice choice={composer.imageChoice} />
          <label className="sr-only" htmlFor="agent-message">
            {t('agente-mensagem')}
          </label>
          <textarea
            id="agent-message"
            rows={3}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={onKey}
            onPaste={composer.onPaste}
            placeholder={t('agente-placeholder')}
            className="w-full resize-y rounded-control border border-line-strong bg-bg px-2.5 py-2 text-[13.5px] text-fg outline-none focus:border-accent focus:shadow-[0_0_0_1px_var(--accent)]"
          />
          <div className="flex flex-wrap items-center gap-2">
            <div className="w-[260px]">
              <Select
                aria-label={t('agente-motor')}
                value={engineChoice}
                onValueChange={setEngineChoice}
                options={engineOptions}
              />
            </div>
            <AttachButton onFiles={composer.attachments.add} disabled={running} />
            <EffortSelect
              conversationId={conversationId}
              engine={running && liveEngine ? null : upcoming}
              disabled={running}
            />
            <ComputerAccessSwitch conversationId={conversationId} disabled={running} />
            <span className="text-[11.5px] text-dim">{t('agente-enter-envia')}</span>
            <span className="flex-1" />
            {running ? (
              <Button
                variant="danger"
                onClick={() => void chat.cancel()}
                icon={<Icon name="x" size={12} />}
              >
                {t('comum-cancelar')}
              </Button>
            ) : null}
            <Button
              type="submit"
              variant="primary"
              disabled={!composer.canSend(draft)}
              icon={<Icon name="arrowRight" size={12} />}
            >
              {composer.unsupported.length ? t('agente-enviar-assim-mesmo') : t('agente-enviar')}
            </Button>
          </div>
        </form>
      </section>
      <CostDialog
        estimate={chat.confirm?.estimate ?? null}
        onConfirm={chat.confirmCost}
        onCancel={chat.dismissConfirm}
      />
    </div>
  );
}
