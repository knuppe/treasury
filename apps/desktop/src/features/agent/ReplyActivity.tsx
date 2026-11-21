/**
 * O que o agente fez antes de responder, recolhido numa linha só para não poluir a
 * conversa: o raciocínio ("Raciocínio · 12 s") e as ferramentas usadas ("Consultou 3
 * ferramentas"), lado a lado, cada um abre o detalhe logo abaixo. Enquanto a resposta chega,
 * a linha mostra "Pensando…" ou a ferramenta em andamento e a última linha do raciocínio;
 * ao terminar, fica só o resumo. Quem abriu continua aberto (por mensagem, em memória).
 */
import { useId } from 'react';

import { Icon, LiveIndicator, Markdown, Private } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { useAgentSession } from '@/state/agentSession';

import { Pill } from './ui';

export interface ActivityTool {
  name: string;
  /** Parâmetros resumidos. */
  args: string;
  /** `null` = ainda rodando. */
  result: { ok: boolean; summary: string } | null;
}

/** Ferramentas nativas de uma CLI chegam como `cli.<nome>`. */
const CLI_PREFIX = 'cli.';

export function formatSeconds(ms: number): string {
  const s = ms / 1000;
  if (s < 1) return t('agente-menos-1-s');
  if (s < 60) return t('agente-segundos', { s: Math.round(s) });
  const m = Math.floor(s / 60);
  const r = Math.round(s % 60);
  return r ? t('agente-minutos-segundos', { m, s: r }) : t('agente-minutos', { m });
}

function useExpanded(key: string): [boolean, (open: boolean) => void] {
  const open = useAgentSession((s) => Boolean(s.expanded[key]));
  const set = useAgentSession((s) => s.setExpanded);
  return [open, (o) => set(key, o)];
}

function Disclosure({
  label,
  open,
  onToggle,
  controls,
  live,
  tone,
}: {
  label: string;
  open: boolean;
  onToggle: () => void;
  controls: string;
  live?: boolean;
  tone?: 'warning';
}) {
  return (
    <button
      type="button"
      aria-expanded={open}
      aria-controls={controls}
      onClick={onToggle}
      className={cx(
        'flex max-w-full items-center gap-1.5 rounded-control px-1 py-0.5 text-left text-[12px] hover:bg-surface-2',
        tone === 'warning' ? 'text-warning' : 'text-dim hover:text-fg',
      )}
    >
      <Icon
        name="chevronRight"
        size={11}
        className={cx('shrink-0 transition-transform', open && 'rotate-90')}
      />
      {live ? <LiveIndicator label={label} tone="accent" /> : <span>{label}</span>}
    </button>
  );
}

function lastLine(text: string): string {
  const lines = text
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean);
  return lines[lines.length - 1] ?? '';
}

function ToolRow({ tool }: { tool: ActivityTool }) {
  const native = tool.name.startsWith(CLI_PREFIX);
  const name = native ? tool.name.slice(CLI_PREFIX.length) : tool.name;
  const r = tool.result;
  return (
    <li
      className="flex flex-col gap-0.5 py-1.5"
      aria-label={t('agente-ferramenta-nome', { nome: name })}
    >
      <div className="flex min-w-0 items-center gap-2">
        {r == null ? (
          <LiveIndicator label="" tone="accent" />
        ) : r.ok ? (
          <span aria-label={t('agente-ok')} className="text-[11px] text-positive">
            {'✓'}
          </span>
        ) : (
          <span aria-label={t('agente-erro')} className="text-[11px] text-negative">
            {'×'}
          </span>
        )}
        <span className="shrink-0 font-mono text-[12px] text-fg">{name}</span>
        {native ? (
          <Pill tone="warning" title={t('agente-ferramenta-cli')}>
            {'CLI'}
          </Pill>
        ) : null}
        {tool.args && tool.args !== '{}' ? (
          <code className="min-w-0 truncate font-mono text-[11px] text-dim">
            <Private>{tool.args}</Private>
          </code>
        ) : null}
      </div>
      {r?.summary ? (
        <Private>
          <p className={cx('pl-5 text-[11.5px] leading-snug', r.ok ? 'text-dim' : 'text-negative')}>
            {r.summary}
          </p>
        </Private>
      ) : null}
    </li>
  );
}

