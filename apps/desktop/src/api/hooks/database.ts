/** Hooks de `database.*`: tamanho, maiores tabelas, manutenção e expurgo do banco. */
import type { DatabaseInfo } from '../generated/DatabaseInfo';
import type { MaintenanceArgs } from '../generated/MaintenanceArgs';
import type { MaintenanceResult } from '../generated/MaintenanceResult';
import type { PurgeArgs } from '../generated/PurgeArgs';
import type { PurgePreview } from '../generated/PurgePreview';
import type { PurgeResult } from '../generated/PurgeResult';
import type { TableRows } from '../generated/TableRows';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useDatabaseInfo = () => useCore<DatabaseInfo>('database', 'info');

export const purgePreview = (args: PurgeArgs) => call<PurgePreview>('database.purge_preview', args);

export const usePurge = () =>
  useCoreMutation<PurgeArgs, PurgeResult>('database.purge', { affects: 'all', sensitive: true });

export const useMaintenance = () =>
  useCoreMutation<MaintenanceArgs, MaintenanceResult>('database.run', {
    affects: ['database'],
  });

/** As 100 primeiras linhas de uma tabela, só leitura (segredos ocultos). */
export const useTableRows = (table: string | null) =>
  useCore<TableRows>('database', 'table_rows', { table }, { enabled: table != null });
