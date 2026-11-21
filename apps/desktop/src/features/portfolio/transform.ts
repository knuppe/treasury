/**
 * Transformações puras dos dados do núcleo para as tabelas e os gráficos da
 * carteira (testadas em `transform.test.ts`). Nada aqui desenha; os valores em
 * `number` servem só para desenho (os textos de dinheiro vêm dos centavos).
 */
import type { AccountView } from '@/api/generated/AccountView';
import type { AssetClass } from '@/api/generated/AssetClass';
import type { BridgeLines } from '@/api/generated/BridgeLines';
import type { ClassSummary } from '@/api/generated/ClassSummary';
import type { CorporateAction } from '@/api/generated/CorporateAction';
import type { DividendCalendarItem } from '@/api/generated/DividendCalendarItem';
import type { IndexValuePoint } from '@/api/generated/IndexValuePoint';
import type { PassiveIncome } from '@/api/generated/PassiveIncome';
import type { Performance } from '@/api/generated/Performance';
import type { PositionView } from '@/api/generated/PositionView';
import type { PricePoint } from '@/api/generated/PricePoint';
import type { Trade } from '@/api/generated/Trade';
import { assignColors, seriesCap } from '@/charts';
import type { ThemeId } from '@/design/themes';
import { t as tr } from '@/i18n';
import { baseCurrency, formatMonthShort, minorUnits } from '@/lib/format';

import { decimalSeparator, decimalToNumber } from './decimal';
import { CLASS_LABEL, CLASS_ORDER, classLabel, deflatorLabel } from './labels';

export const OTHERS_KEY = 'outras';

// ── Cores por classe ─────────────────────────────────────────────

/**
 * Cor de cada classe presa à entidade: a ordem é a fixa do núcleo (`CLASS_ORDER`),
 * filtrada pelas classes que a carteira tem. A partir da (cap + 1)ª, "Outras".
 */
export function classColors(
  classes: Iterable<AssetClass>,
  theme: ThemeId,
): { colors: Map<AssetClass, string>; grouped: Set<AssetClass> } {
  const present = new Set(classes);
  const keys = CLASS_ORDER.filter((c) => present.has(c));
  const map = assignColors(keys, theme) as Map<AssetClass, string>;
  const grouped = new Set(keys.slice(seriesCap(theme)));
  return { colors: map, grouped };
}

export interface AllocationSlice {
  key: AssetClass | typeof OTHERS_KEY;
  label: string;
  /** Centavos (BRL). */
  value: number;
  /** 0–1 */
  weight: number;
  targetPct: number | null;
  deviationPp: number | null;
  outOfBand: boolean;
  color: string;
  /** Classes juntadas em "Outras". */
  members: AssetClass[];
}

/** Fatias do donut de alocação: só classes com valor, maiores primeiro, "Outras" no fim. */
export function allocationSlices(
  byClass: readonly ClassSummary[],
  theme: ThemeId,
  othersColor: string,
): AllocationSlice[] {
  const held = byClass.filter((c) => c.market_value_base > 0);
  const { colors, grouped } = classColors(
    held.map((c) => c.class),
    theme,
  );
  const slices: AllocationSlice[] = [];
  const others: AllocationSlice = {
    key: OTHERS_KEY,
    label: tr('carteira-outras'),
    value: 0,
    weight: 0,
    targetPct: null,
    deviationPp: null,
    outOfBand: false,
    color: othersColor,
    members: [],
  };
  for (const c of held) {
    if (grouped.has(c.class)) {
      others.value += c.market_value_base;
      others.weight += c.weight;
      others.members.push(c.class);
      if (c.target_pct != null) others.targetPct = (others.targetPct ?? 0) + c.target_pct;
      continue;
    }
    slices.push({
      key: c.class,
      label: classLabel(c.class, c.label),
      value: c.market_value_base,
      weight: c.weight,
      targetPct: c.target_pct,
      deviationPp: c.deviation_pp,
      outOfBand: c.out_of_band,
      color: colors.get(c.class) ?? othersColor,
      members: [c.class],
    });
  }
  slices.sort((a, b) => b.value - a.value);
  if (others.members.length > 0) {
    if (others.targetPct != null) others.deviationPp = others.weight * 100 - others.targetPct;
    others.label = tr('carteira-outras-de', {
      classes: others.members.map((c) => CLASS_LABEL[c]).join(', '),
    });
    slices.push(others);
  }
  return slices;
}

