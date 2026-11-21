/**
 * Caixa de entrada de arquivos (Lançamentos › Caixa de entrada): os arquivos que chegaram na
 * pasta de entrada, já guardados no cofre e fora da pasta. "Importar" abre a janela única de
 * importação com o arquivo (o importador certo para cada tipo); depois de importado, sai da
 * lista. "Descartar" tira da lista sem importar.
 */
import { useState } from 'react';

import { call } from '@/api/client';
import type { InboxFile } from '@/api/generated/InboxFile';
import { useInbox, inboxFile, useSetInboxStatus } from '@/api/hooks/inbox';
import { useAppSettings } from '@/api/hooks/settings';
import { Button, EmptyState, toast } from '@/design/components';
import { ErrorBlock, LoadingBlock } from '@/features/portfolio/components/States';
import { Checkbox } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

import { DocumentTextDialog } from './DocumentTextDialog';
import { useImportHub } from './importHub';

const STATUS_KEY = {
  pending: 'caixa-status-pendente',
  imported: 'caixa-status-importado',
  dismissed: 'caixa-status-descartado',
  unrecognized: 'caixa-status-desconhecido',
} as const;

export function InboxPanel() {
  const [all, setAll] = useState(false);
  const q = useInbox(all);
  const settings = useAppSettings();
  const setStatus = useSetInboxStatus();
  const folder = settings.data?.inbox_folder?.trim() || '~/Treasury/Inbox';
  const [reading, setReading] = useState<InboxFile | null>(null);
  const showText = async (id: string) => {
    try {
      setReading(await call<InboxFile>('import.inbox_file', { id }));
    } catch (e) {
      toast({ tone: 'error', title: t('caixa-erro-abrir'), description: errorText(e) });
    }
  };

  const open = async (id: string) => {
    try {
      const f = await inboxFile(id);
      useImportHub.getState().openWith([f], { [f.name]: `inbox:${id}` });
    } catch (e) {
      toast({ tone: 'error', title: t('caixa-erro-abrir'), description: errorText(e) });
    }
  };

  return (
    <section aria-label={t('caixa-titulo')} className="flex flex-col gap-3">
      <p className="text-[12.5px] text-dim">
        {settings.data && !settings.data.watch_inbox
          ? t('caixa-desligada')
          : t('caixa-explica', { pasta: folder })}
      </p>
      <div className="flex items-center gap-3">
        <Checkbox checked={all} onChange={setAll} label={t('caixa-mostrar-todos')} />
      </div>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState
          title={t('caixa-vazia')}
          description={t('caixa-vazia-texto', { pasta: folder })}
        />
      ) : (
        <ul
          aria-label={t('caixa-lista')}
          className="flex flex-col rounded-card border border-line bg-surface px-3"
        >
          {q.data.map((i) => (
            <li
              key={i.id}
              className="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-line/60 py-2 last:border-0"
            >
              <div className="flex min-w-48 flex-1 flex-col">
                <span className="text-[13px] text-fg">{i.filename}</span>
                <span className="text-[11.5px] text-dim">
                  {[
                    i.label,
                    formatBytes(i.size),
                    formatDateTime(i.created_at),
                    t(STATUS_KEY[i.status]),
                  ].join(' · ')}
                </span>
              </div>
              {i.status === 'pending' ? (
                <Button size="sm" variant="primary" onClick={() => void open(i.id)}>
                  {t('caixa-importar')}
                </Button>
              ) : null}
              {/\.pdf$/i.test(i.filename) ? (
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={t('documento-ver-texto-nome', { nome: i.filename })}
                  onClick={() => void showText(i.id)}
                >
                  {t('documento-ver-texto')}
                </Button>
              ) : null}
              {i.status === 'pending' || i.status === 'unrecognized' ? (
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={t('caixa-descartar-nome', { nome: i.filename })}
                  loading={setStatus.isPending}
                  onClick={() => setStatus.mutate({ id: i.id, status: 'dismissed' })}
                >
                  {t('caixa-descartar')}
                </Button>
              ) : null}
            </li>
          ))}
        </ul>
      )}
      <DocumentTextDialog file={reading} onClose={() => setReading(null)} />
    </section>
  );
}
