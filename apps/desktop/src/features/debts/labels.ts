/**
 * Rótulos dos enums de dívidas (`debt.*`), no idioma do app (catálogo `dividas.ftl`). Cada
 * tabela lê o catálogo na hora (getter), então pode ser importada e guardada em constante sem
 * congelar o idioma. Siglas de sistema e de índice (SAC, Price, TR, IPCA…) ficam como estão.
 */
import type { AmortSystem } from '@/api/generated/AmortSystem';
import type { CorrectionReference } from '@/api/generated/CorrectionReference';
import type { InterestBasis } from '@/api/generated/InterestBasis';
import type { LoanIndexer } from '@/api/generated/LoanIndexer';
import type { LoanKind } from '@/api/generated/LoanKind';
import type { PrepayMode } from '@/api/generated/PrepayMode';
import type { Prorata } from '@/api/generated/Prorata';
import type { SpreadMode } from '@/api/generated/SpreadMode';
import { t } from '@/i18n';
import type { MessageKey, Messages } from '@/i18n/keys';

/** Chaves sem argumentos (as dos rótulos). */
type PlainKey = { [K in MessageKey]: [Messages[K]] extends [never] ? K : never }[MessageKey];

/** Tabela de rótulos: chave do catálogo (lida a cada acesso) ou texto fixo (siglas). */
function labels<V extends string>(
  entries: Record<V, PlainKey | { fixed: string }>,
): Record<V, string> {
  const out = {} as Record<V, string>;
  for (const [value, entry] of Object.entries(entries) as [V, PlainKey | { fixed: string }][]) {
    Object.defineProperty(out, value, {
      enumerable: true,
      get: () => (typeof entry === 'string' ? t(entry) : entry.fixed),
    });
  }
  return out;
}

export const KIND_LABEL: Record<LoanKind, string> = labels({
  mortgage: 'dividas-tipo-imobiliario',
  vehicle: 'dividas-tipo-veiculo',
  personal: 'dividas-tipo-pessoal',
  payroll: 'dividas-tipo-consignado',
  student: 'dividas-tipo-estudantil',
  other: 'dividas-tipo-outro',
});

export const SYSTEM_LABEL: Record<AmortSystem, string> = {
  sac: 'SAC',
  price: 'Price (TP)',
  bullet: 'Bullet',
};

export const SYSTEM_HINT: Record<AmortSystem, string> = labels({
  sac: 'dividas-sistema-sac-dica',
  price: 'dividas-sistema-price-dica',
  bullet: 'dividas-sistema-bullet-dica',
});

/** Pós-fixados: na Price, a parcela é recalculada a cada período pelo fator do índice. */
export const SYSTEM_HINT_POST: Record<AmortSystem, string> = labels({
  sac: 'dividas-sistema-sac-dica-pos',
  price: 'dividas-sistema-price-dica-pos',
  bullet: 'dividas-sistema-bullet-dica-pos',
});

export const INDEXER_LABEL: Record<LoanIndexer, string> = labels({
  none: 'dividas-indexador-nenhum',
  tr: { fixed: 'TR' },
  ipca: { fixed: 'IPCA' },
  igpm: { fixed: 'IGP-M' },
  cdi: { fixed: 'CDI' },
  selic: { fixed: 'Selic' },
  inpc: { fixed: 'INPC' },
  incc: { fixed: 'INCC' },
  poupanca: 'dividas-indexador-poupanca',
});

/** Índices de taxa diária (pós-fixados): juros do período pelo índice acumulado. */
export const POST_INDEXERS = ['cdi', 'selic'] as const satisfies readonly LoanIndexer[];
export const isPostIndexer = (i: LoanIndexer) => (POST_INDEXERS as readonly string[]).includes(i);

/** Correção monetária do saldo (pré + índice): TR, IPCA, IGP-M, INPC, INCC, poupança. */
export const CORRECTION_INDEXERS = [
  'none',
  'tr',
  'ipca',
  'igpm',
  'inpc',
  'incc',
  'poupanca',
] as const satisfies readonly LoanIndexer[];

export const PRORATA_LABEL: Record<Prorata, string> = labels({
  none: 'dividas-prorata-nenhum',
  calendar: 'dividas-prorata-corridos',
  business: 'dividas-prorata-uteis',
});

export const BASIS_LABEL: Record<InterestBasis, string> = labels({
  monthly: 'dividas-base-mensal',
  bd252: 'dividas-base-252',
  dc365: 'dividas-base-365',
});

export const REFERENCE_LABEL: Record<CorrectionReference, string> = labels({
  month: 'dividas-referencia-mes',
  anniversary: 'dividas-referencia-aniversario',
});

export const SPREAD_MODE_LABEL: Record<SpreadMode, string> = labels({
  exp252: 'dividas-spread-exponencial',
  lin360: 'dividas-spread-linear',
});

export const MODE_LABEL: Record<PrepayMode, string> = labels({
  reduce_term: 'dividas-modo-prazo',
  reduce_installment: 'dividas-modo-parcela',
});

export const MODE_HINT: Record<PrepayMode, string> = labels({
  reduce_term: 'dividas-modo-prazo-dica',
  reduce_installment: 'dividas-modo-parcela-dica',
});

export const KINDS = Object.keys(KIND_LABEL) as LoanKind[];
export const INDEXERS: readonly LoanIndexer[] = CORRECTION_INDEXERS;
