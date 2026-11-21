/**
 * Ponte com o app Tauri para o agente: o evento `ai://alerts` (alertas novos das rotinas
 * em segundo plano). Fora do Tauri (navegador de desenvolvimento, testes) os listeners não
 * disparam.
 */
import { isTauri } from '@/api/client';
import type { Alert } from '@/api/generated/Alert';

export type Unlisten = () => void;

/** Assina um evento do app. Devolve a função de cancelar (no-op fora do Tauri). */
export function listenApp<T>(event: string, handler: (payload: T) => void): Unlisten {
  if (!isTauri()) return () => {};
  let off: Unlisten | null = null;
  let cancelled = false;
  void import('@tauri-apps/api/event')
    .then(({ listen }) => listen<T>(event, (e) => handler(e.payload)))
    .then((fn) => {
      if (cancelled) fn();
      else off = fn;
    })
    .catch(() => {});
  return () => {
    cancelled = true;
    off?.();
  };
}

export const onNewAlerts = (handler: (alerts: Alert[]) => void): Unlisten =>
  listenApp<Alert[]>('ai://alerts', handler);
