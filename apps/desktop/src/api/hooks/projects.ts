/** Consultas e mutações dos projetos (`projects.*`): reformas, benfeitorias, viagens… */
import type { BulkResult } from '../generated/BulkResult';
import type { ProjectEntry } from '../generated/ProjectEntry';
import type { ProjectInput } from '../generated/ProjectInput';
import type { ProjectItemInput } from '../generated/ProjectItemInput';
import type { ProjectItemPatch } from '../generated/ProjectItemPatch';
import type { ProjectLinkArgs } from '../generated/ProjectLinkArgs';
import type { ProjectListArgs } from '../generated/ProjectListArgs';
import type { ProjectPatch } from '../generated/ProjectPatch';
import type { ProjectRemoveArgs } from '../generated/ProjectRemoveArgs';
import type { ProjectView } from '../generated/ProjectView';
import { useCore, useCoreMutation } from './coreQuery';

export const useProjects = (args: ProjectListArgs = {}) =>
  useCore<ProjectView[]>('projects', 'list', args);
export const useProject = (id: string | null | undefined) =>
  useCore<ProjectView>('projects', 'get', { id }, { enabled: Boolean(id) });
/** Lançamentos ligados e itens lançados à mão, em ordem de data. */
export const useProjectEntries = (id: string | null | undefined) =>
  useCore<ProjectEntry[]>('projects', 'entries', { id }, { enabled: Boolean(id) });

const AFFECTS = { affects: ['projects'] } as const;

export const useCreateProject = () =>
  useCoreMutation<ProjectInput, ProjectView>('projects.create', AFFECTS);
export const useUpdateProject = () =>
  useCoreMutation<ProjectPatch, ProjectView>('projects.update', AFFECTS);
export const useDeleteProject = () =>
  useCoreMutation<{ id: string }, null>('projects.delete', AFFECTS);
/** Liga (ou tira, com `project_id: null`) lançamentos de um projeto. */
export const useLinkProject = () =>
  useCoreMutation<ProjectLinkArgs, BulkResult>('projects.link', AFFECTS);
/** Item lançado à mão no projeto (fora das contas). */
export const useAddProjectItem = () =>
  useCoreMutation<ProjectItemInput, ProjectEntry>('projects.add_item', AFFECTS);
export const useUpdateProjectItem = () =>
  useCoreMutation<ProjectItemPatch, ProjectEntry>('projects.update_item', AFFECTS);
/** Tira entradas do projeto: lançamentos ficam sem projeto, itens são apagados. */
export const useRemoveProjectEntries = () =>
  useCoreMutation<ProjectRemoveArgs, BulkResult>('projects.remove_entries', AFFECTS);
