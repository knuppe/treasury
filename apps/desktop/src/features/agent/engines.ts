/** Regras de prontidão dos motores (espelho de `Engine::ready` no núcleo). */
import type { Engine } from '@/api/generated/Engine';
import type { TaskRoute } from '@/api/generated/TaskRoute';
import type { AiTask } from '@/api/generated/AiTask';
import { t } from '@/i18n';

/** Motivo de o motor não estar pronto, ou `null` se está. */
export function engineProblem(e: Engine): string | null {
  if (!e.enabled) return t('agente-problema-desligado');
  if (e.needs_key && (!e.secret.set || e.secret.needs_reentry))
    return t('agente-problema-sem-chave');
  if (e.kind === 'cli' && !e.effective_template) return t('agente-problema-sem-template');
  if (e.kind !== 'cli' && !e.model) return t('agente-problema-sem-modelo');
  return null;
}

/**
 * Motores da rota de uma tarefa, na ordem, só os ligados (como `routing::candidates`):
 * sem rota própria, tarefas que não são chat usam a rota de chat.
 */
export function routeEngines(
  routes: readonly TaskRoute[],
  engines: readonly Engine[],
  task: AiTask = 'chat',
): Engine[] {
  const ids = (k: AiTask) => routes.find((r) => r.task === k)?.engine_ids ?? [];
  let list = ids(task);
  if (list.length === 0 && task !== 'chat') list = ids('chat');
  return list
    .map((id) => engines.find((e) => e.id === id))
    .filter((e): e is Engine => Boolean(e?.enabled));
}

/** O motor que vai receber os dados: o primeiro pronto da rota (ou o escolhido). */
export function nextEngine(
  routes: readonly TaskRoute[],
  engines: readonly Engine[],
  explicitId?: string | null,
): Engine | null {
  if (explicitId) return engines.find((e) => e.id === explicitId) ?? null;
  return routeEngines(routes, engines).find((e) => engineProblem(e) == null) ?? null;
}
