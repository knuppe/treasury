/**
 * Agente companheiro: painel lateral fixo, presente em todas as telas (⌘J), com a mesma
 * conversa da tela 10 (`ai.chat_*`: streaming, ferramentas, motor visível, confirmação de
 * custo). Diferenças do painel:
 *
 * - envia o contexto da tela aberta (chip "Contexto", com prévia e opção de não enviar);
 * - com a conversa vazia, mostra pedidos rápidos ("O que posso fazer aqui?", cadastro
 *   guiado) e sugestões determinísticas da tela (`ai.suggestions`, sem motor); somem na
 *   primeira mensagem;
 * - as conversas anteriores ficam numa vista própria (botão no cabeçalho), que toma o
 *   lugar do fio enquanto aberta;
 * - aprova ou rejeita propostas ali mesmo;
 * - liga e desliga o modo autônomo nas opções (só escritas reversíveis; sensíveis sempre
 *   item a item).
 *
 * O painel fica montado enquanto o cofre está aberto (fechar só esconde), então a conversa e
 * a resposta em andamento sobrevivem à navegação.
 */
import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { Link, useNavigate } from 'react-router';

import { screenMeta } from '@/app/screens';

import type { Suggestion } from '@/api/generated/Suggestion';
import {
  useAiSettings,
  useConversation,
  useConversations,
  useEngines,
  useRoutes,
  useSetAiSettings,
  useSuggestions,
} from '@/api/hooks/ai';
import {
  Button,
  Icon,
  IconButton,
  ScannerVoice,
  Private,
  Tooltip,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import {
  ChatRenderContext,
  CostDialog,
  EngineIndicator,
  LiveTurnView,
  NoEngineHint,
  SavedMessage,
  chatTimeline,
  type ChatLinks,
  type ChatRender,
} from '@/features/agent/ChatPanel';
import {
  AttachButton,
  AttachmentTray,
  DropOverlay,
  EffortSelect,
  ImageModeChoice,
  UnsupportedNotice,
  useComposer,
} from '@/features/agent/composer';
import { ComputerAccessSwitch } from '@/features/agent/ComputerAccess';
import { ClearHistoryButton, ConversationRows } from '@/features/agent/ConversationHistory';
import { EnginePicker } from '@/features/agent/EnginePicker';
import { engineProblem, nextEngine, routeEngines } from '@/features/agent/engines';
import { useChat } from '@/features/agent/useChat';
import {
  ErrorBlock,
  LoadingBlock,
  Notice,
  errorMessage,
} from '@/features/portfolio/components/States';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { useAgentSession, useComputerAccess, useConversationEngine } from '@/state/agentSession';
import { COMPANION_MAX_WIDTH, COMPANION_MIN_WIDTH, useCompanion } from '@/state/companion';

import { InlineProposal } from './InlineProposal';
import { useCompanionContext, type CompanionContext } from './useCompanionContext';

const LINKS: ChatLinks = {
  settings: '/agente?aba=configurar',
  proposal: (id) => `/agente?aba=aprovacao&proposta=${encodeURIComponent(id)}`,
};

/** Tela Agente ("Agente"), citada nos atalhos para a tela cheia. */
const AGENT_SCREEN = screenMeta('agente');
const agentScreenLabel = () => `${AGENT_SCREEN.index} · ${AGENT_SCREEN.title}`;

/**
 * Pedidos rápidos da conversa vazia. O texto vai para o motor no idioma do app (o agente
 * responde no idioma de quem pergunta). O do cadastro guiado ("onde vocês têm contas?") é
 * igual ao das sugestões do núcleo.
 */
export const quickPrompts = () =>
  [
    {
      id: 'here',
      label: t('companheiro-pedido-aqui-rotulo'),
      prompt: t('companheiro-pedido-aqui'),
    },
    {
      id: 'summary',
      label: t('companheiro-pedido-resumo-rotulo'),
      prompt: t('companheiro-pedido-resumo'),
    },
    {
      id: 'onboarding',
      label: t('companheiro-pedido-cadastro-rotulo'),
      prompt: t('companheiro-pedido-cadastro'),
    },
  ] as const;

// --- Peças ------------------------------------------------------------------------------

function AutonomySwitch() {
  const settings = useAiSettings();
  const set = useSetAiSettings();
  const on = settings.data?.autonomous ?? false;
  return (
    <div className="flex items-start gap-3 px-2 py-1.5">
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className={cx('text-[12.5px]', on ? 'text-warning' : 'text-fg')}>
          {t('agente-autonomo')}
        </span>
        <span className="text-[11px] leading-snug text-dim">
          {on ? t('companheiro-autonomo-ligado-dica') : t('companheiro-autonomo-desligado-dica')}
        </span>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={on}
        aria-label={t('agente-autonomo')}
        disabled={!settings.data || set.isPending}
        onClick={() =>
          set.mutate(
            { autonomous: !on },
            {
              onSuccess: (s) =>
                toast({
                  tone: 'info',
                  title: s.autonomous
                    ? t('companheiro-autonomo-ligado')
                    : t('companheiro-autonomo-desligado'),
                  description: s.autonomous
                    ? t('companheiro-autonomo-ligado-texto')
                    : t('companheiro-autonomo-desligado-texto'),
                }),
              onError: (e) =>
                toast({
                  tone: 'error',
                  title: t('companheiro-nao-foi-possivel'),
                  description: errorMessage(e),
                }),
            },
          )
        }
        className={cx(
          'relative mt-0.5 inline-block h-4 w-7 shrink-0 rounded-full transition-colors disabled:opacity-50',
          on ? 'bg-warning' : 'bg-line-strong',
        )}
      >
        <span
          aria-hidden
          className={cx(
            'absolute top-0.5 size-3 rounded-full bg-bg transition-[left]',
            on ? 'left-3.5' : 'left-0.5',
          )}
        />
      </button>
    </div>
  );
}

/** Opções do painel: modo autônomo e atalhos para a tela 10 e os motores. */
function OptionsMenu({ fullView }: { fullView: string }) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (root.current && !root.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onDown);
    return () => document.removeEventListener('mousedown', onDown);
  }, [open]);
  const link =
    'flex items-center gap-2 rounded-control px-2 py-1.5 text-[12.5px] text-fg hover:bg-surface-2';
  return (
    <div
      ref={root}
      className="relative"
      onKeyDown={(e) => {
        if (e.key === 'Escape' && open) {
          e.preventDefault();
          e.stopPropagation();
          setOpen(false);
          button.current?.focus();
        }
      }}
    >
      <IconButton
        ref={button}
        size="sm"
        label={t('companheiro-opcoes')}
        aria-haspopup="dialog"
        aria-expanded={open}
        pressed={open}
        icon={<Icon name="sliders" size={14} />}
        onClick={() => setOpen((o) => !o)}
      />
      {open ? (
        <div
          role="dialog"
          aria-label={t('companheiro-opcoes')}
          className="absolute top-[calc(100%+6px)] right-0 z-[60] flex w-72 animate-pop-in flex-col gap-1 rounded-card border border-panel pop-surface p-1.5"
        >
          <AutonomySwitch />
          <div className="my-1 border-t border-line" />
          <Link to={fullView} className={link} onClick={() => setOpen(false)}>
            <Icon name="arrowRight" size={13} className="text-dim" />
            {t('companheiro-tela-cheia', { tela: agentScreenLabel() })}
          </Link>
          <Link to={LINKS.settings} className={link} onClick={() => setOpen(false)}>
            <Icon name="key" size={13} className="text-dim" />
            {t('agente-configurar-motores')}
          </Link>
        </div>
      ) : null}
    </div>
  );
}

