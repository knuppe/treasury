/**
 * Hooks de `physical.*`: patrimônio físico (imóveis, veículos, terrenos, equipamentos),
 * com benfeitorias, avaliações, financiamentos ligados, venda e a tabela FIPE. Chaves
 * `['physical', acao, args]`.
 */
import type { Deflator } from '../generated/Deflator';
import type { FipeOption } from '../generated/FipeOption';
import type { IptuInput } from '../generated/IptuInput';
import type { IptuPayArgs } from '../generated/IptuPayArgs';
import type { IptuTxnMatch } from '../generated/IptuTxnMatch';
import type { IptuView } from '../generated/IptuView';
import type { FipeOptionsArgs } from '../generated/FipeOptionsArgs';
import type { PhysicalAssetInput } from '../generated/PhysicalAssetInput';
import type { PhysicalAssetPatch } from '../generated/PhysicalAssetPatch';
import type { PhysicalAssetView } from '../generated/PhysicalAssetView';
import type { PhysicalHistoryPoint } from '../generated/PhysicalHistoryPoint';
import type { PhysicalImprovementInput } from '../generated/PhysicalImprovementInput';
import type { PhysicalImprovementPatch } from '../generated/PhysicalImprovementPatch';
import type { PhysicalSaleArgs } from '../generated/PhysicalSaleArgs';
import type { PhysicalSummary } from '../generated/PhysicalSummary';
import type { PhysicalValuationInput } from '../generated/PhysicalValuationInput';
import { useCore, useCoreMutation } from './coreQuery';

export interface PhysicalRead {
  as_of?: string;
  deflator?: Deflator;
  include_archived?: boolean;
}

export const usePhysicalSummary = (args: PhysicalRead = {}) =>
  useCore<PhysicalSummary>('physical', 'summary', args);

export const usePhysicalAssets = (args: PhysicalRead = {}) =>
  useCore<PhysicalAssetView[]>('physical', 'list', args);

export const usePhysicalAsset = (id: string | null | undefined, args: PhysicalRead = {}) =>
  useCore<PhysicalAssetView>('physical', 'get', { id, ...args }, { enabled: Boolean(id) });

export const usePhysicalHistory = (id: string | null | undefined) =>
  useCore<PhysicalHistoryPoint[]>('physical', 'history', { id }, { enabled: Boolean(id) });

/** Marcas, modelos ou anos da tabela FIPE (rede, só quando o formulário pede). */
export const useFipeOptions = (args: FipeOptionsArgs | null) =>
  useCore<FipeOption[]>('physical', 'fipe_options', args ?? {}, {
    enabled: Boolean(args),
    staleTime: 24 * 60 * 60 * 1000,
    retry: false,
  });

const AFFECTS = { affects: ['physical'] } as const;

export const useCreatePhysical = () =>
  useCoreMutation<PhysicalAssetInput, PhysicalAssetView>('physical.create', AFFECTS);
export const useUpdatePhysical = () =>
  useCoreMutation<PhysicalAssetPatch, PhysicalAssetView>('physical.update', AFFECTS);
export const useDeletePhysical = () =>
  useCoreMutation<{ id: string }, null>('physical.delete', AFFECTS);
export const useAddImprovement = () =>
  useCoreMutation<PhysicalImprovementInput, PhysicalAssetView>('physical.add_improvement', AFFECTS);
export const useUpdateImprovement = () =>
  useCoreMutation<PhysicalImprovementPatch, PhysicalAssetView>(
    'physical.update_improvement',
    AFFECTS,
  );
export const useDeleteImprovement = () =>
  useCoreMutation<{ id: string }, PhysicalAssetView>('physical.delete_improvement', AFFECTS);
export const useAddValuation = () =>
  useCoreMutation<PhysicalValuationInput, PhysicalAssetView>('physical.add_valuation', AFFECTS);
export const useDeleteValuation = () =>
  useCoreMutation<{ id: string }, PhysicalAssetView>('physical.delete_valuation', AFFECTS);
export const useLinkLoan = () =>
  useCoreMutation<{ asset_id: string; loan_id: string }, PhysicalAssetView>(
    'physical.link_loan',
    AFFECTS,
  );
export const useUnlinkLoan = () =>
  useCoreMutation<{ asset_id: string; loan_id: string }, PhysicalAssetView>(
    'physical.unlink_loan',
    AFFECTS,
  );
export const useSellPhysical = () =>
  useCoreMutation<PhysicalSaleArgs, PhysicalAssetView>('physical.sell', AFFECTS);
export const useRefreshFipe = () =>
  useCoreMutation<{ id: string }, PhysicalAssetView>('physical.refresh_fipe', AFFECTS);

/** Carnês de IPTU de um imóvel (Brasil), do ano mais novo para o mais antigo. */
export const useIptu = (assetId: string | null | undefined) =>
  useCore<IptuView[]>(
    'physical',
    'iptu_list',
    { asset_id: assetId },
    { enabled: Boolean(assetId) },
  );
export const useSaveIptu = () =>
  useCoreMutation<IptuInput, IptuView>('physical.iptu_save', AFFECTS);
export const useDeleteIptu = () =>
  useCoreMutation<{ id: string }, null>('physical.iptu_delete', AFFECTS);
export const usePayIptu = () =>
  useCoreMutation<IptuPayArgs, IptuView>('physical.iptu_pay', AFFECTS);
export const useUnpayIptu = () =>
  useCoreMutation<{ installment_id: string }, IptuView>('physical.iptu_unpay', AFFECTS);
/** Lançamentos que parecem pagar a parcela do IPTU. */
export const useIptuMatches = (installmentId: string | null) =>
  useCore<IptuTxnMatch[]>(
    'physical',
    'iptu_matches',
    { installment_id: installmentId },
    { enabled: Boolean(installmentId) },
  );
