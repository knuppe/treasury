/**
 * Detalhar os itens de um lançamento (docs/consumption.md): a Compra abre já ligada à despesa, com
 * a data, a loja (a descrição do lançamento) e o membro dela. Se o lançamento já tem os itens,
 * abre a compra para editar. Usado pelo menu da linha em 04 · Lançamentos e pelo lembrete de
 * Para resolver (`/lancamentos?detalhar=<id>`).
 */
import type { TxnView } from '@/api/generated/TxnView';
import { useReceipt } from '@/api/hooks/receipts';
import { useTxn } from '@/api/hooks/transactions';

import { PurchaseDialog } from './PurchaseDialog';

export function ItemizeTxn({ txn, onClose }: { txn: TxnView; onClose: () => void }) {
  const existing = useReceipt(txn.receipt_id ?? null);
  if (txn.receipt_id) {
    return existing.data ? <PurchaseDialog purchase={existing.data} onClose={onClose} /> : null;
  }
  return (
    <PurchaseDialog
      purchase={null}
      txn={{
        id: txn.id,
        date: txn.date,
        description: txn.payee ?? txn.description,
        amount: txn.amount,
        member_id: txn.member_id,
        account_name: txn.account_name,
      }}
      onClose={onClose}
    />
  );
}

/** O mesmo, a partir do id (link de Para resolver). */
export function ItemizeTxnById({ id, onClose }: { id: string; onClose: () => void }) {
  const txn = useTxn(id);
  return txn.data ? <ItemizeTxn txn={txn.data} onClose={onClose} /> : null;
}
