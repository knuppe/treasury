/**
 * Rótulos da análise de ativos (catálogo `pesquisa.ftl`, por getters: o app remonta ao trocar
 * o idioma) (`research.*`): grupos de indicadores, operadores do
 * screener, fontes e situação da coleta, e as classes que têm fundamentos.
 */
import type { AssetClass } from '@/api/generated/AssetClass';
import type { CmpOp } from '@/api/generated/CmpOp';
import type { IndicatorGroup } from '@/api/generated/IndicatorGroup';
import type { Logic } from '@/api/generated/Logic';
import type { SyncSource } from '@/api/generated/SyncSource';
import type { SyncStatus } from '@/api/generated/SyncStatus';
import { t } from '@/i18n';

export const GROUP_LABEL: Record<IndicatorGroup, string> = {
  get market() {
    return t('pesquisa-grupo-mercado');
  },
  get valuation() {
    return t('pesquisa-grupo-valuation');
  },
  get profitability() {
    return t('pesquisa-grupo-rentabilidade');
  },
  get debt() {
    return t('pesquisa-grupo-endividamento');
  },
  get growth() {
    return t('pesquisa-grupo-crescimento');
  },
  get per_share() {
    return t('pesquisa-grupo-por-acao');
  },
  get dividends() {
    return t('pesquisa-grupo-dividendos');
  },
  get fair_price() {
    return t('pesquisa-grupo-preco-justo');
  },
  get fii() {
    return t('pesquisa-grupo-fii');
  },
  get bank() {
    return t('pesquisa-grupo-bancos');
  },
  get score() {
    return t('pesquisa-grupo-score');
  },
};

/** Ordem dos grupos no painel de indicadores (preço justo tem aba própria). */
export const PANEL_GROUPS: readonly IndicatorGroup[] = [
  'market',
  'valuation',
  'profitability',
  'debt',
  'growth',
  'per_share',
  'dividends',
  'fii',
  'bank',
];

/** Classes com fundamentos (as do universo do núcleo). */
export const FUNDAMENTAL_CLASSES: readonly AssetClass[] = [
  'stock_br',
  'fii',
  'bdr',
  'stock_us',
  'reit',
];

export function hasFundamentals(cls: AssetClass): boolean {
  return FUNDAMENTAL_CLASSES.includes(cls);
}

export const LOGIC_LABEL: Record<Logic, string> = {
  get and() {
    return t('pesquisa-logica-e');
  },
  get or() {
    return t('pesquisa-logica-ou');
  },
};

export const OP_LABEL: Record<CmpOp, string> = {
  gt: '>',
  gte: '≥',
  lt: '<',
  lte: '≤',
  eq: '=',
  ne: '≠',
  get between() {
    return t('pesquisa-op-entre');
  },
  get in() {
    return t('pesquisa-op-um-de');
  },
  get not_in() {
    return t('pesquisa-op-nao-um-de');
  },
  get contains() {
    return t('pesquisa-op-contem');
  },
  get exists() {
    return t('pesquisa-op-disponivel');
  },
  get missing() {
    return t('pesquisa-op-indisponivel');
  },
};

export const SYNC_SOURCE_LABEL: Record<SyncSource, string> = {
  get fca() {
    return t('pesquisa-fonte-fca');
  },
  get dfp() {
    return t('pesquisa-fonte-dfp');
  },
  get itr() {
    return t('pesquisa-fonte-itr');
  },
  get fii() {
    return t('pesquisa-fonte-fii');
  },
  get fii_quarterly() {
    return t('pesquisa-fonte-fii-trimestral');
  },
  get ipe() {
    return t('pesquisa-fonte-ipe');
  },
  get b3() {
    return t('pesquisa-fonte-b3');
  },
  get b3_listados() {
    return t('pesquisa-fonte-b3-listados');
  },
  get sec() {
    return t('pesquisa-fonte-sec');
  },
  get bacen_ifdata() {
    return t('pesquisa-fonte-bacen');
  },
  get funds() {
    return t('pesquisa-fonte-fundos');
  },
  get crypto() {
    return t('pesquisa-fonte-cripto');
  },
  get brapi() {
    return t('pesquisa-fonte-brapi');
  },
};

export const SYNC_SOURCE_ORDER: readonly SyncSource[] = [
  'fca',
  'dfp',
  'itr',
  'fii',
  'fii_quarterly',
  'ipe',
  'b3',
  'b3_listados',
  'sec',
  'bacen_ifdata',
  'funds',
  'crypto',
  'brapi',
];

export const SYNC_STATUS_LABEL: Record<SyncStatus, string> = {
  get updated() {
    return t('pesquisa-situacao-atualizado');
  },
  get unchanged() {
    return t('pesquisa-situacao-sem-mudanca');
  },
  get not_found() {
    return t('pesquisa-situacao-inexistente');
  },
  get skipped() {
    return t('pesquisa-situacao-pulado');
  },
  get error() {
    return t('pesquisa-situacao-erro');
  },
};

/** Campos de texto e booleanos do screener (além dos indicadores do catálogo). */
export const TEXT_FIELDS = [
  {
    key: 'ticker',
    get label() {
      return t('pesquisa-campo-ticker');
    },
  },
  {
    key: 'name',
    get label() {
      return t('pesquisa-campo-nome');
    },
  },
  {
    key: 'class',
    get label() {
      return t('pesquisa-campo-classe');
    },
  },
  {
    key: 'sector',
    get label() {
      return t('pesquisa-campo-setor');
    },
  },
  {
    key: 'segment',
    get label() {
      return t('pesquisa-campo-segmento');
    },
  },
  {
    key: 'currency',
    get label() {
      return t('pesquisa-campo-moeda');
    },
  },
  {
    key: 'score_band',
    get label() {
      return t('pesquisa-campo-faixa-score');
    },
  },
  {
    key: 'red_flags',
    get label() {
      return t('pesquisa-campo-red-flags');
    },
  },
] as const;

export const BOOL_FIELDS = [
  {
    key: 'held',
    get label() {
      return t('pesquisa-campo-na-carteira');
    },
  },
  {
    key: 'watched',
    get label() {
      return t('pesquisa-campo-na-watchlist');
    },
  },
] as const;
