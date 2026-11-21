/**
 * Transformações puras da aba IR: linhas da apuração do ano, valores prontos para colar
 * no programa da Receita, DARF vencido e as linhas da declaração (rendimentos e bens).
 */
import type { Darf } from '@/api/generated/Darf';
import type { IncomeLine } from '@/api/generated/IncomeLine';
import type { LossPool } from '@/api/generated/LossPool';
import type { TaxMonth } from '@/api/generated/TaxMonth';
import type { TaxYear } from '@/api/generated/TaxYear';
import { t } from '@/i18n';
import { formatLocale, MINUS } from '@/lib/format';

/** Centavos → `1.234,56` (sem `R$`), como o programa da declaração aceita colado. */
export function copyText(cents: number): string {
  const abs = (Math.abs(cents) / 100).toLocaleString('pt-BR', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  return cents < 0 ? `${MINUS}${abs}` : abs;
}

/** Alíquota decimal do núcleo (`"0.15"`) → `15%`. */
export function rateLabel(rate: string): string {
  const n = Number(rate) * 100;
  const code = formatLocale() === 'xx' ? 'pt-BR' : formatLocale();
  return Number.isFinite(n) ? `${n.toLocaleString(code, { maximumFractionDigits: 2 })}%` : rate;
}

/** Resultado de um grupo de prejuízo no mês (0 se o grupo não aparece). */
export function poolResult(m: TaxMonth, pool: LossPool): number {
  return m.pools.find((p) => p.pool === pool)?.result ?? 0;
}

/** Meses com alguma coisa a mostrar (vendas, imposto ou valor acumulado). */
export function activeMonths(year: TaxYear): TaxMonth[] {
  return year.months.filter(
    (m) => m.status !== 'no_activity' || m.carried_in > 0 || m.darf_id != null,
  );
}

/** Mês a abrir por padrão: o último a pagar sem DARF, senão o último com vendas. */
export function defaultMonth(year: TaxYear): string | null {
  const months = activeMonths(year);
  const toPay = [...months].reverse().find((m) => m.status === 'to_pay' && !m.darf_id);
  return toPay?.month ?? months.at(-1)?.month ?? null;
}

/** DARF em aberto depois do vencimento. */
export function darfOverdue(d: Darf, today: string): boolean {
  return d.status === 'pending' && d.due_date < today;
}

export function sumLines(lines: readonly IncomeLine[]): number {
  return lines.reduce((s, l) => s + l.amount, 0);
}

export interface DeclarationLine {
  key: string;
  /** Ficha do programa da Receita. */
  sheet: string;
  /** Código sugerido (confira no programa). */
  code: string | null;
  label: string;
  amount: number;
}

/**
 * Linhas da declaração vindas do relatório anual: rendimentos isentos (dividendos,
 * rendimentos de FII, ganhos isentos de ações), tributação exclusiva (JCP) e prejuízos a
 * compensar em 31/12. Códigos sugeridos; os de FII mudam entre anos e ficam em branco.
 */
export function declarationLines(y: TaxYear): DeclarationLine[] {
  const lines: DeclarationLine[] = [
    {
      key: 'dividends',
      sheet: t('ir-ficha-isentos'),
      code: '09',
      label: t('ir-linha-dividendos'),
      amount: sumLines(y.dividends),
    },
    {
      key: 'exempt-stocks',
      sheet: t('ir-ficha-isentos'),
      code: '20',
      label: t('ir-linha-isentos-acoes'),
      amount: y.exempt_stock_gains,
    },
    {
      key: 'fii',
      sheet: t('ir-ficha-isentos'),
      code: null,
      label: t('ir-linha-fii'),
      amount: sumLines(y.fii_income),
    },
    {
      key: 'jcp',
      sheet: t('ir-ficha-exclusiva'),
      code: '10',
      label: t('ir-linha-jcp'),
      amount: sumLines(y.jcp),
    },
  ];
  return lines;
}

/** Prejuízos a compensar em 31/12 (Renda variável), por grupo. */
export function lossesAtYearEnd(y: TaxYear): { pool: LossPool; amount: number }[] {
  return [
    { pool: 'common' as const, amount: y.loss_common },
    { pool: 'day_trade' as const, amount: y.loss_day_trade },
    { pool: 'fii' as const, amount: y.loss_fii },
  ];
}

/** Anos para escolher: do atual para trás. */
export function yearOptions(today: string, count = 5): number[] {
  const y = Number(today.slice(0, 4));
  return Array.from({ length: count }, (_, i) => y - i);
}
