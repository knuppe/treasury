/**
 * Mock de `export.*`: confere o caminho como o núcleo (absoluto, com nome de
 * arquivo, fora das pastas do Treasury) e devolve as contagens. Nada é gravado.
 */
import type { ExportResult } from '@/api/generated/ExportResult';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { filterTxns } from './ledger/domain';
import { invalid, str } from './ledger/errors';
import { db } from './ledger/store';

function checkDestination(path: string): string {
  const p = path.trim();
  if (!p.startsWith('/') && !/^[A-Za-z]:[\\/]/.test(p))
    invalid('escolha um caminho completo para o arquivo');
  if (p.endsWith('/') || p.endsWith('\\'))
    invalid('o destino é uma pasta; informe o nome do arquivo');
  if (/\/\.treasury(\/|$)/.test(p))
    invalid('escolha uma pasta fora das pastas de dados e de backups do Treasury');
  return p;
}

registerMockHandlers(
  guarded({
    'export.json': (args): ExportResult => {
      const path = checkDestination(str(args, 'path'));
      const counts: Record<string, number> = {};
      let bytes = 0;
      for (const [table, map] of Object.entries(db.t)) {
        counts[table] = (map as Map<string, unknown>).size;
        bytes += JSON.stringify([...(map as Map<string, unknown>).values()]).length;
      }
      return { path, bytes, counts };
    },
    'export.transactions_csv': (args): ExportResult => {
      const path = checkDestination(str(args, 'path'));
      const list = filterTxns(args.filter ?? {});
      return { path, bytes: 120 + list.length * 96, counts: { txn: list.length } };
    },
  }),
);
