/**
 * Ajustes › IA, em quatro abas (a escolhida fica no parâmetro `ia` da URL):
 *
 * - **Motores**: os motores de IA e o roteamento por tarefa com fallback;
 * - **Acesso do agente**: o que o agente pode fazer e ver (permissões, rotinas, MCPs extras,
 *   quais MCPs das CLIs da pessoa entram nas conversas e, em Ajustes, a memória);
 * - **Custos**: uso e custo, limites de gasto e a tabela de preços;
 * - **Histórico**: conversas e anexos guardados no cofre.
 *
 * Seção autocontida (só usa os comandos `ai.*`), para montar em Ajustes e na aba
 * Configurar de Agente (que já tem a própria aba de memória).
 *
 * ```tsx
 * import { AiSettingsSection } from '@/features/agent/settings';
 * <AiSettingsSection />
 * ```
 */
import { useSearchParams } from 'react-router';

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import { t } from '@/i18n';

import { MemoryPanel } from '../MemoryPanel';
import { AgentOptionsPanel } from './AgentOptionsPanel';
import { CliMcpsPanel } from './CliMcpsPanel';
import { HistoryPanel } from './HistoryPanel';
import { EnginesPanel } from './EnginesPanel';
import { LimitsPanel } from './LimitsPanel';
import { McpExtrasPanel } from './McpExtrasPanel';
import { PricesPanel } from './PricesPanel';
import { RoutesPanel } from './RoutesPanel';
import { UsagePanel } from './UsagePanel';

const TABS = ['motores', 'acesso', 'custos', 'historico'] as const;
type AiTab = (typeof TABS)[number];

const isTab = (v: string | null): v is AiTab => TABS.includes(v as AiTab);

export interface AiSettingsSectionProps {
  /** Título acima das abas (omita quando a tela já tem o seu). */
  heading?: string | null;
  /** Mostra a memória do agente na aba Acesso (a tela do Agente tem aba própria). */
  withMemory?: boolean;
  className?: string;
}

export function AiSettingsSection({
  heading = t('agente-ajustes-titulo'),
  withMemory = false,
  className,
}: AiSettingsSectionProps) {
  const [params, setParams] = useSearchParams();
  const raw = params.get('ia');
  const tab: AiTab = isTab(raw) ? raw : 'motores';
  const go = (next: AiTab) =>
    setParams(
      (p) => {
        const out = new URLSearchParams(p);
        if (next === 'motores') out.delete('ia');
        else out.set('ia', next);
        return out;
      },
      { replace: true },
    );

  const intro = (text: string) => <p className="max-w-[80ch] text-[13px] text-dim">{text}</p>;

  return (
    <section aria-label={t('agente-ajustes-ia')} className={className ?? 'flex flex-col gap-4'}>
      {heading ? <h2 className="label-caps text-fg">{heading}</h2> : null}
      <Tabs value={tab} onValueChange={(v) => go(v as AiTab)}>
        <TabsList aria-label={t('agente-ajustes-abas')}>
          <TabsTrigger value="motores">{t('agente-ajustes-aba-motores')}</TabsTrigger>
          <TabsTrigger value="acesso">{t('agente-ajustes-aba-acesso')}</TabsTrigger>
          <TabsTrigger value="custos">{t('agente-ajustes-aba-custos')}</TabsTrigger>
          <TabsTrigger value="historico">{t('agente-ajustes-aba-historico')}</TabsTrigger>
        </TabsList>
        <TabsContent value="motores" className="flex flex-col gap-4 pt-4">
          {intro(t('agente-ajustes-aba-motores-dica'))}
          <EnginesPanel index="01" />
          <RoutesPanel index="02" />
        </TabsContent>
        <TabsContent value="acesso" className="flex flex-col gap-4 pt-4">
          {intro(t('agente-ajustes-aba-acesso-dica'))}
          <AgentOptionsPanel index="01" />
          <McpExtrasPanel index="02" />
          <CliMcpsPanel index="03" />
          {withMemory ? <MemoryPanel index="04" /> : null}
        </TabsContent>
        <TabsContent value="custos" className="flex flex-col gap-4 pt-4">
          {intro(t('agente-ajustes-aba-custos-dica'))}
          <UsagePanel index="01" />
          <div className="grid items-start gap-4 @min-[1100px]:grid-cols-2">
            <LimitsPanel index="02" />
            <PricesPanel index="03" />
          </div>
        </TabsContent>
        <TabsContent value="historico" className="flex flex-col gap-4 pt-4">
          {intro(t('agente-ajustes-aba-historico-dica'))}
          <HistoryPanel index="01" />
        </TabsContent>
      </Tabs>
    </section>
  );
}
