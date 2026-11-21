/** Hooks de `changes.*`: histórico de alterações, detalhe, desfazer e refazer. */
import type { ChangeListArgs } from '../generated/ChangeListArgs';
import type { ChangeSetDetail } from '../generated/ChangeSetDetail';
import type { ChangeSetSummary } from '../generated/ChangeSetSummary';
import { useCore, useCoreMutation } from './coreQuery';

export const useChanges = (args: ChangeListArgs) =>
  useCore<ChangeSetSummary[]>('changes', 'list', args);

export const useChange = (id: string | null) =>
  useCore<ChangeSetDetail>('changes', 'get', { id }, { enabled: Boolean(id) });

/** Desfazer/refazer podem tocar em qualquer tabela: invalidam tudo (menos o cofre). */
export const useUndo = () =>
  useCoreMutation<{ id: string }, ChangeSetSummary>('changes.undo', { affects: 'all' });

export const useRedo = () =>
  useCoreMutation<{ id: string }, ChangeSetSummary>('changes.redo', { affects: 'all' });
