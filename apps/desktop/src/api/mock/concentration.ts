/**
 * Mock da concentração da carteira (`portfolio/concentration.rs`) e das etiquetas de ativos
 * (`assets/labels.rs`): limites em memória (ativo 20% e grupo 30% por padrão, o limite próprio
 * de cada ativo), etiquetas com os ativos e a marca de grupo de concentração. Os alertas saem do
 * resumo da carteira com as mesmas regras do núcleo: acima do limite, `warning`; a partir de
 * 1,5 vez, `critical`; a classe só tem limite com alvo (alvo + tolerância).
 */
import type { AssetBrief } from '../generated/AssetBrief';
import type { AssetLabel } from '../generated/AssetLabel';
import type { ConcentrationAlert } from '../generated/ConcentrationAlert';
import type { ConcentrationItem } from '../generated/ConcentrationItem';
import type { ConcentrationReport } from '../generated/ConcentrationReport';
import type { ConcentrationSeverity } from '../generated/ConcentrationSeverity';
import type { PortfolioSummary } from '../generated/PortfolioSummary';
import type { RebalanceConcentrationLine } from '../generated/RebalanceConcentrationLine';
import type { SetAssetLabelsArgs } from '../generated/SetAssetLabelsArgs';
import type { SetConcentrationLimitArgs } from '../generated/SetConcentrationLimitArgs';
import type { UpdateAssetLabelArgs } from '../generated/UpdateAssetLabelArgs';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';
import { getAsset, hasAsset } from './assets';

const ASSET_DEFAULT_PCT = 20;
const GROUP_DEFAULT_PCT = 30;
const CRITICAL_FACTOR = 1.5;
const MAX_PER_ASSET = 20;
const MAX_NAME = 40;

/** Etiquetas de exemplo (inventadas): "Exterior" é grupo de concentração; "Bancos", não. */
const SEED_LABELS: Omit<AssetLabel, 'id'>[] = [
  {
    name: 'Bancos',
    is_concentration_group: false,
    concentration_limit_pct: null,
    asset_ids: ['ast-bbas3', 'ast-itub4'],
  },
  {
    name: 'Exterior',
    is_concentration_group: true,
    concentration_limit_pct: null,
    asset_ids: ['ast-aapl', 'ast-msft', 'ast-voo'],
  },
];

interface State {
  assetDefault: number | null;
  groupDefault: number | null;
  /** Limite próprio por ativo (% da carteira). */
  assetLimits: Map<string, number>;
  /** `null`: ainda não semeadas (as de exemplo só entram com os ativos do cofre). */
  labels: AssetLabel[] | null;
  seq: number;
}

let state: State = fresh();

function fresh(): State {
  return { assetDefault: null, groupDefault: null, assetLimits: new Map(), labels: null, seq: 0 };
}

onMockReset(() => {
  state = fresh();
});

function labels(): AssetLabel[] {
  if (state.labels == null) {
    state.labels = SEED_LABELS.map((l, i) => ({
      ...l,
      id: `lbl-${i + 1}`,
      asset_ids: l.asset_ids.filter((id) => hasAsset(id)),
    })).filter((l) => l.asset_ids.length > 0);
    state.seq = state.labels.length;
  }
  return state.labels;
}

const byName = (a: AssetLabel, b: AssetLabel) =>
  a.name.localeCompare(b.name, 'pt-BR', { sensitivity: 'base' });

function listLabels(): AssetLabel[] {
  return labels()
    .map((l) => ({ ...l, asset_ids: [...l.asset_ids].sort() }))
    .sort(byName);
}

function cleanName(raw: unknown): string {
  const n = (typeof raw === 'string' ? raw : '').trim();
  if (!n || [...n].length > MAX_NAME) mockFail('invalid', 'etiqueta inválida');
  return n;
}

/** A etiqueta pelo nome (sem diferenciar maiúsculas), criada se não existir. */
function ensure(name: string): AssetLabel {
  const found = labels().find((l) => l.name.toLowerCase() === name.toLowerCase());
  if (found) return found;
  state.seq += 1;
  const l: AssetLabel = {
    id: `lbl-${state.seq}`,
    name,
    is_concentration_group: false,
    concentration_limit_pct: null,
    asset_ids: [],
  };
  labels().push(l);
  return l;
}

function checkPct(p: unknown): number | null {
  if (p == null) return null;
  const v = Number(p);
  if (!(v > 0 && v <= 100)) mockFail('invalid', 'o limite deve estar entre 0% e 100%');
  return v;
}

export function setConcentrationLimit(a: SetConcentrationLimitArgs): void {
  const pct = checkPct(a.pct);
  if (a.scope === 'asset') {
    if (!a.asset_id) mockFail('invalid', 'informe o ativo do limite');
    getAsset(a.asset_id);
    if (pct == null) state.assetLimits.delete(a.asset_id);
    else state.assetLimits.set(a.asset_id, pct);
  } else if (a.scope === 'asset_default') state.assetDefault = pct;
  else if (a.scope === 'group_default') state.groupDefault = pct;
  else mockFail('invalid', 'escopo de limite inválido');
}

