/**
 * Aviso da caixa de entrada de arquivos: o app manda `inbox://new` quando um arquivo que chegou
 * na pasta de entrada vira item da caixa; aqui vira um aviso com o atalho para revisar (e a
 * lista da caixa se atualiza).
 */
import { useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { useNavigate } from 'react-router';

import type { InboxItem } from '@/api/generated/InboxItem';
import { INBOX_KEY } from '@/api/hooks/inbox';
import { toast } from '@/design/components';
import { listenApp } from '@/features/agent/tauriBridge';
import { t } from '@/i18n';

export function useInboxNotices() {
  const client = useQueryClient();
  const navigate = useNavigate();
  useEffect(
    () =>
      listenApp<InboxItem>('inbox://new', (item) => {
        void client.invalidateQueries({ queryKey: INBOX_KEY });
        void client.invalidateQueries({ queryKey: ['attention'] });
        toast({
          tone: 'info',
          title:
            item.status === 'unrecognized'
              ? t('caixa-aviso-desconhecido', { nome: item.filename })
              : t('caixa-aviso-novo', { tipo: item.label }),
          description: item.filename,
          action: {
            label: t('caixa-aviso-revisar'),
            onClick: () => void navigate('/lancamentos?aba=caixa'),
          },
        });
      }),
    [client, navigate],
  );
}
