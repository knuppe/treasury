/**
 * Formatação dos números da análise de ativos. No núcleo (`research::indicators`), razões
 * vêm em fração (0,083 = 8,3%), múltiplos em vezes e dinheiro em unidades da moeda (não
 * centavos); o histórico (`research.history`) vem em centavos. Sem moeda informada, o valor
 * é dos fundamentos da CVM, em reais.
 */
import type { IndicatorUnit } from '@/api/generated/IndicatorUnit';
import { t } from '@/i18n';
import { currencyPrefix, formatLocale, formatNumber, formatPercent, MINUS } from '@/lib/format';

import { formatUnitPrice } from '../portfolio/decimal';

const nf = (decimals: number) => {
  const code = formatLocale();
  return new Intl.NumberFormat(code === 'xx' ? 'pt-BR' : code, {
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
  });
};

/**
 * Valor grande em unidades da moeda: `R$ 412,3 bi`, `USD 12,9 mi`, `R$ 950 mil` (a escala
 * vem do catálogo: `bi` em pt-BR, `B` em inglês).
 */
export function compactAmount(value: number, currency = 'BRL'): string {
  if (!Number.isFinite(value)) return '—';
  const abs = Math.abs(value);
  const sign = value < 0 ? MINUS : '';
  const prefix = `${currencyPrefix(currency)} `;
  const num = (n: number) => `${sign}${prefix}${nf(1).format(n)}`;
  if (abs >= 1e12) return t('pesquisa-compacto-tri', { valor: num(abs / 1e12) });
  if (abs >= 1e9) return t('pesquisa-compacto-bi', { valor: num(abs / 1e9) });
  if (abs >= 1e6) return t('pesquisa-compacto-mi', { valor: num(abs / 1e6) });
  if (abs >= 1e4) return t('pesquisa-compacto-mil', { valor: num(abs / 1e3) });
  return `${sign}${prefix}${nf(2).format(abs)}`;
}

/** Valor por ação/cota na moeda: `R$ 38,60`; abaixo de 1, até 4 casas (`R$ 0,0952`). */
export function unitAmount(value: number, currency = 'BRL', exact?: string | null): string {
  if (!Number.isFinite(value)) return '—';
  const text = exact ?? value.toFixed(6);
  return formatUnitPrice(text, currency, { maxDecimals: Math.abs(value) < 1 ? 4 : 2 });
}

/** Múltiplo: `5,42×`. */
export function formatMultiple(value: number): string {
  return `${formatNumber(value, 2)}×`;
}

/**
 * Valor de um indicador pela unidade: percentual (fração → `8,3%`), múltiplo (`5,42×`),
 * contagem (`1.234`) ou dinheiro (grande: compacto; por ação: com centavos).
 */
export function formatIndicatorValue(
  value: number | null | undefined,
  unit: IndicatorUnit,
  currency?: string | null,
  exact?: string | null,
): string {
  if (value == null || !Number.isFinite(value)) return '—';
  switch (unit) {
    case 'percent':
      return formatPercent(value, { decimals: 1 });
    case 'multiple':
      return formatMultiple(value);
    case 'count':
      return formatNumber(value, 0);
    case 'money':
      return Math.abs(value) >= 1e6
        ? compactAmount(value, currency ?? 'BRL')
        : unitAmount(value, currency ?? 'BRL', exact);
  }
}

/** Bytes no idioma do app (base 1000, como os tamanhos da documentação): `230,4 MB`. */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  if (bytes < 1000) return `${bytes} B`;
  if (bytes < 1e6) return `${nf(1).format(bytes / 1e3)} kB`;
  if (bytes < 1e9) return `${nf(1).format(bytes / 1e6)} MB`;
  return `${nf(2).format(bytes / 1e9)} GB`;
}

/** Margem até o preço justo, com sinal: `+12,3%` / `−8,0%`. */
export function formatUpside(v: number | null | undefined): string {
  if (v == null || !Number.isFinite(v)) return '—';
  return formatPercent(v, { decimals: 1, signed: true });
}
