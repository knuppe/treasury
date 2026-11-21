/**
 * Mock dos dados dos ativos com fonte (`assets.data_gaps`, `assets.field_sources`,
 * `assets.enrich_apply`, `assets.field_lock`): leituras fixas por ticker, como se a coleta da
 * CVM, da SEC e do Fundamentus já tivesse rodado, e as mesmas regras do núcleo (nível da
 * fonte, +5 por fonte que concorda, −25 com fonte do mesmo nível discordando, conflito,
 * valor do usuário protegido, trava). Versão simplificada de `assets/enrich.rs`.
 */
import type { AssetClass } from '../generated/AssetClass';
import type { AssetData } from '../generated/AssetData';
import type { AssetFieldSources } from '../generated/AssetFieldSources';
import type { DataGaps } from '../generated/DataGaps';
import type { FieldBand } from '../generated/FieldBand';
import type { FieldCandidate } from '../generated/FieldCandidate';
import type { FieldObservation } from '../generated/FieldObservation';
import type { FieldOrigin } from '../generated/FieldOrigin';
import type { FieldStatus } from '../generated/FieldStatus';
import type { StoredFieldObservation } from '../generated/StoredFieldObservation';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';
import { allAssets, getAsset } from './assets';

const COLUMNS = ['sector', 'segment', 'isin', 'cnpj'] as const;
type Column = (typeof COLUMNS)[number];

const STOCKS: AssetClass[] = ['stock_br', 'bdr', 'stock_us', 'reit', 'stock_intl'];
const LISTED: AssetClass[] = [
  'stock_br',
  'fii',
  'etf_br',
  'bdr',
  'stock_us',
  'etf_us',
  'reit',
  'stock_intl',
  'etf_intl',
];
const FUNDS: AssetClass[] = ['etf_br', 'fund', 'pension'];

/** Catálogo (mesma ordem e classes do núcleo). */
const FIELDS: [string, AssetClass[]][] = [
  ['sector', STOCKS],
  ['subsector', ['stock_br', 'bdr']],
  ['segment', ['fii', 'stock_br', 'bdr']],
  ['isin', LISTED],
  ['cnpj', ['fii', 'etf_br', 'fund', 'pension']],
  ['issuer_cnpj', ['stock_br', 'bdr']],
  ['issuer_name', STOCKS],
  ['share_kind', ['stock_br', 'fii', 'stock_us']],
  ['listing_segment', ['stock_br']],
  ['cvm_code', ['stock_br']],
  ['cik', ['stock_us', 'reit']],
  ['fii_mandate', ['fii']],
  ['fii_management', ['fii']],
  ['fund_class_cvm', FUNDS],
  ['fund_class_anbima', FUNDS],
  ['benchmark', FUNDS],
  ['manager', FUNDS],
  ['administrator', FUNDS],
  ['lei', ['stock_br', 'bdr']],
  ['website', ['stock_br', 'bdr']],
  ['activity', ['stock_br', 'bdr']],
  ['cnpj_status', ['stock_br', 'bdr', 'fii', 'etf_br', 'fund']],
];

type Seed = [field: string, value: string, source: string, tier: number, note?: string];

const AS_OF = '2026-10-14';

