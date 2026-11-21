/**
 * Roteamento por tarefa: para cada tarefa, os motores na ordem de tentativa (o próximo
 * entra se o anterior falhar, estiver sem chave ou sem cota). Cada mudança grava na hora.
 */
import type { AiTask } from '@/api/generated/AiTask';
import type { Engine } from '@/api/generated/Engine';
import type { TaskRoute } from '@/api/generated/TaskRoute';
import { useEngines, useRoutes, useSetRoute } from '@/api/hooks/ai';
import { Icon, IconButton, InstrumentCard, Select, toast } from '@/design/components';
import { ErrorBlock, LoadingBlock, errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import { engineProblem } from '../engines';
import { ENGINE_KIND_LABEL, TASK_LABEL, labels } from '../format';
import { Pill } from '../ui';

const ADD = '__add__';

/** Move o item `i` de `ids` por `delta` posições (sem sair da lista). */
export function moveId(ids: readonly string[], i: number, delta: number): string[] {
  const j = i + delta;
  if (j < 0 || j >= ids.length) return [...ids];
  const next = [...ids];
  [next[i], next[j]] = [next[j]!, next[i]!];
  return next;
}

const STRICT_NOTE: Partial<Record<AiTask, string>> = labels({
  chat: 'agente-rota-chat-nota',
  monthly_close: 'agente-rota-rotina-nota',
  alerts: 'agente-rota-rotina-nota',
  categorize: 'agente-rota-categorizacao-nota',
  study: 'agente-rota-estudo-nota',
});

function RouteRow({ route, engines }: { route: TaskRoute; engines: readonly Engine[] }) {
  const set = useSetRoute();
  const save = (ids: string[]) =>
    set.mutate(
      { task: route.task, engine_ids: ids },
      {
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('agente-nao-salvou-rota'),
            description: errorMessage(e),
          }),
      },
    );
  const ids = route.engine_ids;
  const available = engines.filter((e) => !ids.includes(e.id));
  const label = TASK_LABEL[route.task] ?? route.label;
  return (
    <li aria-label={t('agente-rota', { tarefa: label })} className="flex flex-col gap-2 px-4 py-3">
      <div className="flex flex-wrap items-baseline gap-2">
        <h3 className="text-[13.5px] font-medium text-fg">{label}</h3>
        {STRICT_NOTE[route.task] ? (
          <span className="text-[11.5px] text-dim">{STRICT_NOTE[route.task]}</span>
        ) : null}
      </div>
      {ids.length === 0 ? (
        <p className="text-[12.5px] text-dim">{t('agente-sem-motor-rota')}</p>
      ) : (
        <ol className="flex flex-col gap-1">
          {ids.map((id, i) => {
            const e = engines.find((x) => x.id === id);
            const problem = e ? engineProblem(e) : t('agente-motor-removido');
            return (
              <li
                key={id}
                className="flex items-center gap-2 rounded-control border border-line bg-bg/40 px-2 py-1"
              >
                <span className="num w-5 text-[11px] text-accent">{i + 1}</span>
                <span className="min-w-0 flex-1 truncate text-[13px] text-fg">{e?.name ?? id}</span>
                {e ? (
                  <span className="text-[11px] text-dim">{ENGINE_KIND_LABEL[e.kind]}</span>
                ) : null}
                {problem ? <Pill tone="negative">{problem}</Pill> : null}
                <IconButton
                  size="sm"
                  label={t('agente-subir', { motor: e?.name ?? id })}
                  icon={<Icon name="arrowUp" />}
                  disabled={i === 0 || set.isPending}
                  onClick={() => save(moveId(ids, i, -1))}
                />
                <IconButton
                  size="sm"
                  label={t('agente-descer', { motor: e?.name ?? id })}
                  icon={<Icon name="arrowDown" />}
                  disabled={i === ids.length - 1 || set.isPending}
                  onClick={() => save(moveId(ids, i, 1))}
                />
                <IconButton
                  size="sm"
                  label={t('agente-tirar-rota', { motor: e?.name ?? id })}
                  icon={<Icon name="x" />}
                  disabled={set.isPending}
                  onClick={() => save(ids.filter((x) => x !== id))}
                />
              </li>
            );
          })}
        </ol>
      )}
      {available.length > 0 ? (
        <div className="w-[280px]">
          <Select
            aria-label={t('agente-adicionar-motor-rota', { tarefa: label })}
            value={ADD}
            onValueChange={(v) => {
              if (v !== ADD) save([...ids, v]);
            }}
            options={[
              {
                value: ADD,
                label: ids.length
                  ? t('agente-adicionar-fallback')
                  : t('agente-adicionar-motor-mais'),
                disabled: true,
              },
              ...available.map((e) => ({
                value: e.id,
                label: e.name,
                hint: ENGINE_KIND_LABEL[e.kind],
              })),
            ]}
          />
        </div>
      ) : null}
    </li>
  );
}

export function RoutesPanel({ index = '02' }: { index?: string }) {
  const routes = useRoutes();
  const engines = useEngines();
  return (
    <InstrumentCard index={index} title={t('agente-roteamento')} padded={false}>
      {routes.isLoading || engines.isLoading ? (
        <LoadingBlock />
      ) : routes.error || engines.error ? (
        <ErrorBlock error={routes.error ?? engines.error} className="m-3" />
      ) : (
        <ul aria-label={t('agente-rotas')} className="divide-y divide-line">
          {(routes.data ?? []).map((r) => (
            <RouteRow key={r.task} route={r} engines={engines.data ?? []} />
          ))}
        </ul>
      )}
    </InstrumentCard>
  );
}
