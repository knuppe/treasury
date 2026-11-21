import { useState, type ReactNode } from 'react';

import { isApiError } from '@/api/client';
import { Button, Dialog, DialogClose, Icon, LiveIndicator } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

/** Carregando: indicador discreto no lugar do conteúdo. */
export function LoadingBlock({
  label = t('controles-carregando'),
  className,
}: {
  label?: string;
  className?: string;
}) {
  return (
    <div className={cx('flex min-h-24 items-center justify-center py-8', className)} role="status">
      <LiveIndicator label={label} tone="accent" />
    </div>
  );
}

export function errorMessage(error: unknown): string {
  if (isApiError(error)) return error.message;
  if (error instanceof Error) return error.message;
  return t('carteira-falha-desconhecida');
}

/** Erro de uma consulta, com a mensagem do núcleo e um botão de tentar de novo. */
export function ErrorBlock({
  error,
  onRetry,
  title = t('carteira-erro-carregar'),
  className,
}: {
  error: unknown;
  onRetry?: () => void;
  title?: string;
  className?: string;
}) {
  return (
    <div
      role="alert"
      className={cx(
        'flex flex-wrap items-start gap-3 rounded-card border border-negative/40 bg-[var(--negative-soft)] px-4 py-3',
        className,
      )}
    >
      <span className="mt-0.5 text-negative">
        <Icon name="alert" size={14} />
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-[13px] font-medium text-fg">{title}</p>
        <p className="text-[12.5px] text-dim">{errorMessage(error)}</p>
      </div>
      {onRetry ? (
        <Button size="sm" onClick={onRetry} icon={<Icon name="refresh" size={12} />}>
          {t('controles-tentar-de-novo')}
        </Button>
      ) : null}
    </div>
  );
}

/** Aviso em faixa (informação ou atenção) dentro de uma tela. */
export function Notice({
  tone = 'info',
  title,
  children,
  actions,
  className,
}: {
  tone?: 'info' | 'warning';
  title: ReactNode;
  children?: ReactNode;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <div
      role={tone === 'warning' ? 'status' : 'note'}
      className={cx(
        'flex flex-wrap items-start gap-3 rounded-card border border-l-2 px-4 py-3',
        tone === 'warning'
          ? 'border-warning/40 border-l-warning bg-[var(--warning-soft)]'
          : 'border-line border-l-accent bg-[var(--accent-faint)]',
        className,
      )}
    >
      <span className={cx('mt-0.5', tone === 'warning' ? 'text-warning' : 'text-accent')}>
        <Icon name={tone === 'warning' ? 'alert' : 'info'} size={14} />
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <p className="text-[13px] font-medium text-fg">{title}</p>
        {children ? <div className="text-[12.5px] leading-relaxed text-dim">{children}</div> : null}
      </div>
      {actions ? <div className="flex flex-wrap items-center gap-2">{actions}</div> : null}
    </div>
  );
}

/** Botão "Excluir" com confirmação (ação irreversível pela interface; o histórico desfaz). */
export function ConfirmButton({
  label,
  title,
  description,
  confirmLabel = t('carteira-excluir'),
  onConfirm,
  pending,
  size = 'sm',
}: {
  label: ReactNode;
  title: string;
  description: ReactNode;
  confirmLabel?: string;
  onConfirm: () => Promise<unknown> | void;
  pending?: boolean;
  size?: 'sm' | 'md';
}) {
  const [open, setOpen] = useState(false);
  return (
    <Dialog
      open={open}
      onOpenChange={setOpen}
      size="sm"
      title={title}
      description={description}
      trigger={
        <Button size={size} variant="ghost">
          {label}
        </Button>
      }
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button
            variant="danger"
            loading={pending}
            onClick={() => {
              void Promise.resolve(onConfirm()).then(() => setOpen(false));
            }}
          >
            {confirmLabel}
          </Button>
        </>
      }
    />
  );
}