function ContextChip({ ctx }: { ctx: CompanionContext }) {
  const share = useCompanion((s) => s.shareContext);
  const setShare = useCompanion((s) => s.setShareContext);
  const [preview, setPreview] = useState(false);
  const usable = ctx.value != null;
  return (
    <div className="flex flex-col gap-1.5">
      {preview ? (
        <div
          id="companion-context-preview"
          role="region"
          aria-label={t('companheiro-contexto-previa')}
          className="flex flex-col gap-1 rounded-control border border-line bg-bg/60 p-2"
        >
          <p className="text-[11px] text-dim">
            {share ? t('companheiro-contexto-vai') : t('companheiro-contexto-desligado')}
          </p>
          {usable ? (
            <Private>
              <pre
                data-selectable
                className="max-h-48 overflow-auto font-mono text-[10.5px] whitespace-pre-wrap text-fg"
              >
                {JSON.stringify(ctx.value, null, 1)}
              </pre>
            </Private>
          ) : (
            <p className="text-[11px] text-warning">{t('companheiro-contexto-grande')}</p>
          )}
        </div>
      ) : null}
      <div className="flex min-w-0 items-center gap-2 px-0.5 text-[11px]">
        <button
          type="button"
          aria-expanded={preview}
          aria-controls="companion-context-preview"
          onClick={() => setPreview((v) => !v)}
          className={cx(
            'inline-flex min-w-0 items-center gap-1 rounded-sm hover:text-fg',
            share && usable ? 'text-dim' : 'text-dim line-through',
          )}
          title={t('companheiro-contexto-ver')}
        >
          <Icon
            name="chevronRight"
            size={10}
            className={cx('shrink-0 transition-transform', preview && 'rotate-90')}
          />
          <span className="truncate">{t('companheiro-contexto-pagina', { tela: ctx.label })}</span>
        </button>
        <label className="ml-auto inline-flex shrink-0 items-center gap-1.5 text-dim">
          <input
            type="checkbox"
            className="size-3 accent-[var(--accent)]"
            checked={share}
            onChange={(e) => setShare(e.target.checked)}
          />
          {t('companheiro-enviar-contexto')}
        </label>
      </div>
    </div>
  );
}