/** Leituras "baixadas" por ticker. */
const READINGS: Record<string, Seed[]> = {
  PETR4: [
    ['sector', 'Petróleo e Gás', 'cvm_fca', 1],
    ['sector', 'Petróleo, Gás e Biocombustíveis', 'fundamentus', 3],
    ['subsector', 'Exploração, Refino e Distribuição', 'fundamentus', 3],
    ['isin', 'BRPETRACNPR6', 'cvm_fca', 1],
    ['issuer_cnpj', '33.000.167/0001-01', 'cvm_fca', 1],
    ['issuer_name', 'PETRÓLEO BRASILEIRO S.A. PETROBRAS', 'cvm_fca', 1],
    ['share_kind', 'PN', 'cvm_fca', 1],
    ['listing_segment', 'Nível 2', 'cvm_fca', 1],
    ['cvm_code', '9512', 'cvm_fca', 1],
  ],
  ITUB4: [
    ['subsector', 'Bancos', 'cvm_fca', 1],
    ['subsector', 'Intermediários Financeiros', 'b3', 1],
    ['isin', 'BRITUBACNPR1', 'cvm_fca', 1],
    ['share_kind', 'PN', 'cvm_fca', 1],
  ],
  TAEE11: [
    ['sector', 'Energia Elétrica', 'cvm_fca', 1],
    ['sector', 'Energia Elétrica', 'fundamentus', 3],
    ['isin', 'BRTAEECDAM10', 'cvm_fca', 1],
    ['issuer_cnpj', '07.859.971/0001-30', 'cvm_fca', 1],
    ['share_kind', 'UNIT', 'cvm_fca', 1],
  ],
  HGLG11: [
    ['segment', 'Logística', 'cvm_fii', 1],
    ['segment', 'Logística', 'fundamentus', 3],
    ['cnpj', '11.728.688/0001-47', 'cvm_fii', 1],
    ['fii_mandate', 'Renda', 'cvm_fii', 1],
    ['fii_management', 'Ativa', 'fundamentus', 3],
  ],
  BOVA11: [
    ['cnpj', '10.406.511/0001-61', 'cvm_registro', 1, 'matched_by_name'],
    ['fund_class_cvm', 'Ações', 'cvm_registro', 1, 'matched_by_name'],
    ['benchmark', 'Ibovespa', 'cvm_registro', 1, 'matched_by_name'],
    [
      'manager',
      'BlackRock Brasil Gestora de Investimentos Ltda',
      'cvm_registro',
      1,
      'matched_by_name',
    ],
  ],
  AAPL: [
    ['sector', 'Electronic Computers', 'sec', 1],
    ['issuer_name', 'Apple Inc.', 'sec', 1],
    ['cik', '320193', 'sec', 1],
  ],
};

/** Leituras da rede (B3, GLEIF, BrasilAPI), só depois de `assets.enrich_lookup`. */
const NETWORK: Record<string, Seed[]> = {
  TAEE11: [
    ['sector', 'Utilidade Pública', 'b3', 1],
    ['subsector', 'Energia Elétrica', 'b3', 1],
    ['segment', 'Energia Elétrica', 'b3', 1],
    ['activity', 'Transmissão de Energia Elétrica.', 'b3', 1],
    ['website', 'https://www.taesa.com.br', 'b3', 1],
    ['cnpj_status', 'ATIVA', 'brasilapi', 3],
  ],
  PETR4: [
    ['sector', 'Petróleo, Gás e Biocombustíveis', 'b3', 1],
    ['lei', '5493000J801JZRCMFE49', 'gleif', 1],
    ['cnpj_status', 'ATIVA', 'brasilapi', 3],
  ],
};

const norm = (s: string) =>
  s
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .trim();

const WEIGHT: Record<number, number> = { 1: 90, 2: 80, 3: 60, 4: 15 };
const band = (c: number): FieldBand => (c >= 85 ? 'high' : c >= 60 ? 'medium' : 'low');

interface Meta {
  origin: FieldOrigin;
  source: string | null;
  as_of: string | null;
  confidence: number | null;
  locked: boolean;
}

interface State {
  meta: Map<string, Meta>;
  attrs: Map<string, string>;
  history: Map<string, StoredFieldObservation[]>;
  /** Tickers já consultados na rede. */
  looked: Set<string>;
}

let st: State = fresh();
function fresh(): State {
  return { meta: new Map(), attrs: new Map(), history: new Map(), looked: new Set() };
}
onMockReset(() => {
  st = fresh();
});

const k = (id: string, f: string) => `${id}:${f}`;

function observations(ticker: string | null): FieldObservation[] {
  const t = ticker ?? '';
  const seeds = [...(READINGS[t] ?? []), ...(st.looked.has(t) ? (NETWORK[t] ?? []) : [])];
  // Com a classificação da B3, o setor do FCA só confere (não disputa).
  const b3 = seeds.some(([f, , src]) => f === 'sector' && src === 'b3');
  return seeds
    .filter(([f, , src]) => !(b3 && f === 'sector' && src === 'cvm_fca'))
    .map(([field, value, source, tier, note]) => ({
      field,
      value,
      value_norm: norm(value),
      scheme: null,
      source,
      tier,
      source_ref: null,
      as_of: AS_OF,
      note: note ?? null,
    }));
}

