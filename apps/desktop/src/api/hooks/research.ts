/**
 * Hooks da análise de ativos (`research.*`). Chaves `['research', acao, args]`.
 *
 * Unidades do núcleo: razões em fração (0,06 = 6%), múltiplos em vezes, dinheiro dos
 * indicadores em unidades da moeda; o histórico (`research.history`) em centavos.
 * Os comandos de ativo aceitam `asset_id` (cadastrado) ou `ticker` (papel do universo).
 */
import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from '@tanstack/react-query';

import { call, type ApiError } from '@/api/client';
import type { MethodExplain } from '@/api/generated/MethodExplain';
import type { MethodRun } from '@/api/generated/MethodRun';
import type { ScreenMethod } from '@/api/generated/ScreenMethod';
import type { AssetTag } from '@/api/generated/AssetTag';
import type { MarketMapPoint } from '@/api/generated/MarketMapPoint';
import type { MarketOverview } from '@/api/generated/MarketOverview';
import type { DocumentsStatus } from '@/api/generated/DocumentsStatus';
import type { CompareAssetsArgs } from '@/api/generated/CompareAssetsArgs';
import type { CompareView } from '@/api/generated/CompareView';
import type { CorporateDocument } from '@/api/generated/CorporateDocument';
import type { DocumentsArgs } from '@/api/generated/DocumentsArgs';
import type { NewsView } from '@/api/generated/NewsView';
import type { FairPriceArgs } from '@/api/generated/FairPriceArgs';
import type { FairPriceView } from '@/api/generated/FairPriceView';
import type { HistoryArgs } from '@/api/generated/HistoryArgs';
import type { HistoryView } from '@/api/generated/HistoryView';
import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { IndicatorsView } from '@/api/generated/IndicatorsView';
import type { RankArgs } from '@/api/generated/RankArgs';
import type { RankResult } from '@/api/generated/RankResult';
import type { ResearchSettings } from '@/api/generated/ResearchSettings';
import type { ResearchSettingsView } from '@/api/generated/ResearchSettingsView';
import type { ResearchStatus } from '@/api/generated/ResearchStatus';
import type { ResearchSyncReport } from '@/api/generated/ResearchSyncReport';
import type { Screen } from '@/api/generated/Screen';
import type { ScreenArgs } from '@/api/generated/ScreenArgs';
import type { ScreenInput } from '@/api/generated/ScreenInput';
import type { ScreenPatch } from '@/api/generated/ScreenPatch';
import type { ScreenResult } from '@/api/generated/ScreenResult';
import type { SubjectArgs } from '@/api/generated/SubjectArgs';
import type { SyncArgs } from '@/api/generated/SyncArgs';

import { cleanArgs } from './portfolio';

export const researchKeys = {
  all: ['research'] as const,
  of: (action: string, args?: object) => ['research', action, cleanArgs(args)] as const,
};

function read<T>(action: string, args?: object, enabled = true) {
  const a = cleanArgs(args);
  return {
    queryKey: researchKeys.of(action, a),
    queryFn: () => call<T>(`research.${action}`, a),
    enabled,
  };
}

const hasSubject = (s: SubjectArgs | null | undefined) => Boolean(s?.asset_id || s?.ticker);

// ── Leituras ─────────────────────────────────────────────────────

/** Catálogo de indicadores (fórmula, unidade, ordem natural). Não muda durante a sessão. */
export function useCatalog() {
  return useQuery({ ...read<IndicatorInfo[]>('catalog'), staleTime: Infinity });
}

export function useResearchStatus() {
  return useQuery(read<ResearchStatus>('status'));
}

export function useResearchSettings() {
  return useQuery(read<ResearchSettingsView>('get_settings'));
}

export function useIndicators(subject: SubjectArgs | null) {
  return useQuery(read<IndicatorsView>('indicators', subject ?? {}, hasSubject(subject)));
}

export function useFairPrice(args: FairPriceArgs | null) {
  return useQuery({
    ...read<FairPriceView>('fair_price', args ?? {}, hasSubject(args)),
    placeholderData: keepPreviousData,
  });
}

export function useHistory(args: HistoryArgs | null) {
  return useQuery(read<HistoryView>('history', args ?? {}, hasSubject(args)));
}

/** Notícias do ativo (docs/asset-overview.md): portais por RSS e, se ligado, o Google. */
export function useAssetNews(subject: SubjectArgs | null) {
  return useQuery({
    ...read<NewsView>('news', subject ?? {}, hasSubject(subject)),
    staleTime: 10 * 60 * 1000,
  });
}

export function useDocuments(args: DocumentsArgs | null) {
  return useQuery(read<CorporateDocument[]>('documents', args ?? {}, args != null));
}

