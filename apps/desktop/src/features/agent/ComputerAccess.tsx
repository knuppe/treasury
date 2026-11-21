/**
 * Liberação, por conversa, da leitura do computador pelo agente. Desligada por padrão: o
 * agente só usa as ferramentas do Treasury. Ligada, CLIs locais (Claude Code, Gemini CLI,
 * OpenCode) podem ler arquivos da pasta pessoal (sem terminal, escrita ou web); motores de
 * API e locais nunca têm ferramentas do computador. Vale para a conversa em que foi ligada, só
 * na memória da sessão, e some ao bloquear o cofre.
 */
import { Icon, Tooltip, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { useAgentSession, useComputerAccess } from '@/state/agentSession';

export function ComputerAccessSwitch({
  conversationId,
  disabled,
  compact,
}: {
  conversationId: string | null;
  disabled?: boolean;
  compact?: boolean;
}) {
  const on = useComputerAccess(conversationId);
  const set = useAgentSession((s) => s.setComputer);
  return (
    <Tooltip
      content={on ? t('agente-computador-ligado-dica') : t('agente-computador-desligado-dica')}
    >
      <button
        type="button"
        role="switch"
        aria-checked={on}
        aria-label={t('agente-computador-rotulo')}
        disabled={disabled}
        onClick={() => {
          set(conversationId, !on);
          toast({
            tone: 'info',
            title: on ? t('agente-computador-desligado') : t('agente-computador-liberado'),
            description: on ? t('agente-computador-volta') : t('agente-computador-ligado-dica'),
          });
        }}
        className={cx(
          'inline-flex h-7 shrink-0 items-center gap-1.5 rounded-control border px-2 text-[11px] whitespace-nowrap transition-colors disabled:opacity-50',
          on ? 'border-warning/60 text-warning' : 'border-line text-dim hover:text-fg',
        )}
      >
        <Icon name={on ? 'unlock' : 'lock'} size={11} />
        {compact
          ? t('agente-computador')
          : on
            ? t('agente-computador-leitura')
            : t('agente-computador-bloqueado')}
      </button>
    </Tooltip>
  );
}
