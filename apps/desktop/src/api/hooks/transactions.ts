/** Hooks de `transactions.*`: consulta paginada, escrita, lote, tags e anexos. */
import { keepPreviousData } from '@tanstack/react-query';

import type { CrossAccountDuplicate } from '../generated/CrossAccountDuplicate';
import type { RemoveCopiesResult } from '../generated/RemoveCopiesResult';
import type { AttachmentContent } from '../generated/AttachmentContent';
import type { AttachmentInput } from '../generated/AttachmentInput';
import type { AttachmentMeta } from '../generated/AttachmentMeta';
import type { BulkResult } from '../generated/BulkResult';
import type { EditTagsArgs } from '../generated/EditTagsArgs';
import type { SameDescription } from '../generated/SameDescription';
import type { SetCategoryArgs } from '../generated/SetCategoryArgs';
import type { SetMemberArgs } from '../generated/SetMemberArgs';
import type { SetStatusArgs } from '../generated/SetStatusArgs';
import type { Tag } from '../generated/Tag';
import type { TagSummary } from '../generated/TagSummary';
import type { LinkOperationArgs } from '../generated/LinkOperationArgs';
import type { LinkOperationResult } from '../generated/LinkOperationResult';
import type { LinkOperationsArgs } from '../generated/LinkOperationsArgs';
import type { LinkOperationsResult } from '../generated/LinkOperationsResult';
import type { LinkTransfersArgs } from '../generated/LinkTransfersArgs';
import type { OperationLinkPreview } from '../generated/OperationLinkPreview';
import type { OperationCandidate } from '../generated/OperationCandidate';
import type { LinkTransfersResult } from '../generated/LinkTransfersResult';
import type { UnlinkTransferArgs } from '../generated/UnlinkTransferArgs';
import type { UnlinkTransferResult } from '../generated/UnlinkTransferResult';
import type { TransferCandidate } from '../generated/TransferCandidate';
import type { TransferCandidatesArgs } from '../generated/TransferCandidatesArgs';
import type { TransferInput } from '../generated/TransferInput';
import type { TransferPair } from '../generated/TransferPair';
import type { TransferResult } from '../generated/TransferResult';
import type { TxnFilter } from '../generated/TxnFilter';
import type { TxnInput } from '../generated/TxnInput';
import type { TxnPage } from '../generated/TxnPage';
import type { TxnPatch } from '../generated/TxnPatch';
import type { TxnView } from '../generated/TxnView';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

/** Página de lançamentos; mantém a anterior na tela enquanto a nova chega. */
export const useTxnQuery = (filter: TxnFilter, enabled = true) =>
  useCore<TxnPage>('transactions', 'query', filter, { placeholderData: keepPreviousData, enabled });

export const useTxn = (id: string | null | undefined) =>
  useCore<TxnView>('transactions', 'get', { id }, { enabled: Boolean(id) });

export const useTags = () => useCore<TagSummary[]>('transactions', 'list_tags');

export const useAttachments = (txnId: string | null | undefined) =>
  useCore<AttachmentMeta[]>(
    'transactions',
    'list_attachments',
    { txn_id: txnId },
    { enabled: Boolean(txnId) },
  );

export const useCreateTxn = () =>
  useCoreMutation<TxnInput, TxnView>('transactions.create', { affects: ['ledger'] });

export const useUpdateTxn = () =>
  useCoreMutation<TxnPatch, TxnView>('transactions.update', { affects: ['ledger'] });

export const useDeleteTxns = () =>
  useCoreMutation<{ ids: string[] }, BulkResult>('transactions.delete', { affects: ['ledger'] });

export const useCreateTransfer = () =>
  useCoreMutation<TransferInput, TransferResult>('transactions.create_transfer', {
    affects: ['ledger'],
  });

export const useSetCategory = () =>
  useCoreMutation<SetCategoryArgs, BulkResult>('transactions.set_category', {
    affects: ['ledger'],
  });

export const useSetStatus = () =>
  useCoreMutation<SetStatusArgs, BulkResult>('transactions.set_status', { affects: ['ledger'] });

export const useSetMember = () =>
  useCoreMutation<SetMemberArgs, BulkResult>('transactions.set_member', { affects: ['ledger'] });

/** Põe e tira tags (por nome) de vários lançamentos. */
export const useEditTags = () =>
  useCoreMutation<EditTagsArgs, BulkResult>('transactions.edit_tags', { affects: ['ledger'] });

