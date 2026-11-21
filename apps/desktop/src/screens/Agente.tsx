/**
 * 11 · AGENTE: conversa com o gestor financeiro, fila de aprovação com diff, alertas e
 * rotinas, histórico de ferramentas, memória (do gestor e de procedimentos do agente) e a
 * configuração dos motores de IA.
 *
 * Estado na URL: `?aba=` (conversa | aprovacao | alertas | ferramentas | memoria |
 * configurar) e `&proposta=<id>` destaca uma proposta na fila.
 */
import { useCallback } from 'react';
import { useSearchParams } from 'react-router';

import { screenMeta } from '@/app/screens';
import { useAlertCounts, useEngines, usePendingCount } from '@/api/hooks/ai';
import { ScreenHeader, Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import { AlertsPanel } from '@/features/agent/AlertsPanel';
import { ApprovalQueue } from '@/features/agent/ApprovalQueue';
import { ChatPanel, type ChatLinks } from '@/features/agent/ChatPanel';
import { MemoryPanel } from '@/features/agent/MemoryPanel';
import { ProceduresPanel } from '@/features/agent/ProceduresPanel';
import { AiSettingsSection } from '@/features/agent/settings';
import { ToolHistory } from '@/features/agent/ToolHistory';
import { usePageContext } from '@/features/companion/pageContext';
import { t } from '@/i18n';

const meta = screenMeta('agente');

const TABS = [
  { id: 'conversa', key: 'agente-aba-conversa' },
  { id: 'aprovacao', key: 'agente-aba-aprovacao' },
  { id: 'alertas', key: 'agente-aba-alertas' },
  { id: 'ferramentas', key: 'agente-aba-ferramentas' },
  { id: 'memoria', key: 'agente-aba-memoria' },
  { id: 'configurar', key: 'agente-aba-configurar' },
] as const;

type TabId = (typeof TABS)[number]['id'];

const BASE = meta.path;
const LINKS: ChatLinks = {
  settings: `${BASE}?aba=configurar`,
  proposal: (id) => `${BASE}?aba=aprovacao&proposta=${encodeURIComponent(id)}`,
};

function Count({ n, label }: { n: number; label: string }) {
  if (!n) return null;
  return (
    <span className="num rounded-full bg-accent-soft px-1.5 text-[10px] text-accent">
      <span aria-hidden>{n}</span>
      <span className="sr-only">{label}</span>
    </span>
  );
}

export default function Agente() {
  const [params, setParams] = useSearchParams();
  const raw = params.get('aba');
  const tab = (TABS.some((x) => x.id === raw) ? raw : 'conversa') as TabId;
  const pending = usePendingCount();
  const alerts = useAlertCounts();
  const engines = useEngines();

  const go = useCallback(
    (id: TabId) =>
      setParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          next.set('aba', id);
          if (id !== 'aprovacao') next.delete('proposta');
          return next;
        },
        { replace: true },
      ),
    [setParams],
  );

  // As abas entram na barra ⌘K pelos comandos globais (commands/useGlobalCommands.ts).

  const nPending = pending.data ?? 0;
  const nAlerts = alerts.data?.new ?? 0;
  const counts: Partial<Record<TabId, { n: number; label: string }>> = {
    aprovacao: { n: nPending, label: t('agente-n-pendentes', { n: nPending }) },
    alertas: { n: nAlerts, label: t('agente-n-novos', { n: nAlerts }) },
  };
  const ready = engines.data?.filter((e) => e.enabled).length;
  usePageContext({
    screen: 'agente',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    figures: {
      pending_proposals: pending.data ?? null,
      new_alerts: alerts.data?.new ?? null,
    },
    selection: params.get('proposta') ? { proposal_id: params.get('proposta') } : undefined,
  });

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        meta={ready === undefined ? undefined : t('agente-motores-ligados', { n: ready })}
      />
      <Tabs value={tab} onValueChange={(v) => go(v as TabId)}>
        <TabsList aria-label={t('agente-secoes')}>
          {TABS.map((x) => (
            <TabsTrigger key={x.id} value={x.id}>
              {t(x.key)}
              {counts[x.id] ? <Count {...counts[x.id]!} /> : null}
            </TabsTrigger>
          ))}
        </TabsList>
        <TabsContent value="conversa">
          <ChatPanel links={LINKS} />
        </TabsContent>
        <TabsContent value="aprovacao">
          <ApprovalQueue focusId={params.get('proposta')} settingsHref={LINKS.settings} />
        </TabsContent>
        <TabsContent value="alertas">
          <AlertsPanel />
        </TabsContent>
        <TabsContent value="ferramentas">
          <ToolHistory />
        </TabsContent>
        <TabsContent value="memoria">
          <div className="flex flex-col gap-6">
            <MemoryPanel />
            <ProceduresPanel />
          </div>
        </TabsContent>
        <TabsContent value="configurar">
          <AiSettingsSection heading={null} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
