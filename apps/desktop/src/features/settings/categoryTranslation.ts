/**
 * Depois de trocar o idioma com o cofre aberto (docs/i18n-design.md › Escolha do idioma › Depois):
 * se ainda há categorias padrão com o nome padrão de outro idioma, oferece uma vez traduzi-las.
 * As que a pessoa renomeou ficam como estão (o núcleo só mexe nas que têm um nome padrão).
 */
import type { QueryClient } from '@tanstack/react-query';

import { call } from '@/api/client';
import { invalidateGroups } from '@/api/hooks/coreQuery';
import type { useUndoToast } from '@/api/hooks/undoToast';
import { toast } from '@/design/components';
import { errorText } from '@/features/transactions/ui/errors';
import { t } from '@/i18n';

/** A oferta fica mais tempo na tela que um aviso comum: é uma pergunta. */
const OFFER_MS = 15_000;

export async function offerCategoryTranslation(
  client: QueryClient,
  undoToast: ReturnType<typeof useUndoToast>,
): Promise<void> {
  let n: number;
  try {
    n = await call<number>('categories.untranslated_defaults');
  } catch {
    // Sem cofre aberto (ou bloqueado): nada a oferecer.
    return;
  }
  if (n <= 0) return;
  toast({
    title: t('idioma-categorias-pergunta'),
    description: t('idioma-categorias-detalhe', { n }),
    duration: OFFER_MS,
    action: {
      label: t('idioma-categorias-traduzir'),
      onClick: () => {
        void call<number>('categories.translate_defaults')
          .then(async (done) => {
            await invalidateGroups(client, ['categories']);
            undoToast({ title: t('idioma-categorias-traduzidas', { n: done }) });
          })
          .catch((e: unknown) =>
            toast({
              tone: 'error',
              title: t('idioma-categorias-pergunta'),
              description: errorText(e),
            }),
          );
      },
    },
  });
}
