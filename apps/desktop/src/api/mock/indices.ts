/**
 * Mock de `indices.*` e da coleta `market.sync` / `market.data_sources` /
 * `market.set_fred_key`, com as séries simuladas de `marketSim.ts`. Registra também
 * `fx.list` (câmbio USD/BRL) se o mock de câmbio de outra tela não existir.
 */
import type { RateReferences } from '@/api/generated/RateReferences';
import type { DataProvider } from '../generated/DataProvider';
import type { DataSyncReport } from '../generated/DataSyncReport';
import type { FxRate } from '../generated/FxRate';
import type { IndexSeries } from '../generated/IndexSeries';
import type { IndexValuePoint } from '../generated/IndexValuePoint';
import type { PriceSyncResult } from '../generated/PriceSyncResult';
import type { SeriesSyncResult } from '../generated/SeriesSyncResult';
import { brapiMockSources } from './dataSources';
import {
  extraMockHandler,
  mockFail,
  onMockReset,
  registerMockHandlers,
  requireUnlocked,
} from '../mockRegistry';
import { allAssets, lastPriceDate, syncSource } from './assets';
import {
  addMonths,
  dec,
  deflatorFactor,
  mockToday,
  SERIES,
  seriesPoints,
  txt,
  usdBrlPoints,
} from './marketSim';
import { referenceProfile } from './catalog';
import { syncedAssetIds } from './portfolio';

const isTest = import.meta.env.MODE === 'test';

interface State {
  fredKey: boolean;
  lastSyncAt: string | null;
  fetchedAt: string;
}

function initial(): State {
  const at = `${addMonths(mockToday(), 0)}T06:00:00Z`;
  return { fredKey: true, lastSyncAt: at, fetchedAt: at };
}

let state = initial();
onMockReset(() => {
  state = initial();
});

function date(v: unknown): string {
  if (typeof v !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(v))
    mockFail('invalid', `data inválida: ${String(v)}`);
  return v;
}

function providerOf(source: string): DataProvider | null {
  const head = source.split(':')[0];
  if (head === 'bcb_sgs' || head === 'fred' || head === 'yahoo') return head;
  return null;
}

function listSeries(): IndexSeries[] {
  return SERIES.map((s) => {
    const pts = seriesPoints(s.id);
    const official = pts.filter((p) => !p.estimated);
    return {
      id: s.id,
      name: s.name,
      kind: s.kind,
      unit: s.unit,
      frequency: s.frequency,
      country: s.country,
      source: s.source,
      last_fetched_at: pts.length ? state.fetchedAt : null,
      last_official_date: official.length ? official[official.length - 1]!.date : null,
      last_date: pts.length ? pts[pts.length - 1]!.date : null,
      count: pts.length,
      provider: providerOf(s.source),
      in_use: true,
    };
  }).sort((a, b) => a.country.localeCompare(b.country) || a.id.localeCompare(b.id));
}

async function wait(ms: number): Promise<void> {
  if (isTest || ms <= 0) return;
  await new Promise((resolve) => setTimeout(resolve, ms));
}

