/**
 * Aviso de sucesso com "Desfazer" para escritas que não devolvem o id do conjunto de
 * alterações (dívidas, IR…): logo depois da escrita, o conjunto mais recente é o dela.
 * Só oferece o botão se esse conjunto for do usuário e de agora há pouco — uma escrita que
 * não mudou nada não cria conjunto, e o botão não pode desfazer outra alteração. Quem já
 * tem o id (`change_set_id` de um lote) passa `changeSetId`; `null` = nada a desfazer.
 */
import { useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';

import { call } from '../client';
import type { ChangeSetSummary } from '../generated/ChangeSetSummary';
import { toast, type ToastInput } from '@/design/components';
import { t } from '@/i18n';
import { invalidateAll } from './coreQuery';

/** Janela em que o conjunto mais recente ainda é considerado o desta escrita. */
const FRESH_MS = 15_000;

export function useUndoToast(): (
  input: Omit<ToastInput, 'action'> & { changeSetId?: string | null },
) => void {
  const client = useQueryClient();
  return useCallback(
    ({ changeSetId, ...input }) => {
      void (async () => {
        let id: string | null = changeSetId ?? null;
        if (changeSetId === undefined) {
          try {
            const list = await call<ChangeSetSummary[]>('changes.list', {
              limit: 1,
              author_kind: 'user',
            });
            const last = list[0];
            const fresh = last ? Date.now() - Date.parse(last.created_at) < FRESH_MS : false;
            id = last && fresh && last.status === 'applied' ? last.id : null;
          } catch {
            id = null;
          }
        }
        toast({
          tone: 'success',
          ...input,
          ...(id
            ? {
                action: {
                  label: t('componentes-desfazer'),
                  onClick: () => {
                    void call('changes.undo', { id })
                      .then(() => invalidateAll(client))
                      .then(() =>
                        toast({ title: t('componentes-desfeito'), description: input.title }),
                      )
                      .catch((e: unknown) =>
                        toast({
                          tone: 'error',
                          title: t('componentes-nao-desfez'),
                          description: e instanceof Error ? e.message : String(e),
                        }),
                      );
                  },
                },
              }
            : {}),
        });
      })();
    },
    [client],
  );
}
