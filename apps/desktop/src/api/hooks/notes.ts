/**
 * Hooks de `notes.*` (notas de corretagem em PDF, padrão SINACOR). A prévia é uma
 * chamada direta (arquivos e senha não vão para o cache); o commit é uma mutação
 * sensível (não guarda as variáveis, que levam a senha do PDF).
 */
import type { NoteCommitArgs } from '../generated/NoteCommitArgs';
import type { NoteCommitResult } from '../generated/NoteCommitResult';
import type { NotePreview } from '../generated/NotePreview';
import type { NotePreviewArgs } from '../generated/NotePreviewArgs';
import { call } from '../client';
import { useCoreMutation } from './coreQuery';

export const previewNotes = (args: NotePreviewArgs) => call<NotePreview>('notes.preview', args);

export const useCommitNotes = () =>
  useCoreMutation<NoteCommitArgs, NoteCommitResult>('notes.commit', {
    affects: ['ledger', 'market'],
    sensitive: true,
  });