export function ReplyActivity({
  id,
  reasoning,
  reasoningMs,
  tools,
  running,
  answering,
}: {
  /** Chave da mensagem (estado de aberto/fechado). */
  id: string;
  reasoning: string[];
  reasoningMs: number | null;
  tools: ActivityTool[];
  /** A resposta ainda está chegando. */
  running: boolean;
  /** Já há texto de resposta na tela (some o "Pensando…"). */
  answering: boolean;
}) {
  const uid = useId();
  const [reasoningOpen, setReasoningOpen] = useExpanded(`${id}:reasoning`);
  const [toolsOpen, setToolsOpen] = useExpanded(`${id}:tools`);
  const openTool = running ? [...tools].reverse().find((t) => t.result == null) : undefined;
  const failed = tools.filter((t) => t.result && !t.result.ok).length;
  const thinking = running && !answering && !openTool;
  const hasReasoning = reasoning.length > 0;
  if (!hasReasoning && tools.length === 0 && !thinking) return null;

  const reasoningLabel = thinking
    ? t('agente-pensando')
    : reasoningMs != null && !running
      ? t('agente-raciocinio-tempo', { tempo: formatSeconds(reasoningMs) })
      : t('agente-raciocinio');
  const toolsLabel = openTool
    ? t('agente-consultando', { ferramenta: openTool.name.replace(CLI_PREFIX, '') })
    : failed
      ? t('agente-consultou-erro', { n: tools.length, erros: failed })
      : t('agente-consultou', { n: tools.length });
  const preview = thinking && !reasoningOpen ? lastLine(reasoning[reasoning.length - 1] ?? '') : '';

  return (
    <div className="flex min-w-0 flex-col gap-1" role="group" aria-label={t('agente-atividade')}>
      <div className="flex min-w-0 flex-wrap items-center gap-x-1 gap-y-0.5">
        {thinking ? (
          // No Fósforo, o "Pensando…" vira a linha do terminal (phosphor.css).
          <p className="phosphor-only phosphor-terminal text-[12px]" aria-hidden>
            {t('casca-phosphor-agente')}
            <span className="phosphor-cursor" />
          </p>
        ) : null}
        {hasReasoning || thinking ? (
          <span className={thinking ? 'phosphor-hide' : undefined}>
            <Disclosure
              label={reasoningLabel}
              open={reasoningOpen}
              onToggle={() => setReasoningOpen(!reasoningOpen)}
              controls={`${uid}-r`}
              live={thinking}
            />
          </span>
        ) : null}
        {(hasReasoning || thinking) && tools.length > 0 ? (
          <span aria-hidden className="text-[11px] text-line-strong">
            {'·'}
          </span>
        ) : null}
        {tools.length > 0 ? (
          <Disclosure
            label={toolsLabel}
            open={toolsOpen}
            onToggle={() => setToolsOpen(!toolsOpen)}
            controls={`${uid}-t`}
            live={Boolean(openTool)}
            tone={failed && !openTool ? 'warning' : undefined}
          />
        ) : null}
      </div>
      {preview ? (
        <Private>
          <p className="truncate pl-5 text-[11.5px] text-dim italic" aria-hidden>
            {preview}
          </p>
        </Private>
      ) : null}
      {reasoningOpen && hasReasoning ? (
        <div
          id={`${uid}-r`}
          role="region"
          aria-label={t('agente-raciocinio-agente')}
          className="ml-1.5 flex flex-col gap-2 border-l border-line py-0.5 pl-3"
        >
          {reasoning.map((r, i) => (
            <Private key={i}>
              <Markdown text={r} tone="dim" />
            </Private>
          ))}
        </div>
      ) : null}
      {toolsOpen && tools.length > 0 ? (
        <ul
          id={`${uid}-t`}
          aria-label={t('agente-ferramentas-usadas')}
          className="ml-1.5 divide-y divide-line border-l border-line pl-3"
        >
          {tools.map((x, i) => (
            <ToolRow key={i} tool={x} />
          ))}
        </ul>
      ) : null}
    </div>
  );
}