registerMockHandlers({
  // Taxas de referência das calculadoras (como o núcleo: só o que foi coletado, com a data).
  'indices.references': (): RateReferences => {
    requireUnlocked();
    const selic = 0.15;
    return {
      policy_rate: {
        name: 'Selic meta',
        series_id: 'selic_meta',
        rate: selic,
        date: '2026-09-17',
        estimated: false,
      },
      cdi: { name: 'CDI', series_id: 'cdi', rate: 0.149, date: '2026-10-30', estimated: false },
      inflation_12m: {
        name: 'IPCA',
        series_id: 'ipca',
        rate: 0.0478,
        date: '2026-09-01',
        estimated: false,
      },
      savings: {
        name: 'poupanca',
        series_id: 'selic_meta',
        rate: 1.005 ** 12 - 1,
        date: '2026-09-17',
        estimated: false,
      },
    };
  },
  'indices.list_series': () => {
    requireUnlocked();
    return listSeries();
  },

  'indices.get_series': (a) => {
    requireUnlocked();
    const id = String(a.id);
    if (!SERIES.some((s) => s.id === id)) return [];
    const to = typeof a.to === 'string' ? date(a.to) : mockToday();
    const from = typeof a.from === 'string' ? date(a.from) : addMonths(to, -24);
    return seriesPoints(id)
      .filter((p) => p.date >= from && p.date <= to)
      .map((p): IndexValuePoint => ({
        date: p.date,
        value: dec(p.value, 6),
        estimated: p.estimated,
      }));
  },

  'indices.factor': (a) => {
    requireUnlocked();
    const from = date(a.from);
    const to = date(a.to);
    const deflator = txt(a.deflator) || 'ipca';
    const seriesId =
      deflator === 'country'
        ? (referenceProfile().inflation[0]?.id ?? null)
        : deflator === 'none'
          ? null
          : deflator === 'm2_us'
            ? 'us_m2'
            : deflator;
    const f = deflatorFactor(deflator, from, to);
    return {
      deflator,
      series_id: seriesId,
      from,
      to,
      factor: f ? dec(f.factor, 10) : null,
      inflation: f ? f.factor - 1 : null,
      estimated: f?.estimated ?? false,
    };
  },

  'market.data_sources': () => {
    requireUnlocked();
    return {
      fred_key_set: state.fredKey,
      ...brapiMockSources(),
      last_sync_at: state.lastSyncAt,
    };
  },

  'market.set_fred_key': (a) => {
    requireUnlocked();
    const k = txt(a.key).trim();
    if (k === '') state.fredKey = false;
    else if (k.length !== 32 || !/^[a-z0-9]+$/i.test(k))
      mockFail('invalid', 'a chave da FRED tem 32 letras e números');
    else state.fredKey = true;
    return null;
  },

  'market.sync': async (): Promise<DataSyncReport> => {
    requireUnlocked();
    const started = new Date().toISOString();
    await wait(1600);
    const series: SeriesSyncResult[] = SERIES.map((s) => {
      const pts = seriesPoints(s.id);
      const needsKey = s.source.startsWith('fred:') && !state.fredKey;
      return {
        series_id: s.id,
        written: needsKey ? 0 : s.frequency === 'daily' ? 1 : 0,
        last_date: needsKey ? null : (pts.filter((p) => !p.estimated).at(-1)?.date ?? null),
        error: needsKey
          ? 'FRED exige uma chave gratuita: cadastre em fred.stlouisfed.org e informe em Fontes de dados'
          : null,
      };
    });
    const fx: SeriesSyncResult[] = [
      { series_id: 'USD', written: 1, last_date: usdBrlPoints().at(-1)?.[0] ?? null, error: null },
    ];
    const ids = syncedAssetIds();
    const assets: PriceSyncResult[] = allAssets()
      .filter((a) => ids.includes(a.id))
      .flatMap((a) => {
        const source = syncSource(a);
        if (!source) return [];
        return [
          {
            asset_id: a.id,
            ticker: a.ticker,
            source,
            prices: 1,
            dividends: 0,
            splits_added: 0,
            last_date: lastPriceDate(a.id),
            error: null,
          },
        ];
      });
    const finished = new Date().toISOString();
    state.lastSyncAt = finished;
    state.fetchedAt = finished;
    const indicesOk = series.every((s) => !s.error);
    return {
      indices: {
        started_at: started,
        finished_at: finished,
        series,
        fx,
        ipca_estimated_months: seriesPoints('ipca').filter((p) => p.estimated).length,
        ok: indicesOk,
      },
      prices: { started_at: started, finished_at: finished, assets, without_source: [], ok: true },
      ok: indicesOk,
    };
  },

  ...(extraMockHandler('fx.list')
    ? {}
    : {
        'fx.list': (a: Record<string, unknown>) => {
          requireUnlocked();
          const base = typeof a.base === 'string' ? a.base : null;
          const quote = typeof a.quote === 'string' ? a.quote : null;
          if ((base && base !== 'USD') || (quote && quote !== 'BRL')) return [];
          const limit = typeof a.limit === 'number' ? a.limit : 200;
          return usdBrlPoints()
            .slice(-limit)
            .reverse()
            .map(([d, v]): FxRate => ({
              id: `fx-usd-${d}`,
              base: 'USD',
              quote: 'BRL',
              date: d,
              rate: dec(v, 4),
              source: 'bcb_ptax',
              fetched_at: state.fetchedAt,
            }));
        },
      }),
});