// ── Rentabilidade ────────────────────────────────────────────────

export interface LineSeriesData {
  key: string;
  name: string;
  /** Em pontos percentuais (12,3 = 12,3%); `null` onde não há dado. */
  values: (number | null)[];
  dashed?: boolean;
}

function pct(v: number | null | undefined): number | null {
  return v == null || !Number.isFinite(v) ? null : Number((v * 100).toFixed(4));
}

/**
 * Séries do gráfico "rentabilidade acumulada × CDI × inflação × benchmarks", todas em
 * % acumulado no mesmo eixo (nunca eixo duplo).
 */
export function performanceLines(perf: Performance): {
  labels: string[];
  series: LineSeriesData[];
} {
  const labels = perf.months.map((m) => formatMonthShort(m));
  const series: LineSeriesData[] = [
    { key: 'carteira', name: tr('carteira-serie-carteira'), values: perf.cumulative.map(pct) },
    { key: 'cdi', name: perf.benchmark_label || 'CDI', values: perf.cdi_cumulative.map(pct) },
  ];
  if (perf.deflator !== 'none') {
    series.push({
      key: 'inflacao',
      name: deflatorLabel(perf.deflator),
      values: perf.inflation_cumulative.map(pct),
    });
    series.push({
      key: 'real',
      name: tr('carteira-serie-real', { deflator: deflatorLabel(perf.deflator) }),
      values: perf.cumulative_real.map(pct),
      dashed: true,
    });
  }
  const hasData = (v: readonly (number | null)[]) => v.some((x) => x != null);
  if (hasData(perf.ibov_cumulative)) {
    series.push({ key: 'ibov', name: 'Ibovespa', values: perf.ibov_cumulative.map(pct) });
  }
  if (hasData(perf.sp500_brl_cumulative)) {
    series.push({
      key: 'sp500',
      name: tr('carteira-serie-sp500', { moeda: baseCurrency() }),
      values: perf.sp500_brl_cumulative.map(pct),
    });
  }
  if (hasData(perf.usd_cumulative)) {
    series.push({
      key: 'usd',
      name: tr('carteira-serie-dolar'),
      values: perf.usd_cumulative.map(pct),
    });
  }
  return { labels, series };
}

/** Retorno de cada mês (barras), em %. */
export function monthlyReturns(perf: Performance): { labels: string[]; values: (number | null)[] } {
  return { labels: perf.months.map((m) => formatMonthShort(m)), values: perf.monthly.map(pct) };
}

// ── Ponte do resultado ───────────────────────────────────────────

export type BridgeStepKey =
  'inicio' | 'preco' | 'proventos' | 'taxas' | 'impostos' | 'aportes' | 'retiradas' | 'fim';

export interface BridgeStep {
  key: BridgeStepKey;
  /** Quanto a linha soma (negativo nas que tiram: taxas, impostos, retiradas), em centavos. */
  amount: number;
  /** Faixa da barra na cascata (centavos): de `low` a `high`. */
  low: number;
  high: number;
  /** Valor no início e no fim: barras cheias desde o zero. */
  total: boolean;
}

/**
 * Degraus da cascata: início → preço → proventos → taxas → impostos → aportes → retiradas
 * → fim. Pela identidade da ponte, o acumulado do último degrau é o valor no fim.
 */
export function bridgeSteps(l: BridgeLines): BridgeStep[] {
  const bar = (key: BridgeStepKey, amount: number): BridgeStep => ({
    key,
    amount,
    low: Math.min(0, amount),
    high: Math.max(0, amount),
    total: true,
  });
  const moves: [BridgeStepKey, number][] = [
    ['preco', l.price_gain],
    ['proventos', l.income],
    ['taxas', -l.fees],
    ['impostos', -l.taxes],
    ['aportes', l.contributions],
    ['retiradas', -l.withdrawals],
  ];
  const out = [bar('inicio', l.start_value)];
  let run = l.start_value;
  for (const [key, amount] of moves) {
    const next = run + amount;
    out.push({ key, amount, low: Math.min(run, next), high: Math.max(run, next), total: false });
    run = next;
  }
  out.push(bar('fim', l.end_value));
  return out;
}

