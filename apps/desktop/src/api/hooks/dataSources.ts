import type { BrapiUsageView } from '@/api/generated/BrapiUsageView';
/**
 * Hooks das fontes de dados (`market.data_sources`, `market.set_fred_key`,
 * `market.sync`). A chave da FRED só entra: o núcleo nunca a devolve.
 */
import type { DataSources } from '../generated/DataSources';
import type { DataSyncReport } from '../generated/DataSyncReport';
import type { FundamentusArgs } from '../generated/FundamentusArgs';
import type { FundamentusIncomeArgs } from '../generated/FundamentusIncomeArgs';
import type { FundamentusIncomeReport } from '../generated/FundamentusIncomeReport';
import type { FundamentusSnapshot } from '../generated/FundamentusSnapshot';
import type { SourceInfo } from '../generated/SourceInfo';
import type { SourceToggleArgs } from '../generated/SourceToggleArgs';
import { useCore, useCoreMutation } from './coreQuery';

export const useDataSources = () => useCore<DataSources>('market', 'data_sources');

export const useSetFredKey = () =>
  useCoreMutation<{ key: string }, null>('market.set_fred_key', {
    affects: ['market'],
    sensitive: true,
  });

/** Token opcional da brapi.dev (segunda fonte para conferir os dados oficiais); vazio remove. */
export const useSetBrapiToken = () =>
  useCoreMutation<{ key: string }, null>('market.set_brapi_token', {
    affects: ['market'],
    sensitive: true,
  });

/** Chave gratuita da Tiingo (cotações dos Estados Unidos); vazio remove. */
export const useSetTiingoToken = () =>
  useCoreMutation<{ key: string }, null>('market.set_tiingo_token', {
    affects: ['market'],
    sensitive: true,
  });

/** Chave da EODHD (cotações de bolsas do mundo todo); vazio remove. */
export const useSetEodhdToken = () =>
  useCoreMutation<{ key: string }, null>('market.set_eodhd_token', {
    affects: ['market'],
    sensitive: true,
  });

/** Lê o plano e o consumo da brapi.dev com o token (rede). */
export const useBrapiTest = () =>
  useCoreMutation<Record<string, never>, BrapiUsageView>('market.brapi_test', {
    affects: ['market'],
  });

/** Porcentagem da cota da brapi que o app se permite usar. */
export const useSetBrapiLimit = () =>
  useCoreMutation<{ pct: number }, BrapiUsageView>('market.set_brapi_limit', {
    affects: ['market'],
  });

/** Coleta índices, câmbio e cotações (pode levar alguns segundos: usa a rede). */
export const useMarketSync = () =>
  useCoreMutation<Record<string, never>, DataSyncReport>('market.sync', { affects: ['market'] });

/** Fontes externas com liga/desliga (`sources.*`). */
export const useSources = () => useCore<SourceInfo[]>('sources', 'list');

export const useSetSource = () =>
  useCoreMutation<SourceToggleArgs, SourceInfo[]>('sources.set', { affects: ['sources'] });

/** Indicadores e proventos do Fundamentus (cache local de um dia). */
export const useFundamentus = (ticker: string | null | undefined) =>
  useCore<FundamentusSnapshot>(
    'research',
    'fundamentus',
    { ticker: ticker ?? '' },
    { enabled: Boolean(ticker), staleTime: 30 * 60_000, retry: false },
  );

export const useRefreshFundamentus = () =>
  useCoreMutation<FundamentusArgs, FundamentusSnapshot>('research.fundamentus', {
    affects: ['sources'],
  });

/** Proventos anunciados de ações e FIIs pelo Fundamentus (viram "a confirmar" na carteira). */
export const useSyncFundamentusIncome = () =>
  useCoreMutation<FundamentusIncomeArgs, FundamentusIncomeReport>(
    'portfolio.sync_fundamentus_income',
    { affects: ['market'] },
  );
