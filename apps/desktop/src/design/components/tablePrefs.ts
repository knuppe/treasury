import { useCallback, useState } from 'react';

import type { SortState } from './dataColumns';

/**
 * Preferências de uma tabela, lembradas neste computador: ordenação, larguras (px por
 * coluna) e colunas escondidas. Sem armazenamento (janela privada, prévia), vale até
 * fechar.
 */
export interface TablePrefs {
  sort?: SortState | null;
  widths?: Record<string, number>;
  hidden?: string[];
}

const KEY = 'treasury.table.';

/** Chave estável a partir do nome da tabela ("Operações de PETR4" → "operacoes-de-petr4"). */
export const tableKey = (name: string) =>
  name
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');

function load(id: string): TablePrefs {
  try {
    const raw = localStorage.getItem(KEY + id);
    const v: unknown = raw ? JSON.parse(raw) : null;
    if (v && typeof v === 'object') return v;
  } catch {
    // Sem armazenamento: começa do padrão.
  }
  return {};
}

function save(id: string, prefs: TablePrefs) {
  try {
    if (!prefs.sort && !prefs.hidden?.length && !Object.keys(prefs.widths ?? {}).length)
      localStorage.removeItem(KEY + id);
    else localStorage.setItem(KEY + id, JSON.stringify(prefs));
  } catch {
    // Sem armazenamento: vale até fechar.
  }
}

export function useTablePrefs(id: string) {
  const [state, setState] = useState<{ id: string; prefs: TablePrefs }>(() => ({
    id,
    prefs: load(id),
  }));
  // Outra tabela no mesmo lugar (ex.: troca de ativo): lê as preferências dela.
  let prefs = state.prefs;
  if (state.id !== id) {
    prefs = load(id);
    setState({ id, prefs });
  }
  const update = useCallback(
    (patch: Partial<TablePrefs> | ((p: TablePrefs) => Partial<TablePrefs>), persist = true) =>
      setState((cur) => {
        const next = {
          ...cur.prefs,
          ...(typeof patch === 'function' ? patch(cur.prefs) : patch),
        };
        if (persist) save(cur.id, next);
        return { id: cur.id, prefs: next };
      }),
    [],
  );
  const reset = useCallback(
    () =>
      setState((cur) => {
        save(cur.id, {});
        return { id: cur.id, prefs: {} };
      }),
    [],
  );
  return { prefs, update, reset };
}
