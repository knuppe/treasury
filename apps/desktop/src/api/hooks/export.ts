/** Hooks de `export.*`: exportação sem criptografia para um caminho escolhido. */
import type { ExportArgs } from '../generated/ExportArgs';
import type { ExportCsvArgs } from '../generated/ExportCsvArgs';
import type { ExportResult } from '../generated/ExportResult';
import { useCoreMutation } from './coreQuery';

export const useExportJson = () =>
  useCoreMutation<ExportArgs, ExportResult>('export.json', { affects: [] });

export const useExportCsv = () =>
  useCoreMutation<ExportCsvArgs, ExportResult>('export.transactions_csv', { affects: [] });
