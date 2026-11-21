/** Hooks de `import.*`: detectar, prévia, gravar e lotes importados. */
import type { CsvPreset } from '../generated/CsvPreset';
import type { DetectArgs } from '../generated/DetectArgs';
import type { DetectResult } from '../generated/DetectResult';
import type { ImportBatch } from '../generated/ImportBatch';
import type { ImportCommitArgs } from '../generated/ImportCommitArgs';
import type { ImportPreview } from '../generated/ImportPreview';
import type { ImportPreviewArgs } from '../generated/ImportPreviewArgs';
import type { ImportResult } from '../generated/ImportResult';
import type { ListBatchesArgs } from '../generated/ListBatchesArgs';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const usePresets = () =>
  useCore<CsvPreset[]>('import', 'list_presets', {}, { staleTime: Infinity });

export const useImportBatches = (args: ListBatchesArgs = {}) =>
  useCore<ImportBatch[]>('import', 'list_batches', args);

/** Etapas do assistente: chamadas diretas (o arquivo não vai para o cache). */
export const detectImport = (args: DetectArgs) => call<DetectResult>('import.detect', args);

export const previewImport = (args: ImportPreviewArgs) =>
  call<ImportPreview>('import.preview', args);

export const useCommitImport = () =>
  useCoreMutation<ImportCommitArgs, ImportResult>('import.commit', { affects: ['ledger'] });