function SuggestionCard({
  s,
  onAsk,
  busy,
}: {
  s: Suggestion;
  onAsk: (prompt: string) => void;
  busy: boolean;
}) {
  const navigate = useNavigate();
  return (
    <li
      className={cx(
        'flex items-center rounded-control border border-l-2 border-line bg-surface transition-colors hover:bg-surface-2',
        s.severity === 'warning' ? 'border-l-warning' : 'border-l-accent',
      )}
    >
      <button
        type="button"
        disabled={busy}
        aria-label={t('companheiro-perguntar', { titulo: s.title })}
        onClick={() => onAsk(s.prompt)}
        className="flex min-w-0 flex-1 flex-col gap-0.5 px-3 py-2 text-left disabled:opacity-50"
      >
        <span className="text-[12.5px] text-fg">
          <Private>{s.title}</Private>
        </span>
        <span className="text-[11.5px] leading-snug text-dim">
          <Private>{s.detail}</Private>
        </span>
      </button>
      {s.route ? (
        <IconButton
          size="sm"
          label={t('companheiro-abrir-tela')}
          className="mr-1.5"
          icon={<Icon name="arrowRight" size={12} />}
          onClick={() => void navigate(s.route!)}
        />
      ) : null}
    </li>
  );
}

/**
 * Conversa vazia: uma saudação curta, os pedidos rápidos e as sugestões da tela. Some na
 * primeira mensagem; nada disso se mistura com o fio.
 */
