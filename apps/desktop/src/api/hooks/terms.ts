/**
 * Termos de uso (docs/terms.md): a versão vigente e o aceite desta instalação, guardado fora do
 * cofre (a tela aparece antes de existir cofre). Nada é enviado.
 */
import type { TermsStatus } from '../generated/TermsStatus';
import { useCore, useCoreMutation } from './coreQuery';

export const useTermsStatus = () =>
  useCore<TermsStatus>('terms', 'status', undefined, { staleTime: Infinity });

export const useAcceptTerms = () =>
  useCoreMutation<{ version: string }, TermsStatus>('terms.accept', { affects: ['terms'] });
