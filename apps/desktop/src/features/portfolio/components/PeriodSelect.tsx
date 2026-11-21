/**
 * Período de análise padronizado da Carteira (docs/portfolio-compare.md): o mesmo seletor em
 * Rentabilidade, Risco, Proventos, Vendas e na ponte do resultado, com a escolha guardada por
 * aba. Opções: no ano, últimos 12, 24 e 60 meses, um ano fechado e desde o início.
 *
 * `periodRange` traduz a escolha para o que cada consulta pede: meses (`YYYY-MM`) para a
 * rentabilidade e a ponte, datas para proventos e vendas, e o número de meses para o risco.
 */
import { Select } from '@/design/components';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { addMonthsToDate } from '@/lib/months';

import { usePortfolioUi, type PeriodScope } from '../state';

export type PeriodKey = 'ytd' | '12m' | '24m' | '36m' | '60m' | 'all' | `y${number}`;

export interface PeriodRange {
  /** Primeiro mês (`YYYY-MM`); sem ele, desde o início. */
  fromMonth?: string;
  /** Último mês (`YYYY-MM`); sem ele, até hoje. */
  toMonth?: string;
  /** Primeiro dia (`YYYY-MM-DD`). */
  fromDate?: string;
  /** Último dia (`YYYY-MM-DD`). */
  toDate?: string;
  /** Meses completos (para o risco, que olha para trás a partir de hoje). */
  months?: number;
}

const ROLLING: Record<string, number> = { '12m': 12, '24m': 24, '36m': 36, '60m': 60 };

export function periodRange(key: PeriodKey, today: string = todayIso()): PeriodRange {
  const year = Number(today.slice(0, 4));
  if (key === 'all') return {};
  if (key === 'ytd') {
    return {
      fromMonth: `${year}-01`,
      fromDate: `${year}-01-01`,
      months: Number(today.slice(5, 7)),
    };
  }
  if (key.startsWith('y')) {
    const y = Number(key.slice(1));
    return {
      fromMonth: `${y}-01`,
      toMonth: `${y}-12`,
      fromDate: `${y}-01-01`,
      toDate: `${y}-12-31`,
      months: 12,
    };
  }
  const n = ROLLING[key] ?? 12;
  const from = addMonthsToDate(today, -(n - 1)).slice(0, 7);
  return { fromMonth: from, fromDate: addMonthsToDate(today, -n), months: n };
}

export function periodLabel(key: PeriodKey): string {
  switch (key) {
    case 'ytd':
      return t('carteira-no-ano');
    case 'all':
      return t('carteira-desde-inicio');
    case '12m':
    case '24m':
    case '36m':
    case '60m':
      return t('carteira-n-meses', { n: ROLLING[key]! });
    default:
      return key.slice(1);
  }
}

/** A escolha de período de uma aba (padrão: a que a aba usava antes). */
export function usePeriod(
  scope: PeriodScope,
  fallback: PeriodKey,
): [PeriodKey, (k: PeriodKey) => void] {
  const key = usePortfolioUi((s) => s.periods[scope]) ?? fallback;
  const set = usePortfolioUi((s) => s.setPeriod);
  return [key, (k) => set(scope, k)];
}

const SCOPE_LABEL: Record<PeriodScope, () => string> = {
  rentabilidade: () => t('carteira-aba-rentabilidade'),
  risco: () => t('carteira-aba-risco'),
  ponte: () => t('carteira-periodo-ponte'),
  proventos: () => t('carteira-aba-proventos'),
  vendas: () => t('carteira-aba-vendas'),
};

export function PeriodSelect({
  scope,
  value,
  onChange,
  years = 6,
  only,
}: {
  scope: PeriodScope;
  value: PeriodKey;
  onChange: (k: PeriodKey) => void;
  /** Quantos anos fechados oferecer (do anterior para trás). */
  years?: number;
  /** Restringe as opções (o risco não aceita "desde o início"). */
  only?: readonly PeriodKey[];
}) {
  const current = Number(todayIso().slice(0, 4));
  const closed = Array.from({ length: years }, (_, i): PeriodKey => `y${current - 1 - i}`);
  const keys = (['ytd', '12m', '24m', '36m', '60m', 'all', ...closed] as PeriodKey[]).filter(
    (k) => !only || only.includes(k),
  );
  return (
    <Select
      aria-label={t('carteira-periodo-de', { aba: SCOPE_LABEL[scope]() })}
      value={value}
      onValueChange={(v) => onChange(v as PeriodKey)}
      className="!h-8 !w-auto min-w-[10rem]"
      options={keys.map((k) => ({ value: k, label: periodLabel(k) }))}
    />
  );
}