function Welcome({
  ctx,
  suggestions,
  onAsk,
  busy,
}: {
  ctx: CompanionContext;
  suggestions: ReturnType<typeof useSuggestions>;
  onAsk: (prompt: string) => void;
  busy: boolean;
}) {
  const list = suggestions.data ?? [];
  return (
    <section aria-label={t('companheiro-sugestoes')} className="my-auto flex flex-col gap-6 py-4">
      <div className="flex flex-col gap-2.5">
        <span
          aria-hidden
          className="inline-flex size-9 items-center justify-center rounded-full bg-accent-soft text-accent"
        >
          <Icon name="agent" size={18} className="scanner-swap" />
          <ScannerVoice speaking={false} className="scale-125" />
        </span>
        <h3 className="text-[16px] font-medium tracking-[-0.005em] text-fg">
          {t('companheiro-como-ajudar')}
        </h3>
        <p className="text-[12.5px] leading-relaxed text-dim">
          {tRich('companheiro-vendo', { tela: <span className="text-fg">{ctx.label}</span> })}
        </p>
      </div>
      <div
        role="group"
        aria-label={t('companheiro-pedidos-rapidos')}
        className="flex flex-col gap-1.5"
      >
        {quickPrompts().map((q) => (
          <button
            key={q.id}
            type="button"
            disabled={busy}
            onClick={() => onAsk(q.prompt)}
            className="group flex items-center gap-2 rounded-control border border-line px-3 py-2 text-left text-[12.5px] text-fg transition-colors hover:border-accent/60 hover:bg-surface-2 disabled:opacity-50"
          >
            <span className="flex-1">{q.label}</span>
            <Icon
              name="arrowRight"
              size={12}
              className="text-dim transition-colors group-hover:text-accent"
            />
          </button>
        ))}
      </div>
      {list.length > 0 ? (
        <div className="flex flex-col gap-2">
          <p className="label-caps text-[10px]">{t('companheiro-nesta-tela')}</p>
          <ul className="flex flex-col gap-1.5">
            {list.map((s) => (
              <SuggestionCard key={s.id} s={s} onAsk={onAsk} busy={busy} />
            ))}
          </ul>
        </div>
      ) : suggestions.isSuccess ? (
        <p className="text-[11.5px] text-dim">{t('companheiro-nada-pendente')}</p>
      ) : null}
    </section>
  );
}

/**
 * Conversas anteriores, numa vista própria que toma o lugar do fio (nunca no meio dele):
 * fixadas primeiro, depois as recentes, com busca pelo título.
 */
function HistoryView({
  current,
  onOpen,
  onNew,
  onBack,
  fullView,
}: {
  current: string | null;
  onOpen: (id: string | null) => void;
  onNew: () => void;
  onBack: () => void;
  fullView: string;
}) {
  const list = useConversations();
  const settings = useAiSettings();
  const [query, setQuery] = useState('');
  const q = query.trim().toLocaleLowerCase();
  const all = list.data ?? [];
  const items = q ? all.filter((c) => c.title.toLocaleLowerCase().includes(q)) : all;
  const pinned = items.filter((c) => c.pinned);
  const recent = items.filter((c) => !c.pinned);
  return (
    <section aria-label={t('companheiro-anteriores')} className="flex min-h-0 flex-1 flex-col">
      <div className="flex shrink-0 items-center gap-1.5 px-3 pt-3 pb-2">
        <IconButton
          size="sm"
          label={t('companheiro-voltar')}
          icon={<Icon name="chevronLeft" size={14} />}
          onClick={onBack}
        />
        <h3 className="text-[13.5px] font-medium text-fg">{t('agente-conversas')}</h3>
        <span className="flex-1" />
        <Button size="sm" variant="secondary" icon={<Icon name="plus" size={12} />} onClick={onNew}>
          {t('agente-nova')}
        </Button>
      </div>
      {all.length > 0 ? (
        <div className="shrink-0 px-4 pb-2">
          <label className="flex h-8 items-center gap-2 rounded-control border border-line bg-bg px-2.5 focus-within:border-accent">
            <Icon name="search" size={12} className="text-dim" />
            <span className="sr-only">{t('companheiro-buscar-conversa')}</span>
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('companheiro-buscar-titulo')}
              className="min-w-0 flex-1 bg-transparent text-[12.5px] text-fg outline-none placeholder:text-dim"
            />
          </label>
        </div>
      ) : null}
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-2 pt-1 pb-3">
        {settings.data && !settings.data.keep_history ? (
          <p className="px-2 text-[12px] text-dim">{t('agente-historico-desligado')}</p>
        ) : null}
        {list.isLoading ? (
          <LoadingBlock />
        ) : list.error ? (
          <ErrorBlock error={list.error} className="mx-2" />
        ) : all.length === 0 ? (
          <p className="px-2 py-6 text-center text-[12.5px] text-dim">
            {t('agente-nenhuma-conversa')}
          </p>
        ) : items.length === 0 ? (
          <p className="px-2 py-6 text-center text-[12.5px] text-dim">
            {t('companheiro-nenhuma-com', { busca: query.trim() })}
          </p>
        ) : (
          <>
            {pinned.length > 0 ? (
              <div className="flex flex-col gap-1">
                <p className="label-caps px-2.5 text-[10px]">{t('companheiro-fixadas')}</p>
                <ConversationRows
                  items={pinned}
                  selected={current}
                  onSelect={onOpen}
                  label={t('companheiro-conversas-fixadas')}
                />
              </div>
            ) : null}
            {recent.length > 0 ? (
              <div className="flex flex-col gap-1">
                <p className="label-caps px-2.5 text-[10px]">{t('companheiro-recentes')}</p>
                <ConversationRows
                  items={recent}
                  selected={current}
                  onSelect={onOpen}
                  label={t('companheiro-conversas-recentes')}
                />
              </div>
            ) : null}
          </>
        )}
      </div>
      <div className="flex shrink-0 items-center gap-2 border-t border-line px-3 py-2">
        <Link
          to={fullView}
          className="text-[11.5px] text-dim underline-offset-2 hover:text-accent hover:underline"
        >
          {t('companheiro-ver-na-tela', { tela: agentScreenLabel() })}
        </Link>
        <span className="flex-1" />
        {all.length > 0 ? <ClearHistoryButton onCleared={() => onOpen(null)} /> : null}
      </div>
    </section>
  );
}

