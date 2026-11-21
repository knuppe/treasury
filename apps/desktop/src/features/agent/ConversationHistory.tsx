/**
 * Conversas gravadas: as linhas (abrir, fixar, apagar) e o botão de apagar o histórico,
 * usados pela lista da tela 10 e pela vista "Conversas" do painel do agente. A conversa
 * nunca é listada dentro do fio: abrir uma troca o fio inteiro.
 */
import { useState } from 'react';

import type { Conversation } from '@/api/generated/Conversation';
import { useClearConversations, useDeleteConversation, usePinConversation } from '@/api/hooks/ai';
import { Button, Dialog, DialogClose, Icon, Private, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';
import { formatDateTime } from '@/lib/format';

function ConversationRow({
  c,
  selected,
  onSelect,
  onDeleted,
}: {
  c: Conversation;
  selected: boolean;
  onSelect: (id: string) => void;
  onDeleted: (id: string) => void;
}) {
  const del = useDeleteConversation();
  const pin = usePinConversation();
  return (
    <li
      className={cx(
        'group flex items-center gap-0.5 rounded-control pr-1',
        selected ? 'bg-accent-soft' : 'hover:bg-surface-2',
      )}
    >
      <button
        type="button"
        aria-current={selected ? 'true' : undefined}
        className="flex min-w-0 flex-1 flex-col gap-0.5 rounded-control px-2.5 py-2 text-left"
        onClick={() => onSelect(c.id)}
      >
        <span className={cx('truncate text-[13px]', selected ? 'text-accent' : 'text-fg')}>
          <Private>{c.title}</Private>
        </span>
        <span className="num text-[10.5px] text-dim">
          {t('agente-conversa-meta', { quando: formatDateTime(c.updated_at), n: c.message_count })}
        </span>
      </button>
      <button
        type="button"
        aria-pressed={c.pinned}
        aria-label={t(c.pinned ? 'agente-desafixar' : 'agente-fixar', { titulo: c.title })}
        title={c.pinned ? t('agente-fixada-dica') : t('agente-fixar-dica')}
        className={cx(
          'rounded-sm p-1.5 focus-visible:opacity-100',
          c.pinned ? 'text-accent' : 'text-dim opacity-0 group-hover:opacity-100 hover:text-fg',
        )}
        onClick={() => pin.mutate({ id: c.id, pinned: !c.pinned })}
      >
        <Icon name="pin" size={12} />
      </button>
      <button
        type="button"
        aria-label={t('agente-apagar-conversa', { titulo: c.title })}
        className="rounded-sm p-1.5 text-dim opacity-0 group-hover:opacity-100 hover:text-negative focus-visible:opacity-100"
        onClick={() =>
          del.mutate(c.id, {
            onSuccess: () => onDeleted(c.id),
            onError: (e) =>
              toast({
                tone: 'error',
                title: t('agente-nao-apagou'),
                description: errorMessage(e),
              }),
          })
        }
      >
        <Icon name="x" size={12} />
      </button>
    </li>
  );
}

export function ConversationRows({
  items,
  selected,
  onSelect,
  label = t('agente-conversas'),
  className,
}: {
  items: Conversation[];
  selected: string | null;
  onSelect: (id: string | null) => void;
  label?: string;
  className?: string;
}) {
  return (
    <ul aria-label={label} className={cx('flex flex-col gap-0.5', className)}>
      {items.map((c) => (
        <ConversationRow
          key={c.id}
          c={c}
          selected={selected === c.id}
          onSelect={onSelect}
          onDeleted={(id) => {
            if (selected === id) onSelect(null);
          }}
        />
      ))}
    </ul>
  );
}

/** "Apagar histórico", com confirmação. Propostas, alertas e memória ficam. */
export function ClearHistoryButton({ onCleared }: { onCleared: () => void }) {
  const clear = useClearConversations();
  const [open, setOpen] = useState(false);
  return (
    <Dialog
      open={open}
      onOpenChange={setOpen}
      size="sm"
      title={t('agente-apagar-todas')}
      description={t('agente-apagar-todas-texto')}
      trigger={
        <Button size="sm" variant="ghost">
          {t('agente-apagar-historico')}
        </Button>
      }
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="danger"
            loading={clear.isPending}
            onClick={() =>
              clear.mutate(undefined, {
                onSuccess: () => {
                  setOpen(false);
                  onCleared();
                },
              })
            }
          >
            {t('agente-apagar-tudo')}
          </Button>
        </>
      }
    />
  );
}
