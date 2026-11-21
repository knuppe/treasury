/** Rótulos (pelo catálogo `bens`) dos enums do patrimônio físico (`physical.*`). */
import type { AcquisitionCostKind } from '@/api/generated/AcquisitionCostKind';
import type { FipeVehicleType } from '@/api/generated/FipeVehicleType';
import type { ImprovementCategory } from '@/api/generated/ImprovementCategory';
import type { PhysicalIndex } from '@/api/generated/PhysicalIndex';
import type { PhysicalKind } from '@/api/generated/PhysicalKind';
import type { PhysicalValuationMethod } from '@/api/generated/PhysicalValuationMethod';
import type { PhysicalValuationSource } from '@/api/generated/PhysicalValuationSource';
import type { ValueBasis } from '@/api/generated/ValueBasis';
import { t } from '@/i18n';

export const KIND_LABEL: Record<PhysicalKind, string> = {
  get real_estate() {
    return t('bens-tipo-imovel');
  },
  get vehicle() {
    return t('bens-tipo-veiculo');
  },
  get land() {
    return t('bens-tipo-terreno');
  },
  get equipment() {
    return t('bens-tipo-equipamento');
  },
  get collectible() {
    return t('bens-tipo-colecao');
  },
  get other() {
    return t('bens-tipo-outro');
  },
};

export const METHOD_LABEL: Record<PhysicalValuationMethod, string> = {
  get none() {
    return t('bens-metodo-nenhum');
  },
  get index() {
    return t('bens-metodo-indice');
  },
  get declining() {
    return t('bens-metodo-declinante');
  },
  get straight_line() {
    return t('bens-metodo-linear');
  },
  get fipe() {
    return t('bens-metodo-fipe');
  },
};

export const METHOD_HINT: Record<PhysicalValuationMethod, string> = {
  get none() {
    return t('bens-metodo-nenhum-dica');
  },
  get index() {
    return t('bens-metodo-indice-dica');
  },
  get declining() {
    return t('bens-metodo-declinante-dica');
  },
  get straight_line() {
    return t('bens-metodo-linear-dica');
  },
  get fipe() {
    return t('bens-metodo-fipe-dica');
  },
};

export const INDEX_LABEL: Record<PhysicalIndex, string> = {
  ipca: 'IPCA',
  igpm: 'IGP-M',
  incc: 'INCC',
};

export const INDEX_HINT: Record<PhysicalIndex, string> = {
  get ipca() {
    return t('bens-indice-ipca-dica');
  },
  get igpm() {
    return t('bens-indice-igpm-dica');
  },
  get incc() {
    return t('bens-indice-incc-dica');
  },
};

export const CATEGORY_LABEL: Record<ImprovementCategory, string> = {
  get renovation() {
    return t('bens-categoria-reforma');
  },
  get expansion() {
    return t('bens-categoria-ampliacao');
  },
  get improvement() {
    return t('bens-categoria-benfeitoria');
  },
  get maintenance() {
    return t('bens-categoria-manutencao');
  },
};

export const COST_KIND_LABEL: Record<AcquisitionCostKind, string> = {
  itbi: 'ITBI',
  get deed() {
    return t('bens-custo-escritura');
  },
  get registry() {
    return t('bens-custo-registro');
  },
  get brokerage() {
    return t('bens-custo-corretagem');
  },
  get other() {
    return t('bens-custo-outro');
  },
};

export const SOURCE_LABEL: Record<PhysicalValuationSource, string> = {
  get manual() {
    return t('bens-fonte-manual');
  },
  get appraisal() {
    return t('bens-fonte-avaliacao');
  },
  get report() {
    return t('bens-fonte-laudo');
  },
  get fipe() {
    return t('bens-metodo-fipe');
  },
  get index() {
    return t('bens-fonte-indice');
  },
};

export const BASIS_LABEL: Record<ValueBasis, string> = {
  get appraisal() {
    return t('bens-base-avaliacao');
  },
  get automatic() {
    return t('bens-base-automatica');
  },
  get cost() {
    return t('bens-base-custo');
  },
  get sold() {
    return t('bens-base-vendido');
  },
  get not_owned() {
    return t('bens-base-nao-comprado');
  },
};

export const FIPE_TYPE_LABEL: Record<FipeVehicleType, string> = {
  get carros() {
    return t('bens-fipe-carro');
  },
  get motos() {
    return t('bens-fipe-moto');
  },
  get caminhoes() {
    return t('bens-fipe-caminhao');
  },
};

export const KINDS = Object.keys(KIND_LABEL) as PhysicalKind[];
export const METHODS = Object.keys(METHOD_LABEL) as PhysicalValuationMethod[];
export const INDEXES = Object.keys(INDEX_LABEL) as PhysicalIndex[];
export const CATEGORIES = Object.keys(CATEGORY_LABEL) as ImprovementCategory[];
export const COST_KINDS = Object.keys(COST_KIND_LABEL) as AcquisitionCostKind[];
export const SOURCES = Object.keys(SOURCE_LABEL) as PhysicalValuationSource[];