// ── Renda passiva ────────────────────────────────────────────────

export function passiveIncomeBars(pi: PassiveIncome): {
  months: string[];
  labels: string[];
  values: number[];
} {
  return {
    months: pi.by_month.map((m) => m.month),
    labels: pi.by_month.map((m) => formatMonthShort(m.month)),
    values: pi.by_month.map((m) => m.amount),
  };
}

/** Quanto falta de patrimônio para a meta (0 se já chegou). */
export function capitalGap(capitalNeeded: number | null, marketValue: number): number | null {
  if (capitalNeeded == null) return null;
  return Math.max(0, capitalNeeded - marketValue);
}

// ── Gráfico de preço ─────────────────────────────────────────────

export interface PricePointNum {
  time: string;
  value: number;
}

export interface TradeMarker {
  time: string;
  side: 'buy' | 'sell';
  /** Texto curto no gráfico: `C 100` / `V 50` (no idioma do app). */
  text: string;
  tradeId: string;
}

export interface PriceChartData {
  /** Fechamentos (ou, sem cotações, o preço de cada operação). */
  line: PricePointNum[];
  /** Preço médio a cada evento (degraus; na base de cada dia: cai pela metade num desdobramento 1:2). */
  avg: PricePointNum[];
  markers: TradeMarker[];
  /** `false` quando não há cotações e a linha é feita dos preços das operações. */
  hasPrices: boolean;
}

function shortQty(q: string): string {
  const n = decimalToNumber(q) ?? 0;
  return Number.isInteger(n)
    ? String(n)
    : String(Number(n.toFixed(4))).replace('.', decimalSeparator());
}

/**
 * Dados do gráfico de preço: linha de fechamentos, marcadores de compra/venda (no
 * pregão da operação ou no seguinte com cotação) e o preço médio ponderado por compra,
 * com custos, reproduzido como no núcleo (vendas não mudam o médio; desdobramentos e
 * grupamentos dividem o médio pelo fator).
 */
