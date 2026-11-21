/**
 * Página dedicada do ativo, `/ativo/<ref>` (docs/asset-page.md): fora das abas de 09 · Ativos,
 * com a trilha da tela de onde a pessoa veio ("Mercado › PETR4" volta ao mesmo recorte; de
 * qualquer outro lugar, "Ativos › PETR4"). `ref` é o id do ativo cadastrado ou o ticker de um
 * papel do universo; o ticker de um ativo cadastrado vai para o id (uma página só por ativo).
 *
 * Cadastrado: tudo o que o app tem dele (`AssetDetail`), o agente ao lado e as conversas feitas
 * aqui. Fora do cadastro: a análise fundamentalista do universo e "Observar".
 */
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Link, Navigate, useLocation, useNavigate, useParams } from 'react-router';

import { findAssetByTicker, useAsset } from '@/api/hooks/assets';
import { Icon } from '@/design/components';
import { AskAgentButton } from '@/features/assets/AssetConversations';
import { AssetNews } from '@/features/assets/AssetNews';
import { AssetOverview } from '@/features/assets/AssetOverview';
import { AssetDetail } from '@/features/assets/AssetDetail';
import { usePageContext } from '@/features/companion/pageContext';
import { LoadingBlock } from '@/features/portfolio/components/States';
import { useUniverseList } from '@/features/research/common';
import { ObserveBar, ResearchSection } from '@/features/research/ResearchSection';
import { t } from '@/i18n';
import { useNavOrigin } from '@/state/navOrigin';

/** Para onde a trilha volta: a tela Mercado de onde veio, ou a lista de Ativos. */
function useCrumbTarget(): { label: string; to: string } {
  const origin = useNavOrigin((s) => s.origin);
  if (origin?.pathname.startsWith('/mercado'))
    return { label: t('casca-tela-mercado'), to: `${origin.pathname}${origin.search}` };
  return {
    label: t('casca-tela-ativos'),
    to: origin?.pathname === '/ativos' ? `/ativos${origin.search}` : '/ativos',
  };
}

function Breadcrumb({ current }: { current: string }) {
  const target = useCrumbTarget();
  return (
    <nav aria-label={t('ativo-trilha')} className="label-caps flex items-center gap-1.5">
      <Link to={target.to} className="hover:text-fg">
        {target.label}
      </Link>
      <Icon name="chevronRight" size={11} className="text-dim" />
      <span aria-current="page" className="num text-fg">
        {current}
      </span>
    </nav>
  );
}

export default function Ativo() {
  const { ref = '' } = useParams();
  const { search } = useLocation();
  const asset = useAsset(ref || null);
  // O ativo saiu do cadastro com a página aberta ("Deixar de observar"): volta ao papel.
  const [lastTicker, setLastTicker] = useState<string | null>(null);
  const ticker = asset.isError ? null : (asset.data?.ticker ?? null);
  if (ticker && ticker !== lastTicker) setLastTicker(ticker);
  // Não é um id: talvez o ticker de um ativo cadastrado.
  const byTicker = useQuery({
    queryKey: ['assets', 'find', ref.toUpperCase()],
    queryFn: () => findAssetByTicker(ref),
    enabled: asset.isError,
  });
  if (asset.isPending || (asset.isError && byTicker.isPending))
    return <LoadingBlock label={t('ativos-carregando-ativo')} />;
  // (Com erro, o React Query ainda guarda o dado antigo: o ativo pode ter saído do cadastro.)
  if (asset.data && !asset.isError)
    return <RegisteredAsset key={asset.data.id} assetId={asset.data.id} />;
  if (lastTicker && lastTicker !== ref.toUpperCase())
    return <Navigate to={`/ativo/${lastTicker}`} replace />;
  if (byTicker.data) return <Navigate to={`/ativo/${byTicker.data.id}${search}`} replace />;
  return <UniverseTicker key={ref} ticker={ref.toUpperCase()} />;
}

function RegisteredAsset({ assetId }: { assetId: string }) {
  const navigate = useNavigate();
  const target = useCrumbTarget();
  const a = useAsset(assetId).data;
  // Contexto para o agente: a página de um ativo (o núcleo liga a conversa a ele).
  usePageContext(
    a
      ? {
          screen: 'ativo',
          title: a.ticker ?? a.name,
          view: 'ativo',
          selection: {
            asset_id: a.id,
            ticker: a.ticker,
            name: a.name,
            class: a.class,
            market: a.market,
            currency: a.currency,
            price_provider: a.price_provider,
            provider_symbol: a.provider_symbol,
          },
        }
      : null,
  );
  return (
    <AssetDetail
      assetId={assetId}
      onBack={() => void navigate(target.to)}
      breadcrumb={<Breadcrumb current={a?.ticker ?? a?.name ?? '…'} />}
      notice={a?.ticker ? <ObserveBar ticker={a.ticker} assetId={a.id} onPage /> : null}
    />
  );
}

function UniverseTicker({ ticker }: { ticker: string }) {
  const universe = useUniverseList();
  const hit = universe.data?.items.find((i) => i.ticker === ticker);
  usePageContext({
    screen: 'ativo',
    title: ticker,
    view: 'universo',
    selection: { ticker, name: hit?.name },
  });
  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3 border-b border-line pb-5">
        <Breadcrumb current={ticker} />
        <div className="flex flex-wrap items-end justify-between gap-4">
          <h2 className="flex items-baseline gap-3 text-[22px] leading-tight font-semibold text-fg">
            <span className="num tracking-[0.04em]">{ticker}</span>
            {hit ? (
              <span className="truncate text-[15px] font-normal text-dim">{hit.name}</span>
            ) : null}
          </h2>
          <AskAgentButton asset={{ ticker, name: hit?.name ?? ticker }} />
        </div>
      </div>
      <ObserveBar ticker={ticker} assetId={hit?.asset_id ?? null} onPage />
      <AssetOverview
        subject={{ ticker }}
        onNews={() => document.getElementById('noticias-do-papel')?.scrollIntoView()}
      />
      <ResearchSection subject={{ ticker }} />
      <div id="noticias-do-papel" className="flex flex-col gap-3">
        <h3 className="label-caps text-fg">{t('ativo-aba-noticias')}</h3>
        <AssetNews subject={{ ticker }} />
      </div>
    </div>
  );
}
