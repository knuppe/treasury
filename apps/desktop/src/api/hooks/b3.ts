/**
 * Hooks de `b3.*` (importação da Área do Investidor B3): detectar, prévia e gravar.
 * Detectar e prévia são chamadas diretas: os arquivos não vão para o cache.
 */
import type { B3CommitArgs } from '../generated/B3CommitArgs';
import type { B3CommitResult } from '../generated/B3CommitResult';
import type { B3DetectResult } from '../generated/B3DetectResult';
import type { B3File } from '../generated/B3File';
import type { B3Preview } from '../generated/B3Preview';
import type { B3PreviewArgs } from '../generated/B3PreviewArgs';
import { call } from '../client';
import { useCoreMutation } from './coreQuery';

export const detectB3 = (args: B3File) => call<B3DetectResult>('b3.detect', args);

export const previewB3 = (args: B3PreviewArgs) => call<B3Preview>('b3.preview', args);

/** Grava operações, proventos, eventos e ativos (e o caixa, se pedido). */
export const useCommitB3 = () =>
  useCoreMutation<B3CommitArgs, B3CommitResult>('b3.commit', { affects: ['ledger', 'market'] });
