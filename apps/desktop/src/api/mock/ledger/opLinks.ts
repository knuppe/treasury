/**
 * Lançamento → operação vinculada (espelho de `txn_link`): aporte na carteira, cotas
 * capital ou meta. O mock de `./operations.ts` grava; as visões de lançamento e as cotas
 * capital leem.
 */
import type { TxnKind } from '@/api/generated/TxnKind';
import type { TxnOperationKind } from '@/api/generated/TxnOperationKind';

import { onMockReset } from '../../mockRegistry';

export interface OperationLink {
  kind: TxnOperationKind;
  target_id: string;
  label: string;
  previous_kind: TxnKind;
  previous_category_id: string | null;
}

const links = new Map<string, OperationLink>();
onMockReset(() => links.clear());

export function operationLinks(): Map<string, OperationLink> {
  return links;
}
