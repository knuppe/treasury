import * as RDialog from '@radix-ui/react-dialog';
import type { ReactElement, ReactNode } from 'react';

import { cx } from '../cx';
import { Icon } from './Icon';
import { IconButton } from './IconButton';
import { useDialogResize } from './useDialogResize';
import { WindowDots } from './WindowDots';
import { t } from '@/i18n';

export interface DialogProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  /** Elemento que abre o diálogo (recebe os props via `asChild`). */
  trigger?: ReactElement;
  title: ReactNode;
  description?: ReactNode;
  /** Índice no cabeçalho, como nos cards (`00`). */
  index?: string;
  children?: ReactNode;
  /** Ações no rodapé, alinhadas à direita. */
  footer?: ReactNode;
  /** `xl`: assistentes com tabelas largas (importações). */
  size?: 'sm' | 'md' | 'lg' | 'xl';
  /**
   * O usuário pode redimensionar pela alça do canto ou pelas bordas e maximizar
   * (assistentes com tabelas). Com `resizeId`, o tamanho fica lembrado.
   */
  resizable?: boolean;
  resizeId?: string;
  className?: string;
}

const SIZES = {
  sm: 'w-[min(92vw,420px)]',
  md: 'w-[min(92vw,540px)]',
  lg: 'w-[min(94vw,760px)]',
  xl: 'w-[min(96vw,1080px)]',
};

/**
 * Diálogo modal (Radix Dialog) com moldura de instrumento: foco preso, Esc
 * fecha, foco volta ao gatilho ao fechar, título e descrição anunciados.
 * Só o corpo rola: o cabeçalho e o rodapé (com a ação principal) ficam sempre
 * à vista, mesmo em janelas baixas.
 */
export function Dialog({
  open,
  onOpenChange,
  trigger,
  title,
  description,
  index,
  children,
  footer,
  size = 'md',
  resizable = false,
  resizeId,
  className,
}: DialogProps) {
  const r = useDialogResize(resizable ? resizeId : undefined);
  const sized = resizable && r.size;
  return (
    <RDialog.Root open={open} onOpenChange={onOpenChange}>
      {trigger ? <RDialog.Trigger asChild>{trigger}</RDialog.Trigger> : null}
      <RDialog.Portal>
        <RDialog.Overlay className="fixed inset-0 z-40 animate-fade-in bg-overlay backdrop-blur-[3px]" />
        <RDialog.Content
          // Sem descrição, `aria-describedby` explícito evita o aviso do Radix.
          {...(description ? {} : { 'aria-describedby': undefined })}
          className={cx(
            'fixed left-1/2 z-50 flex -translate-x-1/2 animate-pop-in flex-col overflow-hidden rounded-card border border-panel pop-surface focus:outline-none',
            // Redimensionável fica no centro da tela: a alça cresce dos dois lados por igual.
            resizable
              ? 'top-1/2 max-h-[96vh] max-w-[98vw] -translate-y-1/2'
              : 'top-[14vh] max-h-[76vh]',
            !sized && SIZES[size],
            className,
          )}
          style={sized ? { width: r.size!.w, height: r.size!.h } : undefined}
        >
          <div className="flex h-10 shrink-0 items-center gap-3 border-b border-line pr-1.5 pl-3.5">
            <WindowDots />
            {index ? <span className="num text-[11px] text-accent">{index}</span> : null}
            <RDialog.Title className="label-caps min-w-0 flex-1 truncate text-fg">
              {title}
            </RDialog.Title>
            {resizable ? (
              <IconButton
                size="sm"
                label={
                  r.maximized ? t('componentes-restaurar-tamanho') : t('componentes-maximizar')
                }
                icon={<Icon name={r.maximized ? 'restore' : 'maximize'} />}
                tooltip={false}
                onClick={r.toggleMaximized}
              />
            ) : null}
            <RDialog.Close asChild>
              <IconButton
                size="sm"
                label={t('comum-fechar')}
                icon={<Icon name="x" />}
                tooltip={false}
              />
            </RDialog.Close>
          </div>
          <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4">
            {description ? (
              <RDialog.Description className="text-[13.5px] leading-relaxed text-dim">
                {description}
              </RDialog.Description>
            ) : null}
            {children}
          </div>
          {footer ? (
            <div
              className={cx(
                'flex shrink-0 items-center justify-end gap-2 border-t border-line px-4 py-3',
                resizable && 'pr-7',
              )}
            >
              {footer}
            </div>
          ) : null}
          {resizable ? (
            <>
              <div
                aria-hidden
                className="absolute top-10 right-0 bottom-4 w-1.5 cursor-ew-resize"
                {...r.handle('right')}
              />
              <div
                aria-hidden
                className="absolute right-4 bottom-0 left-0 h-1.5 cursor-ns-resize"
                {...r.handle('bottom')}
              />
              <div
                role="separator"
                aria-label={t('componentes-redimensionar')}
                title={t('componentes-redimensionar-dica')}
                className="absolute right-0 bottom-0 flex size-5 cursor-nwse-resize items-end justify-end p-1 text-dim hover:text-accent"
                {...r.handle('corner')}
              >
                <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
                  <path
                    d="M9 1 1 9M9 5 5 9"
                    stroke="currentColor"
                    strokeWidth="1.25"
                    strokeLinecap="round"
                  />
                </svg>
              </div>
            </>
          ) : null}
        </RDialog.Content>
      </RDialog.Portal>
    </RDialog.Root>
  );
}

/** Fecha o diálogo em volta ao ser clicado (use com `asChild` em um Button). */
export const DialogClose = RDialog.Close;
