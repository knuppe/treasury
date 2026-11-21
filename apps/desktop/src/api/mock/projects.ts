/** Mock de `projects.*` sobre o livro-caixa fictício (`./ledger/projectsState.ts`). */
import type { BulkResult } from '@/api/generated/BulkResult';
import type { ProjectInput } from '@/api/generated/ProjectInput';
import type { ProjectItemInput } from '@/api/generated/ProjectItemInput';
import type { ProjectItemPatch } from '@/api/generated/ProjectItemPatch';
import type { ProjectPatch } from '@/api/generated/ProjectPatch';

import { optStr, register, str } from './ledger/common';
import {
  addItem,
  createProject,
  deleteProject,
  itemEntry,
  itemRow,
  linkProject,
  listProjects,
  projectEntries,
  projectRow,
  projectView,
  removeEntries,
  updateItem,
  updateProject,
} from './ledger/projectsState';
import { write } from './ledger/store';

register({
  'projects.list': (a) => listProjects(a.include_done !== false, optStr(a, 'physical_asset_id')),
  'projects.get': (a) => projectView(projectRow(str(a, 'id'))),
  'projects.entries': (a) => projectEntries(str(a, 'id')),
  'projects.create': (a) => {
    const i = a as unknown as ProjectInput;
    const id = write(`Projeto criado: ${(i.name ?? '').trim()}`, () => createProject(i)).value;
    return projectView(projectRow(id));
  },
  'projects.update': (a) => {
    const id = str(a, 'id');
    write(`Projeto alterado: ${projectRow(id).name}`, () =>
      updateProject({ ...(a as unknown as ProjectPatch), id }),
    );
    return projectView(projectRow(id));
  },
  'projects.delete': (a) => {
    const id = str(a, 'id');
    write(`Projeto removido: ${projectRow(id).name}`, () => deleteProject(id));
    return null;
  },
  'projects.link': (a): BulkResult => {
    const ids = Array.isArray(a.txn_ids) ? a.txn_ids.map(String) : [];
    const project = typeof a.project_id === 'string' && a.project_id ? a.project_id : null;
    const reason = project
      ? `${ids.length} lançamento(s) no projeto ${projectRow(project).name}`
      : `${ids.length} lançamento(s) tirados do projeto`;
    const rec = write(reason, () => linkProject(ids, project));
    return { updated: rec.value, change_set_id: rec.changeSetId };
  },
  'projects.add_item': (a) => {
    const i = a as unknown as ProjectItemInput;
    const reason = `Item no projeto ${projectRow(str(a, 'project_id')).name}: ${(i.description ?? '').trim()}`;
    return itemEntry(write(reason, () => addItem(i)).value);
  },
  'projects.update_item': (a) => {
    const id = str(a, 'id');
    write(`Item de projeto alterado: ${itemRow(id).description}`, () =>
      updateItem({ ...(a as unknown as ProjectItemPatch), id }),
    );
    return itemEntry(id);
  },
  'projects.remove_entries': (a): BulkResult => {
    const list = (k: string) => (Array.isArray(a[k]) ? (a[k] as unknown[]).map(String) : []);
    const txns = list('txn_ids');
    const items = list('item_ids');
    const rec = write(`${txns.length + items.length} entrada(s) tiradas do projeto`, () =>
      removeEntries(txns, items),
    );
    return { updated: rec.value, change_set_id: rec.changeSetId };
  },
});