export function priceChartData(
  prices: readonly PricePoint[],
  trades: readonly Trade[],
  actions: readonly CorporateAction[] = [],
): PriceChartData {
  const sortedTrades = [...trades].sort((a, b) => a.date.localeCompare(b.date));
  const line: PricePointNum[] = [];
  for (const p of [...prices].sort((a, b) => a.date.localeCompare(b.date))) {
    const v = decimalToNumber(p.close);
    if (v == null) continue;
    if (line.length && line[line.length - 1]!.time === p.date) line[line.length - 1]!.value = v;
    else line.push({ time: p.date, value: v });
  }
  const hasPrices = line.length > 0;
  if (!hasPrices) {
    for (const t of sortedTrades) {
      const v = decimalToNumber(t.price);
      if (v == null) continue;
      if (line.length && line[line.length - 1]!.time === t.date) line[line.length - 1]!.value = v;
      else line.push({ time: t.date, value: v });
    }
  }
  const times = line.map((p) => p.time);
  const firstTime = times[0];
  // Com cotações, a janela do gráfico é a das cotações: operações anteriores ficam
  // fora (mas entram no preço médio).
  const snap = (date: string): string | null => {
    if (!times.length) return null;
    if (hasPrices && firstTime && date < firstTime) return null;
    const i = times.findIndex((t) => t >= date);
    return i >= 0 ? times[i]! : null;
  };

  const markers: TradeMarker[] = [];
  for (const t of sortedTrades) {
    const time = snap(t.date);
    if (!time) continue;
    markers.push({
      time,
      side: t.side,
      text:
        t.side === 'buy'
          ? tr('carteira-marcador-compra', { qtd: shortQty(t.quantity) })
          : tr('carteira-marcador-venda', { qtd: shortQty(t.quantity) }),
      tradeId: t.id,
    });
  }

  type Ev =
    { date: string; kind: 'action'; factor: number } | { date: string; kind: 'trade'; t: Trade };
  const events: Ev[] = [
    ...actions
      .filter((a) => (a.kind === 'split' || a.kind === 'reverse_split') && a.factor)
      .map((a): Ev => ({ date: a.date, kind: 'action', factor: decimalToNumber(a.factor) ?? 1 })),
    ...sortedTrades.map((t): Ev => ({ date: t.date, kind: 'trade', t })),
  ].sort(
    (a, b) =>
      a.date.localeCompare(b.date) ||
      (a.kind === 'action' ? -1 : 1) - (b.kind === 'action' ? -1 : 1),
  );

  let qty = 0;
  let cost = 0;
  const steps: PricePointNum[] = [];
  const push = (date: string) => {
    const v = qty > 1e-12 ? cost / qty : null;
    if (v == null) return;
    const last = steps[steps.length - 1];
    if (last && last.time === date) last.value = v;
    else steps.push({ time: date, value: v });
  };
  for (const e of events) {
    if (e.kind === 'action') {
      if (qty > 0) {
        qty *= e.factor;
        push(e.date);
      }
      continue;
    }
    const q = decimalToNumber(e.t.quantity) ?? 0;
    const p = decimalToNumber(e.t.price) ?? 0;
    const costs = (e.t.fees + e.t.taxes) / 10 ** minorUnits(e.t.currency);
    if (e.t.side === 'buy') {
      qty += q;
      cost += q * p + costs;
    } else {
      const sold = Math.min(q, qty);
      if (qty > 0) cost -= (cost / qty) * sold;
      qty -= sold;
      if (qty <= 1e-12) {
        qty = 0;
        cost = 0;
      }
    }
    push(e.date);
  }
  // Degraus (o gráfico desenha com `LineType.WithSteps`): começa no início da janela e
  // estende o último até o fim da linha.
  let avg = dedupe(steps);
  if (hasPrices && firstTime && avg.length) {
    const before = avg.filter((p) => p.time < firstTime);
    const inside = avg.filter((p) => p.time >= firstTime);
    const carry = before[before.length - 1];
    avg =
      carry && inside[0]?.time !== firstTime
        ? [{ time: firstTime, value: carry.value }, ...inside]
        : inside;
  }
  const lastTime = times[times.length - 1];
  if (avg.length && qty > 0 && lastTime && lastTime > avg[avg.length - 1]!.time) {
    avg.push({ time: lastTime, value: avg[avg.length - 1]!.value });
  }
  return { line, avg, markers, hasPrices };
}

function dedupe(points: PricePointNum[]): PricePointNum[] {
  const out: PricePointNum[] = [];
  for (const p of points) {
    const last = out[out.length - 1];
    if (last && p.time <= last.time) {
      if (p.time === last.time) last.value = p.value;
      continue;
    }
    out.push({ ...p });
  }
  return out;
}

// ── Posições ─────────────────────────────────────────────────────

export type GroupMode = 'class' | 'account' | 'none';

/**
 * Nome de cada classe em pt-BR como o núcleo mandava antes da tradução: dá o `prefsId` fixo das
 * tabelas agrupadas por classe ("Posições · Ações"), para as colunas ajustadas valerem em
 * qualquer idioma. Não traduzir.
 */
const CLASS_PREFS_NAME: Record<AssetClass, string> = {
  stock_br: 'Ações',
  fii: 'FIIs',
  etf_br: 'ETFs',
  bdr: 'BDRs',
  fixed_income: 'Renda fixa',
  treasury_br: 'Tesouro Direto',
  fund: 'Fundos',
  stock_us: 'Ações nos EUA',
  etf_us: 'ETFs nos EUA',
  reit: 'REITs',
  crypto: 'Cripto',
  pension: 'Previdência',
  other: 'Outros',
  stock_intl: 'Ações em outras bolsas',
  etf_intl: 'ETFs em outras bolsas',
};

export interface PositionGroup {
  key: string;
  label: string;
  positions: PositionView[];
  /**
   * Onde a tabela do grupo guarda as preferências de colunas: o nome do grupo em pt-BR de antes
   * da tradução (a classe pelo código, `CLASS_PREFS_NAME`; a conta é o nome dado pela família),
   * para valer em qualquer idioma.
   */
  prefsId: string;
  /** Soma do valor de mercado em BRL (centavos), com o custo onde não há valor. */
  value: number;
  cost: number;
  result: number | null;
  weight: number;
  estimated: boolean;
}

