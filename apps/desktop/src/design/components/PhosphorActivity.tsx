import { useEffect, useState } from 'react';

import { t } from '@/i18n';
import { useActiveTasks, useActivity } from '@/state/activity';

import { Tooltip } from './Tooltip';

const FAIL_MS = 3000;

/**
 * Chip de atividade do Fósforo na barra superior (phosphor.css): `▮ PROCESSANDO_` com o cursor
 * piscando enquanto alguma tarefa roda, `▮ 2 TAREFAS_` com mais de uma e `▮ FALHA_` por 3s
 * quando a última termina com erro. A dica lista o que está rodando. Fora do Fósforo o chip
 * não aparece (`--phosphor-display`); com movimento reduzido, o texto fica parado.
 *
 * A região `aria-live` fica sempre montada, para o leitor de tela anunciar quando o texto
 * entra.
 */
export function PhosphorActivity() {
  const tasks = useActiveTasks();
  const last = useActivity((s) => s.last);
  // A falha que já passou dos 3s (pela hora em que terminou).
  const [cleared, setCleared] = useState<number | null>(null);
  const failed = last?.status === 'failed' && cleared !== last.at;

  useEffect(() => {
    if (!last || last.status !== 'failed') return;
    const timer = window.setTimeout(() => setCleared(last.at), FAIL_MS);
    return () => window.clearTimeout(timer);
  }, [last]);

  const n = tasks.length;
  const text =
    n > 1
      ? t('casca-phosphor-tarefas', { n })
      : n === 1
        ? t('casca-phosphor-processando')
        : failed
          ? t('casca-phosphor-falha')
          : null;

  return (
    <span className="phosphor-activity" aria-live="polite">
      {text ? (
        <Tooltip
          content={
            n ? (
              <ul className="flex flex-col gap-0.5">
                {tasks.map((x) => (
                  <li key={x.id}>{x.label}</li>
                ))}
              </ul>
            ) : (
              (last?.label ?? text)
            )
          }
        >
          <span tabIndex={0} className="phosphor-chip" data-state={n ? 'busy' : 'failed'}>
            {text}
            <span aria-hidden className="phosphor-cursor" />
          </span>
        </Tooltip>
      ) : null}
    </span>
  );
}
