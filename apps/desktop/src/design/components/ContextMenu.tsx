/**
 * Menu de contexto do próprio app (o menu nativo da webview fica desligado desde a 0.2.0):
 * abre com o botão direito no ponteiro, ou com Shift+F10 / tecla Menu junto ao elemento
 * com foco. Setas ↑↓ percorrem (pulando os desativados), Home/End vão às pontas, uma letra
 * pula para o item que começa com ela, Enter ou Espaço escolhem, Esc fecha e devolve o foco
 * a quem abriu. Clique fora, rolagem, Tab e troca de janela fecham sem escolher.
 *
 * Dois jeitos de usar: `<ContextMenu items label>` em volta de um card (ou qualquer área) e
 * `useContextMenu()` para quem abre por conta própria (as linhas da `TechTable`).
 */
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type MouseEvent,
  type ReactNode,
} from 'react';
import { FocusScope } from '@radix-ui/react-focus-scope';
import { createPortal } from 'react-dom';

import { ariaShortcut } from '@/lib/platform';

import { cx } from '../cx';
import { Icon, type IconName } from './Icon';
import { Kbd } from './Kbd';

export interface ContextMenuAction {
  label: string;
  onSelect: () => void;
  icon?: IconName;
  /**
   * Atalho que faz o mesmo na tela (a dica; vira `aria-keyshortcuts`). O menu não registra a
   * tecla: quem o abre registra (a `TechTable` dispara os atalhos dos menus das linhas).
   */
  shortcut?: readonly string[];
  disabled?: boolean;
  /** Por que está desativado (vira a dica do item). */
  disabledReason?: string;
  /** Ação destrutiva (apagar, excluir): em vermelho. */
  danger?: boolean;
}

/** Item, separador ou título de grupo; `null`/`false` somem (para itens condicionais). */
export type ContextMenuEntry =
  ContextMenuAction | { separator: true } | { heading: string } | null | false | undefined;

export interface ContextMenuRequest {
  entries: readonly ContextMenuEntry[];
  /** Nome acessível do menu (`Ações de Nubank`). */
  label: string;
  /** Ponto na janela (clique direito) ou o elemento ao lado do qual abrir (teclado). */
  at: { x: number; y: number } | Element;
  /** Quem recebe o foco ao fechar sem escolher (padrão: o que tinha foco ao abrir). */
  returnFocus?: HTMLElement | null;
}

interface OpenMenu {
  /** Cada abertura é um menu novo (foco no primeiro item, posição medida de novo). */
  seq: number;
  entries: Clean[];
  label: string;
  x: number;
  y: number;
  returnFocus: HTMLElement | null;
}

type Clean = ContextMenuAction | { separator: true } | { heading: string };

const isAction = (e: Clean): e is ContextMenuAction => 'onSelect' in e;

/** Campos e textos copiáveis continuam com o menu nativo (copiar, colar). */
const NATIVE =
  'input, textarea, select, [contenteditable=""], [contenteditable="true"], [data-selectable]';

export function wantsNativeMenu(target: EventTarget | null): boolean {
  return target instanceof Element && Boolean(target.closest(NATIVE));
}

/** Shift+F10 ou a tecla Menu (as teclas de abrir o menu de contexto pelo teclado). */
export function isContextMenuKey(e: { key: string; shiftKey: boolean }): boolean {
  return e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10');
}

/** Separadores nas pontas e repetidos somem; títulos sem itens depois também. */
function clean(entries: readonly ContextMenuEntry[]): Clean[] {
  const list = entries.filter((e): e is Clean => Boolean(e));
  const out: Clean[] = [];
  for (const [i, e] of list.entries()) {
    if ('separator' in e) {
      if (out.length === 0 || 'separator' in out[out.length - 1]!) continue;
    } else if ('heading' in e) {
      const next = list[i + 1];
      if (!next || !isAction(next)) continue;
    }
    out.push(e);
  }
  while (out.length && 'separator' in out[out.length - 1]!) out.pop();
  return out;
}

function pointOf(at: ContextMenuRequest['at']): { x: number; y: number } {
  if (!(at instanceof Element)) return at;
  const r = at.getBoundingClientRect();
  return { x: r.left + Math.min(24, r.width / 2), y: r.top + Math.min(r.height, 32) };
}

/**
 * Estado de um menu de contexto: `open` mostra, `menu` é o elemento a renderizar (num
 * portal, por cima de tudo). Abrir outro fecha o anterior.
 */