export function groupPositions(
  positions: readonly PositionView[],
  mode: GroupMode,
  accounts: readonly Pick<AccountView, 'id' | 'name'>[] = [],
): PositionGroup[] {
  const groups = new Map<string, PositionGroup>();
  const keyOf = (p: PositionView): [string, string, string] => {
    if (mode === 'class')
      return [
        p.asset.class,
        classLabel(p.asset.class, p.asset.class_label),
        Object.hasOwn(CLASS_PREFS_NAME, p.asset.class)
          ? CLASS_PREFS_NAME[p.asset.class]
          : p.asset.class,
      ];
    if (mode === 'account') {
      const id = p.account_id ?? 'todas';
      const name = accounts.find((a) => a.id === id)?.name;
      if (name) return [id, name, name];
      return p.account_id
        ? [id, tr('carteira-conta'), 'Conta']
        : [id, tr('carteira-todas-contas'), 'Todas as contas'];
    }
    return ['todas', tr('carteira-todas-posicoes'), 'Todas as posições'];
  };
  for (const p of positions) {
    const [key, label, prefsLabel] = keyOf(p);
    const g: PositionGroup = groups.get(key) ?? {
      key,
      label,
      prefsId: `Posições · ${prefsLabel}`,
      positions: [],
      value: 0,
      cost: 0,
      result: 0,
      weight: 0,
      estimated: false,
    };
    g.positions.push(p);
    g.value += p.market_value_base ?? p.cost_base;
    g.cost += p.cost_base;
    g.result =
      g.result == null || p.total_result_base == null ? null : g.result + p.total_result_base;
    g.weight += p.weight ?? 0;
    g.estimated ||= p.estimated;
    groups.set(key, g);
  }
  const list = [...groups.values()];
  if (mode === 'class') {
    list.sort(
      (a, b) => CLASS_ORDER.indexOf(a.key as AssetClass) - CLASS_ORDER.indexOf(b.key as AssetClass),
    );
  } else {
    list.sort((a, b) => b.value - a.value);
  }
  return list;
}

/** Escala comum do medidor nominal × real: o maior |XIRR| entre as posições. */
export function meterScale(values: readonly (number | null | undefined)[]): number {
  const max = Math.max(
    0,
    ...values.filter((v): v is number => v != null && Number.isFinite(v)).map(Math.abs),
  );
  return max > 0 ? max : 0.1;
}

// ── Agenda de dividendos ─────────────────────────────────────────

export interface AgendaMonth {
  month: string;
  items: DividendCalendarItem[];
  /** Soma do previsto líquido por moeda (centavos). */
  totals: Map<string, number>;
}

export function agendaByMonth(items: readonly DividendCalendarItem[]): AgendaMonth[] {
  const map = new Map<string, AgendaMonth>();
  for (const it of items) {
    const date = it.pay_date ?? it.ex_date;
    if (!date) continue;
    const month = date.slice(0, 7);
    const m: AgendaMonth = map.get(month) ?? {
      month,
      items: [],
      totals: new Map<string, number>(),
    };
    m.items.push(it);
    if (it.expected_net != null)
      m.totals.set(it.currency, (m.totals.get(it.currency) ?? 0) + it.expected_net);
    map.set(month, m);
  }
  return [...map.values()].sort((a, b) => a.month.localeCompare(b.month));
}

// ── Séries de índices ────────────────────────────────────────────

/**
 * Valores de uma série para o gráfico: taxas diárias (% a.d.) viram % a.a.
 * anualizado (base 252), que é como se lê o CDI; as demais ficam como vêm.
 */
export function indexChartValues(
  kind: string,
  points: readonly IndexValuePoint[],
): { dates: string[]; values: number[]; estimated: boolean[]; unit: string | null } {
  const dates: string[] = [];
  const values: number[] = [];
  const estimated: boolean[] = [];
  for (const p of points) {
    const v = decimalToNumber(p.value);
    if (v == null) continue;
    dates.push(p.date);
    values.push(kind === 'daily_rate' ? Number((((1 + v / 100) ** 252 - 1) * 100).toFixed(4)) : v);
    estimated.push(p.estimated);
  }
  return {
    dates,
    values,
    estimated,
    unit: kind === 'daily_rate' ? tr('carteira-unidade-anualizada') : null,
  };
}
