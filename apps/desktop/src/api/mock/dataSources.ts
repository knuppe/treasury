/**
 * Mock das fontes de dados (`market.data_sources`, `market.set_fred_key`,
 * `market.sync`): chave da FRED (nunca devolvida) e a coleta de índices, câmbio
 * e cotações com o relatório por série/ativo.
 */
import type { DataSources } from '@/api/generated/DataSources';
import type { DataSyncReport } from '@/api/generated/DataSyncReport';

import type { BrapiUsageView } from '../generated/BrapiUsageView';
import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { ready } from './ledger/boot';
import { invalid, str } from './ledger/errors';

let fredKey: string | null = null;
let brapiToken: string | null = null;
let brapiPlan: string | null = null;
let brapiUsed = 0;
let brapiLimit = 80;

/** Token e plano da brapi no mock (o `market.data_sources` de `indices.ts` também usa). */
export function brapiMockSources(): {
  brapi_token_set: boolean;
  brapi: BrapiUsageView | null;
  tiingo_token_set: boolean;
  eodhd_token_set: boolean;
} {
  return {
    brapi_token_set: Boolean(brapiToken),
    brapi: brapiToken ? brapiView() : null,
    tiingo_token_set: Boolean(sourceKeys.tiingo),
    eodhd_token_set: Boolean(sourceKeys.eodhd),
  };
}

/** Espelho de `market::api::brapi_view` (plano lido pelo "Testar token"). */
function brapiView(): BrapiUsageView {
  return {
    plan: brapiPlan,
    quota: 15000,
    used: brapiUsed,
    limit_pct: brapiLimit,
    paid: brapiPlan === 'startup' || brapiPlan === 'pro',
    checked_at: brapiPlan ? new Date().toISOString() : null,
    window_end: null,
  };
}
let lastSync: string | null = null;

onMockReset(() => {
  fredKey = null;
  brapiToken = null;
  brapiPlan = null;
  brapiUsed = 0;
  brapiLimit = 80;
  lastSync = null;
});

const NO_FRED_KEY =
  'para baixar CPI, M2 e Fed Funds dos EUA, cadastre a chave gratuita da FRED em Ajustes › Fontes de dados';

function report(): DataSyncReport {
  const started = new Date();
  const finished = new Date(started.getTime() + 2_400);
  const today = started.toISOString().slice(0, 10);
  const lastMonth = new Date(started.getFullYear(), started.getMonth() - 1, 1)
    .toISOString()
    .slice(0, 10);
  const series = (
    series_id: string,
    written: number,
    last_date: string | null,
    error: string | null = null,
  ) => ({ series_id, written, last_date, error });
  const fred = fredKey
    ? [
        series('fred:CPIAUCSL', 1, lastMonth),
        series('fred:M2SL', 1, lastMonth),
        series('fred:FEDFUNDS', 1, lastMonth),
      ]
    : [
        series('fred:CPIAUCSL', 0, null, NO_FRED_KEY),
        series('fred:M2SL', 0, null, NO_FRED_KEY),
        series('fred:FEDFUNDS', 0, null, NO_FRED_KEY),
      ];
  const indicesOk = Boolean(fredKey);
  return {
    indices: {
      started_at: started.toISOString(),
      finished_at: finished.toISOString(),
      series: [
        series('bcb:433', 1, lastMonth),
        series('bcb:12', 21, today),
        series('bcb:4389', 21, today),
        series('bcb:11', 21, today),
        ...fred,
      ],
      fx: [series('bcb_ptax_venda:USD', 21, today), series('bcb_ptax_venda:EUR', 21, today)],
      ipca_estimated_months: 1,
      ok: indicesOk,
    },
    prices: {
      started_at: started.toISOString(),
      finished_at: finished.toISOString(),
      assets: [
        {
          asset_id: '01mockas-0001',
          ticker: 'IVVB11',
          source: 'yahoo',
          prices: 21,
          dividends: 0,
          splits_added: 0,
          last_date: today,
          error: null,
        },
        {
          asset_id: '01mockas-0002',
          ticker: 'HGLG11',
          source: 'yahoo',
          prices: 21,
          dividends: 1,
          splits_added: 0,
          last_date: today,
          error: null,
        },
        {
          asset_id: '01mockas-0003',
          ticker: 'XPTO3',
          source: 'yahoo',
          prices: 0,
          dividends: 0,
          splits_added: 0,
          last_date: null,
          error: 'ativo sem cotação no Yahoo Finance (XPTO3.SA)',
        },
      ],
      without_source: [
        {
          asset_id: '01mockas-0004',
          ticker: null,
          name: 'CDB Banco Exemplo 120% CDI',
          class: 'fixed_income',
          reason:
            'renda fixa é marcada pela curva do contrato; informe o valor manualmente se quiser',
        },
      ],
      ok: false,
    },
    ok: false,
  };
}

/** Chaves da Tiingo e da EODHD (só se existem; o valor nunca volta). */
const sourceKeys: { tiingo: string | null; eodhd: string | null } = { tiingo: null, eodhd: null };

function setSourceKey(source: 'tiingo' | 'eodhd', args: Record<string, unknown>) {
  ready();
  const k = str(args, 'key').trim();
  if (!k) sourceKeys[source] = null;
  else if (!/^[A-Za-z0-9._-]{10,200}$/.test(k))
    invalid('chave inválida: confira o valor copiado do site da fonte');
  else sourceKeys[source] = k;
  return null;
}

registerMockHandlers({
  'market.data_sources': (): DataSources => {
    ready();
    return {
      fred_key_set: Boolean(fredKey),
      brapi_token_set: Boolean(brapiToken),
      brapi: brapiToken ? brapiView() : null,
      tiingo_token_set: Boolean(sourceKeys.tiingo),
      eodhd_token_set: Boolean(sourceKeys.eodhd),
      last_sync_at: lastSync,
    };
  },
  'market.set_fred_key': (args) => {
    ready();
    const k = str(args, 'key').trim();
    if (!k) fredKey = null;
    else if (!/^[A-Za-z0-9]{32}$/.test(k)) invalid('a chave da FRED tem 32 letras e números');
    else fredKey = k;
    return null;
  },
  'market.brapi_test': () => {
    ready();
    if (!brapiToken) invalid('brapi.dev sem token: cadastre um em Ajustes › Fontes de dados.');
    brapiPlan = 'free';
    brapiUsed = 1248;
    return brapiView();
  },
  'market.set_brapi_limit': (args) => {
    ready();
    const pct = Number(args.pct);
    if (!(pct >= 10 && pct <= 100)) invalid('o limite de uso da brapi vai de 10% a 100% da cota');
    brapiLimit = pct;
    return brapiView();
  },
  'market.set_tiingo_token': (args) => setSourceKey('tiingo', args),
  'market.set_eodhd_token': (args) => setSourceKey('eodhd', args),
  'market.set_brapi_token': (args) => {
    ready();
    const k = str(args, 'key').trim();
    if (!k) brapiToken = null;
    else if (!/^[A-Za-z0-9._-]{10,200}$/.test(k))
      invalid('o token da brapi.dev tem letras, números, hífen ou sublinhado');
    else brapiToken = k;
    brapiPlan = null;
    brapiUsed = 0;
    return null;
  },
  'market.sync': async (): Promise<DataSyncReport> => {
    ready();
    await new Promise((r) => setTimeout(r, import.meta.env.MODE === 'test' ? 0 : 900));
    const r = report();
    lastSync = r.indices.finished_at;
    return r;
  },
});