const assetDefault = () => state.assetDefault ?? ASSET_DEFAULT_PCT;
const groupDefault = () => state.groupDefault ?? GROUP_DEFAULT_PCT;

/** Valor por ativo (somando as contas) e o valor da carteira. */
function weights(s: PortfolioSummary): [[AssetBrief, number][], number] {
  const by: [AssetBrief, number][] = [];
  for (const p of s.positions) {
    const v = Math.max(0, p.market_value_base ?? p.cost_base);
    const e = by.find(([a]) => a.id === p.asset.id);
    if (e) e[1] += v;
    else by.push([p.asset, v]);
  }
  return [by, Math.max(0, s.market_value)];
}

function severity(weight: number, limit: number): ConcentrationSeverity | null {
  if (weight >= limit * CRITICAL_FACTOR) return 'critical';
  if (weight > limit) return 'warning';
  return null;
}

const excess = (weight: number, limit: number, total: number) =>
  Math.round((weight - limit) * total);

const assetLabelOf = (a: AssetBrief) => a.ticker ?? a.name;

/**
 * Alertas de concentração de um resumo. `targets`: alvo e tolerância por classe (em %),
 * como `classTargets()` do mock da carteira.
 */
/** Valor da garantia do FGC por CPF e instituição, em centavos (metodologia). */
const FGC_LIMIT = 250_000_00;
const TESOURO = new Set(['tesouro_selic', 'tesouro_prefixado', 'tesouro_ipca', 'tesouro_renda']);
const FGC = new Set(['cdb', 'rdb', 'lc', 'lci', 'lca', 'poupanca']);

/** Fora do limite padrão (como `concentration::default_exempt`): Tesouro Direto e renda fixa
 * com FGC enquanto o ativo cabe na garantia. Um limite próprio vale sempre. */
function defaultExempt(a: AssetBrief, value: number): boolean {
  if (a.class === 'treasury_br') return true;
  if (a.class !== 'fixed_income' || !hasAsset(a.id)) return false;
  const kind = getAsset(a.id).fixed_income?.kind;
  if (!kind) return false;
  if (TESOURO.has(kind)) return true;
  return FGC.has(kind) && value <= FGC_LIMIT;
}

/** O limite do ativo (fração), ou `null` quando ele está fora do padrão. */
function limitFor(a: AssetBrief, value: number): number | null {
  const own = state.assetLimits.get(a.id);
  if (own != null) return own / 100;
  return defaultExempt(a, value) ? null : assetDefault() / 100;
}

export function concentrationAlerts(
  s: PortfolioSummary,
  targets: Map<string, [number, number]>,
): ConcentrationAlert[] {
  const [assets, total] = weights(s);
  if (total <= 0) return [];
  const out: ConcentrationAlert[] = [];
  for (const [a, v] of assets) {
    const weight = v / total;
    const limit = limitFor(a, v);
    if (limit == null) continue;
    const sev = severity(weight, limit);
    if (sev)
      out.push({
        kind: 'asset',
        ref: a.id,
        label: assetLabelOf(a),
        weight,
        limit,
        severity: sev,
        excess_base: excess(weight, limit, total),
        assets: [a],
      });
  }
  for (const l of labels().filter((x) => x.is_concentration_group)) {
    const members = assets.filter(([a]) => l.asset_ids.includes(a.id));
    const weight = members.reduce((acc, [, v]) => acc + v, 0) / total;
    const limit = (l.concentration_limit_pct ?? groupDefault()) / 100;
    const sev = severity(weight, limit);
    if (sev)
      out.push({
        kind: 'group',
        ref: l.name,
        label: l.name,
        weight,
        limit,
        severity: sev,
        excess_base: excess(weight, limit, total),
        assets: members.map(([a]) => a),
      });
  }
  for (const c of s.by_class) {
    const t = targets.get(c.class);
    if (!t) continue;
    const limit = (t[0] + t[1]) / 100;
    if (limit <= 0) continue;
    const sev = severity(c.weight, limit);
    if (sev)
      out.push({
        kind: 'class',
        ref: c.class,
        label: c.label,
        weight: c.weight,
        limit,
        severity: sev,
        excess_base: excess(c.weight, limit, total),
        assets: [],
      });
  }
  return out.sort(
    (a, b) =>
      Number(b.severity === 'critical') - Number(a.severity === 'critical') ||
      b.excess_base - a.excess_base,
  );
}

