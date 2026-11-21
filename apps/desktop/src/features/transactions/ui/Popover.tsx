import {
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type ReactNode,
  type RefObject,
} from 'react';
import { FocusScope } from '@radix-ui/react-focus-scope';
import { createPortal } from 'react-dom';

import { cx } from '@/design/cx';
import type { Point } from '@/design/pointer';

export interface PopoverProps {
  /** Conteúdo do botão que abre. */
  label: ReactNode;
  /** Nome acessível do botão (quando `label` não é texto). */
  ariaLabel?: string;
  children: ReactNode | ((close: () => void) => ReactNode);
  /** Botão "ativo" (filtro aplicado). */
  active?: boolean;
  align?: 'start' | 'end';
  className?: string;
  panelClassName?: string;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  /** Foco inicial dentro do painel (padrão: o primeiro controle). */
  initialFocus?: RefObject<HTMLElement | null>;
  /**
   * `bare`: botão sem moldura (células de tabela). O painel flutua fora da tabela
   * (portal com posição fixa), para não ser cortado pela célula nem pela rolagem.
   */
  variant?: 'button' | 'bare';
  /**
   * Abre o painel flutuando neste ponto da tela (ex.: no mouse, quando a ação veio do
   * teclado) em vez de junto ao botão.
   */
  anchor?: Point | null;
}

const GAP = 4;
const MARGIN = 8;

/** Posição fixa do painel junto ao botão: abaixo, ou acima se não couber; dentro da tela. */
function placePanel(
  button: HTMLElement,
  panel: HTMLElement,
  align: 'start' | 'end',
): CSSProperties {
  const b = button.getBoundingClientRect();
  const w = panel.offsetWidth;
  const h = panel.offsetHeight;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  let left = align === 'end' ? b.right - w : b.left;
  left = Math.max(MARGIN, Math.min(left, vw - w - MARGIN));
  const below = vh - b.bottom - GAP - MARGIN;
  const above = b.top - GAP - MARGIN;
  const up = h > below && above > below;
  const top = up ? Math.max(MARGIN, b.top - GAP - h) : b.bottom + GAP;
  return { position: 'fixed', left, top, maxHeight: Math.max(160, up ? above : below) };
}

/** Posição fixa do painel num ponto: abaixo e à direita, virando para caber na tela. */
function placeAt(at: Point, panel: HTMLElement): CSSProperties {
  const w = panel.offsetWidth;
  const h = panel.offsetHeight;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const left = Math.max(MARGIN, Math.min(at.x, vw - w - MARGIN));
  const below = vh - at.y - GAP - MARGIN;
  const above = at.y - GAP - MARGIN;
  const up = h > below && above > below;
  const top = up ? Math.max(MARGIN, at.y - GAP - h) : at.y + GAP;
  return { position: 'fixed', left, top, maxHeight: Math.max(160, up ? above : below) };
}

/**
 * Painel flutuante simples (filtros, seletores): abre num botão, fecha com Esc,
 * clique fora ou ao sair pelo Tab; devolve o foco ao botão ao fechar.
 */