/** Por que o ativo tem (ou não) documentos: fonte, emissor, coleta e categorias guardadas. */
export function useDocumentsStatus(args: DocumentsArgs | null) {
  return useQuery(read<DocumentsStatus>('documents_status', args ?? {}, args != null));
}

export function useRank(args: RankArgs | null) {
  return useQuery({
    ...read<RankResult>('rank', args ?? {}, Boolean(args?.indicator)),
    placeholderData: keepPreviousData,
  });
}

/** Screener: só roda quando `args` não é `null` (a pessoa clicou em rodar). */
export function useScreen(args: ScreenArgs | null) {
  return useQuery({
    ...read<ScreenResult>('screen', args ?? {}, args != null),
    placeholderData: keepPreviousData,
  });
}

export function useScreens() {
  return useQuery(read<Screen[]>('screens.list'));
}

export function useCompare(args: CompareAssetsArgs | null) {
  const n = (args?.asset_ids?.length ?? 0) + (args?.tickers?.length ?? 0);
  return useQuery({
    ...read<CompareView>('compare', args ?? {}, n >= 2),
    placeholderData: keepPreviousData,
  });
}

// ── Escritas ─────────────────────────────────────────────────────

/** Telas salvas passam pelo histórico de alterações (desfazer em Ajustes › Histórico). */
function invalidateResearch(client: QueryClient, extra: readonly string[] = []) {
  const modules = new Set(['research', 'changes', ...extra]);
  return client.invalidateQueries({ predicate: (q) => modules.has(String(q.queryKey[0])) });
}

function useResearchWrite<A, R>(cmd: string, extra: readonly string[] = []) {
  const client = useQueryClient();
  return useMutation<R, ApiError, A>({
    mutationFn: (args: A) => call<R>(cmd, args),
    onSuccess: () => invalidateResearch(client, extra),
  });
}

export const useCreateScreen = () =>
  useResearchWrite<ScreenInput, Screen>('research.screens.create');
export const useUpdateScreen = () =>
  useResearchWrite<ScreenPatch, Screen>('research.screens.update');
export const useDeleteScreen = () =>
  useResearchWrite<{ id: string }, null>('research.screens.delete');

/** Tela Mercado: panorama por setor e o mapa Saúde × Atratividade. */
export function useMarketOverview() {
  return useQuery(read<MarketOverview>('market_overview'));
}

export function useMarketMap() {
  return useQuery(read<MarketMapPoint[]>('market_map'));
}

/** Métodos do Mercado: receitas, quantos passam, a lista e a explicação por papel. */
export function useMethods() {
  return useQuery(read<ScreenMethod[]>('methods'));
}

/** Características como tags, com a contagem de hoje (catálogo em methodology/tags). */
export function useTags() {
  return useQuery(read<AssetTag[]>('tags'));
}

export function useMethodCounts() {
  return useQuery(read<Record<string, number>>('method_counts'));
}

export function useMethodRun(key: string | null) {
  return useQuery(read<MethodRun>('method_run', { key: key ?? '' }, key != null));
}

export function useMethodExplain(key: string, ticker: string | null) {
  return useQuery(
    read<MethodExplain>('method_explain', { key, ticker: ticker ?? '' }, Boolean(ticker)),
  );
}

/** Selos: ticker → métodos que ele cumpre hoje. */
export function useMethodBadges() {
  return useQuery(read<Record<string, string[]>>('method_badges'));
}

/** O ativo foi criado pelo Observar (some do cadastro ao deixar de observar sem operação). */
export function useIsObserved(assetId: string | null) {
  return useQuery(read<boolean>('observed', { id: assetId ?? '' }, assetId != null));
}

/** Observar um papel do universo: cria o ativo, se faltar, e põe na watchlist. */
export const useObserve = () =>
  useResearchWrite<{ ticker: string }, { asset_id: string }>('research.observe', [
    'assets',
    'portfolio',
    'score',
  ]);

export const useUnobserve = () =>
  useResearchWrite<{ asset_id: string }, { removed: boolean }>('research.unobserve', [
    'assets',
    'portfolio',
    'score',
  ]);

export const useSetResearchSettings = () =>
  useResearchWrite<Partial<ResearchSettings>, ResearchSettingsView>('research.set_settings');

/**
 * Coleta da análise de ativos (CVM, B3, SEC). Na primeira vez baixa ~230 MB e leva 1–2
 * minutos; depois só o que mudou. O radar de dividendos grava projeções na agenda.
 */
export const useResearchSync = () =>
  useResearchWrite<SyncArgs, ResearchSyncReport>('research.sync', ['portfolio', 'assets', 'score']);