export function concentrationReport(
  s: PortfolioSummary,
  targets: Map<string, [number, number]>,
): ConcentrationReport {
  const [assets, total] = weights(s);
  const w = (v: number) => (total > 0 ? v / total : 0);
  const items: ConcentrationItem[] = assets
    .map(([a, v]) => {
      const own = state.assetLimits.get(a.id);
      const limit = limitFor(a, v);
      return {
        ref: a.id,
        label: assetLabelOf(a),
        weight: w(v),
        limit: limit ?? 1,
        exempt: limit == null,
        custom_limit: own != null,
        value_base: v,
        assets: [a],
      };
    })
    .sort((a, b) => b.value_base - a.value_base);
  const groups: ConcentrationItem[] = listLabels()
    .filter((l) => l.is_concentration_group)
    .map((l) => {
      const members = assets.filter(([a]) => l.asset_ids.includes(a.id));
      const v = members.reduce((acc, [, x]) => acc + x, 0);
      return {
        ref: l.name,
        label: l.name,
        weight: w(v),
        limit: (l.concentration_limit_pct ?? groupDefault()) / 100,
        exempt: false,
        custom_limit: l.concentration_limit_pct != null,
        value_base: v,
        assets: members.map(([a]) => a),
      };
    });
  return {
    market_value_base: total,
    asset_default_pct: assetDefault(),
    group_default_pct: groupDefault(),
    alerts: concentrationAlerts(s, targets),
    assets: items,
    groups,
  };
}

/**
 * Aporte pelos limites (o `concentration` do rebalanceamento sem alvo de classe): cada ativo
 * recebe a parte do aporte proporcional ao espaço que sobra até o limite dele e dos grupos dele;
 * quem já está acima do limite fica com zero. Nunca sugere vender.
 */
export function concentrationRoom(
  s: PortfolioSummary,
  contribution: number,
): RebalanceConcentrationLine[] {
  const [assets, total] = weights(s);
  const after = total + Math.max(0, contribution);
  if (after <= 0) return [];
  const groups = labels().filter((l) => l.is_concentration_group);
  const lines = assets.map(([a, v]) => {
    const limit = (state.assetLimits.get(a.id) ?? assetDefault()) / 100;
    let room = Math.max(0, Math.round(limit * after - v));
    for (const g of groups.filter((x) => x.asset_ids.includes(a.id))) {
      const gv = assets
        .filter(([x]) => g.asset_ids.includes(x.id))
        .reduce((acc, [, x]) => acc + x, 0);
      const gl = (g.concentration_limit_pct ?? groupDefault()) / 100;
      room = Math.min(room, Math.max(0, Math.round(gl * after - gv)));
    }
    // Isento do padrão: o aporte ainda se espalha pelo padrão, mas sem o selo de acima.
    const overLimit = limitFor(a, v) != null && total > 0 && v / total > limit;
    return { a, v, limit, room: overLimit ? 0 : room, overLimit };
  });
  const totalRoom = lines.reduce((acc, l) => acc + l.room, 0);
  const c = Math.max(0, contribution);
  return lines
    .map((l) => {
      const suggested =
        c > 0 && totalRoom > 0 ? Math.min(l.room, Math.floor((c * l.room) / totalRoom)) : 0;
      return {
        asset: l.a,
        current: l.v,
        weight: (l.v + suggested) / after,
        limit: l.limit,
        room: l.room,
        suggested,
        over_limit: l.overLimit,
      };
    })
    .sort((x, y) => y.suggested - x.suggested || y.current - x.current);
}

registerMockHandlers({
  'assets.labels_list': () => {
    requireUnlocked();
    return listLabels();
  },

  'assets.labels_set': (raw) => {
    requireUnlocked();
    const a = raw as unknown as SetAssetLabelsArgs;
    getAsset(String(a.asset_id));
    const names = Array.isArray(a.labels) ? a.labels.map(cleanName) : [];
    if (names.length > MAX_PER_ASSET) mockFail('invalid', 'etiqueta inválida');
    const chosen = names.map(ensure);
    for (const l of labels()) {
      const has = l.asset_ids.includes(a.asset_id);
      const want = chosen.includes(l);
      if (has && !want) l.asset_ids = l.asset_ids.filter((id) => id !== a.asset_id);
      if (!has && want) l.asset_ids.push(a.asset_id);
    }
    // Como no núcleo: as que ficam sem ativo e sem papel de grupo somem.
    state.labels = labels().filter(
      (l) =>
        l.asset_ids.length > 0 || l.is_concentration_group || l.concentration_limit_pct != null,
    );
    return listLabels()
      .filter((l) => l.asset_ids.includes(a.asset_id))
      .map((l) => l.name);
  },

  'assets.label_update': (raw) => {
    requireUnlocked();
    const a = raw as unknown as UpdateAssetLabelArgs;
    const l = ensure(cleanName(a.name));
    if (a.is_concentration_group != null)
      l.is_concentration_group = Boolean(a.is_concentration_group);
    if (a.concentration_limit_pct !== undefined)
      l.concentration_limit_pct = checkPct(a.concentration_limit_pct);
    return { ...l, asset_ids: [...l.asset_ids].sort() };
  },
});
