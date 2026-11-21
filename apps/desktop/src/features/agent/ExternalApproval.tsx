/**
 * Pedido de confirmação de uma ferramenta de MCP extra, no fio da conversa: o agente para
 * até a pessoa permitir ou recusar (`ai.mcp_extra.decide`). Depois da decisão (o clique ou
 * o evento `external_decision`, que também chega quando o pedido expira), o cartão mostra
 * só o resultado.
 */
import { useState } from 'react';

import { useDecideExternal } from '@/api/hooks/ai';
import { Button, Icon, Private } from '@/design/components';
import { errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import type { StreamPart } from './chatStream';
import { Mono, Pill } from './ui';

type ExternalPart = Extract<StreamPart, { type: 'external' }>;

/** Parâmetros acima disso começam recolhidos. */
const LONG_ARGS = 160;

export function ExternalApprovalCard({
  part,
  running,
}: {
  part: ExternalPart;
  /** O job ainda roda (sem ele, um pedido sem resposta não aceita mais clique). */
  running: boolean;
}) {
  const decide = useDecideExternal();
  const [chosen, setChosen] = useState<{ allow: boolean; conversation: boolean } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const long = part.args.length > LONG_ARGS || part.args.split('\n').length > 3;
  const [expanded, setExpanded] = useState(false);
  const outcome = part.allowed ?? chosen?.allow ?? null;
  const remembered = part.allowed != null ? part.remembered : (chosen?.conversation ?? false);
  const answer = (allow: boolean, conversation = false) => {
    setError(null);
    decide.mutate(
      {
        request_id: part.requestId,
        allow,
        ...(conversation ? { scope: 'conversation' as const } : {}),
      },
      {
        onSuccess: () => setChosen({ allow, conversation }),
        onError: (e) => setError(errorMessage(e)),
      },
    );
  };
  const pending = (allow: boolean, conversation: boolean) =>
    decide.isPending &&
    decide.variables.allow === allow &&
    (decide.variables.scope === 'conversation') === conversation;
  return (
    <div
      role="group"
      aria-label={t('agente-mcpx-pedido-rotulo')}
      className="flex flex-col gap-2 rounded-control border border-l-2 border-line border-l-warning bg-surface-2 px-3 py-2.5 text-[12.5px]"
    >
      <p className="flex items-start gap-2 text-fg">
        <Icon name="lock" size={12} className="mt-0.5 shrink-0 text-warning" />
        <span>{t('agente-mcpx-pedido', { ferramenta: part.tool, servidor: part.server })}</span>
      </p>
      {part.args ? (
        <div className="flex flex-col gap-1">
          <span className="label-caps text-[10px]">{t('agente-mcpx-parametros')}</span>
          <Private>
            <Mono
              className={
                long && !expanded
                  ? 'line-clamp-3 text-dim'
                  : 'block max-h-64 overflow-auto text-dim'
              }
            >
              {part.args}
            </Mono>
          </Private>
          {long ? (
            <div>
              <Button
                size="sm"
                variant="ghost"
                aria-expanded={expanded}
                onClick={() => setExpanded((v) => !v)}
              >
                {expanded ? t('agente-mcpx-recolher') : t('agente-mcpx-ver-tudo')}
              </Button>
            </div>
          ) : null}
        </div>
      ) : null}
      {outcome != null ? (
        <p role="status">
          <Pill tone={outcome ? 'positive' : 'negative'}>
            {outcome
              ? remembered
                ? t('agente-mcpx-permitida-conversa', { servidor: part.server })
                : t('agente-mcpx-permitida')
              : t('agente-mcpx-recusada')}
          </Pill>
        </p>
      ) : running ? (
        <div className="flex flex-wrap items-center gap-2">
          <Button
            size="sm"
            variant="primary"
            loading={pending(true, false)}
            disabled={decide.isPending}
            onClick={() => answer(true)}
          >
            {t('agente-mcpx-permitir')}
          </Button>
          <Button
            size="sm"
            variant="secondary"
            loading={pending(true, true)}
            disabled={decide.isPending}
            title={t('agente-mcpx-permitir-conversa-dica', { servidor: part.server })}
            onClick={() => answer(true, true)}
          >
            {t('agente-mcpx-permitir-conversa')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            loading={pending(false, false)}
            disabled={decide.isPending}
            onClick={() => answer(false)}
          >
            {t('agente-mcpx-recusar')}
          </Button>
          <span className="text-[12px] text-dim">{t('agente-mcpx-aguardando')}</span>
        </div>
      ) : null}
      {error ? (
        <p role="alert" className="text-[12px] text-negative">
          {`${t('agente-mcpx-nao-respondeu')}: ${error}`}
        </p>
      ) : null}
    </div>
  );
}
