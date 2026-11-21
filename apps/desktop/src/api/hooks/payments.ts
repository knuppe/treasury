/**
 * Hooks de `payments.*` (docs/ocr-other-documents.md): boleto, Pix "copia e cola" e comprovante
 * lidos no computador, a conta a pagar prevista e o comprovante ligado ao lançamento.
 */
import type { PaymentReadArgs } from '../generated/PaymentReadArgs';
import type { PaymentReading } from '../generated/PaymentReading';
import type { PlanArgs } from '../generated/PlanArgs';
import type { SaveProofArgs } from '../generated/SaveProofArgs';
import { useCoreMutation } from './coreQuery';

/** Lê (não grava): o OCR roda no computador e o núcleo procura os lançamentos que combinam. */
export const useReadPayment = () =>
  useCoreMutation<PaymentReadArgs & { filename?: string }, PaymentReading>('payments.read', {
    affects: [],
  });

/** Conta a pagar prevista (uma vez só por boleto ou Pix): entra nas contas a vencer. */
export const usePlanBill = () =>
  useCoreMutation<PlanArgs, string>('payments.plan', {
    affects: ['ledger'],
  });

/** Liga o comprovante ao lançamento (ou cria um), com os dados lidos na nota. */
export const useSaveProof = () =>
  useCoreMutation<SaveProofArgs, string>('payments.save_proof', {
    affects: ['ledger'],
  });
