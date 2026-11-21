/**
 * 08 · ATIVOS: lista/busca e cadastro de ativos (a página do ativo é `/ativo/<id>`,
 * docs/asset-page.md; `?ativo=<id>` e `?aba=indicadores&ticker=` seguem para lá),
 * watchlist, agenda de dividendos (com proventos a confirmar), índices e cotações, e a
 * análise de ativos: indicadores por ticker, rankings, screener, comparador e os dados da
 * coleta (CVM, B3, SEC).
 *
 * Estado na URL: `?aba=` (lista | watchlist | agenda | indices | indicadores |
 * comparador | dados | bolsas), `?ativo=<id>`, `?novo=ativo`; `?aba=rankings` e
 * `?aba=screener` seguem para 10 · Mercado,
 * `?ticker=` (indicadores) e `?comparar=PETR4,VALE3` (comparador).
 */
import { useCallback } from 'react';
import { Navigate, useNavigate, useSearchParams } from 'react-router';

import { screenMeta } from '@/app/screens';
import type { Asset } from '@/api/generated/Asset';
import {
  Button,
  Icon,
  ScreenHeader,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/design/components';
import { AssetFormDialog } from '@/features/assets/AssetFormDialog';
import { AssetList } from '@/features/assets/AssetList';
import { DividendAgenda } from '@/features/assets/DividendAgenda';
import { WatchlistPanel } from '@/features/assets/Watchlist';
import { IndicesPanel } from '@/features/indices/IndicesPanel';
import { ExchangesPanel } from '@/features/market/ExchangesPanel';
import { Comparador, MAX_COMPARE } from '@/features/research/Comparador';
import { ResearchDataPanel } from '@/features/research/DataPanel';
import { subjectHref } from '@/features/research/common';
import { TickerAnalysis } from '@/features/research/ResearchSection';
import { useCountryFeatures } from '@/api/hooks/country';
import { usePageContext } from '@/features/companion/pageContext';
import { t } from '@/i18n';

const meta = screenMeta('ativos');

// `br`: o universo vem da B3 e da CVM; fora do Brasil a aba some (docs/international.md).
const TABS = [
  { id: 'lista', label: 'casca-tela-ativos', br: false },
  { id: 'watchlist', label: 'ativos-aba-watchlist', br: false },
  { id: 'agenda', label: 'ativos-aba-agenda', br: false },
  { id: 'indices', label: 'ativos-aba-indices', br: false },
  { id: 'indicadores', label: 'ativos-aba-indicadores', br: false },
  { id: 'comparador', label: 'ativos-aba-comparador', br: false },
  { id: 'dados', label: 'ativos-aba-dados', br: true },
  { id: 'bolsas', label: 'ativos-aba-bolsas', br: false },
] as const;

function parseTickers(raw: string | null): string[] {
  return [
    ...new Set(
      (raw ?? '')
        .split(',')
        .map((s) => s.trim().toUpperCase())
        .filter(Boolean),
    ),
  ].slice(0, MAX_COMPARE);
}

/** Rankings e screener foram para 10 · Mercado: os links antigos seguem para lá. */
const MOVED: Record<string, string> = {
  rankings: '/mercado?aba=rankings',
  screener: '/mercado?aba=explorador',
};

export default function Ativos() {
  const [params, setParams] = useSearchParams();
  const navigate = useNavigate();
  const moved = MOVED[params.get('aba') ?? ''];
  const assetId = params.get('ativo');
  const creating = params.get('novo') === 'ativo';
  const features = useCountryFeatures();
  const tabs = TABS.filter((tb) => !tb.br || features.br_market);
  const tab = tabs.some((tb) => tb.id === params.get('aba')) ? params.get('aba')! : 'lista';

  const update = useCallback(
    (patch: Record<string, string | null>) =>
      setParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          for (const [k, v] of Object.entries(patch)) {
            if (v == null) next.delete(k);
            else next.set(k, v);
          }
          return next;
        },
        { replace: false },
      ),
    [setParams],
  );
  const openAsset = (a: Asset) => void navigate(subjectHref(a.id, a.ticker));

  // Contexto para o agente: a aba aberta (a página do ativo tem o seu).
  usePageContext({
    screen: 'ativos',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    filters: { compare: params.get('comparar') ?? undefined },
  });

  if (moved) return <Navigate to={moved} replace />;
  // A página do ativo é a rota própria; os links antigos seguem com os mesmos parâmetros.
  if (assetId) {
    const rest = new URLSearchParams(params);
    rest.delete('ativo');
    const q = rest.toString();
    return <Navigate to={`/ativo/${encodeURIComponent(assetId)}${q ? `?${q}` : ''}`} replace />;
  }
  if (tab === 'indicadores' && params.get('ticker'))
    return <Navigate to={subjectHref(null, params.get('ticker'))} replace />;
  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        actions={
          <Button
            variant="primary"
            icon={<Icon name="plus" size={12} />}
            onClick={() => update({ novo: 'ativo' })}
          >
            {t('ativos-cadastrar')}
          </Button>
        }
      />
      <Tabs value={tab} onValueChange={(v) => update({ aba: v === 'lista' ? null : v })}>
        <TabsList aria-label={t('ativos-secoes')}>
          {tabs.map((tb) => (
            <TabsTrigger key={tb.id} value={tb.id}>
              {t(tb.label)}
            </TabsTrigger>
          ))}
        </TabsList>
        <TabsContent value="lista">
          <AssetList onOpen={openAsset} />
        </TabsContent>
        <TabsContent value="watchlist">
          <WatchlistPanel />
        </TabsContent>
        <TabsContent value="agenda">
          <DividendAgenda />
        </TabsContent>
        <TabsContent value="indices">
          <IndicesPanel />
        </TabsContent>
        <TabsContent value="indicadores">
          <TickerAnalysis
            ticker={null}
            onTicker={(tk) => tk && void navigate(subjectHref(null, tk))}
          />
        </TabsContent>
        <TabsContent value="comparador">
          <Comparador
            tickers={parseTickers(params.get('comparar'))}
            onChange={(tk) => update({ comparar: tk.length ? tk.join(',') : null })}
          />
        </TabsContent>
        {features.br_market ? (
          <TabsContent value="dados">
            <ResearchDataPanel />
          </TabsContent>
        ) : null}
        <TabsContent value="bolsas">
          <ExchangesPanel />
        </TabsContent>
      </Tabs>
      <AssetFormDialog
        open={creating}
        onOpenChange={(o) => !o && update({ novo: null })}
        onSaved={(a) =>
          // Previdência abre direto na aba do plano (tipo, regime, aportes e resgate).
          update({
            ativo: a.id,
            novo: null,
            ...(a.class === 'pension' ? { secao: 'previdencia' } : {}),
          })
        }
      />
    </div>
  );
}