export function Popover({
  label,
  ariaLabel,
  children,
  active,
  align = 'start',
  className,
  panelClassName,
  open: openProp,
  onOpenChange,
  initialFocus,
  variant = 'button',
  anchor,
}: PopoverProps) {
  const [openInternal, setOpenInternal] = useState(false);
  const open = openProp ?? openInternal;
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const id = useId();
  // Botão do mouse pressionado dentro do componente (até soltar).
  const pressing = useRef(false);
  useEffect(() => {
    const up = () => {
      pressing.current = false;
    };
    window.addEventListener('mouseup', up, true);
    return () => window.removeEventListener('mouseup', up, true);
  }, []);
  const floating = variant === 'bare' || Boolean(anchor);
  const [place, setPlace] = useState<CSSProperties | null>(null);

  // Painel flutuante: posiciona no primeiro quadro (fica invisível até lá) e acompanha
  // rolagem e redimensionamento.
  useLayoutEffect(() => {
    if (!open || !floating) return;
    const update = () => {
      if (panel.current && anchor) setPlace(placeAt(anchor, panel.current));
      else if (button.current && panel.current)
        setPlace(placePanel(button.current, panel.current, align));
    };
    const raf = requestAnimationFrame(update);
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [open, floating, align, anchor]);

  const setOpen = (next: boolean) => {
    if (next) setRefocus(false);
    if (openProp === undefined) setOpenInternal(next);
    onOpenChange?.(next);
  };
  // Devolver o foco ao botão depois de fechar (num efeito: o fechamento pode vir do render).
  const [refocus, setRefocus] = useState(false);
  const close = (andRefocus = true) => {
    setOpen(false);
    if (andRefocus) setRefocus(true);
  };
  useEffect(() => {
    if (!open && refocus) button.current?.focus();
  }, [open, refocus]);

  useEffect(() => {
    if (!open) return;
    const target =
      initialFocus?.current ??
      panel.current?.querySelector<HTMLElement>(
        'input, button, select, textarea, [tabindex]:not([tabindex="-1"])',
      );
    target?.focus();
    // Dentro de uma janela, o escopo de foco do painel só pausa o dela depois do primeiro
    // quadro: foca de novo se ela tiver puxado o foco de volta.
    const again = requestAnimationFrame(() => {
      if (
        target &&
        document.activeElement !== target &&
        !panel.current?.contains(document.activeElement)
      )
        target.focus();
    });
    const onDown = (e: MouseEvent) => {
      const t = e.target as Node;
      if (root.current && !root.current.contains(t) && !panel.current?.contains(t)) {
        if (openProp === undefined) setOpenInternal(false);
        onOpenChange?.(false);
      }
    };
    document.addEventListener('mousedown', onDown);
    return () => {
      cancelAnimationFrame(again);
      document.removeEventListener('mousedown', onDown);
    };
  }, [open, initialFocus, openProp, onOpenChange]);

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === 'Escape' && open) {
      e.stopPropagation();
      e.preventDefault();
      close();
    }
  };

  return (
    <div
      ref={root}
      className={cx('relative inline-flex min-w-0', variant === 'bare' && 'max-w-full', className)}
      onKeyDown={onKeyDown}
      onMouseDownCapture={() => {
        pressing.current = true;
      }}
      onBlur={(e) => {
        const to = e.relatedTarget;
        // No WebKit (app no macOS) clicar num botão não dá foco a ele: o foco sobe para o
        // ancestral focável mais próximo (o <main tabIndex=-1>), fora do painel. Sair pelo
        // Tab fecha; clique dentro do painel, não.
        if (pressing.current) return;
        if (
          open &&
          root.current &&
          to &&
          !root.current.contains(to) &&
          !panel.current?.contains(to) &&
          !to.contains(root.current)
        ) {
          close(false);
        }
      }}
    >
      <button
        ref={button}
        type="button"
        aria-expanded={open}
        aria-controls={open ? id : undefined}
        aria-label={ariaLabel}
        onClick={() => setOpen(!open)}
        className={cx(
          variant === 'bare'
            ? 'inline-flex h-6 max-w-full min-w-0 items-center gap-1 rounded-control border border-transparent px-1 text-[12.5px] whitespace-nowrap text-fg transition-colors hover:border-line-strong'
            : 'inline-flex h-8 items-center gap-1.5 rounded-control border px-2.5 text-[12.5px] whitespace-nowrap transition-colors',
          variant === 'button' &&
            (active
              ? 'border-accent/70 bg-accent-soft text-accent'
              : 'border-line-strong bg-surface-2 text-fg hover:border-accent'),
        )}
      >
        {label}
      </button>
      {open
        ? (() => {
            const body = (
              <div
                ref={panel}
                id={id}
                role="dialog"
                aria-label={ariaLabel ?? (typeof label === 'string' ? label : undefined)}
                className={cx(
                  'z-[60] min-w-56 animate-pop-in rounded-card border border-panel pop-surface p-2',
                  floating
                    ? 'flex flex-col overflow-auto'
                    : cx('absolute top-[calc(100%+4px)]', align === 'end' ? 'right-0' : 'left-0'),
                  panelClassName,
                )}
                // Antes de medir, transparente no canto (evita piscar fora do lugar, mas já
                // recebe o foco inicial).
                // `pointerEvents`: uma janela modal aberta desliga o mouse no body; sem isto o
                // clique atravessaria o painel e cairia no fundo (o seletor fechava).
                style={
                  floating
                    ? {
                        pointerEvents: 'auto',
                        ...(place ?? { position: 'fixed', left: 0, top: 0, opacity: 0 }),
                      }
                    : undefined
                }
              >
                {typeof children === 'function' ? children(() => close()) : children}
              </div>
            );
            // O painel flutuante vai para o body, fora de uma janela (diálogo) que prenda o
            // foco: o escopo próprio pausa o da janela, que senão puxaria o foco de volta
            // e o seletor fecharia (WebKit) ou não aceitaria digitação.
            return floating
              ? createPortal(
                  <FocusScope
                    asChild
                    onMountAutoFocus={(e) => e.preventDefault()}
                    onUnmountAutoFocus={(e) => e.preventDefault()}
                  >
                    {body}
                  </FocusScope>,
                  document.body,
                )
              : body;
          })()
        : null}
    </div>
  );
}
