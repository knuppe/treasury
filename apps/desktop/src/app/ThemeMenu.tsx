import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { createPortal } from 'react-dom';

import { Icon, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { THEME_FAMILIES, THEMES, themeIcon, themeLabel, type ThemeId } from '@/design/themes';
import { useUi } from '@/state/ui';
import { t } from '@/i18n';

/**
 * Seletor de tema (barra superior e telas do cofre): abre um menu com todos os temas,
 * agrupados (clássicos, especiais, divertidos). Setas ↑↓ percorrem, Enter escolhe,
 * Esc fecha. A descrição aparece cortada na linha; a dica de cada tema mostra o texto inteiro.
 * O menu flutua no `body` (posição fixa junto ao botão), acima dos cards: temas com
 * vidro/desfoque criam camadas próprias que cobririam um menu dentro da barra.
 */
export function ThemeMenu() {
  const theme = useUi((s) => s.theme);
  const setTheme = useUi((s) => s.setTheme);
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const items = useRef<(HTMLButtonElement | null)[]>([]);
  const menu = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; right: number } | null>(null);

  useLayoutEffect(() => {
    if (!open) return;
    const place = () => {
      const r = button.current?.getBoundingClientRect();
      if (r) setPos({ top: r.bottom + 6, right: Math.max(8, window.innerWidth - r.right) });
    };
    place();
    window.addEventListener('resize', place);
    return () => window.removeEventListener('resize', place);
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const current = THEMES.findIndex((t) => t.id === theme);
    items.current[Math.max(0, current)]?.focus();
    const onDown = (e: MouseEvent) => {
      const t = e.target as Node;
      if (root.current && !root.current.contains(t) && !menu.current?.contains(t)) setOpen(false);
    };
    document.addEventListener('mousedown', onDown);
    return () => document.removeEventListener('mousedown', onDown);
    // Só ao abrir: trocar de tema com o menu aberto não move o foco.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const close = () => {
    setOpen(false);
    button.current?.focus();
  };

  const choose = (id: ThemeId) => {
    setTheme(id);
    close();
  };

  const onMenuKey = (e: KeyboardEvent<HTMLDivElement>) => {
    const list = items.current.filter((el): el is HTMLButtonElement => el !== null);
    const at = list.indexOf(document.activeElement as HTMLButtonElement);
    const move = (to: number) => {
      e.preventDefault();
      list[(to + list.length) % list.length]?.focus();
    };
    if (e.key === 'ArrowDown') move(at + 1);
    else if (e.key === 'ArrowUp') move(at - 1);
    else if (e.key === 'Home') move(0);
    else if (e.key === 'End') move(list.length - 1);
    else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      close();
    } else if (e.key === 'Tab') setOpen(false);
  };

  return (
    <div ref={root} className="relative">
      <Tooltip content={`Tema: ${themeLabel(theme)}`}>
        <button
          ref={button}
          type="button"
          aria-label={t('casca-tema-escolher', { tema: themeLabel(theme) })}
          aria-haspopup="menu"
          aria-expanded={open}
          onClick={() => setOpen((o) => !o)}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown' && !open) {
              e.preventDefault();
              setOpen(true);
            }
          }}
          className={cx(
            'inline-flex size-8 items-center justify-center rounded-control text-dim transition-colors duration-150 hover:bg-surface-2 hover:text-fg',
            open && 'bg-accent-soft text-accent hover:bg-accent-soft hover:text-accent',
          )}
        >
          <Icon name={themeIcon(theme)} />
        </button>
      </Tooltip>
      {open
        ? createPortal(
            <div
              ref={menu}
              role="menu"
              aria-label={t('casca-temas')}
              onKeyDown={onMenuKey}
              style={
                pos
                  ? { position: 'fixed', top: pos.top, right: pos.right }
                  : { position: 'fixed', top: 0, right: 0, opacity: 0 }
              }
              className="z-[85] max-h-[calc(100vh-64px)] w-64 animate-pop-in overflow-y-auto rounded-card border border-panel pop-surface p-1.5"
            >
              {THEME_FAMILIES.map((family) => (
                <div key={family.id} role="group" aria-label={family.label} className="py-1">
                  <p className="label-caps px-2 pt-1 pb-1 text-[10px]" aria-hidden>
                    {family.label}
                  </p>
                  {THEMES.filter((t) => t.family === family.id).map((t) => {
                    const i = THEMES.indexOf(t);
                    const checked = t.id === theme;
                    return (
                      <Tooltip
                        key={t.id}
                        side="left"
                        className="z-[90]"
                        content={
                          <span className="flex flex-col gap-0.5">
                            <span className="font-medium">{t.label}</span>
                            <span className="text-dim">{t.description}</span>
                          </span>
                        }
                      >
                        <button
                          ref={(el) => {
                            items.current[i] = el;
                          }}
                          type="button"
                          role="menuitemradio"
                          aria-checked={checked}
                          tabIndex={-1}
                          onClick={() => choose(t.id)}
                          className={cx(
                            'flex w-full items-center gap-2.5 rounded-control px-2 py-1.5 text-left outline-none focus-visible:bg-accent-soft hover:bg-surface-2',
                            checked && 'text-accent',
                          )}
                        >
                          <span
                            aria-hidden
                            data-theme={t.id}
                            className="flex size-5 shrink-0 items-center justify-center rounded-[4px] border border-line-strong bg-canvas"
                          >
                            <span className="size-2 rounded-full bg-accent" />
                          </span>
                          <span className="flex min-w-0 flex-1 flex-col">
                            <span className="text-[13px] text-fg">{t.label}</span>
                            <span className="truncate text-[11.5px] text-dim">{t.description}</span>
                          </span>
                          {checked ? <Icon name="check" size={14} /> : null}
                        </button>
                      </Tooltip>
                    );
                  })}
                </div>
              ))}
            </div>,
            document.body,
          )
        : null}
    </div>
  );
}