function rank(obs: FieldObservation[]): { ranked: FieldCandidate[]; conflict: boolean } {
  const groups = new Map<string, FieldObservation[]>();
  for (const o of obs) groups.set(o.value_norm, [...(groups.get(o.value_norm) ?? []), o]);
  const best = [...groups.values()]
    .map((g) => [...g].sort((a, b) => a.tier - b.tier))
    .sort((a, b) => a[0]!.tier - b[0]!.tier || b.length - a.length);
  const top = best[0]?.[0]?.tier;
  const conflict = best.filter((g) => g[0]!.tier === top).length > 1;
  const ranked = best.map((g) => {
    const lead = g[0]!;
    const agreeing = [...new Set(g.map((o) => o.source).filter((s) => s !== lead.source))];
    let c = WEIGHT[lead.tier]! + 5 * Math.min(2, agreeing.length);
    if (best.some((o) => o[0]!.value_norm !== lead.value_norm && o[0]!.tier <= lead.tier)) c -= 25;
    if (lead.note === 'matched_by_name') c -= 25;
    c = Math.max(0, Math.min(100, c));
    return {
      value: lead.value,
      source: lead.source,
      tier: lead.tier,
      as_of: lead.as_of,
      source_ref: null,
      confidence: c,
      band: band(c),
      agreeing,
      note: lead.note,
    } satisfies FieldCandidate;
  });
  return { ranked, conflict };
}

function current(id: string, field: string): string | null {
  if ((COLUMNS as readonly string[]).includes(field)) {
    const v = getAsset(id)[field as Column];
    return v && v.trim() ? v : null;
  }
  return st.attrs.get(k(id, field)) ?? null;
}

function statuses(id: string): { fields: FieldStatus[]; obs: FieldObservation[] } {
  const a = getAsset(id);
  const obs = observations(a.ticker);
  const fields: FieldStatus[] = [];
  for (const [field, classes] of FIELDS) {
    const mine = obs.filter((o) => o.field === field);
    const cur = current(id, field);
    if (!classes.includes(a.class) && mine.length === 0 && cur == null) continue;
    const meta = st.meta.get(k(id, field));
    const locked = meta?.locked ?? false;
    const origin: FieldOrigin | null = meta?.origin ?? (cur != null ? 'legacy' : null);
    const { ranked, conflict } = rank(mine);
    let state: FieldStatus['state'];
    let candidate: FieldCandidate | null = null;
    let options: FieldCandidate[] = [];
    if (locked) state = 'ok';
    else if (ranked.length === 0) state = cur != null ? 'ok' : 'missing';
    else if (
      cur != null &&
      ranked.some((c) => norm(c.value) === norm(cur) && c.tier === ranked[0]!.tier)
    )
      state = 'ok';
    else if (conflict) {
      state = 'conflict';
      options = ranked.filter((c) => c.tier === ranked[0]!.tier);
    } else {
      candidate = ranked[0]!;
      state =
        origin == null ? 'fill' : origin === 'user' || origin === 'legacy' ? 'diverge' : 'update';
    }
    fields.push({
      field,
      current: cur,
      origin,
      locked,
      current_source: meta?.source ?? null,
      current_as_of: meta?.as_of ?? null,
      current_confidence: meta?.confidence ?? null,
      state,
      candidate,
      options,
      effect: null,
    });
  }
  return { fields, obs };
}

const str = (v: unknown) => (typeof v === 'string' ? v : '');