export function useContextMenu() {
  const [state, setState] = useState<OpenMenu | null>(null);
  const openedByKey = useRef(0);
  const seq = useRef(0);

  const open = useCallback((req: ContextMenuRequest) => {
    const entries = clean(req.entries);
    if (!entries.some(isAction)) return false;
    const { x, y } = pointOf(req.at);
    const active = document.activeElement;
    setState({
      seq: ++seq.current,
      entries,
      label: req.label,
      x,
      y,
      returnFocus:
        req.returnFocus !== undefined
          ? req.returnFocus
          : active instanceof HTMLElement && active !== document.body
            ? active
            : null,
    });
    return true;
  }, []);

  const close = useCallback(() => setState(null), []);

  /** Clique direito: abre no ponteiro (ou no elemento, se veio do teclado). */
  const onContextMenu = useCallback(
    (e: MouseEvent<Element>, req: Omit<ContextMenuRequest, 'at'>) => {
      if (wantsNativeMenu(e.target)) return false;
      e.preventDefault();
      e.stopPropagation();
      // O navegador também dispara `contextmenu` depois de Shift+F10 / tecla Menu.
      if (Date.now() - openedByKey.current < 400) return true;
      const fromKeyboard = e.clientX === 0 && e.clientY === 0;
      return open({
        ...req,
        at: fromKeyboard ? (e.target as Element) : { x: e.clientX, y: e.clientY },
      });
    },
    [open],
  );

  /** Shift+F10 / tecla Menu sobre o elemento com foco. */
  const onKeyDown = useCallback(
    (e: KeyboardEvent<Element>, req: Omit<ContextMenuRequest, 'at'>) => {
      if (!isContextMenuKey(e) || e.altKey || e.metaKey || e.ctrlKey) return false;
      if (wantsNativeMenu(e.target)) return false;
      e.preventDefault();
      e.stopPropagation();
      openedByKey.current = Date.now();
      return open({ ...req, at: e.target as Element });
    },
    [open],
  );

  const menu = state ? <MenuPopup key={state.seq} menu={state} onClose={close} /> : null;
  return { open, close, isOpen: state !== null, menu, onContextMenu, onKeyDown };
}

