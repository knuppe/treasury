/**
 * Esforço de raciocínio da conversa ("Esforço: Padrão / Baixo / Médio / Alto / Máximo"),
 * guardado só na sessão (`agentSession`). O núcleo traduz para cada motor (orçamento de
 * raciocínio da Anthropic e do Gemini, `reasoning_effort` da OpenAI e da xAI, `--effort` das
 * CLIs); onde o motor ou o modelo não tem esse controle, a opção fica desativada.
 */
import * as RSelect from '@radix-ui/react-select';

import type { Effort } from '@/api/generated/Effort';
import type { Engine } from '@/api/generated/Engine';
import { Icon, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { useAgentSession, useConversationEffort } from '@/state/agentSession';

import { labels } from '../format';

export const EFFORT_LABEL: Record<Effort, string> = labels({
  default: 'agente-esforco-padrao',
  low: 'agente-esforco-baixo',
  medium: 'agente-esforco-medio',
  high: 'agente-esforco-alto',
  max: 'agente-esforco-maximo',
});

const EFFORT_HINT: Record<Effort, string> = labels({
  default: 'agente-esforco-padrao-dica',
  low: 'agente-esforco-baixo-dica',
  medium: 'agente-esforco-medio-dica',
  high: 'agente-esforco-alto-dica',
  max: 'agente-esforco-maximo-dica',
});

const ORDER: Effort[] = ['default', 'low', 'medium', 'high', 'max'];

/** O motor que vai responder aceita o controle de esforço? (`null` = ainda não se sabe.) */
export function effortSupported(engine: Pick<Engine, 'caps'> | null | undefined): boolean {
  return engine ? engine.caps.effort : true;
}

export function EffortSelect({
  conversationId,
  engine,
  disabled,
  compact,
}: {
  conversationId: string | null;
  /** Motor que vai receber o próximo turno (o escolhido ou o primeiro da rota). */
  engine: Pick<Engine, 'name' | 'caps'> | null;
  disabled?: boolean;
  compact?: boolean;
}) {
  const effort = useConversationEffort(conversationId);
  const set = useAgentSession((s) => s.setEffort);
  const supported = effortSupported(engine);
  const tip = supported
    ? t('agente-esforco-dica')
    : t('agente-esforco-sem-controle', { motor: engine?.name ?? t('agente-este-motor') });
  const label = supported ? EFFORT_LABEL[effort] : EFFORT_LABEL.default;
  return (
    <RSelect.Root
      value={supported ? effort : 'default'}
      onValueChange={(v) => set(conversationId, v as Effort)}
      disabled={disabled || !supported}
    >
      <Tooltip content={tip}>
        {/* O span recebe a dica mesmo com o gatilho desativado. */}
        <span className="inline-flex">
          <RSelect.Trigger
            aria-label={t('agente-esforco-rotulo')}
            className={cx(
              'inline-flex h-7 shrink-0 items-center gap-1.5 rounded-control border px-2 text-[11px] whitespace-nowrap outline-none transition-colors focus-visible:border-accent disabled:cursor-not-allowed disabled:opacity-50',
              supported && effort !== 'default'
                ? 'border-accent/60 text-accent'
                : 'border-line text-dim hover:text-fg',
            )}
          >
            <Icon name="contrast" size={11} />
            {compact ? label : t('agente-esforco-valor', { nivel: label })}
            <Icon name="chevronDown" size={10} />
          </RSelect.Trigger>
        </span>
      </Tooltip>
      <RSelect.Portal>
        <RSelect.Content
          position="popper"
          sideOffset={4}
          collisionPadding={8}
          className="z-[60] min-w-[200px] animate-pop-in overflow-hidden rounded-card border border-panel pop-surface"
        >
          <RSelect.Viewport className="p-1">
            {ORDER.map((v) => (
              <RSelect.Item
                key={v}
                value={v}
                className="relative flex h-8 cursor-default select-none items-center gap-3 rounded-control pr-2 pl-7 text-[13px] text-fg outline-none data-[highlighted]:bg-accent-soft data-[highlighted]:text-accent"
              >
                <RSelect.ItemIndicator className="absolute left-2 inline-flex">
                  <Icon name="check" size={13} />
                </RSelect.ItemIndicator>
                <RSelect.ItemText>{EFFORT_LABEL[v]}</RSelect.ItemText>
                <span className="ml-auto text-[11px] text-dim">{EFFORT_HINT[v]}</span>
              </RSelect.Item>
            ))}
          </RSelect.Viewport>
        </RSelect.Content>
      </RSelect.Portal>
    </RSelect.Root>
  );
}
