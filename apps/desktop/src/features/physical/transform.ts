/**
 * Transformações puras do patrimônio físico (08 · Carteira › Bens): textos do método de
 * avaliação, séries das figuras (valor × custo × avaliações; patrimônio do bem × saldo
 * devedor) e totais por tipo.
 */
import type { PhysicalAssetView } from '@/api/generated/PhysicalAssetView';
import type { PhysicalHistoryPoint } from '@/api/generated/PhysicalHistoryPoint';
import type { PhysicalKind } from '@/api/generated/PhysicalKind';
import type { PhysicalValuation } from '@/api/generated/PhysicalValuation';
import type { StackedAreaInput } from '@/charts';
import { rateText } from '@/features/debts/transform';
import { t } from '@/i18n';

import { INDEX_LABEL, KIND_LABEL, SOURCE_LABEL } from './labels';

/** Como o valor é estimado, numa linha curta (ex.: "IGP-M", "12% a.a.", "FIPE 004478-4"). */
export function methodText(a: PhysicalAssetView): string {
  switch (a.valuation_method) {
    case 'index':
      return t('bens-metodo-texto-indice', { indice: INDEX_LABEL[a.index_series ?? 'ipca'] });
    case 'declining':
      return t('bens-metodo-texto-declinante', { taxa: rateText(a.depreciation_rate ?? '12') });
    case 'straight_line':
      return a.residual_pct && Number(a.residual_pct) > 0
        ? t('bens-metodo-texto-linear-residual', {
            anos: a.useful_life_years ?? 5,
            residual: rateText(a.residual_pct),
          })
        : t('bens-metodo-texto-linear', { anos: a.useful_life_years ?? 5 });
    case 'fipe':
      return a.fipe_code
        ? t('bens-metodo-texto-fipe-codigo', {
            codigo: a.fipe_code,
            taxa: rateText(a.depreciation_rate ?? '12'),
          })
        : t('bens-metodo-texto-fipe', { taxa: rateText(a.depreciation_rate ?? '12') });
    case 'none':
      return t('bens-metodo-texto-nenhum');
  }
}

/** Soma dos custos de aquisição (ITBI, escritura…). */
export function acquisitionTotal(a: Pick<PhysicalAssetView, 'acquisition_costs'>): number {
  return a.acquisition_costs.reduce((s, c) => s + c.amount, 0);
}

/** Nomes dos membros, ou "Família". */
export function ownersText(
  ids: readonly string[],
  members: readonly { id: string; name: string }[],
) {
  const names = ids
    .map((id) => members.find((m) => m.id === id)?.name)
    .filter((n): n is string => Boolean(n));
  return names.length ? names.reduce((a, b) => t('bens-donos-par', { a, b })) : t('bens-familia');
}

export interface AppraisalPoint {
  /** Índice do mês no eixo. */
  index: number;
  date: string;
  value: number;
  source: string;
}

export interface ValueChartData {
  months: string[];
  cost: number[];
  value: number[];
  estimated: boolean[];
  appraisals: AppraisalPoint[];
}

/** Valor estimado × custo mês a mês, com as avaliações nos meses delas. */
export function valueChartData(
  history: readonly PhysicalHistoryPoint[],
  valuations: readonly PhysicalValuation[],
): ValueChartData {
  const months = history.map((p) => p.month);
  const appraisals: AppraisalPoint[] = [];
  for (const v of valuations) {
    const index = months.indexOf(v.date.slice(0, 7));
    if (index >= 0)
      appraisals.push({ index, date: v.date, value: v.value, source: SOURCE_LABEL[v.source] });
  }
  return {
    months,
    cost: history.map((p) => p.cost_basis),
    value: history.map((p) => p.value),
    estimated: history.map((p) => p.estimated),
    appraisals,
  };
}

/** Patrimônio do bem e saldo devedor empilhados (a soma é o valor do bem). */
export function equityModel(
  history: readonly PhysicalHistoryPoint[],
  currency: string,
): StackedAreaInput {
  return {
    months: history.map((p) => p.month),
    currency,
    totalLabel: t('bens-valor-do-bem'),
    totals: history.map((p) => p.value),
    series: [
      {
        key: 'equity',
        name: t('bens-patrimonio-do-bem'),
        color: '@s0',
        values: history.map((p) => p.equity),
        stack: 'valor',
      },
      {
        key: 'loan',
        name: t('bens-saldo-devedor'),
        color: '@s1',
        values: history.map((p) => p.loan_balance),
        stack: 'valor',
      },
    ],
  };
}

export interface KindTotal {
  kind: PhysicalKind;
  label: string;
  count: number;
  value: number;
}

/** Valor por tipo de bem (só os não vendidos), do maior para o menor. */
export function totalsByKind(assets: readonly PhysicalAssetView[]): KindTotal[] {
  const map = new Map<PhysicalKind, KindTotal>();
  for (const a of assets) {
    if (a.figures.sold) continue;
    const k = map.get(a.kind) ?? { kind: a.kind, label: KIND_LABEL[a.kind], count: 0, value: 0 };
    k.count += 1;
    k.value += a.figures.value;
    map.set(a.kind, k);
  }
  return [...map.values()].sort((x, y) => y.value - x.value);
}

/** Contratos que ainda não financiam nenhum bem (para o seletor de "ligar financiamento"). */
export function freeLoans<L extends { id: string }>(
  loans: readonly L[],
  assets: readonly PhysicalAssetView[],
): L[] {
  const used = new Set(assets.flatMap((a) => a.loans.map((l) => l.loan_id)));
  return loans.filter((l) => !used.has(l.id));
}
