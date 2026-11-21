/**
 * Tempo real: o núcleo avisa (`data://changed`) quando o agente, um cliente MCP ou uma
 * rotina grava algo, e as telas abertas relêem só os módulos afetados. Escritas da própria
 * interface já invalidam o que mudaram, então são ignoradas aqui.
 */
import { useQueryClient, type QueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';

import type { DataEvent } from '../generated/DataEvent';
import { toast } from '@/design/components';
import { t } from '@/i18n';
import { listenApp } from '@/features/agent/tauriBridge';
import { AFFECTS, invalidateAll, invalidateGroups, type AffectGroup } from './coreQuery';

/** Módulo do comando → grupos de consultas a reler. */
const GROUPS: Record<string, readonly AffectGroup[] | 'all'> = {
  transactions: ['ledger'],
  import: ['ledger'],
  rules: ['rules'],
  recurrences: ['ledger'],
  cards: ['ledger'],
  installments: ['installments'],
  budget: ['ledger'],
  goals: ['ledger'],
  accounts: ['accounts'],
  categories: ['categories'],
  household: ['household'],
  portfolio: ['market'],
  assets: ['market'],
  notes: ['market'],
  b3: ['market'],
  market: ['market'],
  indices: ['market'],
  fx: ['fx'],
  debt: ['debt'],
  physical: ['physical'],
  tax: ['tax'],
  tax_burden: ['tax_burden'],
  settings: ['settings'],
  sources: ['sources'],
  database: ['database'],
  crud: 'all',
  changes: 'all',
};

const DEBOUNCE_MS = 400;

/** Agrupa os avisos de uma rajada e relê cada módulo uma vez. */
export function makeDataEventHandler(client: QueryClient) {
  let pending = new Set<string>();
  let timer: ReturnType<typeof setTimeout> | null = null;
  const flush = () => {
    timer = null;
    const modules = pending;
    pending = new Set();
    const groups = new Set<AffectGroup>();
    let all = false;
    for (const m of modules) {
      const g = GROUPS[m];
      if (g === 'all') all = true;
      else if (g) g.forEach((x) => groups.add(x));
    }
    if (all) {
      void invalidateAll(client);
      return;
    }
    void invalidateGroups(client, [...groups]);
    // Módulos sem grupo (agente, análise…) e a fila de aprovação, sempre.
    const covered = new Set<string>([...groups].flatMap((g) => [...AFFECTS[g]]));
    for (const m of [...modules, 'ai']) {
      if (!covered.has(m)) void client.invalidateQueries({ queryKey: [m] });
    }
  };
  return (e: DataEvent) => {
    if (e.actor === 'user') return;
    // Cliente MCP novo neste cofre (docs/local-security.md, item 14): avisa já, em qualquer tela.
    if (e.cmd === 'mcp.client_new')
      toast({ tone: 'warning', title: t('ajustes-mcp-cliente-novo-toast') });
    pending.add(e.module);
    if (!timer) timer = setTimeout(flush, DEBOUNCE_MS);
  };
}

export function useDataEvents(): void {
  const client = useQueryClient();
  useEffect(() => listenApp<DataEvent>('data://changed', makeDataEventHandler(client)), [client]);
}
