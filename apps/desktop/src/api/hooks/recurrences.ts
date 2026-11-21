/** Hooks de `recurrences.*`: lista, prévia das próximas datas, criar, pausar e apagar. */
import type { PreviewArgs } from '../generated/PreviewArgs';
import type { Recurrence } from '../generated/Recurrence';
import type { RecurrenceDeleteArgs } from '../generated/RecurrenceDeleteArgs';
import type { RecurrenceDeleteResult } from '../generated/RecurrenceDeleteResult';
import type { RecurrenceInput } from '../generated/RecurrenceInput';
import type { RecurrenceListArgs } from '../generated/RecurrenceListArgs';
import type { RecurrencePatch } from '../generated/RecurrencePatch';
import type { RecurrencePreview } from '../generated/RecurrencePreview';
import { useCore, useCoreMutation } from './coreQuery';

export const useRecurrences = (args: RecurrenceListArgs = {}) =>
  useCore<Recurrence[]>('recurrences', 'list', args);

/** Prévia ao vivo (sem gravar). Erros de validação voltam como erro da consulta. */
export const useRecurrencePreview = (args: PreviewArgs | null) =>
  useCore<RecurrencePreview>('recurrences', 'preview', args, {
    enabled: Boolean(args),
    staleTime: Infinity,
  });

export const useCreateRecurrence = () =>
  useCoreMutation<RecurrenceInput, Recurrence>('recurrences.create', { affects: ['ledger'] });

export const useUpdateRecurrence = () =>
  useCoreMutation<RecurrencePatch, Recurrence>('recurrences.update', { affects: ['ledger'] });

export const useDeleteRecurrence = () =>
  useCoreMutation<RecurrenceDeleteArgs, RecurrenceDeleteResult>('recurrences.delete', {
    affects: ['ledger'],
  });
