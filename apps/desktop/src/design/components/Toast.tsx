import * as RToast from '@radix-ui/react-toast';
import { create } from 'zustand';
import { t as tr } from '@/i18n';

import { cx } from '../cx';
import { Icon } from './Icon';

export type ToastTone = 'info' | 'success' | 'warning' | 'error';

export interface ToastInput {
  title: string;
  description?: string;
  tone?: ToastTone;
  /** Milissegundos na tela (padrão 5 s; erros 8 s; 8 s quando há ação). */
  duration?: number;
  /** Botão no aviso (ex.: Desfazer). */
  action?: { label: string; onClick: () => void };
}

interface ToastItem extends Required<Pick<ToastInput, 'title' | 'tone'>> {
  id: string;
  description?: string;
  duration?: number;
  action?: { label: string; onClick: () => void };
}

interface ToastStore {
  toasts: ToastItem[];
  push: (toast: ToastItem) => void;
  dismiss: (id: string) => void;
  clear: () => void;
}

export const useToastStore = create<ToastStore>()((set) => ({
  toasts: [],
  push: (toast) => set((s) => ({ toasts: [...s.toasts.slice(-4), toast] })),
  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
  clear: () => set({ toasts: [] }),
}));

let counter = 0;

/** Mostra uma notificação. Devolve o id (para `dismissToast`). */
export function toast(input: ToastInput): string {
  counter += 1;
  const id = `toast-${counter}`;
  useToastStore.getState().push({ tone: 'info', ...input, id });
  return id;
}

export function dismissToast(id: string): void {
  useToastStore.getState().dismiss(id);
}

const TONE = {
  info: {
    bar: 'border-l-accent',
    text: 'text-accent',
    icon: 'info',
    get label() {
      return tr('componentes-toast-aviso');
    },
  },
  success: {
    bar: 'border-l-positive',
    text: 'text-positive',
    icon: 'check',
    get label() {
      return tr('componentes-toast-feito');
    },
  },
  warning: {
    bar: 'border-l-warning',
    text: 'text-warning',
    icon: 'alert',
    get label() {
      return tr('componentes-toast-atencao');
    },
  },
  error: {
    bar: 'border-l-negative',
    text: 'text-negative',
    icon: 'alert',
    get label() {
      return tr('componentes-toast-erro');
    },
  },
} as const;

/** Região das notificações. Renderize uma vez, perto da raiz do app. */
export function Toaster() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  return (
    <RToast.Provider swipeDirection="right" label={tr('componentes-notificacoes')}>
      {toasts.map((t) => {
        const tone = TONE[t.tone];
        return (
          <RToast.Root
            key={t.id}
            type={t.tone === 'error' ? 'foreground' : 'background'}
            duration={t.duration ?? (t.tone === 'error' || t.action ? 8000 : 5000)}
            onOpenChange={(open) => {
              if (!open) dismiss(t.id);
            }}
            className={cx(
              'pointer-events-auto flex animate-slide-in items-start gap-3 rounded-card border border-l-2 border-panel pop-surface py-3 pr-2 pl-3.5 data-[swipe=move]:translate-x-[var(--radix-toast-swipe-move-x)]',
              tone.bar,
            )}
          >
            <span className={cx('mt-0.5', tone.text)}>
              <Icon name={tone.icon} size={14} />
            </span>
            <div className="min-w-0 flex-1">
              <RToast.Title className="text-[13px] font-medium text-fg">
                <span className="sr-only">{tone.label}: </span>
                {t.title}
              </RToast.Title>
              {t.description ? (
                <RToast.Description className="mt-0.5 text-[12.5px] leading-snug text-dim">
                  {t.description}
                </RToast.Description>
              ) : null}
            </div>
            {t.action ? (
              <RToast.Action
                altText={t.action.label}
                onClick={t.action.onClick}
                className="label-caps shrink-0 self-center rounded-control px-2 py-1 text-accent hover:bg-surface"
              >
                {t.action.label}
              </RToast.Action>
            ) : null}
            <RToast.Close
              aria-label={tr('componentes-fechar-notificacao')}
              className="inline-flex size-6 shrink-0 items-center justify-center rounded-control text-dim hover:bg-surface hover:text-fg"
            >
              <Icon name="x" size={12} />
            </RToast.Close>
          </RToast.Root>
        );
      })}
      <RToast.Viewport className="pointer-events-none fixed right-4 bottom-4 z-[80] flex w-[min(92vw,380px)] flex-col gap-2 outline-none" />
    </RToast.Provider>
  );
}