const RENDER: ChatRender = {
  proposal: (id, label) => <InlineProposal id={id} label={label} />,
};

function ResizeHandle() {
  const width = useCompanion((s) => s.width);
  const setWidth = useCompanion((s) => s.setWidth);
  const start = useRef<{ x: number; w: number } | null>(null);
  const onDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    start.current = { x: e.clientX, w: width };
    e.currentTarget.setPointerCapture(e.pointerId);
  };
  const onMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (!start.current) return;
    setWidth(start.current.w + (start.current.x - e.clientX));
  };
  const onUp = (e: ReactPointerEvent<HTMLDivElement>) => {
    start.current = null;
    e.currentTarget.releasePointerCapture(e.pointerId);
  };
  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === 'ArrowLeft') setWidth(width + 24);
    else if (e.key === 'ArrowRight') setWidth(width - 24);
    else return;
    e.preventDefault();
  };
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label={t('companheiro-largura')}
      aria-valuenow={width}
      aria-valuemin={COMPANION_MIN_WIDTH}
      aria-valuemax={COMPANION_MAX_WIDTH}
      tabIndex={0}
      onPointerDown={onDown}
      onPointerMove={onMove}
      onPointerUp={onUp}
      onKeyDown={onKey}
      className="absolute inset-y-0 -left-1 z-10 w-2 cursor-col-resize outline-none hover:bg-accent/30 focus-visible:bg-accent/40"
    />
  );
}

// --- Painel -----------------------------------------------------------------------------

/**
 * Mantém a conversa do painel viva enquanto o cofre está aberto (a resposta em andamento
 * continua chegando com o painel fechado ou em outra tela) e desenha o painel só aberto.
 */
export function CompanionHost() {
  const open = useCompanion((s) => s.open);
  const setConversation = useCompanion((s) => s.setConversation);
  const carryComputer = useAgentSession((s) => s.carryComputer);
  const chat = useChat((id) => {
    setConversation(id);
    carryComputer(id);
  });
  return open ? <CompanionPanel chat={chat} /> : null;
}