registerMockHandlers({
  'assets.data_gaps': (args): DataGaps => {
    requireUnlocked();
    const ids = Array.isArray(args.asset_ids)
      ? (args.asset_ids as string[])
      : allAssets()
          .filter((a) => !a.archived)
          .map((a) => a.id);
    const summary = {
      assets: 0,
      missing_sector: 0,
      missing_isin: 0,
      fill: 0,
      fill_high: 0,
      updates: 0,
      conflicts: 0,
      divergences: 0,
    };
    const assets: AssetData[] = [];
    for (const id of ids) {
      const a = getAsset(id);
      const { fields } = statuses(id);
      summary.assets += 1;
      for (const f of fields) {
        const applicable = FIELDS.find(([n]) => n === f.field)?.[1].includes(a.class) ?? false;
        if (applicable && f.current == null && f.field === 'sector') summary.missing_sector += 1;
        if (applicable && f.current == null && f.field === 'isin') summary.missing_isin += 1;
        if (f.state === 'fill') {
          summary.fill += 1;
          if (f.candidate?.band === 'high') summary.fill_high += 1;
        }
        if (f.state === 'update') summary.updates += 1;
        if (f.state === 'conflict') summary.conflicts += 1;
        if (f.state === 'diverge') summary.divergences += 1;
      }
      const shown =
        args.include_ok === true
          ? fields
          : fields.filter((f) => f.state !== 'ok' && f.state !== 'missing');
      if (args.include_ok === true || shown.length > 0) {
        assets.push({
          asset_id: id,
          ticker: a.ticker,
          name: a.name,
          class: a.class,
          fields: shown,
        });
      }
    }
    return { summary, assets };
  },

  'assets.field_sources': (args): AssetFieldSources => {
    requireUnlocked();
    const id = str(args.asset_id);
    const { fields, obs } = statuses(id);
    return {
      asset_id: id,
      fields: fields.map((status) => ({
        status,
        observations: obs.filter((o) => o.field === status.field),
        history: st.history.get(k(id, status.field)) ?? [],
      })),
    };
  },

  'assets.enrich_apply': (args) => {
    requireUnlocked();
    const items = Array.isArray(args.items) ? (args.items as Record<string, unknown>[]) : [];
    if (items.length === 0) mockFail('invalid', 'nenhum campo escolhido');
    const assets = new Set<string>();
    for (const it of items) {
      const id = str(it.asset_id);
      const field = str(it.field);
      const a = getAsset(id);
      const label = a.ticker ?? a.name;
      const meta = st.meta.get(k(id, field));
      if (meta?.locked)
        mockFail('conflict', `o campo ${field} de ${label} está travado para sugestões`);
      const mine = observations(a.ticker).filter((o) => o.field === field);
      const c = rank(mine).ranked.find((x) => norm(x.value) === norm(str(it.value)));
      if (!c) {
        mockFail(
          'invalid',
          `o valor "${str(it.value)}" para ${field} de ${label} não está em nenhuma fonte registrada no cofre`,
        );
      }
      const cur = current(id, field);
      const origin = meta?.origin ?? (cur != null ? 'legacy' : null);
      if (
        cur != null &&
        norm(cur) !== norm(c.value) &&
        it.replace !== true &&
        (origin === 'user' || origin === 'legacy')
      ) {
        mockFail(
          'conflict',
          `o campo ${field} de ${label} já tem um valor seu; para trocar, peça a troca (replace)`,
        );
      }
      if ((COLUMNS as readonly string[]).includes(field)) a[field as Column] = c.value;
      else st.attrs.set(k(id, field), c.value);
      st.meta.set(k(id, field), {
        origin: 'enrichment',
        source: c.source,
        as_of: c.as_of,
        confidence: c.confidence,
        locked: false,
      });
      st.history.set(
        k(id, field),
        mine.map((o) => ({
          value: o.value,
          source: o.source,
          tier: o.tier,
          source_ref: o.source_ref,
          published_at: o.as_of,
          observed_at: new Date().toISOString(),
        })),
      );
      assets.add(id);
    }
    return { fields: items.length, assets: assets.size };
  },

  'assets.enrich_lookup': (args) => {
    requireUnlocked();
    const id = str(args.asset_id);
    const a = getAsset(id);
    const t = a.ticker ?? '';
    const fresh = st.looked.has(t) && args.refresh !== true;
    const n = fresh ? 0 : (NETWORK[t]?.length ?? 0);
    st.looked.add(t);
    const status = fresh ? 'fresh' : n > 0 ? 'ok' : 'not_found';
    const covered = ['stock_br', 'bdr', 'fii', 'etf_br', 'fund', 'pension'].includes(a.class);
    const stock = a.class === 'stock_br' || a.class === 'bdr';
    const sources = !covered
      ? []
      : (stock ? ['b3', 'gleif', 'brasilapi'] : ['brasilapi']).map((source) => ({
          source,
          status,
          detail: null,
        }));
    const { fields } = statuses(id);
    return {
      asset_id: id,
      sources,
      observations: n,
      data: { asset_id: id, ticker: a.ticker, name: a.name, class: a.class, fields },
    };
  },

  'assets.enrich_lookup_all': () => {
    requireUnlocked();
    let observations = 0;
    let assets = 0;
    for (const a of allAssets().filter((x) => !x.archived)) {
      const t = a.ticker ?? '';
      assets += 1;
      if (!st.looked.has(t)) observations += NETWORK[t]?.length ?? 0;
      st.looked.add(t);
    }
    return { assets, fetched: assets, observations, errors: 0 };
  },

  'assets.field_lock': (args) => {
    requireUnlocked();
    const id = str(args.asset_id);
    const field = str(args.field);
    getAsset(id);
    const prev = st.meta.get(k(id, field));
    st.meta.set(k(id, field), {
      origin: prev?.origin ?? (current(id, field) != null ? 'legacy' : 'user'),
      source: prev?.source ?? null,
      as_of: prev?.as_of ?? null,
      confidence: prev?.confidence ?? null,
      locked: args.locked === true,
    });
    return null;
  },
});