/** Os escolhidos e os lançamentos com a mesma descrição dentro do filtro. */
export const sameDescription = (ids: readonly string[], filter: TxnFilter = {}) =>
  call<SameDescription>('transactions.same_description', { ids, filter });

/** Ids de todos os lançamentos do filtro (sem paginação). */
export const queryTxnIds = (filter: TxnFilter) => call<string[]>('transactions.query_ids', filter);

export const useCreateTag = () =>
  useCoreMutation<{ name: string; color?: string }, Tag>('transactions.create_tag', {
    affects: ['ledger'],
  });

export const useAddAttachment = () =>
  useCoreMutation<AttachmentInput, AttachmentMeta>('transactions.add_attachment', {
    affects: ['ledger'],
  });

export const useDeleteAttachment = () =>
  useCoreMutation<{ id: string }, null>('transactions.delete_attachment', { affects: ['ledger'] });

/** Grava o anexo no caminho escolhido no diálogo de salvar (núcleo; só a interface). */
export const saveAttachment = (id: string, path: string) =>
  call<{ path: string; bytes: number }>('transactions.save_attachment', { id, path });

/** Conteúdo de um anexo (sob demanda, sem cache: pode ser grande). */
export const fetchAttachment = (id: string) =>
  call<AttachmentContent>('transactions.get_attachment', { id });

/** Pares soltos que parecem transferência entre contas próprias. */
export const useTransferCandidates = (args: TransferCandidatesArgs = {}, enabled = true) =>
  useCore<TransferCandidate[]>('transactions', 'transfer_candidates', args, { enabled });

/** Lançamentos repetidos em outra conta (mesmo identificador do banco). */
export const useCrossAccountDuplicates = (enabled = true) =>
  useCore<CrossAccountDuplicate[]>('transactions', 'cross_account_duplicates', {}, { enabled });

/** Apaga as cópias confirmadas (só as que são de fato cópias e não pernas de transferência). */
export const useRemoveCopies = () =>
  useCoreMutation<{ ids: string[] }, RemoveCopiesResult>('transactions.remove_copies', {
    affects: ['ledger'],
  });

export const useLinkTransfers = () =>
  useCoreMutation<LinkTransfersArgs, LinkTransfersResult>('transactions.link_transfers', {
    affects: ['ledger'],
  });

/** Desfaz a ligação de uma transferência: as duas pernas ficam soltas (e o par não volta como sugestão). */
export const useUnlinkTransfer = () =>
  useCoreMutation<UnlinkTransferArgs, UnlinkTransferResult>('transactions.unlink_transfer', {
    affects: ['ledger'],
  });

export const useDismissTransfer = () =>
  useCoreMutation<TransferPair, null>('transactions.dismiss_transfer', { affects: ['ledger'] });

/** Operações que o lançamento pode ter pago (carteira, cotas capital, metas). */
export const useOperationCandidates = (txnId: string | null) =>
  useCore<OperationCandidate[]>(
    'transactions',
    'operation_candidates',
    { txn_id: txnId },
    { enabled: Boolean(txnId) },
  );

/** Operações para vários lançamentos (as que combinam com mais deles primeiro). */
export const useOperationCandidatesMany = (txnIds: readonly string[] | null) =>
  useCore<OperationCandidate[]>(
    'transactions',
    'operation_candidates',
    { txn_ids: txnIds ?? [] },
    { enabled: Boolean(txnIds && txnIds.length > 0) },
  );

/** Prévia do vínculo em lote: quais entram, quais ficam de fora e o total. */
export const useOperationLinkPreview = (args: LinkOperationsArgs | null) =>
  useCore<OperationLinkPreview>('transactions', 'operation_link_preview', args ?? {}, {
    enabled: args != null,
  });

/** Vincula vários lançamentos à mesma operação (pula os que não servem). */
export const useLinkOperations = () =>
  useCoreMutation<LinkOperationsArgs, LinkOperationsResult>('transactions.link_operations', {
    affects: ['ledger'],
  });

/** Vincula o lançamento à operação: vira aporte, sem categoria. */
export const useLinkOperation = () =>
  useCoreMutation<LinkOperationArgs, LinkOperationResult>('transactions.link_operation', {
    affects: ['ledger'],
  });

/** Desfaz o vínculo: o lançamento volta ao tipo e à categoria de antes. */
export const useUnlinkOperation = () =>
  useCoreMutation<{ txn_id: string }, { change_set_id: string | null }>(
    'transactions.unlink_operation',
    { affects: ['ledger'] },
  );
