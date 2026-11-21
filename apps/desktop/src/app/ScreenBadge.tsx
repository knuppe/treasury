/**
 * Contadores ao lado do nome de uma tela na barra lateral. Cada tela que precisar de um
 * contador acrescenta o componente aqui (carregado só quando o cofre está aberto).
 */
import type { ComponentType } from 'react';

import { useQuoteEvents } from '@/api/hooks/market';
import { AgentNavBadge } from '@/features/agent/AgentNavBadge';

/** Sem contador: só escuta as cotações intradiárias novas da rotina do app. */
function QuotesListener() {
  useQuoteEvents();
  return null;
}

const BADGES: Partial<Record<string, ComponentType>> = {
  agente: AgentNavBadge,
  ativos: QuotesListener,
};

export function ScreenBadge({ screenId }: { screenId: string }) {
  const Badge = BADGES[screenId];
  return Badge ? <Badge /> : null;
}
