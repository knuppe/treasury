/** Mock de `changes.*`: histórico de alterações com desfazer/refazer e conflitos. */
import type { AuthorKind } from '@/api/generated/AuthorKind';

import { registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { str } from './ledger/errors';
import { changeDetail, changeSet, changeSummary, db, redo, undo } from './ledger/store';

registerMockHandlers(
  guarded({
    'changes.list': (args) => {
      const limit = Math.min(Math.max(Number(args.limit ?? 50), 1), 500);
      const offset = Math.max(Number(args.offset ?? 0), 0);
      const kind = args.author_kind as AuthorKind | undefined;
      const table = typeof args.table_name === 'string' ? args.table_name : null;
      const rowId = typeof args.row_id === 'string' ? args.row_id : null;
      return [...db.changeSets]
        .filter((c) => !kind || c.author.kind === kind)
        .filter(
          (c) =>
            !table || c.items.some((it) => it.table === table && (!rowId || it.rowId === rowId)),
        )
        .sort((a, b) => b.created_at.localeCompare(a.created_at) || b.id.localeCompare(a.id))
        .slice(offset, offset + limit)
        .map(changeSummary);
    },
    'changes.get': (args) => changeDetail(changeSet(str(args, 'id'))),
    'changes.undo': (args) => undo(str(args, 'id')),
    'changes.redo': (args) => redo(str(args, 'id')),
  }),
);
