/**
 * 10 · MERCADO: encontrar ativos no mercado todo (docs/design/market-explorer.md).
 * Panorama por setor, explorador (screener com o score do universo), mapa Saúde ×
 * Atratividade e rankings. Rankings e screener moraram em Ativos; os links antigos
 * redirecionam para cá. A página de um papel fora do cadastro (com Observar) e o comparador
 * continuam em Ativos, que valem também fora do Brasil.
 *
 * Estado na URL: `?aba=` (panorama | metodos | explorador | mapa | rankings), `?setor=` e
 * `?segmento=` (explorador).
 *
 * Nada aqui é recomendação: mostra quem cumpre critérios públicos (CVM Res. 20/2021).
 */
import { useCallback } from 'react';
import { useSearchParams } from 'react-router';

import { screenMeta } from '@/app/screens';
import { ScreenHeader, Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import { MarketMapPanel, MarketOverviewPanel, NotAdvice } from '@/features/market/MarketPanels';
import { MethodsPanel } from '@/features/market/MethodsPanel';
import { usePageContext } from '@/features/companion/pageContext';
import { Rankings } from '@/features/research/Rankings';
import { Screener } from '@/features/research/Screener';
import { t } from '@/i18n';

const meta = screenMeta('mercado');

export const MARKET_TABS = [
  { id: 'panorama', label: 'mercado-aba-panorama' },
  { id: 'metodos', label: 'mercado-aba-metodos' },
  { id: 'explorador', label: 'mercado-aba-explorador' },
  { id: 'mapa', label: 'mercado-aba-mapa' },
  { id: 'rankings', label: 'ativos-aba-rankings' },
] as const;

export default function Mercado() {
  const [params, setParams] = useSearchParams();
  const tab = MARKET_TABS.some((tb) => tb.id === params.get('aba'))
    ? params.get('aba')!
    : 'panorama';
  const update = useCallback(
    (patch: Record<string, string | null>) =>
      setParams((prev) => {
        const next = new URLSearchParams(prev);
        for (const [k, v] of Object.entries(patch)) {
          if (v == null) next.delete(k);
          else next.set(k, v);
        }
        return next;
      }),
    [setParams],
  );
  const sector = params.get('setor');
  const segment = params.get('segmento');
  usePageContext({
    screen: 'mercado',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    filters: { sector: sector ?? segment ?? undefined },
  });

  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader index={meta.index} title={meta.title} description={meta.summary} />
      <Tabs
        value={tab}
        onValueChange={(v) =>
          update({ aba: v === 'panorama' ? null : v, setor: null, segmento: null })
        }
      >
        <TabsList aria-label={t('mercado-secoes')}>
          {MARKET_TABS.map((tb) => (
            <TabsTrigger key={tb.id} value={tb.id}>
              {t(tb.label)}
            </TabsTrigger>
          ))}
        </TabsList>
        <TabsContent value="panorama">
          <MarketOverviewPanel
            onSector={(s, kind) =>
              update(
                kind === 'acoes'
                  ? { aba: 'explorador', setor: s, segmento: null }
                  : { aba: 'explorador', setor: null, segmento: s },
              )
            }
          />
        </TabsContent>
        <TabsContent value="metodos">
          <MethodsPanel />
        </TabsContent>
        <TabsContent value="explorador">
          <div className="flex flex-col gap-3">
            <NotAdvice />
            <Screener key={`${sector}|${segment}`} sector={sector} segment={segment} />
          </div>
        </TabsContent>
        <TabsContent value="mapa">
          <MarketMapPanel />
        </TabsContent>
        <TabsContent value="rankings">
          <div className="flex flex-col gap-3">
            <NotAdvice />
            <Rankings />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
