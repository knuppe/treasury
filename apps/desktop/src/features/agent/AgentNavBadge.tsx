/**
 * Contador do item Agente na barra lateral: propostas esperando aprovação + alertas
 * novos. Também escuta `ai://alerts` (rotinas em segundo plano do app) para atualizar.
 */
import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import { aiKeys, useAlertCounts, usePendingCount } from '@/api/hooks/ai';
import { t } from '@/i18n';

import { onNewAlerts } from './tauriBridge';

/** Invalida alertas e contadores quando o app avisa de alertas novos. */
export function useAlertEvents(): void {
  const client = useQueryClient();
  useEffect(
    () =>
      onNewAlerts(() => {
        void client.invalidateQueries({ queryKey: ['ai', 'alerts'] });
        void client.invalidateQueries({ queryKey: aiKeys.alertCount });
        void client.invalidateQueries({ queryKey: aiKeys.routines });
        void client.invalidateQueries({ queryKey: ['attention'] });
      }),
    [client],
  );
}

export function AgentNavBadge() {
  useAlertEvents();
  const pending = usePendingCount();
  const alerts = useAlertCounts();
  const p = pending.data ?? 0;
  const a = alerts.data?.new ?? 0;
  if (p + a === 0) return null;
  const parts = [
    p ? t('agente-badge-propostas', { n: p }) : null,
    a ? t('agente-badge-alertas', { n: a }) : null,
  ].filter(Boolean);
  return (
    <span
      title={parts.join(' · ')}
      className="num inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-accent px-1 text-[10px] font-medium text-accent-ink"
    >
      <span aria-hidden>{p + a}</span>
      <span className="sr-only">{parts.join(', ')}</span>
    </span>
  );
}