function CompanionPanel({ chat }: { chat: ReturnType<typeof useChat> }) {
  const open = useCompanion((s) => s.open);
  const setOpen = useCompanion((s) => s.setOpen);
  const conversationId = useCompanion((s) => s.conversationId);
  const setConversation = useCompanion((s) => s.setConversation);
  const share = useCompanion((s) => s.shareContext);
  const pendingDraft = useCompanion((s) => s.draft);
  const ctx = useCompanionContext();
  const engines = useEngines();
  const routes = useRoutes();
  const settings = useAiSettings();
  const detail = useConversation(conversationId);
  const suggestions = useSuggestions(ctx.screen, open);
  const computerAccess = useComputerAccess(conversationId);
  const [draft, setDraft] = useState('');
  // Fio ou lista de conversas anteriores (a lista toma o lugar do fio, nunca o meio dele).
  const [view, setView] = useState<'chat' | 'history'>('chat');
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const endRef = useRef<HTMLDivElement>(null);

  // Texto pedido por outra parte do app (ex.: barra de comando) entra no campo uma vez.
  const [seenDraft, setSeenDraft] = useState<string | null>(null);
  if (pendingDraft !== seenDraft) {
    setSeenDraft(pendingDraft);
    if (pendingDraft != null) {
      setDraft(pendingDraft);
      setView('chat');
    }
  }
  useEffect(() => {
    if (pendingDraft == null) return;
    useCompanion.getState().takeDraft();
    inputRef.current?.focus();
  }, [pendingDraft]);

  useEffect(() => {
    if (open) inputRef.current?.focus({ preventScroll: true });
  }, [open]);

  const engineList = engines.data ?? [];
  // Motores prontos da rota de chat: com mais de um, o cabeçalho deixa trocar nesta conversa.
  const ready = routeEngines(routes.data ?? [], engineList).filter((e) => engineProblem(e) == null);
  const chosenEngine = useConversationEngine(conversationId);
  const setEngine = useAgentSession((s) => s.setEngine);
  const explicit = chosenEngine && ready.some((e) => e.id === chosenEngine) ? chosenEngine : null;
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
  const empty = saved.length === 0 && turns.length === 0;
  const loadingSaved = conversationId != null && detail.isLoading;

  // Acompanha o fim da conversa só se o usuário já está lá (rolar para cima para ler, ou
  // para responder um cartão, não é atropelado pelo texto que chega).
  const logRef = useRef<HTMLDivElement>(null);
  const stick = useRef(true);
  const lastPart = turns[turns.length - 1]?.stream.parts.length ?? 0;
  const lastText = turns[turns.length - 1]?.stream.parts.at(-1);
  const lastLen = lastText && 'text' in lastText ? String(lastText.text).length : 0;
  useEffect(() => {
    // Rola só a lista da conversa: `scrollIntoView` rolaria também a janela inteira.
    const log = logRef.current;
    if (view === 'chat' && stick.current && log) log.scrollTop = log.scrollHeight;
  }, [view, conversationId, saved.length, turns.length, lastPart, lastLen]);

  const candidates = explicit ? ready.filter((e) => e.id === explicit) : ready;
  const composer = useComposer({ conversationId, candidates, running });
  // Arquivos mandados por outra parte do app (ex.: imagens e "Analisar com IA" da
  // importação): viram anexos e o campo recebe o foco. Assina a loja direto (os que chegam
  // antes ou depois de o painel abrir entram do mesmo jeito) e foca de novo depois que a
  // janela de onde vieram termina de fechar (ela devolve o foco a quem a abriu).
  const addAttachments = composer.attachments.add;
  useEffect(() => {
    const timers: number[] = [];
    const focusInput = () => {
      inputRef.current?.focus();
      for (const ms of [120, 400])
        timers.push(
          window.setTimeout(() => {
            const el = inputRef.current;
            if (el && document.activeElement !== el) el.focus();
          }, ms),
        );
    };
    const take = () => {
      const files = useCompanion.getState().takeFiles();
      if (!files?.length) return;
      setView('chat');
      addAttachments(files);
      focusInput();
    };
    take();
    const unsubscribe = useCompanion.subscribe((s) => {
      if (s.files?.length) take();
    });
    return () => {
      unsubscribe();
      timers.forEach((id) => window.clearTimeout(id));
    };
  }, [addAttachments]);

  const showChat = () => {
    stick.current = true;
    setView('chat');
  };
  const startNew = () => {
    setConversation(null);
    showChat();
    inputRef.current?.focus();
  };

  const send = (text: string) => {
    const message = text.trim();
    if (!composer.canSend(text)) return;
    setDraft('');
    stick.current = true;
    const attachments = composer.sentAttachments();
    void chat.send(
      {
        message,
        ...(conversationId && settings.data?.keep_history !== false
          ? { conversation_id: conversationId }
          : {}),
        ...(share && ctx.value ? { page_context: ctx.value } : {}),
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
      send(draft);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      setOpen(false);
    }
  };

  const fullView = `/agente?aba=conversa${
    conversationId ? `&conversa=${encodeURIComponent(conversationId)}` : ''
  }`;
  const autonomous = settings.data?.autonomous ?? false;
  const history = view === 'history';

  return (
    <aside
      aria-label={t('companheiro-rotulo')}
      className="relative flex min-h-0 w-full flex-col border-l border-line bg-surface"
    >
      <ResizeHandle />
      <header className="flex h-14 shrink-0 items-center gap-2.5 border-b border-line pr-2 pl-4">
        <span
          aria-hidden
          className="inline-flex size-7 shrink-0 items-center justify-center rounded-full bg-accent-soft text-accent"
        >
          <Icon name="agent" size={14} className="scanner-swap" />
          <ScannerVoice speaking={running} />
        </span>
        <div className="flex min-w-0 flex-1 flex-col gap-0.5">
          <div className="flex items-center gap-2">
            <h2 className="text-[13.5px] leading-none font-medium text-fg">
              {t('casca-tela-agente')}
            </h2>
            {autonomous ? (
              <Tooltip content={t('companheiro-autonomo-tag-dica')}>
                <span
                  tabIndex={0}
                  className="label-caps rounded-sm border border-warning/50 px-1 text-[9.5px] leading-[14px] text-warning"
                >
                  {t('companheiro-autonomo-tag')}
                </span>
              </Tooltip>
            ) : null}
          </div>
          {ready.length > 1 && !(running && liveEngine) ? (
            <EnginePicker
              engines={ready}
              value={explicit}
              current={upcoming}
              onChange={(id) => setEngine(conversationId, id)}
              disabled={running}
            />
          ) : (
            <EngineIndicator
              compact
              engine={running && liveEngine ? liveEngine : upcoming}
              live={Boolean(running && liveEngine)}
            />
          )}
        </div>
        <div className="flex shrink-0 items-center gap-0.5">
          <IconButton
            size="sm"
            label={t('companheiro-anteriores')}
            pressed={history}
            icon={<Icon name="history" size={14} />}
            onClick={() => (history ? showChat() : setView('history'))}
          />
          <IconButton
            size="sm"
            label={t('companheiro-nova-conversa')}
            icon={<Icon name="plus" size={14} />}
            disabled={running}
            onClick={startNew}
          />
          <OptionsMenu fullView={fullView} />
          <IconButton
            size="sm"
            label={t('companheiro-fechar')}
            shortcut={['mod', 'J']}
            icon={<Icon name="x" size={13} />}
            onClick={() => setOpen(false)}
          />
        </div>
      </header>
      {history ? (
        <HistoryView
          current={conversationId}
          fullView={fullView}
          onBack={showChat}
          onNew={startNew}
          onOpen={(id) => {
            setConversation(id);
            if (id) showChat();
          }}
        />
      ) : (
        <>
          <ChatRenderContext.Provider value={RENDER}>
            <div
              ref={logRef}
              onScroll={() => {
                const el = logRef.current;
                if (el) stick.current = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
              }}
              className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto px-4 pt-5 pb-4"
              aria-live="polite"
              role="log"
              aria-label={t('companheiro-log')}
            >
              {routeEmpty ? (
                <Notice tone="warning" title={t('agente-sem-motor-titulo')}>
                  <NoEngineHint links={LINKS} />
                </Notice>
              ) : null}
              {conversationId && detail.error ? <ErrorBlock error={detail.error} /> : null}
              {chatTimeline(saved, turns).map((item) =>
                item.kind === 'saved' ? (
                  <SavedMessage key={item.message.id} m={item.message} links={LINKS} />
                ) : (
                  <LiveTurnView
                    key={item.turn.id}
                    turn={item.turn}
                    showUser={item.showUser}
                    links={LINKS}
                  />
                ),
              )}
              {empty && !loadingSaved ? (
                <Welcome ctx={ctx} suggestions={suggestions} onAsk={send} busy={running} />
              ) : null}
              <div ref={endRef} />
            </div>
          </ChatRenderContext.Provider>
        </>
      )}
      <div className={cx('flex shrink-0 flex-col gap-2 px-3 pt-1 pb-3', history && 'hidden')}>
        <form
          className="relative flex flex-col gap-1.5 rounded-card border border-line-strong bg-bg p-1.5 transition-[border-color,box-shadow] duration-150 focus-within:border-accent focus-within:shadow-[0_0_0_1px_var(--accent)]"
          onSubmit={(e) => {
            e.preventDefault();
            send(draft);
          }}
          {...composer.drop.props}
        >
          <DropOverlay over={composer.drop.over} />
          {composer.attachments.items.length || composer.unsupported.length ? (
            <div className="flex flex-col gap-1.5 px-1 pt-1">
              <AttachmentTray draft={composer.attachments} unsupported={composer.unsupportedIds} />
              <UnsupportedNotice items={composer.unsupported} engines={candidates} />
              <ImageModeChoice choice={composer.imageChoice} />
            </div>
          ) : null}
          <label className="sr-only" htmlFor="companion-message">
            {t('companheiro-mensagem')}
          </label>
          <textarea
            id="companion-message"
            ref={inputRef}
            rows={2}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={onKey}
            onPaste={composer.onPaste}
            placeholder={t('companheiro-placeholder')}
            className="max-h-48 min-h-[44px] w-full resize-none bg-transparent px-2 pt-1.5 text-[13px] leading-relaxed text-fg outline-none placeholder:text-dim [field-sizing:content]"
          />
          <div className="flex flex-wrap items-center gap-1.5">
            <AttachButton onFiles={composer.attachments.add} disabled={running} compact />
            <EffortSelect
              conversationId={conversationId}
              engine={running && liveEngine ? null : upcoming}
              disabled={running}
              compact
            />
            <ComputerAccessSwitch conversationId={conversationId} disabled={running} compact />
            <span className="flex-1" />
            {running ? (
              <Button
                size="sm"
                variant="danger"
                onClick={() => void chat.cancel()}
                icon={<Icon name="x" size={12} />}
              >
                {t('comum-cancelar')}
              </Button>
            ) : null}
            <Button
              size="sm"
              type="submit"
              variant="primary"
              disabled={!composer.canSend(draft)}
              icon={<Icon name="arrowUp" size={12} />}
            >
              {composer.unsupported.length ? t('companheiro-enviar-assim') : t('agente-enviar')}
            </Button>
          </div>
        </form>
        <ContextChip ctx={ctx} />
        {/* Avisos dos termos de uso (docs/terms.md §5). */}
        <p className="px-1 text-[11.5px] leading-snug text-dim">
          {empty && upcoming && upcoming.kind !== 'local'
            ? t('companheiro-aviso-envio', { provedor: upcoming.provider_label })
            : t('companheiro-aviso')}
        </p>
      </div>
      <CostDialog
        estimate={chat.confirm?.estimate ?? null}
        onConfirm={chat.confirmCost}
        onCancel={chat.dismissConfirm}
      />
    </aside>
  );
}

/** Aba fixa na borda direita para abrir o agente (com o painel fechado). */
export function CompanionLauncher() {
  const open = useCompanion((s) => s.open);
  const setOpen = useCompanion((s) => s.setOpen);
  if (open) return null;
  return (
    <Tooltip content={t('companheiro-rotulo')} side="left">
      <button
        type="button"
        aria-label={t('companheiro-abrir')}
        aria-keyshortcuts="Meta+J Control+J"
        onClick={() => setOpen(true)}
        className="fixed top-1/2 right-0 z-40 flex h-14 w-9 -translate-y-1/2 items-center justify-center rounded-l-card border border-r-0 border-accent/50 bg-surface text-accent shadow-lg transition-[width,border-color] duration-150 hover:w-11 hover:border-accent"
      >
        <Icon name="agent" size={20} className="scanner-swap" />
        <ScannerVoice speaking={false} className="scale-125" />
      </button>
    </Tooltip>
  );
}