function MenuPopup({ menu, onClose }: { menu: OpenMenu; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const items = useRef<(HTMLButtonElement | null)[]>([]);

  // Cabe na janela: vira para a esquerda/para cima quando não cabe no ponto.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const pad = 8;
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    let left = menu.x;
    let top = menu.y;
    if (left + w + pad > vw) left = Math.max(pad, menu.x - w);
    if (top + h + pad > vh) top = Math.max(pad, vh - h - pad);
    el.style.left = `${left}px`;
    el.style.top = `${top}px`;
    el.style.visibility = 'visible';
  }, [menu]);

  const enabled = () =>
    items.current.filter((el): el is HTMLButtonElement => el !== null && !el.disabled);

  useEffect(() => {
    items.current.find((el) => el && !el.disabled)?.focus({ preventScroll: true });
  }, []);

  const dismiss = useCallback(
    (restore: boolean) => {
      onClose();
      if (restore && menu.returnFocus?.isConnected) menu.returnFocus.focus({ preventScroll: true });
    },
    [menu.returnFocus, onClose],
  );

  useEffect(() => {
    const outside = (e: Event) => {
      if (ref.current && e.target instanceof Node && ref.current.contains(e.target)) return;
      dismiss(false);
    };
    const away = () => dismiss(false);
    document.addEventListener('pointerdown', outside, true);
    document.addEventListener('scroll', outside, true);
    window.addEventListener('resize', away);
    window.addEventListener('blur', away);
    return () => {
      document.removeEventListener('pointerdown', outside, true);
      document.removeEventListener('scroll', outside, true);
      window.removeEventListener('resize', away);
      window.removeEventListener('blur', away);
    };
  }, [dismiss]);

  const choose = (action: ContextMenuAction) => {
    if (action.disabled) return;
    // Fecha e devolve o foco antes: o diálogo que a ação abrir guarda quem o abriu.
    dismiss(true);
    action.onSelect();
  };

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const list = enabled();
    const at = list.indexOf(document.activeElement as HTMLButtonElement);
    const move = (to: number) => list[(to + list.length) % list.length]?.focus();
    e.stopPropagation();
    switch (e.key) {
      case 'ArrowDown':
        move(at + 1);
        break;
      case 'ArrowUp':
        move(at < 0 ? list.length - 1 : at - 1);
        break;
      case 'Home':
        move(0);
        break;
      case 'End':
        move(list.length - 1);
        break;
      case 'Escape':
        dismiss(true);
        break;
      case 'Tab':
        dismiss(true);
        break;
      case 'F10':
      case 'ContextMenu':
        if (isContextMenuKey(e)) dismiss(true);
        else return;
        break;
      default: {
        if (e.key.length !== 1 || e.metaKey || e.ctrlKey || e.altKey || e.key === ' ') return;
        const letter = e.key.toLocaleLowerCase('pt-BR');
        const order = [...list.slice(at + 1), ...list.slice(0, at + 1)];
        order
          .find((el) => (el.dataset.label ?? '').toLocaleLowerCase('pt-BR').startsWith(letter))
          ?.focus();
      }
    }
    e.preventDefault();
  };

  let actionIndex = 0;
  // O escopo de foco pausa o de uma janela (diálogo) aberta, que senão puxaria o foco
  // de volta; `pointerEvents` porque a janela modal desliga o mouse no body.
  return createPortal(
    <FocusScope
      asChild
      onMountAutoFocus={(e) => e.preventDefault()}
      onUnmountAutoFocus={(e) => e.preventDefault()}
    >
      <div
        ref={ref}
        role="menu"
        aria-label={menu.label}
        aria-orientation="vertical"
        onKeyDown={onKeyDown}
        onContextMenu={(e) => {
          e.preventDefault();
          e.stopPropagation();
        }}
        style={{
          position: 'fixed',
          left: menu.x,
          top: menu.y,
          visibility: 'hidden',
          pointerEvents: 'auto',
        }}
        className="z-[80] max-h-[calc(100vh-16px)] min-w-52 max-w-80 animate-pop-in overflow-y-auto rounded-card border border-panel pop-surface p-1.5"
      >
        {menu.entries.map((entry, i) => {
          if ('separator' in entry) {
            return <div key={`sep-${i}`} role="separator" className="my-1 h-px bg-line" />;
          }
          if ('heading' in entry) {
            return (
              <p
                key={`h-${i}`}
                role="presentation"
                className="label-caps truncate px-2 pt-1.5 pb-1 text-[10px]"
              >
                {entry.heading}
              </p>
            );
          }
          const slot = actionIndex++;
          return (
            <button
              key={`${entry.label}-${i}`}
              ref={(el) => {
                items.current[slot] = el;
              }}
              type="button"
              role="menuitem"
              tabIndex={-1}
              disabled={entry.disabled}
              aria-disabled={entry.disabled || undefined}
              title={entry.disabled ? entry.disabledReason : undefined}
              data-label={entry.label}
              aria-keyshortcuts={entry.shortcut ? ariaShortcut(entry.shortcut) : undefined}
              onClick={() => choose(entry)}
              onPointerMove={(e) => {
                if (!entry.disabled && document.activeElement !== e.currentTarget)
                  e.currentTarget.focus({ preventScroll: true });
              }}
              className={cx(
                'flex h-8 w-full items-center gap-2.5 rounded-control px-2 text-left text-[13px] outline-none',
                'focus:bg-accent-soft disabled:cursor-default disabled:opacity-45',
                entry.danger ? 'text-negative' : 'text-fg',
              )}
            >
              <span
                aria-hidden
                className={cx(
                  'flex size-4 shrink-0 items-center justify-center',
                  entry.danger ? 'text-negative' : 'text-dim',
                )}
              >
                {entry.icon ? <Icon name={entry.icon} size={13} /> : null}
              </span>
              <span className="min-w-0 flex-auto truncate">{entry.label}</span>
              {entry.shortcut ? (
                <span aria-hidden className="ml-3 flex">
                  <Kbd keys={entry.shortcut} />
                </span>
              ) : null}
            </button>
          );
        })}
      </div>
    </FocusScope>,
    document.body,
  );
}

export interface ContextMenuProps {
  /** Itens; uma função é chamada só ao abrir (para itens que dependem do estado atual). */
  items: readonly ContextMenuEntry[] | (() => readonly ContextMenuEntry[]);
  label: string;
  children: ReactNode;
  disabled?: boolean;
  className?: string;
}

/**
 * Área com menu de contexto (um card, por exemplo). Não muda o layout (`display:
 * contents`); o teclado abre o menu a partir de qualquer elemento com foco dentro dela.
 */
export function ContextMenu({ items, label, children, disabled, className }: ContextMenuProps) {
  const ctx = useContextMenu();
  const req = () => ({ entries: typeof items === 'function' ? items() : items, label });
  return (
    <div
      className={cx('contents', className)}
      onContextMenu={disabled ? undefined : (e) => ctx.onContextMenu(e, req())}
      onKeyDown={disabled ? undefined : (e) => ctx.onKeyDown(e, req())}
    >
      {children}
      {ctx.menu}
    </div>
  );
}
