/** Para que serve cada série no app (linha de apoio na tabela de índices), no idioma atual. */
import { t } from '@/i18n';

const SERIES_USE = {
  ipca: 'indices-uso-ipca',
  igpm: 'indices-uso-igpm',
  cdi: 'indices-uso-cdi',
  selic: 'indices-uso-selic',
  selic_meta: 'indices-uso-selic-meta',
  tr: 'indices-uso-tr',
  m2_br: 'indices-uso-m2-br',
  us_cpi: 'indices-uso-us-cpi',
  us_m2: 'indices-uso-us-m2',
  fed_funds: 'indices-uso-fed-funds',
  ibov: 'indices-uso-ibov',
  sp500: 'indices-uso-sp500',
  estr: 'indices-uso-estr',
  corra: 'indices-uso-corra',
  sonia: 'indices-uso-sonia',
  boe_bank_rate: 'indices-uso-boe-bank-rate',
  hicp_ea: 'indices-uso-hicp-ea',
  cpi_ca: 'indices-uso-cpi-ca',
  cpi_gb: 'indices-uso-cpi-gb',
  cpih_gb: 'indices-uso-cpih-gb',
  cpi_us_oecd: 'indices-uso-cpi-us-oecd',
} as const;

/** Linha de apoio da série (genérica para as do catálogo de países). */
export function seriesUse(id: string): string {
  if (id in SERIES_USE) return t(SERIES_USE[id as keyof typeof SERIES_USE]);
  if (id.startsWith('hicp_') || id.startsWith('cpi_')) return t('indices-uso-inflacao-pais');
  if (id.startsWith('wb_cpi_')) return t('indices-uso-wb-cpi');
  if (id.startsWith('rate3m_')) return t('indices-uso-rate3m');
  if (id.startsWith('fx_')) return t('indices-uso-fx');
  return id;
}
